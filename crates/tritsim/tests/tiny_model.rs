use std::path::PathBuf;
use trit_core::tritfmt::TritWriter;
use tritsim::model::{KvCache, Model};

/// Deterministic pseudo-random floats without rand: xorshift on a fixed seed.
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
            .map(|_| match (self.next_f32() * 3.0).abs() as u32 {
                0 => 0i8,
                1 => 1,
                _ => -1,
            })
            .collect()
    }
}

const CFG: &str = r#"{"hidden_size":16,"intermediate_size":32,"num_hidden_layers":2,
  "num_attention_heads":4,"num_key_value_heads":2,"vocab_size":32,
  "rope_theta":10000.0,"rms_norm_eps":1e-5,"hidden_act":"relu2"}"#;

fn build_tiny_model() -> PathBuf {
    let path = std::env::temp_dir().join("tritsim_tiny.trit");
    let mut rng = Rng(0x5eed_2026);
    let mut w = TritWriter::create(&path, CFG).unwrap();
    let (h, ff, kvh) = (16usize, 32usize, 8usize); // kvh = num_kv_heads * head_dim = 2*4
    w.write_f32("model.embed_tokens.weight", &[32, h], &rng.vec(32 * h)).unwrap();
    for i in 0..2 {
        let p = format!("model.layers.{i}.");
        let ones = vec![1.0f32; h];
        w.write_f32(&format!("{p}input_layernorm.weight"), &[h], &ones).unwrap();
        w.write_trit(&format!("{p}self_attn.q_proj.weight"), &[h, h], &rng.trits(h * h), 0.1).unwrap();
        w.write_trit(&format!("{p}self_attn.k_proj.weight"), &[kvh, h], &rng.trits(kvh * h), 0.1).unwrap();
        w.write_trit(&format!("{p}self_attn.v_proj.weight"), &[kvh, h], &rng.trits(kvh * h), 0.1).unwrap();
        w.write_trit(&format!("{p}self_attn.o_proj.weight"), &[h, h], &rng.trits(h * h), 0.1).unwrap();
        w.write_f32(&format!("{p}post_attention_layernorm.weight"), &[h], &ones).unwrap();
        w.write_trit(&format!("{p}mlp.gate_proj.weight"), &[ff, h], &rng.trits(ff * h), 0.1).unwrap();
        w.write_trit(&format!("{p}mlp.up_proj.weight"), &[ff, h], &rng.trits(ff * h), 0.1).unwrap();
        w.write_trit(&format!("{p}mlp.down_proj.weight"), &[h, ff], &rng.trits(h * ff), 0.1).unwrap();
    }
    w.write_f32("model.norm.weight", &[16], &vec![1.0; 16]).unwrap();
    w.finish().unwrap();
    path
}

#[test]
fn forward_is_deterministic_and_finite() {
    let path = build_tiny_model();
    let model = Model::load(&path).unwrap();
    let run = || {
        let mut cache = KvCache::new(&model.cfg);
        let mut out = Vec::new();
        for (pos, tok) in [1u32, 5, 9, 2].iter().enumerate() {
            out.push(model.forward(*tok, pos, &mut cache));
        }
        out
    };
    let (a, b) = (run(), run());
    assert_eq!(a, b, "two identical runs must produce identical logits");
    assert_eq!(a[0].len(), 32);
    assert!(a.iter().flatten().all(|v| v.is_finite()));
}

#[test]
fn causality_past_logits_unaffected_by_future_tokens() {
    let path = build_tiny_model();
    let model = Model::load(&path).unwrap();
    let logits_at = |tokens: &[u32], upto: usize| {
        let mut cache = KvCache::new(&model.cfg);
        let mut last = Vec::new();
        for (pos, tok) in tokens.iter().take(upto + 1).enumerate() {
            last = model.forward(*tok, pos, &mut cache);
        }
        last
    };
    // position-1 logits must be identical whatever comes at position 2+
    let a = logits_at(&[1, 5, 9, 2], 1);
    let b = logits_at(&[1, 5, 30, 30], 1);
    assert_eq!(a, b);
}

#[test]
fn greedy_argmax_is_stable() {
    let path = build_tiny_model();
    let model = Model::load(&path).unwrap();
    let mut cache = KvCache::new(&model.cfg);
    let logits = model.forward(3, 0, &mut cache);
    let argmax = |l: &[f32]| l.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)).unwrap().0;
    let t1 = argmax(&logits);
    let mut cache2 = KvCache::new(&model.cfg);
    assert_eq!(argmax(&model.forward(3, 0, &mut cache2)), t1);
}
