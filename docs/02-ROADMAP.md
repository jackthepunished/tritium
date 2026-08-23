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

Ordered by evidence, not by preference. Each entry states why it is where it is.

## A1. Baselines, actually run

**The largest credibility gap in the project.** `benches/` is built, records a
row per baseline, and honestly reports `unavailable` — because neither llama.cpp
nor bitnet.cpp is installed on the machine every published number came from. So
every comparison so far is Tritium against its own past, plus a roofline measured
on the same host.

This is first because it is cheap, it is blocking every efficiency claim the
project exists to make, and it is the one item where the answer could be
genuinely unflattering. A ternary runtime that loses to llama.cpp Q4_K_M on the
same host is something we need to know before building anything else on the
premise that it does not.

- Install llama.cpp and bitnet.cpp; run `benches/run.sh` with both present.
- Same prompts, same thread count, same context cap, greedy decoding.
- Commit the raw CSV.

**Gate:** a committed CSV with `status=ok` rows for both baselines, and a
published comparison — whichever way it goes.

## A2. The f32 LM head

**The biggest remaining technical lever, by arithmetic.** At batch 1 the tied
f32 head moves 1313 MB per token against 521 MB for every ternary projection
combined. The ternary kernels are already at 32x scalar; there is far more left
in the 72% of traffic that is not ternary than in the 28% that is.

An int8 head would cut bytes/token from 1834 MB to roughly 849 MB. If the same
fraction of roofline held, that is about 2.2x — larger than any remaining kernel
work can plausibly return. It is a prediction from the roofline, and the point of
the milestone is to test it rather than to quote it.

The open question is quality cost, which is unmeasured. G2 is the arbiter.

**Gate:** bytes/token and decode tok/s both measured before and after, with G2
held at its frozen value or the regression argued explicitly in
[06-GATES.md](06-GATES.md).

## A3. Energy per token

J/token is named "the headline metric" in [04-BENCHMARKS.md](04-BENCHMARKS.md),
and the project has never reported one, because the reader refuses to estimate
and no machine in the loop has exposed a counter. The development host has no
RAPL zones at all.

This is blocked on hardware access rather than on work: it needs a box with a
real energy counter, or an inline meter. Until then the column stays empty and
the source reads `none`, which is the correct behaviour and not a placeholder to
be filled with a guess.

**Gate:** J/token reported from a real counter on at least one machine, next to
a baseline measured the same way on the same machine.

## A4. ARM throughput

The NEON kernels are now *correct* — executed on aarch64 CI hardware, matching
the reference exactly across the differential corpus including the worst case for
the `i16` accumulators. They have never been *timed*, so no ARM performance claim
exists and none should be made.

This matters disproportionately for positioning: the edge devices the project
targets are overwhelmingly ARM, so "verified on x86" is a weaker story than it
looks.

**Gate:** decode tok/s and bytes/token on real ARM hardware, published next to
the x86 figures, with the kernel named.

## A5. KV cache precision and context

f32 and preallocated: 315 MB at a 2048 cap. int8 pages would cut that
substantially, and the ratio gets worse as context grows. Lower priority than
A1-A2 because it is resident footprint rather than per-token traffic, so it
moves RSS rather than tok/s.

**Gate:** RSS measured before and after at equal context, G2 held.

## A6. Batched prefill

Currently batch 1 everywhere. Prefill is the compute-bound phase, so it is where
batching actually pays, and it is also what would make `PARALLEL_MIN_BYTES` worth
revisiting — with a measurement, as before.

Explicitly last: it is a throughput feature for a serving story the project does
not yet have, and it should not jump the queue ahead of claims that are already
being made.

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
