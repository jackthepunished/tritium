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

## Sanctioned changes

Numbers here move only with an entry below, naming the phase, the reason, and
the before/after.

| Phase | What changed | Before | After | Reason |
|---|---|---|---|---|
| _(none yet)_ | | | | |

The one change already anticipated: RoPE currently computes
`theta.powf(-2.0 * i / head_dim)` in **f32** (`crates/tritsim/src/math.rs`),
giving angle errors up to ~1e-3 rad at position 2048. Moving both
implementations onto a shared f64 `inv_freq`/`rope_angle` is expected to shift
G2 slightly — in the direction of the reference, since the reference is more
accurate. That gets its own isolated phase and its own row above.
