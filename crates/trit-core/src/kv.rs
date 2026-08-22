//! Key/value cache.
//!
//! One flat `Box<[f32]>` per layer, preallocated to `max_seq`. Flat rather than
//! `Vec<Vec<f32>>` so a layer's pages are contiguous: attention reads the whole
//! history of one head every step, and that is a stride walk either way, but the
//! contiguous form keeps the prefetcher useful and makes the footprint exact.
//!
//! At `max_seq = 2048` on BitNet 2B4T this is
//! `30 layers * 2 * 2048 * 5 kv heads * 128 dims * 4 bytes = 315 MB`, which is
//! the single largest runtime allocation after the model itself. Quantizing it
//! is a known, separate piece of work; `bytes()` exists so a benchmark can say
//! how much of the per-token traffic it accounts for.

use crate::config::ModelConfig;

pub struct KvCache {
    k: Vec<Box<[f32]>>,
    v: Vec<Box<[f32]>>,
    per_pos: usize,
    max_seq: usize,
    len: usize,
}

impl KvCache {
    pub fn new(cfg: &ModelConfig) -> Self {
        let per_pos = cfg.num_kv_heads * cfg.head_dim();
        let per_layer = cfg.max_seq * per_pos;
        Self {
            k: (0..cfg.num_layers).map(|_| vec![0.0; per_layer].into_boxed_slice()).collect(),
            v: (0..cfg.num_layers).map(|_| vec![0.0; per_layer].into_boxed_slice()).collect(),
            per_pos,
            max_seq: cfg.max_seq,
            len: 0,
        }
    }

    /// Positions written so far.
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn max_seq(&self) -> usize {
        self.max_seq
    }

    /// Forget everything, without reallocating.
    ///
    /// The stale values are not zeroed: `len` bounds every read, so nothing past
    /// it is ever observed. A test asserts a reused cache gives the same logits
    /// as a fresh one.
    pub fn reset(&mut self) {
        self.len = 0;
    }

    /// Resident bytes.
    pub fn bytes(&self) -> u64 {
        (self.k.len() + self.v.len()) as u64 * (self.max_seq * self.per_pos) as u64 * 4
    }

    /// Write this position's keys and values for one layer.
    pub fn write(&mut self, layer: usize, pos: usize, k: &[f32], v: &[f32]) {
        debug_assert_eq!(k.len(), self.per_pos);
        debug_assert_eq!(v.len(), self.per_pos);
        let r = pos * self.per_pos..(pos + 1) * self.per_pos;
        self.k[layer][r.clone()].copy_from_slice(k);
        self.v[layer][r].copy_from_slice(v);
        self.len = self.len.max(pos + 1);
    }

    /// Keys and values for one layer, over positions `0..=pos`.
    pub fn read(&self, layer: usize, pos: usize) -> (&[f32], &[f32]) {
        let n = (pos + 1) * self.per_pos;
        (&self.k[layer][..n], &self.v[layer][..n])
    }

    pub fn per_pos(&self) -> usize {
        self.per_pos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> ModelConfig {
        ModelConfig::from_json(
            r#"{"hidden_size":16,"intermediate_size":32,"num_hidden_layers":2,
                "num_attention_heads":2,"num_key_value_heads":1,"vocab_size":8,
                "max_position_embeddings":4}"#,
        )
        .unwrap()
    }

    #[test]
    fn write_then_read_round_trips() {
        let c = cfg();
        let mut kv = KvCache::new(&c);
        let per = kv.per_pos();
        kv.write(0, 0, &vec![1.0; per], &vec![2.0; per]);
        kv.write(0, 1, &vec![3.0; per], &vec![4.0; per]);
        let (k, v) = kv.read(0, 1);
        assert_eq!(k.len(), 2 * per);
        assert_eq!(k[0], 1.0);
        assert_eq!(k[per], 3.0);
        assert_eq!(v[per], 4.0);
        assert_eq!(kv.len(), 2);
    }

    #[test]
    fn layers_do_not_alias() {
        let c = cfg();
        let mut kv = KvCache::new(&c);
        let per = kv.per_pos();
        kv.write(0, 0, &vec![1.0; per], &vec![1.0; per]);
        kv.write(1, 0, &vec![9.0; per], &vec![9.0; per]);
        assert_eq!(kv.read(0, 0).0[0], 1.0);
        assert_eq!(kv.read(1, 0).0[0], 9.0);
    }

    #[test]
    fn reset_makes_stale_values_unreachable() {
        let c = cfg();
        let mut kv = KvCache::new(&c);
        let per = kv.per_pos();
        kv.write(0, 0, &vec![7.0; per], &vec![7.0; per]);
        kv.reset();
        assert_eq!(kv.len(), 0);
        assert!(kv.is_empty());
        // Overwriting position 0 must be all that a new sequence observes.
        kv.write(0, 0, &vec![1.0; per], &vec![1.0; per]);
        assert_eq!(kv.read(0, 0).0[0], 1.0);
    }

    #[test]
    fn bytes_matches_the_allocation() {
        let c = cfg();
        let kv = KvCache::new(&c);
        // 2 layers * 2 (k and v) * 4 positions * 1 head * 8 dims * 4 bytes
        assert_eq!(kv.bytes(), (2 * 2 * 4) * 8 * 4);
    }
}
