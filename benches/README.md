# benches

Comparative measurement for the Tritium runtime.

The measurement *engine* lives in `crates/tritd/src/bench.rs`, because it needs
`Session` internals. This directory holds the fixed inputs, the orchestrator, the
baseline adapters, and the committed raw data.

## What is measured

`benches/run.sh --model models/bitnet-2b4t.trit` always produces:

- **decode tok/s**, median of prompt/run rates after a warmup; **TTFT**, the
  minimum observed time to first token (not a median). Prefill throughput is
  currently a zero placeholder and must not be interpreted as a measurement.
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

For attributing decode time to phases, or for diagnosing dispatch cost:

```sh
cargo run --release -p tritd --example decode_profile -- --threads 4
cargo run --release -p tritd --example decode_profile -- --threads 8 --pool-probe
cargo run --release -p tritd --example decode_profile -- --threads 8 --bw-probe
```

`decode_profile` wraps the injected `MatvecBackend`, so it attributes a real
decode to ternary projections, the dense head and the serial remainder without
changing the runtime. It asserts 210 ternary and one dense call per token, so a
wrong attribution window fails rather than printing a plausible split.
`--pool-probe` times empty dispatches against a sweep of inter-job gaps;
`--bw-probe` compares per-matvec dispatch against a fork-once static
partitioning of identical work, which is how the memory system's actual ceiling
was separated from dispatch cost. One thread count per process, for the reason
above.

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

ARM kernel changes can use `.github/workflows/arm-kernel-comparison.yml` for a
paired native comparison against the PR base. It builds both revisions before
timing, alternates AB/BA order for four pairs at 1/2/4 threads, and uploads
raw outputs plus CPU/compiler and binary identities. The parser's regressions
run with `python3 benches/test_compare_arm.py`. On a native aarch64 host, use
`python3 benches/compare_arm.py BASELINE_BINARY CANDIDATE_BINARY --out RESULTS`.
Check cache capacity before interpreting the 113 MB pass as DRAM traffic: the
Neoverse-N2 CI runner reports 128 MiB L3. These results measure kernels, not
full-model decode or energy.

### Native ARM full-model comparison

`.github/workflows/arm-model-comparison.yml` runs the real BitNet b1.58 2B4T
checkpoint on `ubuntu-24.04-arm`. It downloads pinned public bf16 and I2_S
checkpoints, verifies their SHA256 hashes, converts the bf16 weights to `.trit`,
and builds every binary before measurement. It is manually dispatched because
it downloads several gigabytes and runs thousands of decode steps; the small
Python harness regressions run in ordinary PR CI. Once the workflow is on the
default branch, start it with `gh workflow run arm-model-comparison.yml --ref BRANCH`.
The baseline is explicitly pinned
to `902fc8a` (before the NEON mask shuffle); it is not the current PR base.
The candidate is the workflow checkout. bitnet.cpp and its submodules are pinned.
The workflow applies `patches/bitnet-arm-src1-cont.patch`: upstream declares
`src1_cont` inside `GGML_USE_LLAMAFILE` but uses it unconditionally, preventing
the default ARM build. This moves that declaration outside the guard without
changing the expression or math. The comparison is labelled **bitnet.cpp +
build fix**, and the applied diff is retained in the results.

**The pinned ARM I2_S baseline is diagnostic only.** Its non-AVX2 dot fallback
already produces signed ternary sums, but its caller subtracts the activation
sum again. A uniform +1-weight/+1-activation 64-element dot produces 0 instead
of 64. `audit_bitnet_i2s.py` reproduces this source-branch failure; the workflow
retains its JSON without blocking Tritium's independent correctness/measurement
gates. The workflow passes `--bitnet-audit results/bitnet-i2s-validation.json` to
the harness, which skips a failed comparator and records an explicit excluded
status and null rate. A passing spot check would not establish full-model quality either.
Do not use this baseline's rate for a competitive speed claim. An independently
validated I2_S revision or ARM TL1 baseline remains future comparison work.

The independent oracle must execute all 24 positions across Reference, Folded
and IntMlp, with cosine >0.999999 and matching top-1. A missing model fails the
workflow instead of allowing the ignored test's skip path. Before timing,
four prompts' greedy continuations must match before/after at 1/2/4 threads.

After building and acquiring the models, reproduce the timing stage with:

```sh
python3 benches/test_compare_model.py
python3 benches/audit_bitnet_i2s.py \
  --llama-source /path/to/bitnet/3rdparty/llama.cpp --out results
python3 benches/compare_model.py \
  --baseline /path/to/baseline/target/release/tritd \
  --candidate /path/to/candidate/target/release/tritd \
  --bitnet /path/to/bitnet/build/bin/llama-bench \
  --model models/bitnet-2b4t.trit \
  --gguf /path/to/ggml-model-i2_s.gguf \
  --tokenizer models/bitnet-2b4t/tokenizer.json \
  --suite benches/prompts/short.jsonl \
  --profile target/release/examples/decode_profile \
  --bitnet-audit results/bitnet-i2s-validation.json \
  --out results
```

The harness runs four rounds at each width, with balanced runtime order and
fresh processes. Every model is read into page cache before each invocation.
Omitting BitNet after a failed audit preserves balanced before/after Tritium
order. The bootstrap measurements retained all three runtimes' raw timings;
their subsequently discovered BitNet numerical failure is labelled in the report.
Tritium measures four short prompts, greedy, up to 32 tokens each, one warmup
and one measured pass; its invocation rate is the median of four prompt rates.
bitnet.cpp measures empty-context tg32, three repeats, reporting their mean.
Its `test_gen` feeds synthetic random token IDs; it does not sample and
detokenize generated text as Tritium does. This is an additional workload
difference, not a cross-runtime output-quality check.
Tables take the median of four invocation rates. Only the two Tritium versions
have identical workloads; their speedup is the median of paired ratios.

Raw JSON, commands, stdout/stderr, observed token counts, source/toolchain/CPU
identities and binary/model hashes accompany the summary. Phase profiles run
separately and include instrumentation overhead. Ternary/dense times include
their worker synchronization; empty-dispatch probes are diagnostic and cannot
be subtracted as an exact synchronization share. The remainder includes
attention, other transformer operations, sampling and profiler bookkeeping.
Hosted native ARM results do not establish edge-device performance or energy.
