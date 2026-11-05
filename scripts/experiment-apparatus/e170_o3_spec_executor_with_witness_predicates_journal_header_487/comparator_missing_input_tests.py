"""comparator_q5_same_shape.py 的缺输入单测：--expected-freeze、--record-paths、--anchors-b15、--b17、--positive-control-row-input（连同 --scratch）
哪一样没给，那一节在结果行里报本次未跑、往标准错误打 NOT_RUN_INPUT_NOT_GIVEN、整趟退 77，别的节都不读（路径都是假的，读了就会炸）；
五样都给了不走这一支。文件名以 comparator 起头，不进冻结的 sha256。
跑法：在本目录 `python3 -B -m unittest comparator_missing_input_tests`。"""
from __future__ import annotations

import argparse
import contextlib
import io
import os
import tempfile
import unittest

import comparator_q5_same_shape as comparator

PAIR = "Z1-I2:states:judged:e170_control:today.out:reference.out"
SECTIONS = ["freeze_check_today", "q5e_records", "b15", "b17", "q5_pc2"]


class MissingOptionalInputs(unittest.TestCase):
    def test_without_the_optional_inputs_every_section_is_reported_and_the_run_exits_77(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            output = os.path.join(root, "q5.out")
            error = io.StringIO()
            with contextlib.redirect_stderr(error):
                exit_code = comparator.main(["--pair", PAIR, "--output", output])
            with open(output, encoding="utf-8") as handle:
                lines = handle.read().splitlines()
        self.assertEqual(exit_code, 77)
        not_run = [line.split()[1] for line in lines if line.startswith("name=not_run ")]
        self.assertEqual(not_run, [f"section={section}" for section in SECTIONS])
        self.assertEqual(error.getvalue().count("NOT_RUN_INPUT_NOT_GIVEN test=comparator_q5_same_shape.py"), len(SECTIONS))
        self.assertEqual(lines[-1], f"name=done emitted={len(lines)}")

    def test_a_positive_control_without_a_scratch_directory_is_reported_too(self) -> None:
        given = argparse.Namespace(expected_freeze="f", record_paths="r", anchors_b15="Z3-I3", b17="p:k", positive_control_row_input="Z3-I3", scratch=None)
        self.assertEqual(comparator.sections_without_input(given), [("q5_pc2", "--scratch")])

    def test_every_optional_input_given_leaves_nothing_to_report(self) -> None:
        given = argparse.Namespace(expected_freeze="f", record_paths="r", anchors_b15="Z3-I3", b17="p:k", positive_control_row_input="Z3-I3", scratch="s")
        self.assertEqual(comparator.sections_without_input(given), [])


if __name__ == "__main__":
    unittest.main()
