//! Model configuration, parsed from the `config.json` embedded in a `.trit`.

use anyhow::Result;
use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    Silu,
    Relu2,
}

/// Numerics ladder, ordered so the invalid "integer MLP without norm folding"
/// state is unrepresentable: `IntMlp` extends `Folded` extends `Reference`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Numerics {
    /// Textbook path: rmsnorm materialized, activations quantized per matvec.
    Reference,
    /// The 1/rms scalar is folded out of the element datapath.
    Folded,
    /// The squared-ReLU stage is computed in i64; the f32 activation vector for
    /// the down projection is never materialized.
    #[default]
    IntMlp,
}

impl Numerics {
    pub fn folded(self) -> bool {
        !matches!(self, Numerics::Reference)
    }
    pub fn int_mlp(self) -> bool {
        matches!(self, Numerics::IntMlp)
    }

    /// The best rung this architecture actually supports. The integer MLP path
    /// is relu2-specific and needs per-layer FFN sub-norms.
    pub fn best_for(cfg: &ModelConfig, has_ffn_sub_norm: bool) -> Self {
        if cfg.act == Act::Relu2 && has_ffn_sub_norm {
            Numerics::IntMlp
        } else {
            Numerics::Folded
        }
    }
}

impl std::str::FromStr for Numerics {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        Ok(match s {
            "reference" => Numerics::Reference,
            "folded" => Numerics::Folded,
            "intmlp" | "int-mlp" => Numerics::IntMlp,
            other => anyhow::bail!("unknown numerics mode {other:?} (reference|folded|intmlp)"),
        })
    }
}

#[derive(Deserialize)]
struct Raw {
    hidden_size: usize,
    intermediate_size: usize,
    num_hidden_layers: usize,
    num_attention_heads: usize,
    num_key_value_heads: Option<usize>,
    vocab_size: usize,
    rope_theta: Option<f32>,
    rms_norm_eps: Option<f32>,
    hidden_act: Option<String>,
    max_position_embeddings: Option<usize>,
    tie_word_embeddings: Option<bool>,
}

#[derive(Clone, Debug)]
pub struct ModelConfig {
    pub hidden_size: usize,
    pub intermediate_size: usize,
    pub num_layers: usize,
    pub num_heads: usize,
    pub num_kv_heads: usize,
    pub vocab_size: usize,
    pub rope_theta: f32,
    pub rms_eps: f32,
    pub act: Act,
    pub max_seq: usize,
    pub tie_word_embeddings: bool,
}

impl ModelConfig {
    pub fn from_json(s: &str) -> Result<Self> {
        let r: Raw = serde_json::from_str(s)?;
        anyhow::ensure!(
            r.hidden_size > 0 && r.num_attention_heads > 0,
            "degenerate config"
        );
        anyhow::ensure!(
            r.hidden_size.is_multiple_of(r.num_attention_heads),
            "hidden_size {} is not divisible by num_attention_heads {}",
            r.hidden_size,
            r.num_attention_heads
        );
        let act = match r.hidden_act.as_deref() {
            Some("relu2") | Some("relu-squared") | Some("relu_squared") => Act::Relu2,
            _ => Act::Silu,
        };
        Ok(Self {
            hidden_size: r.hidden_size,
            intermediate_size: r.intermediate_size,
            num_layers: r.num_hidden_layers,
            num_heads: r.num_attention_heads,
            num_kv_heads: r.num_key_value_heads.unwrap_or(r.num_attention_heads),
            vocab_size: r.vocab_size,
            rope_theta: r.rope_theta.unwrap_or(10000.0),
            rms_eps: r.rms_norm_eps.unwrap_or(1e-5),
            act,
            // The KV cache is preallocated, so context is capped at 2048; never
            // claim more than the checkpoint itself supports.
            max_seq: 2048.min(r.max_position_embeddings.unwrap_or(2048)),
            tie_word_embeddings: r.tie_word_embeddings.unwrap_or(false),
        })
    }

    pub fn head_dim(&self) -> usize {
        self.hidden_size / self.num_heads
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hf_config() {
        let cfg = ModelConfig::from_json(
            r#"{"hidden_size":64,"intermediate_size":128,"num_hidden_layers":2,
                "num_attention_heads":4,"num_key_value_heads":2,"vocab_size":100,
                "rope_theta":10000.0,"rms_norm_eps":1e-5,"hidden_act":"relu2",
                "tie_word_embeddings":true}"#,
        )
        .unwrap();
        assert_eq!(cfg.num_layers, 2);
        assert_eq!(cfg.num_kv_heads, 2);
        assert_eq!(cfg.act, Act::Relu2);
        assert_eq!(cfg.max_seq, 2048);
        assert_eq!(cfg.head_dim(), 16);
        assert!(cfg.tie_word_embeddings);
    }

    #[test]
    fn rejects_a_head_split_that_does_not_divide() {
        assert!(ModelConfig::from_json(
            r#"{"hidden_size":10,"intermediate_size":8,"num_hidden_layers":1,
                "num_attention_heads":4,"vocab_size":8}"#
        )
        .is_err());
    }

    #[test]
    fn numerics_ladder_is_ordered() {
        assert!(Numerics::Reference < Numerics::Folded);
        assert!(Numerics::Folded < Numerics::IntMlp);
        assert!(!Numerics::Reference.folded());
        assert!(Numerics::Folded.folded() && !Numerics::Folded.int_mlp());
        assert!(Numerics::IntMlp.folded() && Numerics::IntMlp.int_mlp());
    }
}
