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

Where that leaves the numbers, measured on a Ryzen 9 8945HX over the four-prompt
short suite. The frozen single-prompt reference in [06-GATES.md](06-GATES.md)
reads slightly lower (14.89 tok/s, 470 ms, 27.3 GB/s) because it is a different
input, and it — not this table — is what a regression is measured against:

| | Pre-pivot | Now |
|---|---|---|
| Decode | 0.16 tok/s | 15.7 tok/s |
| Time to first token | ~33 s | 440 ms |
| Peak RSS | 6.10 GiB | 1.80 GiB |
| Achieved bandwidth | ~0.3 GB/s | 29.0 GB/s (~61-68% of the measured roofline) |

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

## A2. Parallel scaling

**The largest lever in the project, by a wide margin.** Tritium is ordinary per
core and last by a factor of two in aggregate. If it scaled like bitnet.cpp,
13.28 tok/s at one thread becomes roughly 34 at eight — before any other
change, and past bitnet.cpp's 31.90.

The cause is measured, not guessed. A decode step issues 210 ternary matvecs
whose largest is 4.4 MB, and each one currently pays a rayon fork/join.
Lowering `PARALLEL_MIN_BYTES` to let them parallelize makes things dramatically
worse, not better:

| threads | layer time |
|---|---|
| 1 | 37.4 ms |
| 2 | 66-74 ms |
| 4 | 112-115 ms |
| 8 | 173-175 ms |

Even two threads is 1.8x worse. So `PARALLEL_MIN_BYTES` is correct and must
stay until something cheaper than per-call fork/join exists. ggml gets its
scaling from a persistent pool with cheap barriers; that is the shape of the
fix, and it is a real piece of engineering rather than a tuning change.

**Gate:** decode tok/s at 1, 2, 4, 8 and 16 threads published next to the
baseline curve, with the same suite. The number that matters is the *slope*,
not the peak.

## A3. The f32 LM head

Still worth doing, at a smaller number than first estimated, and now
independently corroborated: bitnet.cpp stores `token_embd` as f16 where Tritium
stores f32, so it moves ~1178 MB per token against our 1834 MB. Part of its
advantage is simply that.

The instrumented decode puts the head at 26.9 ms of a 64.4 ms token — 42% of
the time, not the 72% the byte share suggests, because the head runs at
48.8 GB/s while the ternary path runs at 13.9. Halving its bytes should take
the token to roughly 51 ms: about **1.26x**, plus peak RSS from 1846 MB to
around 1190 MB.

The values are already bf16 — `tritc` widens them from the checkpoint on the
way in and nothing ever adds precision — so storing and reading them as bf16 is
lossless relative to the source. There is no quality experiment to run, which
is why this ranks above anything that trades accuracy.

**Gate:** bytes/token and decode tok/s measured before and after, with G2 held
at its frozen value.

## A4. Energy per token

J/token is named "the headline metric" in [04-BENCHMARKS.md](04-BENCHMARKS.md)
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

The NEON kernels are *correct* — executed on aarch64 CI hardware, matching the
reference across the differential corpus including the worst case for the `i16`
accumulators. They have never been *timed*, so no ARM performance claim exists.

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
