# BitNet b1.58 2B4T bf16 checkpoint notes (Task 5 inspection)

Source: microsoft/bitnet-b1.58-2B-4T-bf16, inspected 2026-08-12.

From config.json:

| Field | Value |
|---|---|
| model_type | bitnet (BitNetForCausalLM) |
| hidden_size | 2560 |
| intermediate_size | 6912 |
| num_hidden_layers | 30 |
| num_attention_heads | 20 (head_dim 128) |
| num_key_value_heads | 5 (GQA 4:1) |
| rope_theta | 500000.0 |
| rms_norm_eps | 1e-05 |
| vocab_size | 128256 |
| hidden_act | relu2 (squared ReLU, matches Act::Relu2 mapping) |
| tie_word_embeddings | true (no lm_head.weight; loader falls back to embed) |
| torch_dtype | bfloat16 |
| max_position_embeddings | 4096 (tritsim caps context at 2048) |

Tensor names: confirmed against model.safetensors (332 tensors, single file,
4.5 GB). LLaMA-style projection names; both sub-norms present per layer
(self_attn.attn_sub_norm.weight, mlp.ffn_sub_norm.weight); no lm_head.weight
(tied embeddings confirmed). Matches tritsim's loader and tritc's
TERNARY_SUFFIXES with no changes needed.

## Task 5/10 investigation outcomes (deviations from plan assumptions)

1. Conversion recon err is 0.724 mean, far above the plan's 0.15 gate. The
   gate was miscalibrated: QAT master weights are continuous (only 5.4% sit
   within 0.05 of a ternary point; gaussian ternarization baseline is 0.51).
   The forward pass defines the model, not master-weight proximity. Zero
   fraction 0.377 is sane. Decision: proceed; the logit comparison is the
   real gate.
2. The reference DOES quantize online, but not in modeling_bitnet: config
   carries quantization_config {quant_method: bitnet, linear_class:
   autobitlinear, quantization_mode: online}, and transformers' bitnet
   integration swaps projections for AutoBitLinear (WeightQuant = per-tensor
   absmean ternary; activation_quant = per-token absmax int8; sub-norm
   applied to the projection input). Recipe matches tritsim exactly, so
   dump_logits.py uses the native path unpatched and asserts AutoBitLinear
   modules are present (guards against silent unquantized fallback).
   Requires accelerate installed.

Memory note for tritsim on this model: unpacked trits in Vec<i8> ~2.4 GB,
f32 embeddings ~1.3 GB (currently cloned into lm_head when tied: another
1.3 GB). Fine for a golden model; optimize only if the host OOMs.
