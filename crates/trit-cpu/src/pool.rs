//! A persistent worker pool for row-parallel matvecs.
//!
//! # Why this exists
//!
//! A decode step issues 210 ternary matvecs, and the largest of them is 4.4 MB.
//! Handing each one to a work-stealing pool costs more than the matvec: measured
//! on a 32-thread Zen 4, letting them parallelise that way took layer time from
//! 37.4 ms at one thread to 173 ms at eight. A dedicated, correctly-sized
//! work-stealing pool and a broadcast primitive were both measured too, and both
//! remained *slower than not parallelising at all*.
//!
//! What works is the shape ggml uses: workers spawned once, waiting on a
//! sequence counter, with no allocation and no scheduler involvement per job.
//! Same measurement, same machine: 37.1 / 20.0 / 12.8 / 14.1 ms at 1 / 2 / 4 / 8
//! threads.
//!
//! # Why it does not spin
//!
//! Jobs arrive roughly every 300 us during decode. Spinning through that gap
//! wastes a core per worker, which is affordable on a 32-thread desktop and is
//! not affordable on the four-core edge parts this runtime exists for. Workers
//! spin briefly, then park. `thread::park` and `unpark` carry a token, so a
//! worker that parks after the dispatcher has already unparked it wakes
//! immediately rather than missing the job.
//!
//! # Why the unsafe is sound
//!
//! The job outlives no call. [`Pool::run`] publishes a pointer to a caller-owned
//! closure, wakes the workers, runs slot 0 itself, and then blocks until every
//! other slot has reported completion. Only after that does it return, so no
//! worker can hold a reference to the closure or to `y` past the borrow. The
//! closure is erased to a *thin* pointer plus a monomorphised trampoline that
//! knows its concrete type, so nothing here transmutes a fat pointer.
//!
//! Panics are the one hazard: a worker that unwinds would never report done and
//! would hang the dispatcher. Slot bodies are wrapped in `catch_unwind`, the
//! panic is recorded, and `run` resumes it on the calling thread once every slot
//! has been accounted for.

use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;

/// Iterations spun before parking. Long enough to cover a job that is already
/// in flight, short enough not to burn a core while decode is between layers.
const SPIN_LIMIT: u32 = 4_000;

/// One unit of work: the erased closure and a trampoline that can call it.
#[derive(Clone, Copy)]
struct Task {
    /// Thin pointer to the caller's closure.
    data: *const (),
    /// Monomorphised over the element type and the closure's concrete type by
    /// [`Pool::run`], so the erased pointers below can be recovered exactly.
    call: unsafe fn(*const (), usize, *mut (), usize),
    /// One `(ptr, element count)` per slot, carved from the caller's output.
    /// Disjoint.
    parts: *const (*mut (), usize),
    n_parts: usize,
}

// SAFETY: the pointers inside are only dereferenced between the sequence bump
// and the completion barrier in `run`, during which the caller is blocked and
// the data it points at is alive and not aliased. `run` is the only producer.
unsafe impl Send for Task {}
unsafe impl Sync for Task {}

struct Shared {
    /// Bumped once per job. Workers wake when it changes.
    seq: AtomicUsize,
    /// Slots finished for the current job.
    done: AtomicUsize,
    /// The current job. `None` between jobs.
    task: Mutex<Option<Task>>,
    /// Workers that have entered their loop. `Pool::new` blocks on this.
    ready: AtomicUsize,
    /// Set when a worker's slot panicked, so `run` can re-raise on the caller.
    panicked: AtomicBool,
    /// Told to exit at process teardown.
    stop: AtomicBool,
    threads: usize,
}

/// A pool of `threads` slots, of which slot 0 is the calling thread.
pub struct Pool {
    shared: Arc<Shared>,
    handles: Vec<thread::Thread>,
}

impl Pool {
    fn new(threads: usize) -> Self {
        let shared = Arc::new(Shared {
            seq: AtomicUsize::new(0),
            done: AtomicUsize::new(0),
            task: Mutex::new(None),
            ready: AtomicUsize::new(0),
            panicked: AtomicBool::new(false),
            stop: AtomicBool::new(false),
            threads,
        });
        let mut handles = Vec::with_capacity(threads.saturating_sub(1));
        // Slot 0 is whoever calls `run`; spawn the rest.
        for slot in 1..threads {
            let shared = Arc::clone(&shared);
            let h = thread::Builder::new()
                .name(format!("trit-worker-{slot}"))
                .spawn(move || worker(shared, slot))
                .expect("spawn worker");
            handles.push(h.thread().clone());
        }
        // Wait for every worker to have read the initial sequence number.
        //
        // Without this, a worker that first runs *after* a job is dispatched
        // latches the already-bumped sequence as its baseline, waits for the job
        // after this one, and never reports the current one. The dispatcher then
        // blocks forever. Spawning is not starting.
        while shared.ready.load(Ordering::Acquire) < threads.saturating_sub(1) {
            std::hint::spin_loop();
        }
        Self { shared, handles }
    }

    pub fn threads(&self) -> usize {
        self.shared.threads
    }

    /// Split `y` into `chunk`-sized pieces and run `f(slot, piece)` across the
    /// pool, returning only once every piece is complete.
    ///
    /// `f` is called with the slot index and a disjoint sub-slice of `y`, so it
    /// may write freely without synchronisation.
    pub fn run<T, F>(&self, y: &mut [T], chunk: usize, f: &F)
    where
        T: Send,
        F: Fn(usize, &mut [T]) + Sync,
    {
        let parts: Vec<(*mut (), usize)> = y
            .chunks_mut(chunk)
            .map(|c| (c.as_mut_ptr() as *mut (), c.len()))
            .collect();
        if parts.is_empty() {
            return;
        }
        // More chunks than slots would silently drop work; the caller sizes
        // `chunk` from the slot count, so this is a contract check rather than a
        // recoverable condition.
        assert!(
            parts.len() <= self.shared.threads,
            "{} chunks for {} slots",
            parts.len(),
            self.shared.threads
        );

        /// Recovers the closure's concrete type and calls it.
        ///
        /// # Safety
        /// `data` must point to a live `F`, and `ptr`/`len` must describe a
        /// `[T]` that no other slot is touching.
        unsafe fn trampoline<T, F>(data: *const (), slot: usize, ptr: *mut (), len: usize)
        where
            F: Fn(usize, &mut [T]) + Sync,
        {
            let f = &*(data as *const F);
            f(slot, std::slice::from_raw_parts_mut(ptr as *mut T, len));
        }

        let task = Task {
            data: f as *const F as *const (),
            call: trampoline::<T, F>,
            parts: parts.as_ptr(),
            n_parts: parts.len(),
        };

        // Only slots with a chunk report, and only those are woken.
        //
        // The byte cap frequently gives a job fewer slots than the pool has, and
        // waking every worker then costs a futex wake per idle worker per
        // matvec: 210 matvecs a token times fifteen idle workers is where the
        // sixteen-thread regression came from.
        //
        // This stays free of the straggler race because an idle worker never
        // touches `done`. Every worker counted here had work, and `run` blocks
        // until all of them report, so none can still be running when the next
        // job resets the counter.
        let reporters = parts.len() - 1;
        self.shared.done.store(0, Ordering::Relaxed);
        self.shared.panicked.store(false, Ordering::Relaxed);
        *self.shared.task.lock().unwrap_or_else(|e| e.into_inner()) = Some(task);

        // Release: everything above must be visible to a worker that observes
        // the new sequence number.
        self.shared.seq.fetch_add(1, Ordering::Release);
        for h in self.handles.iter().take(reporters) {
            h.unpark();
        }

        // Slot 0 is this thread.
        let mine = run_slot(&task, 0);

        // Block until every other slot has reported. This is what makes the
        // pointers in `task` sound: nothing escapes the call.
        let mut spins = 0u32;
        while self.shared.done.load(Ordering::Acquire) < reporters {
            spins = spins.saturating_add(1);
            if spins < SPIN_LIMIT {
                std::hint::spin_loop();
            } else {
                thread::yield_now();
            }
        }

        *self.shared.task.lock().unwrap_or_else(|e| e.into_inner()) = None;
        drop(parts);

        if let Err(p) = mine {
            resume_unwind(p);
        }
        if self.shared.panicked.load(Ordering::Acquire) {
            panic!("a pool worker panicked");
        }
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::Release);
        self.shared.seq.fetch_add(1, Ordering::Release);
        // Every worker, not just the ones a job would have used: they all have
        // to observe `stop` and leave.
        for h in &self.handles {
            h.unpark();
        }
    }
}

/// Run one slot of the current task. Returns the panic payload if it unwound.
fn run_slot(task: &Task, slot: usize) -> Result<(), Box<dyn std::any::Any + Send>> {
    if slot >= task.n_parts {
        return Ok(());
    }
    // SAFETY: `parts` points at a live Vec owned by the blocked caller, `slot`
    // is in bounds, and each slot touches a disjoint piece.
    let (ptr, len) = unsafe { *task.parts.add(slot) };
    catch_unwind(AssertUnwindSafe(|| unsafe {
        (task.call)(task.data, slot, ptr, len)
    }))
}

fn worker(shared: Arc<Shared>, slot: usize) {
    let mut last = shared.seq.load(Ordering::Acquire);
    shared.ready.fetch_add(1, Ordering::Release);
    loop {
        // Spin briefly, then park. `unpark` leaves a token, so a worker that
        // parks just after being woken returns from `park` immediately.
        let mut spins = 0u32;
        loop {
            let now = shared.seq.load(Ordering::Acquire);
            if now != last {
                last = now;
                break;
            }
            spins = spins.saturating_add(1);
            if spins < SPIN_LIMIT {
                std::hint::spin_loop();
            } else {
                thread::park();
            }
        }

        if shared.stop.load(Ordering::Acquire) {
            return;
        }

        let task = match *shared.task.lock().unwrap_or_else(|e| e.into_inner()) {
            Some(t) => t,
            // Woken with no task: teardown, or a spurious wake between jobs.
            None => continue,
        };

        // This job gave us no chunk. Report nothing: the dispatcher counts only
        // the slots it handed work to, and an idle worker reporting would
        // release the barrier while another slot is still writing.
        if slot >= task.n_parts {
            continue;
        }

        if run_slot(&task, slot).is_err() {
            shared.panicked.store(true, Ordering::Release);
        }
        // Release: the caller's acquire-load of `done` must see our writes.
        shared.done.fetch_add(1, Ordering::Release);
    }
}

static POOL: OnceLock<Pool> = OnceLock::new();

/// The process-wide pool, sized on first use.
///
/// Returns `None` when a different thread count is requested than the pool was
/// built with, so the caller falls back rather than silently using the wrong
/// width. In practice the backend is constructed once per process.
pub fn global(threads: usize) -> Option<&'static Pool> {
    let p = POOL.get_or_init(|| Pool::new(threads.max(1)));
    (p.threads() == threads.max(1)).then_some(p)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU32;

    #[test]
    fn every_slot_runs_exactly_once() {
        let pool = Pool::new(4);
        for _ in 0..64 {
            let mut y = vec![0i32; 4];
            let calls = AtomicU32::new(0);
            pool.run(&mut y, 1, &|slot, out| {
                calls.fetch_add(1, Ordering::Relaxed);
                out[0] = slot as i32 + 1;
            });
            assert_eq!(y, vec![1, 2, 3, 4]);
            assert_eq!(calls.load(Ordering::Relaxed), 4);
        }
    }

    /// The barrier is the safety argument, so it gets a test: no write may
    /// land after `run` returns.
    #[test]
    fn run_does_not_return_before_every_slot_finishes() {
        let pool = Pool::new(4);
        for _ in 0..200 {
            let mut y = vec![0i32; 4096];
            pool.run(&mut y, 1024, &|slot, out| {
                // Uneven work, so a slot that was not waited on would be
                // observable as a zero below.
                for _ in 0..(slot * 500) {
                    std::hint::spin_loop();
                }
                out.iter_mut().for_each(|v| *v = 7);
            });
            assert!(y.iter().all(|&v| v == 7), "a slot had not finished");
        }
    }

    /// Idle slots must not report completion.
    ///
    /// This is the invariant that makes it safe to wake only the slots with
    /// work. When idle workers reported too, their increments satisfied the
    /// barrier before the working slots had finished, and `run` returned while
    /// another thread was still writing into the caller's buffer. That is a
    /// use-after-scope; it showed up here as a zero in the output.
    #[test]
    fn idle_slots_do_not_release_the_barrier_early() {
        let pool = Pool::new(8);
        for _ in 0..500 {
            let mut y = vec![0i32; 3];
            pool.run(&mut y, 1, &|slot, out| {
                // Later slots finish last, so an early release is observable.
                for _ in 0..(slot * 2000) {
                    std::hint::spin_loop();
                }
                out[0] = 5;
            });
            assert_eq!(y, vec![5, 5, 5], "barrier released before a slot wrote");
        }
    }

    #[test]
    fn results_do_not_depend_on_slot_count() {
        let expect: Vec<i32> = (0..1000).map(|i| i * 3).collect();
        for threads in [1usize, 2, 3, 8] {
            let pool = Pool::new(threads);
            let mut y = vec![0i32; 1000];
            let chunk = 1000usize.div_ceil(threads);
            pool.run(&mut y, chunk, &|slot, out| {
                let base = slot * chunk;
                for (j, v) in out.iter_mut().enumerate() {
                    *v = ((base + j) * 3) as i32;
                }
            });
            assert_eq!(y, expect, "{threads} slots changed the result");
        }
    }

    #[test]
    fn a_panicking_slot_does_not_hang_the_pool() {
        let pool = Pool::new(4);
        let mut y = vec![0i32; 4];
        let r = catch_unwind(AssertUnwindSafe(|| {
            pool.run(&mut y, 1, &|slot, _| {
                if slot == 2 {
                    panic!("slot {slot} exploded");
                }
            });
        }));
        assert!(r.is_err(), "the panic must reach the caller");

        // And the pool is still usable afterwards.
        let mut y2 = vec![0i32; 4];
        pool.run(&mut y2, 1, &|_, out| out[0] = 9);
        assert_eq!(y2, vec![9, 9, 9, 9]);
    }
}
