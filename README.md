# tritium

**High-efficiency ternary inference engine for edge silicon. Run a useful
language model on cheap, low-power hardware — no cloud, no GPU, no multipliers.**

A *trit* is a ternary digit. Tritium is the hydrogen isotope that powers
self-luminous devices for decades without an external energy source. Same idea:
language models that glow on their own, offline, on hardware measured in tens of
dollars and single-digit watts.

## The thesis

4-bit and 8-bit quantization on edge companion computers — drones, robots, edge
IPCs — bottlenecks on the memory bus and dissipates heat doing continuous
floating-point matrix multiplication. BitNet-style 1.58-bit models put every
weight in {-1, 0, +1}, which turns the inner loop from multiply-accumulate into
**select-accumulate**:

```
w == +1  ->  acc += a
w == -1  ->  acc -= a
w ==  0  ->  skip
```

No multiplier. Two bits per weight on the wire. Tritium is the runtime that
cashes that check on hardware people already own, and keeps a synthesizable RTL
path open for hardware they don't yet.

## What runs today

Measured on an AMD Ryzen 9 8945HX (Zen 4, AVX-512 VNNI) with BitNet b1.58 2B4T.
Every number here is reproducible with `tritd bench`; the frozen reference is
[docs/06-GATES.md](docs/06-GATES.md).

| | Before the pivot | Now |
|---|---|---|
| Decode | 0.16 tok/s | **14.89 tok/s** |
| Time to first token | ~33 s | **470 ms** |
| Peak RSS | 6.10 GiB | **1.80 GiB** |
| Achieved bandwidth | ~0.3 GB/s | **27.3 GB/s** (66% of the 41.1 GB/s this host sustains) |
| Quality vs the HF reference | mean logit cosine 0.9991, top-1 100% | unchanged |

Decoding at batch 1 touches every weight once per token, so throughput is bounded
by `tok/s ~= bandwidth / bytes_per_token`. For this model that is **521 MB of
ternary weights and 1313 MB of f32 embeddings** — the LM head, not the ternary
projections, is the thing to attack next.

## Architecture

| Crate | What it is | Status |
|---|---|---|
| `crates/tritc` | Converter: HF BitNet checkpoint → packed `.trit` v1. Folds norms, quantizes, verifies. | working |
| `crates/trit-core` | Format, transformer, KV cache, RoPE, sampler, tokenizer traits. No `unsafe` outside one mmap. | working |
| `crates/trit-cpu` | Bit-sliced SIMD kernels: AVX-512 VNNI / AVX-512BW / AVX2 / NEON / portable scalar. | x86 verified on hardware; aarch64 compile-verified only |
| `crates/tritd` | Host daemon and C runtime: `run`, `serve`, `bench`, `info`, plus a C ABI. | working |
| `crates/trit-rtl` | Hardware-in-the-loop backend over the Verilated core. | working |
| `crates/tritsim` | Independent golden reference. The oracle every other path is diffed against. | working |
| `rtl/` | Multiplier-free SystemVerilog `tritcore` + Verilator testbenches. | 64 lanes, simulation-first |
| `benches/` | Comparative harness vs llama.cpp / bitnet.cpp baselines. | not built yet |

## Quickstart

```sh
# 1. fetch the checkpoint (~5 GB)
./scripts/fetch_model.sh

# 2. convert to .trit v1, verifying the result
cargo run --release -p tritc -- convert \
    --input models/bitnet-2b4t --output models/bitnet-2b4t.trit --verify

# 3. generate
cargo run --release -p tritd -- run \
    --model models/bitnet-2b4t.trit \
    --prompt "The capital of France is" --steps 64

# 4. measure
cargo run --release -p tritd -- bench --model models/bitnet-2b4t.trit --threads 4
```

`tritd serve --model models/bitnet-2b4t.trit` exposes `POST /v1/completions`
(OpenAI-subset, with SSE streaming), `GET /v1/models`, `/healthz` and `/metrics`
on loopback.

## Hardware support

| Target | Kernel | Primitive | Status |
|---|---|---|---|
| x86-64 + AVX-512 VNNI | `avx512vnni` | `vpdpbusd` under a `k` mask | verified, 32.5x scalar |
| x86-64 + AVX-512BW | `avx512bw` | masked `maddubs` | verified, 23.9x scalar |
| x86-64 + AVX2 | `avx2` | byte-spread mask + `maddubs` | verified, 13.9x scalar |
| ARM64 + `dotprod` | `neon-dotprod` | `sdot` against ones | compiles; **not yet run on hardware** |
| ARM64 baseline | `neon` | `vtstq` mask + `vpadalq` | compiles; **not yet run on hardware** |
| anything | `scalar` | set-bit iteration, skips the 42% zeros | verified |
| RTL under Verilator | `trit-rtl` | 64-lane adder tree, no multipliers | verified, 24.4 s/token |

The kernel is chosen at runtime by feature detection. `--kernel <name>` pins one
and **errors** if this CPU cannot run it, so a benchmark can never silently
measure a different path than the one it names.

### About `popcount`

Popcount is the natural primitive when *activations* are also 1-bit. Here weights
are 1.58-bit but activations are int8, so a popcount of a weight mask only counts
terms — it cannot recover the sum. The production kernels mask activation bytes
and accumulate the two planes separately.

The bit-serial popcount formulation is implemented anyway, in
`crates/trit-cpu/src/bitserial.rs` behind a feature flag, because it is the one
that maps directly onto the RTL adder tree. It measures **22x slower** than the
AVX-512 path at int8 activations. It stays as a third independent implementation
for the differential tests, and because it becomes the right kernel if
activations ever drop below 8 bits. Where popcount does pay today is the scalar
fallback's set-bit iteration and the zero-fraction statistics.

## Correctness

Three implementations of the same arithmetic must agree:

- **`tritsim`** — the oracle. Naive, scalar, deliberately obvious.
- **`trit-core` + `trit-cpu`** — the runtime. Zero-copy, vectorized, threaded.
- **`rtl/tritcore`** — the hardware, under Verilator.

The ternary accumulators are `i32`, and integer addition is exact and
order-independent, so agreement between them is demanded **exactly** — not within
a tolerance. Lane order, thread count and the RTL's beat-serial accumulation all
have to produce identical bits. Only the f32 tail (attention, LM head) admits
reassociation, and the cross-implementation check still measures cosine 1.000000
at every position on the real checkpoint.

That check earns its keep. It caught a folded `w_scale * x_scale` constant that
rounded differently from the reference's two separate multiplications — invisible
for four positions, and by position 7 enough to change the generated text.

```sh
cargo test --workspace --exclude trit-rtl   # unit, differential, cross-implementation
make -C rtl test                            # golden vectors, bit-exact
make -C rtl synth                           # asserts zero multiplier cells
cargo test -p tritsim --release --test cross_implementation -- --ignored  # real checkpoint
```

## The `.trit` format

Ternary weights are stored as **bit planes**: each 16-byte beat carries 64
columns of one row as `{u64 pos, u64 neg}`. Two bits per weight, same as an
interleaved code, but both masks for a column block arrive together — which is
what the SIMD kernel wants and what the RTL core consumes directly. One beat is
exactly one hardware cycle's weight input, so a memory-mapped `.trit` *is* the
beat stream; nothing between the page cache and the accumulator rewrites a byte.

Full specification, including both plane invariants and the v0 migration path:
[docs/01-TRIT-FORMAT.md](docs/01-TRIT-FORMAT.md).

## Documentation

1. [docs/01-TRIT-FORMAT.md](docs/01-TRIT-FORMAT.md) — the `.trit` v1 specification
2. [docs/01-ARCHITECTURE.md](docs/01-ARCHITECTURE.md) — system design and the honest bandwidth math
3. [docs/02-ROADMAP.md](docs/02-ROADMAP.md) — milestone plan
4. [docs/03-RESEARCH.md](docs/03-RESEARCH.md) — ternary LLM background and reading list
5. [docs/04-BENCHMARKS.md](docs/04-BENCHMARKS.md) — measurement methodology and targets
6. [docs/06-GATES.md](docs/06-GATES.md) — the frozen regression reference
7. [docs/05-POSITIONING.md](docs/05-POSITIONING.md) — market wedge and build-in-public plan

## Honest limits

- Batch size 1. No batched prefill, no continuous batching.
- Context capped at 2048; the KV cache is f32 and preallocated (315 MB).
- Verified on x86-64 only. The NEON kernels compile and are written against the
  same contract, but have not been executed — no aarch64 hardware or emulator was
  available. No ARM performance number is claimed.
- No FPGA silicon yet. The RTL is simulation-first: the 64-term single-cycle
  reduction and 64 parallel activation reads are fine under Verilator and are not
  yet timing-closed on a board.
- `benches/` does not exist, so there is no llama.cpp comparison here yet. The
  numbers above are Tritium against its own past, plus a measured roofline.
- Energy per token is reported only where a real counter exists. It is never
  estimated.

## Lineage

Direct continuation of [ternoise](https://github.com/jackthepunished/ternoise) —
ternary compute on FPGA, built in public. The denoiser proved the primitive;
tritium scales the same primitive (ternary MAC → add/sub tree) to transformer
inference, and now runs it on commodity silicon as well.

## Ground rules

- Every performance claim ships with a reproducible benchmark script.
- `tritsim` is the source of truth; the CPU runtime and the RTL are both wrong
  until they match it.
- Build in public. No emojis in project communications.
