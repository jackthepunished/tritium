# tritium

**Edge-native inference for ternary LLMs. Run a useful language model on cheap, low-power hardware — no cloud, no GPU, no multipliers.**

A *trit* is a ternary digit. Tritium is the hydrogen isotope that powers self-luminous devices for decades without an external energy source. Same idea: language models that glow on their own, offline, on hardware measured in tens of dollars and single-digit watts.

## The one-sentence thesis

BitNet-style 1.58-bit models (weights in {-1, 0, +1}) replace matrix multiplication with add/subtract/skip — an operation FPGAs and eventually ASICs do almost for free — yet everyone still runs them on CPUs and GPUs where that advantage is wasted. tritium is the hardware-native runtime that actually cashes the ternary check.

## What exists / what we build

| Layer | What it is | Language |
|---|---|---|
| `tritc` | Model converter: HF BitNet checkpoint → packed `.trit` format | Rust |
| `tritd` | Host runtime: tokenizer, sampler, KV cache, FPGA orchestration | Rust |
| `tritcore` | Ternary matmul engine + transformer dataflow on FPGA | SystemVerilog |
| `tritsim` | Bit-accurate software reference of tritcore, used as golden model | Rust |
| `tritbench` | Benchmark harness: tokens/s, J/token, TTFT vs Pi/Jetson baselines | Rust |

## Why this team

Direct continuation of [ternoise](https://github.com/jackthepunished/ternoise) — ternary compute on FPGA, built in public. The denoiser proved the primitive; tritium scales the same primitive (ternary MAC → add/sub tree) to transformer inference.

## Status

Pre-code. Docs-first. Read in order:

1. [docs/01-ARCHITECTURE.md](docs/01-ARCHITECTURE.md) — system design and the honest bandwidth math
2. [docs/02-ROADMAP.md](docs/02-ROADMAP.md) — 6-week milestone plan
3. [docs/03-RESEARCH.md](docs/03-RESEARCH.md) — ternary LLM background and reading list
4. [docs/04-BENCHMARKS.md](docs/04-BENCHMARKS.md) — measurement methodology and targets
5. [docs/05-POSITIONING.md](docs/05-POSITIONING.md) — market wedge, Founders Inc fit, build-in-public plan

## Ground rules

- Every performance claim ships with a reproducible benchmark script.
- `tritsim` is the source of truth; RTL is wrong until it matches bit-for-bit.
- Build in public. No emojis in project communications.
