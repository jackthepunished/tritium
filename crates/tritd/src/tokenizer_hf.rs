//! HuggingFace `tokenizers` behind the core's `Tokenizer` trait.

use anyhow::{Context, Result};
use trit_core::tokenizer::Tokenizer;

pub struct HfTokenizer {
    inner: tokenizers::Tokenizer,
    eos: Vec<u32>,
}

/// End-of-sequence names seen across the checkpoints this runtime targets.
const EOS_CANDIDATES: &[&str] = &["<|eot_id|>", "</s>", "<|end_of_text|>", "<|endoftext|>"];

impl HfTokenizer {
    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let inner = tokenizers::Tokenizer::from_file(path)
            .map_err(anyhow::Error::msg)
            .with_context(|| format!("load tokenizer {}", path.display()))?;
        let eos = EOS_CANDIDATES
            .iter()
            .filter_map(|n| inner.token_to_id(n))
            .collect();
        Ok(Self { inner, eos })
    }
}

impl Tokenizer for HfTokenizer {
    fn encode(&self, text: &str, add_special: bool) -> Result<Vec<u32>> {
        Ok(self
            .inner
            .encode(text, add_special)
            .map_err(anyhow::Error::msg)?
            .get_ids()
            .to_vec())
    }

    fn decode(&self, ids: &[u32], skip_special: bool) -> Result<String> {
        self.inner
            .decode(ids, skip_special)
            .map_err(anyhow::Error::msg)
    }

    fn id_to_bytes(&self, id: u32) -> Result<Vec<u8>> {
        // Decode without skipping special tokens and without cleanup, so the
        // bytes are exactly what this token contributes; the Detokenizer
        // reassembles codepoints split across token boundaries.
        Ok(self
            .inner
            .decode(&[id], false)
            .map_err(anyhow::Error::msg)?
            .into_bytes())
    }

    fn eos_ids(&self) -> &[u32] {
        &self.eos
    }

    fn vocab_size(&self) -> usize {
        self.inner.get_vocab_size(true)
    }
}
