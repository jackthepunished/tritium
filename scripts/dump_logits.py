#!/usr/bin/env python3
"""Dump per-position logits from the HF reference model for comparison with tritsim.

IMPORTANT: transformers' native BitNet modeling runs the bf16 master weights
UNQUANTIZED. The deployed model (and tritsim) use the QAT forward: per-tensor
absmean ternary weights, per-token absmax int8 activations. This script patches
that quantization into every ternary projection so the reference matches the
model's real inference semantics. Verified against modeling_bitnet source:
sub-norms and relu2 (via ACT2FN) are already in the native module; only the
linear quantization is missing.

Usage: python3 scripts/dump_logits.py models/bitnet-2b4t "The capital of France is" logits.json
Requires: pip install torch transformers
"""
import json
import sys

import torch
import torch.nn.functional as F
from transformers import AutoModelForCausalLM, AutoTokenizer

TERNARY_SUFFIXES = (
    "q_proj", "k_proj", "v_proj", "o_proj", "gate_proj", "up_proj", "down_proj",
)


def quantized_forward(self, x):
    # Weight: per-tensor absmean scale, RoundClip to {-1, 0, +1} (BitNet b1.58).
    w = self.weight.float()
    ws = w.abs().mean() + 1e-8
    wq = torch.clamp(torch.round(w / ws), -1, 1) * ws
    # Activation: per-token absmax int8, matching tritsim's absmax_quantize.
    xf = x.float()
    xs = (xf.abs().amax(dim=-1, keepdim=True) / 127.0).clamp(min=1e-30)
    xq = torch.clamp(torch.round(xf / xs), -127, 127) * xs
    return F.linear(xq, wq).to(x.dtype)


def main():
    model_dir, prompt, out_path = sys.argv[1], sys.argv[2], sys.argv[3]
    tok = AutoTokenizer.from_pretrained(model_dir)
    model = AutoModelForCausalLM.from_pretrained(model_dir, torch_dtype=torch.bfloat16)
    model.eval()

    patched = 0
    for name, mod in model.named_modules():
        if isinstance(mod, torch.nn.Linear) and name.endswith(TERNARY_SUFFIXES):
            mod.forward = quantized_forward.__get__(mod, torch.nn.Linear)
            patched += 1
    print(f"patched QAT quantization into {patched} linear layers")
    assert patched > 0, "no layers patched: tensor naming changed, do not trust this dump"

    ids = tok(prompt, return_tensors="pt").input_ids
    with torch.no_grad():
        logits = model(ids).logits[0].float()  # [n_pos, vocab]
    json.dump(
        {"prompt_ids": ids[0].tolist(), "logits": logits.tolist()},
        open(out_path, "w"),
    )
    print(f"wrote {out_path}: {logits.shape[0]} positions x {logits.shape[1]} vocab")


if __name__ == "__main__":
    main()
