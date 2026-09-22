# Regression gates

The frozen reference for the hardware-agnostic pivot. Every number here was
measured on the pre-pivot tree at commit `c3351d1` (v0 `.trit`, scalar kernel,
single-threaded) before any pivot code landed.

**Rule: nothing in the pivot may change a number in this file without an
explicit, argued entry in the "Sanctioned changes" section below.**

## Environment of record

| | |
|---|---|
| Host | AMD Ryzen 9 8945HX (Zen 4), 32 threads |
| ISA | `avx2 avx512f avx512bw avx512dq avx512vl avx512_vnni avx512_vpopcntdq avx512_bf16 gfni` |
| rustc | 1.95.0 (59807616e 2026-04-14) |
| Verilator | 5.020 |
| Yosys | 0.33 |
| Model | `models/bitnet-2b4t.trit`, 1,836,137,811 bytes, `.trit` v0 |

Other hosts will differ on timing and RSS. The correctness gates are
host-independent and must hold everywhere.

## G1 — Unit and integration tests

```
cargo test --workspace
```

All pass. `trit-core` 12, `tritsim` lib 19, `tiny_model.rs` 6.

## G2 — Real-checkpoint logit agreement (the headline gate)

```
tritsim compare --model models/bitnet-2b4t.trit --dump logits.json
```

```
6 positions: mean cosine 0.9991, top1 match 100.0%
```

Last position, top-3 both sides:

```
ours   [(12366, 18.34601), (539, 14.126393), (264, 13.815544)]
theirs [(12366, 18.5),     (539, 14.25),     (264, 13.75)]
```

Single-position variant (`--dump bos_dump.json`): `1 positions: mean cosine
0.9977, top1 match 100.0%` — identical under `TRITSIM_INT_MLP=1`.

CLI acceptance thresholds (`crates/tritsim/src/main.rs`): cosine >= 0.98,
top-1 >= 0.90. The measured values sit far above them; the gate is the
**measured** value, not the threshold.

## G3 — Greedy text, byte-exact

```
tritsim run --model models/bitnet-2b4t.trit \
            --tokenizer models/bitnet-2b4t/tokenizer.json \
            --prompt "The capital of France is" --steps 16
```

```
 Paris. Paris is a city in the north of France, and it is the
```

Must reproduce byte-for-byte in `Reference`, `Folded` and `IntMlp` modes, and
on every backend (`cpu`, `rtl`) and every CPU kernel.

## G4 — RTL bit-exactness

```
make -C rtl lint          # clean
make -C rtl test          # all sets, exact i32 equality
```

```
SET vectors/exact_64x64:      64 rows OK
SET vectors/padded_5x100:      5 rows OK
SET vectors/extremes_4x128:    4 rows OK
SET vectors/zeros_3x64:        3 rows OK
SET vectors/wide_2x6912:       2 rows OK
SET vectors/model_k_proj_l0:   8 rows OK
SET <invalid-code>:            err raised OK
```

`extremes_4x128` feeds `x = -128` deliberately. Any CPU kernel that negates in
the i8 domain fails here. This set must never be weakened.

## G5 — Multiplier-free synthesis

```
make -C rtl synth
```

Yosys asserts zero `$mul` and zero `$macc` cells. Baseline cell count 33,659
(`MAX_COLS` chparam'd to 512; `$_DFFE_PP_` 4096 is the flattened activation
memory). Generic synthesis, no device target.

## G6 — Hardware-in-the-loop parity

```
cargo test -p tritsim --features rtl --release
```

`random_shapes_match_cpu_exactly` compares the Verilated core against the CPU
kernel on 8 shapes (1x64, 3x64, 2x100, 5x129, 7x640, 2x2560, 2x6912, 16x61) and
requires exact equality. All pass.

## G7 — Resource baseline (the numbers the pivot exists to move)

| Metric | Pre-pivot | Why |
|---|---|---|
| Peak RSS, real model | **6,401,024 KB (6.10 GiB)** | `read_trit` unpacks to `Vec<i8>` (~2.08 GB), embeddings f32 (~1.31 GB), `lm_head` cloned from `embed` when tied (another ~1.31 GB) |
| Model load + 1 forward | ~33 s | dominated by unpack + copy |
| Decode | **~6.3 s/token** | scalar, branch-per-weight, single-threaded |
| Bytes touched per token | 521 MB ternary + 1,313 MB `lm_head` f32 | `lm_head` is 2.5x the ternary weights |

These are targets to improve, not invariants. G1-G6 are invariants.

## G8 — trit-core against the tritsim oracle

Two independent implementations of the same model must agree.

```
cargo test -p tritsim --release --test cross_implementation              # tiny fixtures
cargo test -p tritsim --release --test cross_implementation -- --ignored # real checkpoint
```

On the real checkpoint, 8 positions, all three numerics rungs:
**cosine 1.000000 and identical top-1 at every position.**

This gate found a real bug. The production path had precomputed
`w_scale * x_scale` into a single constant and multiplied once, where the
reference multiplies twice: `acc as f32 * w_scale * x_scale`. f32 multiplication
is not associative, so `acc * (w * x)` and `(acc * w) * x` round differently.
The drift was invisible at short contexts (cosine 1.000000 for four positions),
grew with position to 0.999624 by position 7, and changed the 16th generated
token from "the" to "known". Ordering the multiplications to match the reference
restored cosine 1.000000 everywhere.

## Post-pivot results

The pivot's purpose was to move the G7 numbers without moving G1-G6. Measured
on the environment of record with `models/bitnet-2b4t.trit`:

| Metric | Pre-pivot | After the pivot | After A2+A3 |
|---|---|---|---|
| Decode | 0.16 tok/s | 14.89 tok/s | **27.90 tok/s** |
| Time to first token | ~33 s | 470 ms | **239 ms** |
| Peak RSS | 6,401,024 KB | 1,891,184 KB | **1,249,280 KB** |
| Bytes per token | 1,834,352,640 | 1,834,352,640 | **1,177,681,920** |
| Achieved bandwidth | ~0.3 GB/s | 27.3 GB/s | **32.9 GB/s** |

Decode is at 4 threads, which is where this host peaks.

The roofline fraction is a ratio against a probe run in the same invocation, and
that probe is itself noisy: across the five thread counts in the committed CSV
it read 48.25, 58.27, 57.24, 52.67 and 49.78 GB/s. At 4 threads, 32.85 against
57.24 is **57%**. Quote the pair, never the fraction alone, and never a fixed
ceiling for this host. The pivot's "66% of roofline" was computed against a
single-pass probe that under-reported, so it was optimistic on top of that.

Kernel throughput on a 2560x6912 tensor, planes resident in L3:

| Kernel | ms | GB/s | vs scalar |
|---|---|---|---|
| scalar | 9.10 | 0.5 | 1.00x |
| avx512vnni | 0.28 | 15.8 | **32.45x** |
| avx512bw | 0.38 | 11.6 | 23.91x |
| avx2 | 0.66 | 6.7 | 13.87x |
| bitserial (popcount) | 6.12 | 0.7 | 1.49x |

### Against other runtimes

Same host, same session, greedy, 32 tokens, median of three. bitnet.cpp runs the
identical checkpoint; llama.cpp runs Qwen2.5-3B Q4_K_M because mainline cannot
load `i2_s`, so that column is a different model at a different quality point.

| threads | tritium | llama.cpp | bitnet.cpp |
|---|---|---|---|
| 1  | **19.40** | 15.40 | 13.61 |
| 2  | **26.73** | 22.06 | 19.10 |
| 4  | **27.90** | 27.83 | 26.72 |
| 8  | 25.93 | 26.18 | **32.49** |
| 16 | 20.72 | 23.88 | **31.21** |

Ahead at 1, 2 and 4 threads, 1.43x bitnet.cpp per core. Still behind above
4 threads: our curve peaks and declines where bitnet.cpp keeps climbing to
32.49. That gap is the remaining scaling work.

**Superseded on our side by the A2 dispatch fix (2026-09-22).** Tritium now
measures 33.02/34.61/32.89 tok/s at 4/8/16 threads and peaks at eight rather
than four. The baseline columns above are **not** re-run -- llama.cpp and
bitnet.cpp are not configured in the WSL working setup -- so the two sides of
this table are no longer from the same session and it must not be read as a
current head-to-head. Re-running it with both baselines installed is the
outstanding measurement.

### Threading

Measured, not assumed, and the measurement changed twice.

Before a persistent pool existed, splitting each of the 210 ternary matvecs per
token across the machine cost more than it saved: 14.13 tok/s at one thread
against 2.40 at 32. `PARALLEL_MIN_BYTES` was set to 32 MiB to prevent it.

With the pool, dispatch is a sequence-counter bump and an unpark, and the
threshold drops to 64 KiB. Four mechanisms were measured in the real decode
loop; a dedicated work-stealing pool and a broadcast primitive both stayed
slower than not parallelising at all.

| Mechanism | 1t | 2t | 4t | 8t |
|---|---|---|---|---|
| serial | 37.6 | 37.6 | 36.9 | 36.7 |
| rayon, global pool | 36.0 | 65.0 | 108.8 | 174.0 |
| rayon, sized pool | 35.8 | 50.0 | 69.7 | 148.9 |
| rayon, broadcast | 35.9 | 47.2 | 57.5 | 89.8 |
| persistent spin barrier | 37.1 | **20.0** | **12.8** | 14.1 |

Layer time in ms. All produce byte-identical output.

G1-G6 are unchanged throughout. G2 still reports 0.9991/100%, G3 is still
byte-identical in every numerics mode on both implementations, and the real
checkpoint still decodes byte-identically through the Verilated RTL core --
24.4 s/token in simulation, via `tritd --backend rtl`.

## Sanctioned changes

Numbers here move only with an entry below, naming the phase, the reason, and
the before/after.

| Phase | What changed | Before | After | Reason |
|---|---|---|---|---|
| B | `.trit` v0 -> v1 bit planes | v0 codes | v1 planes | The migration is proven exact: `tritc upgrade` of the v0 file is byte-identical to a fresh convert. G2 and G3 unchanged. |
| B | Recorded zero fraction | 0.377 | **0.4219** | The old figure was hand-transcribed into checkpoint-notes.md and wrong. The byte-identity of the upgrade proves the trits themselves did not change. |
| C | RTL weight interface | `w_data[127:0]` | `w_pos`/`w_neg` | Synthesis is unchanged at 33,659 cells, still multiplier-free. |
| A2 | Automatic thread count | one per core | one per core, max 8 | Interleaved pairs give 1.17x at 2, 1.18x at 4, 1.17x at 8 and **0.94x at 16**. One per core is the wrong automatic answer on a large machine. Explicit `--threads` is unchanged. |
| A3 | Dense 2-D tensor storage | f32 | bf16 when the source is bf16 | The checkpoint is bf16 and `tritc` was widening it, so this restores the source precision rather than reducing it. Every value round-trips. Bytes per token 1,834,352,640 -> 1,177,681,920; G2 and G3 unchanged. Existing `.trit` files still load; the benefit needs a re-convert, and narrowing an existing file is not offered because a stored f32 does not record whether it was bf16 first. |
| A2b | Pool spin window | 4000 `pause` iterations | 200 us deadline (`TRIT_POOL_SPIN_US`) | The old window was sized against a belief that jobs arrive ~300 us apart; a decode profile measures the gap between consecutive matvecs at ~17 us, so the window expired inside decode's own gaps and most of the 210 dispatches per token paid a futex wake. An empty dispatch at four slots costs 1-4 us with workers spinning and 50-60 us once parked. Decode at 4/8/16 threads moves 26.47/24.34/20.30 to 33.42/34.79/33.34 tok/s in interleaved pairs, the peak moves from four threads to eight, and the decline above four is gone. G1-G3 and G8 unchanged; G3 verified byte-identical at 1/2/4/8/16 threads and in all three numerics rungs. Costs CPU above four threads (+14% at eight, +43% at sixteen) and saves it at or below four; energy unmeasured. Full report: [../benches/results/WSL-20260922-A2.md](../benches/results/WSL-20260922-A2.md). |
| A3 | Bandwidth probe | single pass | best of five | The single-pass probe scattered 45-50 GB/s where the machine sustains ~53, so `roofline_pct` flattered every result. Reported fraction drops from ~62% to 53-57% with no runtime change. |

The one change already anticipated: RoPE currently computes
`theta.powf(-2.0 * i / head_dim)` in **f32** (`crates/tritsim/src/math.rs`),
giving angle errors up to ~1e-3 rad at position 2048. Moving both
implementations onto a shared f64 `inv_freq`/`rope_angle` is expected to shift
G2 slightly — in the direction of the reference, since the reference is more
accurate. That gets its own isolated phase and its own row above.
