# Three-way comparison after the A2 dispatch fix

23 September 2026, same host and same session for all three runtimes, one
process per thread count. Greedy, 32 tokens. Tritium decodes the four-prompt
suite and reports the median across prompt/run decode rates; the baselines
generate from an empty context through `llama-bench`, median of three, because
`llama-bench` takes no prompt file. [Raw CSV](WSL-20260923-compare.csv).

Reproduced with the commands in the repository README, against
llama.cpp `eab8ee4` and bitnet.cpp `0b341e5` built on 2026-08-25.

| threads | tritium | llama.cpp Q4_K_M | bitnet.cpp I2_S | vs bitnet.cpp |
|---|---|---|---|---|
| 1  | **17.29** | 14.71 | 12.70 | 1.36x |
| 2  | **26.41** | 22.49 | 20.43 | 1.29x |
| 4  | **34.53** | 22.80 | 28.08 | 1.23x |
| 8  | **34.03** | 25.64 | 31.13 | 1.09x |
| 16 | **32.64** | 23.66 | 30.28 | 1.08x |

bitnet.cpp runs the identical checkpoint, so that is the apples-to-apples
column: **Tritium now leads it at every thread count**, where before the A2 fix
it led at one, two and four and lost at eight and sixteen. llama.cpp runs
Qwen2.5-3B Q4_K_M because mainline cannot load the `i2_s` type -- a different
model at a different quality point, which is why the bits-per-weight column is
in the CSV and why this table is not "ternary beats 4-bit".

## What changed against the August run

The August comparison on this host read 27.90 / 25.93 / 20.72 for Tritium at
4 / 8 / 16 threads against bitnet.cpp's 26.72 / 32.49 / 31.21. Our column moved
because of the A2 dispatch fix. The baseline columns moved too, and by enough
to be worth stating:

| | Aug 25 | Sep 23 |
|---|---|---|
| bitnet.cpp, 8 threads | 32.49 | 31.13 |
| bitnet.cpp, 16 threads | 31.21 | 30.28 |
| llama.cpp, 4 threads | 27.83 | **22.80** |
| tritium, 1 thread | 19.40 | 17.29 |

bitnet.cpp is within 4% across the two sessions, which is the main reason to
trust this one. **llama.cpp's four-thread figure is not**: 22.80 against 27.83
is an 18% drop, and it sits below its own eight-thread number, which its August
curve did not. That single point should be treated as suspect until re-run. It
does not affect the bitnet.cpp comparison, which is the one that matters,
because the two baselines are independent invocations.

The August session ran from the Windows-side filesystem and this one is
WSL-native, and Tritium's own one-thread figure moved 19.40 to 17.29 across the
same gap, so some session-level offset affects everything here. That is exactly
why both sides are measured in one session rather than carried across.

## Limitations

- Short context, batch 1, empty-context generation for the baselines against a
  four-prompt suite for Tritium. See [the benches README](../README.md) for what
  that does and does not make comparable.
- No energy measurement: no RAPL zone is exposed in this environment, so the
  `joules_per_token` column is empty for all three runtimes rather than
  estimated.
- x86-64 only. No ARM figure for any of the three.
- One measurement per thread count per runtime, each already a median of three
  internally. This is not an interleaved paired experiment between runtimes,
  and it is not claimed as one.
