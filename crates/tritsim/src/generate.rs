use crate::model::{KvCache, Model};
use anyhow::Result;

fn argmax(l: &[f32]) -> u32 {
    l.iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .unwrap()
        .0 as u32
}

/// Feed `prompt_ids`, then greedily decode `steps` tokens (stops early on eos).
pub fn greedy_ids(model: &Model, prompt_ids: &[u32], steps: usize, eos_id: Option<u32>) -> Result<Vec<u32>> {
    anyhow::ensure!(!prompt_ids.is_empty(), "empty prompt");
    let mut cache = KvCache::new(&model.cfg);
    let mut logits = Vec::new();
    for (pos, tok) in prompt_ids.iter().enumerate() {
        logits = model.forward(*tok, pos, &mut cache);
    }
    let mut out = Vec::new();
    let mut pos = prompt_ids.len();
    for _ in 0..steps {
        let next = argmax(&logits);
        if Some(next) == eos_id {
            break;
        }
        out.push(next);
        logits = model.forward(next, pos, &mut cache);
        pos += 1;
    }
    Ok(out)
}

pub fn generate(
    model: &Model,
    tokenizer: &tokenizers::Tokenizer,
    prompt: &str,
    steps: usize,
    eos_id: Option<u32>,
) -> Result<String> {
    let enc = tokenizer.encode(prompt, true).map_err(anyhow::Error::msg)?;
    let ids = greedy_ids(model, enc.get_ids(), steps, eos_id)?;
    tokenizer.decode(&ids, true).map_err(anyhow::Error::msg)
}
