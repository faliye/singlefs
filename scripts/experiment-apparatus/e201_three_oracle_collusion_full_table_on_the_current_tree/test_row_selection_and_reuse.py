"""current_tree_row_driver.py 的选行与「不按键复用」的单测：不给 --rows、--first-slice 就是表里全部行（给了是指定范围，给空串是不跑表里的行，
full_table_run.py 的空变异与阳性对照行就是这样起的）；输出根里已有的同号行目录（做完了的、没做完的都算）一律清掉重做，不读回里面的结果；
限时乘 3 只给 --timed-out-before-rows 点名的行；不认 --skip-finished、--rerun-compile-failed-from、--rerun-of-timed-out-rows、--all（用法错退 2）。
每条用例自己建、自己删临时目录；不编、不跑装置。跑法：在本目录 `python3 -B -m unittest test_row_selection_and_reuse`。"""
from __future__ import annotations

import argparse
import contextlib
import io
import os
import tempfile
import unittest
from unittest import mock

import current_tree_row_driver as driver
from bad_implementation_table import SelectedRow


def selected_rows() -> list[SelectedRow]:
    return [SelectedRow(number, 1 if number <= 2 else 2, "core/mount.rs", number, f"name-{number}", "0" * 12) for number in (1, 2, 3)]


def write(path: str, text: str) -> None:
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)


def namespace(root: str, timed_out_before_rows: frozenset[int] = frozenset()) -> argparse.Namespace:
    return argparse.Namespace(streams="1,2", writable_mount_probe=False, positive_control=None, disable_oracle=None,
                              no_executor=False, keep_dumps=True, o3_control_bad_states=None, output_root=root,
                              work_root=os.path.join(root, "work"), baseline_process_wall_seconds=100.0, timed_out_before_rows=timed_out_before_rows)


class RowSelection(unittest.TestCase):
    def numbers(self, **overrides) -> list[int]:
        values = {"rows": None, "first_slice": False}
        values.update(overrides)
        return [row.number for row in driver.choose_rows(argparse.Namespace(**values), selected_rows())]

    def test_no_selection_is_every_row_of_the_table(self) -> None:
        self.assertEqual(self.numbers(), [1, 2, 3])

    def test_a_given_selection_is_a_narrowed_scope(self) -> None:
        self.assertEqual(self.numbers(rows="2,3"), [2, 3])
        self.assertEqual(self.numbers(first_slice=True), [1, 2])

    def test_an_empty_rows_value_selects_no_table_row(self) -> None:
        self.assertEqual(self.numbers(rows=""), [])

    def test_the_retired_row_arguments_are_not_recognised(self) -> None:
        environment = {name: value for name, value in os.environ.items() if name != driver.BREAK_VARIABLE}
        for argument in (["--skip-finished"], ["--rerun-compile-failed-from", "x"], ["--rerun-of-timed-out-rows"], ["--all"]):
            error = io.StringIO()
            with mock.patch.dict(os.environ, environment, clear=True), contextlib.redirect_stderr(error), self.assertRaises(SystemExit) as raised:
                driver.main(["--repo-root", "r", "--work-root", "w", "--output-root", "o", "--baseline-timing", "b", "--timing-output", "t", *argument])
            self.assertEqual(raised.exception.code, 2, argument)
            self.assertIn("unrecognized arguments", error.getvalue(), argument)


class NoReuseByKey(unittest.TestCase):
    def test_an_existing_row_directory_is_cleared_and_redone_without_reading_it(self) -> None:
        """做完了的（有 row-result.tsv，结局是 ok 或超时）与没做完的行目录都清掉，接着往下做（同步线道树那一步换成抛出）。"""
        class Stop(Exception):
            pass

        def stop_after_clearing(source_root, destination_root):
            raise Stop

        with tempfile.TemporaryDirectory() as root:
            arguments = namespace(root)
            write(os.path.join(root, "row-3", "row-result.tsv"), "status\tok\ndevice_exit\t0\n")
            write(os.path.join(root, "row-4", "row-result.tsv"), f"status\tok\ndevice_exit\t{driver.ROW_TIMEOUT_EXIT_CODE}\n")
            write(os.path.join(root, "row-5", "partial.txt"), "left over from an interrupted run")
            original = driver.sync_tree
            driver.sync_tree = stop_after_clearing
            try:
                for number in (3, 4, 5):
                    with self.assertRaises(Stop):
                        driver.run_row_on_the_lane(number, None, None, [], set(), set(), arguments, 0)
            finally:
                driver.sync_tree = original
            self.assertEqual([os.listdir(os.path.join(root, f"row-{number}")) for number in (3, 4, 5)], [[], [], []])

    def test_only_the_rows_the_caller_names_get_three_times_the_time_limit(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            arguments = namespace(root, frozenset({6}))
            named, other = driver.row_time_limit(arguments, 6), driver.row_time_limit(arguments, 7)
        self.assertEqual(named, (driver.device_timing.row_time_limit_seconds(100.0, True), True))
        self.assertEqual(other, (driver.device_timing.row_time_limit_seconds(100.0, False), False))
        self.assertGreater(named[0], other[0])

    def test_the_rows_timed_out_before_are_parsed_from_the_command_line(self) -> None:
        parser = driver.argument_parser()
        common = ["--repo-root", "r", "--work-root", "w", "--output-root", "o", "--baseline-timing", "b", "--timing-output", "t"]
        self.assertEqual(parser.parse_args(common).timed_out_before_rows, frozenset())
        self.assertEqual(parser.parse_args([*common, "--timed-out-before-rows", "3,7"]).timed_out_before_rows, frozenset({3, 7}))


if __name__ == "__main__":
    unittest.main()
