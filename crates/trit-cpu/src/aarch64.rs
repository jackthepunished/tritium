//! aarch64 kernels: baseline NEON, and NEON with the `dotprod` extension.
//!
//! NEON has no mask registers, so the 64-bit plane words are expanded to byte
//! lanes of `0x00`/`0xFF` first -- the same shape as the AVX2 path. `vtstq_u8`
//! does the test-and-materialize in one instruction where AVX2 needs an AND plus
//! a compare.
//!
//! As on x86, the two planes are accumulated separately and subtracted only
//! after widening, so no `i8` is ever negated and `x = -128` is exact. See the
//! module docs in `x86.rs` for why that matters.
//!
//! # Verification status
//!
//! These kernels are **compile-verified only** in the environment they were
//! written in (an x86_64 host with no aarch64 emulator available). They type
//! check against the aarch64 target, but neither has been executed. The
//! differential suite in `tests/differential.rs` is what proves them, and it
//! must be run on real aarch64 hardware -- or under `qemu-aarch64 -cpu max` for
//! `dotprod` and `-cpu cortex-a55` for the baseline path -- before either is
//! trusted. Do not quote aarch64 performance numbers until that has happened.

use std::arch::aarch64::*;
use trit_core::planes::{BEAT_BYTES, LANES};

/// Bit selector for a 16-byte group: byte `j` tests bit `j % 8` of the mask byte
/// it was given.
const BIT_SEL: [u8; 16] = [1, 2, 4, 8, 16, 32, 64, 128, 1, 2, 4, 8, 16, 32, 64, 128];

/// Expand 16 mask bits (two bytes) to 16 lanes of `0x00` / `0xFF`.
#[inline]
#[target_feature(enable = "neon")]
unsafe fn spread16(mask: u64, group: usize) -> uint8x16_t {
    let b0 = (mask >> (group * 16)) as u8;
    let b1 = (mask >> (group * 16 + 8)) as u8;
    let spread = vcombine_u8(vdup_n_u8(b0), vdup_n_u8(b1));
    // vtstq is a bitwise AND followed by "!= 0", giving 0xFF where the bit is
    // set -- one instruction instead of AND + CMPEQ.
    vtstq_u8(spread, vld1q_u8(BIT_SEL.as_ptr()))
}

/// Baseline NEON. 16 columns per step, four steps per beat.
///
/// Accumulates in `i16` and flushes to `i32` every 64 beats. Each
/// `vpadalq_s8` step adds at most `2 * 127 = 254` per `i16` lane, so 128
/// accumulations is the true overflow bound; 64 leaves a factor of two of
/// headroom and keeps the flush on a power-of-two boundary.
///
/// # Safety
/// The current CPU must support `neon` (baseline on aarch64). Slice lengths are
/// established by the safe caller in `lib.rs`.
#[target_feature(enable = "neon")]
pub unsafe fn matvec(beats: &[u8], rows: usize, beats_per_row: usize, xq: &[i8], y: &mut [i32]) {
    const FLUSH: usize = 64;
    let stride = beats_per_row * BEAT_BYTES;

    for r in 0..rows {
        let row = beats.as_ptr().add(r * stride);
        let mut acc32 = vdupq_n_s32(0);
        let mut acc_p16 = vdupq_n_s16(0);
        let mut acc_n16 = vdupq_n_s16(0);

        for b in 0..beats_per_row {
            let o = b * BEAT_BYTES;
            let pos = u64::from_le_bytes(*(row.add(o) as *const [u8; 8]));
            let neg = u64::from_le_bytes(*(row.add(o + 8) as *const [u8; 8]));
            if pos | neg == 0 {
                continue;
            }
            let xbase = xq.as_ptr().add(b * LANES);
            for g in 0..4 {
                let x = vld1q_s8(xbase.add(g * 16));
                let pm = vreinterpretq_s8_u8(spread16(pos, g));
                let nm = vreinterpretq_s8_u8(spread16(neg, g));
                // Pairwise-add the masked bytes into the i16 accumulators.
                acc_p16 = vpadalq_s8(acc_p16, vandq_s8(x, pm));
                acc_n16 = vpadalq_s8(acc_n16, vandq_s8(x, nm));
            }
            if (b + 1) % FLUSH == 0 {
                acc32 = vpadalq_s16(acc32, acc_p16);
                acc32 = vsubq_s32(acc32, vpaddlq_s16(acc_n16));
                acc_p16 = vdupq_n_s16(0);
                acc_n16 = vdupq_n_s16(0);
            }
        }
        acc32 = vpadalq_s16(acc32, acc_p16);
        acc32 = vsubq_s32(acc32, vpaddlq_s16(acc_n16));
        y[r] = vaddvq_s32(acc32);
    }
}

/// `sdot Vd.4s, Vn.16b, Vm.16b` -- signed dot product of four byte groups,
/// accumulating into four `i32` lanes.
///
/// Emitted as inline asm rather than through `vdotq_s32`, because that intrinsic
/// is still unstable on stable Rust (`stdarch_neon_dotprod`, rust#117224) and
/// this crate builds on stable. The instruction itself is Armv8.2-A `dotprod`,
/// which the dispatcher feature-detects before selecting this kernel.
///
/// # Safety
/// The current CPU must support the `dotprod` extension.
#[inline]
#[target_feature(enable = "neon")]
unsafe fn sdot(acc: int32x4_t, a: int8x16_t, b: int8x16_t) -> int32x4_t {
    let mut out = acc;
    std::arch::asm!(
        "sdot {out:v}.4s, {a:v}.16b, {b:v}.16b",
        out = inout(vreg) out,
        a = in(vreg) a,
        b = in(vreg) b,
        options(pure, nomem, nostack)
    );
    out
}

/// NEON with `dotprod` (Armv8.2-A). One `sdot` against a vector of ones folds
/// the widening and the accumulate into a single instruction, so a 16-lane group
/// costs a mask expand, an AND and a dot-product -- and the `i16` overflow
/// bookkeeping disappears entirely.
///
/// # Safety
/// The current CPU must support `neon` and the `dotprod` extension. Slice
/// lengths are established by the safe caller.
#[target_feature(enable = "neon")]
pub unsafe fn matvec_dotprod(
    beats: &[u8],
    rows: usize,
    beats_per_row: usize,
    xq: &[i8],
    y: &mut [i32],
) {
    let ones = vdupq_n_s8(1);
    let stride = beats_per_row * BEAT_BYTES;

    for r in 0..rows {
        let row = beats.as_ptr().add(r * stride);
        let mut acc_p = vdupq_n_s32(0);
        let mut acc_n = vdupq_n_s32(0);

        for b in 0..beats_per_row {
            let o = b * BEAT_BYTES;
            let pos = u64::from_le_bytes(*(row.add(o) as *const [u8; 8]));
            let neg = u64::from_le_bytes(*(row.add(o + 8) as *const [u8; 8]));
            if pos | neg == 0 {
                continue;
            }
            let xbase = xq.as_ptr().add(b * LANES);
            for g in 0..4 {
                let x = vld1q_s8(xbase.add(g * 16));
                let pm = vreinterpretq_s8_u8(spread16(pos, g));
                let nm = vreinterpretq_s8_u8(spread16(neg, g));
                acc_p = sdot(acc_p, vandq_s8(x, pm), ones);
                acc_n = sdot(acc_n, vandq_s8(x, nm), ones);
            }
        }
        y[r] = vaddvq_s32(acc_p) - vaddvq_s32(acc_n);
    }
}
