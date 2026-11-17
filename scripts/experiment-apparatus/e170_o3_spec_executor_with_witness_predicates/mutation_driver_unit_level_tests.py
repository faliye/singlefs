"""mutation_driver_unit_level.py 的单测：在一个手造的小目录上整张表跑一遍。
行的原文命中一次、副本里替换、单测红，记 caught；原文不在目录里的整张表不跑；原目录收尾逐文件没动。
文件名以 mutation_driver 起头，不进冻结的 sha256。
跑法：在本目录 `python3 -B -m unittest mutation_driver_unit_level_tests`。"""
from __future__ import annotations

import contextlib
import io
import os
import tempfile
import unittest

import mutation_driver_unit_level as driver

UNIT_TEST_TEXT = """import unittest

import value_module


class ValueTests(unittest.TestCase):
    def test_the_value(self) -> None:
        self.assertEqual(value_module.VALUE, 1)
"""


def write(path: str, text: str) -> None:
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)


class WholeTable(unittest.TestCase):
    def run_driver(self, table_text: str) -> tuple[int, list[str]]:
        with tempfile.TemporaryDirectory() as root:
            directory = os.path.join(root, "executor")
            os.makedirs(directory)
            write(os.path.join(directory, "value_module.py"), "VALUE = 1\n")
            write(os.path.join(directory, driver.TEST_MODULE + ".py"), UNIT_TEST_TEXT)
            table = os.path.join(root, "table.tsv")
            write(table, table_text)
            output = os.path.join(root, "result.out")
            with contextlib.redirect_stdout(io.StringIO()):
                exit_code = driver.main(["--executor-directory", directory, "--table", table, "--scratch", os.path.join(root, "scratch"), "--output", output])
            with open(output, encoding="utf-8") as handle:
                return exit_code, handle.read().splitlines()

    def test_a_row_is_caught(self) -> None:
        exit_code, lines = self.run_driver("X1_值改成2\tVALUE = 1\tVALUE = 2\n")
        self.assertEqual(exit_code, 0)
        self.assertTrue(any(line.startswith("name=mutation mutation=X1_值改成2 result=caught ") for line in lines), lines)
        counts = next(line for line in lines if line.startswith("name=mutation_counts "))
        self.assertEqual(counts, "name=mutation_counts caught=1 invalid=0 not_red=0 timeout=0 total=1 table_rows=1")
        self.assertIn("name=restore result=已还原，基线仍全绿", lines)

    def test_a_row_whose_original_text_is_not_in_the_directory_aborts_the_table(self) -> None:
        exit_code, lines = self.run_driver("X1_值改成2\tVALUE = 7\tVALUE = 2\n")
        self.assertEqual(exit_code, 3)
        self.assertIn("name=mutation_aborted reason=anchor_not_hit_exactly_once", lines)


if __name__ == "__main__":
    unittest.main()
