//! Matvec backend abstraction.
//!
//! The ternary matvec is the one operation that has a CPU implementation, an RTL
//! implementation, and a reference implementation, all of which must agree
//! exactly. This trait is the seam between them.
//!
//! It is injected, not global. A process-wide selector works for a single-shot
//! CLI but breaks as soon as two sessions want different backends, and it makes
//! test ordering matter.

use crate::planes::TritPlanes;

/// `y[r] = sum(x[c] for c in pos) - sum(x[c] for c in neg)`, i32 accumulators.
///
/// Implementations must be **bit-exact** with
/// [`crate::matvec::ternary_matvec_planes`] for every input, including `xq`
/// values of `-128`. Integer addition is exact and order-independent, so this is
/// achievable by construction rather than by tuning; a backend that cannot meet
/// it is wrong, not approximate.
///
/// `xq` may be as long as `planes.padded_cols()`, with zeros past `cols`.
pub trait MatvecBackend: Send + Sync + std::fmt::Debug {
    fn matvec(&self, planes: &TritPlanes<'_>, xq: &[i8], y: &mut [i32]);

    /// Identifier for logs and benchmark reports, e.g. `cpu/avx512vnni x16`.
    fn name(&self) -> &str;

    fn threads(&self) -> usize {
        1
    }

    /// Dense weights, still in the precision the file stores them in.
    ///
    /// The point of not widening on load is bandwidth. Decode is memory-bound
    /// and the tied head is the largest single stream in it, so reading it as
    /// bf16 halves the bytes that stream per token. The values themselves are
    /// unchanged: the checkpoint is bf16, `tritc` used to widen it, and nothing
    /// downstream ever added precision.
    fn dense_matvec(
        &self,
        w: DenseWeights<'_>,
        rows: usize,
        cols: usize,
        x: &[f32],
        y: &mut [f32],
    ) {
        dense_matvec_reference(w, rows, cols, x, y)
    }

    /// Dense f32 matrix-vector product, row-major.
    ///
    /// Not ternary, and not incidental: the LM head is 128256 x 2560 f32 on
    /// BitNet 2B4T, so at batch 1 it moves 1315 MB per token against 521 MB for
    /// every ternary projection combined. A backend that vectorizes the ternary
    /// kernel and leaves this one scalar has not moved end-to-end throughput.
    ///
    /// The default is the portable reference; backends override it.
    fn f32_matvec(&self, w: &[f32], rows: usize, cols: usize, x: &[f32], y: &mut [f32]) {
        f32_matvec_reference(w, rows, cols, x, y)
    }
}

/// A dense weight matrix as stored, without a widening pass.
#[derive(Clone, Copy, Debug)]
pub enum DenseWeights<'a> {
    F32(&'a [f32]),
    /// Raw bf16 bit patterns. Widening is `bits << 16` reinterpreted as f32,
    /// which is exact, so a kernel may widen in registers and never materialise
    /// the f32 form.
    Bf16(&'a [u16]),
}

impl DenseWeights<'_> {
    pub fn len(&self) -> usize {
        match self {
            DenseWeights::F32(w) => w.len(),
            DenseWeights::Bf16(w) => w.len(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Bytes this matrix streams per pass. The number decode is bounded by.
    pub fn bytes(&self) -> usize {
        match self {
            DenseWeights::F32(w) => w.len() * 4,
            DenseWeights::Bf16(w) => w.len() * 2,
        }
    }
}

/// Exact widening of one bf16 bit pattern.
#[inline]
pub fn bf16_to_f32(bits: u16) -> f32 {
    f32::from_bits((bits as u32) << 16)
}

/// Portable dense matvec over either precision.
pub fn dense_matvec_reference(
    w: DenseWeights<'_>,
    rows: usize,
    cols: usize,
    x: &[f32],
    y: &mut [f32],
) {
    assert_eq!(w.len(), rows * cols);
    assert_eq!(x.len(), cols);
    assert_eq!(y.len(), rows);
    match w {
        DenseWeights::F32(w) => f32_matvec_reference(w, rows, cols, x, y),
        DenseWeights::Bf16(w) => {
            for (r, out) in y.iter_mut().enumerate() {
                let row = &w[r * cols..(r + 1) * cols];
                *out = row.iter().zip(x).map(|(a, b)| bf16_to_f32(*a) * b).sum();
            }
        }
    }
}

/// Portable dense f32 matvec.
pub fn f32_matvec_reference(w: &[f32], rows: usize, cols: usize, x: &[f32], y: &mut [f32]) {
    assert_eq!(w.len(), rows * cols);
    assert_eq!(x.len(), cols);
    assert_eq!(y.len(), rows);
    for (r, out) in y.iter_mut().enumerate() {
        let row = &w[r * cols..(r + 1) * cols];
        *out = row.iter().zip(x).map(|(a, b)| a * b).sum();
    }
}

/// The portable reference backend. Always available, no dependencies.
#[derive(Debug, Default, Clone, Copy)]
pub struct ScalarBackend;

impl MatvecBackend for ScalarBackend {
    fn matvec(&self, planes: &TritPlanes<'_>, xq: &[i8], y: &mut [i32]) {
        crate::matvec::ternary_matvec_planes(planes, xq, y)
    }
    fn name(&self) -> &str {
        "reference/scalar"
    }
}
