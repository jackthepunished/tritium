#!/usr/bin/env bash
set -euo pipefail
# Downloads the bf16 master weights for BitNet b1.58 2B4T (~5 GB).
# Requires: pip install "huggingface_hub[cli]"
MODEL_DIR="${1:-models/bitnet-2b4t}"
hf download microsoft/bitnet-b1.58-2B-4T-bf16 \
  --include "*.safetensors" --include "*.safetensors.index.json" \
  --include "config.json" --include "tokenizer*" \
  --local-dir "$MODEL_DIR"
echo "downloaded to $MODEL_DIR"
