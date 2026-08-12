/// BitNet b1.58 weight quantization: scale = mean(|w|), trits = clamp(round(w/scale), -1, 1).
pub fn absmean_quantize(w: &[f32]) -> (Vec<i8>, f32) {
    let scale = w.iter().map(|v| v.abs()).sum::<f32>() / w.len() as f32 + 1e-8;
    let trits = w
        .iter()
        .map(|v| (v / scale).round().clamp(-1.0, 1.0) as i8)
        .collect();
    (trits, scale)
}

/// Per-token int8 activation quantization: scale = max(|x|)/127.
pub fn absmax_quantize(x: &[f32]) -> (Vec<i8>, f32) {
    let maxabs = x.iter().fold(0.0f32, |m, v| m.max(v.abs()));
    if maxabs == 0.0 {
        return (vec![0; x.len()], 1.0);
    }
    let scale = maxabs / 127.0;
    let xq = x
        .iter()
        .map(|v| (v / scale).round().clamp(-127.0, 127.0) as i8)
        .collect();
    (xq, scale)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absmean_hand_computed() {
        // mean(|w|) = (0.5 + 1.2 + 0.1 + 2.0) / 4 = 0.95
        // w/s = [0.526, -1.263, 0.105, 2.105] -> round+clamp -> [1, -1, 0, 1]
        let (trits, scale) = absmean_quantize(&[0.5, -1.2, 0.1, 2.0]);
        assert_eq!(trits, vec![1, -1, 0, 1]);
        assert!((scale - 0.95).abs() < 1e-6);
    }

    #[test]
    fn absmean_all_zero_is_safe() {
        let (trits, scale) = absmean_quantize(&[0.0, 0.0]);
        assert_eq!(trits, vec![0, 0]);
        assert!(scale > 0.0);
    }

    #[test]
    fn absmax_hand_computed() {
        // max|x| = 2.54, scale = 2.54/127 = 0.02
        // xq = round([0, -127, 63.5]) = [0, -127, 64] (round half away from zero)
        let (xq, scale) = absmax_quantize(&[0.0, -2.54, 1.27]);
        assert_eq!(xq, vec![0, -127, 64]);
        assert!((scale - 0.02).abs() < 1e-6);
    }

    #[test]
    fn absmax_all_zero_is_safe() {
        let (xq, scale) = absmax_quantize(&[0.0; 4]);
        assert_eq!(xq, vec![0, 0, 0, 0]);
        assert_eq!(scale, 1.0);
    }
}
