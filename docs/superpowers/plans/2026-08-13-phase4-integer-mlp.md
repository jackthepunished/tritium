# tritium Phase 4 (integer-exact MLP activation path) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Eliminate the numerics report's hardest open item — the relu2 stage's ~1.5e10 f32 dynamic range — by computing the down-projection's int8 codes exactly in i64 integer arithmetic from the gate/up matvec accumulators, with no floating point in the element path.

**The math (each step checkable):** For a token, gate and up BitLinear outputs are `g_i = acc_g_i * S_g` and `u_i = acc_u_i * S_u`, where `S_g = w_scale_g * x_scale > 0` and `S_u = w_scale_u * x_scale > 0` are uniform over i. Then

```text
a_i = relu(g_i)^2 * u_i = relu(acc_g_i)^2 * acc_u_i * (S_g^2 * S_u) = t_i * K
```

with integer `t_i = relu(acc_g_i)^2 * acc_u_i` and uniform positive `K`. The ffn_sub_norm + down-BitLinear pair consumes absmax codes of `a .* gain`, and codes are invariant to the uniform K (norm-folding, Phase 2) — so codes come from `t .* gain` directly. Bound: `|acc| <= 2560 * 127 = 325,120 < 2^18.4`, so `|t_i| < 2^55` — exact in i64. The remaining scalars: down's activation scale is `max|t*gain|/127 * K * r`, and the folded sub-norm scalar is `r = 1/sqrt(mean(a^2) + eps)` with `mean(a^2) = K^2 * mean(t^2)` — both computed once per token in f64 from the exact `t_i`.

What this buys the hardware: the MLP element path between the three matvecs is two integer multiplies (square, then times acc_u — inherent to squared-ReLU, streamable through 2 DSPs) plus a per-channel gain multiply for the code search; no wide floats anywhere. The ternary weight datapath remains multiplier-free; this is activation math, already scoped outside that claim.

**Honest caveat to verify, not assume:** codes from exact-i64 `t .* gain` can differ from the f32 path's codes at rounding boundaries (the f32 path rounds `a_i` before quantizing; ours doesn't round). The i64 path is the *more* exact one, but the gate must show the model doesn't care: identical compare metrics and identical greedy text on the real checkpoint vs the Phase-2 folded path. Also `t_i` up to 2^55 exceeds f64's 2^53 integer-exact range in theory; measured accumulators are far smaller (mlp_act max 1.5e10 in *scaled* units), and the code search uses i128 for the comparison-critical `|t_i| * gain_as_fixed` only if a test shows f64 conversion flips a code — decide from the property test, record the outcome.

**Tech Stack:** Rust only (design phase; RTL for this block comes with board bring-up).

## Global Constraints

- Repo root `/mnt/d/dev/tritium`; `CARGO_TARGET_DIR=$HOME/.cache/tritium-target`. No emojis. Task = green + commit.
- Gate standard: `TRITSIM_INT_MLP=1` (implies folded) vs Phase-2 folded path on the real checkpoint — identical compare stdout metrics and identical greedy text on both test prompts.
- Property tests must cover adversarial accumulators at the theoretical bound (acc = +/-325,120), not just random values.

## File Structure

```
crates/tritsim/src/math.rs    # int_mlp_codes(acc_g, acc_u, gain, k, eps) -> (codes, x_scale_for_down)
crates/tritsim/src/model.rs   # BitLinear::acc() exposing raw i32 accumulators; TRITSIM_INT_MLP path
docs/01-ARCHITECTURE.md       # integer MLP path paragraph
docs/03b-NUMERICS.md          # mark the relu2 item solved, with the i64 bound argument
```

---

### Task 1: `int_mlp_codes` with property tests

**Interfaces:** `math::int_mlp_codes(acc_g: &[i32], acc_u: &[i32], gain: &[f32], k: f64, eps: f32) -> (Vec<i8>, f32)` where `k = (S_g as f64)^2 * S_u as f64`; returns the int8 codes for the down projection and the single f32 activation scale to hand `apply_prequantized` (i.e. `max|t*gain|/127 * K * r` with `r` per the formula above).

- [ ] Failing property test: for 500 random cases (n=64) plus bound cases (accs at +/-325,120, gains up to 2), codes equal the reference computed the f32-folded way — `a_i = (relu(acc_g*sg))^2 * acc_u*su`, `scaled_absmax_codes(a, gain, eps)` — allowing ZERO code differences on random cases; on deliberately boundary-straddling cases (construct `t*gain` ratios landing within 1e-7 of .5 code boundaries) record and assert a documented tolerance (expect 0 or single-ulp flips; the test prints the count).
- [ ] Implement (t in i64; max-search and r in f64; if the boundary test shows f64-conversion flips beyond single-ulp cases, switch the comparison to i128 fixed-point and re-run — record which branch was taken in the commit message).
- [ ] Commit.

### Task 2: Model integration behind `TRITSIM_INT_MLP`

- `BitLinear::acc(&self, xq: &[i8]) -> Vec<i32>` (raw backend matvec, no scaling).
- In the folded MLP path: when `TRITSIM_INT_MLP` is set (requires folded; assert), compute `acc_g`, `acc_u` via `.acc(codes)`, `k` from the three scales, call `int_mlp_codes`, feed `down.apply_prequantized`. The f32 `a` vector is never built.
- [ ] Tiny-model test: forward with folded vs folded+int-mlp — logits agree within 1e-4 relative (tiny random model, no claim of exactness — rounding differs by design); determinism holds.
- [ ] Commit.

### Task 3: Real-model gate

- [ ] `compare` A/B: folded vs folded+int-mlp — identical stdout metrics. Greedy A/B both prompts, 12 steps — identical text. (Background; ~3 min/run CPU backend.)
- [ ] Commit gate results into roadmap week-4 notes.

### Task 4: Docs + PR

- [ ] ARCHITECTURE: integer MLP path paragraph (the math chain above, the 2-DSP hardware note). NUMERICS: mark item solved with the bound argument and the gate evidence. README status line.
- [ ] Full verification: workspace tests, `--features rtl` tests, `make -C rtl lint test synth`.
- [ ] Branch push, PR, finishing-a-development-branch.

## Self-Review Notes

- The scale-invariance reuse is the same theorem as Phase 2, applied to a uniform *product* of scales; the property test checks the composed claim, not just the parts.
- The f64-exactness caveat (2^55 > 2^53) is explicitly tested at the theoretical bound rather than waved at measured ranges.
- No RTL in this phase by design: the block's hardware needs the board-phase DSP/timing context; the design is locked and proven against the model first.
