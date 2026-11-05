"""`list_check.py` 的单测：登记第五节 5.2 名单核对的几样阳性对照（Q2·有、Q2·没有、Q2·注释行、分类三类加未分类）与改后核对（第九节 M6–M9）。
表与名单都是手造的，不读真的仓。"""

from __future__ import annotations

import os
import tempfile
import unittest

import list_check
from bad_implementation_table import SelectedRow
from result_lines import ResultLines

FILE = "crates/singlefs-core/src/mount.rs"
# 手造的改名表：两行与 .claude/kb/term-renames.md 里「正则」「整串」两种行同形，旧名、新名都是造的词（不用登记过的旧名，门禁 doc-text 的 term-renames 格全仓搜旧名）。
RENAMES_TEXT = ("<!-- term-renames:table -->\n| 旧 | 新 | 匹配 | 是什么 |\n|---|---|---|---|\n"
                "| `(?<![第每各叶外内深一两三几多0-9])(?<!第 [0-9] )(?<!第 [0-9][0-9] )甲段 ?0(?![0-9.])(?! ?冒烟)` | 乙段名 | 正则 | 中文术语 |\n"
                "| 旧称丙 | 新称丙 | 整串 | 中文术语 |\n")


def table_line(name: str, original: str = "原文", replacement: str = "替换文", file: str = FILE) -> str:
    return "\t".join((name, file, original, replacement, "-p singlefs-core --lib", "测试名"))


def selected(number: int, name: str, digest_prefix: str = "000000000000", file: str = FILE) -> SelectedRow:
    return SelectedRow(number, 2, file, 0, name, digest_prefix)


def lookup_of(table_text: str, rows: list[SelectedRow]) -> tuple[list[int], list[int]]:
    with tempfile.TemporaryDirectory() as root:
        lines = ResultLines(os.path.join(root, "out"))
        not_found, ambiguous = list_check.lookup_lines(lines, "手造", rows, list_check.table_lines_from_text(table_text))
        lines.finish()
    return [row.number for row in not_found], [row.number for row in ambiguous]


class LookupByName(unittest.TestCase):
    """Q2-逐项：与逐行驱动同一读法，名字逐字比、跳过 `#` 行、只收 6 列。"""

    def test_a_name_missing_from_the_table_is_the_only_item_not_found(self) -> None:
        rows = [selected(1, "甲名"), selected(2, "乙名"), selected(3, "丙名")]
        self.assertEqual(lookup_of(table_line("甲名") + "\n" + table_line("丙名") + "\n", rows), ([2], []))

    def test_every_name_on_its_own_row_leaves_nothing_not_found(self) -> None:
        rows = [selected(1, "甲名"), selected(2, "乙名")]
        self.assertEqual(lookup_of(table_line("甲名") + "\n" + table_line("乙名") + "\n", rows), ([], []))

    def test_a_name_only_on_a_comment_line_is_not_found(self) -> None:
        # 名字本身以 # 起头，这一行不跳过就会被按名字取到：跳不跳过 # 行在这一格上结局不同
        rows = [selected(1, "#甲名")]
        self.assertEqual(lookup_of(table_line("#甲名") + "\n", rows), ([1], []))

    def test_a_name_on_two_rows_is_ambiguous_and_not_counted_as_not_found(self) -> None:
        rows = [selected(1, "甲名")]
        self.assertEqual(lookup_of(table_line("甲名") + "\n" + table_line("甲名", "别的原文") + "\n", rows), ([], [1]))


class Classification(unittest.TestCase):
    """读法 3：手造一项甲、一项乙（三样相同、名字只差一处登记改名，另有一行三样也相同）、一项丙（同锚点替换文不同）、一项未分类。"""

    def classify(self, row: SelectedRow, worktree: str, reference: str = "", record: str = "", history: str = "") -> list_check.Classification:
        return list_check.classify(row, list_check.table_lines_from_text(reference), list_check.table_lines_from_text(worktree), record,
                                   list_check.term_renames_from_text(RENAMES_TEXT), None,
                                   [("手造历史", list_check.table_lines_from_text(history))])

    def registration_row(self, number: int, history_line: str) -> SelectedRow:
        name = history_line.split("\t")[0]
        return selected(number, name, list_check.line_digest_prefix(history_line))

    def test_a_name_in_the_reference_table_gone_from_the_worktree_and_named_in_the_record_is_old_format(self) -> None:
        line = table_line("实一：读者仍把退役的位 0 当受支持的布局身份（旧镜像挂得上）")
        entry = self.classify(self.registration_row(236, line), table_line("别的"), reference=line,
                              record="撤掉「实一…读者仍把退役的位 0 当受支持的布局身份（旧镜像挂得上）」那一项", history=line)
        self.assertEqual((entry.klass, entry.in_reference_table, entry.in_worktree_table, entry.named_in_the_record), ("jia", 1, 0, True))

    def test_the_same_triple_under_the_renamed_name_is_retargeted_even_with_a_second_identical_triple(self) -> None:
        old = table_line("实审：树分裂 甲段 0：内部节点装不下也不切")
        worktree = table_line("树分裂 核心规则：内部节点装不下也不切") + "\n" + table_line("实审：树分裂 乙段名：内部节点装不下也不切") + "\n"
        entry = self.classify(self.registration_row(141, old), worktree, history=old)
        self.assertEqual((entry.klass, entry.chosen_name, len(entry.triple_matches)), ("yi", "实审：树分裂 乙段名：内部节点装不下也不切", 2))

    def test_a_single_identical_triple_under_another_name_is_retargeted(self) -> None:
        old = table_line("实二六：抬 F 预演报错照样往下发；种子 4000000001")
        entry = self.classify(self.registration_row(168, old), table_line("实二六：抬 F 预演报错照样往下发；第二次取不到落点那一格") + "\n", history=old)
        self.assertEqual((entry.klass, entry.chosen_name), ("yi", "实二六：抬 F 预演报错照样往下发；第二次取不到落点那一格"))

    def test_the_same_anchor_with_another_replacement_is_gone_not_retargeted(self) -> None:
        old = table_line("步 4：用最新根自己带的 F", replacement="最新根的 F")
        entry = self.classify(self.registration_row(33, old), table_line("步 4：用目标根自己带的 F", replacement="目标根的 F") + "\n", history=old)
        self.assertEqual((entry.klass, len(entry.triple_matches), len(entry.anchor_matches)), ("bing", 0, 1))

    def test_two_identical_triples_with_no_renamed_name_among_them_stay_unclassified(self) -> None:
        old = table_line("某条：照样往下发")
        entry = self.classify(self.registration_row(9, old), table_line("甲说法") + "\n" + table_line("乙说法") + "\n", history=old)
        self.assertEqual(entry.klass, "unclassified")

    def test_a_registration_line_missing_from_every_history_table_stays_unclassified(self) -> None:
        entry = self.classify(selected(10, "没有登记行的名字"), table_line("别的") + "\n", history=table_line("别的"))
        self.assertEqual((entry.klass, entry.reference_kind), ("unclassified", "not_found"))

    def test_an_r1_row_located_without_substitution_takes_the_line_by_its_full_sha256(self) -> None:
        r1_line = table_line("实二十：甲段 0 L8：当末条")
        r1_row = list_check.R1Row(True, list_check.TableLine(*r1_line.split("\t")[:4], r1_line).sha256(), "ok")
        kind, line, found_in = list_check.reference_line(selected(70, "实二十：甲段 0 L8：当末条", "ffffffffffff"), r1_row,
                                                         [("手造历史", list_check.table_lines_from_text(r1_line))])
        self.assertEqual((kind, line.raw_line if line else None, found_in), ("r1_line", r1_line, "手造历史"))


class TermRenames(unittest.TestCase):
    def test_a_regex_rename_applies_to_a_name_and_the_whole_string_rows_too(self) -> None:
        renames = list_check.term_renames_from_text(RENAMES_TEXT)
        self.assertEqual(len(renames), 2)
        self.assertEqual(list_check.renamed("实二十：甲段 0 L8：旧称丙", renames), "实二十：乙段名 L8：新称丙")
        self.assertEqual(list_check.renamed("第 1 甲段 0 号", renames), "第 1 甲段 0 号")


def list_text(rows: list[SelectedRow]) -> str:
    return "".join("\t".join((str(row.number), str(row.slice_number), row.file, str(row.group_index), row.name, row.line_digest_prefix)) + "\n"
                   for row in rows)


class AfterLists(unittest.TestCase):
    """名单改后（第六节）：手造四行（两张表各两行），撤第 2 行（丙）、改引第 3 行（乙），两个目录各写一份改后的名单。"""

    def run_after_check(self, step_8_rows: list[SelectedRow], step_4_only_rows: list[SelectedRow]) -> tuple[bool, str]:
        before = {list_check.STEP_8_LIST_BEFORE: [selected(1, "一号"), selected(2, "二号")],
                  list_check.STEP_4_ONLY_LIST_BEFORE: [selected(3, "三号旧名"), selected(4, "四号")]}
        gone = list_check.Classification(before[list_check.STEP_8_LIST_BEFORE][1], "bing", "registration_line", None, "-", 0, 0, False,
                                         (), (), "二号", "-")
        retargeted = list_check.Classification(before[list_check.STEP_4_ONLY_LIST_BEFORE][0], "yi", "registration_line", None, "-", 0, 0,
                                               False, ("三号新名",), ("三号新名",), "三号旧名", "三号新名")
        worktree = list_check.table_lines_from_text("\n".join(table_line(name) for name in ("一号", "三号新名", "四号")))
        with tempfile.TemporaryDirectory() as root:
            for directory in list_check.LIST_DIRECTORIES:
                os.makedirs(os.path.join(root, directory))
                for name, rows in ((list_check.STEP_8_LIST, step_8_rows), (list_check.STEP_4_ONLY_LIST, step_4_only_rows)):
                    with open(os.path.join(root, directory, name), "w", encoding="utf-8") as handle:
                        handle.write(list_text(rows))
            output = os.path.join(root, "out")
            lines = ResultLines(output)
            passed = list_check.after_lines(lines, root, before, {2: gone, 3: retargeted}, worktree)
            lines.finish()
            with open(output, encoding="utf-8") as handle:
                return passed, handle.read()

    def test_lists_with_the_gone_row_dropped_and_the_renamed_row_retargeted_pass(self) -> None:
        passed, text = self.run_after_check([selected(1, "一号")], [selected(3, "三号新名"), selected(4, "四号")])
        self.assertTrue(passed, text)
        self.assertIn("name=after_lists directories_identical=true rows=3 withdrawn_rows=2 retargeted_rows=3 unclassified_rows=none "
                      "row_set_equals_before_minus_withdrawn=true numbers_unique=true retargeted_names_are_the_chosen_ones=true "
                      "items_not_hit_once=none passed=true", text)

    def test_lists_that_keep_the_gone_row_fail(self) -> None:
        passed, text = self.run_after_check([selected(1, "一号"), selected(2, "二号")], [selected(3, "三号新名"), selected(4, "四号")])
        self.assertFalse(passed)
        self.assertIn("row_set_equals_before_minus_withdrawn=false", text)

    def test_lists_that_keep_the_old_name_fail(self) -> None:
        passed, text = self.run_after_check([selected(1, "一号")], [selected(3, "三号旧名"), selected(4, "四号")])
        self.assertFalse(passed)
        self.assertIn("retargeted_names_are_the_chosen_ones=false items_not_hit_once=3 passed=false", text)

    def test_the_expected_text_of_the_lists_keeps_the_sixth_column_and_renumbers_nothing(self) -> None:
        before = {list_check.STEP_8_LIST_BEFORE: [selected(1, "一号", "aaaaaaaaaaaa"), selected(2, "二号")],
                  list_check.STEP_4_ONLY_LIST_BEFORE: [selected(3, "三号旧名", "bbbbbbbbbbbb")]}
        retargeted = list_check.Classification(before[list_check.STEP_4_ONLY_LIST_BEFORE][0], "yi", "registration_line", None, "-", 0, 0,
                                               False, ("三号新名",), (), "三号旧名", "三号新名")
        gone = list_check.Classification(before[list_check.STEP_8_LIST_BEFORE][1], "jia", "registration_line", None, "-", 1, 0, True,
                                         (), (), "二号", "-")
        expected = list_check.expected_after_lists(before, {2: gone, 3: retargeted})
        self.assertEqual(expected, {list_check.STEP_8_LIST: f"1\t2\t{FILE}\t0\t一号\taaaaaaaaaaaa\n",
                                    list_check.STEP_4_ONLY_LIST: f"3\t2\t{FILE}\t0\t三号新名\tbbbbbbbbbbbb\n"})


class WholeTableRows(unittest.TestCase):
    def test_the_row_set_is_the_union_of_the_first_columns_with_gaps_kept(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            for name, rows in ((list_check.STEP_8_LIST, [selected(1, "一号"), selected(3, "三号")]),
                               (list_check.STEP_4_ONLY_LIST, [selected(118, "扩表一号"), selected(120, "扩表三号")])):
                with open(os.path.join(root, name), "w", encoding="utf-8") as handle:
                    handle.write(list_text(rows))
            self.assertEqual(list_check.whole_table_row_numbers(root), (1, 3, 118, 120))


if __name__ == "__main__":
    unittest.main()
