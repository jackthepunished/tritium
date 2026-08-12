#!/usr/bin/env python3
"""Dump per-position logits from the HF reference model for comparison with tritsim.

Usage: python3 scripts/dump_logits.py models/bitnet-2b4t "The capital of France is" logits.json
Requires: pip install torch transformers
"""
import json
import sys

import torch
from transformers import AutoModelForCausalLM, AutoTokenizer

model_dir, prompt, out_path = sys.argv[1], sys.argv[2], sys.argv[3]
tok = AutoTokenizer.from_pretrained(model_dir)
model = AutoModelForCausalLM.from_pretrained(
    model_dir, torch_dtype=torch.float32, trust_remote_code=True
)
model.eval()
ids = tok(prompt, return_tensors="pt").input_ids
with torch.no_grad():
    logits = model(ids).logits[0]  # [n_pos, vocab]
json.dump(
    {"prompt_ids": ids[0].tolist(), "logits": logits.tolist()},
    open(out_path, "w"),
)
print(f"wrote {out_path}: {logits.shape[0]} positions x {logits.shape[1]} vocab")
