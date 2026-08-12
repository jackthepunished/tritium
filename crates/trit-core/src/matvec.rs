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

#[cfg(test)]
mod tests {
    use super::*;

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
}
