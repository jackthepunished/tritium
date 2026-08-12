# Roadmap — 6 weeks to a demo that cannot be faked

Each week ends with a public artifact (repo tag + short post). The cadence is the ternoise playbook: show the thing running, show the numbers, show the failure modes.

## Week 1 — tritsim: correct before fast

- [x] Rust workspace scaffold (`tritc`, `tritsim` crates; `tritd`, `tritbench` deferred to their phases).
- [x] Load `microsoft/bitnet-b1.58-2B-4T` (bf16 master weights) via safetensors; quantize with the b1.58 recipe, report zero-fraction and reconstruction stats.
- [x] Implement decode loop in tritsim: embedding, RMSNorm, ternary linear (select-accumulate, int8 activations, absmax quant), RoPE, GQA attention, relu2 gated MLP with sub-norms, greedy sampling.
- [x] Match reference logits (HF transformers with native online BitNet quantization) on fixed prompts: mean cosine 0.999, top-1 100% (near-tie rule at BOS documented in compare output).
- **Artifact:** tritsim generating coherent text on CPU. Post: "ternary LLM inference in pure Rust, no multiplies in the hot loop."

## Week 2 — .trit format + RTL matmul core in simulation

- [x] Freeze `model.trit` v0: packing, tiling order, manifest. `tritc` emits it; tritsim consumes it (same bytes the FPGA will see). (Landed with Phase 0.)
- [x] SystemVerilog: trit unpack + lane array (N=64) + adder tree + accumulator (`rtl/trit_matvec.sv`). Fixed-point RMSNorm/activation units deferred to week 3: bit-exact verification requires migrating tritsim's f32 norm math to matching fixed-point first (Q-format design), which belongs with board bring-up.
- [x] Verilator testbench driven by tritsim-generated vectors (`tritsim vectors`, `make -C rtl test`); bit-exact on random, padding, extremes, zeros, 6912-wide, and real layer-0 k_proj tiles, plus an invalid-encoding error check.
- **Artifact:** waveform + "RTL matches golden model bit-for-bit" post with the testbench harness.

## Week 3 — numerics, norm folding, synthesis numbers (reframed)

Original premise ("port to the ternoise dev board") corrected: no board exists —
ternoise is deliberately simulation-first, board chosen from synthesis numbers.
tritium follows the same doctrine.

- [x] Measure per-stage numeric ranges on the real checkpoint (`TRITSIM_STATS`); write docs/03b-NUMERICS.md. Massive activations confirmed (residual max ~138k; relu2 stage ~1.5e10).
- [x] Norm folding: eliminate the per-element rsqrt/divide from the datapath (absmax codes are scale-invariant); proved on the real model — identical compare metrics and greedy outputs, property-tested across 1000 adversarial vectors.
- [x] Yosys generic synthesis of trit_matvec with a zero-multiplier assertion (`make -C rtl synth`): ~29.5k datapath cells at 64 lanes, no $mul/$macc.
- [x] Board-selection memo (docs/04b-BOARD-MEMO.md) with roofline x synthesis numbers; purchase decision to Bahadir. **Decision gate (moved from silicon to paper, where it belongs pre-purchase).**
- **Artifact:** the numerics report + the "no multipliers, no dividers" synthesis check.

## Week 3b — first silicon contact (after board purchase)

- [ ] Layer sequencer + weight streaming under Verilator end-to-end first (ternoise M5 pattern), then port to the chosen board.
- [ ] Single transformer layer end-to-end on hardware, output matches tritsim.
- [ ] Measure: achieved bandwidth, lane utilization, clock against the roofline.
- **Artifact:** scope/ILA screenshot + measured GB/s vs the ARCHITECTURE table.

## Week 4 — full model on hardware

- [ ] Layer sequencer: run all layers per token, host-driven descriptor chain, double-buffered DMA.
- [ ] tritd: tokenizer, sampler, chat loop talking to the board.
- [ ] KV cache v1 (BRAM hot window, 2K cap). Embeddings/lm_head on host if fabric-constrained.
- [ ] Target model: whatever fits the bandwidth budget conversationally — 160M–700M class first, 2B stretch.
- **Artifact:** video — prompt in, tokens out, network cable visibly unplugged.

## Week 5 — numbers nobody can argue with

- [ ] `tritbench`: fixed prompt suite, 3 runs, medians; tok/s decode, TTFT, J/token via INA226 on the 12V rail.
- [ ] Baselines on identical prompts: bitnet.cpp on Raspberry Pi 5, Jetson Orin Nano (and one x86 laptop for reference).
- [ ] Per-layer profiler output (cycle counts, DMA stalls) — first public outing of the profiler angle.
- **Artifact:** benchmark report (BENCHMARKS.md filled with real data) + reproduction scripts. This is the credibility post.

## Week 6 — the demo and the ask

- [ ] Package the board as a self-contained "answer box": battery pack, e-ink or serial display, wake-on-button. Offline by construction.
- [ ] 2-minute demo video: what it is, why ternary, the J/token chart, the roadmap to ASIC.
- [ ] Write the one-pager (from 05-POSITIONING). Apply to Founders Inc; the application is the six weeks of public artifacts.
- **Artifact:** demo video + application sent.

## Explicit risks

| Risk | Mitigation |
|---|---|
| DDR controller/DMA integration eats week 3 | It always does. Week 3 scope is ONE layer; sequencing is week 4. |
| 2B model too slow on ternoise board | Ship 700M-class conversational demo; show 2B math for the better board. Honesty beats vaporware. |
| No small ternary checkpoint with decent quality | Fall back to microsoft 2B4T at low tok/s for quality demo + small model for speed demo; two-board story. |
| RTL debugging spiral | tritsim vectors at every block boundary; never debug quality and correctness at the same time. |
| Board dies / tooling hell | tritsim IS a demo ("no-multiply LLM runtime in Rust") and the profiler is a standalone deliverable. |
