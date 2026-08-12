# Measured numerics — fixed-point design inputs

Source: `TRITSIM_STATS` recorder over two prompts ("The capital of France is",
the 18-token variant) with 16 greedy decode steps each, on the pinned
BitNet b1.58 2B4T checkpoint. Values are the max over both runs; `count` is
recorded vectors (per layer per token).

| Stage | max abs | mean abs | max L2 | Hardware implication |
|---|---|---|---|---|
| residual | 137,694 | ~1,000 | 481,460 | Massive activations are real (BOS worst). Needs >= 18 integer bits plus fraction; bf16's 8-bit mantissa rounds at step ~1024 here, which is part of the reference-vs-f32 noise seen in Phase 0. v1 keeps f32; an integer residual needs Q18.13-class width (i32) and a per-tensor scale story. |
| mlp_act (pre ffn_sub_norm) | 1.55e10 | ~5.7e5 | 1.6e10 | The hard one: relu2 squares the gate, so this stage spans ~10 decades. This is why the architecture has ffn_sub_norm immediately after. Direct fixed-point needs ~34+ integer bits; flagged as its own design item for the FPGA phase (candidates: per-token pre-scaling of gate/up, or computing the absmax codes in a scaled domain). f32 in v1. |
| norm_out.ffn_sub | 565 | 0.41 | 769 | int16-friendly after quantization. |
| norm_out.attn_sub | 83 | 1.4 | 518 | int16-friendly. |
| ctx (attention out) | 62 | 0.82 | 301 | int16-friendly. |
| q / k / v | 10 / 14 / 62 | ~0.7-1.6 | 66-204 | Comfortable. |
| norm_out.input | 0.14 | 0.008 | 0.9 | Tiny (input_layernorm gains ~0.01) — but irrelevant to hardware after norm folding: absmax codes are scale-invariant, so the absolute magnitude here never enters the datapath. |
| norm_out.post_attn | 16 | 0.71 | 65 | Same note as above. |
| logits | 24 | ~4 | 3,528 | Host-side f32 in v1 (lm_head is not ternary). |

## Consequences adopted

1. **Norm folding** (see ARCHITECTURE, updated in this phase): every RMSNorm in
   this model feeds either an absmax int8 quantizer (BitLinear) or the host-side
   lm_head. absmax codes are invariant to the uniform 1/rms factor, so the
   per-element datapath needs no division or rsqrt anywhere — only the
   elementwise gain multiply and a per-token scalar (sum of squares -> one
   rsqrt) folded into the output scale.
2. **Quantized-domain stages (q/k/v/ctx/norm outputs) fit int16** with room;
   accumulators stay i32 as in the matvec core.
3. **Residual and mlp_act stay f32 in v1** (host/tritsim reference and the
   first hardware bring-up keep these off the critical ternary path).
   Full-integer residual and the relu2 dynamic-range problem are the two open
   fixed-point items, in that order of difficulty reversed: mlp_act is harder.
4. The raw stats JSONs are regenerable (`TRITSIM_STATS=out.json tritsim run ...`)
   and not committed.
5. **Collect stats in the default (unfolded) mode.** The folded path
   (`TRITSIM_FOLDED_NORM=1`) never materializes normed vectors — that is its
   point — so `norm_out.*` stages are absent from its reports by design. The
   tables above were measured unfolded.
