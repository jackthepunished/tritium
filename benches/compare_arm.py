#!/usr/bin/env python3
"""Run prebuilt kernelbench binaries serially, with alternating A/B order.

Use on native ARM only for ARM performance claims. Both builds must finish
before starting this script. Raw output and runner metadata accompany results.
"""
import argparse
import json
import math
import platform
import statistics
import subprocess
from pathlib import Path


def parse(output):
    metrics = {}
    section = None
    for line in output.splitlines():
        if line.startswith('kernel '):
            section = 'kernel'
        elif line.startswith('Streaming pass:'):
            section = 'streaming'
        elif line.startswith('Dense pass:'):
            section = 'dense'
        elif line.startswith('threads ') and section != 'streaming':
            section = 'cached'
        fields = line.split()
        if not fields:
            continue
        try:
            if section == 'kernel' and fields[0] in ('scalar', 'neon', 'neon-dotprod'):
                metrics['kernel/' + fields[0]] = float(fields[1])
            elif section in ('cached', 'streaming') and fields[0].isdigit():
                metrics[section] = float(fields[1])
            elif section == 'dense' and fields[0] in ('f32', 'bf16'):
                metrics['dense/' + fields[0]] = float(fields[2])
        except (ValueError, IndexError) as e:
            raise ValueError(f'Invalid measurement row: {line}') from e
    expected = {'kernel/scalar', 'kernel/neon', 'kernel/neon-dotprod',
                'cached', 'streaming', 'dense/f32', 'dense/bf16'}
    if set(metrics) != expected or any(not math.isfinite(v) or v <= 0 for v in metrics.values()):
        raise ValueError(f'Incomplete or invalid ARM measurements: {metrics}')
    return metrics


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('baseline', type=Path)
    p.add_argument('candidate', type=Path)
    p.add_argument('--out', type=Path, required=True)
    args = p.parse_args()
    if platform.machine() != 'aarch64':
        p.error('Run on a native aarch64 runner; do not report emulation timings')
    args.out.mkdir(parents=True, exist_ok=True)
    binaries = {v: getattr(args, v).resolve() for v in ('baseline', 'candidate')}
    samples = []
    for threads in (1, 2, 4):
        for repeat in range(4):
            order = ('baseline', 'candidate') if repeat % 2 == 0 else ('candidate', 'baseline')
            for variant in order:
                result = subprocess.run([str(binaries[variant]), '--threads', str(threads)],
                                        check=True, text=True, capture_output=True)
                name = f't{threads}-r{repeat + 1}-{variant}'
                (args.out / (name + '.txt')).write_text(result.stdout + result.stderr)
                sample = dict(threads=threads, repeat=repeat + 1, variant=variant,
                              ms=parse(result.stdout))
                samples.append(sample)
                print(name, sample['ms'], flush=True)
    summary = []
    for threads in (1, 2, 4):
        for metric in samples[0]['ms']:
            medians = {v: statistics.median(s['ms'][metric] for s in samples
                       if s['threads'] == threads and s['variant'] == v) for v in binaries}
            paired = []
            for repeat in range(1, 5):
                pair = {s['variant']: s['ms'][metric] for s in samples
                        if s['threads'] == threads and s['repeat'] == repeat}
                paired.append(pair['baseline'] / pair['candidate'])
            summary.append(dict(threads=threads, metric=metric, median_ms=medians,
                                paired_speedups=paired,
                                median_paired_speedup=statistics.median(paired)))
    (args.out / 'comparison.json').write_text(json.dumps(dict(samples=samples, summary=summary), indent=2) + '\n')
    print(json.dumps(summary, indent=2))


if __name__ == '__main__':
    main()
