//! Bit-sliced ternary matvec for commodity SIMD silicon.
//!
//! Every kernel computes the same function of the same bytes:
//!
//! ```text
//! y[r] = sum(x[c] for c where pos bit c set) - sum(x[c] for c where neg bit c set)
//! ```
//!
//! reading the `.trit` bit planes in place -- no unpacking, no dequantization,
//! no floating point. Because the accumulators are integers and integer addition
//! is exact and order-independent, every kernel here is **bit-identical** to the
//! portable one and to the RTL core. That is what lets the differential tests
//! demand exact equality rather than a tolerance, and why swapping kernels can
//! never move a logit.
//!
//! ## Kernel selection
//!
//! Resolved once into a function pointer, cached in a `OnceLock`. Override with
//! the `TRIT_CPU_KERNEL` environment variable or [`force_kernel`]; both **error**
//! on a kernel this CPU cannot run rather than falling back, so a CI runner
//! without AVX-512 fails loudly instead of silently re-testing the scalar path.

use std::sync::OnceLock;
use trit_core::backend::MatvecBackend;
use trit_core::planes::TritPlanes;

pub mod f32_kernels;
pub mod pool;
pub mod scalar;

#[cfg(target_arch = "x86_64")]
pub mod x86;

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

#[cfg(feature = "bitserial")]
pub mod bitserial;

/// A kernel: beats, row count, beats per row, activations, output.
///
/// `unsafe` because the ISA-specific implementations require their target
/// features; the safe wrappers in this module establish every other precondition
/// before calling one.
pub type MatvecFn = unsafe fn(&[u8], usize, usize, &[i8], &mut [i32]);

/// Every kernel this build contains, in preference order.
///
/// Listed unconditionally per architecture so `available_kernels` can report
/// what the *CPU* supports separately from what the *build* contains.
fn all_kernels() -> Vec<(&'static str, MatvecFn, bool)> {
    let mut v: Vec<(&'static str, MatvecFn, bool)> = Vec::new();
    #[cfg(target_arch = "x86_64")]
    {
        v.push((
            "avx512vnni",
            x86::matvec_avx512vnni as MatvecFn,
            is_x86_feature_detected!("avx512f")
                && is_x86_feature_detected!("avx512bw")
                && is_x86_feature_detected!("avx512vnni"),
        ));
        v.push((
            "avx512bw",
            x86::matvec_avx512bw as MatvecFn,
            is_x86_feature_detected!("avx512f") && is_x86_feature_detected!("avx512bw"),
        ));
        v.push((
            "avx2",
            x86::matvec as MatvecFn,
            is_x86_feature_detected!("avx2"),
        ));
    }
    #[cfg(target_arch = "aarch64")]
    {
        v.push((
            "neon-dotprod",
            aarch64::matvec_dotprod as MatvecFn,
            std::arch::is_aarch64_feature_detected!("dotprod"),
        ));
        // NEON is baseline on aarch64.
        v.push(("neon", aarch64::matvec as MatvecFn, true));
    }
    #[cfg(feature = "bitserial")]
    v.push(("bitserial", bitserial::matvec as MatvecFn, true));
    v.push(("scalar", scalar_shim as MatvecFn, true));
    v
}

/// # Safety
/// Always safe; the signature matches [`MatvecFn`] so the scalar path can sit in
/// the same dispatch table.
unsafe fn scalar_shim(b: &[u8], rows: usize, bpr: usize, xq: &[i8], y: &mut [i32]) {
    scalar::matvec(b, rows, bpr, xq, y)
}

/// Names of the kernels this build contains that this CPU can actually run.
pub fn available_kernels() -> Vec<&'static str> {
    all_kernels()
        .into_iter()
        .filter(|k| k.2)
        .map(|k| k.0)
        .collect()
}

#[derive(Debug)]
pub struct UnsupportedKernel {
    pub requested: String,
    /// What this CPU *can* run. Carried rather than looked up, so the dense f32
    /// dispatch can raise the same error about its own kernel list.
    pub available: Vec<&'static str>,
}

impl std::fmt::Display for UnsupportedKernel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "kernel {:?} is not available on this CPU (have: {})",
            self.requested,
            self.available.join(", ")
        )
    }
}
impl std::error::Error for UnsupportedKernel {}

fn lookup(name: &str) -> Result<(&'static str, MatvecFn), UnsupportedKernel> {
    all_kernels()
        .into_iter()
        .find(|k| k.0 == name && k.2)
        .map(|k| (k.0, k.1))
        .ok_or_else(|| UnsupportedKernel {
            requested: name.to_string(),
            available: available_kernels(),
        })
}

static KERNEL: OnceLock<(&'static str, MatvecFn)> = OnceLock::new();

fn resolve() -> (&'static str, MatvecFn) {
    *KERNEL.get_or_init(|| {
        if let Ok(name) = std::env::var("TRIT_CPU_KERNEL") {
            // Deliberately fatal: a typo or an unsupported request must not
            // silently degrade to a slower kernel and report success.
            return lookup(&name).unwrap_or_else(|e| panic!("TRIT_CPU_KERNEL: {e}"));
        }
        let k = all_kernels();
        let best = k.iter().find(|x| x.2).expect("scalar is always available");
        (best.0, best.1)
    })
}

/// Pin the kernel for the process. Errors -- never falls back -- if this CPU
/// cannot run it.
///
/// The choice is cached for the process, so this can only take effect before
/// anything has resolved a kernel. Asking for the one already in force is fine;
/// asking for a *different* one after the fact is an error rather than a
/// silently ignored request. Returning `Ok` with someone else's kernel would
/// let a CI job believe it had tested `scalar` while it re-tested `avx512vnni`,
/// which is the exact failure this whole forcing mechanism exists to prevent.
pub fn force_kernel(name: &str) -> Result<&'static str, UnsupportedKernel> {
    let (n, f) = lookup(name)?;
    let _ = KERNEL.set((n, f));
    let in_force = resolve().0;
    if in_force != n {
        return Err(UnsupportedKernel {
            requested: format!("{name} (already resolved to {in_force} earlier in this process)"),
            available: available_kernels(),
        });
    }
    Ok(in_force)
}

/// The kernel in force.
pub fn kernel_name() -> &'static str {
    resolve().0
}

/// `y[r] = W[r] . xq` over the bit planes, with the active kernel.
///
/// `xq` must be at least `planes.padded_cols()` long with zeros in the padding;
/// padded columns are clear in both planes, so they contribute nothing.
/// Establish every length relation the kernels rely on.
///
/// Factored out of [`ternary_matvec_planes`] so the threaded path in
/// [`CpuBackend::matvec`] can hold the identical contract before it calls a raw
/// kernel. `CpuBackend::matvec` is public trait API: an `xq` shorter than
/// `padded_cols()` would otherwise be read past the end through
/// `xq.as_ptr().add(b * LANES)` -- undefined behaviour rather than a panic.
fn assert_shapes(planes: &TritPlanes<'_>, xq: &[i8], y: &[i32]) {
    assert!(
        xq.len() >= planes.padded_cols(),
        "activation buffer is {} long, need {} (padded to a whole beat)",
        xq.len(),
        planes.padded_cols()
    );
    assert_eq!(y.len(), planes.rows(), "output length must match row count");
    assert_eq!(
        planes.as_bytes().len(),
        planes.rows() * planes.beats_per_row() * trit_core::planes::BEAT_BYTES
    );
    debug_assert!(planes.check_invariants().is_ok(), "planes violate P1/P2");
}

pub fn ternary_matvec_planes(planes: &TritPlanes<'_>, xq: &[i8], y: &mut [i32]) {
    let (rows, bpr) = (planes.rows(), planes.beats_per_row());
    // Established here so the unsafe kernels carry exactly one obligation each:
    // that the CPU supports their target features.
    assert_shapes(planes, xq, y);
    let beats = planes.as_bytes();

    let (_, f) = resolve();
    // SAFETY: `f` came from the dispatch table, which only ever yields kernels
    // whose target features this CPU was detected to support. All length
    // relations are asserted above.
    unsafe { f(beats, rows, bpr, xq, y) }
}

/// Allocating convenience wrapper.
pub fn ternary_matvec_planes_vec(planes: &TritPlanes<'_>, xq: &[i8]) -> Vec<i32> {
    let mut y = vec![0i32; planes.rows()];
    ternary_matvec_planes(planes, xq, &mut y);
    y
}

/// The CPU backend.
///
/// Threading splits **rows**, never a reduction, so results stay bit-identical
/// to the single-threaded path regardless of thread count or scheduling.
#[derive(Debug)]
pub struct CpuBackend {
    threads: usize,
    name: String,
}

/// Ceiling on the automatically chosen thread count.
///
/// Decode at batch 1 is memory-bound, so past a handful of cores the extra
/// threads contend for bandwidth instead of adding it, and the per-job wake-ups
/// start to cost more than the slice they enable. Measured against the previous
/// implementation in interleaved pairs on a 32-thread Zen 4: 1.17x at two
/// threads, 1.18x at four, 1.17x at eight, and 0.94x at sixteen.
///
/// So one thread per core is the wrong automatic answer on a large machine. An
/// explicit `--threads` is still honoured exactly, including into the region
/// where it regresses; this only changes what "decide for me" means.
const DEFAULT_MAX_THREADS: usize = 8;

impl CpuBackend {
    /// `threads = 0` means "choose", which is one per core up to
    /// [`DEFAULT_MAX_THREADS`].
    pub fn new(threads: usize) -> Self {
        let threads = if threads == 0 {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
                .min(DEFAULT_MAX_THREADS)
        } else {
            threads
        };
        Self {
            name: format!("cpu/{}x{}", kernel_name(), threads),
            threads,
        }
    }
}

impl Default for CpuBackend {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Minimum plane bytes before a ternary matvec is worth splitting across
/// threads.
///
/// This was 32 MiB, chosen to keep *every* per-layer tensor in this model class
/// on one thread, because splitting them through a work-stealing pool cost more
/// than the matvec: end-to-end decode measured 14.06 tok/s single-threaded
/// against 2.40 tok/s at 32 threads. That threshold was a workaround for the
/// dispatch mechanism, not a property of the work.
///
/// With [`pool`], dispatch is a sequence-counter bump and an unpark, so the
/// crossover moves down by roughly three orders of magnitude. What remains is a
/// floor below which even that is not worth it: a 64 KiB tensor is around 4000
/// beats, which one core finishes in the time the wake-up takes.
const PARALLEL_MIN_BYTES: usize = 64 << 10;

/// Plane bytes each slot should get before another slot is worth waking.
///
/// Slot count has to track the work, not the machine. A decode step's matvecs
/// run from roughly 400 KiB to 4.4 MiB, and splitting the small ones across
/// every core costs more in wake-ups than the slice saves: measured end-to-end,
/// an uncapped pool peaked at 20.28 tok/s on four threads and fell to 16.46 on
/// eight and 11.00 on sixteen, worse than not parallelising at all. At 256 KiB
/// the peak is within 1.5% of uncapped and the sixteen-thread case recovers to
/// 14.70. Larger slices (1 MiB) give up too much at two and four threads.
///
/// With a cap the thread count becomes a ceiling rather than an instruction, so
/// asking for more threads than the work can use is harmless.
const BYTES_PER_SLOT_DEFAULT: usize = 256 << 10;

/// Resolved once. `TRIT_BYTES_PER_SLOT=0` disables the cap entirely, which is
/// how the default was chosen rather than guessed.
pub(crate) fn bytes_per_slot() -> usize {
    static V: OnceLock<usize> = OnceLock::new();
    *V.get_or_init(|| {
        std::env::var("TRIT_BYTES_PER_SLOT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(BYTES_PER_SLOT_DEFAULT)
    })
}

/// How many slots a job of `bytes` should use, given the pool width and the
/// number of independent output rows available to split.
pub(crate) fn slots_for(bytes: usize, threads: usize, rows: usize) -> usize {
    let per_slot = bytes_per_slot();
    if per_slot == 0 {
        return threads.min(rows).max(1);
    }
    (bytes / per_slot).clamp(1, threads).min(rows).max(1)
}

impl MatvecBackend for CpuBackend {
    fn matvec(&self, planes: &TritPlanes<'_>, xq: &[i8], y: &mut [i32]) {
        let pool = (self.threads > 1)
            .then(|| pool::global(self.threads))
            .flatten();
        let Some(pool) = pool else {
            return ternary_matvec_planes(planes, xq, y);
        };
        if planes.as_bytes().len() < PARALLEL_MIN_BYTES {
            return ternary_matvec_planes(planes, xq, y);
        }
        // The threaded branch calls the raw kernel, so it must establish the
        // same preconditions the dispatching wrapper would have.
        assert_shapes(planes, xq, y);
        let bpr = planes.beats_per_row();
        let stride = bpr * trit_core::planes::BEAT_BYTES;
        let beats = planes.as_bytes();
        let (_, f) = resolve();
        // Chunk by rows. Each chunk is an independent set of outputs reading a
        // disjoint slice of beats, so no reduction is split and the result does
        // not depend on how the work was divided.
        // Slots are bounded by the work as well as by the pool.
        let slots = slots_for(planes.as_bytes().len(), self.threads, planes.rows());
        if slots < 2 {
            return ternary_matvec_planes(planes, xq, y);
        }
        let chunk = planes.rows().div_ceil(slots).max(1);
        pool.run(y, chunk, &|slot, out| {
            let start = slot * chunk;
            let sub = &beats[start * stride..(start + out.len()) * stride];
            // SAFETY: same contract as ternary_matvec_planes; lengths are
            // derived from the validated planes view.
            unsafe { f(sub, out.len(), bpr, xq, out) }
        });
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn threads(&self) -> usize {
        self.threads
    }

    fn f32_matvec(&self, w: &[f32], rows: usize, cols: usize, x: &[f32], y: &mut [f32]) {
        f32_kernels::f32_matvec(w, rows, cols, x, y, self.threads)
    }
}
