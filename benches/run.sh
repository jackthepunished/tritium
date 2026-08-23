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

# Both llama.cpp and bitnet.cpp ship `llama-bench`, whose table is the same
# shape in each. One parser serves both.
#
#   | model | size | params | backend | threads | test | t/s |
#
# The t/s cell is "27.90 ± 0.09"; take the mean. `tg<N>` is the token-generation
# row, which is the decode number -- `pp` rows measure prefill and are not
# comparable to a decode figure.
# One invocation. The t/s cell reads "27.90 +/- 0.09"; the separator is
# multibyte, so the first decimal number in the cell is taken rather than
# splitting on it.
llama_bench_once() {   # bin model threads tokens
    "$1" -m "$2" -t "$3" -n "$4" -p 0 -r 3 2>/dev/null \
        | awk -F'|' -v want="tg$4" '
            $0 ~ want {
                cell = $(NF-1)
                if (match(cell, /[0-9]+\.[0-9]+/)) print substr(cell, RSTART, RLENGTH)
            }' \
        | tail -1
}

# Median of BASELINE_RUNS invocations, per the methodology in
# docs/04-BENCHMARKS.md.
#
# Median rather than best, and it matters here: llama-bench on this host returns
# ~26.8 tok/s five times out of six and then a ~39 outlier, so a best-of rule
# would publish the outlier as the baseline's score. Tritium's own numbers vary
# by under 2% across invocations, so a best-of rule would also be asymmetric --
# it would flatter whichever runtime is noisier. Median is robust to both.
BASELINE_RUNS="${BASELINE_RUNS:-3}"
llama_bench_tps() {   # bin model threads tokens
    local vals=()
    for _ in $(seq 1 "$BASELINE_RUNS"); do
        local v
        v="$(llama_bench_once "$1" "$2" "$3" "$4")"
        [ -n "$v" ] && vals+=("$v")
    done
    [ ${#vals[@]} -eq 0 ] && return 0
    printf '%s\n' "${vals[@]}" | sort -g | awk '{a[NR]=$1} END {print (NR%2) ? a[(NR+1)/2] : (a[NR/2]+a[NR/2+1])/2}'
}

file_mb() { [ -f "$1" ] && echo $(( $(stat -c%s "$1") / 1000000 )) || echo ""; }

echo "==> llama.cpp"
LLAMA="${LLAMA_CPP_BIN:-$(command -v llama-bench || true)}"
if [ -z "$LLAMA" ]; then
    record_unavailable llama.cpp Q4_K_M 4.5 "no llama-bench on PATH; set LLAMA_CPP_BIN"
elif [ -z "${LLAMA_GGUF:-}" ]; then
    record_unavailable llama.cpp Q4_K_M 4.5 "found $LLAMA but LLAMA_GGUF is unset"
elif [ ! -f "${LLAMA_GGUF}" ]; then
    record_unavailable llama.cpp Q4_K_M 4.5 "LLAMA_GGUF does not exist: $LLAMA_GGUF"
else
    echo "    running $LLAMA"
    TPS="$(llama_bench_tps "$LLAMA" "$LLAMA_GGUF" "$THREADS" "$TOKENS")"
    if [ -n "$TPS" ]; then
        # A DIFFERENT model at a different quality point -- see the comparability
        # note in benches/README.md. The quant and bits/w columns carry that, and
        # the note names the file so nobody has to guess what was measured.
        printf '%s,%s,llama.cpp,,%s,Q4_K_M,4.5,%s,%s,,cpu,%s,,,,,,,,none,ok,%s MB file; different model and quality point\n' \
            "$HOST" "$DATE" "$(basename "$LLAMA_GGUF")" "$SUITE" "$THREADS" "$TPS" "$(file_mb "$LLAMA_GGUF")" >> "$OUT"
        echo "    $TPS tok/s"
    else
        record_unavailable llama.cpp Q4_K_M 4.5 "ran but no tg$TOKENS row was parsed"
    fi
fi

echo "==> bitnet.cpp"
BITNET="${BITNET_CPP_BIN:-$(command -v bitnet-cli || true)}"
if [ -z "$BITNET" ]; then
    record_unavailable bitnet.cpp I2_S 2.0 "no bitnet-cli on PATH; set BITNET_CPP_BIN"
elif [ -z "${BITNET_GGUF:-}" ]; then
    record_unavailable bitnet.cpp I2_S 2.0 "found $BITNET but BITNET_GGUF is unset"
elif [ ! -f "${BITNET_GGUF}" ]; then
    record_unavailable bitnet.cpp I2_S 2.0 "BITNET_GGUF does not exist: $BITNET_GGUF"
else
    echo "    running $BITNET"
    TPS="$(llama_bench_tps "$BITNET" "$BITNET_GGUF" "$THREADS" "$TOKENS")"
    if [ -n "$TPS" ]; then
        # The one true apples-to-apples row: same checkpoint, same host, same
        # thread count. Its embeddings are f16 where ours are f32, which is a
        # real difference in bytes/token and is noted rather than hidden.
        printf '%s,%s,bitnet.cpp,,%s,I2_S,2.0,%s,%s,,cpu,%s,,,,,,,,none,ok,%s MB file; same checkpoint; f16 token_embd\n' \
            "$HOST" "$DATE" "$(basename "$BITNET_GGUF")" "$SUITE" "$THREADS" "$TPS" "$(file_mb "$BITNET_GGUF")" >> "$OUT"
        echo "    $TPS tok/s"
    else
        record_unavailable bitnet.cpp I2_S 2.0 "ran but no tg$TOKENS row was parsed"
    fi
fi

echo
echo "wrote $OUT"
echo "render with: python3 benches/report.py $OUT"
