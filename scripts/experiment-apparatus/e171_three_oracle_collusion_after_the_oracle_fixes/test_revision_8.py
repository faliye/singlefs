"""E171 第 1 次重跑第一段·修订 8 那一趟几份装置的单测（研究一侧变异表 scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_revision_8.tsv
经 mutation_driver.py --unit-test-module test_revision_8 跑；第 2 次重跑修订 4 加的几条另由 scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_current_state_preconditions.tsv 带它跑）。每条用例一个场景、自己建自己删临时目录；只调纯函数与进程内的小件，不起装置、不编、不连别的台
（第 2 步那一条起修订 11 单元脚本的 PRINT_ONLY=1，只打命令、一条都不跑）。"""
from __future__ import annotations

import json
import os
import shutil
import subprocess
import tempfile
import types
import unittest

import contrast_revision_6_against_revision_8 as contrast
import frozen_tree_lines_revision_8 as judge
import revision_8_placeholders as placeholders
import rows_orchestrator_revision_8 as rows
import s2_compare as s2
import segment_one_units_revision_8 as units

HERE = os.path.dirname(os.path.abspath(__file__))


class TemporaryDirectoryCase(unittest.TestCase):
    def setUp(self) -> None:
        self.scratch = tempfile.mkdtemp(prefix="e171-r8-test-")

    def tearDown(self) -> None:
        shutil.rmtree(self.scratch, ignore_errors=True)

    def path(self, *parts: str) -> str:
        return os.path.join(self.scratch, *parts)


class FrozenTreeJudgeTests(TemporaryDirectoryCase):
    def test_the_announcement_line_of_the_second_rerun_is_read_from_the_compatibility_removal_record(self) -> None:
        # 第 2 次重跑登记第十一节末段：宣布行写进删兼容那一批的调度记录、行首「- E171 第 2 次重跑拍树宣布（<日期>）：」；第 1 次重跑那种行首不认
        text = "# 记录\n\n- E171 最终主线拍树宣布（2026-10-08）：旧的那一种\n- E171 第 2 次重跑拍树宣布（2026-11-03）：拍了\n"
        self.assertEqual(judge.announcement_lines(text), [(4, "- E171 第 2 次重跑拍树宣布（2026-11-03）：拍了")])
        self.assertEqual(judge.RECORDS_FILE, "records/2026-10-31-删兼容旧版本代码.md")

    def test_snapshot_table_difference_catches_a_content_change_on_the_same_path(self) -> None:
        self.assertEqual(judge.table_differences({"a.rs": "1", "b.rs": "2"}, {"a.rs": "1", "b.rs": "3"}), [("b.rs", "2", "3")])

    def test_block_with_one_context_line_off_does_not_apply_without_fuzz(self) -> None:
        base, middle, _ = judge.three_versions(self.scratch)
        shifted = judge.write_text(self.path("shifted.rs"), open(middle, encoding="utf-8").read().replace("line 17\n", "line 17 changed\n"))
        after = judge.write_text(self.path("after.rs"), open(shifted, encoding="utf-8").read().replace("line 19\n", "line 19\ninserted C\n"))
        fixture = judge.write_text(self.path("fuzzy.diff"), judge.unified_diff_with_labels(shifted, after, "s", "t"))
        self.assertEqual(judge.apply_blocks_in_order(middle, [fixture], self.scratch), (None, 0))

    def test_fixture_header_with_a_tab_and_stamp_after_the_label_is_flagged_and_labelled_one_is_clean(self) -> None:
        stamped = judge.write_text(self.path("stamped.diff"), "--- a\tstamp\n+++ b\tstamp\n@@ -1 +1 @@\n")
        clean = judge.write_text(self.path("clean.diff"), "--- a\n+++ b\n@@ -1 +1 @@\n")
        self.assertEqual(len(judge.fixture_header_problems(stamped)), 2)
        self.assertEqual(judge.fixture_header_problems(clean), [])

    def test_announcement_lines_are_numbered_from_one(self) -> None:
        self.assertEqual(judge.announcement_lines(f"# 记录\n{judge.ANNOUNCEMENT_PREFIX}（日期）：x\n"), [(2, f"{judge.ANNOUNCEMENT_PREFIX}（日期）：x")])

    def test_last_merged_after_sha256_wins_and_device_swap_overrides(self) -> None:
        first = judge.write_text(self.path("1.sha256"), f"{'a' * 64}  crates/x.rs\n")
        second = judge.write_text(self.path("2.sha256"), f"{'c' * 64}  crates/x.rs\n")
        self.assertEqual(judge.last_sha256_by_path([first, second], "d" * 64), {"crates/x.rs": "c" * 64, judge.DEVICE: "d" * 64})

    def test_class_a_check_that_does_not_hold_is_named(self) -> None:
        product = judge.fake_class_a_product({"new_pool_file_creation_root_writes": "false"})
        self.assertEqual(judge.class_a_problems(product, "\n".join(judge.CLASS_A_DEVICE_LINES)), ["new_pool_file_creation_root_writes:holds_false"])

    def test_a12_at_least_one_check_fails_on_zero_hits(self) -> None:
        tree = judge.fake_repository(self.path("tree"))
        name, relative, needle, _ = next(item for item in judge.GREPS if item[3] == "at_least_one")
        judge.write_text(os.path.join(tree, relative), open(os.path.join(tree, relative), encoding="utf-8").read().replace(needle, ""))
        self.assertEqual([check for check, *_, holds in judge.a12_results(tree) if not holds], [name])

    def test_known_difference_removed_but_leaving_another_change_does_not_hold(self) -> None:
        repository = judge.fake_repository(self.path("repo"))
        tree = judge.copy_as_tree(repository, self.path("tree"))
        device = os.path.join(repository, judge.DEVICE)
        with_arm = open(device, encoding="utf-8").read().replace("// device line 5\n", "// device line 5\n        ArmA => {}\n")
        judge.write_text(self.path("with-arm.rs"), with_arm)
        fixture = judge.write_text(self.path("known.diff"), judge.unified_diff_with_labels(os.path.join(tree, judge.DEVICE), self.path("with-arm.rs"), "t", "r"))
        judge.write_text(device, with_arm.replace("// device line 30\n", "// device line 30\n        ArmB => {}\n"))
        holds, _ = judge.s16_2(repository, {"tree": tree, "H5": {"known_repo_differences": [{"fixture": fixture}]}})
        self.assertFalse(holds)

    def test_blocks_that_apply_but_do_not_reach_the_tree_device_are_not_identical(self) -> None:
        base, middle, final = judge.three_versions(self.scratch)
        fixture = judge.write_text(self.path("b.diff"), judge.unified_diff_with_labels(base, middle, "a", "b"))
        os.environ["FROZEN_TREE_R8_SELFTEST"] = "1"
        os.environ["FROZEN_TREE_R8_EXPECTED_BASE_SHA256"] = judge.sha256_of_file(base)
        try:
            config = {"H5": {"base_device_copy": base, "blocks": [{"name": "x", "source_kind": "revision_9_constants", "source": "修订 9", "fixture": fixture,
                                                                    "fixture_sha256": judge.sha256_of_file(fixture)}]}}
            holds, _, summary = judge.s16_3(self.scratch, config, judge.sha256_of_file(final))
        finally:
            del os.environ["FROZEN_TREE_R8_SELFTEST"], os.environ["FROZEN_TREE_R8_EXPECTED_BASE_SHA256"]
        self.assertFalse(holds)
        self.assertIn("identical=false", summary)

    def test_step_0_triggers_s16_1_when_the_snapshot_record_says_the_tree_differed_from_the_repository(self) -> None:
        os.environ["FROZEN_TREE_R8_SELFTEST"] = "1"
        try:
            repository, tree, config_path = judge.fake_snapshot_world(self.path("world"))
            config = json.load(open(config_path, encoding="utf-8"))
            judge.write_text(os.path.join(repository, "crates/fake/src/row_42.rs"), "fn anchor_42() { 仓里多改了一行 }\n")
            snapshot = self.path("snapshot-differing.out")
            judge.at_snapshot(repository, config_path, snapshot)
            config["H3"].update(snapshot_product=snapshot, snapshot_product_sha256=judge.sha256_of_file(snapshot))
            judge.write_text(config_path, json.dumps(config))
            output = self.path("frozen-tree.out")
            judge.step_0(repository, config_path, output)
        finally:
            for name in ("FROZEN_TREE_R8_SELFTEST", "FROZEN_TREE_R8_EXPECTED_BASE_SHA256", "FROZEN_TREE_R8_EXPECTED_REVISION_6_DEVICE_SHA256"):
                os.environ.pop(name, None)
        self.assertIn("s16_1", judge.verdict_fields(output)["triggered"].split(","))

    def test_p1f_base_digest_is_the_third_column_of_its_bad_states_row(self) -> None:
        stream = self.path(judge.P1F_STREAM)
        judge.write_text(os.path.join(stream, "bad-expected.tsv"), f"3\t{judge.P1F_ROW_NAME}\tP\tF\n")
        judge.write_text(os.path.join(stream, "bad-states.tsv"), f"3\t22\t{'A' * 64}\t{'b' * 64}\t-\n")
        self.assertEqual(judge.p1f_base_digest(self.scratch), (3, "a" * 64))


class PlaceholderTests(TemporaryDirectoryCase):
    def h14_config(self, main_states: int) -> dict:
        product = judge.write_text(self.path("draft.out"), "E7RESULT name=s1 check=confirmed_loss_key_choices local=72 crates=72 holds=true\n"
                                   "E7RESULT name=s1 check=confirmed_loss_in_flight_writes local=24 crates=24 holds=true\n"
                                   f"E7RESULT name=s1 check=confirmed_loss_states local=1 crates={main_states} holds=true\n")
        return {"H6": {"draft_copy_stream_constants_product": product},
                "H14": {"key_choices": 72, "in_flight_writes": 24, "reverse_states": 72, "main_states": main_states, "piece_panic_states": [1152, 1152, 576]}}

    def test_h14_holds_when_main_states_is_key_choices_times_two_to_the_in_flight_writes(self) -> None:
        self.assertTrue(placeholders.check_h14(self.scratch, self.h14_config(72 * 2 ** 24), "start").holds)

    def test_h18_with_the_revision_6_landing_names_does_not_hold(self) -> None:
        config = {"H18": dict(zip(("peer_kit", "peer_snapshot", "peer_out"), placeholders.REVISION_6_PEER_NAMES))}
        self.assertFalse(placeholders.check_h18(self.scratch, config, "start").holds)

    def test_h20_saying_steps_3_and_9_go_to_other_hosts_does_not_hold(self) -> None:
        config = {"H20": {"steps_3_and_9_on_other_hosts": True, "why": "想分到第二台去跑"}}
        self.assertFalse(placeholders.check_h20(self.scratch, config, "start").holds)

    def test_h15_without_the_dumps_root_holds_when_the_comparator_product_is_there(self) -> None:
        judge.write_text(self.path("repo", "c.out"), "E7RESULT name=done emitted=0\n")
        h15 = {"comparator_product": "c.out", "dumps_root": None, "product_prefix": "p", "product_suffix": "-r6-r3", "rows_tag": "-resume2",
               "rows_finished": True, "stopped_at": None}
        outcome = placeholders.check_h15(self.path("repo"), {"H15": h15}, "contrast")
        self.assertTrue(outcome.filled and outcome.holds, outcome.detail)
        self.assertFalse(placeholders.check_h15(self.path("repo"), {"H15": dict(h15, dumps_root="gone")}, "contrast").holds)

    def test_dry_run_config_does_not_hold_at_start(self) -> None:
        self.assertFalse(placeholders.check_tree(self.scratch, {"dry_run": True, "tree": self.scratch}, "start").holds)


class UnitSchedulerTests(TemporaryDirectoryCase):
    def test_step_7_is_first_in_the_queue_even_when_another_unit_is_estimated_longer(self) -> None:
        longer = units.Unit("step-9", (), 10, 1, 99999)
        order = units.queue_order([longer, units.unit_table()["step-7"]])
        self.assertEqual([unit.name for unit in order], ["step-7", "step-9"])

    def test_downstream_of_step_7_includes_the_z1_unit_that_also_waits_for_step_9(self) -> None:
        self.assertEqual(units.downstream(units.unit_table(), {"step-7"}), {"step-7", "step-8", "step-9-z1"})

    def test_a_unit_does_not_fit_when_the_threads_run_out(self) -> None:
        table = units.unit_table()
        budget = units.Budget(25)
        self.assertFalse(units.fits(table["step-9"], [table["step-7"], table["step-3-o1-crash-judge"]], budget))
        self.assertTrue(units.fits(table["step-9"], [table["step-7"]], budget))

    def test_attempt_tags_start_empty_and_count_from_two(self) -> None:
        self.assertEqual([units.attempt_tag(number) for number in (1, 2, 3)], ["", "-attempt2", "-attempt3"])

    def test_resume_refuses_a_product_whose_line_count_changed_since_green(self) -> None:
        repo = self.path("repo")
        product = judge.write_text(os.path.join(repo, "research/results/p.out"), "a\nb\n")
        logs = self.path("logs")
        judge.write_text(os.path.join(logs, "sequence.log"), "2026-10-09 ✓ 单元 step-1 绿 attempt=1 tag=- 产物 research/results/p.out（2 行）\n")
        judge.write_text(os.path.join(logs, units.MANIFEST), f"step-1\t1\t{judge.sha256_of_file(product)}\tresearch/results/p.out\tproduct\n")
        settings = types.SimpleNamespace(repo=repo, logs=logs)
        records, manifest = units.green_records(settings), units.manifest_entries(settings)
        self.assertEqual(units.unit_problems(settings, "step-1", records, manifest), [])
        judge.write_text(product, "a\nb\nc\n")
        self.assertIn("product_line_count:research/results/p.out", units.unit_problems(settings, "step-1", records, manifest))

    def test_concurrency_line_names_a_unit_that_started_while_step_7_ran(self) -> None:
        item = types.SimpleNamespace(started=10.0, ended=20.0, unit=types.SimpleNamespace(name="step-7"))
        line = units.concurrency_line(item, {"step-7": (10.0, 20.0), "step-9": (12.0, 25.0), "step-1": (1.0, 5.0)})
        self.assertIn("units=step-9 ", line)


class RowsWrapperTests(TemporaryDirectoryCase):
    def test_existing_stop_file_stops_picking_new_rows(self) -> None:
        stop_file = judge.write_text(self.path("stop"), "红\n")
        rows.STATE["stop_file"] = stop_file
        try:
            queue = [rows.one.Job("s11", (242,), 1.0)]
            self.assertIsNone(rows.pick_job_unless_stopped(queue, 1, None))
        finally:
            rows.STATE["stop_file"] = None

    def test_changed_mutations_table_in_the_tree_stops_the_rows(self) -> None:
        tree = self.path("tree")
        judge.write_text(os.path.join(tree, judge.MUTATIONS_TABLE), "# 表\n")
        judge.write_text(os.path.join(tree, "Cargo.toml"), "[workspace]\n")   # 快照里要有文件：空集的树在核 H8 之前就当认不全拒掉
        context = types.SimpleNamespace(repo=self.scratch, tree="tree", snapshot_digest=judge.row_driver.snapshot_digest(tree)[0],
                                        research_root=self.scratch)
        rows.STATE["mutations_sha256"] = "0" * 64
        original = rows.ORIGINAL["snapshot_check"]
        rows.ORIGINAL["snapshot_check"] = lambda context: None
        try:
            with self.assertRaises(rows.one.Stopped):
                rows.snapshot_check_with_mutations_table(context)
        finally:
            rows.ORIGINAL["snapshot_check"] = original
            rows.STATE["mutations_sha256"] = None

    def context_for(self, tree_name: str) -> types.SimpleNamespace:
        """快照摘要写成一个对不上的值：认得全的树会走到 Z5 停，认不全的要在那之前拒掉。"""
        return types.SimpleNamespace(repo=self.scratch, tree=tree_name, snapshot_digest="0" * 64, research_root=self.scratch)

    def test_a_tree_without_snapshot_files_is_refused_as_unreadable_instead_of_judged_as_z5(self) -> None:
        judge.write_text(self.path("tree", judge.MUTATIONS_TABLE), "# 表\n")   # 只有 .tsv：快照是空集，摘要是空输入的 sha256
        with self.assertRaises(rows.one.RunCondition):
            rows.snapshot_check_with_mutations_table(self.context_for("tree"))

    def test_a_tree_that_was_never_taken_is_refused_as_unreadable_instead_of_judged_as_z5(self) -> None:
        with self.assertRaises(rows.one.RunCondition):
            rows.snapshot_check_with_mutations_table(self.context_for("never-taken"))

    def test_a_tree_with_snapshot_files_and_another_digest_still_stops_on_z5(self) -> None:
        judge.write_text(self.path("tree", "Cargo.toml"), "[workspace]\n")
        with self.assertRaises(rows.one.Stopped):
            rows.snapshot_check_with_mutations_table(self.context_for("tree"))


class H16RunConditionTests(TemporaryDirectoryCase):
    """逐行编排外层读 H16：缺一项就拒、点名缺哪几项（第 2 次重跑登记修订 4）。"""
    FULL_H16 = {"run_date": "2026-11-02", "run_suffix": "-r8", "dumps_root": "scripts/experiment-apparatus/target/fake-dumps", "logs_directory": "/fake/logs"}

    def test_h16_with_every_item_filled_misses_nothing(self) -> None:
        self.assertEqual(rows.h16_missing_items({"H16": dict(self.FULL_H16)}), [])

    def test_h16_without_the_logs_directory_key_names_it(self) -> None:
        h16 = {key: value for key, value in self.FULL_H16.items() if key != "logs_directory"}
        self.assertEqual(rows.h16_missing_items({"H16": h16}), ["logs_directory"])

    def test_h16_with_a_null_logs_directory_names_it(self) -> None:
        self.assertEqual(rows.h16_missing_items({"H16": dict(self.FULL_H16, logs_directory=None)}), ["logs_directory"])

    def test_the_rows_context_is_refused_when_the_logs_directory_is_null(self) -> None:
        config = {"tree": "tree", "H3": {"snapshot_digest": "0" * 64}, "H16": dict(self.FULL_H16, logs_directory=None)}
        with self.assertRaises(judge.RunConditionError) as raised:
            rows.context_of(config, self.scratch, "timing.out")
        self.assertIn("logs_directory", str(raised.exception))

    def test_the_rows_layer_plan_exits_78_before_configuring_when_h16_lacks_the_logs_directory(self) -> None:
        h16 = {key: value for key, value in self.FULL_H16.items() if key != "logs_directory"}
        config = judge.write_text(self.path("config.json"), json.dumps({"tree": "tree", "H16": h16}))
        configured = rows.one.snapshot_check
        code = rows.main(["plan", "--config", config, "--repo", self.scratch, "--baseline-timing", "timing.out", "--stop-file", self.path("stop")])
        self.assertEqual(code, rows.one.EXIT_RUN_CONDITION)
        self.assertIs(rows.one.snapshot_check, configured)   # 没装上这一层的几处：拒在 configure 之前


class S2CompareTests(TemporaryDirectoryCase):
    """第 2 步逐文件比（s2_compare.py，停机 S2）：E170 导出与本装置基线，流一、流二各九件。"""

    def export(self, name: str, streams: tuple[str, ...] = s2.S2_STREAMS) -> str:
        """一份导出：每条流八个整比的文件与 impl-verdicts.tsv（最后一列是耗时，各份不同）。"""
        root = self.path(name)
        for stream in streams:
            for file_name in s2.S2_WHOLE_FILES:
                judge.write_text(os.path.join(root, stream, file_name), f"{stream} {file_name}\n")
            judge.write_text(os.path.join(root, stream, s2.IMPLEMENTATION_VERDICTS_FILE), f"0\t1\tok\t{name}-ns\n1\t2\tred\t{name}-ns\n")
        return root

    def product_lines(self, e170: str, device: str) -> list[str]:
        output = self.path("s2.out")
        self.assertEqual(s2.main([e170, device, output]), 0)
        return open(output, encoding="utf-8").read().splitlines()

    def verdict_of(self, lines: list[str]) -> str:
        return [line for line in lines if line.startswith("E7RESULT name=verdict ")][0]

    def test_two_identical_exports_compare_eighteen_files_and_none_differ(self) -> None:
        lines = self.product_lines(self.export("e170"), self.export("device"))
        self.assertEqual(self.verdict_of(lines), "E7RESULT name=verdict s2_differing_files=0 s2_compared_files=18")
        self.assertEqual(lines[-1], "E7RESULT name=done emitted=21")

    def test_impl_verdicts_that_differ_only_in_the_timing_column_count_as_the_same(self) -> None:
        lines = self.product_lines(self.export("e170"), self.export("device"))
        verdict_lines = [line for line in lines if "file=impl-verdicts.tsv_without_timing" in line]
        self.assertEqual(len(verdict_lines), 2)
        self.assertTrue(all(line.endswith(" equal=true") for line in verdict_lines), verdict_lines)

    def test_impl_verdicts_that_differ_before_the_timing_column_count_as_different(self) -> None:
        device = self.export("device")
        judge.write_text(os.path.join(device, s2.S2_STREAMS[0], s2.IMPLEMENTATION_VERDICTS_FILE), "0\t1\tred\tdevice-ns\n1\t2\tred\tdevice-ns\n")
        self.assertEqual(self.verdict_of(self.product_lines(self.export("e170"), device)), "E7RESULT name=verdict s2_differing_files=1 s2_compared_files=18")

    def test_one_changed_byte_in_states_counts_one_differing_file(self) -> None:
        device = self.export("device")
        judge.write_text(os.path.join(device, s2.S2_STREAMS[1], "states.tsv"), f"{s2.S2_STREAMS[1]} states.tsw\n")
        lines = self.product_lines(self.export("e170"), device)
        self.assertEqual(self.verdict_of(lines), "E7RESULT name=verdict s2_differing_files=1 s2_compared_files=18")
        self.assertIn(f"E7RESULT name=s2 stream={s2.S2_STREAMS[1]} file=states.tsv", [line.split(" e170_sha256=")[0] for line in lines if line.endswith("equal=false")])

    def test_a_file_missing_on_both_sides_counts_as_different(self) -> None:
        e170, device = self.export("e170"), self.export("device")
        for root in (e170, device):
            os.remove(os.path.join(root, s2.S2_STREAMS[0], "in-place-subsets.tsv"))
        lines = self.product_lines(e170, device)
        self.assertIn(f"E7RESULT name=s2 stream={s2.S2_STREAMS[0]} file=in-place-subsets.tsv e170_sha256=absent device_sha256=absent equal=false", lines)
        self.assertEqual(self.verdict_of(lines), "E7RESULT name=verdict s2_differing_files=1 s2_compared_files=18")

    def test_the_third_stream_is_not_compared(self) -> None:
        streams = s2.STREAM_NAMES   # 三条流都导了，第三条两份不同
        e170, device = self.export("e170", streams), self.export("device", streams)
        judge.write_text(os.path.join(device, streams[2], "states.tsv"), "流三另一份\n")
        self.assertEqual(self.verdict_of(self.product_lines(e170, device)), "E7RESULT name=verdict s2_differing_files=0 s2_compared_files=18")

    def test_a_missing_export_directory_exits_78_and_writes_no_product(self) -> None:
        output = self.path("s2.out")
        self.assertEqual(s2.main([self.export("e170"), self.path("never-dumped"), output]), s2.EXIT_RUN_CONDITION)
        self.assertFalse(os.path.exists(output))

    def test_an_existing_product_is_not_overwritten_and_exits_2(self) -> None:
        output = judge.write_text(self.path("s2.out"), "上一次的产物\n")
        self.assertEqual(s2.main([self.export("e170"), self.export("device"), output]), s2.EXIT_USAGE)
        self.assertEqual(open(output, encoding="utf-8").read(), "上一次的产物\n")


class UnitScriptStepTwoTests(TemporaryDirectoryCase):
    """修订 11 单元脚本第 2 步：照修订 8 那一步起 E170 导出装置、再用研究目录里的 s2_compare.py 比（不再退 2）。"""

    def test_step_2_prints_the_e170_dump_and_the_s2_compare_of_the_research_directory(self) -> None:
        environment = dict(os.environ, REPO=self.scratch, RESEARCH=HERE, PRINT_ONLY="1", PREFIX="research/results/fake-s1", TAG="", SUFFIX="-r8",
                           DUMPS="scripts/experiment-apparatus/target/fake-dumps", E170_BIN="/bin/false", THREADS="1",
                           S2_BASELINE="s2-device-baseline-streams-1-2", UNIT_LOG=self.path("unit.log"))
        completed = subprocess.run(["bash", os.path.join(HERE, "segment_one_unit_revision_11.sh"), "step-2"], env=environment,
                                   capture_output=True, text=True, timeout=60)
        self.assertEqual(completed.returncode, 0, completed.stdout + completed.stderr)
        self.assertIn(f"{HERE}/s2_compare.py scripts/experiment-apparatus/target/fake-dumps/s2-e170-export", completed.stdout)
        self.assertIn("s2_differing_files=0 s2_compared_files=18", completed.stdout)


class ContrastTests(TemporaryDirectoryCase):
    def test_lines_of_the_same_kind_are_paired_by_their_identity_fields_not_by_order(self) -> None:
        old = ["E7RESULT name=q1_row row=15 stream=a batch=1 O2=1", "E7RESULT name=q1_row row=23 stream=a batch=1 O2=0"]
        new = ["E7RESULT name=q1_row row=23 stream=a batch=1 O2=0", "E7RESULT name=q1_row row=15 stream=a batch=1 O2=1"]
        self.assertFalse([cell for cell in contrast.contrast_cells(old, new) if cell.endswith(" same=false")])

    def test_the_step_comparisons_and_the_tree_table_are_reported_as_not_reconstructible_without_a_count(self) -> None:
        repo, revision_6, revision_8 = contrast.fake_runs(self.scratch, revision_6_dumps=False)
        output = self.path("contrast.out")
        contrast.write_contrast(repo, revision_6, revision_8, output)
        lines = open(output, encoding="utf-8").read().splitlines()
        missing = [line for line in lines if "name=contrast_s15 " in line or "name=contrast_tree_file " in line]
        self.assertEqual(len(missing), len(contrast.S15_STEPS) + 1)
        self.assertTrue(all(" status=not_reconstructible reason=revision_6_tree_and_exports_not_reconstructible" in line for line in missing))
        summary = next(line for line in lines if "name=contrast_summary " in line)
        self.assertIn(" s15_lines=not_reconstructible tree_files_differing=not_reconstructible ", summary)

    def test_h15_without_the_dumps_root_still_has_everything_the_cells_and_row_times_need(self) -> None:
        filled = {"comparator_product": "c.out", "dumps_root": None, "product_prefix": "p", "product_suffix": "-r6-r3", "rows_tag": "-resume2"}
        self.assertEqual(contrast.missing_h15_keys(filled), [])
        self.assertEqual(contrast.missing_h15_keys(dict(filled, comparator_product=None)), ["comparator_product"])

    def test_row_times_without_the_ledger_are_not_put_on_this_host(self) -> None:
        repo, revision_6, _ = contrast.fake_runs(self.scratch, revision_6_dumps=False)
        hosts = {line.split(" row=")[1].split(" ")[0]: line.split(" host=")[1].split(" ")[0]
                 for line in contrast.row_time_lines(repo, revision_6) if line.startswith("name=contrast_row_time ")}
        self.assertEqual(hosts, {"0": "ledger_absent", "15": "ledger_absent"})

    def test_an_absent_row_product_is_named_instead_of_counting_as_no_rows(self) -> None:
        repo, revision_6, _ = contrast.fake_runs(self.scratch)
        present = {line.split(" stem=")[1].split(" ")[0]: line.endswith("present=true")
                   for line in contrast.row_time_lines(repo, revision_6) if line.startswith("name=contrast_row_product ")}
        self.assertEqual(present, {stem: stem == "rows-driver" for stem in contrast.ROW_PRODUCT_STEMS})

    def test_row_time_takes_the_host_from_the_ledger(self) -> None:
        repo, revision_6, _ = contrast.fake_runs(self.scratch)
        hosts = {line.split(" row=")[1].split(" ")[0]: line.split(" host=")[1].split(" ")[0]
                 for line in contrast.row_time_lines(repo, revision_6) if line.startswith("name=contrast_row_time ")}
        self.assertEqual(hosts, {"0": "2", "15": "1"})


if __name__ == "__main__":
    unittest.main()
