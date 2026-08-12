#!/usr/bin/env python3
"""Sub-layer trace of reference layer 0 for a single BOS token.

Prints output norms (and o_proj/down_proj input norms) for every projection
and norm in layer 0. Compare against tritsim's TRITSIM_TRACE=1 layer-0 detail.

Usage: python3 scripts/trace_layer0.py models/bitnet-2b4t
"""
import sys

import torch
from transformers import AutoModelForCausalLM, AutoTokenizer

model_dir = sys.argv[1]
tok = AutoTokenizer.from_pretrained(model_dir)
model = AutoModelForCausalLM.from_pretrained(model_dir, torch_dtype=torch.bfloat16)
model.eval()

layer = model.model.layers[0]
mods = {
    "input_layernorm": layer.input_layernorm,
    "q_proj": layer.self_attn.q_proj,
    "k_proj": layer.self_attn.k_proj,
    "v_proj": layer.self_attn.v_proj,
    "o_proj": layer.self_attn.o_proj,
    "self_attn": layer.self_attn,
    "post_attention_layernorm": layer.post_attention_layernorm,
    "gate_proj": layer.mlp.gate_proj,
    "up_proj": layer.mlp.up_proj,
    "down_proj": layer.mlp.down_proj,
    "mlp": layer.mlp,
}


def show(name):
    def hook(mod, args, out):
        t = out[0] if isinstance(out, tuple) else out
        v = t.reshape(-1).float()
        i = args[0].reshape(-1).float() if args and torch.is_tensor(args[0]) else None
        pre = f" in_norm {float(i.norm()):10.4f}" if i is not None else ""
        print(f"{name:26}{pre} out_norm {float(v.norm()):10.4f} first4 {[round(float(x), 4) for x in v[:4]]}")

    return hook


for n, m in mods.items():
    m.register_forward_hook(show(n))

ids = torch.tensor([[tok.bos_token_id]])
with torch.no_grad():
    model(ids)
