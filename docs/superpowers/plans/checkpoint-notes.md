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

Tensor names: to be confirmed against safetensors once download completes
(expected LLaMA-style plus optional attn_sub_norm / ffn_sub_norm).

Memory note for tritsim on this model: unpacked trits in Vec<i8> ~2.4 GB,
f32 embeddings ~1.3 GB (currently cloned into lm_head when tied: another
1.3 GB). Fine for a golden model; optimize only if the host OOMs.
