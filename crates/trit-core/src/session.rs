//! A decode session: prompt in, tokens out.
//!
//! Owns the KV cache, the scratch buffers and the sampler, so the CLI, the HTTP
//! server and the C ABI all drive one interface and cannot drift.

use std::sync::Arc;

use anyhow::Result;

use crate::config::Numerics;
use crate::kv::KvCache;
use crate::model::{Model, Scratch};
use crate::sampler::{make_sampler, Sampler, SamplerParams};

pub struct Session {
    model: Arc<Model>,
    cache: KvCache,
    scratch: Scratch,
    sampler: Box<dyn Sampler>,
    logits: Vec<f32>,
    history: Vec<u32>,
    pos: usize,
}

impl Session {
    pub fn new(model: Arc<Model>, params: &SamplerParams) -> Self {
        let cfg = model.config();
        Self {
            cache: KvCache::new(cfg),
            scratch: Scratch::new(cfg),
            logits: vec![0.0; cfg.vocab_size],
            history: Vec::new(),
            pos: 0,
            sampler: make_sampler(params),
            model,
        }
    }

    pub fn model(&self) -> &Arc<Model> {
        &self.model
    }
    pub fn pos(&self) -> usize {
        self.pos
    }
    /// Positions left before the context is full.
    pub fn remaining(&self) -> usize {
        self.model.config().max_seq.saturating_sub(self.pos)
    }
    pub fn numerics(&self) -> Numerics {
        self.model.numerics()
    }
    pub fn kv_bytes(&self) -> u64 {
        self.cache.bytes()
    }
    pub fn logits(&self) -> &[f32] {
        &self.logits
    }
    pub fn history(&self) -> &[u32] {
        &self.history
    }

    /// Forget the conversation without reallocating.
    pub fn reset(&mut self) {
        self.cache.reset();
        self.history.clear();
        self.pos = 0;
    }

    /// Run the prompt through, returning the logits at the last position.
    pub fn prefill(&mut self, ids: &[u32]) -> Result<&[f32]> {
        anyhow::ensure!(!ids.is_empty(), "prompt is empty");
        anyhow::ensure!(
            ids.len() <= self.remaining(),
            "prompt is {} tokens but only {} positions remain",
            ids.len(),
            self.remaining()
        );
        for &t in ids {
            self.step_raw(t)?;
        }
        Ok(&self.logits)
    }

    /// Feed one token and produce the next logits.
    pub fn advance(&mut self, token: u32) -> Result<&[f32]> {
        anyhow::ensure!(self.remaining() > 0, "context is full");
        self.step_raw(token)?;
        Ok(&self.logits)
    }

    fn step_raw(&mut self, token: u32) -> Result<()> {
        self.model.forward_into(
            token,
            self.pos,
            &mut self.cache,
            &mut self.scratch,
            &mut self.logits,
        )?;
        self.history.push(token);
        self.pos += 1;
        Ok(())
    }

    /// Sample from the current logits. Does not advance the model.
    pub fn sample(&mut self) -> u32 {
        self.sampler.sample(&mut self.logits, &self.history)
    }

    /// Sample the next token and feed it back in. `None` once the context is
    /// full.
    pub fn next_token(&mut self) -> Result<Option<u32>> {
        let t = self.sample();
        if self.remaining() == 0 {
            return Ok(None);
        }
        self.advance(t)?;
        Ok(Some(t))
    }
}
