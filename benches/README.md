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

For per-kernel, streaming ternary, and dense-head measurements:

```sh
cargo build --release -p trit-cpu --example kernelbench
for T in 1 2 4 8 16; do
    target/release/examples/kernelbench --threads "$T"
done
python3 benches/test_kernelbench.py target/release/examples/kernelbench
```

Each invocation measures one positive thread count (default 1) and reports the
actual pool width. The runtime pool keeps the width of its first initialization,
so sweeping counts inside one process would silently run later widths serially.
`--list-kernels` lists supported ternary kernels without allocating benchmark
tensors. Streaming "implied tok/s" is a bandwidth projection, not a model decode
measurement; use the full-model harness for the latter. ARM emulation can check
kernel correctness, but its timings must not be reported as ARM throughput.

## Baselines

`run.sh` looks for llama.cpp and bitnet.cpp and records a row either way:

| variable | meaning |
|---|---|
| `LLAMA_CPP_BIN` | a **`llama-bench`** binary; otherwise searched on `PATH` |
| `LLAMA_GGUF` | a `.gguf` of a comparable model |
| `BITNET_CPP_BIN` | bitnet.cpp's own **`llama-bench`** binary (it builds one) |
| `BITNET_GGUF` | the `i2_s` `.gguf` of the same checkpoint |
| `BASELINE_RUNS` | invocations per baseline; the median is recorded (default 3) |

Both binaries must be `llama-bench`, not `llama-cli` or `run_inference.py`: the
adapter passes llama-bench flags and parses its result table. Anything else
fails loudly rather than producing a number from a format nobody checked.

### What the baseline rows actually measure

`llama-bench -p 0 -n N` generates N tokens from an **empty context** and never
reads a prompt file, so baseline rows carry `llama-bench:tgN-empty-context` in
the `suite` column rather than the suite Tritium ran. Labelling them with the
suite would claim they saw input they never did.

Both sides still measure steady-state batch-1 decode, which is what makes the
comparison meaningful — but Tritium decodes with a short prompt in context and
llama-bench decodes with none. At these context lengths the KV-attention
difference is small; it is recorded rather than argued about.

A baseline is recorded only when **every** invocation yields a sample. A partial
set would publish a one-sample "median of three" as a successful measurement;
short sets are recorded unavailable with the count instead.

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
