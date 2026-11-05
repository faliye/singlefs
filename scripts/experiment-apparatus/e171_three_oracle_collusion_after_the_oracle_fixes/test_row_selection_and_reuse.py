"""row_driver.py 的选行与按键复用的单测：不给 --rows、--first-slice 就是表里全部行（给了是指定范围，给空串是不跑表里的行）；
行目录里同键、做完了的结果直接读回，键不同、没做完、编不过、超时的行目录先清掉重做；不认 --skip-finished、--rerun-compile-failed-from、
--rerun-of-timed-out-rows、--all（用法错退 2）；上一次超时的行重做时留标记文件，中途停下也认。每条用例自己建、自己删临时目录；不编、不跑装置。
跑法：在本目录 `python3 -B -m unittest test_row_selection_and_reuse`。"""
from __future__ import annotations

import argparse
import contextlib
import io
import os
import tempfile
import unittest

import row_driver
from bad_implementation_table import SelectedRow


def selected_rows() -> list[SelectedRow]:
    return [SelectedRow(number, 1 if number <= 2 else 2, "core/mount.rs", number, f"name-{number}", "0" * 12) for number in (1, 2, 3)]


def write(path: str, text: str) -> None:
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)


def namespace(root: str) -> argparse.Namespace:
    return argparse.Namespace(snapshot_digest="digest", streams="1,2,3", writable_mount_probe=False, positive_control=None, disable_oracle=None,
                              no_executor=False, keep_dumps=False, o3_control_bad_states=None, output_root=root,
                              work_root=os.path.join(root, "work"), baseline_process_wall_seconds=100.0)


class RowSelection(unittest.TestCase):
    def numbers(self, **overrides) -> list[int]:
        values = {"rows": None, "first_slice": False}
        values.update(overrides)
        return [row.number for row in row_driver.choose_rows(argparse.Namespace(**values), selected_rows())]

    def test_no_selection_is_every_row_of_the_table(self) -> None:
        self.assertEqual(self.numbers(), [1, 2, 3])

    def test_a_given_selection_is_a_narrowed_scope(self) -> None:
        self.assertEqual(self.numbers(rows="2,3"), [2, 3])
        self.assertEqual(self.numbers(first_slice=True), [1, 2])

    def test_an_empty_rows_value_selects_no_table_row(self) -> None:
        self.assertEqual(self.numbers(rows=""), [])

    def test_the_retired_row_arguments_are_not_recognised(self) -> None:
        for argument in (["--skip-finished"], ["--rerun-compile-failed-from", "x"], ["--rerun-of-timed-out-rows"], ["--all"]):
            error = io.StringIO()
            with contextlib.redirect_stderr(error), self.assertRaises(SystemExit) as raised:
                row_driver.main(["--repo-root", "r", "--work-root", "w", "--output-root", "o", "--baseline-timing", "b", "--timing-output", "t", *argument])
            self.assertEqual(raised.exception.code, 2, argument)
            self.assertIn("unrecognized arguments", error.getvalue(), argument)


class ReuseByKey(unittest.TestCase):
    def test_a_finished_row_directory_with_the_same_key_is_read_back(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            arguments = namespace(root)
            key = row_driver.reuse_key_of(arguments, 3, "-")
            write(os.path.join(root, "row-3", "row-result.tsv"), f"status\tok\ndevice_exit\t0\nreuse_key\t{key}\n")
            self.assertEqual(row_driver.run_row_on_the_lane(3, None, None, [], set(), set(), arguments, 0),
                             {"status": "ok", "device_exit": "0", "reuse_key": key})

    def test_a_row_directory_with_another_key_or_no_result_is_cleared_for_a_redo(self) -> None:
        class Stop(Exception):
            pass

        def stop_after_clearing(source_root, destination_root):
            raise Stop

        with tempfile.TemporaryDirectory() as root:
            arguments = namespace(root)
            write(os.path.join(root, "row-4", "row-result.tsv"), "status\tok\ndevice_exit\t0\nreuse_key\tsome-other-key\n")
            write(os.path.join(root, "row-5", "partial.txt"), "left over from an interrupted run")
            original = row_driver.sync_tree
            row_driver.sync_tree = stop_after_clearing
            try:
                for number in (4, 5):
                    with self.assertRaises(Stop):
                        row_driver.run_row_on_the_lane(number, None, None, [], set(), set(), arguments, 0)
            finally:
                row_driver.sync_tree = original
            self.assertEqual(os.listdir(os.path.join(root, "row-4")), [])
            self.assertEqual(os.listdir(os.path.join(root, "row-5")), [])

    def test_the_key_changes_with_what_changes_the_result(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            base = row_driver.reuse_key_of(namespace(root), 3, "name")
            other_digest = namespace(root)
            other_digest.snapshot_digest = "another"
            other_streams = namespace(root)
            other_streams.streams = "1,2"
            self.assertEqual(base, row_driver.reuse_key_of(namespace(root), 3, "name"))
            self.assertNotEqual(base, row_driver.reuse_key_of(other_digest, 3, "name"))
            self.assertNotEqual(base, row_driver.reuse_key_of(other_streams, 3, "name"))
            self.assertNotEqual(base, row_driver.reuse_key_of(namespace(root), 4, "name"))
            self.assertNotEqual(base, row_driver.reuse_key_of(namespace(root), 3, "another-name"))

    def test_a_timed_out_row_leaves_a_marker_that_survives_an_interrupted_redo(self) -> None:
        class Stop(Exception):
            pass

        def stop_after_clearing(source_root, destination_root):
            raise Stop

        with tempfile.TemporaryDirectory() as root:
            arguments = namespace(root)
            key = row_driver.reuse_key_of(arguments, 6, "-")
            write(os.path.join(root, "row-6", "row-result.tsv"), f"status\tok\ndevice_exit\t{row_driver.ROW_TIMEOUT_EXIT_CODE}\nreuse_key\t{key}\n")
            write(os.path.join(root, "row-7", "partial.txt"), "left over from an interrupted run")
            original = row_driver.sync_tree
            row_driver.sync_tree = stop_after_clearing
            try:
                for number in (6, 7):
                    with self.assertRaises(Stop):
                        row_driver.run_row_on_the_lane(number, None, None, [], set(), set(), arguments, 0)
            finally:
                row_driver.sync_tree = original
            self.assertEqual(os.listdir(os.path.join(root, "row-6")), [row_driver.TIMED_OUT_MARKER_FILE])
            self.assertTrue(row_driver.marked_as_timed_out_before(os.path.join(root, "row-6"), key))
            self.assertFalse(row_driver.marked_as_timed_out_before(os.path.join(root, "row-6"), "another-key"))
            self.assertEqual(os.listdir(os.path.join(root, "row-7")), [])
            self.assertFalse(row_driver.marked_as_timed_out_before(os.path.join(root, "row-7"), key))

    def test_a_compile_failure_a_pipe_defect_and_a_timeout_are_not_reused(self) -> None:
        self.assertFalse(row_driver.result_is_reusable({"status": row_driver.STATUS_COMPILE_FAILED, "device_exit": "-"}))
        self.assertFalse(row_driver.result_is_reusable({"status": row_driver.STATUS_MEASUREMENT_PIPE_DEFECT}))
        self.assertFalse(row_driver.result_is_reusable({"status": row_driver.STATUS_OK, "device_exit": str(row_driver.ROW_TIMEOUT_EXIT_CODE)}))
        self.assertTrue(row_driver.result_is_reusable({"status": row_driver.STATUS_OK, "device_exit": "0"}))


if __name__ == "__main__":
    unittest.main()
