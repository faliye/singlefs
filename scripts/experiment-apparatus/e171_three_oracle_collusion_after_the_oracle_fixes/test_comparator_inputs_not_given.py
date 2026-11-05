"""segment_one_comparator.py 的缺输入单测：--second-baseline、--baseline-o3、--rows-root、--q5-main、--q5-reverse、--control 哪一样没给，
那一节在结果行里报本次未跑、往标准错误打 NOT_RUN_INPUT_NOT_GIVEN、整趟退 77，别的节都不读；六样都给了不走这一支。
每条用例自己建、自己删临时目录。
跑法：在本目录 `python3 -B -m unittest test_comparator_inputs_not_given`。"""
from __future__ import annotations

import argparse
import contextlib
import io
import os
import tempfile
import unittest

import segment_one_comparator as comparator

SECTIONS = ["z1_two_baselines", "o3_baseline", "rows", "q5_main", "q5_reverse", "a8_positive_controls"]


class InputsNotGiven(unittest.TestCase):
    def test_a_run_with_only_the_baseline_reports_every_section_and_exits_77(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            dumps_root = os.path.join(root, "dumps")
            baseline = os.path.join(dumps_root, "baseline-run1")
            os.makedirs(baseline)
            output = os.path.join(root, "comparator.out")
            error = io.StringIO()
            with contextlib.redirect_stderr(error):
                exit_code = comparator.main(["--baseline", baseline, "--output", output, "--dumps-root", dumps_root,
                                             "--product-prefix", "p-", "--product-suffix=-s.out"])
            with open(output, encoding="utf-8") as handle:
                lines = handle.read().splitlines()
        self.assertEqual(exit_code, comparator.NOT_RUN_EXIT_CODE)
        self.assertEqual([line.split()[2] for line in lines if line.startswith("E7RESULT name=not_run ")], [f"section={section}" for section in SECTIONS])
        self.assertEqual(error.getvalue().count("NOT_RUN_INPUT_NOT_GIVEN test=segment_one_comparator.py"), len(SECTIONS))
        self.assertEqual(lines[-1], f"E7RESULT name=done emitted={len(lines)}")

    def test_every_input_given_leaves_nothing_to_report(self) -> None:
        given = argparse.Namespace(second_baseline="b2", baseline_o3="o3", rows_root="rows", q5_main="main", q5_reverse="reverse")
        self.assertEqual(comparator.sections_without_input(given, {"O2": ["control.out"]}), [])

    def test_a_missing_control_list_is_reported_as_the_positive_controls_section(self) -> None:
        given = argparse.Namespace(second_baseline="b2", baseline_o3="o3", rows_root="rows", q5_main="main", q5_reverse="reverse")
        self.assertEqual(comparator.sections_without_input(given, {}), [("a8_positive_controls", "--control")])


if __name__ == "__main__":
    unittest.main()
