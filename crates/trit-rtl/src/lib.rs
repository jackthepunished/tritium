//! Hardware-in-the-loop backend: every ternary matvec executes on the Verilated
//! `tritcore` engine instead of the CPU.
//!
//! This is the continuity guarantee. The RTL is the thing the project is
//! ultimately building; routing the real model through it under simulation is
//! how a change to the software runtime is proven not to have drifted away from
//! the hardware. It is slow by construction -- a cycle-accurate simulator
//! decoding a 2B model -- and that is the point, not a defect.
//!
//! Since `.trit` v1 stores bit planes in the core's beat layout, the mapped
//! model bytes are streamed in directly: no repacking sits between the file and
//! the DUT.

use std::ffi::c_void;
use std::sync::{Mutex, OnceLock};

use trit_core::backend::MatvecBackend;
use trit_core::planes::TritPlanes;

extern "C" {
    fn trit_rtl_new() -> *mut c_void;
    #[allow(dead_code)]
    fn trit_rtl_free(h: *mut c_void);
    fn trit_rtl_matvec_v1(
        h: *mut c_void,
        beats: *const u8,
        x: *const i8,
        rows: u32,
        cols: u32,
        y_out: *mut i32,
    ) -> i32;
}

struct Core(*mut c_void);
// The Mutex below serializes every access; the raw pointer never crosses
// threads unlocked. The Verilated model is a single global instance with
// internal state, so this is a correctness requirement, not a convenience.
unsafe impl Send for Core {}

static CORE: OnceLock<Mutex<Core>> = OnceLock::new();

fn core() -> &'static Mutex<Core> {
    CORE.get_or_init(|| Mutex::new(Core(unsafe { trit_rtl_new() })))
}

/// Stream v1 beats through the Verilated core.
///
/// `cols` must already be a whole number of beats; `xq` must be at least that
/// long with zeros in the padding.
pub fn matvec_beats(beats: &[u8], rows: usize, cols: usize, xq: &[i8], y: &mut [i32]) {
    assert_eq!(
        cols % trit_core::planes::LANES,
        0,
        "cols must be a whole number of beats"
    );
    assert_eq!(
        beats.len(),
        rows * (cols / trit_core::planes::LANES) * trit_core::planes::BEAT_BYTES
    );
    assert!(xq.len() >= cols);
    assert_eq!(y.len(), rows);

    let guard = core().lock().unwrap();
    // SAFETY: the shim reads `rows * cols/64 * 16` bytes from `beats` and `cols`
    // bytes from `x`, both asserted above, and writes exactly `rows` i32s to
    // `y`. The handle is valid for the process lifetime and is held under the
    // lock for the whole call.
    let rc = unsafe {
        trit_rtl_matvec_v1(
            guard.0,
            beats.as_ptr(),
            xq.as_ptr(),
            rows as u32,
            cols as u32,
            y.as_mut_ptr(),
        )
    };
    drop(guard);
    assert_eq!(
        rc, 0,
        "RTL core error {rc} (1 = overlapping planes, 2 = row count mismatch)"
    );
}

/// The `MatvecBackend` implementation.
#[derive(Debug, Default)]
pub struct RtlBackend;

impl RtlBackend {
    pub fn new() -> Self {
        Self
    }
}

impl MatvecBackend for RtlBackend {
    fn matvec(&self, planes: &TritPlanes<'_>, xq: &[i8], y: &mut [i32]) {
        // padded_cols, not cols: the payload is already whole beats, and the
        // padding is zero in both planes so it contributes nothing.
        matvec_beats(
            planes.as_bytes(),
            planes.rows(),
            planes.padded_cols(),
            xq,
            y,
        );
    }

    fn name(&self) -> &str {
        "rtl/verilator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trit_core::matvec::ternary_matvec_planes_vec;
    use trit_core::planes::{beats_per_row, pack_planes, LANES};

    fn pad(x: &[i8], cols: usize) -> Vec<i8> {
        let mut v = vec![0i8; beats_per_row(cols) * LANES];
        v[..x.len()].copy_from_slice(x);
        v
    }

    #[test]
    fn hand_case_matches() {
        // W = [[1,-1,0],[0,1,1]], x = [10,20,30] -> [-10, 50]
        let beats = pack_planes(&[1, -1, 0, 0, 1, 1], 2, 3).unwrap();
        let planes = TritPlanes::new(&beats, 2, 3, 1.0).unwrap();
        let mut y = vec![0i32; 2];
        RtlBackend.matvec(&planes, &pad(&[10, 20, 30], 3), &mut y);
        assert_eq!(y, vec![-10, 50]);
    }

    /// The RTL must agree with the software reference exactly, on every shape
    /// the real model uses and across the full i8 activation range.
    #[test]
    fn matches_the_reference_exactly() {
        let mut state = 0x0dd5_eedu64;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        for &(rows, cols) in &[
            (1usize, 64usize),
            (3, 64),
            (2, 100),
            (5, 129),
            (7, 640),
            (2, 2560),
            (2, 6912),
            (16, 61),
        ] {
            let trits: Vec<i8> = (0..rows * cols)
                .map(|_| [-1i8, 0, 1][(next() % 3) as usize])
                .collect();
            // Full i8 range, so -128 appears: the RTL sign-extends to 32 bits
            // before negating, and the CPU kernels must match that.
            let x: Vec<i8> = (0..cols).map(|_| (next() & 0xff) as u8 as i8).collect();
            let beats = pack_planes(&trits, rows, cols).unwrap();
            let planes = TritPlanes::new(&beats, rows, cols, 1.0).unwrap();
            let xp = pad(&x, cols);

            let mut y = vec![0i32; rows];
            RtlBackend.matvec(&planes, &xp, &mut y);
            assert_eq!(
                y,
                ternary_matvec_planes_vec(&planes, &xp),
                "shape {rows}x{cols}"
            );
        }
    }
}
