# Roadmap

## How this document changed

The original roadmap was six numbered weeks ending in an FPGA demo box, written
when the plan was "prove the primitive in simulation, buy a board, ship the
demo." That plan produced everything it promised through its own week 3b, and
then the project pivoted: the runtime became hardware-agnostic, and the same
ternary primitive now runs usefully on commodity CPUs as well as on the fabric
it was designed for.

Two things follow. First, week numbering is gone. It was already fiction —
the pivot alone took longer than a week — and this project has always actually
run on **gates**: a milestone is done when a stated, measured condition holds,
not when a calendar says so. Second, there are now two tracks. They share the
`.trit` format, the correctness gates and most of the code, and they are worked
in whatever order the evidence favours rather than strictly in sequence.

What has not changed: every milestone ends in a public artifact, every
performance claim ships with a script that reproduces it, and misses get
published rather than buried.

## Done

| | Outcome | Gate |
|---|---|---|
| Ternary inference, correct | `tritsim` decodes BitNet b1.58 2B4T in pure Rust, no multiplies in the hot loop | G2: mean logit cosine 0.9991 vs the HF reference, top-1 100% |
| `.trit` v0 and the RTL core | Multiplier-free SystemVerilog `tritcore`, driven by oracle-generated vectors | G4: bit-exact on every set including the `x = -128` extremes; G5: Yosys asserts zero `$mul`/`$macc` |
| Numerics | Per-stage ranges measured on the real checkpoint; norm folding removes the rsqrt and divide from the datapath | Identical compare metrics and greedy output, property-tested across 1000 adversarial vectors |
| Hardware in the loop | The real 2B model decodes end-to-end through the Verilated core | G6: byte-identical to the CPU backend, at 24.4 s/token |
| Integer-exact MLP | The squared-ReLU stage never exists in f32 | Greedy text character-identical on both prompts |
| **The pivot** | `.trit` v1 bit planes, SIMD kernels, host daemon, C ABI, comparative harness | G7 moved 93x on decode; G1-G6 unmoved |
| Review hardening | 14 findings fixed, including an `i16` overflow in the NEON kernel and a streaming detokenizer that destroyed every multi-byte codepoint | NEON kernels executed on aarch64 hardware for the first time; all gates re-verified |

Where that leaves the numbers, measured on a Ryzen 9 8945HX at 4 threads over
the four-prompt suite. [06-GATES.md](06-GATES.md) is the frozen reference; this
host drifts between runs, so treat absolutes as approximate and speedups as
claims that need interleaved pairs:

| | Pre-pivot | After the pivot | After A2+A3 |
|---|---|---|---|
| Decode | 0.16 tok/s | 14.89 tok/s | **27.90 tok/s** |
| Time to first token | ~33 s | 470 ms | **239 ms** |
| Peak RSS | 6.10 GiB | 1.80 GiB | **1.19 GiB** |
| Bytes per token | 1834 MB | 1834 MB | **1178 MB** |

## The board-purchase gate

Still open, and deliberately so. The selection memo
([04b-BOARD-MEMO.md](04b-BOARD-MEMO.md)) recommends a Kria KV260 on
roofline-versus-synthesis grounds, and the gate that was meant to unblock the
purchase **passed** when the real model decoded through the Verilated core.

The pivot changed the urgency, not the conclusion. A working CPU runtime means
the project has something demonstrable without a board, which removes the
schedule pressure that would otherwise have forced the purchase early. The
decision is Bahadir's and nothing here commits money.

---

# Track A — the runtime

Ordered by evidence, not by preference. Each entry states why it is where it
is, and the order changed once the baselines were actually run — measuring
beat reasoning, which is the whole point of putting A1 first.

## A1. Baselines, actually run — DONE

Both baselines are installed, measured and committed
(`benches/results/DESKTOP-HV2MQTM-20260823.csv`). Same host, same prompts,
greedy, 32 tokens, median of three invocations:

| threads | tritium | llama.cpp Q4_K_M | bitnet.cpp I2_S |
|---|---|---|---|
| 1  | 13.28 | **13.91** | 12.54 |
| 2  | 14.61 | **20.01** | 19.64 |
| 4  | 15.23 | 25.54 | **27.29** |
| 8  | 15.23 | 24.72 | **31.90** |
| 16 | 14.98 | 22.84 | **30.51** |

bitnet.cpp runs the identical checkpoint, so that is the apples-to-apples
column. llama.cpp runs Qwen2.5-3B Q4_K_M — mainline cannot load the `i2_s`
type and its converter has no BitNet entry — so it is a different model at a
different quality point, and the bits/w column carries that.

**The answer was unflattering, which is what this item was for.** All three
runtimes land within 11% of each other at one thread. Only Tritium fails to
scale: 1.15x from one thread to eight, against bitnet.cpp's 2.54x and
llama.cpp's 1.78x, ending 2.09x behind bitnet.cpp.

Nothing here says the kernels are bad; per core we are unremarkable rather than
ahead. Everything here says the runtime cannot use a machine.

An earlier version of this table claimed Tritium was the fastest of the three
per core, by 1.29x. That was a measurement artifact: our model file had been hot
in page cache for an hour of testing while the baseline GGUFs were freshly
downloaded and cold. `benches/run.sh` now pre-warms every model before
measuring, and `tritd bench` reports a median rather than its best run, so its
figure is produced the same way the baselines' are.

Two levers fall out of it, and they reorder the rest of this track.

## A2. Parallel scaling — DONE, partly

`crates/trit-cpu/src/pool.rs` replaces per-matvec fork/join with a persistent
pool on a sequence counter. Four mechanisms were measured in the real decode
loop first; a dedicated work-stealing pool and a broadcast primitive both stayed
slower than not parallelising at all.

**Worth 1.17x**, measured in interleaved pairs against the previous
implementation because this host drifts up to 30% between runs. That is well
short of the 1.59x the isolated prototype suggested, and the smaller number is
the true one.

The gate asked for the slope, not the peak, and the slope improved: 1.34x from
one thread to eight, against 1.15x before. It is still short of bitnet.cpp's
2.39x, and the curve now peaks at four threads and declines rather than
flattening. **That decline is what is left of this item**, and it is why the
automatic thread count caps at eight.

Worth noting the desktop understates it. On a four-core edge part, which is the
hardware this project exists for, going from effectively single-threaded to
using the cores is worth more than it is on a 32-thread machine that was already
fast.

## A3. The f32 LM head — DONE

`tritc` was reading the checkpoint's bf16 tensors, widening them to f32 and
storing f32. Two-dimensional dense tensors now keep the source precision. The
1-D norm gains stay f32: a few kilobytes each, and they feed the absmax
quantiser.

**Worth 1.37x**, measured in paired runs with the same binary and only the model
file differing. That beat the 1.26x projection, the first estimate in this
project to come in under rather than over. Bytes per token 1834 MB to 1178 MB,
peak RSS 1846 MB to 1220 MB, model file 1836 MB to 1179 MB.

Exact with respect to the source, so there was no quality experiment to run:
G2 held at 0.9991 / 100% and G3 stayed byte-identical.

Independently corroborated by the competition. bitnet.cpp already stored
`token_embd` as f16, and that was most of its remaining advantage; closing it is
what moved Tritium ahead per core.

## Where that leaves the comparison

Same host, same session, greedy, 32 tokens, median of three:

| threads | tritium | llama.cpp Q4_K_M | bitnet.cpp I2_S |
|---|---|---|---|
| 1  | **19.40** | 15.40 | 13.61 |
| 2  | **26.73** | 22.06 | 19.10 |
| 4  | **27.90** | 27.83 | 26.72 |
| 8  | 25.93 | 26.18 | **32.49** |
| 16 | 20.72 | 23.88 | **31.21** |

Ahead at 1, 2 and 4 threads, and 1.43x bitnet.cpp per core on the identical
checkpoint, having been last in that column before A2 and A3. Still behind above
four threads, where bitnet.cpp keeps climbing and Tritium declines.

## A4. Energy per token

**Now the top open item on this track.** J/token is named "the headline metric"
in [04-BENCHMARKS.md](04-BENCHMARKS.md)
and has never been reported, because the reader refuses to estimate and no
machine in the loop has exposed a counter. The development host has no RAPL
zones at all.

Blocked on hardware access rather than on work. Until then the column stays
empty and the source reads `none`, which is the correct behaviour and not a
placeholder to be filled with a guess. Now that both baselines run on this
host, an energy-capable box would produce the comparison the project exists to
make.

**Gate:** J/token from a real counter, next to a baseline measured the same way
on the same machine.

## A5. ARM throughput

Software preparation, 2026-09-22: kernelbench now measures one thread count per
process, avoiding the global pool's silent serial fallback at later widths.
A new sparse flush-boundary regression exposed a baseline NEON overflow; the
fix passes the actual ARM kernels under QEMU, with native CI validation still
pending. The fresh x86 baseline and reproduction commands are in
[the WSL validation report](../benches/results/WSL-20260922.md). There is no
current access to a physical target device, so the end-to-end ARM gate remains
open; emulation timings are not ARM throughput measurements.

The earlier differential corpus passed on aarch64 CI hardware, including wide
all-nonzero rows. The newly added sparse cases extend that coverage as described
above. No ARM timing has been published, so no ARM performance claim exists.

This matters disproportionately for positioning: the edge devices this project
targets are overwhelmingly ARM, so "verified on x86" is a weaker story than it
looks. It sits below A2 only because the scaling defect would travel to ARM
unchanged, and measuring it twice would be wasted work.

**Gate:** decode tok/s on real ARM hardware, published next to the x86 figures,
with the kernel named.

## A6. KV cache precision and context

f32 and preallocated: 315 MB at a 2048 cap. int8 pages would cut that
substantially, and the ratio worsens as context grows. Below A2-A3 because it
moves resident footprint rather than per-token traffic — RSS, not tok/s.

**Gate:** RSS measured before and after at equal context, G2 held.

## A7. Batched prefill

Batch 1 everywhere today. Prefill is the compute-bound phase, so it is where
batching pays, and it is also what would make `PARALLEL_MIN_BYTES` worth
revisiting — after A2, and with a measurement.

Explicitly last: a throughput feature for a serving story the project does not
yet have.

---

# Track B — silicon

The RTL is not a museum piece and the pivot did not retire it. It is a backend
that CI exercises on every push, its bit-exactness is a frozen invariant, and it
is the only path to the efficiency numbers that justify the whole thesis. What
changed is that it is no longer the *only* way to have a demo, which means it can
be done properly rather than urgently.

## B1. Board bring-up

Follows the purchase. Port the sequencer, get a single transformer layer running
end-to-end on the part, output matching the oracle.

**Gate:** one layer on hardware, bit-identical to `tritsim`. Artifact: scope or
ILA capture next to the golden vectors.

## B2. Achieved bandwidth against the roofline

The number the architecture document has been predicting without evidence.
Measure achieved GB/s, lane utilization and clock, and compare to the roofline
the board was chosen on.

**Gate:** measured GB/s published against the predicted figure, including the
gap and its explanation.

## B3. Full model on the part

Layer sequencer, host-driven descriptor chain, double-buffered DMA, KV cache with
a BRAM hot window. Embeddings and head on the PS side if fabric-constrained —
which, given section 3 of the architecture, is where they belong anyway.

**Gate:** a full decode on hardware, output matching the oracle, with tok/s and
J/token measured on the 12V rail.

## B4. Timing closure and the honest RTL limits

Two things are known-unfinished and should not be discovered by a board:

- The 64-term single-cycle reduction and 64 parallel activation reads are fine
  under Verilator and are not placed on a part.
- `MAX_COLS` is 8192. Every projection in this model class fits, and the wrappers
  now reject anything wider rather than wrapping the address silently — but a
  larger model would need real banking.

---

## Risks

| Risk | Mitigation |
|---|---|
| A baseline beats us on the same host | Publish it. A ternary runtime that loses to Q4_K_M on x86 is a finding about x86, not about ternary — but only if we measured it before claiming otherwise. This is why A1 is first. |
| The int8 head costs real quality | G2 is the arbiter and it is frozen. Ship it as a flag rather than a default if the cosine moves; the bytes/token win is not worth a lobotomized model. |
| No hardware with an energy counter | J/token stays unreported. The column being empty is the honest outcome, and estimating one would poison the single claim the project exists to make. |
| ARM turns out slow | Then say so. The NEON kernels were written against the same contract as the x86 ones and are correct; if they are slow, that is a kernel problem with a known shape, not a surprise. |
| Board arrives and timing does not close | Reduce lane count. The design scales roughly linearly with lanes and the roofline says the memory link binds long before the adder tree does. |
| The project drifts into being a CPU inference engine | The RTL parity gate runs on every push. Drift is detectable by construction, not by vigilance. |

## Ground rules

- Every performance claim ships with a reproducible script.
- `tritsim` is the source of truth. The CPU runtime and the RTL are both wrong
  until they match it.
- Gates move only with an argued entry in [06-GATES.md](06-GATES.md).
- Energy is measured or absent, never estimated.
- Build in public. No emojis in project communications.
