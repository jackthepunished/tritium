//! Portable kernel. Correct everywhere, and the reference every SIMD path is
//! diffed against.
//!
//! Iterating set bits rather than scanning all 64 columns skips zero weights for
//! free. The real BitNet b1.58 2B4T checkpoint measures a 0.4219 zero fraction,
//! so this touches roughly 58% of the columns a dense scan would.

use trit_core::planes::{BEAT_BYTES, LANES};

/// `y[r] = sum(x[c] for c in pos) - sum(x[c] for c in neg)`.
///
/// The two sums are kept apart and subtracted in `i32`; no `i8` is ever negated,
/// because `-(-128)` is not representable and the golden vector set
/// `extremes_4x128` feeds exactly that value.
pub fn matvec(beats: &[u8], rows: usize, beats_per_row: usize, xq: &[i8], y: &mut [i32]) {
    let stride = beats_per_row * BEAT_BYTES;
    for r in 0..rows {
        let row = &beats[r * stride..(r + 1) * stride];
        let (mut acc_pos, mut acc_neg) = (0i32, 0i32);
        for b in 0..beats_per_row {
            let o = b * BEAT_BYTES;
            let mut pos = u64::from_le_bytes(row[o..o + 8].try_into().unwrap());
            let mut neg = u64::from_le_bytes(row[o + 8..o + 16].try_into().unwrap());
            let x = &xq[b * LANES..b * LANES + LANES];
            while pos != 0 {
                acc_pos += x[pos.trailing_zeros() as usize] as i32;
                pos &= pos - 1;
            }
            while neg != 0 {
                acc_neg += x[neg.trailing_zeros() as usize] as i32;
                neg &= neg - 1;
            }
        }
        y[r] = acc_pos - acc_neg;
    }
}
