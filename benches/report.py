#!/usr/bin/env python3
"""Render a benchmark CSV as a markdown table.

Rows whose status is not "ok" are rendered explicitly as unavailable, with the
reason. They are never dropped: a table that silently omits a baseline reads as
though the comparison was run and was favourable.
"""
import csv
import sys
from collections import defaultdict


def fmt(v, nd=2, suffix=""):
    if v in (None, "", "None"):
        return "--"
    try:
        return f"{float(v):.{nd}f}{suffix}"
    except ValueError:
        return str(v)


def main(path):
    with open(path, newline="") as f:
        rows = list(csv.DictReader(f))
    if not rows:
        print("no rows")
        return

    by_host = defaultdict(list)
    for r in rows:
        by_host[r["host"]].append(r)

    for host, rs in by_host.items():
        print(f"## {host}\n")
        print(
            "| runtime | quant | bits/w | threads | tok/s | TTFT | bytes/token "
            "| GB/s | % roofline | peak RSS | J/token |"
        )
        print("|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|")
        # Thread count first, then runtime, so each thread count reads as one
        # comparable block. Scaling is the thing these rows exist to show.
        def _key(r):
            try:
                t = int(r.get("threads") or 0)
            except ValueError:
                t = 0
            return (t, r["runtime"])

        for r in sorted(rs, key=_key):
            if r["status"] != "ok":
                note = r.get("note", "") or r["status"]
                print(
                    f"| {r['runtime']} | {r['quant'] or '--'} | {r['bits_per_weight'] or '--'} "
                    f"| {r.get('threads') or '--'} | _not measured_ | | | | | | | <!-- {note} -->"
                )
                continue
            bpt = r["weight_bytes_per_token"]
            bpt = f"{int(bpt)/1e6:.0f} MB" if bpt else "--"
            print(
                f"| {r['runtime']} | {r['quant']} | {r['bits_per_weight']} "
                f"| {r.get('threads') or '--'} "
                f"| {fmt(r['decode_tok_per_s'])} | {fmt(r['ttft_ms'], 0, ' ms')} | {bpt} "
                f"| {fmt(r['achieved_gbps'], 1)} | {fmt(r['roofline_pct'], 0, '%')} "
                f"| {fmt(r['peak_rss_mb'], 0, ' MB')} | {fmt(r['joules_per_token'], 3)} |"
            )
        print()

        missing = [r for r in rs if r["status"] != "ok"]
        if missing:
            print("Not measured on this host:\n")
            for r in missing:
                print(f"- **{r['runtime']}** -- {r.get('note') or r['status']}")
            print()

        print(
            "Comparability note: llama.cpp `Q2_K` is about 2.6 bits/weight and `Q4_K_M`\n"
            "about 4.5, both post-training quantizations of a model that was not trained\n"
            "for them. Tritium's ternary weights are 2.0 bits/weight and\n"
            "quantization-aware trained. A tokens-per-second comparison across those is a\n"
            "comparison of different quality points, so read it next to the bits/weight\n"
            "column and a quality check, not on its own.\n"
        )


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print(f"usage: {sys.argv[0]} <results.csv>", file=sys.stderr)
        raise SystemExit(2)
    main(sys.argv[1])
