"""row_driver.py 的选行与「不按键复用」的单测：不给 --rows、--first-slice 就是表里全部行（给了是指定范围，给空串是不跑表里的行）；
输出根里已有的行目录（做完的、没做完的、上一次超时的）一律清掉重做，驱动不凭它当做完；限时乘 3 只看调用方给的 --timed-out-before；
--report-only 照调用方放进输出根的行目录重报，点名的行有一个没有 row-result.tsv 就一行都不报；不认 --skip-finished、--rerun-compile-failed-from、
--rerun-of-timed-out-rows、--all（用法错退 2）。可复用的结果只在判定库里，入库条件与键由 reuse_library.py 的自证与 test_revision_11 钉。
每条用例自己建、自己删临时目录；不编、不跑装置。跑法：在本目录 `python3 -B -m unittest test_row_selection_and_reuse`。"""
from __future__ import annotations

import argparse
import contextlib
import io
import os
import tempfile
import unittest

import device_timing
import row_driver
from bad_implementation_table import STATUS_LOCATED, MutationRow, SelectedRow


def selected_rows() -> list[SelectedRow]:
    return [SelectedRow(number, 1 if number <= 2 else 2, "core/mount.rs", number, f"name-{number}", "0" * 12) for number in (1, 2, 3)]


def write(path: str, text: str) -> None:
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)


def namespace(root: str, timed_out_before: bool = False) -> argparse.Namespace:
    return argparse.Namespace(streams="1,2,3", writable_mount_probe=False, positive_control=None, disable_oracle=None,
                              no_executor=False, keep_dumps=False, o3_control_bad_states=None, output_root=root, repo_root=root, threads=1,
                              work_root=os.path.join(root, "work"), baseline_process_wall_seconds=100.0, timed_out_before=timed_out_before)


class Stop(Exception):
    """桩：同步线道树那一步抛它，用例停在「行目录清过之后」。"""


def stop_after_clearing(source_root: str, destination_root: str) -> None:
    raise Stop


@contextlib.contextmanager
def replaced(**functions):
    """把 row_driver 里点名的函数换成桩，出 with 换回原件。"""
    originals = {name: getattr(row_driver, name) for name in functions}
    for name, function in functions.items():
        setattr(row_driver, name, function)
    try:
        yield
    finally:
        for name, function in originals.items():
            setattr(row_driver, name, function)


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


class NoReuseByKey(unittest.TestCase):
    MUTATION = MutationRow("m-row", "crates/singlefs-core/src/a.rs", "x", "y", "-p singlefs-core", "t_row",
                           raw_line="m-row\tcrates/singlefs-core/src/a.rs\tx\ty\t-p singlefs-core\tt_row")

    def test_an_existing_row_directory_is_cleared_for_a_redo_whatever_it_holds(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            arguments = namespace(root)
            write(os.path.join(root, "row-3", "row-result.tsv"), "status\tok\ndevice_exit\t0\n")
            write(os.path.join(root, "row-4", "row-result.tsv"), f"status\tok\ndevice_exit\t{row_driver.ROW_TIMEOUT_EXIT_CODE}\n")
            write(os.path.join(root, "row-5", "partial.txt"), "left over from an interrupted run")
            with replaced(sync_tree=stop_after_clearing):
                for number in (3, 4, 5):
                    with self.assertRaises(Stop):
                        row_driver.run_row_on_the_lane(number, None, None, [], set(), set(), arguments, 0)
            for number in (3, 4, 5):
                self.assertEqual(os.listdir(os.path.join(root, f"row-{number}")), [], number)

    def run_one_row(self, root: str, number: int, timed_out_before: bool) -> tuple[list[int], dict[str, object]]:
        """桩掉同步、挑行、编译与装置（装置照超时退 253），交回 (装置拿到的限时, row-result.tsv 的字段)。"""
        limits: list[int] = []

        def device_with_the_limit(repo_root, binary, row_directory, threads, extra, time_limit_seconds):
            limits.append(time_limit_seconds)
            return row_driver.ROW_TIMEOUT_EXIT_CODE, 1

        with replaced(sync_tree=lambda source_root, destination_root: None,
                      prepare_row=lambda *arguments, **keywords: row_driver.RowChoice(self.MUTATION, STATUS_LOCATED, "-", True, 0, ()),
                      compile_one_mutation=lambda *arguments, **keywords: (0, 1, "crates/singlefs-core"), run_device=device_with_the_limit):
            row_driver.run_row_on_the_lane(number, None, None, [], set(), set(), namespace(root, timed_out_before), 0)
        return limits, row_driver.read_result(os.path.join(root, f"row-{number}"))

    def test_the_time_limit_is_multiplied_only_when_the_caller_gives_timed_out_before(self) -> None:
        plain = device_timing.row_time_limit_seconds(100.0, False)
        with tempfile.TemporaryDirectory() as root:
            write(os.path.join(root, "row-6", "row-result.tsv"), f"status\tok\ndevice_exit\t{row_driver.ROW_TIMEOUT_EXIT_CODE}\n")
            limits, fields = self.run_one_row(root, 6, False)
            self.assertEqual((limits, fields["time_limit_seconds"], fields["timed_out_before"]), ([plain], str(plain), "false"))
            limits, fields = self.run_one_row(root, 7, True)
            self.assertEqual((limits, fields["time_limit_seconds"], fields["timed_out_before"]), ([3 * plain], str(3 * plain), "true"))

    def test_report_only_reads_the_rows_the_caller_placed_and_refuses_a_missing_one(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            write(os.path.join(root, "row-0", "row-result.tsv"), "row_number\t0\nstatus\tok\n")
            write(os.path.join(root, "row-9", "row-result.tsv"), "row_number\t9\nstatus\tok\n")
            write(os.path.join(root, "row-4", "device.out"), "E7RESULT name=half\n")
            self.assertEqual(row_driver.reported_rows(root, [9, 0]), ([{"row_number": "9", "status": "ok"}, {"row_number": "0", "status": "ok"}], []))
            self.assertEqual(row_driver.reported_rows(root, [0, 4, 5]), ([], [4, 5]))


if __name__ == "__main__":
    unittest.main()
