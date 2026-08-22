//! Bit-serial popcount kernel.
//!
//! Decomposes the int8 activations into 8 bit planes and accumulates with
//! `popcount`:
//!
//! ```text
//! x[c] = sum over b of  xp_b[c] * w_b      w_b = 2^b, and w_7 = -2^7
//!
//! y[r] = sum over b of  w_b * ( popcount(pos & xp_b) - popcount(neg & xp_b) )
//! ```
//!
//! This is the textbook popcount formulation, and it is the one that maps
//! directly onto the RTL adder tree. It is **not** the production kernel, and
//! this module is behind a non-default feature for that reason.
//!
//! Honest accounting at int8 activations: a 64-column beat costs 8 planes x 2
//! masks x (AND + popcount + weighted accumulate) ~= 24 operations, against
//! about 8 for the AVX-512 mask-select path -- plus a per-token transpose of the
//! activation vector into bit planes, which batch-1 decoding never amortizes. It
//! is 3-4x slower here.
//!
//! It earns its place two ways. It is a genuinely independent third
//! implementation for the differential tests -- it shares no code with either
//! mask-select kernel, so agreement between them is real evidence. And it
//! becomes the right kernel if activations ever drop below 8 bits, where the
//! plane count falls and mask-select's advantage disappears.

use trit_core::planes::{BEAT_BYTES, LANES};

/// Transpose 64 int8 activations into 8 bit planes.
///
/// Plane `b` has bit `k` set when bit `b` of `x[k]` is set. Bit 7 is the sign
/// bit, weighted `-2^7` by the caller, which is what makes the reconstruction
/// exact in two's complement.
fn transpose_beat(x: &[i8]) -> [u64; 8] {
    debug_assert_eq!(x.len(), LANES);
    let mut planes = [0u64; 8];
    for (k, &v) in x.iter().enumerate() {
        let byte = v as u8;
        for (b, plane) in planes.iter_mut().enumerate() {
            *plane |= (((byte >> b) & 1) as u64) << k;
        }
    }
    planes
}

/// # Safety
/// Always safe; the signature matches `MatvecFn` so it can share the dispatch
/// table.
pub unsafe fn matvec(beats: &[u8], rows: usize, beats_per_row: usize, xq: &[i8], y: &mut [i32]) {
    // The transpose is per activation vector, not per row, so it is hoisted out
    // of the row loop -- otherwise the cost would be multiplied by `rows`.
    let planes: Vec<[u64; 8]> =
        (0..beats_per_row).map(|b| transpose_beat(&xq[b * LANES..(b + 1) * LANES])).collect();

    let stride = beats_per_row * BEAT_BYTES;
    for r in 0..rows {
        let row = &beats[r * stride..(r + 1) * stride];
        let mut acc = 0i64;
        for b in 0..beats_per_row {
            let o = b * BEAT_BYTES;
            let pos = u64::from_le_bytes(row[o..o + 8].try_into().unwrap());
            let neg = u64::from_le_bytes(row[o + 8..o + 16].try_into().unwrap());
            if pos | neg == 0 {
                continue;
            }
            let xp = &planes[b];
            for (bit, &plane) in xp.iter().enumerate() {
                let count =
                    (pos & plane).count_ones() as i64 - (neg & plane).count_ones() as i64;
                if count == 0 {
                    continue;
                }
                // Two's complement: the top bit carries negative weight.
                let weight = if bit == 7 { -(1i64 << 7) } else { 1i64 << bit };
                acc += count * weight;
            }
        }
        y[r] = acc as i32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transpose_reconstructs_every_i8() {
        // Reconstructing from the planes must recover the original value for the
        // whole i8 range, sign bit included.
        for start in [-128i32, -1, 0, 1, 127] {
            let x: Vec<i8> = (0..LANES).map(|i| (start + i as i32).clamp(-128, 127) as i8).collect();
            let planes = transpose_beat(&x);
            for (k, &v) in x.iter().enumerate() {
                let mut got = 0i32;
                for (b, &p) in planes.iter().enumerate() {
                    if (p >> k) & 1 == 1 {
                        got += if b == 7 { -(1 << 7) } else { 1 << b };
                    }
                }
                assert_eq!(got, v as i32, "column {k}");
            }
        }
    }
}
