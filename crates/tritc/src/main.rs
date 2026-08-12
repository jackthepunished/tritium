mod convert;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "tritc", about = "tritium model converter")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Convert a HF checkpoint directory (config.json + *.safetensors) to .trit
    Convert {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    match Cli::parse().cmd {
        Cmd::Convert { input, output } => {
            let r = convert::convert(&input, &output)?;
            println!(
                "converted {} tensors ({} ternary), mean zero frac {:.3}, mean recon err {:.4}",
                r.tensors, r.ternary_tensors, r.mean_zero_frac, r.mean_recon_err
            );
        }
    }
    Ok(())
}
