//! Hardware-in-the-loop matvec: routes through the Verilated trit_matvec core
//! via the C shim in rtl/shim. Compiled only with `--features rtl`.

use std::ffi::c_void;
use std::sync::{Mutex, OnceLock};

use trit_core::planes::{pack_planes, BEAT_BYTES, LANES};

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
// The shim serializes all access through the Mutex below; the raw pointer
// never crosses threads unlocked.
unsafe impl Send for Core {}

static CORE: OnceLock<Mutex<Core>> = OnceLock::new();

fn core() -> &'static Mutex<Core> {
    CORE.get_or_init(|| Mutex::new(Core(unsafe { trit_rtl_new() })))
}

/// Stream a `.trit` v1 beat slice straight through the RTL core.
///
/// The beats are handed over as-is: v1's payload layout is exactly the core's
/// weight input, so a memory-mapped model needs no repacking on this path. The
/// caller is responsible for `beats.len() == rows * (cols / 64) * 16` with
/// `cols` already a multiple of 64.
pub fn rtl_matvec_beats(beats: &[u8], rows: usize, cols: usize, xq: &[i8]) -> Vec<i32> {
    assert_eq!(cols % LANES, 0, "cols must be a whole number of beats");
    assert_eq!(beats.len(), rows * (cols / LANES) * BEAT_BYTES);
    assert_eq!(xq.len(), cols);

    let mut y = vec![0i32; rows];
    let guard = core().lock().unwrap();
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
        "RTL core error {rc} (overlapping planes or row underflow)"
    );
    y
}

/// Exact drop-in for `trit_core::matvec::ternary_matvec`, executed by the RTL.
///
/// The oracle holds weights as one `i8` per trit, so this path still packs them
/// per call. That repack is not the reason the RTL path is slow: for a
/// 2560x6912 tensor it is a few tens of milliseconds against roughly 100k
/// Verilator ticks. It disappears entirely for callers that already hold v1
/// beats -- see `rtl_matvec_beats`.
///
/// Pads cols to a multiple of 64 (zero planes, zero activations) -- exact, since
/// padded terms contribute nothing.
pub fn rtl_matvec(trits: &[i8], rows: usize, cols: usize, xq: &[i8]) -> Vec<i32> {
    assert_eq!(trits.len(), rows * cols);
    assert_eq!(xq.len(), cols);
    let cp = cols.div_ceil(LANES) * LANES;

    let mut x = vec![0i8; cp];
    x[..cols].copy_from_slice(xq);

    let mut padded = vec![0i8; rows * cp];
    for r in 0..rows {
        padded[r * cp..r * cp + cols].copy_from_slice(&trits[r * cols..(r + 1) * cols]);
    }
    let beats = pack_planes(&padded, rows, cp).expect("ternary input");

    rtl_matvec_beats(&beats, rows, cp, &x)
}

#[cfg(test)]
mod tests {
    use super::*;
    use trit_core::matvec::ternary_matvec;

    #[test]
    fn hand_case_through_rtl() {
        // Phase-1 hand case: W=[[1,-1,0],[0,1,1]], x=[10,20,30] -> [-10, 50]
        let w: Vec<i8> = vec![1, -1, 0, 0, 1, 1];
        assert_eq!(rtl_matvec(&w, 2, 3, &[10, 20, 30]), vec![-10, 50]);
    }

    #[test]
    fn random_shapes_match_cpu_exactly() {
        let mut state = 0x0dd5_eedu64;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        let shapes: Vec<(usize, usize)> = vec![
            (1, 64),
            (3, 64),
            (2, 100), // padding path
            (5, 129), // padding path, off by one
            (7, 640),
            (2, 2560),
            (2, 6912), // real model width
            (16, 61),
        ];
        for (rows, cols) in shapes {
            let trits: Vec<i8> = (0..rows * cols)
                .map(|_| [(-1i8), 0, 1][(next() % 3) as usize])
                .collect();
            let x: Vec<i8> = (0..cols).map(|_| (next() & 0xff) as u8 as i8).collect();
            assert_eq!(
                rtl_matvec(&trits, rows, cols, &x),
                ternary_matvec(&trits, rows, cols, &x),
                "shape {rows}x{cols}"
            );
        }
    }
}
