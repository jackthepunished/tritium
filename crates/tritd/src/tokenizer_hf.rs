//! HuggingFace `tokenizers` behind the core's `Tokenizer` trait.

use std::collections::HashMap;

use anyhow::{Context, Result};
use tokenizers::decoders::DecoderWrapper;
use trit_core::tokenizer::Tokenizer;

pub struct HfTokenizer {
    inner: tokenizers::Tokenizer,
    eos: Vec<u32>,
    /// Byte-level alphabet, inverted: the character a vocabulary token is
    /// spelled with, back to the byte it stands for.
    ///
    /// `Some` only when this tokenizer actually decodes byte-level. See
    /// [`HfTokenizer::id_to_bytes`] for why the mapping is applied here rather
    /// than delegated to `Tokenizer::decode`.
    byte_of: Option<HashMap<char, u8>>,
}

/// End-of-sequence names seen across the checkpoints this runtime targets.
const EOS_CANDIDATES: &[&str] = &["<|eot_id|>", "</s>", "<|end_of_text|>", "<|endoftext|>"];

/// The GPT-2 byte-level alphabet, as `char -> byte`.
///
/// Byte-level BPE spells every one of the 256 byte values as a single printable
/// character, so a vocabulary entry is a string even when it stands for bytes
/// that are not valid UTF-8 on their own. The bijection is the one from GPT-2:
/// bytes that are already printable (`!`..=`~`, `¡`..=`¬`, `®`..=`ÿ`) map to
/// themselves, and the remaining 68 are assigned `U+0100` upwards in ascending
/// byte order.
fn byte_level_alphabet() -> HashMap<char, u8> {
    let printable = |b: u8| (b'!'..=b'~').contains(&b) || (0xA1..=0xAC).contains(&b) || b >= 0xAE;
    let mut map = HashMap::with_capacity(256);
    let mut n = 0u32;
    for b in 0..=u8::MAX {
        let c = if printable(b) {
            b as char
        } else {
            let c = char::from_u32(256 + n).expect("in range");
            n += 1;
            c
        };
        map.insert(c, b);
    }
    map
}

/// Does this tokenizer's decoder chain go through `ByteLevel`?
fn is_byte_level(d: Option<&DecoderWrapper>) -> bool {
    match d {
        Some(DecoderWrapper::ByteLevel(_)) => true,
        Some(DecoderWrapper::Sequence(s)) => {
            s.get_decoders().iter().any(|d| is_byte_level(Some(d)))
        }
        _ => false,
    }
}

impl HfTokenizer {
    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let inner = tokenizers::Tokenizer::from_file(path)
            .map_err(anyhow::Error::msg)
            .with_context(|| format!("load tokenizer {}", path.display()))?;
        let eos = EOS_CANDIDATES
            .iter()
            .filter_map(|n| inner.token_to_id(n))
            .collect();
        let byte_of = is_byte_level(inner.get_decoder()).then(byte_level_alphabet);
        Ok(Self {
            inner,
            eos,
            byte_of,
        })
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

    /// The exact bytes this one token contributes.
    ///
    /// **Not** `decode(&[id])`. `ByteLevel::decode_chain` finishes with
    /// `String::from_utf8_lossy`, so a token holding only part of a codepoint
    /// comes back as U+FFFD and the original bytes are gone before
    /// `Detokenizer` ever sees them -- which defeats the entire point of
    /// holding an incomplete tail. On the 2B4T vocabulary, U+1F600 splits as
    /// token 76460 (`f0 9f 98`) plus token 222 (`80`), and decoding each alone
    /// yields `ef bf bd` twice: the streamed text is two replacement characters
    /// rather than an emoji.
    ///
    /// So for a byte-level tokenizer the vocabulary entry is mapped back
    /// through the alphabet directly, which is lossless by construction.
    /// Anything else -- a non-byte-level tokenizer, or an added token spelled
    /// with characters outside the alphabet -- falls back to `decode`, which is
    /// correct for tokens that stand for whole codepoints.
    fn id_to_bytes(&self, id: u32) -> Result<Vec<u8>> {
        if let (Some(map), Some(token)) = (self.byte_of.as_ref(), self.inner.id_to_token(id)) {
            let mut out = Vec::with_capacity(token.len());
            if token.chars().all(|c| match map.get(&c) {
                Some(b) => {
                    out.push(*b);
                    true
                }
                None => false,
            }) {
                return Ok(out);
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alphabet_is_the_gpt2_bijection() {
        let map = byte_level_alphabet();
        assert_eq!(map.len(), 256, "must be a bijection over all byte values");

        // Printable bytes stand for themselves.
        assert_eq!(map[&'!'], 0x21);
        assert_eq!(map[&'~'], 0x7E);
        assert_eq!(map[&'ð'], 0xF0);

        // The 68 non-printable bytes take U+0100 upwards in ascending byte
        // order: 0x00..=0x20 (33 of them), then 0x7F..=0xA0, then 0xAD.
        assert_eq!(map[&'\u{0100}'], 0x00);
        assert_eq!(map[&'\u{0120}'], 0x20); // space, the leading-space marker
        assert_eq!(map[&'\u{0121}'], 0x7F);
        assert_eq!(map[&'\u{0122}'], 0x80);
        assert_eq!(map[&'\u{0143}'], 0xAD); // the last one
    }

    /// The bytes of U+1F600, spelled the way the 2B4T vocabulary spells them.
    /// This is the case `decode(&[id])` destroys.
    #[test]
    fn alphabet_recovers_a_split_codepoint() {
        let map = byte_level_alphabet();
        let bytes = |s: &str| -> Vec<u8> { s.chars().map(|c| map[&c]).collect() };
        // token 76460 then token 222, as tokenized by the real vocabulary.
        assert_eq!(bytes("ðŁĺ"), vec![0xF0, 0x9F, 0x98]);
        assert_eq!(bytes("Ģ"), vec![0x80]);

        let mut all = bytes("ðŁĺ");
        all.extend(bytes("Ģ"));
        assert_eq!(String::from_utf8(all).unwrap(), "\u{1F600}");
    }
}
