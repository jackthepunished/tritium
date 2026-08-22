//! Rotary position embeddings, precomputed per position.
//!
//! The reference implementation recomputes `theta.powf(...)` and `sin_cos()` for
//! every element of every head: on BitNet 2B4T that is 20 query heads plus 5 KV
//! heads over 128 pairs, so 1600 transcendental pairs per token, all of which
//! collapse to the same `head_dim/2` values.
//!
//! This computes those `head_dim/2` values once per position and shares them
//! across every head.
//!
//! # Why the arithmetic is deliberately f32
//!
//! `inv_freq` is computed in `f32`, exactly as the reference does, and the angle
//! and `sin_cos` follow in `f32`. Computing it in `f64` would be *more accurate*
//! -- at position 2048 the `f32` path accumulates roughly 1e-3 radians of angle
//! error -- but it would also disagree with the oracle the model was validated
//! against, and disagreement is what the two-implementation cross-check exists
//! to detect. Precomputing the same `f32` values is a pure speedup with
//! bit-identical output. Raising RoPE precision is a separate, deliberate
//! numerics change that needs its own quality gate.

pub struct RopeTable {
    head_dim: usize,
    inv_freq: Vec<f32>,
    /// `(sin, cos)` for the position currently loaded.
    sin_cos: Vec<(f32, f32)>,
    pos: Option<usize>,
}

impl RopeTable {
    pub fn new(head_dim: usize, theta: f32) -> Self {
        let half = head_dim / 2;
        let inv_freq = (0..half)
            .map(|i| theta.powf(-2.0 * i as f32 / head_dim as f32))
            .collect();
        Self { head_dim, inv_freq, sin_cos: vec![(0.0, 0.0); half], pos: None }
    }

    /// Load the trig for `pos`, if it is not already loaded.
    pub fn seek(&mut self, pos: usize) {
        if self.pos == Some(pos) {
            return;
        }
        for (sc, f) in self.sin_cos.iter_mut().zip(&self.inv_freq) {
            *sc = (pos as f32 * f).sin_cos();
        }
        self.pos = Some(pos);
    }

    /// Rotate-half RoPE (HuggingFace LLaMA convention): the pair for index `i` is
    /// `(v[i], v[i + head_dim/2])`. Applies to every head in `v`.
    pub fn apply(&self, v: &mut [f32]) {
        let half = self.head_dim / 2;
        for head in v.chunks_mut(self.head_dim) {
            for (i, &(sin, cos)) in self.sin_cos.iter().enumerate() {
                let (a, b) = (head[i], head[i + half]);
                head[i] = a * cos - b * sin;
                head[i + half] = a * sin + b * cos;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The reference formulation, inline, so the table cannot drift from it.
    fn rope_reference(v: &mut [f32], head_dim: usize, pos: usize, theta: f32) {
        let half = head_dim / 2;
        for head in v.chunks_mut(head_dim) {
            for i in 0..half {
                let inv_freq = theta.powf(-2.0 * i as f32 / head_dim as f32);
                let angle = pos as f32 * inv_freq;
                let (sin, cos) = angle.sin_cos();
                let (a, b) = (head[i], head[i + half]);
                head[i] = a * cos - b * sin;
                head[i + half] = a * sin + b * cos;
            }
        }
    }

    #[test]
    fn table_is_bit_identical_to_recomputing_per_element() {
        let (head_dim, heads, theta) = (128usize, 3usize, 500000.0f32);
        let mut state = 0x5EEDu64;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            ((state >> 40) as f32 / (1u64 << 24) as f32) - 0.5
        };
        let base: Vec<f32> = (0..head_dim * heads).map(|_| next() * 20.0).collect();

        let mut table = RopeTable::new(head_dim, theta);
        // Cover position 0, small positions, and the far end of the context,
        // where the f32 angle error is largest.
        for pos in [0usize, 1, 7, 255, 1023, 2047] {
            let mut a = base.clone();
            let mut b = base.clone();
            table.seek(pos);
            table.apply(&mut a);
            rope_reference(&mut b, head_dim, pos, theta);
            assert_eq!(a.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
                       b.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
                       "pos {pos}: table differs from per-element recomputation");
        }
    }

    #[test]
    fn position_zero_is_the_identity() {
        let mut table = RopeTable::new(8, 10000.0);
        table.seek(0);
        let mut v = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let before = v.clone();
        table.apply(&mut v);
        assert_eq!(v, before);
    }

    #[test]
    fn seek_is_idempotent() {
        let mut table = RopeTable::new(8, 10000.0);
        table.seek(5);
        let first = table.sin_cos.clone();
        table.seek(5);
        assert_eq!(table.sin_cos, first);
    }
}
