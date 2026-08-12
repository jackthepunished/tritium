#!/usr/bin/env python3
"""Per-layer hidden-state trace of the reference for divergence bisection.

Runs a single BOS token and prints the L2 norm + first 4 components of the
embedding output and every layer output. Compare against
TRITSIM_TRACE=1 tritsim run output.

Usage: python3 scripts/trace_layers.py models/bitnet-2b4t
"""
import sys

import torch
from transformers import AutoModelForCausalLM, AutoTokenizer

model_dir = sys.argv[1]
tok = AutoTokenizer.from_pretrained(model_dir)
model = AutoModelForCausalLM.from_pretrained(model_dir, torch_dtype=torch.bfloat16)
model.eval()

bos = tok.bos_token_id
print("bos id:", bos)
ids = torch.tensor([[bos]])
with torch.no_grad():
    out = model(ids, output_hidden_states=True)
for i, h in enumerate(out.hidden_states):
    v = h[0, 0].float()
    tag = "embed " if i == 0 else f"layer {i - 1:2}"
    print(f"{tag} norm {float(v.norm()):12.4f} first4 {[round(float(x), 4) for x in v[:4]]}")
logits = out.logits[0, 0].float()
print("logits norm", float(logits.norm()), "argmax", int(logits.argmax()))

if len(sys.argv) > 2:
    import json

    json.dump(
        {"prompt_ids": [bos], "logits": [logits.tolist()]},
        open(sys.argv[2], "w"),
    )
    print("wrote", sys.argv[2])
