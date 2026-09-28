# ARM mask expansion: paired native measurements

28 September 2026. Native `ubuntu-24.04-arm` CI, Neoverse-N2, four exposed
cores, 128 MiB reported L3. Rust 1.98.1 / LLVM 22.1.8. Baseline is merged
PR #11 (`902fc8a`); candidate source is `2b55288`. Exact checkout and binary
identities are in [metadata](ARM-20260928-mask-shuffle-metadata.json).

## Change and evidence

The existing `spread16` constructs two repeated bytes with scalar shifts and
broadcasts. Cross-compiled release assembly shows 16 byte broadcasts, 14
shifts and eight lane inserts per nonzero beat. Replace these with a register
`TBL` shuffle: two transfers to vector registers and eight shuffles per beat.
Four shuffle-index constants load outside the loop. The old bit-selector load
was also outside the loop; the earlier roadmap claim that it loaded on every
group was incorrect. Instruction counts alone do not establish throughput.

Both NEON kernels use the helper. Accumulation, the int16 flush schedule,
zero-beat handling, floating-point operations and the file layout are unchanged.

## Native comparison

The comparison job builds baseline and candidate with the same toolchain,
finishes both builds before measuring, then runs four pairs per thread count
in alternating AB/BA order. Each invocation uses its own process and the
unchanged kernelbench dimensions and repetition counts. Table times are
medians of four invocations; speedups are medians of the four paired ratios.

| Measurement | Threads | Baseline ms | Candidate ms | Paired speedup |
|---|---:|---:|---:|---:|
| NEON dot-product, 2560x6912 | 1 | 2.35 | 1.74 | 1.351x |
| Baseline NEON, 2560x6912 | 1 | 2.50 | 1.89 | 1.323x |
| Scalar control, 2560x6912 | 1 | 9.895 | 9.895 | 1.001x |
| Larger 65536x6912 tensor | 1 | 60.535 | 44.630 | 1.356x |
| Larger 65536x6912 tensor | 2 | 30.175 | 22.320 | 1.352x |
| Larger 65536x6912 tensor | 4 | 19.510 | 11.465 | 1.325x |

The single-thread dot-product pairs all round to 1.351x. The two-thread larger
tensor pairs range from 1.350x to 1.353x. Four-thread results are noisy: larger
tensor ratios range from 1.174x to 2.123x, and unchanged dense controls also
move substantially. Do not claim dense-path improvements or a precise
four-thread speedup from this run. Inputs and output times are rounded by
kernelbench to two decimals in milliseconds.

## Limits and corrections

These are native kernel measurements, not QEMU timings or model decode rates.
The 113 MB tensor is smaller than this runner's reported 128 MiB L3. The old
label "well past L3" was false for this host; neither this run nor the earlier
ARM numbers establish sustained DRAM bandwidth. The cached dot-product kernel
processes about 2.54 GB/s of packed weights after the change, versus 1.88 before.
That is effective weight throughput, not a memory-controller measurement.

The measured gain supports reducing mask-expansion work. It does not prove
that mask expansion is the sole bottleneck, that all ARM CPUs benefit equally,
or that whole-model decode improves by the same percentage. No checkpoint was
loaded on ARM, no energy was measured, and A5's full-model decode gate stays open.

## Reproduction and correctness

`benches/compare_arm.py BASELINE_BINARY CANDIDATE_BINARY --out DIRECTORY` runs
on a native aarch64 host after both kernelbench binaries have been built.
The workflow `.github/workflows/arm-kernel-comparison.yml` records the CPU,
compiler, checkouts, binary hashes and raw output. It uses the PR base checkout
and candidate checkout on the same runner.

Local checks passed: all 65,536 masks in each of four byte groups (262,144
expansions), the differential suite forced to scalar/NEON/NEON-dotprod under
QEMU (30 passed), host CPU tests (18 passed), aarch64 clippy and formatting.
Native full-suite validation also passed, including the exhaustive mask test
and separate forced `neon`, `neon-dotprod` and `scalar` corpus runs:
[ARM test job](https://github.com/jackthepunished/tritium/actions/runs/36463758119/job/109068551359).

[Raw output](ARM-20260928-mask-shuffle-raw.txt),
[paired samples and summaries](ARM-20260928-mask-shuffle.json), and
[native run](https://github.com/jackthepunished/tritium/actions/runs/36463758172).
