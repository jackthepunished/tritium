//! Production numerics.
//!
//! The formulas are the ones `tritsim` proved against the HuggingFace reference
//! (mean logit cosine 0.9991, top-1 100%). This is an independent
//! implementation of them -- buffer-reusing, allocation-free on the hot path --
//! and the two agreeing is the cross-check that keeps either honest.
//!
//! Every reduction that can lose precision runs in `f64`, for the reason
//! documented on `absmean_quantize`: projection tensors reach tens of millions
//! of elements, where sequential `f32` summation drifts enough to matter.

use crate::config::Act;

/// The one home of the rms-scalar policy: `f64` mean-square in,
/// `1/sqrt(ms + eps)` out. Every consumer -- rmsnorm, norm folding, the integer
/// MLP -- goes through this, so eps placement and precision cannot drift apart.
#[inline]
pub fn inv_rms(mean_sq: f64, eps: f32) -> f64 {
    1.0 / (mean_sq + eps as f64).sqrt()
}

#[inline]
pub fn mean_sq_f64(x: &[f32]) -> f64 {
    x.iter().map(|v| (*v as f64) * (*v as f64)).sum::<f64>() / x.len() as f64
}

pub fn rmsnorm_into(x: &[f32], gain: &[f32], eps: f32, out: &mut [f32]) {
    let r = inv_rms(mean_sq_f64(x), eps) as f32;
    for ((o, v), g) in out.iter_mut().zip(x).zip(gain) {
        *o = v * r * g;
    }
}

pub fn rmsnorm(x: &[f32], gain: &[f32], eps: f32) -> Vec<f32> {
    let mut out = vec![0.0; x.len()];
    rmsnorm_into(x, gain, eps, &mut out);
    out
}

pub fn softmax_inplace(x: &mut [f32]) {
    let m = x.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let mut sum = 0.0;
    for v in x.iter_mut() {
        *v = (*v - m).exp();
        sum += *v;
    }
    for v in x.iter_mut() {
        *v /= sum;
    }
}

#[inline]
pub fn activate(act: Act, x: f32) -> f32 {
    match act {
        Act::Silu => x / (1.0 + (-x).exp()),
        Act::Relu2 => {
            let r = x.max(0.0);
            r * r
        }
    }
}

/// Per-token absmax int8 quantization into a caller-owned buffer.
///
/// `out` may be longer than `x` -- the padding a whole-beat kernel needs -- and
/// the tail is zeroed. Returns the scale.
pub fn absmax_codes_into(x: &[f32], out: &mut [i8]) -> f32 {
    debug_assert!(out.len() >= x.len());
    let maxabs = x.iter().fold(0.0f32, |m, v| m.max(v.abs()));
    out[x.len()..].fill(0);
    if maxabs == 0.0 {
        out[..x.len()].fill(0);
        return 1.0;
    }
    let scale = maxabs / 127.0;
    for (o, v) in out.iter_mut().zip(x) {
        // Clamp to +/-127, not -128: the negative extreme has no positive
        // counterpart, and keeping the range symmetric means a kernel may negate
        // a code without overflowing. (The SIMD kernels avoid negation anyway,
        // so they stay correct for -128 inputs from any other source.)
        *o = (v / scale).round().clamp(-127.0, 127.0) as i8;
    }
    scale
}

/// Norm folding: absmax codes are invariant under uniform positive scaling, so
/// for a BitLinear fed by rmsnorm the codes come from `x .* g` alone and the
/// `1/rms` factor never enters the per-element datapath.
///
/// Returns `(scale of x .* g, r = 1/rms)`; their product is the scale plain
/// quantization of the normed vector would have produced.
pub fn scaled_absmax_codes_into(
    x: &[f32],
    gain: &[f32],
    eps: f32,
    scratch: &mut Vec<f32>,
    out: &mut [i8],
) -> (f32, f32) {
    scratch.clear();
    scratch.extend(x.iter().zip(gain).map(|(a, b)| a * b));
    let scale = absmax_codes_into(scratch, out);
    let r = inv_rms(mean_sq_f64(x), eps) as f32;
    (scale, r)
}

/// f64 absmax codes; the sibling of `absmax_codes_into` for the integer MLP's
/// code search. Same policy: round half away from zero, clamp to +/-127,
/// all-zero input yields scale 1.0.
pub fn absmax_codes_f64_into(z: &[f64], out: &mut [i8]) -> f64 {
    debug_assert!(out.len() >= z.len());
    out[z.len()..].fill(0);
    let maxabs = z.iter().fold(0f64, |m, v| m.max(v.abs()));
    if maxabs == 0.0 {
        out[..z.len()].fill(0);
        return 1.0;
    }
    let scale = maxabs / 127.0;
    for (o, v) in out.iter_mut().zip(z) {
        *o = (v / scale).round().clamp(-127.0, 127.0) as i8;
    }
    scale
}

/// Integer-exact squared-ReLU: relu2 without floats in the element math.
///
/// With `g_i = acc_g_i * S_g` and `u_i = acc_u_i * S_u` (uniform positive
/// scales), `relu(g)^2 * u = t_i * K` for the integer
/// `t_i = relu(acc_g_i)^2 * acc_u_i` and `K = S_g^2 * S_u`. Absmax codes are
/// invariant to `K`, so the down projection's codes come from `t .* gain`
/// directly -- and the f32 activation vector, whose dynamic range reaches 1.5e10
/// on the real checkpoint, is never materialized at all.
///
/// `|t| < 2^55` for this model's widths (each accumulator is bounded by
/// `2560 * 127 = 325_120`), which is exact in `i64`; the `f64` used for the code
/// search adds at most `2^-53` relative error, far below the 1/254 code
/// granularity.
///
/// Owns all of the `K` algebra, so the exponent bookkeeping lives in one place.
/// Returns the activation scale for the down projection. The codes themselves do
/// not depend on `K` at all -- only the returned scale does.
#[allow(clippy::too_many_arguments)]
pub fn int_mlp_codes_into(
    acc_g: &[i32],
    acc_u: &[i32],
    gain: &[f32],
    s_g: f64,
    s_u: f64,
    eps: f32,
    t_buf: &mut Vec<i64>,
    z_buf: &mut Vec<f64>,
    out: &mut [i8],
) -> f32 {
    let n = acc_g.len();
    assert_eq!(n, acc_u.len());
    assert_eq!(n, gain.len());

    t_buf.clear();
    t_buf.extend(acc_g.iter().zip(acc_u).map(|(&g, &u)| {
        let rg = g.max(0) as i64;
        rg * rg * u as i64
    }));
    z_buf.clear();
    z_buf.extend(t_buf.iter().zip(gain).map(|(&ti, &gi)| ti as f64 * gi as f64));

    let qs = absmax_codes_f64_into(z_buf, out);
    if qs == 1.0 && z_buf.iter().all(|&v| v == 0.0) {
        return 1.0;
    }
    // Folded scalars: a = K*t with K = s_g^2 * s_u; r = 1/sqrt(mean(a^2) + eps).
    let k = s_g * s_g * s_u;
    let mean_t2 = t_buf
        .iter()
        .map(|&v| {
            let f = v as f64;
            f * f
        })
        .sum::<f64>()
        / n as f64;
    (qs * k * inv_rms(k * k * mean_t2, eps)) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn xs(state: &mut u64) -> u64 {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        *state
    }
    fn xs_f32(state: &mut u64) -> f32 {
        ((xs(state) >> 40) as f32 / (1u64 << 24) as f32) - 0.5
    }

    #[test]
    fn absmax_hand_computed() {
        // max|x| = 2.54, scale = 0.02; round half away from zero
        let mut out = [0i8; 3];
        let s = absmax_codes_into(&[0.0, -2.54, 1.27], &mut out);
        assert_eq!(out, [0, -127, 64]);
        assert!((s - 0.02).abs() < 1e-6);
    }

    #[test]
    fn absmax_pads_the_tail_with_zeros() {
        let mut out = [9i8; 8];
        absmax_codes_into(&[1.0, -1.0], &mut out);
        assert_eq!(&out[2..], &[0; 6], "padding must be zero for whole-beat kernels");
    }

    #[test]
    fn absmax_all_zero_is_safe() {
        let mut out = [9i8; 4];
        assert_eq!(absmax_codes_into(&[0.0; 4], &mut out), 1.0);
        assert_eq!(out, [0; 4]);
    }

    /// Norm folding must produce EXACTLY the codes that quantizing after
    /// rmsnorm would, including for the massive activations this model shows
    /// (residual max_abs 137,694 measured on the real checkpoint).
    #[test]
    fn folding_codes_match_norm_then_quantize() {
        let mut st = 0xF01Du64;
        for case in 0..1000 {
            let n = 1 + (xs(&mut st) % 64) as usize;
            let mut x: Vec<f32> = (0..n).map(|_| xs_f32(&mut st) * 10.0).collect();
            let g: Vec<f32> = (0..n).map(|_| xs_f32(&mut st) + 1.0).collect();
            // Inject the occasional outlier spike.
            if case % 7 == 0 {
                x[0] = 1e4;
            }
            if case % 11 == 0 {
                x[n / 2] = -3.4e4;
            }

            let mut folded = vec![0i8; n];
            let mut scratch = Vec::new();
            let (s, r) = scaled_absmax_codes_into(&x, &g, 1e-5, &mut scratch, &mut folded);

            let normed = rmsnorm(&x, &g, 1e-5);
            let mut direct = vec![0i8; n];
            let ds = absmax_codes_into(&normed, &mut direct);

            assert_eq!(folded, direct, "case {case}: folded codes differ");
            let rel = ((s * r) - ds).abs() / ds.abs().max(1e-30);
            assert!(rel < 1e-4, "case {case}: scale {s}*{r} vs {ds}");
        }
    }

    /// The integer path must agree with the f32 folded reference to within one
    /// code step, and the accumulator bound must hold exactly.
    #[test]
    fn int_mlp_matches_the_f32_folded_reference() {
        let mut st = 0xBEEFu64;
        let mut flips = 0usize;
        let mut total = 0usize;
        for _ in 0..500 {
            let n = 1 + (xs(&mut st) % 48) as usize;
            let acc_g: Vec<i32> = (0..n).map(|_| (xs(&mut st) % 650_001) as i32 - 325_000).collect();
            let acc_u: Vec<i32> = (0..n).map(|_| (xs(&mut st) % 650_001) as i32 - 325_000).collect();
            // Dyadic gains: exactly representable, so any disagreement is the
            // integer path's, not the test fixture's rounding.
            let gain: Vec<f32> = (0..n).map(|_| ((xs(&mut st) % 17) as f32 - 8.0) * 0.25).collect();
            let (s_g, s_u) = (1.7e-4f64, 3.1e-4f64);

            let mut t = Vec::new();
            let mut z = Vec::new();
            let mut codes = vec![0i8; n];
            int_mlp_codes_into(&acc_g, &acc_u, &gain, s_g, s_u, 1e-5, &mut t, &mut z, &mut codes);

            // f32 reference: materialize the activation and fold as usual.
            let a: Vec<f32> = acc_g
                .iter()
                .zip(&acc_u)
                .map(|(&g, &u)| {
                    let gv = g as f64 * s_g;
                    let uv = u as f64 * s_u;
                    (gv.max(0.0) * gv.max(0.0) * uv) as f32
                })
                .collect();
            let mut scratch = Vec::new();
            let mut ref_codes = vec![0i8; n];
            scaled_absmax_codes_into(&a, &gain, 1e-5, &mut scratch, &mut ref_codes);

            for (c, rc) in codes.iter().zip(&ref_codes) {
                assert!((*c as i32 - *rc as i32).abs() <= 1, "code differs by more than one step");
                if c != rc {
                    flips += 1;
                }
                total += 1;
            }
        }
        // The f32 reference is the less accurate side; a small number of
        // boundary flips is expected and bounded.
        assert!(flips * 1000 <= total, "too many code flips: {flips}/{total}");
    }

    /// At the theoretical accumulator bound the i64 product must be exact even
    /// though it exceeds 2^53 and cannot be represented in f64.
    #[test]
    fn int_mlp_is_exact_at_the_accumulator_bound() {
        let bound = 2560 * 127; // 325_120
        let acc_g = vec![bound; 4];
        let acc_u = vec![bound; 4];
        let gain = vec![1.0f32; 4];
        let mut t = Vec::new();
        let mut z = Vec::new();
        let mut codes = vec![0i8; 4];
        int_mlp_codes_into(&acc_g, &acc_u, &gain, 1e-4, 1e-4, 1e-5, &mut t, &mut z, &mut codes);
        let expect = (bound as i64) * (bound as i64) * (bound as i64);
        assert_eq!(t[0], expect, "i64 product must be exact");
        assert!(expect.unsigned_abs() > (1u64 << 53), "this case must exceed f64 integer range");
        assert_eq!(codes, vec![127; 4], "all equal, so all saturate to the max code");
    }

    /// Codes are invariant to the effective scales; only the returned scale
    /// depends on them. This is the algebra the whole folded path rests on.
    #[test]
    fn int_mlp_codes_do_not_depend_on_the_scales() {
        let acc_g: Vec<i32> = vec![100, -5, 3000, 0, 42];
        let acc_u: Vec<i32> = vec![-7, 900, 12, 5, -300];
        let gain = vec![0.5f32, 1.0, -0.25, 2.0, 1.5];
        let mut a = vec![0i8; 5];
        let mut b = vec![0i8; 5];
        let (mut t, mut z) = (Vec::new(), Vec::new());
        let s1 = int_mlp_codes_into(&acc_g, &acc_u, &gain, 1e-4, 2e-4, 1e-5, &mut t, &mut z, &mut a);
        let s2 = int_mlp_codes_into(&acc_g, &acc_u, &gain, 7e-3, 5e-2, 1e-5, &mut t, &mut z, &mut b);
        assert_eq!(a, b, "codes must be scale-invariant");
        assert!(s1 != s2, "scales must differ");
    }
}
