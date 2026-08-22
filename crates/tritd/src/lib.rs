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

        let tk_path = opts.tokenizer.clone().unwrap_or_else(|| default_tokenizer_path(&opts.model));
        let tokenizer = HfTokenizer::from_file(&tk_path)?;

        anyhow::ensure!(
            tokenizer.vocab_size() <= model.config().vocab_size,
            "tokenizer vocab ({}) exceeds the model's ({}) -- mismatched tokenizer?",
            tokenizer.vocab_size(),
            model.config().vocab_size
        );

        Ok(Self { model, tokenizer, backend_name, kernel_name: trit_cpu::kernel_name() })
    }
}

/// `tokenizer.json` beside the model file, which is where `tritc convert`
/// output and a downloaded checkpoint both put it.
pub fn default_tokenizer_path(model: &Path) -> PathBuf {
    model.parent().unwrap_or(Path::new(".")).join("tokenizer.json")
}

/// Read a prompt from a literal, a file, or stdin (`-`).
pub fn read_prompt(literal: Option<&str>, file: Option<&Path>) -> Result<String> {
    match (literal, file) {
        (Some(s), None) => Ok(s.to_string()),
        (None, Some(p)) if p == Path::new("-") => {
            use std::io::Read;
            let mut s = String::new();
            std::io::stdin().read_to_string(&mut s).context("read prompt from stdin")?;
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
    fn tokenizer_path_defaults_beside_the_model() {
        assert_eq!(
            default_tokenizer_path(Path::new("/models/m.trit")),
            PathBuf::from("/models/tokenizer.json")
        );
        assert_eq!(default_tokenizer_path(Path::new("m.trit")), PathBuf::from("tokenizer.json"));
    }

    #[test]
    fn prompt_sources_are_mutually_exclusive() {
        assert!(read_prompt(Some("hi"), None).is_ok());
        assert!(read_prompt(Some("hi"), Some(Path::new("f"))).is_err());
        assert!(read_prompt(None, None).is_err());
    }
}
