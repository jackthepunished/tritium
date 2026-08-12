# Architecture

## 1. The core insight

A BitNet b1.58 linear layer computes `y = W·x` where every weight is -1, 0, or +1 and activations are int8. The inner loop is therefore not multiply-accumulate but **select-accumulate**:

```
for each weight w, activation a:
    w == +1  ->  acc += a
    w == -1  ->  acc -= a
    w ==  0  ->  skip
```

On a GPU this still burns multiplier throughput and 2-bit weights get dequantized to int8/fp16 first. On an FPGA it maps to a mux and an adder tree — no DSP blocks required, so a modest fabric can host thousands of parallel lanes. The compute side of ternary inference is nearly free; **the real constraint is streaming weights from memory**, and the entire design is organized around that fact.

## 2. The honest bandwidth math (read this first)

Autoregressive decoding at batch size 1 touches every weight once per token. Tokens/sec is bounded by:

```
tok/s ≈ effective_memory_bandwidth / packed_model_size
```

Packed ternary weight sizes (2 bits/weight; base-3 packing of 5 trits/byte gives 1.6 bits/weight, ~20% smaller, at the cost of unpack logic):

| Model | Params | 2-bit packed | ~tok/s @ 1.6 GB/s (DDR3 board) | ~tok/s @ 12 GB/s (Kria/Zynq US+ class) |
|---|---|---|---|---|
| BitNet-ish 160M | 0.16B | 40 MB | ~40 | ~300 |
| BitNet-ish 700M | 0.7B | 175 MB | ~9 | ~68 |
| BitNet b1.58 2B4T | 2.4B | ~600 MB* | ~2.6 | ~20 |

*2B4T keeps embeddings/head in higher precision; real footprint is larger — measure, don't trust this table.

Consequences baked into the design:

- **Weight streaming, not weight caching.** Weights DMA through the core in a single pass per token; only activations, norms, and KV pages live on-chip/in fast memory.
- **The compute array must saturate the memory link,** nothing more. Sizing the adder-tree array beyond `bandwidth × lanes_per_byte` is wasted LUTs.
- **Prefill is the exception:** with N prompt tokens you amortize one weight pass over N tokens of work, so prefill runs compute-bound and fast. Time-to-first-token will look great; steady-state decode is the honest number.
- **Board choice is a bandwidth choice.** A $100 Artix/DDR3 board demos the architecture; a Kria KV260-class part (DDR4, ~12 GB/s usable) makes a 700M–2B model conversational. HBM parts are the ASIC-story teaser, not the near-term plan.

## 3. System overview

```
             +----------------------------- host (Rust) ------------------------------+
             |                                                                        |
  HF ckpt -> | tritc: quant-check -> pack trits -> layer manifest -> model.trit       |
             |                                                                        |
  prompt  -> | tritd: tokenizer -> scheduler -> sampler <-> KV cache manager          |
             |            |  DMA descriptors        ^ logits                          |
             +------------|--------------------------|----------------------------+---+
                          v                          |
             +--------- FPGA (tritcore) ------------------------------------------+
             |  weight DMA --> unpack (2b->trit) --> lane array (mux+adder trees) |
             |  activation SRAM <---------------------- accumulators              |
             |  RMSNorm / SiLU / RoPE fixed-point units                           |
             |  KV cache pages (BRAM hot set, DDR spill)                          |
             +--------------------------------------------------------------------+
```

### 3.1 `tritc` — model converter

- Input: Hugging Face BitNet checkpoint (start: `microsoft/bitnet-b1.58-2B-4T`; also retrain-small targets, see RESEARCH doc).
- Validates that weights are genuinely ternary post-quantization; records per-tensor scales.
- Packs weights 2 bits/trit (base-3 later), reorders into the exact stream order the lane array consumes — **memory layout is decided by the RTL, not the checkpoint**.
- Emits `model.trit`: header (arch hyperparams), per-layer manifest (offsets, scales), packed payload. Versioned format, checksummed.

### 3.2 `tritsim` — golden model

Bit-accurate Rust implementation of everything tritcore does, including fixed-point norm/activation approximations and accumulator widths. Runs the full model on CPU (slow is fine). Every RTL block is verified against tritsim vectors before integration; end-to-end token streams must match exactly. This is also the fallback demo if hardware slips.

### 3.3 `tritcore` — FPGA engine

- **Lane array:** N parallel lanes, each lane = trit decoder + add/sub mux + pipelined adder tree feeding a 24–32b accumulator. N sized to saturate memory bandwidth (e.g., 128 lanes × int8 at 200 MHz ≈ 25.6 GB/s of weight consumption capacity — far above a DDR3 link, so N=64 is plenty there).
- **Dataflow:** output-stationary per tile; activations for the current layer held in SRAM, weight tiles streamed. Layer-by-layer execution, no pipelining across layers in v1.
- **Nonlinear units:** RMSNorm needs no per-element division or rsqrt anywhere in the datapath — every norm in this architecture feeds an absmax int8 quantizer (BitLinear) or the host-side lm_head, and absmax codes are invariant to the uniform 1/rms factor. The codes come from `x .* g` directly; the rms survives only as one per-token scalar (sum of squares, one rsqrt) folded into the output scale. Proved bit-identical on the real checkpoint (see docs/03b-NUMERICS.md and `scaled_absmax_codes` in tritsim). Activation (relu2) via the quantized path; RoPE via precomputed sin/cos tables (its multipliers are outside the ternary datapath claim). All mirrored exactly in tritsim.
- **Activation quant:** absmax int8 per-token (as in BitNet), computed on-chip.
- **Integer MLP path (Phase 4):** the squared-ReLU stage never exists in f32. With g = acc_g * S_g and u = acc_u * S_u from the gate/up matvecs (uniform positive scales), relu(g)^2 * u = t * K for integer t = relu(acc_g)^2 * acc_u (|t| < 2^55, exact in i64) and uniform K = S_g^2 * S_u — and by norm folding the down projection's absmax codes come from t .* gain directly. Hardware cost: two streaming integer multiplies per element (inherent to relu2; ~2 DSPs), outside the multiplier-free ternary weight datapath. Proved against the real checkpoint (identical metrics and greedy text vs the folded f32 path).
- **KV cache:** int8 pages; hot window in BRAM, older pages in DDR. v1 caps context at 2K tokens.
- **Interface:** AXI4 on Zynq-class (PS handles tritd), or FT601/USB3 + DMA on pure-fabric boards.

### 3.4 `tritd` — host runtime

- Tokenizer (HF `tokenizers` crate), greedy/top-p sampling, chat template handling.
- Builds DMA descriptor chains per layer; overlaps next-tile DMA with current-tile compute (double buffering).
- Owns the profiler hooks: per-layer cycle counts, DMA stall counters, J/token via INA226 readout. This is where the inference-profiler OSS work folds in.

## 4. Phasing (matches ROADMAP)

- **P0 — tritsim + tritc:** correct ternary inference on CPU, `.trit` format frozen.
- **P1 — single-layer RTL:** matmul core passes tritsim vectors in simulation, then on the ternoise dev board.
- **P2 — end-to-end small model:** full decode loop on hardware, 160M–700M class model.
- **P3 — benchmark + demo:** tokens/s/W vs Pi 5 and Jetson Orin Nano running bitnet.cpp; fully-offline Q&A box demo.

## 5. Non-goals (v1)

- Training or QAT — we consume checkpoints, we don't make them.
- Batching, multi-user serving, speculative decoding.
- Long context (>2K), vision/multimodal.
- Beating a GPU on absolute tok/s. The metric is **tokens per joule per dollar**, not peak throughput.

## 6. Open questions

- Base-3 packing (1.6 b/w): worth ~20% bandwidth for the unpack LUT cost? Decide after P1 utilization numbers.
- 2B4T's non-ternary layers (embeddings, lm_head): run on PS/host in int8, or burn fabric? Start on host.
- Kria KV260 vs staying on the ternoise board for P2 — decision gate at end of week 3.
