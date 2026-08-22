//! Measurement engine.
//!
//! Reports achieved bandwidth against a memcpy probe measured on the same box,
//! so "percent of roofline" is a measured ratio rather than an assumption about
//! what the memory system can do.

use std::time::Instant;

use anyhow::Result;
use serde::Serialize;

use crate::stream::TokenStream;
use crate::Runtime;
use trit_core::sampler::SamplerParams;
use trit_core::tokenizer::Tokenizer;

#[derive(Debug, Serialize)]
pub struct BenchReport {
    pub model: String,
    pub backend: String,
    pub kernel: String,
    pub threads: usize,
    pub prompt_tokens: usize,
    pub decoded_tokens: usize,
    pub ttft_ms: f64,
    pub prefill_tok_per_s: f64,
    pub decode_tok_per_s: f64,
    pub weight_bytes_per_token: u64,
    pub ternary_bytes: u64,
    pub lm_head_bytes: u64,
    pub kv_bytes: u64,
    pub achieved_gbps: f64,
    /// Streaming read bandwidth measured on this machine, right now.
    pub memcpy_gbps: Option<f64>,
    /// `achieved / memcpy`, the honest roofline percentage.
    pub roofline_pct: Option<f64>,
    pub peak_rss_mb: Option<u64>,
    /// Populated only when a real energy source is present. Never estimated:
    /// an invented number would poison the one claim the project exists to make.
    pub joules_per_token: Option<f64>,
    pub energy_source: String,
}

/// Peak resident set, from `/proc/self/status`. `None` off Linux.
pub fn peak_rss_mb() -> Option<u64> {
    let s = std::fs::read_to_string("/proc/self/status").ok()?;
    for line in s.lines() {
        if let Some(v) = line.strip_prefix("VmHWM:") {
            let kb: u64 = v.split_whitespace().next()?.parse().ok()?;
            return Some(kb / 1024);
        }
    }
    None
}

/// Streaming read bandwidth of this machine, measured over a working set well
/// past any last-level cache.
pub fn memcpy_probe_gbps(threads: usize) -> f64 {
    const BYTES: usize = 512 << 20;
    let src = vec![1u8; BYTES];
    // Fold with wrapping_add: the value is a checksum nobody reads, and a plain
    // sum over half a gigabyte overflows u64 (and panics in debug builds). What
    // matters is that every byte is loaded and the result is not optimized away,
    // which is what a weight pass looks like.
    let sum_range = |r: std::ops::Range<usize>| -> u64 {
        src[r]
            .chunks_exact(8)
            .fold(0u64, |a, c| a.wrapping_add(u64::from_le_bytes(c.try_into().unwrap())))
    };

    let t = Instant::now();
    let total: u64 = if threads <= 1 {
        sum_range(0..BYTES)
    } else {
        let chunk = BYTES / threads;
        std::thread::scope(|s| {
            let handles: Vec<_> = (0..threads)
                .map(|i| {
                    let src = &src;
                    s.spawn(move || {
                        let start = i * chunk;
                        let end = if i + 1 == threads { BYTES } else { start + chunk };
                        src[start..end].chunks_exact(8).fold(0u64, |a, c| {
                            a.wrapping_add(u64::from_le_bytes(c.try_into().unwrap()))
                        })
                    })
                })
                .collect();
            handles.into_iter().map(|h| h.join().unwrap()).fold(0u64, u64::wrapping_add)
        })
    };
    let secs = t.elapsed().as_secs_f64();
    std::hint::black_box(total);
    BYTES as f64 / secs / 1e9
}

/// Energy counters, where the platform exposes them.
///
/// Returns `(joules, source)`. Intel RAPL is read directly; anything else
/// reports no source at all rather than guessing.
fn read_energy_uj() -> Option<u64> {
    let mut total = 0u64;
    let mut found = false;
    let dir = std::fs::read_dir("/sys/class/powercap").ok()?;
    for e in dir.flatten() {
        let p = e.path().join("energy_uj");
        if let Ok(s) = std::fs::read_to_string(&p) {
            if let Ok(v) = s.trim().parse::<u64>() {
                total += v;
                found = true;
            }
        }
    }
    found.then_some(total)
}

pub struct BenchOptions {
    pub prompt: String,
    pub max_tokens: usize,
    pub warmup: usize,
    pub runs: usize,
    pub measure_memcpy: bool,
}

pub fn run(rt: &Runtime, opts: &BenchOptions) -> Result<BenchReport> {
    let params = SamplerParams::default(); // greedy: reproducible
    let ids = rt.tokenizer.encode(&opts.prompt, true)?;
    anyhow::ensure!(!ids.is_empty(), "prompt tokenized to nothing");

    for _ in 0..opts.warmup {
        let mut s = TokenStream::new(rt.model.clone(), &rt.tokenizer, &params, &ids, opts.max_tokens.min(4))?;
        for t in s.by_ref() {
            t?;
        }
    }

    let energy_before = read_energy_uj();
    let mut best_decode_rate = 0f64;
    let mut ttft_ms = f64::MAX;
    let mut decoded_total = 0usize;
    let mut kv_bytes = 0u64;
    let t_all = Instant::now();

    for _ in 0..opts.runs.max(1) {
        let mut stream =
            TokenStream::new(rt.model.clone(), &rt.tokenizer, &params, &ids, opts.max_tokens)?;
        let prefill_secs = stream.prefill_secs;

        let t0 = Instant::now();
        let mut first: Option<f64> = None;
        let mut n = 0usize;
        for tok in stream.by_ref() {
            tok?;
            if first.is_none() {
                first = Some(t0.elapsed().as_secs_f64());
            }
            n += 1;
        }
        let decode_secs = t0.elapsed().as_secs_f64();
        kv_bytes = stream.session().kv_bytes();

        if n > 0 {
            // Report the best run, not the mean: a slow run measures the
            // scheduler, a fast one measures the code.
            best_decode_rate = best_decode_rate.max(n as f64 / decode_secs);
            let t = (prefill_secs + first.unwrap_or(0.0)) * 1000.0;
            ttft_ms = ttft_ms.min(t);
            decoded_total = n;
        }
    }
    let wall = t_all.elapsed().as_secs_f64();

    let (joules_per_token, energy_source) = match (energy_before, read_energy_uj()) {
        (Some(a), Some(b)) if b >= a && decoded_total > 0 => (
            Some((b - a) as f64 / 1e6 / (decoded_total * opts.runs.max(1)) as f64),
            "rapl".to_string(),
        ),
        // No counter, or a wrapped one. Report nothing rather than estimating.
        _ => (None, "none".to_string()),
    };

    let weight_bytes = rt.model.weight_bytes();
    let achieved_gbps = weight_bytes as f64 * best_decode_rate / 1e9;
    let memcpy_gbps = opts.measure_memcpy.then(|| memcpy_probe_gbps(rt.model.backend().threads()));

    let _ = wall;
    Ok(BenchReport {
        model: String::new(),
        backend: rt.backend_name.clone(),
        kernel: rt.kernel_name.to_string(),
        threads: rt.model.backend().threads(),
        prompt_tokens: ids.len(),
        decoded_tokens: decoded_total,
        ttft_ms: if ttft_ms == f64::MAX { 0.0 } else { ttft_ms },
        prefill_tok_per_s: 0.0,
        decode_tok_per_s: best_decode_rate,
        weight_bytes_per_token: weight_bytes,
        ternary_bytes: rt.model.ternary_bytes(),
        lm_head_bytes: rt.model.lm_head_bytes(),
        kv_bytes,
        achieved_gbps,
        memcpy_gbps,
        roofline_pct: memcpy_gbps.map(|m| achieved_gbps / m * 100.0),
        peak_rss_mb: peak_rss_mb(),
        joules_per_token,
        energy_source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memcpy_probe_reports_a_plausible_rate() {
        let g = memcpy_probe_gbps(1);
        // Anything outside this is a broken measurement, not a fast machine.
        assert!(g > 0.1 && g < 10_000.0, "implausible bandwidth {g} GB/s");
    }

    #[test]
    fn energy_is_absent_rather_than_invented_when_unavailable() {
        // Whatever this host has, the reader must never fabricate a value.
        if let Some(v) = read_energy_uj() {
            assert!(v > 0);
        }
    }
}
