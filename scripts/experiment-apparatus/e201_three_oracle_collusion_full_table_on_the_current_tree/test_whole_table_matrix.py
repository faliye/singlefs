"""whole_table_matrix.py 的单测：整表漏判集只收登记读法批 1 计入分母的行、按流一流二批 1 判；O3 没跑的行 O3 两列不算漏；轨迹那几样照 collusion_matrix 出。
导出目录换成手造的 RunDirectory（不读盘上的导出格式）。跑法：在本目录 `python3 -m unittest test_whole_table_matrix`。"""

from __future__ import annotations

import os
import tempfile
import unittest
from unittest import mock

import whole_table_matrix
import e171_research_side  # noqa: F401
from run_directory import CallRow, RecoverySummary, RunDirectory, StateRow, StreamRun

STREAM_ONE, STREAM_TWO = "new_pool_file_creation", "fixed_script_through_e_then_unmount"
SUMMARY = RecoverySummary("root", "ok", "-", 0)


def state(number: int, segment: int, batch: int = 1, crash_judge: str = "green", pool_checker: str = "green") -> StateRow:
    return StateRow(number, number, segment, batch, "-", "-", None, crash_judge, "-", pool_checker, "-", True, SUMMARY, SUMMARY, "-",
                    "green", "-", "-", "-", "-")


def call(index: int, model_word: str = "agrees") -> CallRow:
    return CallRow(index, "create_inode", 0, 0, "ok", model_word, "-", "-", "-", "-", "-")


def run_with(o2_red_segments=(), crash_red=False) -> RunDirectory:
    streams = {}
    for name in (STREAM_ONE, STREAM_TWO):
        states = [state(segment, segment, pool_checker="red" if segment in o2_red_segments and name == STREAM_ONE else "green",
                        crash_judge="red" if crash_red and name == STREAM_TWO else "green") for segment in range(1, 4)]
        streams[name] = StreamRun(name, True, states, [call(1)], status={"ending": "finished"})
    streams["first_file_unlinked_then_inodes_across_one_leaf"] = StreamRun("first_file_unlinked_then_inodes_across_one_leaf", False)
    return RunDirectory("/fake", streams)


class Recorder:
    def __init__(self):
        self.lines: list[str] = []

    def result(self, body: str) -> None:
        self.lines.append(body)


def t1(*rows: tuple[int, str]) -> str:
    return "\n".join(f"E7RESULT name=row row={row} status=ok category_registration_batch_1={category}" for row, category in rows)


class MatrixTests(unittest.TestCase):
    def build(self, runs: dict[int, RunDirectory], o3_rows: set[int], categories: str) -> tuple[dict, list[str]]:
        with tempfile.TemporaryDirectory() as root:
            for number in runs:
                os.makedirs(os.path.join(root, f"row-{number}", "dump"))
            os.makedirs(os.path.join(root, "row-0", "dump"))
            loader = lambda path: runs[int(os.path.basename(os.path.dirname(path)).removeprefix("row-"))]
            o3_reader = lambda path: {} if int(os.path.basename(os.path.dirname(path)).removeprefix("row-")) in o3_rows else None
            recorder = Recorder()
            with mock.patch.object(whole_table_matrix, "load_run_directory", loader), mock.patch.object(whole_table_matrix, "read_o3", o3_reader):
                missed = whole_table_matrix.matrix_lines(recorder, root, categories, (STREAM_ONE, STREAM_TWO))
        return missed, recorder.lines

    def test_only_counted_rows_enter_the_missed_sets(self):
        missed, _ = self.build({15: run_with(), 23: run_with(o2_red_segments={2})}, set(), t1((15, "counted"), (23, "counted"), (42, "untouched")))
        self.assertEqual(missed["O2"], [15])
        self.assertEqual(missed["O1"], [15, 23])
        self.assertEqual(missed["O3甲"], [], "O3 没跑的行不算漏")

    def test_an_untouched_row_is_left_out_and_the_empty_mutation_row_is_not_a_table_row(self):
        missed, lines = self.build({15: run_with(), 23: run_with()}, {15, 23}, t1((15, "counted"), (23, "untouched")))
        self.assertEqual(missed["O3甲"], [15])
        self.assertFalse(any(line.startswith("name=matrix_row row=0 ") for line in lines))

    def test_the_trajectory_fields_come_from_the_segments(self):
        _, lines = self.build({15: run_with(o2_red_segments={1, 3})}, set(), t1((15, "counted")))
        line = next(line for line in lines if f"stream={STREAM_ONE} " in line and " batch=1 " in line)
        self.assertIn("O2_red=2 ", line)
        self.assertIn("O2_positive_segments=2 ", line)
        self.assertIn("O2_last=1 ", line)
        self.assertIn("O2_single_points=2", line)


if __name__ == "__main__":
    unittest.main()
