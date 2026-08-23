//! x86_64 kernels: AVX2, AVX-512BW, and AVX-512 with VNNI.
//!
//! # Why the two planes are accumulated separately
//!
//! The compact formulation `sel = (x & pos) - (x & neg)` in the **i8 domain is
//! wrong**: for `x = -128` and a `-1` weight it computes `0 - (-128)`, which
//! overflows `i8` and wraps back to `-128` instead of `+128`. The RTL core
//! sign-extends to 32 bits before negating, so it would disagree -- and the
//! golden vector set `extremes_4x128` feeds `-128` deliberately.
//!
//! Every kernel here therefore keeps `sum(x & pos)` and `sum(x & neg)` apart and
//! subtracts only after widening. Nothing negates an `i8`. The cost is one extra
//! accumulator and one extra instruction per beat; the benefit is a kernel with
//! no precondition on the activation values at all.
//!
//! # Accumulator bounds
//!
//! The widest tensor in BitNet b1.58 2B4T is 6912 columns, so each sum is
//! bounded by `6912 * 128 = 884_736` -- three orders of magnitude inside `i32`.
//! Integer addition is exact and order-independent, so lane order, thread order
//! and the RTL's beat-serial order all give bit-identical results.

use std::arch::x86_64::*;
use trit_core::planes::{BEAT_BYTES, LANES};

/// Byte-spread indices for `_mm256_shuffle_epi8`.
///
/// `shuffle_epi8` is **lane-local**: indices address the 16 bytes of their own
/// 128-bit lane, not all 32. Broadcasting the 32-bit mask with `set1_epi32`
/// replicates bytes `b0..b3` inside every dword, so lane 0 reads indices 0 and 1
/// (bytes `b0`, `b1`) and lane 1 reads indices 2 and 3 (bytes `b2`, `b3`).
#[rustfmt::skip]
const SPREAD_IDX: [i8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0,  1, 1, 1, 1, 1, 1, 1, 1,
    2, 2, 2, 2, 2, 2, 2, 2,  3, 3, 3, 3, 3, 3, 3, 3,
];

/// Bit selector: byte `j` tests bit `j % 8` of the mask byte it received.
#[rustfmt::skip]
const BIT_SEL: [i8; 32] = [
    1, 2, 4, 8, 16, 32, 64, -128,  1, 2, 4, 8, 16, 32, 64, -128,
    1, 2, 4, 8, 16, 32, 64, -128,  1, 2, 4, 8, 16, 32, 64, -128,
];

/// Expand 32 mask bits to 32 bytes of `0x00` / `0xFF`.
/// # Safety
/// Requires `avx2`.
#[inline]
#[target_feature(enable = "avx2")]
unsafe fn spread32(mask: u32) -> __m256i {
    let bcast = _mm256_set1_epi32(mask as i32);
    let idx = _mm256_loadu_si256(SPREAD_IDX.as_ptr() as *const __m256i);
    let sel = _mm256_loadu_si256(BIT_SEL.as_ptr() as *const __m256i);
    let spread = _mm256_shuffle_epi8(bcast, idx);
    // Equal to the selector exactly when that single bit survived the AND.
    _mm256_cmpeq_epi8(_mm256_and_si256(spread, sel), sel)
}

/// AVX2. 32 columns per step, two steps per 64-column beat.
///
/// # Safety
/// The current CPU must support `avx2`. Every slice length relation is
/// established by the safe caller in `lib.rs`; this function performs no other
/// unchecked operation.
#[target_feature(enable = "avx2")]
pub unsafe fn matvec(beats: &[u8], rows: usize, beats_per_row: usize, xq: &[i8], y: &mut [i32]) {
    let ones_u8 = _mm256_set1_epi8(1);
    let ones_i16 = _mm256_set1_epi16(1);
    let stride = beats_per_row * BEAT_BYTES;

    for (r, out) in y.iter_mut().enumerate().take(rows) {
        let row = beats.as_ptr().add(r * stride);
        let mut acc = _mm256_setzero_si256();

        for b in 0..beats_per_row {
            let o = b * BEAT_BYTES;
            let pos = u64::from_le_bytes(*(row.add(o) as *const [u8; 8]));
            let neg = u64::from_le_bytes(*(row.add(o + 8) as *const [u8; 8]));
            // Whole beat empty: 42% of them on the real checkpoint carry few
            // enough bits that skipping the fully-empty ones is worth a branch.
            if pos | neg == 0 {
                continue;
            }
            let xbase = xq.as_ptr().add(b * LANES);

            for half in 0..2 {
                let pm = spread32((pos >> (half * 32)) as u32);
                let nm = spread32((neg >> (half * 32)) as u32);
                let x = _mm256_loadu_si256(xbase.add(half * 32) as *const __m256i);

                // maddubs takes the UNSIGNED operand first and the SIGNED second.
                // Reversing them reinterprets the activations as u8 and is
                // silently wrong for every negative value.
                let p16 = _mm256_maddubs_epi16(ones_u8, _mm256_and_si256(x, pm));
                let n16 = _mm256_maddubs_epi16(ones_u8, _mm256_and_si256(x, nm));
                // |p16|, |n16| <= 2*127 = 254, so the difference cannot overflow
                // i16; widening to i32 every step means nothing accumulates here.
                let d16 = _mm256_sub_epi16(p16, n16);
                acc = _mm256_add_epi32(acc, _mm256_madd_epi16(d16, ones_i16));
            }
        }
        *out = hsum_epi32_avx2(acc);
    }
}

/// # Safety
/// Requires `avx2`.
#[inline]
#[target_feature(enable = "avx2")]
unsafe fn hsum_epi32_avx2(v: __m256i) -> i32 {
    let lo = _mm256_castsi256_si128(v);
    let hi = _mm256_extracti128_si256(v, 1);
    let s = _mm_add_epi32(lo, hi);
    let s = _mm_add_epi32(s, _mm_shuffle_epi32(s, 0b01_00_11_10));
    let s = _mm_add_epi32(s, _mm_shuffle_epi32(s, 0b00_01_00_01));
    _mm_cvtsi128_si32(s)
}

/// AVX-512BW. A whole 64-column beat per step, and the plane masks go straight
/// into `k` registers -- no byte-spread at all, which is where AVX2 spends most
/// of its instruction budget.
///
/// # Safety
/// The current CPU must support `avx512f` and `avx512bw`. Slice lengths are
/// established by the safe caller.
#[target_feature(enable = "avx512f,avx512bw")]
pub unsafe fn matvec_avx512bw(
    beats: &[u8],
    rows: usize,
    beats_per_row: usize,
    xq: &[i8],
    y: &mut [i32],
) {
    let ones_u8 = _mm512_set1_epi8(1);
    let ones_i16 = _mm512_set1_epi16(1);
    let stride = beats_per_row * BEAT_BYTES;

    for (r, out) in y.iter_mut().enumerate().take(rows) {
        let row = beats.as_ptr().add(r * stride);
        let mut acc = _mm512_setzero_si512();

        for b in 0..beats_per_row {
            let o = b * BEAT_BYTES;
            let pos = u64::from_le_bytes(*(row.add(o) as *const [u8; 8]));
            let neg = u64::from_le_bytes(*(row.add(o + 8) as *const [u8; 8]));
            if pos | neg == 0 {
                continue;
            }
            let x = _mm512_loadu_si512(xq.as_ptr().add(b * LANES) as *const __m512i);
            let p = _mm512_maskz_mov_epi8(pos, x);
            let n = _mm512_maskz_mov_epi8(neg, x);
            let p16 = _mm512_maddubs_epi16(ones_u8, p);
            let n16 = _mm512_maddubs_epi16(ones_u8, n);
            let d16 = _mm512_sub_epi16(p16, n16);
            acc = _mm512_add_epi32(acc, _mm512_madd_epi16(d16, ones_i16));
        }
        *out = _mm512_reduce_add_epi32(acc);
    }
}

/// AVX-512 with VNNI: `vpdpbusd` folds the multiply-by-one and the widening
/// accumulate into a single instruction, so a 64-column beat costs two masked
/// moves and two dot-products.
///
/// The two planes need separate accumulators here -- `vpdpbusd` cannot subtract
/// -- which is the same structure the `-128` correctness argument already
/// requires, so it costs nothing extra.
///
/// # Safety
/// The current CPU must support `avx512f`, `avx512bw` and `avx512vnni`. Slice
/// lengths are established by the safe caller.
#[target_feature(enable = "avx512f,avx512bw,avx512vnni")]
pub unsafe fn matvec_avx512vnni(
    beats: &[u8],
    rows: usize,
    beats_per_row: usize,
    xq: &[i8],
    y: &mut [i32],
) {
    let ones_u8 = _mm512_set1_epi8(1);
    let stride = beats_per_row * BEAT_BYTES;

    for (r, out) in y.iter_mut().enumerate().take(rows) {
        let row = beats.as_ptr().add(r * stride);
        let mut acc_pos = _mm512_setzero_si512();
        let mut acc_neg = _mm512_setzero_si512();

        for b in 0..beats_per_row {
            let o = b * BEAT_BYTES;
            let pos = u64::from_le_bytes(*(row.add(o) as *const [u8; 8]));
            let neg = u64::from_le_bytes(*(row.add(o + 8) as *const [u8; 8]));
            if pos | neg == 0 {
                continue;
            }
            let x = _mm512_loadu_si512(xq.as_ptr().add(b * LANES) as *const __m512i);
            // Non-saturating dpbusd. The saturating dpbusds would clip silently
            // and is not what the RTL does.
            acc_pos = _mm512_dpbusd_epi32(acc_pos, ones_u8, _mm512_maskz_mov_epi8(pos, x));
            acc_neg = _mm512_dpbusd_epi32(acc_neg, ones_u8, _mm512_maskz_mov_epi8(neg, x));
        }
        *out = _mm512_reduce_add_epi32(acc_pos) - _mm512_reduce_add_epi32(acc_neg);
    }
}
