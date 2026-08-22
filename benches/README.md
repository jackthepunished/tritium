# benches

Comparative measurement for the Tritium runtime.

The measurement *engine* lives in `crates/tritd/src/bench.rs`, because it needs
`Session` internals. This directory holds the fixed inputs, the orchestrator, the
baseline adapters, and the committed raw data.

## What is measured

`benches/run.sh --model models/bitnet-2b4t.trit` always produces:

- **decode tok/s** and **TTFT**, best of N runs after a warmup
- **bytes/token**, exact from the format rather than estimated
- **achieved GB/s**, and that as a percentage of a **memcpy probe measured on the
  same machine in the same run** — so "percent of roofline" is a measured ratio,
  not an assumption about what the memory system can do
- **peak RSS**, from `/proc/self/status`
- **J/token**, *only* where a real energy counter exists (Intel RAPL via
  `/sys/class/powercap`). Otherwise the column is empty and the source reads
  `none`. It is never estimated: an invented energy number would poison the one
  claim this project exists to make.

Kernel microbenchmarks are separate:

```sh
cargo bench -p tritbench
```

Those tensors are L3-resident, so they measure compute throughput rather than the
streaming regime a real decode runs in. Use `tritd bench` for the end-to-end
number.

## Baselines

`run.sh` looks for llama.cpp and bitnet.cpp and records a row either way:

| variable | meaning |
|---|---|
| `LLAMA_CPP_BIN` | `llama-bench` or `llama-cli`; otherwise searched on `PATH` |
| `LLAMA_GGUF` | a `.gguf` of a comparable model |
| `BITNET_CPP_BIN` | bitnet.cpp's inference binary |

When a baseline is absent, the CSV gets a row with `status=unavailable` and a
note saying exactly what was looked for. `report.py` renders those as
"_not measured_" rather than dropping them, because a table that silently omits a
comparison reads as though the comparison was run and went well.

## Comparability

This is the part that is easy to get wrong.

llama.cpp `Q2_K` is roughly 2.6 bits/weight and `Q4_K_M` roughly 4.5, both
**post-training** quantizations of a model that was not trained for them. BitNet
ternary is 2.0 bits/weight and **quantization-aware trained** — a different model,
not the same model compressed differently.

So a tokens-per-second comparison between them is a comparison of two different
quality points, and is only meaningful alongside the `bits_per_weight` column and
a quality measurement. `tritsim compare` against a HuggingFace logit dump is the
quality check; it belongs next to any throughput claim.

## Results

`benches/results/*.csv` holds raw data, one file per host and date, committed.
Render one with:

```sh
python3 benches/report.py benches/results/<host>-<date>.csv
```
