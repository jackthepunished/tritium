//! Decode-phase profiler. Diagnostic only; not part of the runtime.
//!
//! Splits a real decode into the four buckets that matter for A2's remaining
//! scaling gap, without changing production code: `MatvecBackend` is injected
//! rather than global (architecture 4.1), so a wrapper around `CpuBackend`
//! sees every ternary projection and the dense head with the real model, the
//! real prompts and the real numerics ladder.
//!
//!   ternary   -- the 210 `matvec` calls per token, broken out by tensor shape
//!   dense     -- the single bf16 `dense_matvec` for the LM head
//!   other     -- total minus the two above: attention, RoPE, the norm/quantize
//!                stages, the integer MLP fold and the residual adds, all of
//!                which are serial scalar code
//!
//! Dispatch and synchronization cost is measured separately by `--pool-probe`,
//! which runs the real pool with a closure that does nothing. Multiply its
//! per-dispatch figure by 211 to get the per-token synchronization floor.
//!
//! The wrapper costs two `Instant::now()` calls per matvec, about 10 us per
//! token against a token that takes tens of milliseconds.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use trit_core::backend::{DenseWeights, MatvecBackend};
use trit_core::model::Model;
use trit_core::planes::TritPlanes;
use trit_core::sampler::SamplerParams;
use trit_core::session::Session;
use trit_core::tokenizer::Tokenizer as _;
use trit_core::tritfmt::TritFile;
use tritd::HfTokenizer;

#[derive(Default, Debug, Clone, Copy)]
struct Bucket {
    calls: u64,
    nanos: u128,
}

impl Bucket {
    fn add(&mut self, d: Duration) {
        self.calls += 1;
        self.nanos += d.as_nanos();
    }
    fn ms(&self) -> f64 {
        self.nanos as f64 / 1e6
    }
}

#[derive(Default, Debug)]
struct Stats {
    /// Keyed by (rows, cols) so q/k/v/o/gate/up/down separate themselves.
    ternary: BTreeMap<(usize, usize), Bucket>,
    dense: Bucket,
}

impl Stats {
    /// Fold another window in. Decode is measured per prompt, because the
    /// prefill between prompts has to be dropped; without merging, each
    /// `take` would discard the previous prompt's decode.
    fn merge(&mut self, other: Stats) {
        for (k, v) in other.ternary {
            let e = self.ternary.entry(k).or_default();
            e.calls += v.calls;
            e.nanos += v.nanos;
        }
        self.dense.calls += other.dense.calls;
        self.dense.nanos += other.dense.nanos;
    }

    fn ternary_total(&self) -> Bucket {
        self.ternary.values().fold(Bucket::default(), |mut a, b| {
            a.calls += b.calls;
            a.nanos += b.nanos;
            a
        })
    }
}

#[derive(Debug)]
struct TimedBackend {
    inner: trit_cpu::CpuBackend,
    name: String,
    stats: Mutex<Stats>,
}

impl TimedBackend {
    fn new(threads: usize) -> Self {
        let inner = trit_cpu::CpuBackend::new(threads);
        Self {
            name: format!("profile/{}", inner.name()),
            inner,
            stats: Mutex::new(Stats::default()),
        }
    }
    fn take(&self) -> Stats {
        std::mem::take(&mut self.stats.lock().unwrap())
    }
}

impl MatvecBackend for TimedBackend {
    fn matvec(&self, planes: &TritPlanes<'_>, xq: &[i8], y: &mut [i32]) {
        let key = (planes.rows(), planes.cols());
        let t = Instant::now();
        self.inner.matvec(planes, xq, y);
        let d = t.elapsed();
        self.stats
            .lock()
            .unwrap()
            .ternary
            .entry(key)
            .or_default()
            .add(d);
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn threads(&self) -> usize {
        self.inner.threads()
    }
    fn dense_matvec(
        &self,
        w: DenseWeights<'_>,
        rows: usize,
        cols: usize,
        x: &[f32],
        y: &mut [f32],
    ) {
        let t = Instant::now();
        self.inner.dense_matvec(w, rows, cols, x, y);
        let d = t.elapsed();
        self.stats.lock().unwrap().dense.add(d);
    }
    fn f32_matvec(&self, w: &[f32], rows: usize, cols: usize, x: &[f32], y: &mut [f32]) {
        let t = Instant::now();
        self.inner.f32_matvec(w, rows, cols, x, y);
        let d = t.elapsed();
        self.stats.lock().unwrap().dense.add(d);
    }
}

/// Mirrors `trit_cpu`'s private `slots_for`, so the report can say whether a
/// given projection was split at all. Kept in step by hand; the numbers it
/// prints are labelled as derived, not measured.
fn slots_for(bytes: usize, threads: usize, rows: usize) -> usize {
    let per_slot: usize = std::env::var("TRIT_BYTES_PER_SLOT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(256 << 10);
    if per_slot == 0 {
        return threads.min(rows).max(1);
    }
    (bytes / per_slot).clamp(1, threads).min(rows).max(1)
}

struct Args {
    model: PathBuf,
    threads: usize,
    tokens: usize,
    runs: usize,
    pool_probe: bool,
    bw_probe: bool,
}

fn parse() -> Result<Args> {
    let mut a = Args {
        model: PathBuf::from("models/bitnet-2b4t.trit"),
        threads: 4,
        tokens: 32,
        runs: 3,
        pool_probe: false,
        bw_probe: false,
    };
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < argv.len() {
        match argv[i].as_str() {
            "--model" => {
                a.model = PathBuf::from(argv.get(i + 1).context("--model needs a path")?);
                i += 2;
            }
            "--threads" => {
                a.threads = argv.get(i + 1).context("--threads needs N")?.parse()?;
                i += 2;
            }
            "--tokens" => {
                a.tokens = argv.get(i + 1).context("--tokens needs N")?.parse()?;
                i += 2;
            }
            "--runs" => {
                a.runs = argv.get(i + 1).context("--runs needs N")?.parse()?;
                i += 2;
            }
            "--pool-probe" => {
                a.pool_probe = true;
                i += 1;
            }
            "--bw-probe" => {
                a.bw_probe = true;
                i += 1;
            }
            other => anyhow::bail!("unknown argument {other:?}"),
        }
    }
    anyhow::ensure!(a.threads > 0, "--threads must be positive");
    Ok(a)
}

/// Cost of one `pool.run` dispatch with a closure that does nothing, so the
/// figure is wake-up plus barrier and nothing else.
fn pool_probe(threads: usize) {
    println!();
    println!("Pool dispatch probe: empty job, {threads} pool threads");
    if threads < 2 {
        println!("(single thread: no dispatch happens)");
        return;
    }
    let pool = match trit_cpu::pool::global(threads) {
        Some(p) => p,
        None => {
            eprintln!("pool width mismatch; run one width per process");
            return;
        }
    };

    // Gap between dispatches. Decode leaves roughly 17 us of serial scalar work
    // between consecutive matvecs (3.5 ms/token over 210 of them), and a worker
    // that parks in that gap costs a futex wake on the next dispatch. Sweeping
    // the gap separates "waking a parked worker" from "the dispatch itself".
    let gaps = [0u64, 2, 5, 10, 20, 50, 100];
    print!("{:<8}", "slots");
    for g in gaps {
        print!("{:>12}", format!("gap {g}us"));
    }
    println!("      (us per dispatch)");

    for slots in 2..=threads {
        let mut y = vec![0u8; slots];
        print!("{:<8}", slots);
        for g in gaps {
            let gap = Duration::from_micros(g);
            let spin_gap = || {
                if g == 0 {
                    return;
                }
                let t = Instant::now();
                while t.elapsed() < gap {
                    std::hint::spin_loop();
                }
            };
            for _ in 0..100 {
                pool.run(&mut y, 1, &|_, _| {});
                spin_gap();
            }
            let reps = 400;
            let t = Instant::now();
            for _ in 0..reps {
                pool.run(&mut y, 1, &|_, _| {});
                spin_gap();
            }
            let us = t.elapsed().as_secs_f64() * 1e6 / reps as f64 - g as f64;
            print!("{:>12.2}", us);
        }
        println!();
    }
}

/// Does a wider split buy bandwidth at all, once dispatch is removed?
///
/// Streams ~500 MB of planes -- a token's worth of ternary weight traffic --
/// in 4.42 MB tiles, the gate/up shape. Two partitionings over identical work:
///
///   pool   -- one `CpuBackend::matvec` per tile, which is what decode does
///             today: 150 parallel dispatches per token
///   static -- one `thread::scope` for the whole sweep, each thread owning a
///             fixed row band of every tile, so the barrier is paid once
///             instead of once per tile
///
/// The gap between them is dispatch cost. The `static` curve alone is the
/// bandwidth the memory system will give this access pattern at that width.
fn bw_probe(threads: usize) {
    use trit_core::planes::beats_per_row;

    let (rows, cols) = (6912usize, 2560usize);
    let bpr = beats_per_row(cols);
    let tile = rows * bpr * trit_core::planes::BEAT_BYTES;
    let tiles = 512usize * 1024 * 1024 / tile;
    println!(
        "Bandwidth probe: {tiles} tiles of {rows}x{cols} ({:.2} MB each, {:.0} MB total)",
        tile as f64 / 1e6,
        (tile * tiles) as f64 / 1e6
    );

    let mut buf = vec![0u8; tile * tiles];
    // Plane words drawn so no lane is set in both planes (invariant P1).
    // cols is a multiple of 64, so there is no padding to keep clear (P2).
    let mut rng = 0x243F_6A88_85A3_08D3u64;
    let mut next = || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };
    for w in buf.chunks_exact_mut(16) {
        let (a, b) = (next(), next());
        w[..8].copy_from_slice(&(a & !b).to_le_bytes());
        w[8..].copy_from_slice(&(b & !a).to_le_bytes());
    }
    let x = vec![7i8; cols];
    let bytes = (tile * tiles) as f64;

    println!(
        "{:<10} {:>10} {:>10} {:>12}",
        "mode", "threads", "ms", "GB/s"
    );
    {
        let t = threads;
        // static: one barrier for the whole sweep.
        let ms_static = {
            let buf = &buf;
            let x = &x;
            let t0 = Instant::now();
            std::thread::scope(|sc| {
                for w in 0..t {
                    let lo = rows * w / t;
                    let hi = rows * (w + 1) / t;
                    sc.spawn(move || {
                        let mut y = vec![0i32; hi - lo];
                        let stride = bpr * trit_core::planes::BEAT_BYTES;
                        for ti in 0..tiles {
                            let o = ti * tile;
                            let band = &buf[o + lo * stride..o + hi * stride];
                            let p =
                                TritPlanes::new_unchecked_bits(band, hi - lo, cols, 1.0).unwrap();
                            trit_cpu::ternary_matvec_planes(&p, x, &mut y);
                        }
                        std::hint::black_box(&y);
                    });
                }
            });
            t0.elapsed().as_secs_f64() * 1000.0
        };

        // pool: one dispatch per tile, exactly as decode does.
        let ms_pool = {
            let be = trit_cpu::CpuBackend::new(t);
            let mut y = vec![0i32; rows];
            let t0 = Instant::now();
            for ti in 0..tiles {
                let o = ti * tile;
                let p = TritPlanes::new_unchecked_bits(&buf[o..o + tile], rows, cols, 1.0).unwrap();
                be.matvec(&p, &x, &mut y);
            }
            std::hint::black_box(&y);
            t0.elapsed().as_secs_f64() * 1000.0
        };

        for (name, ms) in [("static", ms_static), ("pool", ms_pool)] {
            println!(
                "{:<10} {:>10} {:>10.1} {:>12.1}",
                name,
                t,
                ms,
                bytes / (ms / 1000.0) / 1e9
            );
        }
    }
}

fn main() -> Result<()> {
    let args = parse()?;

    if args.pool_probe {
        pool_probe(args.threads);
        return Ok(());
    }
    if args.bw_probe {
        bw_probe(args.threads);
        return Ok(());
    }

    let file = TritFile::open(&args.model)?;
    let backend = Arc::new(TimedBackend::new(args.threads));
    let probe = backend.clone();
    let model = Model::load(file, backend)?;
    let tk = HfTokenizer::from_file(&tritd::default_tokenizer_path(&args.model))?;

    // The same four prompts benches/prompts/short.jsonl carries, so the
    // profile describes the configuration the published table measures.
    let prompts = [
        "The capital of France is",
        "To make bread you need",
        "fn fibonacci(n: u32) -> u32 {",
        "Q: Why is the sky blue?\nA:",
    ];
    let encoded: Vec<Vec<u32>> = prompts
        .iter()
        .map(|p| tk.encode(p, true))
        .collect::<Result<_>>()?;

    println!(
        "model {} | backend {} | threads {} | {} tokens x {} prompts x {} runs",
        args.model.display(),
        probe.name(),
        args.threads,
        args.tokens,
        prompts.len(),
        args.runs
    );

    // Warm the page cache and the pool before anything is attributed.
    {
        let mut s = Session::new(model.clone(), &SamplerParams::default());
        s.prefill(&encoded[0])?;
        for _ in 0..4 {
            if s.next_token()?.is_none() {
                break;
            }
        }
    }
    let _ = probe.take();

    let mut decode_secs = 0f64;
    let mut decoded = 0usize;
    let mut acc = Stats::default();
    for _ in 0..args.runs.max(1) {
        for ids in &encoded {
            let mut s = Session::new(model.clone(), &SamplerParams::default());
            s.prefill(ids)?;
            // Prefill is a different regime; only decode is attributed.
            let _ = probe.take();
            let t = Instant::now();
            let mut n = 0;
            while n < args.tokens {
                if s.next_token()?.is_none() {
                    break;
                }
                n += 1;
            }
            decode_secs += t.elapsed().as_secs_f64();
            decoded += n;
            acc.merge(probe.take());
        }
    }

    let st = acc;
    let tern = st.ternary_total();
    // 210 ternary matvecs and one dense head per decode step. A different
    // number means the attribution window is wrong, not that the model changed.
    let per_step = tern.calls as f64 / decoded as f64;
    assert!(
        (per_step - 210.0).abs() < 0.5 && st.dense.calls as usize == decoded,
        "attribution window is wrong: {per_step} ternary and {} dense calls per token",
        st.dense.calls as f64 / decoded as f64
    );
    let total_ms = decode_secs * 1000.0;
    let per_tok = |ms: f64| ms / decoded as f64;
    let other_ms = total_ms - tern.ms() - st.dense.ms();

    println!();
    println!(
        "decoded {decoded} tokens in {:.1} ms -> {:.3} tok/s, {:.2} ms/token",
        total_ms,
        decoded as f64 / decode_secs,
        per_tok(total_ms)
    );
    println!();
    println!(
        "{:<28} {:>12} {:>12} {:>9} {:>10}",
        "phase", "ms/token", "share", "calls/tok", "GB/s"
    );

    let tern_bytes: usize = 521_011_200;
    let dense_bytes: usize = 656_670_720;
    let row = |name: &str, b: Bucket, bytes: Option<usize>| {
        let ms = per_tok(b.ms());
        println!(
            "{:<28} {:>12.2} {:>11.1}% {:>9.1} {:>10}",
            name,
            ms,
            b.ms() / total_ms * 100.0,
            b.calls as f64 / decoded as f64,
            match bytes {
                Some(by) => format!("{:.1}", by as f64 / (ms / 1000.0) / 1e9),
                None => "-".into(),
            }
        );
    };
    row("ternary projections", tern, Some(tern_bytes));
    row("dense bf16 head", st.dense, Some(dense_bytes));
    println!(
        "{:<28} {:>12.2} {:>11.1}% {:>9} {:>10}",
        "other (serial scalar)",
        per_tok(other_ms),
        other_ms / total_ms * 100.0,
        "-",
        "-"
    );

    println!();
    println!(
        "{:<18} {:>8} {:>10} {:>12} {:>9} {:>8} {:>10}",
        "ternary tensor", "rows", "cols", "MB/call", "ms/token", "slots", "GB/s"
    );
    for ((rows, cols), b) in &st.ternary {
        let bytes = rows * cols.div_ceil(64) * 16;
        let ms = per_tok(b.ms());
        let per_call = b.calls as f64 / decoded as f64;
        println!(
            "{:<18} {:>8} {:>10} {:>12.2} {:>9.2} {:>8} {:>10.1}",
            format!("{}x{}", rows, cols),
            rows,
            cols,
            bytes as f64 / 1e6,
            ms,
            slots_for(bytes, args.threads, *rows),
            bytes as f64 * per_call / (ms / 1000.0) / 1e9
        );
    }
    Ok(())
}
