//! Reference ternary matvec. `trit-cpu` provides the SIMD implementations; these
//! are the portable definitions every one of them is diffed against.

use crate::planes::TritPlanes;

/// The ternary hot path: select-accumulate only, no multiplies on weights.
/// `trits` is row-major (rows x cols); returns i32 accumulators per row.
pub fn ternary_matvec(trits: &[i8], rows: usize, cols: usize, xq: &[i8]) -> Vec<i32> {
    assert_eq!(trits.len(), rows * cols);
    assert_eq!(xq.len(), cols);
    let mut y = vec![0i32; rows];
    for r in 0..rows {
        let row = &trits[r * cols..(r + 1) * cols];
        let mut acc = 0i32;
        for c in 0..cols {
            match row[c] {
                1 => acc += xq[c] as i32,
                -1 => acc -= xq[c] as i32,
                _ => {}
            }
        }
        y[r] = acc;
    }
    y
}

/// Planar reference: `y[r] = sum(x[c] for c in pos) - sum(x[c] for c in neg)`.
///
/// `xq` must be at least `planes.padded_cols()` long, with zeros in the padding.
/// Padded columns are clear in both planes, so they contribute nothing.
///
/// The two sums are accumulated separately and subtracted in `i32`. No `i8` is
/// ever negated -- `-(-128)` is not representable in `i8`, and the golden vector
/// set `extremes_4x128` feeds exactly that value. The RTL core sign-extends to
/// 32 bits before negating, so this matches it; an i8-domain negation would not.
///
/// Both sums fit `i32` comfortably: the widest tensor in BitNet 2B4T is 6912
/// columns, bounding each at `6912 * 127 = 877_824`. Integer addition is exact
/// and order-independent, so any reordering -- SIMD lanes, threads, the RTL's
/// beat-serial accumulation -- produces bit-identical results. That is what lets
/// every differential test demand exact equality rather than a tolerance.
pub fn ternary_matvec_planes(planes: &TritPlanes<'_>, xq: &[i8], y: &mut [i32]) {
    assert!(
        xq.len() >= planes.padded_cols(),
        "activation buffer is {} long, need {} (padded to a whole beat)",
        xq.len(),
        planes.padded_cols()
    );
    assert_eq!(y.len(), planes.rows(), "output length must match row count");

    for (r, out) in y.iter_mut().enumerate() {
        let (mut acc_pos, mut acc_neg) = (0i32, 0i32);
        for b in 0..planes.beats_per_row() {
            let (mut pos, mut neg) = planes.beat(r, b);
            let base = b * crate::planes::LANES;
            // Iterating set bits rather than all 64 columns skips the zeros for
            // free. The real checkpoint measures 37.7% zeros.
            while pos != 0 {
                acc_pos += xq[base + pos.trailing_zeros() as usize] as i32;
                pos &= pos - 1;
            }
            while neg != 0 {
                acc_neg += xq[base + neg.trailing_zeros() as usize] as i32;
                neg &= neg - 1;
            }
        }
        *out = acc_pos - acc_neg;
    }
}

/// Convenience wrapper allocating the output.
pub fn ternary_matvec_planes_vec(planes: &TritPlanes<'_>, xq: &[i8]) -> Vec<i32> {
    let mut y = vec![0i32; planes.rows()];
    ternary_matvec_planes(planes, xq, &mut y);
    y
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planes::{pack_planes, TritPlanes};

    fn padded(x: &[i8], cols: usize) -> Vec<i8> {
        let mut v = vec![0i8; crate::planes::beats_per_row(cols) * crate::planes::LANES];
        v[..x.len()].copy_from_slice(x);
        v
    }

    #[test]
    fn hand_computed_2x3() {
        // W = [[1,-1,0],[0,1,1]], x = [10,20,30]
        // y = [10-20, 20+30] = [-10, 50]
        let w: Vec<i8> = vec![1, -1, 0, 0, 1, 1];
        assert_eq!(ternary_matvec(&w, 2, 3, &[10, 20, 30]), vec![-10, 50]);
    }

    #[test]
    fn matches_float_reference() {
        // vs naive integer matmul of the same ternary weights
        let w: Vec<i8> = vec![1, 0, -1, -1, 1, 0, 0, 0, 1];
        let x: Vec<i8> = vec![-128i16 as i8, 127, 3]; // extremes are fine in i32 acc
        let y = ternary_matvec(&w, 3, 3, &x);
        for r in 0..3 {
            let expect: i32 = (0..3).map(|c| w[r * 3 + c] as i32 * x[c] as i32).sum();
            assert_eq!(y[r], expect);
        }
    }

    #[test]
    fn planar_matches_the_scalar_reference_on_the_hand_case() {
        let w: Vec<i8> = vec![1, -1, 0, 0, 1, 1];
        let beats = pack_planes(&w, 2, 3).unwrap();
        let p = TritPlanes::new(&beats, 2, 3, 1.0).unwrap();
        assert_eq!(ternary_matvec_planes_vec(&p, &padded(&[10, 20, 30], 3)), vec![-10, 50]);
    }

    #[test]
    fn planar_matches_scalar_across_shapes_including_minus_128() {
        let mut state = 0x5EEDu64;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        for &(rows, cols) in &[
            (1, 64),
            (3, 64),
            (2, 100),
            (5, 129),
            (7, 640),
            (2, 2560),
            (2, 6912),
            (16, 61),
        ] {
            let trits: Vec<i8> =
                (0..rows * cols).map(|_| [-1i8, 0, 1][(next() % 3) as usize]).collect();
            // full i8 range, so -128 appears
            let x: Vec<i8> = (0..cols).map(|_| (next() & 0xff) as u8 as i8).collect();
            let beats = pack_planes(&trits, rows, cols).unwrap();
            let p = TritPlanes::new(&beats, rows, cols, 1.0).unwrap();
            assert_eq!(
                ternary_matvec_planes_vec(&p, &padded(&x, cols)),
                ternary_matvec(&trits, rows, cols, &x),
                "shape {rows}x{cols}"
            );
        }
    }

    #[test]
    fn minus_128_against_a_negative_weight_is_exact() {
        // The trap an i8-domain negation falls into: -(-128) is not
        // representable in i8 and wraps back to -128. Expect +128.
        let beats = pack_planes(&[-1], 1, 1).unwrap();
        let p = TritPlanes::new(&beats, 1, 1, 1.0).unwrap();
        assert_eq!(ternary_matvec_planes_vec(&p, &padded(&[-128], 1)), vec![128]);
    }

    #[test]
    fn worst_case_accumulator_stays_well_inside_i32() {
        // Widest real tensor, every weight nonzero, every activation extreme.
        let cols = 6912;
        let trits: Vec<i8> = vec![-1; cols];
        let beats = pack_planes(&trits, 1, cols).unwrap();
        let p = TritPlanes::new(&beats, 1, cols, 1.0).unwrap();
        let y = ternary_matvec_planes_vec(&p, &padded(&vec![-128i8; cols], cols));
        assert_eq!(y[0], 6912 * 128);
        assert!(y[0] < i32::MAX / 2);
    }
}
