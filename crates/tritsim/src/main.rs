use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tritsim::model::Model;

#[derive(Parser)]
#[command(name = "tritsim", about = "tritium golden-model inference")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
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
}

fn main() -> Result<()> {
    match Cli::parse().cmd {
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
    }
    Ok(())
}
