#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Act {
    Silu,
    Relu2,
}

pub fn rmsnorm(x: &[f32], gain: &[f32], eps: f32) -> Vec<f32> {
    // f64 mean-square for consistency with scaled_absmax_codes (and the same
    // accumulation-precision reasoning as absmean_quantize).
    let ms = x.iter().map(|v| (*v as f64) * (*v as f64)).sum::<f64>() / x.len() as f64;
    let r = (1.0 / (ms + eps as f64).sqrt()) as f32;
    x.iter().zip(gain).map(|(v, g)| v * r * g).collect()
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

/// Rotate-half RoPE (HF LLaMA convention): pairs are (v[i], v[i + head_dim/2]).
pub fn rope_inplace(v: &mut [f32], head_dim: usize, pos: usize, theta: f32) {
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

/// Norm folding: absmax int8 codes are invariant to uniform positive scaling,
/// so for a BitLinear fed by rmsnorm the codes can be computed from x .* g
/// alone -- the 1/rms factor never enters the per-element datapath. Returns
/// (codes, scale of x.*g, r = 1/rms) with `scale * r` equal to the scale
/// absmax_quantize would have produced on the normed vector.
/// The rms sum runs in f64 for the same reason as absmean_quantize.
pub fn scaled_absmax_codes(x: &[f32], g: &[f32], eps: f32) -> (Vec<i8>, f32, f32) {
    let z: Vec<f32> = x.iter().zip(g).map(|(a, b)| a * b).collect();
    let (codes, scale) = trit_core::quant::absmax_quantize(&z);
    let ms = x.iter().map(|v| (*v as f64) * (*v as f64)).sum::<f64>() / x.len() as f64;
    let r = (1.0 / (ms + eps as f64).sqrt()) as f32;
    (codes, scale, r)
}

/// Integer-exact MLP activation path (relu2 without floats in the element
/// math): with g_i = acc_g_i * S_g and u_i = acc_u_i * S_u (uniform positive
/// scales), relu(g)^2 * u = t_i * K for integer t_i = relu(acc_g_i)^2 * acc_u_i
/// and K = S_g^2 * S_u. absmax codes are invariant to K (norm folding), so the
/// down projection's codes come from t .* gain directly. |t| < 2^55 for this
/// model's widths — exact in i64; the f64 used for the code search adds
/// <= 2^-53 relative error, far below the 1/254 code granularity.
/// Returns (codes, activation scale for the down projection).
pub fn int_mlp_codes(acc_g: &[i32], acc_u: &[i32], gain: &[f32], k: f64, eps: f32) -> (Vec<i8>, f32) {
    let n = acc_g.len();
    assert_eq!(n, acc_u.len());
    assert_eq!(n, gain.len());
    let t: Vec<i64> = acc_g
        .iter()
        .zip(acc_u)
        .map(|(&g, &u)| {
            let rg = g.max(0) as i64;
            rg * rg * u as i64
        })
        .collect();
    let z: Vec<f64> = t.iter().zip(gain).map(|(&ti, &gi)| ti as f64 * gi as f64).collect();
    let maxabs = z.iter().fold(0f64, |m, v| m.max(v.abs()));
    if maxabs == 0.0 {
        return (vec![0; n], 1.0);
    }
    let qs = maxabs / 127.0;
    let codes = z
        .iter()
        .map(|v| (v / qs).round().clamp(-127.0, 127.0) as i8)
        .collect();
    // folded scalars: a = K*t, r = 1/sqrt(mean(a^2)+eps); scale = qs*K*r
    let mean_t2 = t
        .iter()
        .map(|&v| {
            let f = v as f64;
            f * f
        })
        .sum::<f64>()
        / n as f64;
    let r = 1.0 / (k * k * mean_t2 + eps as f64).sqrt();
    let x_scale = (qs * k * r) as f32;
    (codes, x_scale)
}

pub fn activate(act: Act, x: f32) -> f32 {
    match act {
        Act::Silu => x / (1.0 + (-x).exp()),
        Act::Relu2 => {
            let r = x.max(0.0);
            r * r
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trit_core::quant::absmax_quantize;

    #[test]
    fn folding_codes_match_norm_then_quantize() {
        // absmax int8 codes are invariant to the uniform 1/rms factor, so
        // quantize(rmsnorm(x, g)) and scaled_absmax_codes(x, g) must produce
        // identical codes, and the scales must differ by exactly r = 1/rms
        // (up to f32 rounding).
        let mut state = 0x5eed_f01du64;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            ((state >> 40) as f32 / (1u64 << 24) as f32) - 0.5
        };
        for case in 0..1000 {
            let n = 64;
            let mut x: Vec<f32> = (0..n).map(|_| next() * 4.0).collect();
            let g: Vec<f32> = (0..n).map(|_| next() * 2.0).collect();
            if case % 3 == 1 {
                x[case % n] = 1.0e4; // massive-activation spike
            }
            if case % 3 == 2 {
                x[case % n] = -3.4e4;
            }
            let eps = 1e-5;
            let (ref_codes, ref_scale) = absmax_quantize(&rmsnorm(&x, &g, eps));
            let (codes, scale, r) = scaled_absmax_codes(&x, &g, eps);
            assert_eq!(codes, ref_codes, "case {case}: codes diverged");
            let folded = scale * r;
            assert!(
                (folded - ref_scale).abs() <= 1e-6 * ref_scale.abs().max(1e-30),
                "case {case}: folded scale {folded} vs {ref_scale}"
            );
        }
    }

    #[test]
    fn int_mlp_codes_match_f32_folded_reference() {
        // Reference: build a_i = relu(acc_g*sg)^2 * (acc_u*su) in f32 and run
        // the Phase-2 folded quantizer. The i64 path rounds once instead of
        // per-element, so codes may differ by at most 1 at quantization
        // boundaries; the test quantifies flips and bounds them.
        let mut state = 0x147_31d0u64;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        let (mut flips, mut elems) = (0usize, 0usize);
        for case in 0..500 {
            let n = 64;
            // |acc| <= 100 keeps the f32 reference arithmetic exact (< 2^24)
            let acc_g: Vec<i32> = (0..n).map(|_| (next() % 201) as i32 - 100).collect();
            let acc_u: Vec<i32> = (0..n).map(|_| (next() % 201) as i32 - 100).collect();
            let gain: Vec<f32> = (0..n).map(|_| ((next() % 400) as f32 / 100.0) - 2.0).collect();
            let (sg, su) = (1.0f32, 1.0f32);
            let a: Vec<f32> = acc_g
                .iter()
                .zip(&acc_u)
                .map(|(&g, &u)| {
                    let rg = (g as f32 * sg).max(0.0);
                    rg * rg * (u as f32 * su)
                })
                .collect();
            let (ref_codes, s, r) = scaled_absmax_codes(&a, &gain, 1e-5);
            let (codes, x_scale) = int_mlp_codes(&acc_g, &acc_u, &gain, 1.0, 1e-5);
            for (i, (&c, &rc)) in codes.iter().zip(&ref_codes).enumerate() {
                let d = (c as i16 - rc as i16).abs();
                assert!(d <= 1, "case {case} elem {i}: code {c} vs ref {rc}");
                flips += (d != 0) as usize;
                elems += 1;
            }
            let ref_scale = s * r;
            if ref_scale != 0.0 {
                assert!(
                    ((x_scale - ref_scale) / ref_scale).abs() < 1e-5,
                    "case {case}: scale {x_scale} vs {ref_scale}"
                );
            }
        }
        // boundary-straddling by construction never exceeds +/-1 (asserted
        // above); random flips must be rare
        assert!(flips * 1000 <= elems, "flips {flips}/{elems} exceed 0.1%");
        println!("int_mlp_codes: {flips} boundary flips over {elems} elements");
    }

    #[test]
    fn int_mlp_codes_exact_at_theoretical_bound() {
        // accs at the model bound 2560*127: |t| ~ 2^55 exceeds f64's exact
        // integer range (2^53); the induced relative error (<= 2^-53) must
        // stay far below code granularity (1/254). Symmetric construction
        // makes expected codes exact by hand.
        let b = 2560 * 127; // 325,120
        let acc_g = vec![b, b, -b, 0];
        let acc_u = vec![b, -b, b, b];
        let gain = vec![1.0f32, 1.0, 1.0, 1.0];
        let (codes, _) = int_mlp_codes(&acc_g, &acc_u, &gain, 1.0, 1e-5);
        // t = [b^3, -b^3, 0, 0] -> codes [127, -127, 0, 0]
        assert_eq!(codes, vec![127, -127, 0, 0]);
    }

    #[test]
    fn rmsnorm_hand_computed() {
        // x=[3,4]: mean sq = 12.5, rms = 3.53553; y = x/rms * g
        let y = rmsnorm(&[3.0, 4.0], &[1.0, 2.0], 0.0);
        assert!((y[0] - 0.848528).abs() < 1e-5);
        assert!((y[1] - 2.262742).abs() < 1e-5);
    }

    #[test]
    fn softmax_sums_to_one_and_orders() {
        let mut x = vec![1.0, 2.0, 3.0];
        softmax_inplace(&mut x);
        assert!((x.iter().sum::<f32>() - 1.0).abs() < 1e-6);
        assert!(x[2] > x[1] && x[1] > x[0]);
    }

    #[test]
    fn rope_position_zero_is_identity() {
        let mut v = vec![1.0, 2.0, 3.0, 4.0]; // one head, dim 4
        rope_inplace(&mut v, 4, 0, 10000.0);
        assert_eq!(v, vec![1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn rope_hand_computed_dim2() {
        // head_dim=2: single pair (v[0], v[1]), inv_freq=1, angle=pos*1=1 rad
        let mut v = vec![1.0, 0.0];
        rope_inplace(&mut v, 2, 1, 10000.0);
        assert!((v[0] - 1f32.cos()).abs() < 1e-6);
        assert!((v[1] - 1f32.sin()).abs() < 1e-6);
    }

    #[test]
    fn activations() {
        assert_eq!(activate(Act::Relu2, -2.0), 0.0);
        assert_eq!(activate(Act::Relu2, 3.0), 9.0);
        // silu(1) = 1/(1+e^-1) = 0.731058
        assert!((activate(Act::Silu, 1.0) - 0.731058).abs() < 1e-5);
    }
}
