#!/usr/bin/env bash
set -euo pipefail
# Downloads the bf16 master weights for BitNet b1.58 2B4T (~5 GB).
# Requires: pip install "huggingface_hub[cli]"
MODEL_DIR="${1:-models/bitnet-2b4t}"
# Pinned so future repo updates cannot silently change conversion/validation
# results. Bump deliberately and re-run the compare gates when you do.
REVISION="276681394656abdadb8e80e5b2c3db5e5d7fcaff"
hf download microsoft/bitnet-b1.58-2B-4T-bf16 \
  --revision "$REVISION" \
  --include "*.safetensors" --include "*.safetensors.index.json" \
  --include "config.json" --include "tokenizer*" \
  --local-dir "$MODEL_DIR"
echo "downloaded to $MODEL_DIR"
