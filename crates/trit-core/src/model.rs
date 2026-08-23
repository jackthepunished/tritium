//! The production transformer.
//!
//! Weights are never unpacked. A `BitLinear` holds a `TritSpan` -- a `Copy`
//! offset into the memory-mapped `.trit` -- and the kernel reads the bit planes
//! in place. Dense tensors are borrowed from the same mapping. The consequence
//! is that loading a 2B model allocates almost nothing beyond the KV cache, and
//! the resident set is the page cache the OS was going to hold anyway.
//!
//! `Model` carries no lifetime parameter. It holds `Arc<TritFile>` and resolves
//! spans against it, rather than borrowing from it, because a `Model<'a>` would
//! push that lifetime into the session, the sampler, and ultimately the C FFI,
//! where the handle has to be `'static`.

use std::sync::Arc;

use anyhow::{Context, Result};

use crate::backend::MatvecBackend;
use crate::config::{Act, ModelConfig, Numerics};
use crate::kv::KvCache;
use crate::math;
use crate::planes::LANES;
use crate::rope::RopeTable;
use crate::tritfmt::{DenseSpan, TritFile, TritSpan};

/// A ternary projection: a span, not data.
#[derive(Clone, Copy, Debug)]
pub struct BitLinear {
    span: TritSpan,
}

impl BitLinear {
    pub fn rows(&self) -> usize {
        self.span.rows()
    }
    pub fn cols(&self) -> usize {
        self.span.cols()
    }
    pub fn scale(&self) -> f32 {
        self.span.scale()
    }
    /// Columns rounded up to a whole beat -- the length activation buffers need.
    pub fn padded_cols(&self) -> usize {
        self.span.cols().div_ceil(LANES) * LANES
    }
    pub fn bytes(&self) -> usize {
        self.span.bytes()
    }
}

pub struct Layer {
    pub input_norm: Vec<f32>,
    pub q: BitLinear,
    pub k: BitLinear,
    pub v: BitLinear,
    pub o: BitLinear,
    pub attn_sub_norm: Option<Vec<f32>>,
    pub post_norm: Vec<f32>,
    pub gate: BitLinear,
    pub up: BitLinear,
    pub down: BitLinear,
    pub ffn_sub_norm: Option<Vec<f32>>,
}

pub struct Model {
    file: Arc<TritFile>,
    cfg: ModelConfig,
    layers: Vec<Layer>,
    final_norm: Vec<f32>,
    embed: DenseSpan,
    /// Aliases `embed` when the checkpoint ties them -- the same span value, not
    /// a copy. The reference implementation clones the embedding table here,
    /// which costs 1.3 GB on this model for no benefit.
    lm_head: DenseSpan,
    tied: bool,
    backend: Arc<dyn MatvecBackend>,
    numerics: Numerics,
}

/// Summary rather than contents: a model wraps a mapping that is routinely
/// gigabytes.
impl std::fmt::Debug for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Model")
            .field("layers", &self.layers.len())
            .field("hidden_size", &self.cfg.hidden_size)
            .field("vocab_size", &self.cfg.vocab_size)
            .field("numerics", &self.numerics)
            .field("tied_embeddings", &self.tied)
            .field("backend", &self.backend.name())
            .field("weight_bytes", &self.weight_bytes())
            .finish()
    }
}

impl Model {
    pub fn load(file: Arc<TritFile>, backend: Arc<dyn MatvecBackend>) -> Result<Arc<Self>> {
        let cfg = ModelConfig::from_json(file.config_json())?;

        // Shapes are checked here, at the trust boundary, rather than assumed.
        // Every consumer downstream indexes on the config's dimensions, so a
        // tensor that disagrees with the embedded config.json surfaces as a
        // panic deep in the forward pass -- or, worse, as a silently wrong
        // result. A `.trit` is a self-describing container that anything may
        // have written; the loader is the only place that can reject it with a
        // message naming what was wrong.
        let bl = |name: &str, rows: usize, cols: usize| -> Result<BitLinear> {
            let span = file
                .trit_span(name)
                .with_context(|| format!("missing {name}"))?;
            anyhow::ensure!(
                span.rows() == rows && span.cols() == cols,
                "{name} is {}x{}, but config.json implies {rows}x{cols}",
                span.rows(),
                span.cols()
            );
            Ok(BitLinear { span })
        };
        let dense = |name: &str, elems: usize| -> Result<Vec<f32>> {
            let s = file
                .dense_span(name)
                .with_context(|| format!("missing {name}"))?;
            anyhow::ensure!(
                s.elems() == elems,
                "{name} holds {} elements, but config.json implies {elems}",
                s.elems()
            );
            Ok(file.dense(s).into_owned())
        };
        // Absent is allowed; present-but-malformed is not. Presence is decided
        // by `has`, not by whether `dense_span` succeeded: that call also fails
        // when the tensor exists with the wrong dtype, so treating any error as
        // "absent" would read a ternary sub-norm as a missing one -- picking a
        // different numerics rung without saying so.
        let dense_opt = |name: &str, elems: usize| -> Result<Option<Vec<f32>>> {
            if file.has(name) {
                dense(name, elems).map(Some)
            } else {
                Ok(None)
            }
        };

        let (h, kv) = (cfg.hidden_size, cfg.num_kv_heads * cfg.head_dim());
        let ffn = cfg.intermediate_size;

        let mut layers = Vec::with_capacity(cfg.num_layers);
        for i in 0..cfg.num_layers {
            let p = format!("model.layers.{i}.");
            layers.push(Layer {
                input_norm: dense(&format!("{p}input_layernorm.weight"), h)?,
                q: bl(&format!("{p}self_attn.q_proj.weight"), h, h)?,
                k: bl(&format!("{p}self_attn.k_proj.weight"), kv, h)?,
                v: bl(&format!("{p}self_attn.v_proj.weight"), kv, h)?,
                o: bl(&format!("{p}self_attn.o_proj.weight"), h, h)?,
                attn_sub_norm: dense_opt(&format!("{p}self_attn.attn_sub_norm.weight"), h)?,
                post_norm: dense(&format!("{p}post_attention_layernorm.weight"), h)?,
                gate: bl(&format!("{p}mlp.gate_proj.weight"), ffn, h)?,
                up: bl(&format!("{p}mlp.up_proj.weight"), ffn, h)?,
                down: bl(&format!("{p}mlp.down_proj.weight"), h, ffn)?,
                ffn_sub_norm: dense_opt(&format!("{p}mlp.ffn_sub_norm.weight"), ffn)?,
            });
        }

        let vocab_elems = cfg.vocab_size * cfg.hidden_size;
        let embed = file
            .dense_span("model.embed_tokens.weight")
            .context("missing model.embed_tokens.weight")?;
        anyhow::ensure!(
            embed.elems() == vocab_elems,
            "model.embed_tokens.weight holds {} elements, but config.json implies \
             vocab_size * hidden_size = {vocab_elems}",
            embed.elems()
        );
        // Prefer an explicit head; fall back to the tied embedding only when the
        // config says the weights are tied, so a genuinely untied checkpoint
        // missing its head is an error rather than silently wrong output.
        // Prefer an explicit head; fall back to the tied embedding only when the
        // config says the weights are tied, so a genuinely untied checkpoint
        // missing its head is an error rather than silently wrong output. A head
        // that is present but malformed is an error either way -- the fallback
        // is for absence, not for repair.
        let (lm_head, tied) = if file.has("lm_head.weight") {
            let s = file.dense_span("lm_head.weight")?;
            anyhow::ensure!(
                s.elems() == vocab_elems,
                "lm_head.weight holds {} elements, but config.json implies {vocab_elems}",
                s.elems()
            );
            (s, false)
        } else {
            anyhow::ensure!(
                cfg.tie_word_embeddings,
                "lm_head.weight is absent and config does not set tie_word_embeddings"
            );
            (embed, true)
        };

        let has_ffn_sub_norm = layers.iter().all(|l| l.ffn_sub_norm.is_some());
        let numerics = Numerics::best_for(&cfg, has_ffn_sub_norm);

        Ok(Arc::new(Self {
            final_norm: dense("model.norm.weight", h)?,
            file,
            cfg,
            layers,
            embed,
            lm_head,
            tied,
            backend,
            numerics,
        }))
    }

    pub fn config(&self) -> &ModelConfig {
        &self.cfg
    }
    pub fn numerics(&self) -> Numerics {
        self.numerics
    }
    pub fn backend(&self) -> &dyn MatvecBackend {
        self.backend.as_ref()
    }
    /// Share this model's backend with another model instance.
    pub fn backend_arc(&self) -> Arc<dyn MatvecBackend> {
        self.backend.clone()
    }
    pub fn tied_embeddings(&self) -> bool {
        self.tied
    }

    /// Select a numerics rung, rejecting ones this architecture cannot support.
    pub fn set_numerics(&mut self, n: Numerics) -> Result<()> {
        if n.int_mlp() {
            anyhow::ensure!(
                self.cfg.act == Act::Relu2,
                "the integer MLP path is relu2-specific"
            );
            anyhow::ensure!(
                self.layers.iter().all(|l| l.ffn_sub_norm.is_some()),
                "the integer MLP path requires per-layer ffn_sub_norm"
            );
        }
        self.numerics = n;
        Ok(())
    }

    /// Bytes of weights touched per decoded token: every ternary plane plus the
    /// LM head. This is the numerator of the bandwidth roofline.
    pub fn weight_bytes(&self) -> u64 {
        let tern: usize = self
            .layers
            .iter()
            .map(|l| {
                l.q.bytes()
                    + l.k.bytes()
                    + l.v.bytes()
                    + l.o.bytes()
                    + l.gate.bytes()
                    + l.up.bytes()
                    + l.down.bytes()
            })
            .sum();
        tern as u64 + self.lm_head.bytes() as u64
    }

    pub fn ternary_bytes(&self) -> u64 {
        self.layers
            .iter()
            .map(|l| {
                (l.q.bytes()
                    + l.k.bytes()
                    + l.v.bytes()
                    + l.o.bytes()
                    + l.gate.bytes()
                    + l.up.bytes()
                    + l.down.bytes()) as u64
            })
            .sum()
    }

    pub fn lm_head_bytes(&self) -> u64 {
        self.lm_head.bytes() as u64
    }

    /// `acc = W . codes`, raw integer accumulators.
    fn acc_into(&self, w: &BitLinear, codes: &[i8], out: &mut [i32]) {
        let planes = self.file.planes(w.span);
        self.backend.matvec(&planes, codes, &mut out[..w.rows()]);
    }

    /// One decode step. Writes `logits` (length `vocab_size`).
    pub fn forward_into(
        &self,
        token: u32,
        pos: usize,
        cache: &mut KvCache,
        s: &mut Scratch,
        logits: &mut [f32],
    ) -> Result<()> {
        let cfg = &self.cfg;
        let (hd, nh, nkv) = (cfg.head_dim(), cfg.num_heads, cfg.num_kv_heads);
        let h = cfg.hidden_size;
        anyhow::ensure!(
            (token as usize) < cfg.vocab_size,
            "token {token} out of vocab"
        );
        anyhow::ensure!(
            pos < cfg.max_seq,
            "pos {pos} exceeds max_seq {}",
            cfg.max_seq
        );
        anyhow::ensure!(
            logits.len() == cfg.vocab_size,
            "logits buffer must be vocab_size"
        );

        let (folded, int_mlp) = (self.numerics.folded(), self.numerics.int_mlp());

        // Embedding lookup: one row out of the mapping, not the whole table.
        let row = self.file.dense_row(self.embed, token as usize, h)?;
        s.x.clear();
        s.x.extend_from_slice(&row);

        s.rope.seek(pos);

        for (li, layer) in self.layers.iter().enumerate() {
            // ---- attention ----
            let x_scale = norm_quant(
                &s.x,
                &layer.input_norm,
                cfg.rms_eps,
                folded,
                &mut s.mul,
                &mut s.normed,
                &mut s.codes,
            );

            self.acc_into(&layer.q, &s.codes, &mut s.acc);
            scale_into(&s.acc[..layer.q.rows()], layer.q.scale(), x_scale, &mut s.q);
            self.acc_into(&layer.k, &s.codes, &mut s.acc);
            scale_into(&s.acc[..layer.k.rows()], layer.k.scale(), x_scale, &mut s.k);
            self.acc_into(&layer.v, &s.codes, &mut s.acc);
            scale_into(&s.acc[..layer.v.rows()], layer.v.scale(), x_scale, &mut s.v);

            s.rope.apply(&mut s.q);
            s.rope.apply(&mut s.k);
            cache.write(li, pos, &s.k, &s.v);

            let (kc, vc) = cache.read(li, pos);
            s.ctx.clear();
            s.ctx.resize(nh * hd, 0.0);
            let attn_scale = 1.0 / (hd as f32).sqrt();
            for head in 0..nh {
                let kvh = head * nkv / nh;
                let qh = &s.q[head * hd..(head + 1) * hd];
                s.scores.clear();
                s.scores.extend((0..=pos).map(|t| {
                    let base = t * nkv * hd + kvh * hd;
                    qh.iter()
                        .zip(&kc[base..base + hd])
                        .map(|(a, b)| a * b)
                        .sum::<f32>()
                        * attn_scale
                }));
                math::softmax_inplace(&mut s.scores);
                let out = &mut s.ctx[head * hd..(head + 1) * hd];
                for (t, sc) in s.scores.iter().enumerate() {
                    let base = t * nkv * hd + kvh * hd;
                    for (o, vv) in out.iter_mut().zip(&vc[base..base + hd]) {
                        *o += sc * vv;
                    }
                }
            }

            let o_scale = match &layer.attn_sub_norm {
                Some(g) => norm_quant(
                    &s.ctx,
                    g,
                    cfg.rms_eps,
                    folded,
                    &mut s.mul,
                    &mut s.normed,
                    &mut s.codes2,
                ),
                None => math::absmax_codes_into(&s.ctx, &mut s.codes2),
            };
            self.acc_into(&layer.o, &s.codes2, &mut s.acc);
            let ow = layer.o.scale();
            for (i, a) in s.acc[..layer.o.rows()].iter().enumerate() {
                s.x[i] += *a as f32 * ow * o_scale;
            }

            // ---- mlp ----
            let x_scale = norm_quant(
                &s.x,
                &layer.post_norm,
                cfg.rms_eps,
                folded,
                &mut s.mul,
                &mut s.normed,
                &mut s.codes,
            );

            let down_scale = if int_mlp {
                let gain = layer
                    .ffn_sub_norm
                    .as_ref()
                    .expect("set_numerics guarantees ffn_sub_norm on the IntMlp rung");
                self.acc_into(&layer.gate, &s.codes, &mut s.acc);
                s.acc_g.clear();
                s.acc_g.extend_from_slice(&s.acc[..layer.gate.rows()]);
                self.acc_into(&layer.up, &s.codes, &mut s.acc);
                math::int_mlp_codes_into(
                    &s.acc_g,
                    &s.acc[..layer.up.rows()],
                    gain,
                    layer.gate.scale() as f64 * x_scale as f64,
                    layer.up.scale() as f64 * x_scale as f64,
                    cfg.rms_eps,
                    &mut s.t_buf,
                    &mut s.z_buf,
                    &mut s.codes2,
                )
            } else {
                self.acc_into(&layer.gate, &s.codes, &mut s.acc);
                let gw = layer.gate.scale();
                s.gate_f.clear();
                s.gate_f.extend(
                    s.acc[..layer.gate.rows()]
                        .iter()
                        .map(|a| *a as f32 * gw * x_scale),
                );
                self.acc_into(&layer.up, &s.codes, &mut s.acc);
                let uw = layer.up.scale();
                s.act.clear();
                s.act.extend(
                    s.gate_f
                        .iter()
                        .zip(&s.acc[..layer.up.rows()])
                        .map(|(g, u)| math::activate(cfg.act, *g) * (*u as f32 * uw * x_scale)),
                );
                match &layer.ffn_sub_norm {
                    Some(g) => norm_quant(
                        &s.act,
                        g,
                        cfg.rms_eps,
                        folded,
                        &mut s.mul,
                        &mut s.normed,
                        &mut s.codes2,
                    ),
                    None => math::absmax_codes_into(&s.act, &mut s.codes2),
                }
            };

            self.acc_into(&layer.down, &s.codes2, &mut s.acc);
            let dw = layer.down.scale();
            for (i, a) in s.acc[..layer.down.rows()].iter().enumerate() {
                s.x[i] += *a as f32 * dw * down_scale;
            }
        }

        // ---- logits ----
        math::rmsnorm_into(&s.x, &self.final_norm, cfg.rms_eps, &mut s.normed[..h]);
        let head = self.file.dense(self.lm_head);
        self.backend
            .f32_matvec(&head, cfg.vocab_size, h, &s.normed[..h], logits);
        Ok(())
    }
}

/// Normalize `x` by `gain` and quantize to int8 codes, returning the effective
/// activation scale.
///
/// `normed` and `codes` are reusable scratch that may be LONGER than `x`; the
/// working length always comes from `x`, never from the buffer's capacity.
/// Getting that wrong quantizes stale bytes from a previous, wider layer.
#[allow(clippy::too_many_arguments)]
fn norm_quant(
    x: &[f32],
    gain: &[f32],
    eps: f32,
    folded: bool,
    mul: &mut Vec<f32>,
    normed: &mut [f32],
    codes: &mut [i8],
) -> f32 {
    debug_assert!(normed.len() >= x.len() && codes.len() >= x.len());
    if folded {
        let (sc, r) = math::scaled_absmax_codes_into(x, gain, eps, mul, codes);
        sc * r
    } else {
        math::rmsnorm_into(x, gain, eps, &mut normed[..x.len()]);
        math::absmax_codes_into(&normed[..x.len()], codes)
    }
}

/// `out[i] = acc[i] * w_scale * x_scale`, resizing `out` to match.
///
/// The two scales are applied as separate multiplications, in this order, and
/// deliberately NOT pre-multiplied into one. f32 multiplication is not
/// associative: `(acc * w) * x` and `acc * (w * x)` round differently, and the
/// second form drifts from the reference implementation the model's numerics
/// were validated against. Folding them would save one multiply per element
/// against a matvec that already cost thousands -- and it silently changed the
/// generated text at the first near-tie.
#[inline]
fn scale_into(acc: &[i32], w_scale: f32, x_scale: f32, out: &mut Vec<f32>) {
    out.clear();
    out.extend(acc.iter().map(|a| *a as f32 * w_scale * x_scale));
}

/// Every buffer a decode step needs, allocated once.
///
/// Activation code buffers are sized to the *padded* width so the kernel always
/// consumes whole 64-column beats and never needs a scalar tail.
pub struct Scratch {
    x: Vec<f32>,
    normed: Vec<f32>,
    mul: Vec<f32>,
    codes: Vec<i8>,
    codes2: Vec<i8>,
    acc: Vec<i32>,
    acc_g: Vec<i32>,
    q: Vec<f32>,
    k: Vec<f32>,
    v: Vec<f32>,
    ctx: Vec<f32>,
    scores: Vec<f32>,
    gate_f: Vec<f32>,
    act: Vec<f32>,
    t_buf: Vec<i64>,
    z_buf: Vec<f64>,
    rope: RopeTable,
}

impl Scratch {
    pub fn new(cfg: &ModelConfig) -> Self {
        let h = cfg.hidden_size;
        let ff = cfg.intermediate_size;
        let pad = |n: usize| n.div_ceil(LANES) * LANES;
        let widest = pad(h.max(ff));
        let rows = h.max(ff).max(cfg.num_kv_heads * cfg.head_dim());
        Self {
            x: Vec::with_capacity(h),
            normed: vec![0.0; h.max(ff)],
            mul: Vec::with_capacity(h.max(ff)),
            codes: vec![0; widest],
            codes2: vec![0; widest],
            acc: vec![0; rows],
            acc_g: Vec::with_capacity(ff),
            q: Vec::with_capacity(h),
            k: Vec::with_capacity(cfg.num_kv_heads * cfg.head_dim()),
            v: Vec::with_capacity(cfg.num_kv_heads * cfg.head_dim()),
            ctx: Vec::with_capacity(h),
            scores: Vec::with_capacity(cfg.max_seq),
            gate_f: Vec::with_capacity(ff),
            act: Vec::with_capacity(ff),
            t_buf: Vec::with_capacity(ff),
            z_buf: Vec::with_capacity(ff),
            rope: RopeTable::new(cfg.head_dim(), cfg.rope_theta),
        }
    }
}
