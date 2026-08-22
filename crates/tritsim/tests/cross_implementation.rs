//! trit-core (production) against tritsim (oracle), on the same file.
//!
//! These are two independent implementations of the same model. tritsim expands
//! the bit planes to one `i8` per weight and walks them with a scalar loop;
//! trit-core reads the planes in place and dispatches to a SIMD kernel, keeps
//! the LM head memory-mapped, and reuses preallocated scratch. They share the
//! container format and the numerics *policy* -- nothing else.
//!
//! That is what makes agreement here informative. The ternary accumulators are
//! integers, so they must match bit-for-bit; the only admissible difference is
//! f32 reassociation in the LM head, where the production path sums with SIMD
//! lanes and the oracle sums sequentially.

use std::path::PathBuf;
use std::sync::Arc;

use trit_core::backend::MatvecBackend;
use trit_core::config::Numerics;
use trit_core::model::{Model as CoreModel, Scratch};
use trit_core::sampler::SamplerParams;
use trit_core::session::Session;
use trit_core::tritfmt::{TritFile, TritWriter};
use tritsim::model::{ForwardMode, KvCache, Model as SimModel};

struct Rng(u64);
impl Rng {
    fn next_f32(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        ((self.0 >> 40) as f32 / (1u64 << 24) as f32) - 0.5
    }
    fn vec(&mut self, n: usize) -> Vec<f32> {
        (0..n).map(|_| self.next_f32()).collect()
    }
    fn trits(&mut self, n: usize) -> Vec<i8> {
        (0..n)
            .map(|_| match ((self.next_f32() + 0.5) * 3.0) as u32 {
                0 => -1i8,
                1 => 0,
                _ => 1,
            })
            .collect()
    }
}

/// Deliberately not a multiple of 64, so the beat-tail path is exercised end to
/// end rather than only in the kernel unit tests.
const CFG: &str = r#"{"hidden_size":80,"intermediate_size":200,"num_hidden_layers":3,
  "num_attention_heads":5,"num_key_value_heads":1,"vocab_size":48,
  "rope_theta":500000.0,"rms_norm_eps":1e-5,"hidden_act":"relu2",
  "tie_word_embeddings":true,"max_position_embeddings":64}"#;

fn build(stem: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("{stem}.trit"));
    let mut rng = Rng(0x0CEA_2026);
    let mut w = TritWriter::create(&path, CFG).unwrap();
    let (h, ff, kvh, vocab) = (80usize, 200usize, 16usize, 48usize);
    w.write_f32(
        "model.embed_tokens.weight",
        &[vocab, h],
        &rng.vec(vocab * h),
    )
    .unwrap();
    for i in 0..3 {
        let p = format!("model.layers.{i}.");
        let ones = vec![1.0f32; h];
        w.write_f32(&format!("{p}input_layernorm.weight"), &[h], &ones)
            .unwrap();
        w.write_trit(
            &format!("{p}self_attn.q_proj.weight"),
            &[h, h],
            &rng.trits(h * h),
            0.1,
        )
        .unwrap();
        w.write_trit(
            &format!("{p}self_attn.k_proj.weight"),
            &[kvh, h],
            &rng.trits(kvh * h),
            0.1,
        )
        .unwrap();
        w.write_trit(
            &format!("{p}self_attn.v_proj.weight"),
            &[kvh, h],
            &rng.trits(kvh * h),
            0.1,
        )
        .unwrap();
        w.write_trit(
            &format!("{p}self_attn.o_proj.weight"),
            &[h, h],
            &rng.trits(h * h),
            0.1,
        )
        .unwrap();
        let gains_h: Vec<f32> = (0..h).map(|j| 0.5 + 0.01 * j as f32).collect();
        w.write_f32(
            &format!("{p}self_attn.attn_sub_norm.weight"),
            &[h],
            &gains_h,
        )
        .unwrap();
        w.write_f32(&format!("{p}post_attention_layernorm.weight"), &[h], &ones)
            .unwrap();
        w.write_trit(
            &format!("{p}mlp.gate_proj.weight"),
            &[ff, h],
            &rng.trits(ff * h),
            0.1,
        )
        .unwrap();
        w.write_trit(
            &format!("{p}mlp.up_proj.weight"),
            &[ff, h],
            &rng.trits(ff * h),
            0.1,
        )
        .unwrap();
        let gains_ff: Vec<f32> = (0..ff).map(|j| 0.3 + 0.005 * j as f32).collect();
        w.write_f32(&format!("{p}mlp.ffn_sub_norm.weight"), &[ff], &gains_ff)
            .unwrap();
        w.write_trit(
            &format!("{p}mlp.down_proj.weight"),
            &[h, ff],
            &rng.trits(h * ff),
            0.1,
        )
        .unwrap();
    }
    w.write_f32("model.norm.weight", &[h], &vec![1.0; h])
        .unwrap();
    w.finish().unwrap();
    path
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let dot: f64 = a.iter().zip(b).map(|(x, y)| *x as f64 * *y as f64).sum();
    let na: f64 = a.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
    let nb: f64 = b.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
    (dot / (na * nb).max(1e-30)) as f32
}

fn argmax(v: &[f32]) -> usize {
    v.iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .map(|(i, _)| i)
        .unwrap()
}

/// Run the same token sequence through both and compare at every position.
fn compare_mode(stem: &str, sim_mode: ForwardMode, core_mode: Numerics, backend_threads: usize) {
    let path = build(stem);

    let sim = SimModel::load(&path).unwrap();
    let mut sim_cache = KvCache::new(&sim.cfg);

    let file = TritFile::open(&path).unwrap();
    let backend: Arc<dyn MatvecBackend> = Arc::new(trit_cpu::CpuBackend::new(backend_threads));
    let mut core = Arc::try_unwrap(CoreModel::load(file, backend).unwrap())
        .ok()
        .unwrap();
    core.set_numerics(core_mode).unwrap();
    let core = Arc::new(core);
    let mut scratch = Scratch::new(core.config());
    let mut kv = trit_core::kv::KvCache::new(core.config());
    let mut logits = vec![0.0f32; core.config().vocab_size];

    let tokens: Vec<u32> = vec![1, 7, 3, 42, 0, 11, 5, 30, 17, 2];
    for (pos, &t) in tokens.iter().enumerate() {
        let a = sim.forward_with_mode(t, pos, &mut sim_cache, sim_mode);
        core.forward_into(t, pos, &mut kv, &mut scratch, &mut logits)
            .unwrap();

        let c = cosine(&a, &logits);
        assert!(
            c > 0.9999,
            "{stem} pos {pos}: cosine {c} between the two implementations"
        );
        assert_eq!(
            argmax(&a),
            argmax(&logits),
            "{stem} pos {pos}: top-1 differs ({:?} vs {:?})",
            argmax(&a),
            argmax(&logits)
        );
        let worst = a
            .iter()
            .zip(&logits)
            .map(|(x, y)| (x - y).abs() / x.abs().max(1.0))
            .fold(0.0f32, f32::max);
        assert!(
            worst < 1e-3,
            "{stem} pos {pos}: worst relative logit difference {worst}"
        );
    }
}

#[test]
fn reference_paths_agree() {
    compare_mode("xcmp_ref", ForwardMode::Reference, Numerics::Reference, 1);
}

#[test]
fn folded_paths_agree() {
    compare_mode("xcmp_folded", ForwardMode::Folded, Numerics::Folded, 1);
}

#[test]
fn integer_mlp_paths_agree() {
    compare_mode("xcmp_int", ForwardMode::IntMlp, Numerics::IntMlp, 1);
}

/// Threading splits rows, never a reduction, so it must not change the answer.
#[test]
fn threading_does_not_change_the_production_logits() {
    compare_mode("xcmp_threads", ForwardMode::IntMlp, Numerics::IntMlp, 8);
}

/// A reused session must produce exactly what a fresh one does: stale KV entries
/// past `len` are never read.
#[test]
fn session_reset_matches_a_fresh_session() {
    let path = build("xcmp_reset");
    let backend: Arc<dyn MatvecBackend> = Arc::new(trit_cpu::CpuBackend::new(1));
    let model = CoreModel::load(TritFile::open(&path).unwrap(), backend).unwrap();

    let prompt = [1u32, 7, 3];
    let mut a = Session::new(model.clone(), &SamplerParams::default());
    a.prefill(&prompt).unwrap();
    let first: Vec<f32> = a.logits().to_vec();

    // Pollute the cache with a different, longer sequence, then reset and repeat.
    a.reset();
    a.prefill(&[42u32, 11, 5, 30, 17]).unwrap();
    a.reset();
    a.prefill(&prompt).unwrap();
    assert_eq!(
        a.logits(),
        &first[..],
        "reset must leave no trace of the previous sequence"
    );
}

/// The production loader must not silently invent an LM head.
#[test]
fn untied_model_missing_its_head_is_an_error() {
    let path = std::env::temp_dir().join("xcmp_untied.trit");
    let mut w = TritWriter::create(
        &path,
        r#"{"hidden_size":8,"intermediate_size":8,"num_hidden_layers":0,
            "num_attention_heads":2,"vocab_size":4,"tie_word_embeddings":false}"#,
    )
    .unwrap();
    w.write_f32("model.embed_tokens.weight", &[4, 8], &[0.1; 32])
        .unwrap();
    w.write_f32("model.norm.weight", &[8], &[1.0; 8]).unwrap();
    w.finish().unwrap();

    let backend: Arc<dyn MatvecBackend> = Arc::new(trit_cpu::CpuBackend::new(1));
    let err = CoreModel::load(TritFile::open(&path).unwrap(), backend).unwrap_err();
    let msg = format!("{err:#}");
    assert!(msg.contains("tie_word_embeddings"), "{msg}");
}

/// Tied embeddings must alias, not copy: the LM head span is the embedding span.
#[test]
fn tied_embeddings_are_aliased_not_cloned() {
    let path = build("xcmp_tied");
    let backend: Arc<dyn MatvecBackend> = Arc::new(trit_cpu::CpuBackend::new(1));
    let model = CoreModel::load(TritFile::open(&path).unwrap(), backend).unwrap();
    assert!(model.tied_embeddings());
    // vocab * hidden * 4 bytes, counted once.
    assert_eq!(model.lm_head_bytes(), 48 * 80 * 4);
}

/// The same cross-check against the real BitNet b1.58 2B4T checkpoint.
///
/// Ignored by default because it needs `models/bitnet-2b4t.trit` (gitignored,
/// 1.8 GB). Run with:
///   cargo test -p tritsim --release --test cross_implementation -- --ignored --nocapture
#[test]
#[ignore = "needs the real checkpoint"]
fn real_checkpoint_paths_agree() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../models/bitnet-2b4t.trit");
    if !path.exists() {
        eprintln!("{} absent; skipping", path.display());
        return;
    }

    let sim = SimModel::load(&path).unwrap();
    let file = TritFile::open(&path).unwrap();
    let threads: usize = std::env::var("XCMP_THREADS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if let Ok(k) = std::env::var("XCMP_KERNEL") {
        trit_cpu::force_kernel(&k).unwrap();
    }
    eprintln!("threads={threads} kernel={}", trit_cpu::kernel_name());
    let backend: Arc<dyn MatvecBackend> = Arc::new(trit_cpu::CpuBackend::new(threads));
    let core_base = CoreModel::load(file, backend).unwrap();

    // A prompt long enough that RoPE and the KV cache are genuinely exercised.
    let tokens: Vec<u32> = vec![128000, 791, 6864, 315, 9822, 374, 12366, 13];

    for (sim_mode, core_mode) in [
        (ForwardMode::Reference, Numerics::Reference),
        (ForwardMode::Folded, Numerics::Folded),
        (ForwardMode::IntMlp, Numerics::IntMlp),
    ] {
        let mut core = CoreModel::load(TritFile::open(&path).unwrap(), core_base.backend_arc())
            .map(|m| Arc::try_unwrap(m).ok().unwrap())
            .unwrap();
        core.set_numerics(core_mode).unwrap();
        let core = Arc::new(core);

        let mut sim_cache = KvCache::new(&sim.cfg);
        let mut kv = trit_core::kv::KvCache::new(core.config());
        let mut scratch = Scratch::new(core.config());
        let mut logits = vec![0.0f32; core.config().vocab_size];

        for (pos, &t) in tokens.iter().enumerate() {
            let a = sim.forward_with_mode(t, pos, &mut sim_cache, sim_mode);
            core.forward_into(t, pos, &mut kv, &mut scratch, &mut logits)
                .unwrap();

            let c = cosine(&a, &logits);
            let (ta, tb) = (argmax(&a), argmax(&logits));
            // The margin between the top two logits: a flip at a near-tie is a
            // different situation from a flip with real separation.
            let mut sorted: Vec<f32> = a.clone();
            sorted.sort_by(|x, y| y.total_cmp(x));
            let margin = sorted[0] - sorted[1];
            println!(
                "{core_mode:?} pos {pos}: cosine {c:.6} top1 sim={ta} core={tb} margin={margin:.4}"
            );
            // 1 - 1e-6: the two implementations are expected to agree to the
            // last bit here. They did not until the scale multiplications were
            // ordered to match the reference exactly -- f32 multiplication is
            // not associative, and folding w_scale * x_scale into one constant
            // was enough to change the generated text at the first near-tie.
            assert!(c > 0.999999, "{core_mode:?} pos {pos}: cosine {c}");
            assert_eq!(
                ta, tb,
                "{core_mode:?} pos {pos}: top-1 differs (margin {margin:.4})"
            );
        }
    }
}
