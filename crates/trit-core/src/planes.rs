//! Bit-planar ternary weight layout.
//!
//! A ternary weight matrix is stored as a stream of 16-byte **beats**. Each beat
//! carries 64 consecutive columns of one row as two 64-bit masks:
//!
//! ```text
//! bytes  0..8   pos  (little-endian u64) -- bit k set => weight +1
//! bytes  8..16  neg  (little-endian u64) -- bit k set => weight -1
//! ```
//!
//! Neither bit set means the weight is 0. **Both bits set is invalid** -- it is
//! the planar analogue of the `0b11` code the v0 layout rejected, and the RTL
//! core raises its sticky `err` on it. Defining it as an error rather than as
//! some value is deliberate: a CPU kernel that accumulates the two planes
//! separately would compute `+x - x = 0`, while a lane that tests `pos` first
//! would compute `+x`. Making it an error keeps the two from silently diverging.
//!
//! Columns past `cols` in the final beat of a row are zero in both planes, so
//! padded terms contribute nothing and a padded matvec is exact.
//!
//! Interleaving the two planes per beat, rather than storing one plane after the
//! other, means one beat is exactly one cycle's weight input for the RTL core:
//! the mmap'd bytes of a tensor *are* its beat stream, in order.

use anyhow::{bail, ensure, Result};

/// Columns carried by one beat. Matches the RTL core's lane count.
pub const LANES: usize = 64;

/// Bytes per beat: two 64-bit planes.
pub const BEAT_BYTES: usize = 16;

/// Beats needed to cover `cols` columns of one row.
#[inline]
pub const fn beats_per_row(cols: usize) -> usize {
    cols.div_ceil(LANES)
}

/// Mask of the columns a row's final beat actually uses. Bits above this must be
/// zero in both planes.
#[inline]
pub const fn tail_mask(cols: usize) -> u64 {
    match cols % LANES {
        0 => u64::MAX,
        r => (1u64 << r) - 1,
    }
}

/// Total payload size of a `rows x cols` ternary tensor.
#[inline]
pub const fn payload_len(rows: usize, cols: usize) -> usize {
    rows * beats_per_row(cols) * BEAT_BYTES
}

/// Pack row-major ternary values into the beat stream.
pub fn pack_planes(trits: &[i8], rows: usize, cols: usize) -> Result<Vec<u8>> {
    ensure!(
        trits.len() == rows.checked_mul(cols).unwrap_or(usize::MAX),
        "pack_planes: got {} trits for a {rows}x{cols} tensor",
        trits.len()
    );
    let bpr = beats_per_row(cols);
    let mut out = vec![0u8; payload_len(rows, cols)];
    for r in 0..rows {
        for b in 0..bpr {
            let (mut pos, mut neg) = (0u64, 0u64);
            let base = b * LANES;
            let end = (base + LANES).min(cols);
            for (k, c) in (base..end).enumerate() {
                match trits[r * cols + c] {
                    0 => {}
                    1 => pos |= 1u64 << k,
                    -1 => neg |= 1u64 << k,
                    other => bail!("non-ternary value {other} at row {r} col {c}"),
                }
            }
            let o = (r * bpr + b) * BEAT_BYTES;
            out[o..o + 8].copy_from_slice(&pos.to_le_bytes());
            out[o + 8..o + 16].copy_from_slice(&neg.to_le_bytes());
        }
    }
    Ok(out)
}

/// Expand a beat stream back to row-major ternary values.
pub fn unpack_planes(beats: &[u8], rows: usize, cols: usize) -> Result<Vec<i8>> {
    TritPlanes::new(beats, rows, cols, 1.0).map(|p| p.to_trits())
}

/// A borrowed view of one ternary tensor's beat stream.
///
/// Construction validates every length relation and the plane invariants, so
/// every accessor below is infallible and every kernel that takes a `TritPlanes`
/// may rely on the layout without re-checking it.
#[derive(Clone, Copy)]
pub struct TritPlanes<'a> {
    beats: &'a [u8],
    rows: usize,
    cols: usize,
    beats_per_row: usize,
    scale: f32,
}

/// Prints the shape rather than the beat stream: these views wrap hundreds of
/// megabytes and a derived `Debug` would be unusable in a test failure message.
impl std::fmt::Debug for TritPlanes<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TritPlanes")
            .field("rows", &self.rows)
            .field("cols", &self.cols)
            .field("beats_per_row", &self.beats_per_row)
            .field("scale", &self.scale)
            .field("bytes", &self.beats.len())
            .finish()
    }
}

impl<'a> TritPlanes<'a> {
    /// Validate and wrap a beat stream. Rejects wrong lengths, overlapping
    /// planes, and padding bits set past `cols`.
    pub fn new(beats: &'a [u8], rows: usize, cols: usize, scale: f32) -> Result<Self> {
        ensure!(cols > 0, "ternary tensor must have at least one column");
        let want = payload_len(rows, cols);
        ensure!(
            beats.len() == want,
            "beat stream is {} bytes, expected {want} for {rows}x{cols}",
            beats.len()
        );
        let p = Self { beats, rows, cols, beats_per_row: beats_per_row(cols), scale };
        p.check_invariants()?;
        Ok(p)
    }

    /// Wrap a beat stream whose invariants have already been checked.
    ///
    /// Used on the load path for large tensors where the full scan is deferred
    /// to `tritd verify`. Lengths are still validated; only the per-beat scan is
    /// skipped.
    pub fn new_unchecked_bits(beats: &'a [u8], rows: usize, cols: usize, scale: f32) -> Result<Self> {
        ensure!(cols > 0, "ternary tensor must have at least one column");
        let want = payload_len(rows, cols);
        ensure!(
            beats.len() == want,
            "beat stream is {} bytes, expected {want} for {rows}x{cols}",
            beats.len()
        );
        Ok(Self { beats, rows, cols, beats_per_row: beats_per_row(cols), scale })
    }

    #[inline]
    pub fn rows(&self) -> usize {
        self.rows
    }
    #[inline]
    pub fn cols(&self) -> usize {
        self.cols
    }
    #[inline]
    pub fn beats_per_row(&self) -> usize {
        self.beats_per_row
    }
    /// Column count rounded up to a whole number of beats. Activation buffers
    /// are sized to this so the kernel never needs a scalar tail loop.
    #[inline]
    pub fn padded_cols(&self) -> usize {
        self.beats_per_row * LANES
    }
    #[inline]
    pub fn scale(&self) -> f32 {
        self.scale
    }
    #[inline]
    pub fn as_bytes(&self) -> &'a [u8] {
        self.beats
    }

    /// The `(pos, neg)` masks of one beat.
    ///
    /// Reads through `from_le_bytes` rather than casting to `&[u64]`: on x86 and
    /// aarch64 this is a single unaligned load, it needs no `unsafe`, and it is
    /// correct on big-endian hosts.
    #[inline]
    pub fn beat(&self, row: usize, beat: usize) -> (u64, u64) {
        debug_assert!(row < self.rows && beat < self.beats_per_row);
        let o = (row * self.beats_per_row + beat) * BEAT_BYTES;
        let pos = u64::from_le_bytes(self.beats[o..o + 8].try_into().unwrap());
        let neg = u64::from_le_bytes(self.beats[o + 8..o + 16].try_into().unwrap());
        (pos, neg)
    }

    /// The raw beats of one row.
    #[inline]
    pub fn row_bytes(&self, row: usize) -> &'a [u8] {
        debug_assert!(row < self.rows);
        let stride = self.beats_per_row * BEAT_BYTES;
        &self.beats[row * stride..(row + 1) * stride]
    }

    /// Expand to one `i8` per weight. The oracle path; the production path never
    /// calls this.
    pub fn to_trits(&self) -> Vec<i8> {
        let mut out = vec![0i8; self.rows * self.cols];
        for r in 0..self.rows {
            self.row_trits_into(r, &mut out[r * self.cols..(r + 1) * self.cols]);
        }
        out
    }

    /// Expand one row into `out`, which must be exactly `cols` long.
    pub fn row_trits_into(&self, row: usize, out: &mut [i8]) {
        assert_eq!(out.len(), self.cols);
        for b in 0..self.beats_per_row {
            let (mut pos, mut neg) = self.beat(row, b);
            let base = b * LANES;
            while pos != 0 {
                let k = pos.trailing_zeros() as usize;
                pos &= pos - 1;
                if base + k < self.cols {
                    out[base + k] = 1;
                }
            }
            while neg != 0 {
                let k = neg.trailing_zeros() as usize;
                neg &= neg - 1;
                if base + k < self.cols {
                    out[base + k] = -1;
                }
            }
        }
    }

    /// Number of zero weights, by popcount. Exact and O(beats) -- this is where
    /// popcount actually pays on the weights.
    pub fn zero_count(&self) -> u64 {
        let mut nonzero = 0u64;
        for r in 0..self.rows {
            for b in 0..self.beats_per_row {
                let (pos, neg) = self.beat(r, b);
                nonzero += u64::from(pos.count_ones()) + u64::from(neg.count_ones());
            }
        }
        (self.rows as u64) * (self.cols as u64) - nonzero
    }

    /// Full scan of both plane invariants: planes disjoint, padding bits clear.
    pub fn check_invariants(&self) -> Result<()> {
        let tail = tail_mask(self.cols);
        let last = self.beats_per_row - 1;
        for r in 0..self.rows {
            for b in 0..self.beats_per_row {
                let (pos, neg) = self.beat(r, b);
                let overlap = pos & neg;
                if overlap != 0 {
                    bail!(
                        "row {r} beat {b}: pos & neg overlap at column {}",
                        b * LANES + overlap.trailing_zeros() as usize
                    );
                }
                if b == last && (pos | neg) & !tail != 0 {
                    bail!("row {r} beat {b}: padding bits set past column {}", self.cols);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beat_layout_is_hand_computable() {
        // 1 row, 6 cols: [+1, -1, 0, +1, 0, -1]
        // pos bits 0 and 3 -> 0x09 ; neg bits 1 and 5 -> 0x22
        let beats = pack_planes(&[1, -1, 0, 1, 0, -1], 1, 6).unwrap();
        assert_eq!(beats.len(), BEAT_BYTES, "6 cols is one beat");
        assert_eq!(&beats[0..8], &0x09u64.to_le_bytes());
        assert_eq!(&beats[8..16], &0x22u64.to_le_bytes());
    }

    #[test]
    fn roundtrips_including_partial_and_multi_beat_rows() {
        let mut state = 0xA11CEu64;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        // widths either side of the 64-column beat boundary, plus real model widths
        for &(rows, cols) in &[
            (1, 1),
            (1, 63),
            (1, 64),
            (1, 65),
            (3, 100),
            (5, 129),
            (2, 2560),
            (2, 6912),
            (16, 61),
        ] {
            let trits: Vec<i8> =
                (0..rows * cols).map(|_| [-1i8, 0, 1][(next() % 3) as usize]).collect();
            let beats = pack_planes(&trits, rows, cols).unwrap();
            assert_eq!(beats.len(), payload_len(rows, cols));
            assert_eq!(unpack_planes(&beats, rows, cols).unwrap(), trits, "{rows}x{cols}");
        }
    }

    #[test]
    fn padding_bits_are_clear_so_padded_terms_contribute_nothing() {
        let trits: Vec<i8> = vec![1; 65]; // 2 beats, 63 padding columns
        let beats = pack_planes(&trits, 1, 65).unwrap();
        let p = TritPlanes::new(&beats, 1, 65, 1.0).unwrap();
        let (pos, neg) = p.beat(0, 1);
        assert_eq!(pos, 1, "only column 64 is set in the tail beat");
        assert_eq!(neg, 0);
    }

    #[test]
    fn overlapping_planes_are_rejected() {
        let mut beats = pack_planes(&[1, 0, 0, 0], 1, 4).unwrap();
        beats[8] |= 0b1; // set neg bit 0, which pos already holds
        let err = TritPlanes::new(&beats, 1, 4, 1.0).unwrap_err().to_string();
        assert!(err.contains("overlap"), "{err}");
    }

    #[test]
    fn padding_bits_set_are_rejected() {
        let mut beats = pack_planes(&[1, 0, 0, 0], 1, 4).unwrap();
        beats[0] |= 0b1000_0000; // column 7, past cols=4
        let err = TritPlanes::new(&beats, 1, 4, 1.0).unwrap_err().to_string();
        assert!(err.contains("padding bits"), "{err}");
    }

    #[test]
    fn wrong_length_is_rejected() {
        let beats = pack_planes(&[1, 0, 0, 0], 1, 4).unwrap();
        assert!(TritPlanes::new(&beats[..8], 1, 4, 1.0).is_err());
        assert!(TritPlanes::new(&beats, 2, 4, 1.0).is_err());
    }

    #[test]
    fn non_ternary_input_is_rejected() {
        assert!(pack_planes(&[2, 0], 1, 2).is_err());
        assert!(pack_planes(&[1, 0], 1, 3).is_err(), "length mismatch");
    }

    #[test]
    fn zero_count_matches_a_direct_count() {
        let trits: Vec<i8> = vec![0, 1, -1, 0, 0, 1, 0, -1, 0, 0];
        let beats = pack_planes(&trits, 2, 5).unwrap();
        let p = TritPlanes::new(&beats, 2, 5, 1.0).unwrap();
        let expect = trits.iter().filter(|&&t| t == 0).count() as u64;
        assert_eq!(p.zero_count(), expect);
        assert_eq!(p.zero_count(), 6);
    }

    #[test]
    fn tail_mask_covers_exactly_the_used_columns() {
        assert_eq!(tail_mask(64), u64::MAX);
        assert_eq!(tail_mask(128), u64::MAX);
        assert_eq!(tail_mask(1), 1);
        assert_eq!(tail_mask(65), 1);
        assert_eq!(tail_mask(63), (1u64 << 63) - 1);
    }
}
