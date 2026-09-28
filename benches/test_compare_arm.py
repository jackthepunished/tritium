"""Regression checks for the native comparison report parser; no ARM required."""
import unittest
from compare_arm import parse

REPORT = '''requested threads: 1, pool threads: 1
kernel ms GB/s vs scalar
scalar 9.88 0.4 1.00x
neon-dotprod 2.35 1.9 4.21x
neon 2.50 1.8 3.96x
threads ms GB/s
1 2.36 1.9
Streaming pass: 113 MB
threads ms GB/s implied tok/s
1 60.27 1.9 1.6
Dense pass: 16384x2560
precision threads ms GB/s
f32 1 6.79 24.7
bf16 1 4.51 18.6
'''


class ReportTests(unittest.TestCase):
    def test_distinguishes_kernel_cached_streaming_and_dense_times(self):
        self.assertEqual(parse(REPORT), {
            'kernel/scalar': 9.88, 'kernel/neon-dotprod': 2.35,
            'kernel/neon': 2.50, 'cached': 2.36, 'streaming': 60.27,
            'dense/f32': 6.79, 'dense/bf16': 4.51})

    def test_rejects_truncated_report(self):
        with self.assertRaises(ValueError):
            parse(REPORT[:REPORT.index('Dense pass:')])

    def test_rejects_invalid_times(self):
        for value in ('nan', 'inf', '0', '-1', 'bad'):
            with self.subTest(value=value), self.assertRaises(ValueError):
                parse(REPORT.replace('bf16 1 4.51', f'bf16 1 {value}'))


if __name__ == '__main__':
    unittest.main()
