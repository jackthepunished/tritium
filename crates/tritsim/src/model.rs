use crate::config::ModelConfig;
use crate::math::{activate, rmsnorm, rope_inplace, softmax_inplace};
use anyhow::{Context, Result};
use std::path::Path;
use trit_core::matvec::ternary_matvec;
use trit_core::quant::absmax_quantize;
use trit_core::tritfmt::TritReader;

pub struct BitLinear {
    pub trits: Vec<i8>,
    pub rows: usize,
    pub cols: usize,
    pub w_scale: f32,
}

impl BitLinear {
    pub fn apply(&self, x: &[f32]) -> Vec<f32> {
        let (xq, x_scale) = absmax_quantize(x);
        ternary_matvec(&self.trits, self.rows, self.cols, &xq)
            .into_iter()
            .map(|acc| acc as f32 * self.w_scale * x_scale)
            .collect()
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
    pub cfg: ModelConfig,
    pub embed: Vec<f32>, // vocab x hidden
    pub layers: Vec<Layer>,
    pub final_norm: Vec<f32>,
    pub lm_head: Vec<f32>, // vocab x hidden
}

pub struct KvCache {
    // per layer: [max_seq * num_kv_heads * head_dim]
    pub k: Vec<Vec<f32>>,
    pub v: Vec<Vec<f32>>,
    pub len: usize,
}

impl KvCache {
    pub fn new(cfg: &ModelConfig) -> Self {
        let per = cfg.max_seq * cfg.num_kv_heads * cfg.head_dim();
        Self {
            k: (0..cfg.num_layers).map(|_| vec![0.0; per]).collect(),
            v: (0..cfg.num_layers).map(|_| vec![0.0; per]).collect(),
            len: 0,
        }
    }
}

impl Model {
    pub fn load(path: &Path) -> Result<Self> {
        let r = TritReader::open(path)?;
        let cfg = ModelConfig::from_json(r.config_json())?;
        let bl = |name: &str| -> Result<BitLinear> {
            let m = r
                .metas()
                .iter()
                .find(|m| m.name == name)
                .with_context(|| format!("missing {name}"))?;
            let (rows, cols) = (m.shape[0], m.shape[1]);
            let (trits, w_scale) = r.read_trit(name)?;
            Ok(BitLinear { trits, rows, cols, w_scale })
        };
        let f32_opt = |name: &str| r.read_f32(name).ok();

        let mut layers = Vec::with_capacity(cfg.num_layers);
        for i in 0..cfg.num_layers {
            let p = format!("model.layers.{i}.");
            layers.push(Layer {
                input_norm: r.read_f32(&format!("{p}input_layernorm.weight"))?,
                q: bl(&format!("{p}self_attn.q_proj.weight"))?,
                k: bl(&format!("{p}self_attn.k_proj.weight"))?,
                v: bl(&format!("{p}self_attn.v_proj.weight"))?,
                o: bl(&format!("{p}self_attn.o_proj.weight"))?,
                attn_sub_norm: f32_opt(&format!("{p}self_attn.attn_sub_norm.weight")),
                post_norm: r.read_f32(&format!("{p}post_attention_layernorm.weight"))?,
                gate: bl(&format!("{p}mlp.gate_proj.weight"))?,
                up: bl(&format!("{p}mlp.up_proj.weight"))?,
                down: bl(&format!("{p}mlp.down_proj.weight"))?,
                ffn_sub_norm: f32_opt(&format!("{p}mlp.ffn_sub_norm.weight")),
            });
        }
        let embed = r.read_f32("model.embed_tokens.weight")?;
        let lm_head = r.read_f32("lm_head.weight").unwrap_or_else(|_| embed.clone());
        Ok(Self {
            final_norm: r.read_f32("model.norm.weight")?,
            embed,
            lm_head,
            layers,
            cfg,
        })
    }

    pub fn forward(&self, token: u32, pos: usize, cache: &mut KvCache) -> Vec<f32> {
        let cfg = &self.cfg;
        let (hd, nh, nkv) = (cfg.head_dim(), cfg.num_heads, cfg.num_kv_heads);
        let h = cfg.hidden_size;
        let mut x = self.embed[token as usize * h..(token as usize + 1) * h].to_vec();

        for (li, layer) in self.layers.iter().enumerate() {
            // attention block
            let xn = rmsnorm(&x, &layer.input_norm, cfg.rms_eps);
            let mut q = layer.q.apply(&xn);
            let mut k = layer.k.apply(&xn);
            let v = layer.v.apply(&xn);
            rope_inplace(&mut q, hd, pos, cfg.rope_theta);
            rope_inplace(&mut k, hd, pos, cfg.rope_theta);
            let kc = &mut cache.k[li];
            let vc = &mut cache.v[li];
            kc[pos * nkv * hd..(pos + 1) * nkv * hd].copy_from_slice(&k);
            vc[pos * nkv * hd..(pos + 1) * nkv * hd].copy_from_slice(&v);

            let mut ctx = vec![0.0f32; nh * hd];
            let scale = 1.0 / (hd as f32).sqrt();
            for head in 0..nh {
                let kvh = head * nkv / nh;
                let qh = &q[head * hd..(head + 1) * hd];
                let mut scores: Vec<f32> = (0..=pos)
                    .map(|t| {
                        let kh = &kc[t * nkv * hd + kvh * hd..t * nkv * hd + (kvh + 1) * hd];
                        qh.iter().zip(kh).map(|(a, b)| a * b).sum::<f32>() * scale
                    })
                    .collect();
                softmax_inplace(&mut scores);
                let out = &mut ctx[head * hd..(head + 1) * hd];
                for (t, s) in scores.iter().enumerate() {
                    let vh = &vc[t * nkv * hd + kvh * hd..t * nkv * hd + (kvh + 1) * hd];
                    for d in 0..hd {
                        out[d] += s * vh[d];
                    }
                }
            }
            let ctx = match &layer.attn_sub_norm {
                Some(g) => rmsnorm(&ctx, g, cfg.rms_eps),
                None => ctx,
            };
            let attn_out = layer.o.apply(&ctx);
            for i in 0..h {
                x[i] += attn_out[i];
            }

            // mlp block
            let xn = rmsnorm(&x, &layer.post_norm, cfg.rms_eps);
            let g = layer.gate.apply(&xn);
            let u = layer.up.apply(&xn);
            let a: Vec<f32> = g
                .iter()
                .zip(&u)
                .map(|(gv, uv)| activate(cfg.act, *gv) * uv)
                .collect();
            let a = match &layer.ffn_sub_norm {
                Some(gain) => rmsnorm(&a, gain, cfg.rms_eps),
                None => a,
            };
            let mlp_out = layer.down.apply(&a);
            for i in 0..h {
                x[i] += mlp_out[i];
            }
        }
        cache.len = pos + 1;

        // logits: plain f32 matmul against lm_head (not ternary by design)
        let xn = rmsnorm(&x, &self.final_norm, cfg.rms_eps);
        (0..cfg.vocab_size)
            .map(|v| {
                self.lm_head[v * h..(v + 1) * h]
                    .iter()
                    .zip(&xn)
                    .map(|(a, b)| a * b)
                    .sum()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitlinear_matches_dequantized_float_matmul() {
        // W ternary [[1,-1],[0,1]] scale 0.5; x = [1.0, -2.0]
        // dequant W = [[0.5,-0.5],[0,0.5]]; exact y = [1.5, -1.0]
        // int path: absmax x -> scale 2/127, xq=[64,-127]
        // acc = [64+127, -127] = [191, -127]; y = acc * 0.5 * (2/127) = [1.50394, -1.0]
        let bl = BitLinear { trits: vec![1, -1, 0, 1], rows: 2, cols: 2, w_scale: 0.5 };
        let y = bl.apply(&[1.0, -2.0]);
        assert!((y[0] - 1.5).abs() < 0.01, "y0={}", y[0]);
        assert!((y[1] - -1.0).abs() < 0.01, "y1={}", y[1]);
    }
}
