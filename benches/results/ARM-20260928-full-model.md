# BitNet 2B4T: native ARM full-model validation

28 September 2026. Native GitHub `ubuntu-24.04-arm`, Neoverse-N2, four
exposed cores, 128 MiB reported L3. Rust 1.98.1 / LLVM 22.1.8. These are hosted,
virtualized native ARM measurements, not QEMU timings or edge-device results.
No energy was measured.

The NEON mask shuffle improves actual Tritium decode by approximately **28–30%**
in two independent paired runs. The attempted bitnet.cpp I2_S comparator is excluded
from competitive claims: its selected non-AVX2 source path fails an arithmetic
check. Low timing variance does not make that baseline numerically valid.

## Paired decode

Baseline: `902fc8a85fc297662d19ac47dd0224051c3281de`, before the mask shuffle
and after the worker spin-deadline fix. Candidate inference code is merged
PR #12. This PR adds measurement infrastructure; it does not change Tritium
inference arithmetic. Both versions use the identical model file and prompts.

Initial run, source head `e96dba9`,
[workflow](https://github.com/jackthepunished/tritium/actions/runs/36467548042):

| Threads | Before tok/s | After tok/s | Median paired speedup | Paired range |
|---|---:|---:|---:|---:|
| 1 | 3.183 | 4.108 | 1.291x | 1.290–1.294x |
| 2 | 6.030 | 7.766 | 1.288x | 1.284–1.290x |
| 4 | 10.855 | 13.945 | 1.285x | 1.277–1.286x |

Confirmation run, source head `b019f7c`,
[workflow](https://github.com/jackthepunished/tritium/actions/runs/36469787689):

| Threads | Before tok/s | After tok/s | Median paired speedup | Paired range |
|---|---:|---:|---:|---:|
| 1 | 3.203 | 4.152 | 1.296x | 1.293–1.297x |
| 2 | 6.054 | 7.827 | 1.292x | 1.292–1.294x |
| 4 | 10.910 | 14.040 | 1.285x | 1.284–1.292x |

These are separate hosted VMs and rebuilds of the same inference source. Model,
GGUF, tokenizer and suite hashes match between runs. Absolute rates are kept
separate; no cross-host pairing or pooled absolute median is used.

Four rounds per width, separate processes, alternating before/after order with
bitnet diagnostic invocations interleaved. Every model is pre-read into page
cache before each invocation. All builds and downloads finish before timing;
no other build runs inside the measuring VM. Hosted-machine interference and
thermal/frequency variation outside the VM are uncontrolled.

Tritium: four `benches/prompts/short.jsonl` prompts, greedy, IntMlp numerics,
32 requested tokens per prompt, one warmup pass (up to four generated tokens
per prompt), one measured pass. Every recorded invocation decoded 128 tokens,
with 35 prompt tokens summed across the suite. Each invocation reports a median
of prompt rates; the table reports the median of four invocations. Speedups are
medians of four within-round ratios, not ratios of aggregate medians.

In the initial run, peak RSS is 1,222 MiB for both versions at every width.
Median invocation **minimum** TTFT moves from 2,185.7 to 1,692.7 ms at one
thread, 1,148.8 to 888.7 ms at two, and 631.0 to 489.8 ms at four. These are
minima over short prompts within an invocation, aggregated across invocations;
they are not median request latency or fixed-256-token TTFT. In the confirmation run, peak RSS remains 1,222 MiB; the corresponding
before/after TTFT minima aggregate to 2,173.4/1,674.1, 1,144.6/883.1 and
629.3/486.3 ms. Prefill throughput remains a zero placeholder and is not reported
as a measurement.

## Numerical validation and provenance

The independent Rust oracle executes eight positions under each of Reference,
Folded and IntMlp: **24 mode-position checks**, all with cosine >0.999999 and
matching top-1. The printed cosine is rounded to six decimals; `1.000000` does
not assert bit-identical floating-point logits. The workflow requires all 24
checks and rejects the ignored test's missing-checkpoint skip path.

The initial run checks before/after greedy text for four prompts at 1/2/4
threads with a 16-token limit. The confirmation run passes the full 32-token
benchmark limit at all 12 prompt/width pairs. Native scalar, NEON and
NEON-dotprod differential suites remain part of normal CI.

The model is converted on the ARM runner from the pinned public bf16 checkpoint:

- bf16 revision: `276681394656abdadb8e80e5b2c3db5e5d7fcaff`.
- Source safetensors SHA256: `529637ff6dab1f5890767356928693f69ffe61d3b6040a43de9306b37bfd5ae1`.
- `.trit` SHA256: `d245c5b58159cf1dad8116b7fa5b1144b2298c9954bea03eae74bc505b93060d`, identical to the existing x86 asset.
- Per-token weights: 521,011,200 ternary bytes plus 656,670,720 dense bf16 bytes;
  30 layers, hidden size 2560, FFN size 6912.

Exact checkout, compiler, CPU, model, tokenizer, suite, executable and shared
library identities accompany the raw records. Both Tritium versions report
`neon-dotprod`. The model is larger than L3, unlike the earlier 113 MB kernel
microbenchmark; this alone still does not establish a memory bandwidth roofline.

## Phase attribution

Candidate, confirmation runner, 128 decode steps per width; separate profiling
processes after the uninstrumented benchmark:

| Threads | Total ms/token | Ternary ms (share) | bf16 head ms (share) | Remainder ms (share) |
|---|---:|---:|---:|---:|
| 1 | 243.23 | 201.11 (82.7%) | 38.87 (16.0%) | 3.26 (1.3%) |
| 2 | 128.91 | 106.06 (82.3%) | 19.60 (15.2%) | 3.25 (2.5%) |
| 4 | 71.76 | 58.63 (81.7%) | 9.88 (13.8%) | 3.25 (4.5%) |

The dense head already has NEON and scales nearly fourfold in this range.
Ternary projections remain the main cost. The 640x2560 K/V projections cost
**9.70 / 9.70 / 9.71 ms/token** at 1/2/4 threads: the 256 KiB work-per-slot
threshold leaves each 0.41 MB call serial. At four threads, these calls account
for **13.5% of total profiled decode time**.

Most empty-dispatch probe results are about 0.6–2.7 us; the four-slot,
100-us-gap point reaches 9.14 us. These are diagnostics, not actual decode
synchronization attribution or a claim that every worker remains spinning.

**Next bounded experiment:** test a lower ARM dispatch threshold using
`TRIT_BYTES_PER_SLOT`, starting with 128 KiB, against the current default in
interleaved full-model runs. It would permit up to three slots for K/V on this
four-core host. This targets an observed serial cost without changing arithmetic.
The performance gain is unmeasured; retain the setting only if paired decode
improves and the numerical gates hold. Wider NEON kernel work remains relevant
because ternary projections consume most of the time.

The initial harness accidentally invoked `decode_profile --pool-probe` as
though that also profiled the model. That flag returns before model loading.
Independent review caught this: the corrected harness executes model profiles
and empty-dispatch diagnostics separately and requires ternary/dense/remainder
rows in the output. The initial `profile-t*` files are dispatch probes only.
The initial timing samples are retained; they do not supply model phase data.

The model profiler asserts 210 ternary projections and one dense-head call per
step. Phase timings include worker dispatch/synchronization. The remainder
includes other transformer operations, sampling and profiler bookkeeping.
Empty-dispatch timings are separate diagnostics, not an exact subtractable
synchronization share. Instrumented profile rates are not headline benchmark
rates.

## Why the bitnet.cpp I2_S rates are excluded

The attempted comparator pins Microsoft BitNet `0b341e582afbf9e1011f24744b554c96a3477eb5`
and its llama.cpp submodule `390c307752ab78fd8189f359d6954c9ba1be74af`. It uses
a Release Clang 18.1.3 build with `GGML_NATIVE=ON`, CPU only, ARM TL1 disabled,
and the public I2_S GGUF at revision `a1f2f1c765812aa8af3f6eda4a313707064bba15`.
The GGUF SHA256 is `4221b252fdd5fd25e15847adfeb5ee88886506ba50b8a34548374492884c2162`.

A declaration-scope repair was needed to compile on this ARM configuration:
`src1_cont` was declared under `GGML_USE_LLAMAFILE` and used outside it. The
committed patch only moves that unchanged declaration. It is retained with the
build metadata and does not repair the separate numerical defect below.

The selected dot implementation has an AVX2 branch and a scalar fallback. The
fallback already maps packed codes to signed ternary weights, while the active
ordinary matmul caller subtracts the activation sum again. This configuration
detects SVE/i8mm, which disables llamafile in that translation unit. The separate
I2_S tensor-traits backend is not registered. For 64 uniform +1 real activations
(quantized to 127):

| Uniform weight | Expected dot | After fallback + caller correction |
|---|---:|---:|
| −1 | −64 | −128 |
| 0 | 0 | −64 |
| +1 | 64 | 0 |

[Audit result](ARM-20260928-bitnet-i2s-audit.json). This locally compiled
source-branch probe selects the same non-AVX2 branch used on ARM; it is not a
full-model ARM logit comparison. Reproduce it with:

```sh
python3 benches/audit_bitnet_i2s.py \
  --llama-source /path/to/pinned/BitNet/3rdparty/llama.cpp --out audit
```

The script extracts the pinned upstream function unchanged and applies its
caller's correction. The diagnostic JSON records arithmetic pass/fail; compile
or extraction failures are errors. A passing three-case spot check would not
by itself establish full-model quality. See the pinned
[dot implementation](https://github.com/isHuangXin/llama.cpp/blob/390c307752ab78fd8189f359d6954c9ba1be74af/ggml/src/ggml-cpu/quants.c#L1390)
and [active I2_S caller](https://github.com/isHuangXin/llama.cpp/blob/390c307752ab78fd8189f359d6954c9ba1be74af/ggml/src/ggml-cpu/ggml-cpu.c#L1538).
An independent reproduction using the exact quantizer and GEMV wrapper at 2560
elements confirmed the same error: −5120/−2560/0 instead of −2560/0/2560.

The benchmark JSON displays `Q1_0`, but header inspection found 210 I2_S
projection tensors, 121 F32 tensors, one F16 embedding tensor, and zero Q1_0
tensors. The file's `general.file_type=40` metadata is interpreted as Q1_0 by
this newer enum; tensor types remain I2_S. This is a display-label mismatch,
not evidence that the downloaded weights were a different quantization.

Initial diagnostic tg32 rates were 0.688 / 1.371 / 2.695 tok/s at 1/2/4
threads. **These are not a valid competitive baseline and do not support a
Tritium-versus-BitNet speedup claim.** All raw rows are retained with that
qualification. The audit is specific to this pinned ARM configuration; it
does not invalidate the earlier x86 AVX2-path measurements or rank ARM TL1.

Even a corrected comparator would have workload differences: bitnet's tg32
starts with empty context and feeds synthetic random tokens, reports the mean
of three internal repeats, and does not sample/detokenize text. Tritium uses
short prompts and greedy generation. A validated I2_S revision or optimized
ARM TL1 comparison remains follow-up work. llama.cpp Q4_K_M was not measured
on this ARM runner; its historical x86 Qwen comparison is a different model.

## Historical x86 context

The 22 September interleaved A2 run on Ryzen 9 8945HX / WSL measured current
Tritium at 17.947 / 28.618 / 33.417 tok/s at 1/2/4 threads, and 34.790 at eight.
See [the x86 report](WSL-20260922-A2.md). Those are different-host,
different-date results, not paired cross-architecture speedups. No energy or
edge-device inference follows from this comparison.

## Reproduction and retained evidence

The full-model workflow is manually dispatched after this PR is merged;
lightweight harness regressions run in regular PR CI. Bootstrap runs used a
pull-request trigger. [Harness instructions](../README.md#native-arm-full-model-comparison).

Initial run: [samples and raw JSON](ARM-20260928-full-model-initial.json),
[metadata and command lines](ARM-20260928-full-model-initial-metadata.json),
[raw text](ARM-20260928-full-model-initial-raw.txt).
Confirmation run: [samples and raw JSON](ARM-20260928-full-model.json),
[metadata, phase JSON and commands](ARM-20260928-full-model-metadata.json),
[raw text and profiles](ARM-20260928-full-model-raw.txt).

The arithmetic audit was performed locally after the native timing runs; its
host/compiler identity is separate. Both native datasets are annotated with the
subsequently discovered comparator failure. Future workflow runs pass that audit
to the harness, which records the comparator as excluded and skips its timing.
The before/after Tritium order remains balanced. The original diagnostic timings
and all original artifact hashes remain available in the retained evidence.
