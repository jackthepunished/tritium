#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Act {
    Silu,
    Relu2,
}

pub fn rmsnorm(x: &[f32], gain: &[f32], eps: f32) -> Vec<f32> {
    let ms = x.iter().map(|v| v * v).sum::<f32>() / x.len() as f32;
    let r = 1.0 / (ms + eps).sqrt();
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
