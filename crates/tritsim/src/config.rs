use crate::math::Act;
use anyhow::Result;
use serde::Deserialize;

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
}

impl ModelConfig {
    pub fn from_json(s: &str) -> Result<Self> {
        let r: Raw = serde_json::from_str(s)?;
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
            max_seq: 2048,
        })
    }
    pub fn head_dim(&self) -> usize {
        self.hidden_size / self.num_heads
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Act;

    #[test]
    fn parses_hf_config() {
        let cfg = ModelConfig::from_json(
            r#"{"hidden_size":64,"intermediate_size":128,"num_hidden_layers":2,
                "num_attention_heads":4,"num_key_value_heads":2,"vocab_size":100,
                "rope_theta":10000.0,"rms_norm_eps":1e-5,"hidden_act":"relu2"}"#,
        )
        .unwrap();
        assert_eq!(cfg.num_layers, 2);
        assert_eq!(cfg.num_kv_heads, 2);
        assert_eq!(cfg.act, Act::Relu2);
        assert_eq!(cfg.max_seq, 2048);
    }
}
