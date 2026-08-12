# Roadmap — 6 weeks to a demo that cannot be faked

Each week ends with a public artifact (repo tag + short post). The cadence is the ternoise playbook: show the thing running, show the numbers, show the failure modes.

## Week 1 — tritsim: correct before fast

- [ ] Rust workspace scaffold (`tritc`, `tritsim`, `tritd`, `tritbench` crates).
- [ ] Load `microsoft/bitnet-b1.58-2B-4T` (or smaller ternary checkpoint) via safetensors; verify ternary-ness of weights, extract scales.
- [ ] Implement decode loop in tritsim: embedding, RMSNorm, ternary linear (select-accumulate, int8 activations, absmax quant), RoPE, attention, SiLU MLP, sampling.
- [ ] Match reference logits from `bitnet.cpp` (or HF transformers) within quantization tolerance on a fixed prompt set.
- **Artifact:** tritsim generating coherent text on CPU. Post: "ternary LLM inference in pure Rust, no multiplies in the hot loop."

## Week 2 — .trit format + RTL matmul core in simulation

- [ ] Freeze `model.trit` v0: packing, tiling order, manifest. `tritc` emits it; tritsim consumes it (same bytes the FPGA will see).
- [ ] SystemVerilog: trit unpack + lane array (start N=64) + adder tree + accumulator; fixed-point RMSNorm and SiLU units.
- [ ] Verilator testbench driven by tritsim-generated vectors; bit-exact match required per tile, per layer.
- **Artifact:** waveform + "RTL matches golden model bit-for-bit" post with the testbench harness.

## Week 3 — first silicon contact

- [ ] Port core to the ternoise dev board; weight streaming from onboard DDR via DMA.
- [ ] Single transformer layer end-to-end on hardware, output matches tritsim.
- [ ] Measure: achieved bandwidth, lane utilization, clock. This decides N, packing format, and whether P2 stays on this board or moves to a KV260-class part. **Decision gate.**
- **Artifact:** scope/ILA screenshot + measured GB/s. Post the utilization math against the ARCHITECTURE table.

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
