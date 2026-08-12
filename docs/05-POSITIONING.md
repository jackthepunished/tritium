# Positioning — wedge, audience, and the Founders Inc angle

## 1. One-liner

**tritium runs real language models on $50, single-digit-watt hardware — fully offline — by building the first complete hardware-native stack for ternary (1.58-bit) LLMs.**

## 2. Why now

- Native ternary models became *real* in 2024–2025 (BitNet b1.58, the 2B4T open checkpoint) — quality is no longer the blocker.
- Everyone runs them on multiply-hardware (CPU SIMD tricks, GPUs), stranding a ~30x energy advantage on the dominant operation.
- Demand for offline/private/low-power inference is exploding in exactly the categories that can't use a 15W Jetson or a cloud call: toys, wearables, robots, medical devices, defense edge.
- Nobody owns "ternary-native silicon." The window between "models exist" and "incumbent ships a chip" is the opportunity.

## 3. Wedge and ladder

1. **Now (weeks):** open-source FPGA stack + benchmark credibility. The asset is proof and audience, not revenue.
2. **Next (months):** dev-board/module for hardware startups that need offline language capability (the Magical-Toys-shaped customer: needs a voice brain, can't ship cloud latency/cost/privacy). Sell modules + the runtime.
3. **Later (18mo+):** ternary inference ASIC — the select-accumulate datapath at 10–100x FPGA efficiency. The FPGA stack becomes the reference design and the customer funnel.

Fallback wedges if hardware demand is slow: `tritd`'s profiler as a standalone inference-observability tool (software revenue, same codebase); licensing the RTL core.

## 4. Why us

- ternoise: already building ternary compute on FPGA in public — the exact primitive, demonstrated, with an audience watching.
- Rust/C++ systems depth + RTL: the full stack (converter, runtime, RTL, benchmarks) fits in one head; no integration seams between teams that don't exist yet.
- Bench-honest culture: every claim reproducible, misses published. In a space full of "runs LLaMA on a potato" hype, rigor is the brand.

## 5. Founders Inc fit (why apply there specifically)

- Portfolio is ~45% applied AI and ~25% hardware/robotics; they explicitly fund "hard problems in emerging domains" with $100–250K first checks and are comfortable being the only institutional investor.
- Their selection mechanism is "come build on campus for 1–2 weeks" — tritium's demo is a physical box on a table generating text with the network cable unplugged and a power meter reading 4W. That demo is unfakeable and plays perfectly to that filter.
- Portfolio synergy is concrete: their consumer-hardware and robotics companies are the first customers for an offline language module.
- The application IS the build log: six weeks of public artifacts (working RTL, benchmark charts, demo video) instead of a deck.

## 6. Build-in-public plan

- Continue on the ternoise account/identity — same audience, same no-emoji voice; tritium is the obvious act two.
- Cadence: one substantial post per week, keyed to the roadmap artifacts (tritsim text generation, bit-exact RTL, first silicon contact, full-model video, benchmark chart, demo box).
- Every post: one chart or one video, the repo link, and one honest problem encountered. The failure notes are what make the wins believable.
- End state of the six weeks: demo video + benchmark report + one-pager, submitted to f.inc, posted publicly the same day.

## 7. Honest risks (say them before investors do)

- **Model risk:** ternary LLM progress could stall, or frontier labs could stop releasing open ternary checkpoints. Mitigation: architecture also serves int4-ish sparse formats; the runtime/profiler layer is format-agnostic.
- **Incumbent risk:** Qualcomm/Apple/NVIDIA could add low-bit datapaths. Mitigation: they optimize for their platforms and margins; the $5 BOM module segment is beneath them for years.
- **Solo-founder execution risk:** the 6-week plan is scoped to one person deliberately; the f.inc raise funds the second engineer and the board respin.
