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
pub trait MatvecBackend: Send + Sync {
    fn matvec(&self, planes: &TritPlanes<'_>, xq: &[i8], y: &mut [i32]);

    /// Identifier for logs and benchmark reports, e.g. `cpu/avx512vnni x16`.
    fn name(&self) -> &str;

    fn threads(&self) -> usize {
        1
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
