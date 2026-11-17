"""E171 第 1 次重跑·修订 11（按输入复用）的单测：研究一侧变异表 scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_revision_11.tsv
经 mutation_driver.py --unit-test-module test_revision_11 跑。复用那一层照 reuse_library.py 文件头「按项判定」：键经 long_run_item_verdicts 算、判定只在 prune-server 那一份。
每条用例一个场景、自己建自己删临时目录；调 reuse_library.py 与 segment_one_units_revision_11.py 自证的那几格（假仓、假树、假装置、假驱动行、假单元脚本，
另有一格起真的修订 11 单元脚本取用一份造好的状态目录），不起装置、不编、不连别的台。
算键、读写判定库的用例只在临时 prune-server 上跑（scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service），
不是临时服务那一条判错、点名出路，不写真服务。"""
from __future__ import annotations

import os
import shutil
import tempfile
import unittest

import reuse_library as reuse
import segment_one_units_revision_11 as units_layer


class CellCase(unittest.TestCase):
    """跑一格：格函数收一个临时目录，交回 (过没过, 细节)。"""

    def run_cell(self, cell) -> None:
        problem = reuse.prune_server_client.selftest_service_problem()
        self.assertEqual(problem, "", f"格里算键、读写判定库，只在临时 prune-server 上跑：{reuse.prune_server_client.SELFTEST_SERVICE_REMEDY}")
        scratch = tempfile.mkdtemp(prefix="e171-r11-test-")
        try:
            passed, detail = cell(scratch)
        finally:
            shutil.rmtree(scratch, ignore_errors=True)
        self.assertTrue(passed, detail)


class RequestTests(CellCase):
    """请求：只改一样会改结局的输入，键就不同；路径、在副本还是主树不进键；这一层自己拒算的几类。"""

    def test_a_different_unit_kind_has_another_key(self) -> None:
        self.run_cell(reuse.cell_request_kind)

    def test_different_device_arguments_have_another_key(self) -> None:
        self.run_cell(reuse.cell_request_arguments)

    def test_a_different_row_time_limit_has_another_key(self) -> None:
        self.run_cell(reuse.cell_request_limit)

    def test_a_changed_upstream_file_has_another_key(self) -> None:
        self.run_cell(reuse.cell_request_upstream)

    def test_a_changed_selected_row_line_or_mutation_line_has_another_key(self) -> None:
        self.run_cell(reuse.cell_request_mutation)

    def test_a_changed_judging_function_has_another_key(self) -> None:
        self.run_cell(reuse.cell_request_judge_closure)

    def test_a_changed_freeze_reference_has_another_key(self) -> None:
        self.run_cell(reuse.cell_request_freeze_reference)

    def test_a_set_environment_variable_read_by_the_tree_is_refused(self) -> None:
        self.run_cell(reuse.cell_request_environment_set)

    def test_a_mutation_touching_the_module_set_is_refused(self) -> None:
        self.run_cell(reuse.cell_request_module_set_refused)

    def test_the_keys_do_not_depend_on_where_the_tree_is(self) -> None:
        self.run_cell(reuse.cell_key_independent_of_location)


class ValueTests(CellCase):
    """值：确定的打法、取用不覆盖；读数相等的比法；入库条件。"""

    def test_the_same_reading_packs_to_the_same_bytes(self) -> None:
        self.run_cell(reuse.cell_packing_deterministic)

    def test_fetching_refuses_existing_targets_and_missing_roles(self) -> None:
        self.run_cell(reuse.cell_fetch_refuses_existing_targets)

    def test_reading_equality_ignores_timing_and_sees_one_more_byte(self) -> None:
        self.run_cell(reuse.cell_reading_equality)

    def test_timeouts_wrapper_outcomes_and_substituted_rows_are_not_green(self) -> None:
        self.run_cell(reuse.cell_insert_conditions)


class CheckTests(CellCase):
    """RC1 强制现判那一条、RC2 second-run：读数与库里那一份不同就是 S18，那一把键撤成红。"""

    def test_a_forced_rejudge_with_another_reading_triggers_s18(self) -> None:
        self.run_cell(reuse.cell_forced_reading_check)

    def test_a_second_run_with_another_reading_triggers_s18(self) -> None:
        self.run_cell(reuse.cell_second_run_check)


class TwoPassTests(CellCase):
    """RP1：库空、库里有各一遍，拒算与判红的几类照跑，读数与判法两遍相同，账本 N、M 对。"""

    def test_the_second_pass_reuses_everything_except_the_refused_and_the_red(self) -> None:
        self.run_cell(reuse.cell_rp1_two_passes)


class UnitsLayerTests(CellCase):
    """第 0–10 步单元编排层：取来的单元判法照跑、判红照停；真的单元脚本取用一份状态目录里的那一项。"""

    def test_reused_units_are_judged_and_a_red_judgement_stops_the_run(self) -> None:
        self.run_cell(units_layer.cell_reuse_end_to_end)

    def test_the_real_unit_script_fetches_an_item_and_judges_it(self) -> None:
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


if __name__ == "__main__":
    unittest.main()
