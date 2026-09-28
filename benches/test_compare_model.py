#!/usr/bin/env python3
"""Regression checks for measurement validity; no model required."""
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from compare_model import (ORDERS, collect_profiles, parse_bitnet, parse_trit,
                           prepare_output, require_parity, run, summarize)


class Measurements(unittest.TestCase):
    def test_phase_and_empty_dispatch_profiles_are_separate(self):
        output = ('ternary projections 60.00 60.0% 210.0 8.7\n'
                  'dense bf16 head 30.00 30.0% 1.0 21.9\n'
                  'other (serial scalar) 10.00 10.0% - -\n')
        with tempfile.TemporaryDirectory() as directory, patch('compare_model.run') as execute:
            out = Path(directory)
            execute.side_effect = [output, 'empty-dispatch diagnostics']
            collect_profiles(Path('profiler'), Path('model.trit'), 4, out)
            first, second = execute.call_args_list
            self.assertNotIn('--pool-probe', first.args[0])
            self.assertIn('--pool-probe', second.args[0])
            self.assertNotEqual(first.args[1], second.args[1])
            self.assertTrue((out / 'profile-t4.json').exists())
            execute.side_effect = None
            execute.return_value = 'empty-dispatch diagnostics'
            with self.assertRaises(ValueError):
                collect_profiles(Path('profiler'), Path('model.trit'), 4, out)

    def test_reusing_measurement_directory_fails_before_overwriting(self):
        with tempfile.TemporaryDirectory() as directory:
            out = Path(directory)
            (out / 'rustc.txt').write_text('environment evidence')
            prepare_output(out)
            (out / 'identities.json').write_text('previous identities')
            with self.assertRaises(ValueError):
                prepare_output(out)
            self.assertEqual((out / 'identities.json').read_text(), 'previous identities')

    def test_failed_process_keeps_diagnostics(self):
        with tempfile.TemporaryDirectory() as directory:
            stem = Path(directory) / 'failed'
            with self.assertRaises(subprocess.CalledProcessError):
                run([sys.executable, '-c',
                     'import sys; print("diagnostic", file=sys.stderr); sys.exit(3)'], stem)
            self.assertIn('diagnostic', stem.with_suffix('.stderr.txt').read_text())
            self.assertTrue(stem.with_suffix('.command.json').exists())

    def test_bitnet_rejects_wrong_workload_and_invalid_rates(self):
        row = dict(n_threads=2, n_prompt=0, n_gen=32, n_gpu_layers=0, avg_ts=12.5)
        self.assertEqual(parse_bitnet([row], 2)['tok_s'], 12.5)
        for key, value in [('n_threads', 4), ('n_prompt', 8), ('n_gen', 16),
                           ('n_gpu_layers', 1), ('avg_ts', float('nan')),
                           ('avg_ts', 0), ('avg_ts', True)]:
            with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                parse_bitnet([{**row, key: value}], 2)
        for rows in ([], [row, row], row):
            with self.assertRaises(ValueError):
                parse_bitnet(rows, 2)

    def test_trit_preserves_metric_meaning(self):
        row = dict(threads=2, prompts=4, backend='cpu/neon-dotprodx2', decoded_tokens=128,
                   prompt_tokens=36, kernel='neon-dotprod', decode_tok_per_s=15,
                   ttft_ms=200, peak_rss_mb=1250)
        parsed = parse_trit(row, 2)
        self.assertEqual(parsed['ttft_min_ms'], 200)
        self.assertEqual(parsed['peak_rss_mib'], 1250)
        for key, value in [('threads', 1), ('prompts', 1), ('backend', 'rtl'),
                           ('decoded_tokens', 0), ('decode_tok_per_s', float('inf'))]:
            with self.subTest(key=key), self.assertRaises(ValueError):
                parse_trit({**row, key: value}, 2)

    def test_output_parity_is_nonempty_and_exact(self):
        require_parity(' Paris.\n', ' Paris.\n')
        for a, b in [('', ''), ('\n', '\n'), ('Paris.', 'Paris!')]:
            with self.assertRaises(ValueError):
                require_parity(a, b)

    def test_balanced_orders_and_paired_not_ratio_of_medians(self):
        for variant in ORDERS[0]:
            self.assertEqual(sum(order.index(variant) for order in ORDERS), 4)
        self.assertEqual(sum(o.index('baseline') < o.index('candidate') for o in ORDERS), 2)
        samples = []
        for threads in (1, 2, 4):
            for repeat, (b, c) in enumerate([(1, 2), (2, 2), (10, 30), (20, 20)], 1):
                for variant, rate in [('baseline', b), ('candidate', c), ('bitnet', 7)]:
                    samples.append(dict(threads=threads, repeat=repeat, variant=variant,
                                        tok_s=rate, decoded_tokens=128, prompt_tokens=36,
                                        kernel='neon-dotprod'))
        self.assertEqual(summarize(samples)[0]['median_paired_speedup'], 1.5)
        samples[0]['decoded_tokens'] = 127
        with self.assertRaises(ValueError):
            summarize(samples)


if __name__ == '__main__':
    unittest.main()
