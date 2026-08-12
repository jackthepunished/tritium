//! Matvec backend dispatch: CPU golden path or the Verilated RTL core.

use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Backend {
    Cpu,
    #[cfg(feature = "rtl")]
    Rtl,
}

static SELECTED: AtomicU8 = AtomicU8::new(0);

pub fn set_backend(b: Backend) {
    let v = match b {
        Backend::Cpu => 0,
        #[cfg(feature = "rtl")]
        Backend::Rtl => 1,
    };
    SELECTED.store(v, Ordering::Relaxed);
}

pub fn matvec(trits: &[i8], rows: usize, cols: usize, xq: &[i8]) -> Vec<i32> {
    match SELECTED.load(Ordering::Relaxed) {
        #[cfg(feature = "rtl")]
        1 => crate::rtl::rtl_matvec(trits, rows, cols, xq),
        _ => trit_core::matvec::ternary_matvec(trits, rows, cols, xq),
    }
}
