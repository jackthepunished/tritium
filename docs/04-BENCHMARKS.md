# Benchmarks — methodology and targets

Every public performance claim traces to a script in `tritbench/` and a raw-data CSV committed to the repo. If a number can't be reproduced from a fresh clone, it doesn't get posted.

## 1. Metrics

| Metric | Definition | Why it matters |
|---|---|---|
| Decode tok/s | Steady-state tokens/sec, batch 1, after prefill | The honest number; memory-bound regime |
| TTFT | Time to first token, fixed 256-token prompt | Prefill is compute-bound; expect this to look disproportionately good |
| J/token | Wall-plug (or 12V rail) energy per decoded token | **The headline metric.** tokens-per-joule is where ternary hardware wins |
| $/tok/s | BOM cost divided by decode tok/s | The "LLM on $50 hardware" claim, quantified |
| Quality spot-check | Small eval slice (e.g., a fixed 50-prompt QA set) vs bitnet.cpp same-checkpoint | Proves fixed-point approximations didn't lobotomize the model |

Rules: median of 3 runs, 60s warm-up excluded, thermals logged, context capped identically across devices, greedy decoding for comparability.

## 2. Baselines (same checkpoint everywhere it runs)

| Device | Runtime | Why included |
|---|---|---|
| Raspberry Pi 5 (8GB) | bitnet.cpp | The default "cheap edge" answer; closest price peer |
| Jetson Orin Nano | bitnet.cpp / llama.cpp CUDA | The default "edge AI" answer; the power-envelope peer |
| x86 laptop (whatever we have) | bitnet.cpp | Context for readers; not a competitor claim |
| tritium board | tritd + tritcore | Us |

Power measurement: inline USB-C power meter for Pi, barrel-jack INA226 for Jetson and tritium board, identical sampling script. Idle power reported separately from active.

## 3. Targets (set before measuring; misses get reported, not hidden)

- **Must:** end-to-end decode on hardware ≥ 80% of the bandwidth-roofline prediction from ARCHITECTURE §2 (proves the design isn't leaving the theoretical model on the table).
- **Must:** ≥ 3x better J/token than Pi 5 running bitnet.cpp on the same checkpoint.
- **Stretch:** ≥ 2x better J/token than Jetson Orin Nano.
- **Stretch:** conversational feel (≥ 8 tok/s decode) on a ≤ $250 total-BOM board with a ~700M-class model.

If the Must targets fail, the write-up says so and explains why — a credible negative result with a roofline analysis is still a strong build-in-public artifact and directs the board/packing decisions.

## 4. Profiler output (the dev-tools seed)

`tritd --profile` emits per-token JSON: per-layer cycles, DMA stall %, achieved GB/s, lane utilization, energy integral. This is the embryo of the standalone inference-profiler tool — same format targeted at CPU/GPU runtimes later. Design the schema once, here.

## 5. Reporting format

One canonical chart: **x = device, y = J/token (log scale), annotated with tok/s and BOM cost.** Same chart every post, growing as boards/models are added. Consistency makes the story legible over weeks of posting.
