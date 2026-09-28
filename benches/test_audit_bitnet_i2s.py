#!/usr/bin/env python3
"""Subprocess failure regressions; no compiler or checkpoint required."""
import sys
import unittest

from audit_bitnet_i2s import run_step


class AuditProcesses(unittest.TestCase):
    def test_stalled_child_fails_with_stage_and_deadline(self):
        with self.assertRaisesRegex(RuntimeError, 'probe timed out after 0.05 seconds'):
            run_step([sys.executable, '-c', 'import time; time.sleep(10)'],
                     label='probe', timeout=0.05)

    def test_failed_child_retains_diagnostics_for_caller(self):
        result = run_step([sys.executable, '-c',
                           'import sys; print("compiler diagnostic", file=sys.stderr); sys.exit(3)'],
                          label='compiler', timeout=10)
        self.assertEqual(result.returncode, 3)
        self.assertIn('compiler diagnostic', result.stderr)


if __name__ == '__main__':
    unittest.main()
