//! Persistent worker pool for row-parallel matvecs.
//!
//! A decode step issues 210 ternary matvecs; per-job fork/join costs more than
//! the matvec. Measured layer time at 1/2/4/8 threads: 37.6/37.6/36.9/36.7 ms
//! serial, 36.0/65.0/108.8/174.0 with a work-stealing pool, 37.1/20.0/12.8/14.1
//! here. A right-sized pool and a broadcast primitive were also measured and
//! both stayed slower than serial.
//!
//! Workers spin briefly then park; jobs arrive ~300 us apart and spinning
//! through that wastes a core per worker on the four-core parts this targets.

use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;

const SPIN_LIMIT: u32 = 4_000;

/// Erased closure plus a trampoline monomorphised over its concrete type, so
/// nothing here transmutes a fat pointer.
#[derive(Clone, Copy)]
struct Task {
    data: *const (),
    call: unsafe fn(*const (), usize, *mut (), usize),
    /// Disjoint `(ptr, len)` per slot, carved from the caller's output.
    parts: *const (*mut (), usize),
    n_parts: usize,
}

// SAFETY: dereferenced only between the sequence bump and the completion
// barrier in `run`, during which the caller is blocked and the data is alive.
unsafe impl Send for Task {}
unsafe impl Sync for Task {}

struct Shared {
    seq: AtomicUsize,
    done: AtomicUsize,
    task: Mutex<Option<Task>>,
    ready: AtomicUsize,
    panicked: AtomicBool,
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
        // Slot 0 is the caller.
        for slot in 1..threads {
            let shared = Arc::clone(&shared);
            let h = thread::Builder::new()
                .name(format!("trit-worker-{slot}"))
                .spawn(move || worker(shared, slot))
                .expect("spawn worker");
            handles.push(h.thread().clone());
        }
        // A worker that first runs after a job is dispatched would latch the
        // bumped sequence as its baseline and never report that job.
        while shared.ready.load(Ordering::Acquire) < threads.saturating_sub(1) {
            std::hint::spin_loop();
        }
        Self { shared, handles }
    }

    pub fn threads(&self) -> usize {
        self.shared.threads
    }

    /// Run `f(slot, piece)` over `chunk`-sized pieces of `y`, returning only
    /// once every piece is complete. Pieces are disjoint.
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
        assert!(
            parts.len() <= self.shared.threads,
            "{} chunks for {} slots",
            parts.len(),
            self.shared.threads
        );

        /// # Safety
        /// `data` must point to a live `F`; `ptr`/`len` must describe a `[T]`
        /// no other slot is touching.
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

        // Only slots with a chunk are woken and counted. Waking idle workers
        // cost a futex wake each, per matvec, which is where the sixteen-thread
        // regression came from. Safe because an idle worker never touches
        // `done`, so no straggler can satisfy the next job's barrier.
        let reporters = parts.len() - 1;
        self.shared.done.store(0, Ordering::Relaxed);
        self.shared.panicked.store(false, Ordering::Relaxed);
        *self.shared.task.lock().unwrap_or_else(|e| e.into_inner()) = Some(task);

        self.shared.seq.fetch_add(1, Ordering::Release);
        for h in self.handles.iter().take(reporters) {
            h.unpark();
        }

        let mine = run_slot(&task, 0);

        // Blocking here is what makes the pointers in `task` sound.
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
        for h in &self.handles {
            h.unpark();
        }
    }
}

fn run_slot(task: &Task, slot: usize) -> Result<(), Box<dyn std::any::Any + Send>> {
    if slot >= task.n_parts {
        return Ok(());
    }
    // SAFETY: `parts` is owned by the blocked caller; slots are disjoint.
    let (ptr, len) = unsafe { *task.parts.add(slot) };
    catch_unwind(AssertUnwindSafe(|| unsafe {
        (task.call)(task.data, slot, ptr, len)
    }))
}

fn worker(shared: Arc<Shared>, slot: usize) {
    let mut last = shared.seq.load(Ordering::Acquire);
    shared.ready.fetch_add(1, Ordering::Release);
    loop {
        // `unpark` leaves a token, so parking just after a wake returns at once.
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
            None => continue,
        };

        // No chunk: report nothing, or the barrier releases early.
        if slot >= task.n_parts {
            continue;
        }

        if run_slot(&task, slot).is_err() {
            shared.panicked.store(true, Ordering::Release);
        }
        shared.done.fetch_add(1, Ordering::Release);
    }
}

static POOL: OnceLock<Pool> = OnceLock::new();

/// The process-wide pool, sized on first use. `None` if a different width is
/// requested later, so the caller falls back rather than using the wrong one.
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

    #[test]
    fn run_does_not_return_before_every_slot_finishes() {
        let pool = Pool::new(4);
        for _ in 0..200 {
            let mut y = vec![0i32; 4096];
            pool.run(&mut y, 1024, &|slot, out| {
                // Uneven, so an unwaited slot shows up as a zero.
                for _ in 0..(slot * 500) {
                    std::hint::spin_loop();
                }
                out.iter_mut().for_each(|v| *v = 7);
            });
            assert!(y.iter().all(|&v| v == 7), "a slot had not finished");
        }
    }

    /// Wake-only-the-working-slots is safe only if idle slots stay silent.
    #[test]
    fn idle_slots_do_not_release_the_barrier_early() {
        let pool = Pool::new(8);
        for _ in 0..500 {
            let mut y = vec![0i32; 3];
            pool.run(&mut y, 1, &|slot, out| {
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
