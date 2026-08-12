#!/usr/bin/env python3
"""Debug variant of dump_logits.py: ternary weights, NO activation quantization.

Used with tritsim's TRITSIM_NO_ACTQUANT=1 to isolate implementation bugs from
quantization-boundary chaos: if both sides agree in this mode but diverge with
activation quant on, the divergence is int8 boundary sensitivity (bf16 vs f32
rounding flipping quantization levels), not a math bug.

Usage: python3 scripts/dump_logits_noactquant.py models/bitnet-2b4t "prompt" out.json
"""
import json
import sys

import torch
import torch.nn.functional as F
from transformers import AutoModelForCausalLM, AutoTokenizer

model_dir, prompt, out_path = sys.argv[1], sys.argv[2], sys.argv[3]
tok = AutoTokenizer.from_pretrained(model_dir)
model = AutoModelForCausalLM.from_pretrained(model_dir, torch_dtype=torch.bfloat16)
model.eval()


def noquant_forward(self, input):
    if self.rms_norm is not None:
        input = self.rms_norm(input)
    w = self.weight.float()
    ws = w.abs().mean().clamp(min=1e-5)
    wq = torch.clamp(torch.round(w / ws), -1, 1) * ws
    y = F.linear(input.float(), wq).to(input.dtype)
    if self.bias is not None:
        y += self.bias.view(1, -1).expand_as(y)
    return y


patched = 0
for _, mod in model.named_modules():
    if type(mod).__name__ == "AutoBitLinear":
        mod.forward = noquant_forward.__get__(mod)
        patched += 1
print(f"patched {patched} AutoBitLinear modules to no-actquant forward")
assert patched > 0

ids = tok(prompt, return_tensors="pt").input_ids
with torch.no_grad():
    logits = model(ids).logits[0].float()
json.dump({"prompt_ids": ids[0].tolist(), "logits": logits.tolist()}, open(out_path, "w"))
print(f"wrote {out_path}: {logits.shape[0]} positions x {logits.shape[1]} vocab")
