//! Dense f32 matrix-vector product.
//!
//! Present because the LM head dominates per-token bandwidth on this model
//! class: 128256 x 2560 f32 is 1315 MB against 521 MB for every ternary
//! projection combined. Vectorizing the ternary kernel and leaving this scalar
//! would cap the end-to-end speedup at roughly 1.4x no matter how fast the
//! ternary path became.
//!
//! Unlike the ternary kernels, this one is **not** bit-exact against a scalar
//! reference: f32 addition is not associative, so a vectorized reduction sums in
//! a different order. The lanes are summed in a fixed order for a given row
//! length, so results are deterministic and reproducible run to run -- just not
//! identical to a sequential sum. Logit differences land around 1e-6 relative,
//! far below the 1e-2 scale at which top-1 selection changes.

/// Portable reference.
pub fn f32_matvec_scalar(w: &[f32], rows: usize, cols: usize, x: &[f32], y: &mut [f32]) {
    trit_core::backend::f32_matvec_reference(w, rows, cols, x, y)
}

#[cfg(target_arch = "x86_64")]
mod x86 {
    use std::arch::x86_64::*;

    /// # Safety
    /// Requires `avx2` and `fma`. Lengths are checked by the safe caller.
    #[target_feature(enable = "avx2,fma")]
    pub unsafe fn dot(a: *const f32, b: *const f32, n: usize) -> f32 {
        // Four accumulators to cover the FMA latency; the tail is scalar.
        let mut acc0 = _mm256_setzero_ps();
        let mut acc1 = _mm256_setzero_ps();
        let mut acc2 = _mm256_setzero_ps();
        let mut acc3 = _mm256_setzero_ps();
        let chunks = n / 32;
        for i in 0..chunks {
            let p = a.add(i * 32);
            let q = b.add(i * 32);
            acc0 = _mm256_fmadd_ps(_mm256_loadu_ps(p), _mm256_loadu_ps(q), acc0);
            acc1 = _mm256_fmadd_ps(_mm256_loadu_ps(p.add(8)), _mm256_loadu_ps(q.add(8)), acc1);
            acc2 = _mm256_fmadd_ps(_mm256_loadu_ps(p.add(16)), _mm256_loadu_ps(q.add(16)), acc2);
            acc3 = _mm256_fmadd_ps(_mm256_loadu_ps(p.add(24)), _mm256_loadu_ps(q.add(24)), acc3);
        }
        let acc = _mm256_add_ps(_mm256_add_ps(acc0, acc1), _mm256_add_ps(acc2, acc3));
        let lo = _mm256_castps256_ps128(acc);
        let hi = _mm256_extractf128_ps(acc, 1);
        let s = _mm_add_ps(lo, hi);
        let s = _mm_add_ps(s, _mm_movehl_ps(s, s));
        let s = _mm_add_ss(s, _mm_shuffle_ps(s, s, 1));
        let mut total = _mm_cvtss_f32(s);
        for i in chunks * 32..n {
            total += *a.add(i) * *b.add(i);
        }
        total
    }
}

#[cfg(target_arch = "aarch64")]
mod arm {
    use std::arch::aarch64::*;

    /// # Safety
    /// Requires `neon`. Lengths are checked by the safe caller.
    #[target_feature(enable = "neon")]
    pub unsafe fn dot(a: *const f32, b: *const f32, n: usize) -> f32 {
        let mut acc0 = vdupq_n_f32(0.0);
        let mut acc1 = vdupq_n_f32(0.0);
        let mut acc2 = vdupq_n_f32(0.0);
        let mut acc3 = vdupq_n_f32(0.0);
        let chunks = n / 16;
        for i in 0..chunks {
            let p = a.add(i * 16);
            let q = b.add(i * 16);
            acc0 = vfmaq_f32(acc0, vld1q_f32(p), vld1q_f32(q));
            acc1 = vfmaq_f32(acc1, vld1q_f32(p.add(4)), vld1q_f32(q.add(4)));
            acc2 = vfmaq_f32(acc2, vld1q_f32(p.add(8)), vld1q_f32(q.add(8)));
            acc3 = vfmaq_f32(acc3, vld1q_f32(p.add(12)), vld1q_f32(q.add(12)));
        }
        let acc = vaddq_f32(vaddq_f32(acc0, acc1), vaddq_f32(acc2, acc3));
        let mut total = vaddvq_f32(acc);
        for i in chunks * 16..n {
            total += *a.add(i) * *b.add(i);
        }
        total
    }
}

/// Dispatching dense f32 matvec. Rows are independent, so this parallelizes and
/// vectorizes without changing the summation order within any single row.
pub fn f32_matvec(w: &[f32], rows: usize, cols: usize, x: &[f32], y: &mut [f32], threads: usize) {
    assert_eq!(w.len(), rows * cols);
    assert_eq!(x.len(), cols);
    assert_eq!(y.len(), rows);

    #[allow(unused)]
    let simd: Option<unsafe fn(*const f32, *const f32, usize) -> f32> = {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
                Some(x86::dot)
            } else {
                None
            }
        }
        #[cfg(target_arch = "aarch64")]
        {
            Some(arm::dot)
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            None
        }
    };

    let Some(dot) = simd else {
        return f32_matvec_scalar(w, rows, cols, x, y);
    };

    let row = |r: usize| -> f32 {
        // SAFETY: r < rows, so the row lies inside w; cols matches x. The
        // feature check above established the ISA requirement.
        unsafe { dot(w.as_ptr().add(r * cols), x.as_ptr(), cols) }
    };

    if threads <= 1 || rows < 4096 {
        for (r, out) in y.iter_mut().enumerate() {
            *out = row(r);
        }
        return;
    }
    use rayon::prelude::*;
    let chunk = rows.div_ceil(threads).max(1);
    y.par_chunks_mut(chunk).enumerate().for_each(|(i, out)| {
        for (j, o) in out.iter_mut().enumerate() {
            *o = row(i * chunk + j);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_scalar_reference_closely() {
        let (rows, cols) = (37usize, 733usize); // deliberately not a nice multiple
        let mut state = 0xC0FFEEu64;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            ((state >> 40) as f32 / (1u64 << 24) as f32) - 0.5
        };
        let w: Vec<f32> = (0..rows * cols).map(|_| next()).collect();
        let x: Vec<f32> = (0..cols).map(|_| next()).collect();

        let mut a = vec![0.0; rows];
        let mut b = vec![0.0; rows];
        f32_matvec_scalar(&w, rows, cols, &x, &mut a);
        f32_matvec(&w, rows, cols, &x, &mut b, 1);
        for (i, (p, q)) in a.iter().zip(&b).enumerate() {
            // Reassociation only; well inside anything that could move a top-1.
            assert!((p - q).abs() <= 1e-4 * p.abs().max(1.0), "row {i}: {p} vs {q}");
        }
    }

    #[test]
    fn threading_is_deterministic() {
        let (rows, cols) = (8192usize, 128usize);
        let w: Vec<f32> = (0..rows * cols).map(|i| (i % 17) as f32 * 0.01).collect();
        let x: Vec<f32> = (0..cols).map(|i| (i % 5) as f32 * 0.1).collect();
        let mut a = vec![0.0; rows];
        f32_matvec(&w, rows, cols, &x, &mut a, 1);
        for threads in [2usize, 8, 16] {
            let mut b = vec![0.0; rows];
            f32_matvec(&w, rows, cols, &x, &mut b, threads);
            // Rows are independent, so threading cannot change a single bit.
            assert_eq!(a, b, "{threads} threads changed the result");
        }
    }
}
