#!/usr/bin/env python3
"""Reproduce a pinned bitnet.cpp non-AVX2 I2_S arithmetic mismatch.

This is a source-branch correctness probe, not an ARM performance test or a
full-model quality evaluation. It extracts the upstream dot function unchanged,
selects its non-AVX2 branch, and applies the caller's activation-sum correction.
The JSON records pass/fail; a failed arithmetic check does not abort Tritium's
independent measurement workflow. Compiler/extraction errors do abort.
"""
import argparse
import hashlib
import json
import platform
import subprocess
import tempfile
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--llama-source', required=True, type=Path)
    parser.add_argument('--out', required=True, type=Path)
    parser.add_argument('--cc', default='cc')
    args = parser.parse_args()
    source = args.llama_source / 'ggml/src/ggml-cpu/quants.c'
    caller = args.llama_source / 'ggml/src/ggml-cpu/ggml-cpu.c'
    text = source.read_text()
    start = text.index('static void ggml_vec_dot_i2_i8_s_1x1(')
    end = text.index('\nvoid ggml_vec_dot_i2_i8_s(', start)
    if '(tmp[row] - asum) * post_scale' not in caller.read_text():
        raise ValueError('Caller changed; re-evaluate the source-branch audit')
    probe = '#include <stdint.h>\n#include <stddef.h>\n#include <stdio.h>\n'
    probe += text[start:end]
    probe += r'''
int main(void) {
    uint8_t w[16]; int8_t x[64];
    for (int i = 0; i < 64; ++i) x[i] = 127;
    for (int code = 0; code < 3; ++code) {
        for (int i = 0; i < 16; ++i) w[i] = (uint8_t)(code * 0x55);
        float raw = 0;
        ggml_vec_dot_i2_i8_s_1x1(64, &raw, 1, w, 64, x, 0, 1);
        // Uniform real activations +1: quantized to 127, act_scale=127.
        // Weight scale=1. Each packed code denotes a weight code-1.
        float actual = (raw - 64 * 127) / 127.0f;
        float expected = (float)(64 * (code - 1));
        printf("%d %g %g %g\n", code, raw, actual, expected);
    }
    return 0;
}
'''
    args.out.mkdir(parents=True, exist_ok=True)
    cfile = args.out / 'bitnet-i2s-probe.c'
    cfile.write_text(probe)
    with tempfile.TemporaryDirectory(prefix='bitnet-i2s-audit-') as directory:
        binary = Path(directory) / 'probe'
        command = [args.cc, '-std=c11', '-O2', '-U__AVX2__', str(cfile), '-o', str(binary)]
        build = subprocess.run(command, capture_output=True, text=True)
        (args.out / 'bitnet-i2s-compile.txt').write_text(build.stdout + build.stderr)
        build.check_returncode()
        result = subprocess.run([str(binary)], check=True, capture_output=True, text=True)
    cases = []
    for row in result.stdout.splitlines():
        code, raw, actual, expected = map(float, row.split())
        cases.append(dict(packed_code=int(code), raw_dot=raw, actual=actual,
                          expected=expected, matches=actual == expected))
    if len(cases) != 3:
        raise ValueError('Incomplete arithmetic probe')
    report = dict(status='passed' if all(c['matches'] for c in cases) else 'failed',
                  scope='non-AVX2 source-branch arithmetic, not full-model quality',
                  execution_host=platform.machine(), compiler_command=command,
                  source_path='ggml/src/ggml-cpu/quants.c',
                  caller_path='ggml/src/ggml-cpu/ggml-cpu.c',
                  source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),
                  caller_sha256=hashlib.sha256(caller.read_bytes()).hexdigest(), cases=cases)
    report['compiler_version'] = subprocess.run([args.cc, '--version'], check=True,
                                                capture_output=True, text=True).stdout
    output = json.dumps(report, indent=2) + '\n'
    (args.out / 'bitnet-i2s-validation.json').write_text(output)
    print(output)


if __name__ == '__main__':
    main()
