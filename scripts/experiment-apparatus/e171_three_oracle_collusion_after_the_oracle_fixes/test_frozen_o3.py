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
MOVED_EXECUTOR_TEXT = "# 搬家之后：--dumps <scripts/experiment-apparatus/target/e170-dumps>\n"
EDITED_EXECUTOR_TEXT = "# 搬家之后又改了判法：--dumps <scripts/experiment-apparatus/target/e170-dumps>\nverdict = 'green'\n"
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


# 搬迁登记（frozen_o3 文件头「搬迁登记」）的样本：前缀用假的 fixtures/old-home、fixtures/new-home，不用仓里真的目录名
# （真目录改名时全仓的文本替换不会把样本换坏）。fixtures/old-homestead 不在前缀边界上，换不换都一样。
OLD_HOME, MIDDLE_HOME, NEW_HOME = "fixtures/old-home", "fixtures/middle-home", "fixtures/new-home"
HOME_TEXT = "# 跑法：--dumps <{home}/target/e170-dumps>，旁边的 fixtures/old-homestead/notes 不在前缀边界上\nprint('{home}/x')\n"
FROZEN_HOME_TEXT = HOME_TEXT.format(home=OLD_HOME)
MOVED_HOME_TEXT = HOME_TEXT.format(home=NEW_HOME)
MOVED_AND_EDITED_HOME_TEXT = MOVED_HOME_TEXT + "verdict = 'green'\n"
MOVE_SOURCE = "records/2026-10-23-实验装置目录改名.md"


def move_row(old: str, new: str) -> str:
    return f"{old}\t{new}\t2026-10-23\t{MOVE_SOURCE}\n"


class RegisteredDirectoryMoves(unittest.TestCase):
    """红：登记表空、只改了路径；绿：登记了搬迁、只差路径；红：登记了搬迁、除了路径还改了别处。种子与文件同判。"""

    def setUp(self) -> None:
        self.frozen = {"executor_main.py": FROZEN_HOME_TEXT, "executor_core.py": CORE_TEXT}

    def differences_with(self, current_main: str, table_text: str | None, registration: dict[str, tuple[str, str]] | None = None,
                         frozen: dict[str, str] | None = None) -> list[str]:
        repository = FakeExecutorRepository(frozen or self.frozen, {"executor_main.py": current_main, "executor_core.py": CORE_TEXT})
        try:
            if table_text is not None:
                table = os.path.join(repository.root, frozen_o3.PATH_MOVES_TABLE_RELATIVE)
                os.makedirs(os.path.dirname(table))
                with open(table, "w", encoding="utf-8") as handle:
                    handle.write(table_text)
            return frozen_o3.freeze_differences(repository.root, repository.lines, registration or {})
        finally:
            repository.close()

    def test_without_a_move_table_a_path_only_move_is_a_difference(self) -> None:
        self.assertEqual(self.differences_with(MOVED_HOME_TEXT, None), ["executor_main.py:sha256_differs", "freeze_seed:differs"],
                         "登记表不在：只改了路径也照判不同，文件与种子各一条")

    def test_a_move_table_with_only_comments_judges_like_no_table(self) -> None:
        self.assertEqual(self.differences_with(MOVED_HOME_TEXT, "# 搬迁登记（只有注释）\n\n"), ["executor_main.py:sha256_differs", "freeze_seed:differs"],
                         "登记表只有注释与空行：与表不在同判")
        self.assertEqual(self.differences_with(FROZEN_HOME_TEXT, "# 搬迁登记（只有注释）\n"), [], "登记表只有注释、内容没动：不该有不同")

    def test_a_registered_directory_move_with_only_path_differences_is_not_a_difference(self) -> None:
        self.assertEqual(self.differences_with(MOVED_HOME_TEXT, move_row(OLD_HOME, NEW_HOME)), [],
                         "登记了搬迁、此刻与冻结那一版只差登记的前缀：文件与种子都不该记不同")

    def test_a_registered_move_does_not_pass_an_edit_besides_the_paths(self) -> None:
        self.assertEqual(self.differences_with(MOVED_AND_EDITED_HOME_TEXT, move_row(OLD_HOME, NEW_HOME)),
                         ["executor_main.py:sha256_differs", "freeze_seed:differs"], "登记了搬迁、但除了路径还改了别处：照判不同")

    def test_a_move_registered_the_other_way_round_is_still_a_difference(self) -> None:
        self.assertEqual(self.differences_with(MOVED_HOME_TEXT, move_row(NEW_HOME, OLD_HOME)),
                         ["executor_main.py:sha256_differs", "freeze_seed:differs"], "登记的方向反了（新换旧）：换回去对不上，照判不同")

    def test_chained_moves_are_undone_latest_first(self) -> None:
        self.assertEqual(self.differences_with(MOVED_HOME_TEXT, move_row(OLD_HOME, MIDDLE_HOME) + move_row(MIDDLE_HOME, NEW_HOME)), [],
                         "A→B 之后又 B→C：倒着先换 C→B、再 B→A，换回冻结那一版")

    def test_a_move_on_top_of_a_registered_path_only_change_is_not_a_difference(self) -> None:
        frozen = {"executor_main.py": FROZEN_EXECUTOR_TEXT, "executor_core.py": CORE_TEXT}
        registered_now = {"executor_main.py": FROZEN_HOME_TEXT, "executor_core.py": CORE_TEXT}
        registration = {"executor_main.py": (sha256_of_text(FROZEN_EXECUTOR_TEXT), sha256_of_text(FROZEN_HOME_TEXT)),
                        frozen_o3.SEED_NAME: (seed_of_texts(frozen), seed_of_texts(registered_now))}
        self.assertEqual(self.differences_with(MOVED_HOME_TEXT, move_row(OLD_HOME, NEW_HOME), registration, frozen), [],
                         "只差路径的登记那一对的「现在」，再叠一次登记过的搬迁：换回去对上登记的「现在」，不算不同")
        self.assertEqual(self.differences_with(MOVED_HOME_TEXT, None, registration, frozen), ["executor_main.py:sha256_differs", "freeze_seed:differs"],
                         "同一份内容，搬迁没登记：照判不同")

    def test_a_broken_move_table_is_refused_not_guessed(self) -> None:
        with self.assertRaises(ValueError, msg="登记表三列：拒，不猜"):
            self.differences_with(MOVED_HOME_TEXT, f"{OLD_HOME}\t{NEW_HOME}\t2026-10-23\n")


if __name__ == "__main__":
    unittest.main()
