//! Tritium host runtime.
//!
//! Loads a `.trit` model, selects a compute backend, and streams tokens. The
//! Rust API here is what the CLI, the HTTP server and the C ABI all sit on.

pub mod backend;
pub mod bench;
pub mod ffi;
pub mod serve;
pub mod stream;
pub mod tokenizer_hf;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use trit_core::model::Model;
use trit_core::tokenizer::Tokenizer as _;
use trit_core::tritfmt::TritFile;

pub use stream::{Token, TokenStream};
pub use tokenizer_hf::HfTokenizer;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// How to bring a model up.
#[derive(Clone, Debug)]
pub struct LoadOptions {
    pub model: PathBuf,
    /// Defaults to `tokenizer.json` beside the model.
    pub tokenizer: Option<PathBuf>,
    pub backend: String,
    /// `0` means one thread per core.
    pub threads: usize,
    /// Pin a specific CPU kernel; `None` auto-detects.
    pub kernel: Option<String>,
    /// Walk the whole payload at load: plane invariants and the payload hash.
    pub verify: bool,
}

impl Default for LoadOptions {
    fn default() -> Self {
        Self {
            model: PathBuf::new(),
            tokenizer: None,
            backend: "cpu".into(),
            threads: 0,
            kernel: None,
            verify: false,
        }
    }
}

/// A loaded model and its tokenizer.
pub struct Runtime {
    pub model: Arc<Model>,
    pub tokenizer: HfTokenizer,
    pub backend_name: String,
    pub kernel_name: &'static str,
}

impl Runtime {
    pub fn load(opts: &LoadOptions) -> Result<Self> {
        if let Some(k) = &opts.kernel {
            // Errors rather than falling back: benchmarking or testing a kernel
            // the CPU cannot run must fail loudly, not silently measure another.
            trit_cpu::force_kernel(k).map_err(anyhow::Error::msg)?;
        }

        let file = if opts.verify {
            TritFile::open_verified(&opts.model)?
        } else {
            TritFile::open(&opts.model)?
        };

        let backend = backend::make_backend(&opts.backend, opts.threads)?;
        let backend_name = backend.name().to_string();
        let model = Model::load(file, backend)?;

        let tk_path = opts
            .tokenizer
            .clone()
            .unwrap_or_else(|| default_tokenizer_path(&opts.model));
        let tokenizer = HfTokenizer::from_file(&tk_path)?;

        anyhow::ensure!(
            tokenizer.vocab_size() <= model.config().vocab_size,
            "tokenizer vocab ({}) exceeds the model's ({}) -- mismatched tokenizer?",
            tokenizer.vocab_size(),
            model.config().vocab_size
        );

        Ok(Self {
            model,
            tokenizer,
            backend_name,
            kernel_name: trit_cpu::kernel_name(),
        })
    }
}

/// Find `tokenizer.json` for a model, without being told where it is.
///
/// Checked in order:
///   1. beside the model      -- models/m.trit      -> models/tokenizer.json
///   2. the checkpoint dir    -- models/m.trit      -> models/m/tokenizer.json
///   3. beside it, one level down by stem, as `hf` layouts often leave it
///
/// Falls back to (1) so the error message names a concrete path rather than
/// complaining abstractly that nothing was found.
pub fn default_tokenizer_path(model: &Path) -> PathBuf {
    let dir = model.parent().unwrap_or(Path::new("."));
    let beside = dir.join("tokenizer.json");
    if beside.exists() {
        return beside;
    }
    if let Some(stem) = model.file_stem() {
        let in_checkpoint_dir = dir.join(stem).join("tokenizer.json");
        if in_checkpoint_dir.exists() {
            return in_checkpoint_dir;
        }
    }
    beside
}

/// Read a prompt from a literal, a file, or stdin (`-`).
pub fn read_prompt(literal: Option<&str>, file: Option<&Path>) -> Result<String> {
    match (literal, file) {
        (Some(s), None) => Ok(s.to_string()),
        (None, Some(p)) if p == Path::new("-") => {
            use std::io::Read;
            let mut s = String::new();
            std::io::stdin()
                .read_to_string(&mut s)
                .context("read prompt from stdin")?;
            Ok(s)
        }
        (None, Some(p)) => {
            std::fs::read_to_string(p).with_context(|| format!("read prompt from {}", p.display()))
        }
        (Some(_), Some(_)) => anyhow::bail!("give either --prompt or --prompt-file, not both"),
        (None, None) => anyhow::bail!("a prompt is required (--prompt or --prompt-file)"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizer_path_falls_back_to_beside_the_model() {
        // Nothing exists at either candidate, so the fallback names a concrete
        // path for the error message.
        assert_eq!(
            default_tokenizer_path(Path::new("/nonexistent/m.trit")),
            PathBuf::from("/nonexistent/tokenizer.json")
        );
    }

    #[test]
    fn tokenizer_path_finds_the_checkpoint_directory() {
        let dir = std::env::temp_dir().join("tritd_tok_lookup");
        let ckpt = dir.join("mymodel");
        std::fs::create_dir_all(&ckpt).unwrap();
        let tok = ckpt.join("tokenizer.json");
        std::fs::write(&tok, "{}").unwrap();
        // models/mymodel.trit -> models/mymodel/tokenizer.json
        assert_eq!(default_tokenizer_path(&dir.join("mymodel.trit")), tok);

        // A tokenizer sitting beside the model wins.
        let beside = dir.join("tokenizer.json");
        std::fs::write(&beside, "{}").unwrap();
        assert_eq!(default_tokenizer_path(&dir.join("mymodel.trit")), beside);
        std::fs::remove_file(&beside).ok();
    }

    #[test]
    fn prompt_sources_are_mutually_exclusive() {
        assert!(read_prompt(Some("hi"), None).is_ok());
        assert!(read_prompt(Some("hi"), Some(Path::new("f"))).is_err());
        assert!(read_prompt(None, None).is_err());
    }
}
