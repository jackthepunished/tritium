mod convert;
mod legacy_v0;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use trit_core::tritfmt::{TritFile, TritWriter};

#[derive(Parser)]
#[command(name = "tritc", about = "tritium model converter and .trit packager")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Convert a HF checkpoint directory (config.json + *.safetensors) to .trit v1
    Convert {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
        /// Re-open the written file and check every tensor round-trips
        #[arg(long)]
        verify: bool,
    },
    /// Re-encode a legacy .trit v0 file (2-bit codes) as v1 (bit planes)
    Upgrade {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Print the header, tensor table summary, and per-token weight footprint
    Info {
        #[arg(long)]
        model: PathBuf,
        /// List every tensor rather than a per-class summary
        #[arg(long)]
        tensors: bool,
    },
    /// Full payload scan: plane invariants and the payload hash
    Verify {
        #[arg(long)]
        model: PathBuf,
    },
}

fn main() -> Result<()> {
    match Cli::parse().cmd {
        Cmd::Convert {
            input,
            output,
            verify,
        } => {
            let r = convert::convert(&input, &output)?;
            println!(
                "converted {} tensors ({} ternary), mean zero frac {:.3}, mean recon err {:.4}",
                r.tensors, r.ternary_tensors, r.mean_zero_frac, r.mean_recon_err
            );
            if verify {
                let rep = TritFile::open(&output)?.verify()?;
                println!(
                    "verified: {} tensors, {} ternary, {} trits, zero fraction {:.4}, hash {}",
                    rep.tensors,
                    rep.ternary_tensors,
                    rep.total_trits,
                    rep.zero_fraction,
                    match rep.hash_ok {
                        Some(true) => "ok",
                        Some(false) => "MISMATCH",
                        None => "absent",
                    }
                );
            }
        }
        Cmd::Upgrade { input, output } => upgrade(&input, &output)?,
        Cmd::Info { model, tensors } => info(&model, tensors)?,
        Cmd::Verify { model } => {
            let rep = TritFile::open(&model)?.verify()?;
            println!(
                "{}: {} tensors, {} ternary, {} trits, zero fraction {:.4}, hash {}",
                model.display(),
                rep.tensors,
                rep.ternary_tensors,
                rep.total_trits,
                rep.zero_fraction,
                match rep.hash_ok {
                    Some(true) => "ok",
                    Some(false) => "MISMATCH",
                    None => "absent",
                }
            );
        }
    }
    Ok(())
}

/// Re-encode v0 to v1. Pure layout change: the same trits, the same scales, the
/// same tensor order -- so the result is byte-identical to a fresh `convert` of
/// the original checkpoint.
fn upgrade(input: &Path, output: &Path) -> Result<()> {
    let mut v0 = legacy_v0::read(input)?;
    // v0 files were written in the converter's old HashMap iteration order.
    // Sort by name so the upgrade lands in the same canonical layout a fresh
    // `convert` produces -- that byte-identity is the proof the migration is a
    // pure re-encoding and nothing was lost in it.
    v0.tensors.sort_by(|a, b| a.name.cmp(&b.name));
    let mut w = TritWriter::create(output, &v0.config)?;
    let (mut n, mut n_tern) = (0usize, 0usize);
    for t in &v0.tensors {
        match (&t.trits, &t.f32) {
            (Some(trits), _) => {
                w.write_trit(&t.name, &t.shape, trits, t.scale)
                    .with_context(|| format!("writing {}", t.name))?;
                n_tern += 1;
            }
            (_, Some(vals)) => w.write_f32(&t.name, &t.shape, vals)?,
            _ => anyhow::bail!("tensor {} has no data", t.name),
        }
        n += 1;
    }
    w.finish()?;
    println!(
        "upgraded {} -> {}: {n} tensors ({n_tern} ternary), v0 codes -> v1 bit planes",
        input.display(),
        output.display()
    );
    Ok(())
}

fn info(model: &Path, list: bool) -> Result<()> {
    let f = TritFile::open(model)?;
    let bytes = std::fs::metadata(model)?.len();
    println!("{}", model.display());
    println!("  file            {bytes} bytes");
    println!("  payload starts  {}", f.payload_start());
    println!("  tensors         {}", f.metas().len());

    let (mut tern_bytes, mut dense_bytes, mut tern_n, mut trits) = (0u64, 0u64, 0usize, 0u64);
    for m in f.metas() {
        let is_tern = f.trit_span(&m.name).is_ok();
        if is_tern {
            tern_bytes += m.byte_len();
            tern_n += 1;
            trits += m.elem_count()? as u64;
        } else {
            dense_bytes += m.byte_len();
        }
        if list {
            println!("    {:60} {:?} {:?}", m.name, m.dtype, m.shape);
        }
    }
    println!("  ternary         {tern_n} tensors, {trits} weights, {tern_bytes} bytes");
    println!(
        "  dense           {} tensors, {dense_bytes} bytes",
        f.metas().len() - tern_n,
    );
    if trits > 0 {
        println!(
            "  bits/weight     {:.3}",
            tern_bytes as f64 * 8.0 / trits as f64
        );
    }
    // Decoding at batch 1 touches every weight once per token, so this sum is
    // the numerator of the bandwidth roofline: tok/s ~= BW / bytes_per_token.
    println!(
        "  bytes/token     {} ({:.0} MB ternary + {:.0} MB dense)",
        tern_bytes + dense_bytes,
        tern_bytes as f64 / 1e6,
        dense_bytes as f64 / 1e6
    );
    Ok(())
}
