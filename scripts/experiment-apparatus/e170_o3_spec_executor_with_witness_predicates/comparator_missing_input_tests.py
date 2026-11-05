"""比对器与参照的缺输入、缺省范围单测：comparator_two_versions.py 与 mutation_driver_on_rows.py 不给 --batches 就是全部批（1、2、3）；
witness_reference.py（images 模式）与 comparator_bad_image_judge.py 的 --known-bad-dir 与 --known-bad-not-applicable 都不给，已知坏镜像那一节在结果行里
报本次未跑、往标准错误打 NOT_RUN_INPUT_NOT_GIVEN、整趟退 77，别的节都不读（路径都是假的，读了就会炸）；给了 --known-bad-not-applicable
（这个输入没有已知坏镜像这份东西）那一节不读不判、结果行写 known_bad=not_applicable；两个都给退 2。
文件名以 comparator 起头，不进 v2 冻结的 sha256。
跑法：在本目录 `python3 -B -m unittest comparator_missing_input_tests`。"""
from __future__ import annotations

import argparse
import contextlib
import io
import os
import tempfile
import unittest

import comparator_bad_image_judge
import comparator_two_versions
import mutation_driver_on_rows
import witness_reference


def product_lines(path: str) -> list[str]:
    with open(path, encoding="utf-8") as handle:
        return handle.read().splitlines()


class DefaultScope(unittest.TestCase):
    def test_the_comparator_takes_every_batch_when_none_is_given(self) -> None:
        arguments = comparator_two_versions.build_argument_parser().parse_args(
            ["--pair", "I2:states:v1.out:v2.out", "--positive-control-input", "I2", "--scratch", "s", "--output", "o"])
        self.assertEqual(arguments.batches, "1,2,3")

    def test_the_mutation_driver_takes_every_batch_when_none_is_given(self) -> None:
        arguments = mutation_driver_on_rows.build_argument_parser().parse_args(
            ["--mutation", "X21", "--table", "t", "--scratch", "s", "--dumps", "d", "--workers", "1", "--output", "o"])
        self.assertEqual(arguments.batches, "1,2,3")

    def test_a_given_batch_list_is_kept_as_the_narrowed_scope(self) -> None:
        arguments = comparator_two_versions.build_argument_parser().parse_args(
            ["--pair", "I2:states:v1.out:v2.out", "--batches", "1", "--positive-control-input", "I2", "--scratch", "s", "--output", "o"])
        self.assertEqual(arguments.batches, "1")


class MissingKnownBadDirectory(unittest.TestCase):
    def check(self, exit_code: int, error: str, output: str, test_name: str) -> None:
        self.assertEqual(exit_code, 77)
        self.assertIn(f"NOT_RUN_INPUT_NOT_GIVEN test={test_name} reason=--known-bad-dir", error)
        lines = product_lines(output)
        self.assertIn("name=not_run section=known_bad_images input=--known-bad-dir reason=input_not_given", lines)
        self.assertEqual(lines[-1], f"name=done emitted={len(lines)}")

    def test_witness_reference_images_without_the_known_bad_directory_is_not_run(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            output = os.path.join(root, "reference.out")
            error = io.StringIO()
            with contextlib.redirect_stderr(error):
                exit_code = witness_reference.main(["images", "--dumps", "/nowhere", "--v2-output", "/nowhere", "--input-label", "I3", "--output", output])
            self.check(exit_code, error.getvalue(), output, "witness_reference.py")

    def test_the_bad_image_judge_without_the_known_bad_directory_is_not_run(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            output = os.path.join(root, "judge.out")
            error = io.StringIO()
            with contextlib.redirect_stderr(error):
                exit_code = comparator_bad_image_judge.main(["--executor-directory", "/nowhere", "--input-label", "I3", "--dumps", "/nowhere",
                                                             "--freeze-reference", "/nowhere", "--output", output])
            self.check(exit_code, error.getvalue(), output, "comparator_bad_image_judge.py")


class KnownBadNotApplicable(unittest.TestCase):
    def test_the_bad_image_judge_skips_the_known_bad_section_when_it_is_not_applicable(self) -> None:
        given = argparse.Namespace(known_bad_dir=None, known_bad_not_applicable=True, input_label="I3")
        neither = argparse.Namespace(known_bad_dir=None, known_bad_not_applicable=False, input_label="I3")
        emitted: list[str] = []
        self.assertEqual(comparator_bad_image_judge.sections_without_input(given), [])
        self.assertEqual(comparator_bad_image_judge.sections_without_input(neither), [("known_bad_images", "--known-bad-dir")])
        self.assertEqual(comparator_bad_image_judge.known_bad_config_text(given), "known_bad=not_applicable")
        self.assertEqual(comparator_bad_image_judge.judge_known_bad_images(given, None, None, None, emitted.append), "not_applicable")
        self.assertEqual(emitted, [])

    def test_witness_reference_reads_no_known_bad_rows_when_it_is_not_applicable(self) -> None:
        given = argparse.Namespace(known_bad_dir=None, known_bad_not_applicable=True)
        neither = argparse.Namespace(known_bad_dir=None, known_bad_not_applicable=False)
        self.assertEqual(witness_reference.images_sections_without_input(given), [])
        self.assertEqual(witness_reference.images_sections_without_input(neither), [("known_bad_images", "--known-bad-dir")])
        self.assertEqual(witness_reference.known_bad_config_text(given), "known_bad=not_applicable")
        self.assertEqual(witness_reference.known_bad_patch_rows(given), {})

    def test_both_known_bad_options_together_are_a_usage_error(self) -> None:
        for call in (lambda: witness_reference.main(["images", "--dumps", "/nowhere", "--known-bad-dir", "/nowhere", "--known-bad-not-applicable",
                                                     "--v2-output", "/nowhere", "--input-label", "I3", "--output", "/nowhere/out"]),
                     lambda: comparator_bad_image_judge.main(["--executor-directory", "/nowhere", "--input-label", "I3", "--dumps", "/nowhere",
                                                              "--known-bad-dir", "/nowhere", "--known-bad-not-applicable",
                                                              "--freeze-reference", "/nowhere", "--output", "/nowhere/out"])):
            error = io.StringIO()
            with contextlib.redirect_stderr(error), self.assertRaises(SystemExit) as raised:
                call()
            self.assertEqual(raised.exception.code, 2)
            self.assertIn("not allowed with argument", error.getvalue())


if __name__ == "__main__":
    unittest.main()
