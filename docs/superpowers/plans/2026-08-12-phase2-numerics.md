# tritium Phase 2 (numerics, norm folding, synthesis numbers) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace week 3's "port to the ternoise board" premise (no board exists — ternoise is deliberately simulation-first) with the work that actually gates a board purchase: measured per-stage numeric ranges from the real model, elimination of the RMSNorm division from the hardware datapath (proved bit-exact), Yosys synthesis numbers for `trit_matvec`, and a board-selection memo.

**Architecture:** Three legs. (1) `tritsim` gains an env-gated range recorder that logs per-stage max-abs/L2 statistics over real prompts into a report, feeding `docs/03b-NUMERICS.md`. (2) The norm-folding theorem — `absmax_quantize(rmsnorm(x, g))` produces identical int8 codes to `absmax_quantize(x .* g)` with the rms folded into the output scale — is implemented as an alternative BitLinear path and proved bit-identical on the real checkpoint logits. (3) Yosys synthesizes `trit_matvec` generically, asserts zero multiplier cells (ternoise-style), and reports LUT/FF counts; those numbers plus the bandwidth roofline go into a board memo.

**Tech Stack:** Rust, Yosys 0.33+ (installed), existing Verilator regression.

## Global Constraints

- Repo root `/mnt/d/dev/tritium`; `CARGO_TARGET_DIR=$HOME/.cache/tritium-target`. No emojis. Every task ends green + committed.
- The folding proof standard is bit-identical f32 logits on the real checkpoint (not cosine — exact equality), because the claim is algebraic identity, not approximation.
- Synthesis is generic (`synth` without a device target) — the number that matters now is "zero `$mul` cells" plus an order-of-magnitude LUT count; device-accurate numbers come with the board toolchain.
- The board decision itself is the user's; the memo presents options with measured numbers, it does not purchase anything.

---

### Task 1: Range recorder + numerics report

**Files:** Create `crates/tritsim/src/stats.rs`; modify `lib.rs`, `model.rs` (hook points), `main.rs` (`run --stats <path>` flag).

**Interfaces:** `stats::Recorder` (thread-local or passed handle is overkill — use a `OnceLock<Mutex<Recorder>>` enabled by `TRITSIM_STATS=<out.json>`), `stats::record(stage: &str, v: &[f32])` no-op when disabled; on process exit (or explicit `stats::flush()` from main) writes JSON `{stage: {count, max_abs, mean_abs, max_l2}}`. Stages hooked in `model.rs`: `residual`, `norm_out.input`, `norm_out.post_attn`, `norm_out.attn_sub`, `norm_out.ffn_sub`, `q`, `k`, `v`, `ctx`, `mlp_act`, `logits`.

- [ ] Test (in `stats.rs`): enable via direct `Recorder` API, record two slices, assert max_abs/mean_abs/count correct; disabled path records nothing.
- [ ] Implement recorder + hooks (hooks call `stats::record` unconditionally; the function early-returns when disabled so the hot path stays cheap).
- [ ] Run real model: `TRITSIM_STATS=numerics.json tritsim run` over the two Phase-0 prompts and 32 generated tokens each; also `compare` runs. Collect JSON.
- [ ] Write `docs/03b-NUMERICS.md`: table of measured ranges per stage, implications (residual dynamic range incl. the BOS massive-activation outliers vs i32/fixed-point feasibility; which stages are safe in int16/int32; where f32 must survive in v1).
- [ ] Commit: `feat(tritsim): per-stage range recorder + measured numerics report`.

### Task 2: Norm folding — remove the division from the datapath

**Files:** Modify `crates/tritsim/src/math.rs` (add `scaled_absmax_codes`), `model.rs` (folded path behind `TRITSIM_FOLDED_NORM=1`), tests in `tiny_model.rs`.

**The identity:** for `y = rmsnorm(x, g, eps)` and `(yq, ys) = absmax_quantize(y)`: since `y_i = (x_i * g_i) * r` with the scalar `r = 1/sqrt(mean(x^2)+eps)` uniform over i, the int8 codes of `y` equal the int8 codes of `z_i = x_i * g_i` (round(v/ (max|v|/127)) is invariant to uniform positive scaling), and `ys = zs * r`. So BitLinear-after-norm = quantize `x .* g` (no division), multiply the *output* by the folded scalar `r` once. Hardware consequence: no rsqrt/divide per element; one scalar per token per norm.

**Caveat to verify in the test, not assume:** f32 round-trip effects — `(x_i*g_i)*r` computed in f32 then divided by scale can round differently than `x_i*g_i` divided by (scale/r) in rare boundary cases. The real-model gate below settles whether this matters in practice; the property test uses exact-comparison on codes and documents any tolerance needed.

- [ ] Property test (tiny model + random vectors): codes from `absmax_quantize(rmsnorm(x,g,eps))` vs `scaled_absmax_codes(x,g)` identical across 1000 random vectors including huge-outlier vectors (1e4 spikes); output scales agree to 1 ulp-ish (`relative < 1e-6`).
- [ ] Implement `scaled_absmax_codes(x: &[f32], g: &[f32]) -> (Vec<i8>, f32)` (quantize x.*g, return codes + max-based scale) and the folded forward path in `model.rs`: each norm+BitLinear pair becomes elementwise-gain -> quantize -> matvec -> apply `w_scale * z_scale * r` where `r` is computed once per token from `sum(x^2)` (f64 accumulate, then f32).
- [ ] Real-model gate: `TRITSIM_FOLDED_NORM=1 tritsim compare` on `logits.json` — require metrics identical to the unfolded run (same mean cosine to 4 decimals, 100% top1) AND `tritsim run` greedy output string identical for both prompts. If exact logit equality holds, record that; if only near-equality (f32 rounding), record the max deviation in NUMERICS.md and justify.
- [ ] Update `docs/01-ARCHITECTURE.md` (nonlinear-units paragraph): rsqrt no longer needed per element; note the one-DSP-per-token scalar path (sum of squares) and that RoPE keeps its multipliers (outside the ternary datapath claim).
- [ ] Commit: `feat(tritsim): fold rmsnorm scalar out of the quantized datapath, proved on real model`.

### Task 3: Yosys synthesis numbers + no-multiplier assertion

**Files:** Create `rtl/synth.ys`, extend `rtl/Makefile` (`synth` target), `.github`-style check deferred (no CI in repo yet).

```text
# rtl/synth.ys
read_verilog -sv trit_matvec.sv
hierarchy -top trit_matvec
synth -flatten
select -assert-count 0 t:$mul
select -assert-count 0 t:$macc*
stat
```

- [ ] `make -C rtl synth` runs `yosys -s synth.ys`, fails if any multiplier cell exists, prints `stat`.
- [ ] Record LUT-equivalent cell counts and memory bits in the board memo (Task 4). Note: `x_mem` (8192x8 = 64Kb) should map to memory; if `stat` shows it flattened to FFs, add the `ram_style` attribute or accept and note it — decision recorded either way.
- [ ] Commit: `feat(rtl): yosys synthesis target with zero-multiplier assertion`.

### Task 4: Board-selection memo + docs + PR

**Files:** Create `docs/04b-BOARD-MEMO.md`; update `docs/02-ROADMAP.md` week 3 (reframed: sim-first, board from numbers — mirroring ternoise M5), README status.

- [ ] Memo contents: the ARCHITECTURE bandwidth table restated with the measured model sizes; `trit_matvec` synthesis numbers scaled to plausible lane counts; three candidate boards (budget Artix-7/DDR3 class, Kria KV260 class, and "stay in Verilator until tritd exists" as the null option) with what each enables per the roofline; explicit recommendation; the purchase decision left to Bahadir.
- [ ] Roadmap week 3 rewritten to sim-first reality; README status line updated.
- [ ] Full verification: workspace tests, `make -C rtl lint test synth` all green.
- [ ] Branch `phase2-numerics`, push, PR, finishing-a-development-branch.

## Self-Review Notes

- Week-3 premise correction is documented in the plan goal and roadmap edit, not silently changed.
- The folding claim has both a property test (synthetic, adversarial outliers) and a real-model gate (exact logits) — algebra is checked where it can fail (f32 rounding), not assumed.
- Synthesis numbers are explicitly generic; the memo says so, avoiding false precision before a device target exists.
