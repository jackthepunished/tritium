use crate::model::{KvCache, Model};
use anyhow::Result;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
struct Dump {
    prompt_ids: Vec<u32>,
    logits: Vec<Vec<f32>>,
}

pub struct CompareStats {
    pub positions: usize,
    pub mean_cosine: f32,
    pub top1_match_frac: f32,
}

pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    dot / (na * nb).max(1e-12)
}

fn argmax(l: &[f32]) -> usize {
    l.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)).unwrap().0
}

pub fn compare(model: &Model, dump_path: &Path) -> Result<CompareStats> {
    let d: Dump = serde_json::from_str(&std::fs::read_to_string(dump_path)?)?;
    let mut cache = KvCache::new(&model.cfg);
    let (mut cos_sum, mut top1) = (0f32, 0usize);
    for (pos, tok) in d.prompt_ids.iter().enumerate() {
        let ours = model.forward(*tok, pos, &mut cache);
        let theirs = &d.logits[pos];
        let c = cosine(&ours, theirs);
        // centered cosine removes the uncentered-mean component that can
        // dominate raw cosine over a 128k vocab
        let center = |v: &[f32]| {
            let m = v.iter().sum::<f32>() / v.len() as f32;
            v.iter().map(|x| x - m).collect::<Vec<f32>>()
        };
        let cc = cosine(&center(&ours), &center(theirs));
        let top3 = |v: &[f32]| {
            let mut ix: Vec<usize> = (0..v.len()).collect();
            ix.sort_by(|&a, &b| v[b].total_cmp(&v[a]));
            ix[..3].iter().map(|&i| (i, v[i])).collect::<Vec<_>>()
        };
        // Near-tie rule: the reference runs in bf16, so when its own top-2
        // logits sit within NEAR_TIE_MARGIN of each other the argmax is a
        // coin flip between implementations. Count the position as matched
        // if our argmax lands in that tied top-2.
        const NEAR_TIE_MARGIN: f32 = 0.25;
        let t3 = top3(theirs);
        let ours_top = argmax(&ours);
        let exact = ours_top == t3[0].0;
        let near_tie = (t3[0].1 - t3[1].1) < NEAR_TIE_MARGIN && ours_top == t3[1].0;
        let m = exact || near_tie;
        println!(
            "pos {pos:3} cosine {c:.4} centered {cc:.4} top1_match {m}{}",
            if near_tie { " (near-tie)" } else { "" }
        );
        println!("      ours   {:?}", top3(&ours));
        println!("      theirs {:?}", t3);
        cos_sum += c;
        top1 += m as usize;
    }
    let n = d.prompt_ids.len();
    Ok(CompareStats {
        positions: n,
        mean_cosine: cos_sum / n as f32,
        top1_match_frac: top1 as f32 / n as f32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosine_of_identical_vectors_is_one() {
        assert!((cosine(&[1.0, 2.0, 3.0], &[1.0, 2.0, 3.0]) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn cosine_of_orthogonal_vectors_is_zero() {
        assert!(cosine(&[1.0, 0.0], &[0.0, 1.0]).abs() < 1e-6);
    }
}
