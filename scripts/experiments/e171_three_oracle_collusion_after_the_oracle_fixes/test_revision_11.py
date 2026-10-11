"""E171 第 1 次重跑·修订 11（按输入复用）的单测：研究一侧变异表 scripts/mutations/e171_three_oracle_collusion_after_the_oracle_fixes_revision_11.tsv
（RV1–RV15，登记修订 11 第 11.8 节）经 mutation_driver.py --unit-test-module test_revision_11 跑。
每条用例一个场景、自己建自己删临时目录；调 reuse_library.py 与 segment_one_units_revision_11.py 自证的那几格（假树、假装置、假驱动行、假单元脚本，
另有一格起真的修订 11 单元脚本取用一份造好的条目），不起装置、不编、不连别的台。"""
from __future__ import annotations

import shutil
import tempfile
import unittest

import reuse_library as reuse
import segment_one_units_revision_11 as units_layer


class CellCase(unittest.TestCase):
    """跑一格：格函数收一个临时目录，交回 (过没过, 细节)。"""

    def run_cell(self, cell) -> None:
        scratch = tempfile.mkdtemp(prefix="e171-r11-test-")
        try:
            passed, detail = cell(scratch)
        finally:
            shutil.rmtree(scratch, ignore_errors=True)
        self.assertTrue(passed, detail)


class KeySectionTests(CellCase):
    """RP3：十节各有一格，只改那一节对应的一样输入，第二份不该取到第一份的条目；改坏之后内容相同的那一格要取到。"""

    def test_a_different_unit_kind_does_not_reuse(self) -> None:
        self.run_cell(reuse.cell_rp3_kind)

    def test_a_different_toolchain_does_not_reuse(self) -> None:
        self.run_cell(reuse.cell_rp3_build)

    def test_a_changed_closure_file_does_not_reuse(self) -> None:
        self.run_cell(reuse.cell_rp3_closure)

    def test_the_same_source_after_the_mutation_reuses(self) -> None:
        self.run_cell(reuse.cell_rp3_closure_mutated_content)

    def test_a_set_environment_variable_neither_matches_nor_looks_up(self) -> None:
        self.run_cell(reuse.cell_rp3_env)

    def test_a_different_selected_row_line_does_not_reuse(self) -> None:
        self.run_cell(reuse.cell_rp3_mutation)

    def test_a_different_positive_control_does_not_reuse(self) -> None:
        self.run_cell(reuse.cell_rp3_args)

    def test_a_different_row_time_limit_does_not_reuse(self) -> None:
        self.run_cell(reuse.cell_rp3_limit)

    def test_a_changed_upstream_file_does_not_reuse(self) -> None:
        self.run_cell(reuse.cell_rp3_upstream)

    def test_a_changed_driver_module_does_not_reuse(self) -> None:
        self.run_cell(reuse.cell_rp3_research)

    def test_a_different_python_does_not_reuse(self) -> None:
        self.run_cell(reuse.cell_rp3_python)


class LibraryTests(CellCase):
    """RP4–RP8：续跑只补没判的、原子入库、RC4、RC1–RC3、RC5。"""

    def test_a_fetch_cut_in_the_middle_is_fetched_again_and_nothing_else_is_redone(self) -> None:
        self.run_cell(reuse.cell_rp4_resume_fetch)

    def test_a_killed_insert_leaves_no_half_entry(self) -> None:
        self.run_cell(reuse.cell_rp5_atomic_insert)

    def test_an_extra_file_in_the_compiled_dep_info_triggers_s18(self) -> None:
        self.run_cell(reuse.cell_rp6_closure_extra_file)

    def test_a_path_outside_the_tree_in_the_compiled_dep_info_triggers_s18(self) -> None:
        self.run_cell(reuse.cell_rp6_closure_outside_the_tree)

    def test_one_more_byte_in_the_export_of_the_same_key_triggers_s18(self) -> None:
        self.run_cell(reuse.cell_rp7_check_mismatch)

    def test_a_corrupted_entry_is_quarantined_and_run_fresh(self) -> None:
        self.run_cell(reuse.cell_rp8_corrupted_entry)


class TwoPassTests(CellCase):
    """RP1：复用关、开各一遍，永远现跑与不入库的几类照规矩，读数与判法两遍相同，账本 N、M 对。"""

    def test_the_second_pass_reuses_everything_except_the_fresh_always_and_the_uninsertable(self) -> None:
        self.run_cell(reuse.cell_rp1_two_passes)


class UnitsLayerTests(CellCase):
    """第 0–10 步单元编排层：取来的单元判法照跑、判红照停；真的单元脚本取用一份条目。"""

    def test_reused_units_are_judged_and_a_red_judgement_stops_the_run(self) -> None:
        self.run_cell(units_layer.cell_reuse_end_to_end)

    def test_the_real_unit_script_fetches_an_entry_and_judges_it(self) -> None:
        self.run_cell(units_layer.cell_unit_script_fetch)


if __name__ == "__main__":
    unittest.main()
