"""`device_timing.py` 的单测（登记 `research/prompts/e171-r1-prereg.md` 修订 2 第 2.5、2.6 节）：逐行限时的算法、基线 `-timing` 产物的读法与包含自检、
`run` 补的那一行。变异表 `scripts/mutations/e171_three_oracle_collusion_after_the_oracle_fixes.tsv` 的 R9–R13 落在这几条上。"""

from __future__ import annotations

import os
import subprocess
import sys
import unittest

import device_timing

HERE = os.path.dirname(os.path.abspath(__file__))


def timing_lines(process_wall_seconds: list[str], stream_elapsed_nanoseconds: list[int]) -> list[str]:
    lines = [f"E7TIMING name=stream_time stream=s{position} elapsed_ns={elapsed} drive_ns=1 observe_ns=1 write_ns=1"
             for position, elapsed in enumerate(stream_elapsed_nanoseconds)]
    lines += [f"E7TIMING name=process_wall seconds={seconds} exit=0" for seconds in process_wall_seconds]
    return lines


class RowTimeLimitTests(unittest.TestCase):
    def test_a_short_baseline_keeps_the_1800_second_floor(self) -> None:
        self.assertEqual(device_timing.row_time_limit_seconds(100.0, False), 1800)

    def test_a_long_baseline_gets_three_times_its_wall_rounded_up(self) -> None:
        self.assertEqual(device_timing.row_time_limit_seconds(700.0, False), 2100)
        self.assertEqual(device_timing.row_time_limit_seconds(700.1, False), 2101)

    def test_the_rerun_of_a_timed_out_row_triples_the_limit_again(self) -> None:
        self.assertEqual(device_timing.row_time_limit_seconds(700.0, True), 6300)
        self.assertEqual(device_timing.row_time_limit_seconds(100.0, True), 5400)


class BaselineTimingTests(unittest.TestCase):
    def test_a_process_wall_that_contains_the_stream_times_is_accepted(self) -> None:
        timing = device_timing.read_baseline_timing(timing_lines(["3.500"], [1_000_000_000, 2_000_000_000]))
        self.assertEqual(timing, device_timing.BaselineTiming(3.5, 3_000_000_000, 2))
        self.assertTrue(timing.contains_the_stream_times())

    def test_a_process_wall_shorter_than_the_stream_times_is_refused(self) -> None:
        timing = device_timing.read_baseline_timing(timing_lines(["2.999"], [1_000_000_000, 2_000_000_000]))
        self.assertFalse(timing.contains_the_stream_times())

    def test_a_product_without_a_process_wall_or_with_two_is_refused(self) -> None:
        self.assertFalse(device_timing.read_baseline_timing(timing_lines([], [1])).contains_the_stream_times())
        self.assertFalse(device_timing.read_baseline_timing(timing_lines(["9.0", "9.0"], [1])).contains_the_stream_times())

    def test_a_product_without_any_stream_time_is_refused(self) -> None:
        self.assertFalse(device_timing.read_baseline_timing(timing_lines(["9.0"], [])).contains_the_stream_times())


class RunAndTimeTests(unittest.TestCase):
    def test_run_appends_one_process_wall_line_after_the_output_and_keeps_the_exit_code(self) -> None:
        completed = subprocess.run([sys.executable, "-B", os.path.join(HERE, "device_timing.py"), "run", "--", sys.executable, "-c",
                                    "print('E7RESULT name=done emitted=1'); raise SystemExit(3)"], capture_output=True, text=True, check=False)
        lines = completed.stdout.splitlines()
        self.assertEqual(completed.returncode, 3)
        self.assertEqual(lines[0], "E7RESULT name=done emitted=1")
        self.assertEqual(len(lines), 2)
        self.assertTrue(lines[1].startswith("E7TIMING name=process_wall seconds="))
        self.assertTrue(lines[1].endswith(" exit=3"))


if __name__ == "__main__":
    unittest.main()
