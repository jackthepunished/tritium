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
//! **Correctness:** the `ubuntu-24.04-arm` CI job runs
//! `tests/differential.rs` against each kernel this CPU supports, forced by
//! name -- `force_kernel` errors rather than falling back, so a green run
//! cannot mean the scalar path was silently retested. Both `neon` and
//! `neon-dotprod` match the reference and a naive dense dot product across the
//! original corpus on hardware. New cases with zero beats on flush boundaries
//! exposed an i16 overflow that the all-nonzero rows missed. The corrected
//! kernel passes those cases under ARM emulation; a native CI rerun is pending.
//!
//! That job spent its whole existence failing at clippy before reaching a test,
//! so these kernels went unexecuted for far longer than the CI matrix suggested.
//! Two real defects were sitting in code the matrix claimed to cover. A job that
//! cannot reach its assertions is not coverage.
//!
//! **Performance: still unmeasured.** No aarch64 timing has been taken, on CI
//! or anywhere else. Do not quote an ARM throughput number until one has.

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
/// Accumulates in `i16` and flushes to `i32` every [`FLUSH`] **beats**. The
/// bound is on accumulation *steps*, not beats, and the two are not the same
/// number: the inner loop runs `g in 0..4`, so one beat issues four
/// `vpadalq_s8` per accumulator.
///
/// `vpadalq_s8` pairwise-adds two masked `i8` lanes into each `i16` lane, so a
/// single step moves an accumulator by at most `2 * 128 = 256` -- `-128` is a
/// representable activation and two of them pairwise-add to `-256`. That caps
/// the safe run at `32768 / 256 = 128` steps, which is 32 beats.
///
/// [`STEPS_PER_BEAT`] and [`MAX_STEP`] below turn that argument into a
/// compile-time assertion, so the flush interval can never drift back past the
/// bound in silence. `vpadalq_s8` wraps rather than saturating or trapping, so
/// an overflow here would surface as a quietly wrong logit.
///
/// # Safety
/// The current CPU must support `neon` (baseline on aarch64). Slice lengths are
/// established by the safe caller in `lib.rs`.
#[target_feature(enable = "neon")]
pub unsafe fn matvec(beats: &[u8], rows: usize, beats_per_row: usize, xq: &[i8], y: &mut [i32]) {
    /// `vpadalq_s8` calls per accumulator per beat -- the `g in 0..4` loop.
    const STEPS_PER_BEAT: usize = 4;
    /// Largest magnitude one step can add to an `i16` lane: two `-128` lanes.
    const MAX_STEP: usize = 256;
    /// Beats between flushes to the `i32` accumulator.
    const FLUSH: usize = 16;
    const _: () = assert!(
        FLUSH * STEPS_PER_BEAT * MAX_STEP <= i16::MAX as usize + 1,
        "i16 accumulators can overflow before the flush"
    );
    let stride = beats_per_row * BEAT_BYTES;

    for (r, out) in y.iter_mut().enumerate().take(rows) {
        let row = beats.as_ptr().add(r * stride);
        let mut acc32 = vdupq_n_s32(0);
        let mut acc_p16 = vdupq_n_s16(0);
        let mut acc_n16 = vdupq_n_s16(0);

        for b in 0..beats_per_row {
            let o = b * BEAT_BYTES;
            let pos = u64::from_le_bytes(*(row.add(o) as *const [u8; 8]));
            let neg = u64::from_le_bytes(*(row.add(o + 8) as *const [u8; 8]));
            if pos | neg != 0 {
                let xbase = xq.as_ptr().add(b * LANES);
                for g in 0..4 {
                    let x = vld1q_s8(xbase.add(g * 16));
                    let pm = vreinterpretq_s8_u8(spread16(pos, g));
                    let nm = vreinterpretq_s8_u8(spread16(neg, g));
                    // Pairwise-add the masked bytes into the i16 accumulators.
                    acc_p16 = vpadalq_s8(acc_p16, vandq_s8(x, pm));
                    acc_n16 = vpadalq_s8(acc_n16, vandq_s8(x, nm));
                }
            }
            // A zero beat can fall on a flush boundary. Skipping this flush
            // would let later nonzero beats overflow the i16 accumulators.
            if (b + 1) % FLUSH == 0 {
                acc32 = vpadalq_s16(acc32, acc_p16);
                acc32 = vsubq_s32(acc32, vpaddlq_s16(acc_n16));
                acc_p16 = vdupq_n_s16(0);
                acc_n16 = vdupq_n_s16(0);
            }
        }
        acc32 = vpadalq_s16(acc32, acc_p16);
        acc32 = vsubq_s32(acc32, vpaddlq_s16(acc_n16));
        *out = vaddvq_s32(acc32);
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
/// The `dotprod` target feature is enabled on this function, not just detected
/// at runtime: the instruction is written as inline asm, so it is the
/// *assembler* that has to accept `sdot`, and without the feature on the
/// function the build fails rather than falling back.
///
/// # Safety
/// The current CPU must support the `dotprod` extension.
#[inline]
#[target_feature(enable = "neon", enable = "dotprod")]
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
#[target_feature(enable = "neon", enable = "dotprod")]
pub unsafe fn matvec_dotprod(
    beats: &[u8],
    rows: usize,
    beats_per_row: usize,
    xq: &[i8],
    y: &mut [i32],
) {
    let ones = vdupq_n_s8(1);
    let stride = beats_per_row * BEAT_BYTES;

    for (r, out) in y.iter_mut().enumerate().take(rows) {
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
        *out = vaddvq_s32(acc_p) - vaddvq_s32(acc_n);
    }
}
