//! `tritd` -- the Tritium host daemon.

use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use trit_core::config::Numerics;
use trit_core::sampler::SamplerParams;
use trit_core::tokenizer::Tokenizer;
use tritd::bench::{BenchOptions, BenchReport};
use tritd::serve::ServeOptions;
use tritd::stream::TokenStream;
use tritd::{backend, LoadOptions, Runtime};

#[derive(Parser)]
#[command(name = "tritd", version, about = "Tritium ternary inference runtime")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

/// Options shared by every subcommand that loads a model.
#[derive(clap::Args, Clone)]
struct ModelArgs {
    #[arg(long)]
    model: PathBuf,
    /// Defaults to tokenizer.json beside the model.
    #[arg(long)]
    tokenizer: Option<PathBuf>,
    /// Compute backend. `rtl` needs a build with --features rtl.
    #[arg(long, default_value = "cpu")]
    backend: String,
    /// Worker threads; 0 means one per core.
    #[arg(long, default_value_t = 0)]
    threads: usize,
    /// Pin a CPU kernel instead of auto-detecting. Errors if unsupported.
    #[arg(long)]
    kernel: Option<String>,
    /// Numerics rung: reference | folded | intmlp.
    #[arg(long)]
    numerics: Option<String>,
    /// Verify plane invariants and the payload hash while loading.
    #[arg(long)]
    verify: bool,
}

impl ModelArgs {
    fn load(&self) -> Result<Runtime> {
        let mut rt = Runtime::load(&LoadOptions {
            model: self.model.clone(),
            tokenizer: self.tokenizer.clone(),
            backend: self.backend.clone(),
            threads: self.threads,
            kernel: self.kernel.clone(),
            verify: self.verify,
        })?;
        if let Some(n) = &self.numerics {
            let n: Numerics = n.parse()?;
            let model = std::sync::Arc::get_mut(&mut rt.model)
                .expect("model is uniquely owned right after load");
            model.set_numerics(n)?;
        }
        Ok(rt)
    }
}

#[derive(clap::Args, Clone)]
struct SampleArgs {
    /// 0 selects greedy decoding.
    #[arg(long, default_value_t = 0.0)]
    temperature: f32,
    #[arg(long, default_value_t = 1.0)]
    top_p: f32,
    #[arg(long, default_value_t = 0)]
    top_k: usize,
    #[arg(long, default_value_t = 1.0)]
    repeat_penalty: f32,
    #[arg(long, default_value_t = 0)]
    seed: u64,
}

impl From<&SampleArgs> for SamplerParams {
    fn from(a: &SampleArgs) -> Self {
        SamplerParams {
            temperature: a.temperature,
            top_p: a.top_p,
            top_k: a.top_k,
            repetition_penalty: a.repeat_penalty,
            seed: a.seed,
        }
    }
}

#[derive(Subcommand)]
enum Cmd {
    /// Generate text, streaming to stdout
    Run {
        #[command(flatten)]
        model: ModelArgs,
        #[command(flatten)]
        sample: SampleArgs,
        #[arg(long)]
        prompt: Option<String>,
        /// Read the prompt from a file, or `-` for stdin
        #[arg(long)]
        prompt_file: Option<PathBuf>,
        #[arg(long, default_value_t = 64)]
        steps: usize,
    },
    /// Serve an HTTP endpoint with SSE streaming
    Serve {
        #[command(flatten)]
        model: ModelArgs,
        /// Loopback by default: the parser is hand-rolled, so do not expose it
        /// to a network without a reverse proxy in front.
        #[arg(long, default_value = "127.0.0.1:8080")]
        addr: String,
        #[arg(long, default_value_t = 256)]
        max_tokens: usize,
    },
    /// Measure throughput, bandwidth and time-to-first-token
    Bench {
        #[command(flatten)]
        model: ModelArgs,
        #[arg(long, default_value = "The capital of France is")]
        prompt: String,
        /// JSONL prompt suite, one `{"id":..,"text":..}` per line. Measures
        /// every prompt in the file and names it in the report, so a recorded
        /// result identifies the input it came from. Overrides --prompt.
        #[arg(long)]
        suite: Option<PathBuf>,
        #[arg(long, default_value_t = 32)]
        tokens: usize,
        #[arg(long, default_value_t = 1)]
        warmup: usize,
        #[arg(long, default_value_t = 3)]
        runs: usize,
        /// Skip the streaming-bandwidth probe. It runs by default so the
        /// roofline percentage is a measured ratio on this machine rather than
        /// an assumption about what its memory system can do.
        #[arg(long)]
        no_memcpy_probe: bool,
        /// Write the report as JSON
        #[arg(long)]
        json: Option<PathBuf>,
    },
    /// Print the model header, footprint and per-token bandwidth
    Info {
        #[command(flatten)]
        model: ModelArgs,
    },
}

/// Read a JSONL prompt suite: one `{"id": ..., "text": ...}` object per line.
///
/// Blank lines are skipped. A malformed line is an error rather than a silent
/// omission -- a suite that quietly measured fewer prompts than it names would
/// make two runs incomparable without either of them looking wrong.
fn read_suite(path: &std::path::Path) -> Result<Vec<String>> {
    #[derive(serde::Deserialize)]
    struct Entry {
        text: String,
    }
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("read prompt suite {}", path.display()))?;
    let mut out = Vec::new();
    for (i, line) in raw.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let e: Entry = serde_json::from_str(line)
            .with_context(|| format!("{}:{}: not a prompt entry", path.display(), i + 1))?;
        out.push(e.text);
    }
    anyhow::ensure!(!out.is_empty(), "{} has no prompts", path.display());
    Ok(out)
}

fn main() -> Result<()> {
    match Cli::parse().cmd {
        Cmd::Run {
            model,
            sample,
            prompt,
            prompt_file,
            steps,
        } => {
            let text = tritd::read_prompt(prompt.as_deref(), prompt_file.as_deref())?;
            let rt = model.load()?;
            let ids = rt.tokenizer.encode(&text, true)?;
            let params: SamplerParams = (&sample).into();

            let t0 = std::time::Instant::now();
            let mut stream =
                TokenStream::new(rt.model.clone(), &rt.tokenizer, &params, &ids, steps)?;
            let prefill = stream.prefill_secs;

            let mut n = 0usize;
            let mut out = std::io::stdout().lock();
            for tok in stream.by_ref() {
                let t = tok?;
                // Flush per token: this is the whole point of streaming.
                out.write_all(t.text.as_bytes())?;
                out.flush()?;
                n += 1;
            }
            let tail = stream.flush();
            out.write_all(tail.as_bytes())?;
            out.write_all(b"\n")?;
            out.flush()?;

            let decode = t0.elapsed().as_secs_f64() - prefill;
            // Summary to stderr, so `tritd run > out.txt` leaves clean text.
            eprintln!(
                "prefill {} tok in {:.2}s ({:.1} tok/s) | decode {n} tok in {:.2}s ({:.1} tok/s) \
                 | {:.0} MB/token | backend {} | kernel {}",
                ids.len(),
                prefill,
                ids.len() as f64 / prefill.max(1e-9),
                decode,
                n as f64 / decode.max(1e-9),
                rt.model.weight_bytes() as f64 / 1e6,
                rt.backend_name,
                rt.kernel_name,
            );
        }

        Cmd::Serve {
            model,
            addr,
            max_tokens,
        } => {
            let rt = model.load()?;
            tritd::serve::serve(rt, &ServeOptions { addr, max_tokens })?;
        }

        Cmd::Bench {
            model,
            prompt,
            suite,
            tokens,
            warmup,
            runs,
            no_memcpy_probe,
            json,
        } => {
            let path = model.model.clone();
            let (prompts, suite_label) = match &suite {
                Some(p) => (read_suite(p)?, p.display().to_string()),
                None => (vec![prompt], "--prompt".to_string()),
            };
            let rt = model.load()?;
            let mut report = tritd::bench::run(
                &rt,
                &BenchOptions {
                    prompts,
                    suite: suite_label,
                    max_tokens: tokens,
                    warmup,
                    runs,
                    measure_memcpy: !no_memcpy_probe,
                },
            )?;
            report.model = path.display().to_string();
            print_bench(&report);
            if let Some(p) = json {
                std::fs::write(&p, serde_json::to_string_pretty(&report)?)?;
                eprintln!("wrote {}", p.display());
            }
        }

        Cmd::Info { model } => {
            let rt = model.load()?;
            let m = &rt.model;
            let cfg = m.config();
            println!("model           {}", model.model.display());
            println!("backend         {}", rt.backend_name);
            println!("cpu kernel      {}", rt.kernel_name);
            println!(
                "compiled        {}",
                backend::compiled_backends().join(", ")
            );
            println!(
                "kernels here    {}",
                trit_cpu::available_kernels().join(", ")
            );
            println!("layers          {}", cfg.num_layers);
            println!(
                "hidden / ff     {} / {}",
                cfg.hidden_size, cfg.intermediate_size
            );
            println!("heads / kv      {} / {}", cfg.num_heads, cfg.num_kv_heads);
            println!("vocab           {}", cfg.vocab_size);
            println!("context         {}", cfg.max_seq);
            println!("numerics        {:?}", m.numerics());
            println!("tied embeddings {}", m.tied_embeddings());
            println!(
                "bytes/token     {} ({:.0} MB ternary + {:.0} MB lm_head)",
                m.weight_bytes(),
                m.ternary_bytes() as f64 / 1e6,
                m.lm_head_bytes() as f64 / 1e6
            );
        }
    }
    Ok(())
}

fn print_bench(r: &BenchReport) {
    println!("backend         {} ({} threads)", r.backend, r.threads);
    println!("cpu kernel      {}", r.kernel);
    println!("prompt          {} tokens", r.prompt_tokens);
    println!("decoded         {} tokens", r.decoded_tokens);
    println!("ttft            {:.1} ms", r.ttft_ms);
    println!("decode          {:.2} tok/s", r.decode_tok_per_s);
    println!(
        "bytes/token     {} ({:.0} MB ternary + {:.0} MB lm_head), kv {:.0} MB",
        r.weight_bytes_per_token,
        r.ternary_bytes as f64 / 1e6,
        r.lm_head_bytes as f64 / 1e6,
        r.kv_bytes as f64 / 1e6
    );
    println!("achieved        {:.1} GB/s", r.achieved_gbps);
    match (r.memcpy_gbps, r.roofline_pct) {
        (Some(m), Some(p)) => println!("roofline        {:.1} GB/s measured -> {:.0}% of it", m, p),
        _ => println!("roofline        not measured"),
    }
    match r.peak_rss_mb {
        Some(v) => println!("peak rss        {v} MB"),
        None => println!("peak rss        unavailable"),
    }
    match r.joules_per_token {
        Some(j) => println!(
            "energy          {:.3} J/token (source: {})",
            j, r.energy_source
        ),
        // Never estimated: an invented energy number would poison the headline
        // claim this project exists to make.
        None => println!("energy          unavailable (no counter on this host)"),
    }
}
