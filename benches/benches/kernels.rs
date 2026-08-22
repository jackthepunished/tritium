//! Per-kernel microbenchmarks.
//!
//! Reports throughput in bytes of weight planes per second, because that is the
//! quantity that predicts tokens/sec: decoding at batch 1 is memory bound, and a
//! kernel's job is to saturate the link rather than to minimize instructions.
//!
//! These are microbenchmarks. Tensors this size sit in L3, so they measure
//! compute throughput, not the streaming regime a real decode runs in. For the
//! end-to-end number use `tritd bench`, which reports achieved bandwidth against
//! a memcpy probe measured on the same machine.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use trit_core::planes::{beats_per_row, pack_planes, TritPlanes, LANES};

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

/// Weights with the zero fraction measured on the real checkpoint (0.4219), so
/// the sparsity-skipping paths are exercised realistically rather than at a
/// uniform third.
fn weights(rng: &mut Rng, n: usize) -> Vec<i8> {
    (0..n)
        .map(|_| match rng.next() % 10000 {
            0..=4218 => 0,
            4219..=7109 => 1,
            _ => -1,
        })
        .collect()
}

/// Real BitNet b1.58 2B4T shapes.
const SHAPES: &[(&str, usize, usize)] = &[
    ("q_proj 2560x2560", 2560, 2560),
    ("k_proj 640x2560", 640, 2560),
    ("gate_proj 6912x2560", 6912, 2560),
    ("down_proj 2560x6912", 2560, 6912),
];

fn bench_kernels(c: &mut Criterion) {
    let mut rng = Rng(0xBEEF);
    let available = trit_cpu::available_kernels();
    eprintln!("kernels available on this CPU: {available:?}");

    for &(name, rows, cols) in SHAPES {
        let trits = weights(&mut rng, rows * cols);
        let beats = pack_planes(&trits, rows, cols).unwrap();
        let planes = TritPlanes::new(&beats, rows, cols, 1.0).unwrap();
        let mut x = vec![0i8; beats_per_row(cols) * LANES];
        for v in x.iter_mut().take(cols) {
            *v = (rng.next() & 0xff) as u8 as i8;
        }
        let mut y = vec![0i32; rows];

        let mut group = c.benchmark_group(name);
        group.throughput(Throughput::Bytes(beats.len() as u64));

        // The dispatcher caches its choice for the process, so measure each
        // implementation directly rather than forcing and re-forcing.
        group.bench_function(BenchmarkId::new("kernel", "dispatched"), |b| {
            b.iter(|| trit_cpu::ternary_matvec_planes(&planes, &x, &mut y))
        });
        group.bench_function(BenchmarkId::new("kernel", "scalar"), |b| {
            b.iter(|| {
                trit_cpu::scalar::matvec(
                    planes.as_bytes(),
                    planes.rows(),
                    planes.beats_per_row(),
                    &x,
                    &mut y,
                )
            })
        });
        group.bench_function(BenchmarkId::new("kernel", "reference"), |b| {
            b.iter(|| trit_core::matvec::ternary_matvec_planes(&planes, &x, &mut y))
        });
        group.finish();
    }
}

/// The dense f32 path. It moves 1313 MB per token on this model against 521 MB
/// for every ternary projection combined, so it is not a footnote.
fn bench_lm_head(c: &mut Criterion) {
    // A slice of the real head: full vocab would be 1.3 GB.
    let (rows, cols) = (16384usize, 2560usize);
    let w: Vec<f32> = (0..rows * cols)
        .map(|i| ((i % 251) as f32 - 125.0) * 0.001)
        .collect();
    let x: Vec<f32> = (0..cols).map(|i| ((i % 97) as f32 - 48.0) * 0.01).collect();
    let mut y = vec![0.0f32; rows];

    let mut group = c.benchmark_group("lm_head f32");
    group.throughput(Throughput::Bytes((w.len() * 4) as u64));
    group.bench_function("simd", |b| {
        b.iter(|| trit_cpu::f32_kernels::f32_matvec(&w, rows, cols, &x, &mut y, 1))
    });
    group.bench_function("scalar", |b| {
        b.iter(|| trit_cpu::f32_kernels::f32_matvec_scalar(&w, rows, cols, &x, &mut y))
    });
    group.finish();
}

criterion_group!(benches, bench_kernels, bench_lm_head);
criterion_main!(benches);
