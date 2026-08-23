# Architecture

## 1. The core insight

A BitNet b1.58 linear layer computes `y = W·x` where every weight is -1, 0 or +1
and activations are int8. The inner loop is therefore not multiply-accumulate but
**select-accumulate**:

```
for each weight w, activation a:
    w == +1  ->  acc += a
    w == -1  ->  acc -= a
    w ==  0  ->  skip
```

The original reading of this was "so build hardware without multipliers." That is
still true and still the endgame. What the pivot established is that the same
observation pays on silicon people already own, provided the weights are stored
so that a machine can act on them without decoding first:

- On a CPU, "does this weight participate, and with which sign" becomes a **mask**
  over a vector of activations. On AVX-512 the mask *is* a `k` register and the
  whole operation is two masked loads plus two `vpdpbusd`. Measured 32.5x the
  scalar path on this host.
- On fabric, it is a mux and an adder tree, needing no DSP blocks, so a modest
  part can host thousands of lanes.

Both want the identical thing from memory. That is what section 2 is about, and
it is the single structural idea this codebase is organized around.

## 2. The one design commitment: bit planes

A ternary weight has three states, so two bits is the natural budget however you
spend them. Version 0 of the `.trit` format spent them as an **interleaved code**
per weight (`00` = 0, `01` = +1, `10` = -1). Version 1 spends the same two bits
**planar**: a 16-byte *beat* carries 64 consecutive columns of one row as
`{u64 pos, u64 neg}`.

The bit count is identical. What changes is what one load can do with it.

```
y[r] = sum of x[c] where W[r][c] = +1
     - sum of x[c] where W[r][c] = -1
```

With planes, each of those sums is a mask applied to a vector of activations —
no branch, no table lookup, no per-weight arithmetic. With interleaved codes,
extracting the sign of lane `k` means shifting by `2k` and masking, per weight,
which defeats vectorization entirely and is exactly why the pre-pivot scalar
kernel ran a branch per weight at 0.16 tok/s.

The consequence that matters most: **one beat is simultaneously one iteration of
the CPU kernel's inner loop and one cycle's weight input to the RTL core.** A
memory-mapped `.trit` is not a container the runtime decodes into some other
layout. It *is* the layout. Nothing between the page cache and the accumulator
rewrites a byte, on either target.

That is what makes "hardware-agnostic" a structural property rather than an
aspiration. The CPU and the fabric are not two ports of the same idea kept in
sync by discipline; they consume the same bytes.

Full specification: [01-TRIT-FORMAT.md](01-TRIT-FORMAT.md).

## 3. The honest bandwidth math (read this first)

Autoregressive decoding at batch size 1 touches every weight once per token, so
throughput is bounded by:

```
tok/s ~= effective_memory_bandwidth / bytes_touched_per_token
```

Measured on BitNet b1.58 2B4T, reported by `tritd info`:

| | bytes/token | share |
|---|---|---|
| Ternary weights (all 30 layers, every projection) | 521,011,200 | 28% |
| `lm_head` / tied embeddings, f32 | 1,313,341,440 | 72% |
| **Total** | **1,834,352,640** | |
| KV cache, f32, preallocated at 2048 context | 314,572,800 | not per-token |

**The LM head is 2.5x every ternary projection combined.** This is the single
most important number in the document, and it is not what the pre-pivot
architecture expected: the table this section used to carry estimated a "~600 MB
packed" model and reasoned entirely about ternary weight streaming. On a
tied-embedding model with a 128256-word vocabulary at f32, the non-ternary tail
dominates the ternary body more than 2:1.

So the runtime's dense f32 matvec is not incidental. `MatvecBackend::f32_matvec`
exists as a first-class trait method for exactly this reason — a backend that
vectorizes the ternary kernel and leaves the head scalar has not moved end-to-end
throughput at all.

### Where that leaves us

Measured on an AMD Ryzen 9 8945HX (Zen 4, AVX-512 VNNI), 4 threads,
`benches/prompts/short.jsonl`:

| | |
|---|---|
| Decode | 15.7 tok/s |
| Achieved bandwidth | 29.0 GB/s |
| Streaming-read probe, same host, same run | 42-47 GB/s (run to run) |
| Fraction of roofline | ~61-68% |
| Peak RSS | 1846 MB |
| Time to first token | 440 ms |

The roofline figure is a **measured ratio**, not an assumption about what the
memory system can do: `tritd bench` runs a streaming-read probe on the same
machine in the same invocation and divides by it. The probe varies a few GB/s
between runs, so the percentage is quoted as a range rather than to the digit.

These are a fresh four-prompt suite run and sit slightly above the frozen
single-prompt figures in [06-GATES.md](06-GATES.md) (14.89 tok/s, 470 ms,
27.3 GB/s). Different input, different number: the gates file is the regression
reference and is not restated here. Quote it, not this table, when comparing
against a past run.

Consequences baked into the design:

- **Weight streaming, not weight caching.** Weights pass through once per token;
  only activations, norms and KV pages want to be resident.
- **Prefill is the exception.** With N prompt tokens you amortize one weight pass
  over N tokens of work, so prefill runs compute-bound. TTFT looks
  disproportionately good; steady-state decode is the honest number.
- **Bytes are not time. Measure both.** This document originally argued from
  the byte share alone and concluded the head was the thing to attack. An
  instrumented decode says otherwise: the head is 26.9 ms of a 64.4 ms token,
  42% of the time against 72% of the bytes, because it runs at 48.8 GB/s while
  the ternary path runs at 13.9. The two halves sit at opposite ends of the
  machine's efficiency range, so the byte split systematically misleads about
  where time goes. Halving the head's bytes is still worth about 1.26x, and is
  worth doing because it is free — see [02-ROADMAP.md](02-ROADMAP.md) A3 — but
  it is no longer the largest item.

### Where the time actually goes

| | per token | share | achieved |
|---|---|---|---|
| 30 transformer layers (ternary + attention) | 37.5 ms | 58% | 13.9 GB/s |
| LM head | 26.9 ms | 42% | 48.8 GB/s |

The head is already at roughly 92% of this machine's ~53 GB/s ceiling, so no
kernel work will move it: forcing the dense kernel to scalar, AVX2 and AVX-512
in turn gives 28.8, 28.1 and 27.1 ms. The ternary path is the opposite — it is
compute-bound in the kernel at about 15.8 GB/s per core, barely above what it
achieves from DRAM, which means it would scale with cores if the runtime could
use them. Section 7 is why it cannot.

## 4. System overview

```
   HF checkpoint
        |
        v
   +---------+   folds norms, quantizes, packs planes, verifies
   |  tritc  |
   +---------+
        |  .trit v1  (mmap'd; the payload IS the beat stream)
        v
   +-------------------------------------------------------+
   |  trit-core: format, transformer, KV cache, RoPE,       |
   |             sampler, tokenizer traits, numerics ladder |
   +-------------------------------------------------------+
        |  MatvecBackend  (the one seam: bit-exact by contract)
        +----------------------+----------------------+
        v                      v                      v
   +----------+          +-----------+         +---------------+
   | trit-cpu |          | trit-rtl  |         | ScalarBackend |
   | AVX-512  |          | Verilated |         | portable ref  |
   | AVX2     |          | tritcore  |         +---------------+
   | NEON     |          +-----------+
   | scalar   |                |
   +----------+                v
        |               rtl/trit_matvec.sv
        |               (multiplier-free, 64 lanes)
        v
   +---------+   run | serve | bench | info,  plus a C ABI
   |  tritd  |
   +---------+

   tritsim: an independent second implementation of the whole model,
            deliberately naive. Not in this data path -- it is the
            oracle the data path is diffed against.
```

| Crate | Role |
|---|---|
| `tritc` | Converter. HF BitNet checkpoint to packed `.trit` v1; folds norms, records per-tensor scales, verifies the result. |
| `trit-core` | The model. Format reader, transformer, KV cache, RoPE, sampler, tokenizer traits. No `unsafe` outside one mmap. |
| `trit-cpu` | Bit-sliced kernels: AVX-512 VNNI, AVX-512BW, AVX2, NEON, NEON+dotprod, portable scalar, and a bit-serial popcount path behind a feature flag. |
| `trit-rtl` | The Verilated `tritcore` behind the same backend trait. |
| `tritd` | Host daemon and C runtime: `run`, `serve`, `bench`, `info`, plus a C ABI in `include/tritium.h`. |
| `tritsim` | Independent golden reference. The oracle every other path is diffed against. |
| `rtl/` | Multiplier-free SystemVerilog `tritcore`, Verilator testbenches, Yosys synthesis check. |
| `benches/` | Comparative harness against llama.cpp / bitnet.cpp baselines. |

### 4.1 The backend seam

`MatvecBackend` is the one abstraction that matters, and its contract is unusual:
implementations must be **bit-exact** with the portable reference for every
input, including `xq = -128`. Not "within tolerance" — identical.

That is achievable by construction rather than by tuning, because the ternary
accumulators are `i32` and integer addition is exact and order-independent. A
backend that cannot meet it is wrong, not approximate. It is what lets the CPU
kernel, the RTL core and the reference be swapped for one another without moving
a logit, and it is why the differential tests can demand equality instead of a
threshold.

The trait is **injected, not global**. A process-wide selector works for a
single-shot CLI and breaks the moment two sessions want different backends.

### 4.2 Kernel selection

Resolved once by runtime feature detection, cached for the process. `--kernel`
(or `TRIT_CPU_KERNEL`) pins one and **errors** if this CPU cannot run it, rather
than falling back. A silent fallback would let a CI runner without AVX-512 report
green while actually re-testing the scalar path, and would let a benchmark
measure a different kernel than the one it names.

Measured on a 2560x6912 tensor with the planes resident in L3:

| Kernel | vs scalar |
|---|---|
| `avx512vnni` | 32.5x |
| `avx512bw` | 23.9x |
| `avx2` | 13.9x |
| `bitserial` (popcount) | 1.5x |
| `neon`, `neon-dotprod` | correctness verified on aarch64 CI hardware; no timing taken |

### 4.3 About popcount

Popcount is the natural primitive when *activations* are also 1-bit. Here weights
are 1.58-bit but activations are int8, so a popcount of a weight mask only counts
terms — it cannot recover the sum. The production kernels mask activation bytes
and accumulate the two planes separately.

The bit-serial popcount formulation is implemented anyway, behind a feature flag,
because it is the one that maps directly onto the RTL adder tree. It measures
22x slower than AVX-512 at int8 activations. It stays as a third independent
implementation for the differential tests, and because it becomes the right
kernel if activations ever drop below 8 bits.

## 5. Correctness architecture

Three implementations of the same arithmetic must agree:

- **`tritsim`** — the oracle. Naive, scalar, deliberately obvious. It holds one
  `i8` per trit rather than reading planes, because its job is to be evidently
  correct rather than fast.
- **`trit-core` + `trit-cpu`** — the runtime. Zero-copy, vectorized, threaded.
- **`rtl/tritcore`** — the hardware, under Verilator.

Agreement is demanded **exactly** on the integer path. Lane order, thread count
and the RTL's beat-serial accumulation must all produce identical bits. Only the
f32 tail (attention, LM head) admits reassociation, and even there the
cross-implementation check measures cosine 1.000000 at every position on the real
checkpoint.

This is not ceremony. It has caught real defects that no eyeball would:

- A folded `w_scale * x_scale` constant that multiplied once where the reference
  multiplies twice. f32 multiplication is not associative, so the two rounded
  differently: invisible for four token positions, cosine 0.999624 by position 7,
  and a changed word by token 16.
- An oracle that fell back to the embedding matrix for a missing `lm_head` while
  the runtime refused to — meaning the two sides of the parity gate could have
  been comparing different models, in a way the parity test itself could never
  see, because it never got as far as running.

The frozen reference is [06-GATES.md](06-GATES.md). G1-G6 and G8 are
**invariants**: they may not move without an argued entry. G7 holds the resource
numbers the pivot exists to improve, and is expected to move.

A note on what counts as coverage: the aarch64 CI job spent its entire existence
failing at clippy before reaching a single assertion, while the test matrix
advertised ARM coverage. Two real defects were sitting in code that matrix
claimed to cover. **A job that cannot reach its assertions is not coverage.**

## 6. The numerics ladder

Three rungs, each a strictly more exact evaluation of the same model, selected
automatically to the best the architecture supports:

| Rung | What changes |
|---|---|
| `Reference` | Textbook. RMSNorm materialized, activations quantized per matvec. |
| `Folded` | The per-element `1/rms` divide leaves the datapath entirely. Absmax codes are invariant to a uniform positive scale, so the codes come from `x .* g` directly and the rms survives as one per-token scalar folded into the output scale. |
| `IntMlp` | The squared-ReLU stage never exists in f32. With `g = acc_g * S_g` and `u = acc_u * S_u`, `relu(g)^2 * u = t * K` for integer `t` (exact in i64) and uniform `K`. |

Both upper rungs were proved against the real checkpoint before being made the
default. Norm folding matters beyond speed: it removes the rsqrt and the divide
from the hardware datapath, which is a synthesis result as much as a numerics
one.

## 7. Threading

Threading splits **rows**, never a reduction, so results are bit-identical
regardless of thread count or scheduling.

The interesting part is that more threads made it *worse*, and the fix was a
measurement rather than an intuition. A decode step issues 210 ternary matvecs
whose largest is 4.4 MB; splitting each across the machine costs more in
fork/join than it saves. Before a threshold was introduced: 14.13 tok/s at one
thread against 2.40 tok/s at 32 — nearly 6x slower. With `PARALLEL_MIN_BYTES` set
above every per-layer tensor in this model class, the ternary matvecs run inline
and the parallelism that does pay goes to the LM head.

Batched prefill, or a model whose projections are an order of magnitude larger,
would want this revisited — again with a measurement.

**This is the project's largest open defect, and it is now quantified.** Against
bitnet.cpp on the identical checkpoint, Tritium is within 6% at one thread and
2.09x slower at eight, because bitnet.cpp scales 2.54x across that range and
Tritium scales 1.15x. Lowering the threshold does not help — the layer time goes
37.4 ms at one thread to 173 ms at eight, since each of 210 matvecs per token
pays its own fork/join. The fix is a cheaper synchronization primitive, not a
different threshold: ggml gets its scaling from a persistent pool with cheap
barriers. Until that exists, the threshold is load-bearing and must stay.

## 8. The silicon track

The RTL is not deprecated by the pivot and is not a museum piece. It is a
**backend**, exercised by CI on every push, and its bit-exactness is a frozen
invariant.

Proven today:

- `rtl/trit_matvec.sv` consumes `.trit` v1 beats directly — `w_pos`/`w_neg`, 64
  lanes, a combinational select-accumulate tree, i32 accumulator.
- Bit-exact against golden vectors including the `x = -128` extremes set, which
  any implementation that negates in the i8 domain fails.
- Yosys generic synthesis asserts **zero `$mul` and zero `$macc` cells** on every
  run. Baseline 33,659 cells at 64 lanes, of which ~4.1k DFF are the flattened
  activation memory (BRAM on any real part).
- The real 2B4T model decodes end-to-end through the Verilated core, byte-
  identical to the CPU path, at 24.4 s/token. That number is what says silicon is
  the next step, not a defect.

Not proven:

- No timing closure. The 64-term single-cycle reduction and 64 parallel
  activation reads are fine under Verilator and are not yet placed on a part.
- `MAX_COLS` is 8192, which every projection in this model class fits under. The
  wrappers now reject anything wider rather than wrapping the address silently.
- No board. The selection memo ([04b-BOARD-MEMO.md](04b-BOARD-MEMO.md))
  recommends a Kria KV260 on roofline-versus-synthesis grounds; the gate passed
  and the purchase decision is unmade.

## 9. Non-goals (v1)

- Training or QAT. We consume checkpoints, we do not make them.
- Batching, multi-user serving, speculative decoding.
- Long context beyond 2048; vision or multimodal.
- Beating a GPU on absolute tok/s. The metric is tokens per joule per dollar.

## 10. Open questions

- **The f32 head.** int8 is the obvious move and the roofline says ~2.2x. What it
  costs in quality is unmeasured, and the answer decides whether it ships as the
  default or as a flag.
- **Base-3 packing** (1.6 b/w, ~20% less traffic on the ternary body) against the
  unpack cost, on both CPU and fabric. Worth less than it looks while the head
  dominates bytes/token.
- **KV cache precision.** f32 and preallocated at 315 MB. int8 pages would cut
  resident footprint substantially and matter more as context grows.
- **ARM throughput.** The NEON kernels are correct on hardware and have never
  been timed. Until they are, no ARM performance claim exists.
- **Where the RTL's activation memory should live** once a board is real —
  on-chip banking versus DDR spill — which is a placement question the simulator
  cannot answer.
