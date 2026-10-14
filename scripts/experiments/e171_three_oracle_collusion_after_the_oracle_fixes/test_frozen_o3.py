"""E171 冻结执行器核对（`frozen_o3.freeze_differences`，作废条款 Z2）与只差路径的重冻结登记（`freeze-path-only-changes.tsv`）的单测。

跑法：`python3 -B -m unittest test_frozen_o3`（在本目录）。只用临时目录里手造的执行器目录与冻结行，不读仓里的执行器与产物。
"""

from __future__ import annotations

import hashlib
import os
import tempfile
import unittest

import frozen_o3

FROZEN_EXECUTOR_TEXT = "# 冻结那一版：--dumps <research/target/e170-dumps>\n"
MOVED_EXECUTOR_TEXT = "# 搬家之后：--dumps <scripts/experiments/target/e170-dumps>\n"
EDITED_EXECUTOR_TEXT = "# 搬家之后又改了判法：--dumps <scripts/experiments/target/e170-dumps>\nverdict = 'green'\n"
CORE_TEXT = "def judge(): return 'green'\n"


def sha256_of_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def seed_of_texts(texts_by_name: dict[str, str]) -> str:
    hasher = hashlib.sha256()
    for name in sorted(texts_by_name):
        hasher.update(texts_by_name[name].encode("utf-8"))
    return hasher.hexdigest()[:16]


class FakeExecutorRepository:
    """临时目录里一个只有冻结目录的仓：冻结行按 `frozen_texts` 算，目录里的文件按 `current_texts` 写。"""

    def __init__(self, frozen_texts: dict[str, str], current_texts: dict[str, str]) -> None:
        self.temporary = tempfile.TemporaryDirectory(prefix="e171-frozen-o3-test-")
        self.root = self.temporary.name
        directory = os.path.join(self.root, frozen_o3.FROZEN_DIRECTORY_RELATIVE)
        os.makedirs(directory)
        for name, text in current_texts.items():
            with open(os.path.join(directory, name), "w", encoding="utf-8") as handle:
                handle.write(text)
        self.lines = [f"name=freeze file={name} sha256={sha256_of_text(text)}" for name, text in sorted(frozen_texts.items())]
        self.lines.append(f"name=freeze_seed sha256_prefix16={seed_of_texts(frozen_texts)}")
        self.current_seed = seed_of_texts(current_texts)

    def close(self) -> None:
        self.temporary.cleanup()


def path_only_registration_of_the_move() -> dict[str, tuple[str, str]]:
    moved = {"executor_main.py": MOVED_EXECUTOR_TEXT, "executor_core.py": CORE_TEXT}
    frozen = {"executor_main.py": FROZEN_EXECUTOR_TEXT, "executor_core.py": CORE_TEXT}
    return {"executor_main.py": (sha256_of_text(FROZEN_EXECUTOR_TEXT), sha256_of_text(MOVED_EXECUTOR_TEXT)),
            frozen_o3.SEED_NAME: (seed_of_texts(frozen), seed_of_texts(moved))}


class PathOnlyRefreeze(unittest.TestCase):
    def setUp(self) -> None:
        self.frozen = {"executor_main.py": FROZEN_EXECUTOR_TEXT, "executor_core.py": CORE_TEXT}

    def differences_with(self, current_texts: dict[str, str], registration: dict[str, tuple[str, str]]) -> list[str]:
        repository = FakeExecutorRepository(self.frozen, current_texts)
        try:
            return frozen_o3.freeze_differences(repository.root, repository.lines, registration)
        finally:
            repository.close()

    def test_unchanged_executor_has_no_difference_without_any_registration(self) -> None:
        self.assertEqual(self.differences_with(dict(self.frozen), {}), [], "冻结那一版原样在：不该有不同")

    def test_a_registered_path_only_change_with_both_values_matching_is_not_a_difference(self) -> None:
        moved = {"executor_main.py": MOVED_EXECUTOR_TEXT, "executor_core.py": CORE_TEXT}
        self.assertEqual(self.differences_with(moved, path_only_registration_of_the_move()), [],
                         "登记了、冻结时与现在两个值都对上：文件与种子都不该记不同")

    def test_an_unregistered_path_change_is_a_difference_in_the_file_and_the_seed(self) -> None:
        moved = {"executor_main.py": MOVED_EXECUTOR_TEXT, "executor_core.py": CORE_TEXT}
        self.assertEqual(self.differences_with(moved, {}), ["executor_main.py:sha256_differs", "freeze_seed:differs"],
                         "没登记的改动：文件与种子各记一条")

    def test_an_edit_after_the_registration_is_a_difference_again(self) -> None:
        edited = {"executor_main.py": EDITED_EXECUTOR_TEXT, "executor_core.py": CORE_TEXT}
        self.assertEqual(self.differences_with(edited, path_only_registration_of_the_move()),
                         ["executor_main.py:sha256_differs", "freeze_seed:differs"],
                         "登记之后又改了：现在的值对不上登记的，照判不同")

    def test_a_registered_file_without_the_registered_seed_still_reports_the_seed(self) -> None:
        moved = {"executor_main.py": MOVED_EXECUTOR_TEXT, "executor_core.py": CORE_TEXT}
        registration = path_only_registration_of_the_move()
        del registration[frozen_o3.SEED_NAME]
        self.assertEqual(self.differences_with(moved, registration), ["freeze_seed:differs"], "种子没登记：只记种子那一条")

    def test_a_registration_row_with_the_wrong_number_of_columns_is_refused(self) -> None:
        with tempfile.TemporaryDirectory(prefix="e171-frozen-o3-table-") as directory:
            table = os.path.join(directory, "freeze-path-only-changes.tsv")
            with open(table, "w", encoding="utf-8") as handle:
                handle.write("# 注释行跳过\n\nexecutor_main.py\taaa\tbbb\t503b80e2\n")
            with self.assertRaises(ValueError, msg="四列的行该拒，不猜"):
                frozen_o3.path_only_changes_of(table)

    def test_a_registration_table_reads_name_frozen_and_current_values(self) -> None:
        with tempfile.TemporaryDirectory(prefix="e171-frozen-o3-table-") as directory:
            table = os.path.join(directory, "freeze-path-only-changes.tsv")
            with open(table, "w", encoding="utf-8") as handle:
                handle.write("# 注释行跳过\nexecutor_main.py\taaa\tbbb\t503b80e2\t改了一处路径\nfreeze_seed\tccc\tddd\t503b80e2\t跟着变\n")
            self.assertEqual(frozen_o3.path_only_changes_of(table), {"executor_main.py": ("aaa", "bbb"), "freeze_seed": ("ccc", "ddd")},
                             "名字 → (冻结时, 现在)，注释行不进")

    def test_a_missing_registration_table_is_an_empty_registration(self) -> None:
        with tempfile.TemporaryDirectory(prefix="e171-frozen-o3-table-") as directory:
            self.assertEqual(frozen_o3.path_only_changes_of(os.path.join(directory, "absent.tsv")), {}, "表不在：空登记")


if __name__ == "__main__":
    unittest.main()
