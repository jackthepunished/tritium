//! Token sampling.
//!
//! Deterministic for a given seed, with no `rand` dependency: xoshiro256** in a
//! few lines is enough for sampling and keeps the dependency graph small enough
//! to audit.

#[derive(Clone, Copy, Debug)]
pub struct SamplerParams {
    /// `0.0` selects greedy decoding.
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: usize,
    /// `1.0` disables the penalty.
    pub repetition_penalty: f32,
    pub seed: u64,
}

impl Default for SamplerParams {
    fn default() -> Self {
        Self {
            temperature: 0.0,
            top_p: 1.0,
            top_k: 0,
            repetition_penalty: 1.0,
            seed: 0,
        }
    }
}

pub trait Sampler: Send {
    /// Pick a token. `logits` may be modified in place. `history` is the tokens
    /// generated so far, for the repetition penalty.
    fn sample(&mut self, logits: &mut [f32], history: &[u32]) -> u32;
    fn name(&self) -> &str;
}

/// Argmax. Ties break toward the lower index, via `total_cmp` so NaN ordering is
/// total and the result is reproducible.
#[derive(Debug, Default, Clone, Copy)]
pub struct Greedy;

impl Sampler for Greedy {
    fn sample(&mut self, logits: &mut [f32], _history: &[u32]) -> u32 {
        argmax(logits) as u32
    }
    fn name(&self) -> &str {
        "greedy"
    }
}

pub fn argmax(v: &[f32]) -> usize {
    v.iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1).then(b.0.cmp(&a.0)))
        .map(|(i, _)| i)
        .unwrap_or(0)
}

/// xoshiro256** -- small, fast, and good enough for sampling.
struct Rng(u64, u64, u64, u64);

impl Rng {
    fn new(seed: u64) -> Self {
        // SplitMix64 to spread a single seed across the state; a zero state
        // would make xoshiro produce nothing but zeros.
        let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut next = || {
            z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut x = z;
            x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            x ^ (x >> 31)
        };
        Self(next(), next(), next(), next())
    }

    fn next_u64(&mut self) -> u64 {
        let result = self.1.wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.1 << 17;
        self.2 ^= self.0;
        self.3 ^= self.1;
        self.1 ^= self.2;
        self.0 ^= self.3;
        self.2 ^= t;
        self.3 = self.3.rotate_left(45);
        result
    }

    /// Uniform in `[0, 1)`.
    fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }
}

/// Temperature, then top-k, then top-p (nucleus), with an optional repetition
/// penalty. Applied in that order, which is the convention the reference
/// implementations use.
pub struct TopPSampler {
    params: SamplerParams,
    rng: Rng,
    scratch: Vec<(f32, u32)>,
}

impl TopPSampler {
    pub fn new(params: SamplerParams) -> Self {
        Self {
            rng: Rng::new(params.seed),
            params,
            scratch: Vec::new(),
        }
    }
}

impl Sampler for TopPSampler {
    fn sample(&mut self, logits: &mut [f32], history: &[u32]) -> u32 {
        let p = self.params;

        if p.repetition_penalty != 1.0 && !history.is_empty() {
            for &t in history {
                if let Some(l) = logits.get_mut(t as usize) {
                    // Divide when positive, multiply when negative, so the
                    // penalty always moves a logit toward zero.
                    *l = if *l > 0.0 {
                        *l / p.repetition_penalty
                    } else {
                        *l * p.repetition_penalty
                    };
                }
            }
        }
        if p.temperature <= 0.0 {
            return argmax(logits) as u32;
        }

        self.scratch.clear();
        self.scratch.extend(
            logits
                .iter()
                .enumerate()
                .map(|(i, &l)| (l / p.temperature, i as u32)),
        );
        // Descending by logit.
        self.scratch.sort_unstable_by(|a, b| b.0.total_cmp(&a.0));

        let k = if p.top_k == 0 {
            self.scratch.len()
        } else {
            p.top_k.min(self.scratch.len())
        };
        self.scratch.truncate(k);

        // Softmax over the surviving candidates, shifted by the max for
        // stability (the list is sorted, so that is element 0).
        let max = self.scratch[0].0;
        let mut sum = 0.0f32;
        for e in self.scratch.iter_mut() {
            e.0 = (e.0 - max).exp();
            sum += e.0;
        }

        // Nucleus: keep the shortest prefix whose mass reaches top_p.
        let mut cutoff = self.scratch.len();
        if p.top_p < 1.0 {
            let mut acc = 0.0f32;
            for (i, e) in self.scratch.iter().enumerate() {
                acc += e.0 / sum;
                if acc >= p.top_p {
                    cutoff = i + 1;
                    break;
                }
            }
        }
        let kept = &self.scratch[..cutoff];
        let total: f32 = kept.iter().map(|e| e.0).sum();

        let mut r = self.rng.next_f32() * total;
        for e in kept {
            r -= e.0;
            if r <= 0.0 {
                return e.1;
            }
        }
        kept.last().map(|e| e.1).unwrap_or(0)
    }

    fn name(&self) -> &str {
        "top-p"
    }
}

/// `temperature == 0` yields greedy decoding, whatever the other knobs say.
pub fn make_sampler(p: &SamplerParams) -> Box<dyn Sampler> {
    if p.temperature <= 0.0 && p.repetition_penalty == 1.0 {
        Box::new(Greedy)
    } else {
        Box::new(TopPSampler::new(*p))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greedy_picks_the_max_and_breaks_ties_low() {
        assert_eq!(Greedy.sample(&mut [1.0, 5.0, 2.0], &[]), 1);
        assert_eq!(
            Greedy.sample(&mut [5.0, 5.0, 2.0], &[]),
            0,
            "ties break toward the lower index"
        );
    }

    #[test]
    fn zero_temperature_is_greedy() {
        let mut s = make_sampler(&SamplerParams::default());
        assert_eq!(s.name(), "greedy");
        assert_eq!(s.sample(&mut [0.1, 0.9, 0.3], &[]), 1);
    }

    #[test]
    fn same_seed_gives_the_same_sequence() {
        let p = SamplerParams {
            temperature: 1.0,
            seed: 42,
            ..Default::default()
        };
        let draw = || {
            let mut s = TopPSampler::new(p);
            (0..32)
                .map(|_| s.sample(&mut [1.0, 2.0, 3.0, 0.5], &[]))
                .collect::<Vec<_>>()
        };
        assert_eq!(draw(), draw());
    }

    #[test]
    fn different_seeds_diverge() {
        let mk = |seed| {
            let mut s = TopPSampler::new(SamplerParams {
                temperature: 1.0,
                seed,
                ..Default::default()
            });
            (0..64)
                .map(|_| s.sample(&mut [1.0, 1.0, 1.0, 1.0], &[]))
                .collect::<Vec<_>>()
        };
        assert_ne!(mk(1), mk(2));
    }

    #[test]
    fn top_k_one_is_deterministic_and_equals_argmax() {
        let mut s = TopPSampler::new(SamplerParams {
            temperature: 1.0,
            top_k: 1,
            seed: 7,
            ..Default::default()
        });
        for _ in 0..16 {
            assert_eq!(s.sample(&mut [0.1, 9.0, 0.3, 0.2], &[]), 1);
        }
    }

    #[test]
    fn top_p_excludes_the_tail() {
        // One token holds almost all the mass, so a tight nucleus must never
        // select any other.
        let mut s = TopPSampler::new(SamplerParams {
            temperature: 1.0,
            top_p: 0.5,
            seed: 3,
            ..Default::default()
        });
        for _ in 0..64 {
            assert_eq!(s.sample(&mut [20.0, 0.0, 0.0, 0.0], &[]), 0);
        }
    }

    #[test]
    fn repetition_penalty_moves_logits_toward_zero() {
        let mut s = TopPSampler::new(SamplerParams {
            temperature: 0.0,
            repetition_penalty: 2.0,
            ..Default::default()
        });
        // Token 1 leads, but has been seen; the penalty halves it below token 0.
        assert_eq!(s.sample(&mut [3.0, 4.0], &[1]), 0);
        // A negative logit is pushed further down, not up.
        let mut l = [-1.0f32, -3.0];
        s.sample(&mut l, &[0]);
        assert_eq!(l[0], -2.0);
    }
}
