"""comparator_q2_implementation_verdicts.py 的缺输入单测：不给 --freeze-reference，freeze_check 那一节在结果行里报本次未跑、
往标准错误打 NOT_RUN_INPUT_NOT_GIVEN、整趟退 77，别的节都不读（路径都是假的，读了就会炸）；给了 --freeze-reference 不走这一支。
文件名以 comparator 起头，不进冻结的 sha256。
跑法：在本目录 `python3 -B -m unittest comparator_missing_input_tests`。"""
from __future__ import annotations

import argparse
import contextlib
import io
import os
import tempfile
import unittest

import comparator_q2_implementation_verdicts as comparator


class MissingFreezeReference(unittest.TestCase):
    def test_without_the_freeze_reference_the_run_is_not_run_and_exits_77(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            output = os.path.join(root, "q2.out")
            error = io.StringIO()
            with contextlib.redirect_stderr(error):
                exit_code = comparator.main(["--dumps", "/nowhere", "--frozen-verdicts", "/nowhere", "--output", output])
            with open(output, encoding="utf-8") as handle:
                lines = handle.read().splitlines()
        self.assertEqual(exit_code, 77)
        self.assertIn("NOT_RUN_INPUT_NOT_GIVEN test=comparator_q2_implementation_verdicts.py reason=--freeze-reference", error.getvalue())
        self.assertIn("E7RESULT name=not_run section=freeze_check input=--freeze-reference reason=input_not_given", lines)
        self.assertEqual(lines[-1], f"E7RESULT name=done emitted={len(lines)}")

    def test_a_given_freeze_reference_leaves_nothing_to_report(self) -> None:
        given = argparse.Namespace(freeze_reference="/some/freeze.out")
        self.assertEqual(comparator.sections_without_input(given), [])


if __name__ == "__main__":
    unittest.main()
