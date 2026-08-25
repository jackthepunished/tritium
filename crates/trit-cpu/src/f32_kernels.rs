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

use std::sync::OnceLock;

/// Portable reference.
pub fn f32_matvec_scalar(w: &[f32], rows: usize, cols: usize, x: &[f32], y: &mut [f32]) {
    trit_core::backend::f32_matvec_reference(w, rows, cols, x, y)
}

#[cfg(target_arch = "x86_64")]
mod x86 {
    use std::arch::x86_64::*;

    /// bf16 widened in-register, then ordinary FMA against f32 activations.
    ///
    /// Not `vdpbf16ps`: it needs both operands in bf16, rounding the
    /// activations too. Measured 1.25e-1 max relative deviation against 9.7e-6
    /// here, and it was not faster.
    ///
    /// # Safety
    /// Requires `avx512f` and `avx512bw`.
    #[target_feature(enable = "avx512f,avx512bw")]
    pub unsafe fn dot_bf16_avx512(w: *const u16, x: *const f32, n: usize) -> f32 {
        #[inline]
        unsafe fn widen(p: *const u16) -> __m512 {
            let raw = _mm256_loadu_si256(p as *const __m256i);
            _mm512_castsi512_ps(_mm512_slli_epi32(_mm512_cvtepu16_epi32(raw), 16))
        }
        let (mut a0, mut a1) = (_mm512_setzero_ps(), _mm512_setzero_ps());
        let (mut a2, mut a3) = (_mm512_setzero_ps(), _mm512_setzero_ps());
        let chunks = n / 64;
        for i in 0..chunks {
            let (p, q) = (w.add(i * 64), x.add(i * 64));
            a0 = _mm512_fmadd_ps(widen(p), _mm512_loadu_ps(q), a0);
            a1 = _mm512_fmadd_ps(widen(p.add(16)), _mm512_loadu_ps(q.add(16)), a1);
            a2 = _mm512_fmadd_ps(widen(p.add(32)), _mm512_loadu_ps(q.add(32)), a2);
            a3 = _mm512_fmadd_ps(widen(p.add(48)), _mm512_loadu_ps(q.add(48)), a3);
        }
        let acc = _mm512_add_ps(_mm512_add_ps(a0, a1), _mm512_add_ps(a2, a3));
        let mut total = _mm512_reduce_add_ps(acc);
        for i in chunks * 64..n {
            total += f32::from_bits((*w.add(i) as u32) << 16) * *x.add(i);
        }
        total
    }

    /// # Safety
    /// Requires `avx2` and `fma`.
    #[target_feature(enable = "avx2,fma")]
    pub unsafe fn dot_bf16_avx2(w: *const u16, x: *const f32, n: usize) -> f32 {
        #[inline]
        unsafe fn widen(p: *const u16) -> __m256 {
            let raw = _mm_loadu_si128(p as *const __m128i);
            _mm256_castsi256_ps(_mm256_slli_epi32(_mm256_cvtepu16_epi32(raw), 16))
        }
        let (mut a0, mut a1) = (_mm256_setzero_ps(), _mm256_setzero_ps());
        let (mut a2, mut a3) = (_mm256_setzero_ps(), _mm256_setzero_ps());
        let chunks = n / 32;
        for i in 0..chunks {
            let (p, q) = (w.add(i * 32), x.add(i * 32));
            a0 = _mm256_fmadd_ps(widen(p), _mm256_loadu_ps(q), a0);
            a1 = _mm256_fmadd_ps(widen(p.add(8)), _mm256_loadu_ps(q.add(8)), a1);
            a2 = _mm256_fmadd_ps(widen(p.add(16)), _mm256_loadu_ps(q.add(16)), a2);
            a3 = _mm256_fmadd_ps(widen(p.add(24)), _mm256_loadu_ps(q.add(24)), a3);
        }
        let acc = _mm256_add_ps(_mm256_add_ps(a0, a1), _mm256_add_ps(a2, a3));
        let lo = _mm256_castps256_ps128(acc);
        let sm = _mm_add_ps(lo, _mm256_extractf128_ps(acc, 1));
        let sm = _mm_add_ps(sm, _mm_movehl_ps(sm, sm));
        let sm = _mm_add_ss(sm, _mm_shuffle_ps(sm, sm, 1));
        let mut total = _mm_cvtss_f32(sm);
        for i in chunks * 32..n {
            total += f32::from_bits((*w.add(i) as u32) << 16) * *x.add(i);
        }
        total
    }

    /// Indistinguishable from AVX2 at one thread, where a core saturates around
    /// 40 GB/s either way; 1.34x at four, where AVX2 cannot issue fast enough.
    ///
    /// # Safety
    /// Requires `avx512f`. Lengths are checked by the safe caller.
    #[target_feature(enable = "avx512f")]
    pub unsafe fn dot_avx512(a: *const f32, b: *const f32, n: usize) -> f32 {
        let (mut a0, mut a1) = (_mm512_setzero_ps(), _mm512_setzero_ps());
        let (mut a2, mut a3) = (_mm512_setzero_ps(), _mm512_setzero_ps());
        let chunks = n / 64;
        for i in 0..chunks {
            let p = a.add(i * 64);
            let q = b.add(i * 64);
            a0 = _mm512_fmadd_ps(_mm512_loadu_ps(p), _mm512_loadu_ps(q), a0);
            a1 = _mm512_fmadd_ps(_mm512_loadu_ps(p.add(16)), _mm512_loadu_ps(q.add(16)), a1);
            a2 = _mm512_fmadd_ps(_mm512_loadu_ps(p.add(32)), _mm512_loadu_ps(q.add(32)), a2);
            a3 = _mm512_fmadd_ps(_mm512_loadu_ps(p.add(48)), _mm512_loadu_ps(q.add(48)), a3);
        }
        let acc = _mm512_add_ps(_mm512_add_ps(a0, a1), _mm512_add_ps(a2, a3));
        let mut total = _mm512_reduce_add_ps(acc);
        for i in chunks * 64..n {
            total += *a.add(i) * *b.add(i);
        }
        total
    }

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

/// One row's dot product.
///
/// `unsafe` because the ISA-specific implementations require their target
/// features; the safe wrapper establishes every other precondition.
pub type F32DotFn = unsafe fn(*const f32, *const f32, usize) -> f32;

/// # Safety
/// Always safe; the signature matches [`F32DotFn`] so the portable path can sit
/// in the same dispatch table.
unsafe fn dot_scalar(a: *const f32, b: *const f32, n: usize) -> f32 {
    let mut total = 0.0f32;
    for i in 0..n {
        total += *a.add(i) * *b.add(i);
    }
    total
}

/// Every dense kernel this build contains, in preference order.
///
/// Listed unconditionally per architecture so `available_f32_kernels` can report
/// what the *CPU* supports separately from what the *build* contains -- the same
/// arrangement the ternary dispatch uses.
fn all_f32_kernels() -> Vec<(&'static str, F32DotFn, bool)> {
    let mut v: Vec<(&'static str, F32DotFn, bool)> = Vec::new();
    #[cfg(target_arch = "x86_64")]
    {
        v.push((
            "avx512f",
            x86::dot_avx512 as F32DotFn,
            is_x86_feature_detected!("avx512f"),
        ));
        v.push((
            "avx2",
            x86::dot as F32DotFn,
            is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma"),
        ));
    }
    #[cfg(target_arch = "aarch64")]
    {
        // NEON is baseline on aarch64.
        v.push(("neon", arm::dot as F32DotFn, true));
    }
    v.push(("scalar", dot_scalar as F32DotFn, true));
    v
}

/// Names of the dense kernels this build contains that this CPU can run.
pub fn available_f32_kernels() -> Vec<&'static str> {
    all_f32_kernels()
        .into_iter()
        .filter(|k| k.2)
        .map(|k| k.0)
        .collect()
}

fn lookup_f32(name: &str) -> Result<(&'static str, F32DotFn), crate::UnsupportedKernel> {
    all_f32_kernels()
        .into_iter()
        .find(|k| k.0 == name && k.2)
        .map(|k| (k.0, k.1))
        .ok_or_else(|| crate::UnsupportedKernel {
            requested: name.to_string(),
            available: available_f32_kernels(),
        })
}

static F32_KERNEL: OnceLock<(&'static str, F32DotFn)> = OnceLock::new();

fn resolve_f32() -> (&'static str, F32DotFn) {
    *F32_KERNEL.get_or_init(|| {
        if let Ok(name) = std::env::var("TRIT_F32_KERNEL") {
            // Fatal for the same reason as the ternary side: on an AVX-512
            // runner the AVX2 path is otherwise never executed, and a silent
            // fallback would let CI report green while testing one kernel twice.
            return lookup_f32(&name).unwrap_or_else(|e| panic!("TRIT_F32_KERNEL: {e}"));
        }
        let k = all_f32_kernels();
        let best = k.iter().find(|x| x.2).expect("scalar is always available");
        (best.0, best.1)
    })
}

/// Pin the dense kernel for the process. Errors -- never falls back.
///
/// As with the ternary side, the choice is cached for the process. Requesting
/// the kernel already in force succeeds; requesting a different one after
/// something has already resolved is an error, not a silent no-op.
pub fn force_f32_kernel(name: &str) -> Result<&'static str, crate::UnsupportedKernel> {
    let (n, f) = lookup_f32(name)?;
    let _ = F32_KERNEL.set((n, f));
    let in_force = resolve_f32().0;
    if in_force != n {
        return Err(crate::UnsupportedKernel {
            requested: format!("{name} (already resolved to {in_force} earlier in this process)"),
            available: available_f32_kernels(),
        });
    }
    Ok(in_force)
}

/// The dense kernel in force.
pub fn f32_kernel_name() -> &'static str {
    resolve_f32().0
}

/// One row's bf16-weight dot product against f32 activations.
pub type Bf16DotFn = unsafe fn(*const u16, *const f32, usize) -> f32;

/// # Safety
/// Always safe; the signature matches [`Bf16DotFn`].
unsafe fn dot_bf16_scalar(w: *const u16, x: *const f32, n: usize) -> f32 {
    let mut total = 0.0f32;
    for i in 0..n {
        total += trit_core::backend::bf16_to_f32(*w.add(i)) * *x.add(i);
    }
    total
}

/// Selected by the same name as the f32 kernel, so forcing one forces both.
fn bf16_dot_for(name: &str) -> Bf16DotFn {
    match name {
        #[cfg(target_arch = "x86_64")]
        "avx512f" => x86::dot_bf16_avx512 as Bf16DotFn,
        #[cfg(target_arch = "x86_64")]
        "avx2" => x86::dot_bf16_avx2 as Bf16DotFn,
        _ => dot_bf16_scalar as Bf16DotFn,
    }
}

/// Dense matvec over weights in whatever precision the file stores them.
pub fn dense_matvec(
    w: trit_core::backend::DenseWeights<'_>,
    rows: usize,
    cols: usize,
    x: &[f32],
    y: &mut [f32],
    threads: usize,
) {
    assert_eq!(w.len(), rows * cols);
    assert_eq!(x.len(), cols);
    assert_eq!(y.len(), rows);

    let (name, f32_dot) = resolve_f32();
    let bf16_dot = bf16_dot_for(name);

    // One closure per precision: no per-row branch.
    let row: &(dyn Fn(usize) -> f32 + Sync) = match w {
        // SAFETY: r < rows so the row lies inside w, cols matches x, and the
        // dispatch table only yields kernels this CPU supports.
        trit_core::backend::DenseWeights::F32(w) => {
            &move |r: usize| unsafe { f32_dot(w.as_ptr().add(r * cols), x.as_ptr(), cols) }
        }
        trit_core::backend::DenseWeights::Bf16(w) => {
            &move |r: usize| unsafe { bf16_dot(w.as_ptr().add(r * cols), x.as_ptr(), cols) }
        }
    };

    let pool = (threads > 1)
        .then(|| crate::pool::global(threads))
        .flatten();
    let slots = pool
        .map(|_| crate::slots_for(w.bytes(), threads, rows))
        .unwrap_or(1);
    let (Some(pool), true) = (pool, slots > 1) else {
        for (r, out) in y.iter_mut().enumerate() {
            *out = row(r);
        }
        return;
    };
    let chunk = rows.div_ceil(slots).max(1);
    pool.run(y, chunk, &|slot, out| {
        let base = slot * chunk;
        for (j, o) in out.iter_mut().enumerate() {
            *o = row(base + j);
        }
    });
}

/// Dispatching dense f32 matvec. Rows are independent, so this parallelizes and
/// vectorizes without changing the summation order within any single row.
pub fn f32_matvec(w: &[f32], rows: usize, cols: usize, x: &[f32], y: &mut [f32], threads: usize) {
    assert_eq!(w.len(), rows * cols);
    assert_eq!(x.len(), cols);
    assert_eq!(y.len(), rows);

    let (_, dot) = resolve_f32();

    let row = |r: usize| -> f32 {
        // SAFETY: r < rows, so the row lies inside w; cols matches x. The
        // dispatch table only ever yields kernels whose target features this CPU
        // was detected to support.
        unsafe { dot(w.as_ptr().add(r * cols), x.as_ptr(), cols) }
    };

    // The pool the ternary path uses. Two pools contend for the same cores, and
    // these matvecs alternate all decode.
    let pool = (threads > 1)
        .then(|| crate::pool::global(threads))
        .flatten();
    let Some(pool) = pool else {
        for (r, out) in y.iter_mut().enumerate() {
            *out = row(r);
        }
        return;
    };
    let slots = crate::slots_for(rows * cols * 4, threads, rows);
    if slots < 2 {
        for (r, out) in y.iter_mut().enumerate() {
            *out = row(r);
        }
        return;
    }
    let chunk = rows.div_ceil(slots).max(1);
    pool.run(y, chunk, &|slot, out| {
        let base = slot * chunk;
        for (j, o) in out.iter_mut().enumerate() {
            *o = row(base + j);
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
            assert!(
                (p - q).abs() <= 1e-4 * p.abs().max(1.0),
                "row {i}: {p} vs {q}"
            );
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
