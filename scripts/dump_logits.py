#!/usr/bin/env python3
"""Dump per-position logits from the HF reference model for comparison with tritsim.

The bf16 master-weights repo carries quantization_config
{quant_method: bitnet, linear_class: autobitlinear, quantization_mode: online},
so transformers replaces the ternary projections with AutoBitLinear modules that
quantize online: per-tensor absmean ternary weights (WeightQuant) and per-token
absmax int8 activations (activation_quant) -- the same recipe tritsim implements.
No patching needed; we assert the quantized modules are actually present so a
silent fallback to unquantized linears cannot produce a bogus reference.

Usage: python3 scripts/dump_logits.py models/bitnet-2b4t "The capital of France is" logits.json
Requires: pip install torch transformers accelerate
"""
import json
import sys

import torch
from transformers import AutoModelForCausalLM, AutoTokenizer

model_dir, prompt, out_path = sys.argv[1], sys.argv[2], sys.argv[3]
tok = AutoTokenizer.from_pretrained(model_dir)
model = AutoModelForCausalLM.from_pretrained(model_dir, torch_dtype=torch.bfloat16)
model.eval()

n_bitlinear = sum(
    1 for _, m in model.named_modules() if type(m).__name__ == "AutoBitLinear"
)
print(f"AutoBitLinear modules: {n_bitlinear}")
assert n_bitlinear > 0, "no AutoBitLinear modules: reference is unquantized, dump would be bogus"

ids = tok(prompt, return_tensors="pt").input_ids
with torch.no_grad():
    logits = model(ids).logits[0].float()  # [n_pos, vocab]
json.dump(
    {"prompt_ids": ids[0].tolist(), "logits": logits.tolist()},
    open(out_path, "w"),
)
print(f"wrote {out_path}: {logits.shape[0]} positions x {logits.shape[1]} vocab")
