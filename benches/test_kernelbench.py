"""CLI regressions; pass the built kernelbench executable as the first argument."""

import pathlib
import subprocess
import sys
import unittest


BINARY = pathlib.Path(sys.argv.pop(1)).resolve()


class KernelbenchTests(unittest.TestCase):
    def run_bench(self, *args):
        return subprocess.run(
            [str(BINARY), *args], capture_output=True, text=True, timeout=120
        )

    def test_each_invocation_measures_only_its_requested_thread_count(self):
        for threads in (2, 4):
            with self.subTest(threads=threads):
                result = self.run_bench("--threads", str(threads))
                self.assertEqual(result.returncode, 0, result.stderr)
                # This value must come from the actual pool, not just the CLI.
                self.assertIn(f"pool threads: {threads}", result.stdout)
                rows = [line.split() for line in result.stdout.splitlines()]
                ternary = [
                    int(row[0])
                    for row in rows
                    if len(row) in (3, 4) and row[0].isdigit()
                ]
                dense = [
                    int(row[1]) for row in rows if row and row[0] in ("f32", "bf16")
                ]
                self.assertEqual(ternary, [threads, threads])
                self.assertEqual(dense, [threads, threads])

    def test_invalid_thread_requests_are_rejected(self):
        for args in (
            ("--threads", "0"),
            ("--threads", "-1"),
            ("--threads", "many"),
            ("--threads",),
            ("--threads", "2", "--threads", "4"),
            ("--unknown",),
        ):
            with self.subTest(args=args):
                result = self.run_bench(*args)
                self.assertNotEqual(result.returncode, 0)

    def test_listing_kernels_does_not_run_measurements(self):
        result = self.run_bench("--list-kernels")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("scalar", result.stdout.split())
        self.assertEqual(len(result.stdout.splitlines()), 1)


if __name__ == "__main__":
    unittest.main()
