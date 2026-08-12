use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tritsim::backend::{set_backend, Backend};
use tritsim::model::Model;

#[derive(Parser)]
#[command(name = "tritsim", about = "tritium golden-model inference")]
struct Cli {
    /// Matvec backend: cpu, or rtl (Verilated core; needs --features rtl build)
    #[arg(long, global = true, default_value = "cpu")]
    backend: String,
    #[command(subcommand)]
    cmd: Cmd,
}

fn select_backend(name: &str) -> Result<()> {
    match name {
        "cpu" => set_backend(Backend::Cpu),
        #[cfg(feature = "rtl")]
        "rtl" => set_backend(Backend::Rtl),
        #[cfg(not(feature = "rtl"))]
        "rtl" => anyhow::bail!("this binary was built without --features rtl"),
        other => anyhow::bail!("unknown backend {other}"),
    }
    Ok(())
}

#[derive(Subcommand)]
enum Cmd {
    /// Run greedy generation
    Run {
        #[arg(long)]
        model: PathBuf,
        #[arg(long)]
        tokenizer: PathBuf,
        #[arg(long)]
        prompt: String,
        #[arg(long, default_value_t = 64)]
        steps: usize,
    },
    /// Compare per-position logits against a reference dump (scripts/dump_logits.py)
    Compare {
        #[arg(long)]
        model: PathBuf,
        #[arg(long)]
        dump: PathBuf,
    },
    /// Emit golden vector sets for the RTL testbench
    Vectors {
        #[arg(long, default_value = "rtl/vectors")]
        out: PathBuf,
        /// Also cut a tile of real weights from this .trit model
        #[arg(long)]
        model: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let result = run(Cli::parse());
    // Flush even when the command failed: a failed compare run should still
    // keep its collected TRITSIM_STATS report.
    tritsim::stats::flush()?;
    result
}

fn run(cli: Cli) -> Result<()> {
    select_backend(&cli.backend)?;
    match cli.cmd {
        Cmd::Run { model, tokenizer, prompt, steps } => {
            let m = Model::load(&model)?;
            let tk = tokenizers::Tokenizer::from_file(&tokenizer).map_err(anyhow::Error::msg)?;
            let eos = tk.token_to_id("<|eot_id|>").or_else(|| tk.token_to_id("</s>"));
            let text = tritsim::generate::generate(&m, &tk, &prompt, steps, eos)?;
            println!("{text}");
        }
        Cmd::Compare { model, dump } => {
            let m = Model::load(&model)?;
            let s = tritsim::compare::compare(&m, &dump)?;
            println!(
                "{} positions: mean cosine {:.4}, top1 match {:.1}%",
                s.positions, s.mean_cosine, s.top1_match_frac * 100.0
            );
            anyhow::ensure!(
                s.mean_cosine >= 0.98 && s.top1_match_frac >= 0.90,
                "below acceptance thresholds (cosine >= 0.98, top1 >= 0.90)"
            );
        }
        Cmd::Vectors { out, model } => {
            tritsim::vectors::generate_all(&out)?;
            if let Some(m) = model {
                tritsim::vectors::model_tile_set(&out, &m, "model_k_proj_l0", 8)?;
            }
            println!("vectors written to {}", out.display());
        }
    }
    Ok(())
}
