#!/usr/bin/env python3
"""Compare prebuilt Tritium binaries and bitnet.cpp serially on one native host.

No builds/downloads run during measurement. Tritium uses short.jsonl, greedy,
32 tokens, one warmup and one measured pass (median of four prompt rates).
bitnet.cpp uses empty-context tg32, three repeats (mean). Cross-runtime ratios
are contextual comparisons, not identical-workload speedups. Every raw JSON,
stdout/stderr and command is retained; paired speedups only compare Tritium.
"""
import argparse
import hashlib
import json
import math
import platform
import statistics
import subprocess
from pathlib import Path


ORDERS = (
    ('baseline', 'candidate', 'bitnet'),
    ('bitnet', 'candidate', 'baseline'),
    ('candidate', 'baseline', 'bitnet'),
    ('bitnet', 'baseline', 'candidate'),
)


def positive(value):
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise ValueError(f'Not a number: {value!r}')
    if not math.isfinite(value) or value <= 0:
        raise ValueError(f'Invalid measurement: {value!r}')
    return value


def parse_trit(report, threads):
    if (report['threads'] != threads or report['prompts'] != 4
            or report['backend'] != 'cpu'
            or not 0 < report['decoded_tokens'] <= 128):
        raise ValueError(f'Unexpected Tritium workload: {report}')
    return dict(tok_s=positive(report['decode_tok_per_s']),
                ttft_min_ms=positive(report['ttft_ms']),
                peak_rss_mib=report['peak_rss_mb'],
                decoded_tokens=report['decoded_tokens'],
                prompt_tokens=report['prompt_tokens'], kernel=report['kernel'])


def parse_bitnet(rows, threads):
    if not isinstance(rows, list) or len(rows) != 1:
        raise ValueError('Expected exactly one bitnet.cpp tg32 row')
    r = rows[0]
    if (r['n_threads'], r['n_prompt'], r['n_gen'], r['n_gpu_layers']) != (threads, 0, 32, 0):
        raise ValueError(f'Unexpected bitnet.cpp workload: {r}')
    return dict(tok_s=positive(r['avg_ts']))


def require_parity(baseline, candidate):
    if not baseline.strip() or baseline != candidate:
        raise ValueError('Empty or different greedy continuations; inspect parity logs')


def sha256(path):
    with path.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def prewarm(path):
    with path.open('rb') as f:
        while f.read(8 * 1024 * 1024):
            pass


def run(command, stem):
    command = [str(v) for v in command]
    stem.with_suffix('.command.json').write_text(json.dumps(command) + '\n')
    # Write as the process runs, including diagnostics on failure or timeout.
    with stem.with_suffix('.stdout.txt').open('w') as stdout, \
            stem.with_suffix('.stderr.txt').open('w') as stderr:
        subprocess.run(command, check=True, stdout=stdout, stderr=stderr, timeout=900)
    return stem.with_suffix('.stdout.txt').read_text()


def summarize(samples):
    summary = []
    for threads in (1, 2, 4):
        selected = [s for s in samples if s['threads'] == threads]
        medians = {v: statistics.median(s['tok_s'] for s in selected if s['variant'] == v)
                   for v in ORDERS[0]}
        paired = []
        for repeat in range(1, 5):
            pair = {s['variant']: s for s in selected if s['repeat'] == repeat}
            for key in ('decoded_tokens', 'prompt_tokens', 'kernel'):
                if pair['baseline'][key] != pair['candidate'][key]:
                    raise ValueError(f'Mismatched paired workload: {pair}')
            paired.append(pair['candidate']['tok_s'] / pair['baseline']['tok_s'])
        summary.append(dict(threads=threads, median_tok_s=medians,
                            paired_speedups=paired,
                            median_paired_speedup=statistics.median(paired)))
    return summary


def main():
    p = argparse.ArgumentParser(description=__doc__)
    for name in ('baseline', 'candidate', 'bitnet', 'model', 'gguf', 'tokenizer', 'suite', 'profile', 'out'):
        p.add_argument('--' + name, required=True, type=Path)
    a = p.parse_args()
    if platform.machine() != 'aarch64':
        p.error('ARM report requires a native aarch64 host; no emulation timings')
    for name, value in vars(a).items():
        setattr(a, name, value.resolve())
    a.out.mkdir(parents=True, exist_ok=True)
    identities = {name: dict(path=str(path), sha256=sha256(path))
                  for name, path in vars(a).items() if name != 'out'}
    (a.out / 'identities.json').write_text(json.dumps(identities, indent=2) + '\n')
    prompts = [json.loads(line)['text'] for line in a.suite.read_text().splitlines() if line.strip()]
    if len(prompts) != 4:
        raise ValueError('Expected four short prompts')
    # Both versions execute each prompt at every measured width, fresh processes.
    for threads in (1, 2, 4):
        for index, prompt in enumerate(prompts):
            outputs = []
            for variant in ('baseline', 'candidate'):
                outputs.append(run([getattr(a, variant), 'run', '--model', a.model,
                                    '--tokenizer', a.tokenizer, '--threads', threads,
                                    '--prompt', prompt, '--steps', 16],
                                   a.out / f'parity-t{threads}-p{index}-{variant}'))
            require_parity(*outputs)
    (a.out / 'parity.json').write_text(json.dumps(dict(prompts=4, threads=[1, 2, 4],
                                                     max_tokens=16, matching=True)) + '\n')
    samples = []
    for threads in (1, 2, 4):
        for repeat, order in enumerate(ORDERS, 1):
            for variant in order:
                stem = a.out / f't{threads}-r{repeat}-{variant}'
                prewarm(a.gguf if variant == 'bitnet' else a.model)
                if variant == 'bitnet':
                    raw = run([a.bitnet, '-m', a.gguf, '-t', threads, '-p', 0,
                               '-n', 32, '-ngl', 0, '-r', 3, '-o', 'json'], stem)
                    metrics = parse_bitnet(json.loads(raw), threads)
                else:
                    report = stem.with_suffix('.json')
                    run([getattr(a, variant), 'bench', '--model', a.model,
                         '--tokenizer', a.tokenizer, '--suite', a.suite,
                         '--threads', threads, '--tokens', 32, '--runs', 1,
                         '--warmup', 1, '--no-memcpy-probe', '--json', report], stem)
                    metrics = parse_trit(json.loads(report.read_text()), threads)
                sample = dict(threads=threads, repeat=repeat, variant=variant, **metrics)
                samples.append(sample)
                (a.out / 'samples.json').write_text(json.dumps(samples, indent=2) + '\n')
                print(stem.name, metrics, flush=True)
    result = dict(samples=samples, summary=summarize(samples))
    (a.out / 'comparison.json').write_text(json.dumps(result, indent=2) + '\n')
    # Profile separately: phase wrappers add overhead, empty jobs are diagnostic
    # probes, not a directly subtractable estimate of decode synchronization.
    for threads in (1, 2, 4):
        run([a.profile, '--model', a.model, '--threads', threads, '--tokens', 32,
             '--runs', 1, '--pool-probe'], a.out / f'profile-t{threads}')
    print(json.dumps(result['summary'], indent=2), flush=True)


if __name__ == '__main__':
    main()
