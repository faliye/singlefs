"""comparator.py 的缺输入单测：--baseline-repeat-dir、--e170-dump、--o3-baseline、--control、--disabled、--rows-root、--known-bad-dir、
--e170-expected-dir、--image-judgements 哪一样没给，那一节在结果行里报本次未跑、往标准错误打 NOT_RUN_INPUT_NOT_GIVEN、整趟退 77，别的节都不读；
九样都给了不走这一支。冻结行的参照产物是否归档不在这里判（换成假的）。每条用例自己建、自己删临时目录。
跑法：在本目录 `python3 -B -m unittest test_comparator_inputs_not_given`。"""
from __future__ import annotations

import argparse
import contextlib
import io
import os
import tempfile
import unittest
from unittest import mock

import comparator
import frozen_o3

SECTIONS = ["second_baseline", "e170_dump_compare", "o3_baseline", "positive_controls", "disabled_oracles", "rows",
            "bad_images", "bad_images", "bad_images_o3"]


class InputsNotGiven(unittest.TestCase):
    def test_a_run_with_only_the_required_arguments_reports_every_section_and_exits_77(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            output = os.path.join(root, "comparator.out")
            error = io.StringIO()
            with mock.patch.object(frozen_o3, "report_missing_freeze_reference", return_value=False), contextlib.redirect_stderr(error):
                exit_code = comparator.main(["--output", output, "--mutations-table", "/nowhere", "--baseline-dir", "/nowhere/baseline-run1",
                                             "--baseline-device-output", "/nowhere"])
            with open(output, encoding="utf-8") as handle:
                lines = handle.read().splitlines()
        self.assertEqual(exit_code, frozen_o3.NOT_RUN_EXIT_CODE)
        self.assertEqual([line.split()[2] for line in lines if line.startswith("E7RESULT name=not_run ")], [f"section={section}" for section in SECTIONS])
        self.assertEqual(error.getvalue().count("NOT_RUN_INPUT_NOT_GIVEN test=comparator.py"), len(SECTIONS))
        self.assertEqual(lines[-1], f"E7RESULT name=done emitted={len(lines)}")

    def test_every_input_given_leaves_nothing_to_report(self) -> None:
        given = argparse.Namespace(baseline_repeat_dir="b2", e170_dump="d", o3_baseline="o3", control=["c"], disabled=["d"], rows_root=["r"],
                                   known_bad_dir="k", e170_expected_dir="e", image_judgements="j")
        self.assertEqual(comparator.sections_without_input(given), [])


if __name__ == "__main__":
    unittest.main()
