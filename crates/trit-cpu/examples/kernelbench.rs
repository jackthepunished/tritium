//! Per-kernel throughput on a realistic tensor shape.
//!
//! Reports GB/s of weight traffic, which is the number that matters: decoding at
//! batch 1 is memory bound, so a kernel's job is to saturate the link, not to
//! minimize instructions.

use std::time::Instant;
use trit_core::backend::MatvecBackend;
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

fn main() {
    // down_proj from BitNet b1.58 2B4T: the widest tensor in the model.
    let (rows, cols) = (2560usize, 6912usize);
    let mut rng = Rng(0xBEEF);
    let trits: Vec<i8> = (0..rows * cols)
        .map(|_| {
            // Match the measured 0.4219 zero fraction rather than a uniform
            // third, so the sparsity-skipping paths are exercised realistically.
            let r = rng.next() % 10000;
            if r < 4219 {
                0
            } else if r < 7110 {
                1
            } else {
                -1
            }
        })
        .collect();
    let beats = pack_planes(&trits, rows, cols).unwrap();
    let planes = TritPlanes::new(&beats, rows, cols, 1.0).unwrap();
    let mut x = vec![0i8; beats_per_row(cols) * LANES];
    for v in x.iter_mut().take(cols) {
        *v = (rng.next() & 0xff) as u8 as i8;
    }

    let bytes = beats.len() as f64;
    println!("tensor {rows}x{cols}, {:.1} MB of planes, kernels: {:?}", bytes / 1e6, trit_cpu::available_kernels());
    println!();
    println!("{:<14} {:>10} {:>10} {:>12}", "kernel", "ms", "GB/s", "vs scalar");

    // Scalar first so the ratio column has a baseline before the others run.
    let mut order = trit_cpu::available_kernels();
    order.sort_by_key(|k| if *k == "scalar" { 0 } else { 1 });

    let mut scalar_ms = 0f64;
    for name in order {
        // Each kernel needs its own process-lifetime dispatch slot, so call the
        // implementation directly rather than through the cached dispatcher.
        let mut y = vec![0i32; rows];
        let f: fn(&[u8], usize, usize, &[i8], &mut [i32]) = match name {
            "scalar" => |b, r, s, xq, y| trit_cpu::scalar::matvec(b, r, s, xq, y),
            #[cfg(target_arch = "x86_64")]
            "avx2" => |b, r, s, xq, y| unsafe { trit_cpu::x86::matvec(b, r, s, xq, y) },
            #[cfg(target_arch = "x86_64")]
            "avx512bw" => |b, r, s, xq, y| unsafe { trit_cpu::x86::matvec_avx512bw(b, r, s, xq, y) },
            #[cfg(target_arch = "x86_64")]
            "avx512vnni" => |b, r, s, xq, y| unsafe { trit_cpu::x86::matvec_avx512vnni(b, r, s, xq, y) },
            #[cfg(target_arch = "aarch64")]
            "neon" => |b, r, s, xq, y| unsafe { trit_cpu::aarch64::matvec(b, r, s, xq, y) },
            #[cfg(target_arch = "aarch64")]
            "neon-dotprod" => |b, r, s, xq, y| unsafe { trit_cpu::aarch64::matvec_dotprod(b, r, s, xq, y) },
            #[cfg(feature = "bitserial")]
            "bitserial" => |b, r, s, xq, y| unsafe { trit_cpu::bitserial::matvec(b, r, s, xq, y) },
            _ => continue,
        };

        let bpr = beats_per_row(cols);
        f(&beats, rows, bpr, &x, &mut y); // warm
        let reps = 5;
        let t = Instant::now();
        for _ in 0..reps {
            f(&beats, rows, bpr, &x, &mut y);
        }
        let ms = t.elapsed().as_secs_f64() * 1000.0 / reps as f64;
        if name == "scalar" {
            scalar_ms = ms;
        }
        println!(
            "{:<14} {:>10.2} {:>10.1} {:>11.2}x",
            name,
            ms,
            bytes / (ms / 1000.0) / 1e9,
            scalar_ms / ms
        );
    }

    println!();
    println!("Note: {:.1} MB of planes fits in L3, so the table above measures", bytes / 1e6);
    println!("compute throughput, not memory throughput. Real decoding streams");
    println!("521 MB of weights per token from DRAM; see the streaming pass below.");

    println!();
    println!("{:<14} {:>10} {:>10}", "threads", "ms", "GB/s");
    for threads in [1usize, 2, 4, 8, 16, 32] {
        let be = trit_cpu::CpuBackend::new(threads);
        let mut y = vec![0i32; rows];
        be.matvec(&planes, &x, &mut y);
        let reps = 5;
        let t = Instant::now();
        for _ in 0..reps {
            be.matvec(&planes, &x, &mut y);
        }
        let ms = t.elapsed().as_secs_f64() * 1000.0 / reps as f64;
        println!("{:<14} {:>10.2} {:>10.1}", threads, ms, bytes / (ms / 1000.0) / 1e9);
    }

    streaming_pass();
}

/// A working set far larger than L3, which is the regime batch-1 decoding
/// actually runs in: every weight is touched once per token and none of it is
/// resident. This is the number that predicts tokens/sec.
fn streaming_pass() {
    let (rows, cols) = (65536usize, 6912usize); // ~113 MB of planes
    let mut rng = Rng(0xF00D);
    let trits: Vec<i8> = (0..rows * cols)
        .map(|_| {
            let r = rng.next() % 10000;
            if r < 4219 { 0 } else if r < 7110 { 1 } else { -1 }
        })
        .collect();
    let beats = pack_planes(&trits, rows, cols).unwrap();
    let planes = TritPlanes::new(&beats, rows, cols, 1.0).unwrap();
    let mut x = vec![0i8; beats_per_row(cols) * LANES];
    for v in x.iter_mut().take(cols) {
        *v = (rng.next() & 0xff) as u8 as i8;
    }
    let bytes = beats.len() as f64;

    println!();
    println!("Streaming pass: {rows}x{cols} = {:.0} MB of planes, well past L3", bytes / 1e6);
    println!("{:<14} {:>10} {:>10} {:>18}", "threads", "ms", "GB/s", "implied tok/s*");
    for threads in [1usize, 2, 4, 8, 16, 32] {
        let be = trit_cpu::CpuBackend::new(threads);
        let mut y = vec![0i32; rows];
        be.matvec(&planes, &x, &mut y);
        let t = Instant::now();
        be.matvec(&planes, &x, &mut y);
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        let gbps = bytes / (ms / 1000.0) / 1e9;
        // 521 MB of ternary + 1315 MB of dense per token on BitNet 2B4T.
        println!("{:<14} {:>10.2} {:>10.1} {:>17.1}", threads, ms, gbps, gbps * 1e9 / 1.836e9);
    }
    println!("*at this bandwidth against the full 1836 MB/token, ternary + dense.");
}
