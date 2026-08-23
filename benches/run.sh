#!/usr/bin/env bash
# Comparative benchmark harness.
#
# Always measures Tritium. Compares against llama.cpp and bitnet.cpp when they
# are present, and records an explicit "unavailable" row when they are not --
# a missing baseline must be visible in the output, because a table that
# silently omits a comparison reads as though the comparison was favourable.
#
# Usage:
#   benches/run.sh --model models/bitnet-2b4t.trit [--tokens 64] [--out FILE]
#
# Optional baselines, discovered on PATH or via these variables:
#   LLAMA_CPP_BIN   path to llama-cli or llama-bench
#   LLAMA_GGUF      path to a .gguf of a comparable model
#   BITNET_CPP_BIN  path to bitnet.cpp's inference binary
set -uo pipefail

MODEL=""
TOKENS=64
THREADS=4
OUT=""
SUITE="benches/prompts/short.jsonl"

while [ $# -gt 0 ]; do
    case "$1" in
        --model)   MODEL="$2"; shift 2 ;;
        --tokens)  TOKENS="$2"; shift 2 ;;
        --threads) THREADS="$2"; shift 2 ;;
        --suite)   SUITE="$2"; shift 2 ;;
        --out)     OUT="$2"; shift 2 ;;
        -h|--help) sed -n '2,20p' "$0"; exit 0 ;;
        *) echo "unknown argument: $1" >&2; exit 2 ;;
    esac
done

[ -n "$MODEL" ] || { echo "--model is required" >&2; exit 2; }
[ -f "$MODEL" ] || { echo "no such model: $MODEL" >&2; exit 2; }

HOST="$(uname -n)"
DATE="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
[ -n "$OUT" ] || OUT="benches/results/${HOST}-$(date -u +%Y%m%d).csv"
mkdir -p "$(dirname "$OUT")"

HEADER='host,date,runtime,runtime_version,model,quant,bits_per_weight,suite,threads,kernel,backend,decode_tok_per_s,ttft_ms,weight_bytes_per_token,achieved_gbps,memcpy_gbps,roofline_pct,peak_rss_mb,joules_per_token,energy_source,status,note'
[ -s "$OUT" ] || echo "$HEADER" > "$OUT"

CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/.cache/tritium-target}"
export CARGO_TARGET_DIR
TRITD="$CARGO_TARGET_DIR/release/tritd"

echo "==> building tritd"
cargo build --release -p tritd >/dev/null || { echo "build failed" >&2; exit 1; }

echo "==> tritium"
JSON="$(mktemp)"
BENCH_ERR="$(mktemp)"
[ -f "$SUITE" ] || { echo "no such prompt suite: $SUITE" >&2; exit 2; }

if "$TRITD" bench --model "$MODEL" --suite "$SUITE" --tokens "$TOKENS" --threads "$THREADS" --json "$JSON" >/dev/null 2>"$BENCH_ERR"; then
    python3 - "$JSON" "$OUT" "$HOST" "$DATE" "$MODEL" <<'PY'
import json, sys, csv
report, out, host, date, model = sys.argv[1:6]
r = json.load(open(report))
def n(v): return "" if v is None else v
with open(out, "a", newline="") as f:
    csv.writer(f).writerow([
        # The suite comes from the report, not the shell: the CSV then names
        # the input tritd actually measured rather than one passed alongside it.
        host, date, "tritium", "0.1.0", model, "ternary-1.58", 2.0, r["suite"],
        r["threads"], r["kernel"], r["backend"],
        round(r["decode_tok_per_s"], 3), round(r["ttft_ms"], 1),
        r["weight_bytes_per_token"], round(r["achieved_gbps"], 2),
        n(r.get("memcpy_gbps")), n(r.get("roofline_pct")), n(r.get("peak_rss_mb")),
        n(r.get("joules_per_token")), r.get("energy_source", "none"), "ok", "",
    ])
PY
    echo "    recorded"
else
    REASON="$(tr -d '\n,' < "$BENCH_ERR" | tail -c 200)"
    echo "$HOST,$DATE,tritium,0.1.0,$MODEL,ternary-1.58,2.0,$SUITE,$THREADS,,,,,,,,,,,none,error,${REASON:-tritd bench failed}" >> "$OUT"
    echo "    FAILED: $REASON" >&2
fi
rm -f "$JSON" "$BENCH_ERR"

# --- baselines -------------------------------------------------------------
# Absent baselines are RECORDED as unavailable, with what was looked for, so the
# gap is visible in the CSV rather than inferred from a missing row.

record_unavailable() {
    printf '%s,%s,%s,,,%s,%s,%s,%s,,,,,,,,,,,none,unavailable,%s\n' \
        "$HOST" "$DATE" "$1" "$2" "$3" "$SUITE" "$THREADS" "$4" >> "$OUT"
    echo "    $1: unavailable ($4)"
}

echo "==> llama.cpp"
LLAMA="${LLAMA_CPP_BIN:-$(command -v llama-bench || command -v llama-cli || true)}"
if [ -z "$LLAMA" ]; then
    record_unavailable llama.cpp Q4_K_M 4.5 "no llama-bench or llama-cli on PATH; set LLAMA_CPP_BIN"
elif [ -z "${LLAMA_GGUF:-}" ]; then
    record_unavailable llama.cpp Q4_K_M 4.5 "found $LLAMA but LLAMA_GGUF is unset"
else
    echo "    running $LLAMA"
    RAW="$("$LLAMA" -m "$LLAMA_GGUF" -n "$TOKENS" -t "$THREADS" 2>&1 || true)"
    TPS="$(printf '%s' "$RAW" | grep -oE '[0-9]+\.[0-9]+ tokens per second' | tail -1 | grep -oE '^[0-9.]+' || true)"
    if [ -n "$TPS" ]; then
        printf '%s,%s,llama.cpp,,%s,Q4_K_M,4.5,%s,%s,,cpu,%s,,,,,,,,none,ok,\n' \
            "$HOST" "$DATE" "$LLAMA_GGUF" "$SUITE" "$THREADS" "$TPS" >> "$OUT"
        echo "    $TPS tok/s"
    else
        record_unavailable llama.cpp Q4_K_M 4.5 "ran but no tokens-per-second line was parsed"
    fi
fi

echo "==> bitnet.cpp"
BITNET="${BITNET_CPP_BIN:-$(command -v bitnet-cli || true)}"
if [ -z "$BITNET" ]; then
    record_unavailable bitnet.cpp I2_S 2.0 "no bitnet-cli on PATH; set BITNET_CPP_BIN"
else
    echo "    found $BITNET but no adapter is wired up yet"
    record_unavailable bitnet.cpp I2_S 2.0 "binary present, adapter not implemented"
fi

echo
echo "wrote $OUT"
echo "render with: python3 benches/report.py $OUT"
