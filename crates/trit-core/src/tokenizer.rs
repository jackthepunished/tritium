//! Tokenizer abstraction.
//!
//! A trait, not an implementation: the HuggingFace `tokenizers` crate pulls in a
//! C++ build and a thread pool, which the core runtime has no business
//! depending on. `tritd` provides the concrete implementation.

use anyhow::Result;

pub trait Tokenizer: Send + Sync {
    fn encode(&self, text: &str, add_special: bool) -> Result<Vec<u32>>;
    fn decode(&self, ids: &[u32], skip_special: bool) -> Result<String>;
    /// Raw bytes of a single token, for incremental streaming.
    fn id_to_bytes(&self, id: u32) -> Result<Vec<u8>>;
    fn eos_ids(&self) -> &[u32];
    fn vocab_size(&self) -> usize;

    fn is_eos(&self, id: u32) -> bool {
        self.eos_ids().contains(&id)
    }
}

/// Buffers partial UTF-8 across token boundaries.
///
/// Byte-level BPE routinely splits a multi-byte codepoint across two tokens.
/// Decoding each token independently and concatenating emits U+FFFD in the
/// middle of ordinary text -- most visibly on emoji and non-Latin scripts. This
/// holds the incomplete tail until the continuation bytes arrive.
#[derive(Default)]
pub struct Detokenizer {
    pending: Vec<u8>,
}

impl Detokenizer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Feed one token; returns whatever is now completely decodable.
    pub fn push(&mut self, tk: &dyn Tokenizer, id: u32) -> Result<String> {
        self.pending.extend_from_slice(&tk.id_to_bytes(id)?);
        Ok(self.take_complete())
    }

    pub fn push_bytes(&mut self, bytes: &[u8]) -> String {
        self.pending.extend_from_slice(bytes);
        self.take_complete()
    }

    fn take_complete(&mut self) -> String {
        let mut out = String::new();
        // Loop rather than return after the first error: a malformed byte in the
        // middle must not leave the perfectly decodable bytes after it stuck in
        // the buffer waiting for a continuation that will never come.
        loop {
            match std::str::from_utf8(&self.pending) {
                Ok(s) => {
                    out.push_str(s);
                    self.pending.clear();
                    return out;
                }
                Err(e) => {
                    let good = e.valid_up_to();
                    match e.error_len() {
                        // A genuinely invalid sequence. Emit the valid prefix
                        // plus one replacement character, drop the bad bytes,
                        // and keep going.
                        Some(bad) => {
                            out.push_str(std::str::from_utf8(&self.pending[..good]).unwrap());
                            out.push('\u{FFFD}');
                            self.pending.drain(..good + bad);
                        }
                        // A truncated tail: the continuation may still arrive,
                        // so emit the prefix and wait.
                        None => {
                            out.push_str(std::str::from_utf8(&self.pending[..good]).unwrap());
                            self.pending.drain(..good);
                            return out;
                        }
                    }
                }
            }
        }
    }

    /// Flush anything still buffered at end of stream, lossily.
    pub fn flush(&mut self) -> String {
        let out = String::from_utf8_lossy(&self.pending).into_owned();
        self.pending.clear();
        out
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn holds_a_split_codepoint_until_it_completes() {
        // U+1F600 GRINNING FACE is f0 9f 98 80; split it across three pushes.
        let mut d = Detokenizer::new();
        assert_eq!(d.push_bytes(&[0xf0, 0x9f]), "", "incomplete: emit nothing yet");
        assert_eq!(d.push_bytes(&[0x98]), "", "still incomplete");
        assert_eq!(d.push_bytes(&[0x80]), "\u{1F600}");
        assert!(d.is_empty());
    }

    #[test]
    fn emits_the_complete_prefix_and_keeps_the_tail() {
        let mut d = Detokenizer::new();
        // "ab" then the first two bytes of a three-byte codepoint.
        assert_eq!(d.push_bytes(b"ab"), "ab");
        assert_eq!(d.push_bytes(&[0xe2, 0x82]), "");
        assert_eq!(d.push_bytes(&[0xac]), "\u{20AC}"); // EURO SIGN
    }

    #[test]
    fn genuinely_invalid_bytes_do_not_stall_the_stream() {
        let mut d = Detokenizer::new();
        let out = d.push_bytes(&[0xff, b'x']);
        assert!(out.contains('\u{FFFD}'), "{out:?}");
        assert!(out.ends_with('x'));
        assert!(d.is_empty(), "must not wait forever on malformed input");
    }

    #[test]
    fn flush_is_lossy_but_terminates() {
        let mut d = Detokenizer::new();
        d.push_bytes(&[0xf0, 0x9f]);
        assert!(!d.is_empty());
        assert!(d.flush().contains('\u{FFFD}'));
        assert!(d.is_empty());
    }
}
