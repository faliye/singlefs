"""E171 第 1 次重跑·修订 11（按输入复用）的单测：研究一侧变异表 scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_revision_11.tsv
（RV1–RV15，登记修订 11 第 11.8 节）经 mutation_driver.py --unit-test-module test_revision_11 跑；库根两台同一那几条（第 2 次重跑登记修订 4）
另由 scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_current_state_library_root.tsv 带它跑。
每条用例一个场景、自己建自己删临时目录；调 reuse_library.py 与 segment_one_units_revision_11.py 自证的那几格（假树、假装置、假驱动行、假单元脚本，
另有一格起真的修订 11 单元脚本取用一份造好的条目），不起装置、不编、不连别的台。
复用库在 prune-server 里（reuse_library.py 文件头「库」）：跑格的用例只在临时 prune-server 上跑（scripts/prune-server/prune_server_client.py 文件头「命令行」
--with-temporary-service），不是临时服务那一条判错、点名出路，不写真服务。"""
from __future__ import annotations

import os
import shutil
import tempfile
import unittest

import reuse_library as reuse
import reuse_library_root as library_roots
import segment_one_units_revision_11 as units_layer


class CellCase(unittest.TestCase):
    """跑一格：格函数收一个临时目录，交回 (过没过, 细节)。"""

    def run_cell(self, cell) -> None:
        problem = reuse.prune_server_client.selftest_service_problem()
        self.assertEqual(problem, "", f"格里读写复用库，只在临时 prune-server 上跑：{reuse.prune_server_client.SELFTEST_SERVICE_REMEDY}")
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


class FurnishedWorldTests(unittest.TestCase):
    """两份逐行编排自证补假仓（furnish_rows_world）：假仓里的冻结参照是指向真仓 research/results/ 下产物的链接时，链悬空（产物还没生成）也不往真仓写。"""

    def test_a_dangling_reference_link_is_not_written_through(self) -> None:
        with tempfile.TemporaryDirectory(prefix="e171-r11-furnish-") as root:
            repo, tree, real_results = os.path.join(root, "repo"), os.path.join(root, "repo", "tree"), os.path.join(root, "real", "research", "results")
            target = os.path.join(real_results, os.path.basename(reuse.FREEZE_REFERENCE_RELATIVE))
            link = os.path.join(repo, reuse.FREEZE_REFERENCE_RELATIVE)
            os.makedirs(os.path.dirname(link))
            os.makedirs(real_results)
            os.symlink(target, link)
            reuse.furnish_rows_world(repo, tree, (3,))
            self.assertFalse(os.path.exists(target), "悬空的参照链被写穿了：真仓里多了一份假产物")
            self.assertTrue(os.path.islink(link))


class LibraryRootTests(unittest.TestCase):
    """H30 的库根派到两台里哪一台都是同一个绝对路径（reuse_library_root.py，第 2 次重跑登记 research/prompts/e171-r2-prereg.md 修订 4）。
    两份环境数据根相同、家目录不同：第二台的家目录是另一个。"""
    THIS_HOST = {"SINGLEFS_DATA_ROOT": "/shared/singlefs-data", "HOME": "/home/this-host"}
    SECOND_HOST = {"SINGLEFS_DATA_ROOT": "/shared/singlefs-data", "HOME": "/home/second-host"}
    UNDER_HOME = "~/e171-reuse-library-under-the-home-directory"   # 经 ~ 取的任一路径；不写数据根下那一份的样子（门禁 code-tooling 的 shared-data-root-literals 格）

    def test_an_unfilled_library_root_is_the_same_data_root_path_on_both_hosts(self) -> None:
        this_root, this_problem = library_roots.library_root_of({}, self.THIS_HOST)
        second_root, second_problem = library_roots.library_root_of({"library_root": None}, self.SECOND_HOST)
        self.assertEqual((this_problem, second_problem), ("", ""))
        self.assertEqual(this_root, second_root)
        self.assertEqual(this_root, os.path.join("/shared/singlefs-data", "reuse", library_roots.LIBRARY_NAME))

    def test_a_library_root_under_the_home_directory_is_refused(self) -> None:
        root, problem = library_roots.library_root_of({"library_root": self.UNDER_HOME}, self.SECOND_HOST)
        self.assertIn("经 ~ 或相对路径取", problem)
        self.assertNotIn("/home/second-host", root)
        with self.assertRaises(reuse.RunCondition):
            library_roots.require_library_root({"library_root": self.UNDER_HOME}, self.SECOND_HOST)

    def test_an_absolute_library_root_is_kept(self) -> None:
        self.assertEqual(library_roots.library_root_of({"library_root": "/shared/singlefs-data/reuse/other/"}, self.THIS_HOST),
                         ("/shared/singlefs-data/reuse/other", ""))

    def test_the_units_layer_refuses_to_run_with_a_library_root_under_the_home_directory(self) -> None:
        with self.assertRaises(reuse.RunCondition):
            units_layer.configure("/unused", {"H30": {"library_root": self.UNDER_HOME}}, False)


if __name__ == "__main__":
    unittest.main()
