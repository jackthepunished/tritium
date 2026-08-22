//! Streaming token generation.
//!
//! One interface for the CLI, the HTTP server and the C ABI, so none of them can
//! drift from the others.

use std::sync::Arc;

use anyhow::Result;
use trit_core::model::Model;
use trit_core::sampler::SamplerParams;
use trit_core::session::Session;
use trit_core::tokenizer::{Detokenizer, Tokenizer};

/// One generated token and the text it contributes.
///
/// `text` can be empty: byte-level BPE splits multi-byte codepoints across
/// tokens, and the detokenizer holds an incomplete tail until it completes.
#[derive(Clone, Debug)]
pub struct Token {
    pub id: u32,
    pub text: String,
}

pub struct TokenStream<'a> {
    session: Session,
    tokenizer: &'a dyn Tokenizer,
    detok: Detokenizer,
    remaining: usize,
    finished: bool,
    /// Wall time of the prefill pass, for reporting.
    pub prefill_secs: f64,
    pub prompt_tokens: usize,
}

impl<'a> TokenStream<'a> {
    /// Prefill the prompt and prepare to emit `max_tokens` tokens.
    pub fn new(
        model: Arc<Model>,
        tokenizer: &'a dyn Tokenizer,
        params: &SamplerParams,
        prompt_ids: &[u32],
        max_tokens: usize,
    ) -> Result<Self> {
        let mut session = Session::new(model, params);
        let t0 = std::time::Instant::now();
        session.prefill(prompt_ids)?;
        let prefill_secs = t0.elapsed().as_secs_f64();
        // Never ask for more positions than the context has left.
        let remaining = max_tokens.min(session.remaining());
        Ok(Self {
            session,
            tokenizer,
            detok: Detokenizer::new(),
            remaining,
            finished: false,
            prefill_secs,
            prompt_tokens: prompt_ids.len(),
        })
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    /// Anything the detokenizer is still holding at end of stream.
    pub fn flush(&mut self) -> String {
        self.detok.flush()
    }
}

impl Iterator for TokenStream<'_> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished || self.remaining == 0 {
            return None;
        }
        let id = self.session.sample();
        if self.tokenizer.is_eos(id) {
            self.finished = true;
            return None;
        }
        // Feed the token back in so the next call has fresh logits.
        if let Err(e) = self.session.advance(id) {
            self.finished = true;
            return Some(Err(e));
        }
        self.remaining -= 1;
        match self.detok.push(self.tokenizer, id) {
            Ok(text) => Some(Ok(Token { id, text })),
            Err(e) => {
                self.finished = true;
                Some(Err(e))
            }
        }
    }
}
