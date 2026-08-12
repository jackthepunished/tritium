# Research notes — ternary LLMs and why the hardware is the missing piece

## 1. The lineage in five steps

1. **BinaryConnect / BNN (2015–2016)** — binary weights work for small nets; accuracy collapse at scale.
2. **TWN / TTQ (2016–2017)** — ternary {-1, 0, +1} recovers much of the loss; the zero state matters (it's learned sparsity).
3. **BitNet (Oct 2023, Microsoft)** — 1-bit transformer trained from scratch (QAT, not post-training quantization); scaling laws hold.
4. **BitNet b1.58 (Feb 2024)** — the breakout: ternary weights ("1.58 bits" = log2(3)), int8 activations, matches fp16 LLaMA quality at 3B+ params while cutting memory and energy. Key claim: *a new scaling law* — you buy quality with params, not precision.
5. **BitNet b1.58 2B4T (Apr 2025)** — first open-weight, production-quality native ternary model: 2.4B params, 4T training tokens, competitive with Qwen2.5/LLaMA-class peers of similar size. Runs today via `bitnet.cpp`.

Related threads worth tracking: T-MAC (LUT-based low-bit matmul on CPU), Falcon-E / BitNet variants from other labs, "matmul-free LM" (Zhu et al. 2024 — ternary + no attention matmuls, includes an FPGA appendix), and any QAT recipe that produces good sub-1B ternary checkpoints (our speed-demo tier).

## 2. Why post-training quantization is not the same thing

GPTQ/AWQ-style 4-bit PTQ compresses an fp16 model and still dequantizes to run on multiply hardware. Native ternary models are *trained* in the constraint, so:

- there is no dequant step to hide — the math is genuinely add/sub/skip;
- quality at 2 bits is viable (PTQ at 2 bits generally is not);
- the model IS the packed weights — 10x smaller artifact than fp16.

This distinction is the whole company. PTQ improvements help GPUs; native ternary helps whoever builds select-accumulate hardware.

## 3. Why CPUs/GPUs waste the format

- `bitnet.cpp` on CPU: real speedups (claimed 2–6x vs llama.cpp fp16) but it works by packing trits into LUT lookups on wide SIMD — clever, still fighting an ISA built for multiplies, still bound by DRAM.
- GPUs: tensor cores want int8/fp8 at minimum; 2-bit weights get expanded, and batch-1 decode leaves them memory-bound and power-hungry anyway.
- The theoretical prize (BitNet paper's own energy analysis): int8 add ≈ 0.03 pJ vs fp16 mul ≈ 1.1 pJ (45nm figures) — roughly **30x energy advantage on the dominant op**, unclaimable without hardware whose datapath is add/sub-native.

FPGA now, ASIC later, is the only path that actually collects.

## 4. What we must verify ourselves (assumptions to kill early)

- [ ] 2B4T weights are cleanly ternary per-tensor with per-tensor (not per-group) scales — affects `.trit` format. (Week 1)
- [ ] int8 absmax activation quant reproduces reference logits closely enough for stable long generations. (Week 1)
- [ ] Fixed-point RMSNorm/SiLU approximations don't degrade output quality perceptibly. (Week 2, tritsim A/B)
- [ ] A sub-1B ternary checkpoint exists with usable quality, or we accept the two-model demo strategy. (Week 1 survey)
- [ ] Real achievable DDR bandwidth on the ternoise board under streaming load. (Week 3 — this number rules the roadmap)

## 5. Prior FPGA art (know it, cite it, beat it on completeness)

Academic ternary/binary accelerators exist (FINN lineage from AMD/Xilinx research, various BNN CNN accelerators, the matmul-free LM FPGA prototype). Pattern in all of them: CNN-era, or single-kernel proofs, or no end-to-end LLM with tokenizer/KV/sampling actually shipping tokens. Nobody has shipped a **complete, reproducible, open ternary-LLM-on-FPGA stack with honest J/token numbers against commodity edge baselines**. That completeness is the differentiator, and it is exactly a systems-engineering problem, not a research problem.

## 6. Reading list

Papers (read in this order):
1. Wang et al., *BitNet: Scaling 1-bit Transformers for LLMs*, arXiv:2310.11453
2. Ma et al., *The Era of 1-bit LLMs: All LLMs are in 1.58 Bits*, arXiv:2402.17764
3. Ma et al., *BitNet b1.58 2B4T Technical Report*, arXiv:2504.12285
4. Zhu et al., *Scalable MatMul-free Language Modeling*, arXiv:2406.02528 (FPGA section)
5. Wei et al., *T-MAC: CPU Renaissance via Table Lookup*, arXiv:2407.00088
6. Umuroglu et al., *FINN: A Framework for Fast, Scalable Binarized Neural Network Inference*, FPGA'17

Code:
- github.com/microsoft/BitNet (`bitnet.cpp`) — reference kernels + the CPU baseline we benchmark against
- huggingface.co/microsoft/bitnet-b1.58-2B-4T — primary checkpoint
- github.com/Xilinx/finn — dataflow-FPGA compiler patterns worth stealing

Verify all links/models still current at week 1 — this list was compiled 2026-08-12.
