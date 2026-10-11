"""E171 第 1 次重跑·修订 10（修订 8 那一趟的第二段整表）几份装置的单测（研究一侧变异表
scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_revision_10.tsv 经 mutation_driver.py --unit-test-module test_revision_10 跑）。
每条用例一个场景、自己建自己删临时目录；只调纯函数与进程内的小件，不起装置、不编、不连别的台、不起编排器。"""
from __future__ import annotations

import os
import shutil
import tempfile
import types
import unittest

import contrast_segment_two_revision_10 as contrast
import frozen_tree_lines_revision_8 as judge
import revision_10_placeholders as placeholders
import rows_orchestrator_segment_two_revision_10 as layer

STARTED = f"x {placeholders.ROWS_ORCHESTRATOR_STARTED}（…）"


class TemporaryDirectoryCase(unittest.TestCase):
    def setUp(self) -> None:
        self.scratch = tempfile.mkdtemp(prefix="e171-r10-test-")

    def tearDown(self) -> None:
        shutil.rmtree(self.scratch, ignore_errors=True)

    def path(self, *parts: str) -> str:
        return os.path.join(self.scratch, *parts)

    def world(self, name: str = "world", **options) -> tuple[str, dict, dict, placeholders.SegmentOne]:
        repo, config, revision_8_config = placeholders.fake_world(self.path(name), **options)
        return repo, config, revision_8_config, placeholders.segment_one_of(revision_8_config)

    def append_log(self, segment_one: placeholders.SegmentOne, *lines: str) -> None:
        with open(segment_one.sequence_log, "a", encoding="utf-8") as handle:
            handle.write("\n".join(lines) + "\n")


class SegmentOneLogTests(TemporaryDirectoryCase):
    def test_rows_orchestrator_counts_as_running_only_between_its_start_line_and_an_end_line(self) -> None:
        self.assertFalse(placeholders.rows_orchestrator_running([]))
        self.assertTrue(placeholders.rows_orchestrator_running([STARTED]))
        self.assertFalse(placeholders.rows_orchestrator_running([STARTED, "x   逐行编排退 0"]))
        self.assertTrue(placeholders.rows_orchestrator_running([STARTED, "x   逐行编排退 0", STARTED]))

    def test_research_mutations_gate_follows_the_research_side_differs_field_of_step_0(self) -> None:
        for differs, required in (("true", True), ("false", False)):
            repo, _, _, segment_one = self.world(differs, research_side_differs=differs)
            self.assertEqual(placeholders.research_mutations_required(repo, segment_one, placeholders.green_units(segment_one)), required, differs)

    def test_gate_names_a_green_line_whose_product_line_count_changed(self) -> None:
        repo, _, _, segment_one = self.world(step_1_line_count=4)
        self.assertEqual([problem.split(":")[:2] for problem in placeholders.start_gate_problems(repo, segment_one)], [["step-1", "line_count"]])

    def test_baseline_timing_follows_the_tag_of_the_last_green_line_of_step_1(self) -> None:
        repo, _, _, segment_one = self.world()
        relative = segment_one.timing(f"{placeholders.STEP_1_STEM}-attempt2")
        judge.write_text(os.path.join(repo, relative), "E7TIMING name=stream_time stream=a elapsed_ns=1000000000\nE7TIMING name=process_wall seconds=900.000 exit=0\n")
        self.append_log(segment_one, placeholders.green_line("step-1", [relative], {relative: 2}, tag="-attempt2"))
        timing = placeholders.resolve_baseline_timing(repo, segment_one)
        self.assertEqual((timing.tag, timing.path, timing.row_time_limit_seconds), ("-attempt2", relative, 2700))

    def test_row_time_limit_is_three_baselines_rounded_up_with_an_1800_second_floor(self) -> None:
        for wall, limit in (("700.200", 2101), ("100.500", 1800)):
            repo, _, _, segment_one = self.world(wall, process_wall=wall)
            self.assertEqual(placeholders.resolve_baseline_timing(repo, segment_one).row_time_limit_seconds, limit, wall)

    def test_local_gate_needs_step_7_green_and_the_rows_orchestrator_started(self) -> None:
        _, _, _, segment_one = self.world()
        self.assertFalse(placeholders.local_gate(segment_one)[0])
        self.append_log(segment_one, STARTED)
        self.assertFalse(placeholders.local_gate(segment_one)[0])
        self.append_log(segment_one, placeholders.green_line("step-7", [], {}))
        self.assertTrue(placeholders.local_gate(segment_one)[0])

    def test_compare_baselines_carry_the_tag_of_each_green_unit(self) -> None:
        _, _, _, segment_one = self.world()
        self.append_log(segment_one, placeholders.green_line("step-7", [], {}, tag="-attempt3"), placeholders.green_line("step-9", [], {}))
        found = placeholders.compare_baselines(segment_one)
        self.assertEqual(found["stability_baseline_run1"], os.path.join(segment_one.dumps, "baseline-run1-attempt3"))
        self.assertEqual(found["stability_baseline_run2"], os.path.join(segment_one.dumps, "baseline-run2"))


class ConfigCheckTests(TemporaryDirectoryCase):
    def h23_holds(self, dumps_root: str, segment_one_dumps: str | None = None) -> bool:
        repo, config, revision_8_config, _ = self.world()
        config["H23"]["dumps_root"] = dumps_root
        if segment_one_dumps is not None:
            revision_8_config["H16"]["dumps_root"] = segment_one_dumps
        return placeholders.check_h23(repo, config, revision_8_config, "start").holds

    def test_h23_refuses_a_dumps_root_named_like_revision_6_segment_two_with_a_suffix(self) -> None:
        self.assertFalse(self.h23_holds(placeholders.REVISION_6_SEGMENT_TWO_DUMPS + "-r2"))

    def test_h23_refuses_a_dumps_root_that_contains_the_segment_one_dumps_root(self) -> None:
        self.assertFalse(self.h23_holds("scripts/experiments/target/e171-r1-dumps/combined", "scripts/experiments/target/e171-r1-dumps/combined/segment-one"))

    def test_h23_accepts_a_fresh_dumps_root_beside_the_others(self) -> None:
        self.assertTrue(self.h23_holds("scripts/experiments/target/e171-r1-dumps/revision-8-segment-2"))

    def test_h24_refuses_a_label_prefix_that_an_older_label_starts_with(self) -> None:
        _, config, revision_8_config, _ = self.world()
        config["H24"]["label_prefix"] = "e171s"
        self.assertFalse(placeholders.check_h24(self.scratch, config, revision_8_config, "start").holds)

    def test_h25_names_a_required_file_that_was_not_registered(self) -> None:
        files = {path: "0" * 64 for path in placeholders.H25_REQUIRED_FILES[1:]}
        self.assertIn("该登没登的 1 份", placeholders.check_h25(self.scratch, {"H25": {"files": files}}, {}, "start").detail)

    def test_compare_command_points_at_this_runs_products_and_the_tree_table(self) -> None:
        repo, config, revision_8_config, segment_one = self.world()
        self.append_log(segment_one, placeholders.green_line("step-7", [], {}), placeholders.green_line("step-9", [], {}))
        command = placeholders.compare_command(repo, config, revision_8_config)
        self.assertIn("--product-suffix=-r8.out", command)
        self.assertEqual(command[command.index("--mutations-table") + 1], "scripts/experiments/target/fake-tree/crates/mutations.tsv")
        self.assertEqual(command[command.index("--rows-root") + 1], "scripts/experiments/target/e171-r1-dumps/fake-segment-two/segment2-rows")


class LayerTests(TemporaryDirectoryCase):
    def segment_one(self) -> placeholders.SegmentOne:
        return placeholders.SegmentOne("tree", "d" * 64, "e" * 64, "2026-10-09", "-r8", "dumps", self.path("logs"))

    def write_shard(self, attempt: str, label: str, record: dict, finished: bool = False) -> None:
        directory = self.path("dumps", "segment1-rows-shards", attempt, label)
        judge.write_text(os.path.join(directory, "shard.json"), layer.json.dumps(record))
        if finished:
            judge.write_text(os.path.join(directory, "exit"), "0\n")

    def test_snapshot_check_stops_when_the_tree_table_is_not_h8(self) -> None:
        tree = self.path("repo", "tree")
        judge.write_text(os.path.join(tree, "Cargo.toml"), "[workspace]\n")
        judge.write_text(os.path.join(tree, judge.MUTATIONS_TABLE), "# 表\n")
        context = types.SimpleNamespace(repo=self.path("repo"), tree="tree", snapshot_digest=judge.row_driver.snapshot_digest(tree)[0],
                                        research_root=layer.two.REAL_REPOSITORY)
        layer.STATE["tree_table_sha256_h8"] = judge.sha256_of_file(os.path.join(tree, judge.MUTATIONS_TABLE))
        layer.snapshot_check_with_the_tree_table(context)
        judge.write_text(os.path.join(tree, judge.MUTATIONS_TABLE), "# 拍完又改了\n")
        with self.assertRaises(layer.one.Stopped):
            layer.snapshot_check_with_the_tree_table(context)

    def test_local_gate_zeroes_only_the_local_host_slots_until_it_opens(self) -> None:
        _, _, _, segment_one = self.world()
        hosts = layer.one.two_hosts()
        resources, _ = layer.one.resources_from_json({"hosts": [layer.one.resources_entry(1, 10, 25, 25), layer.one.resources_entry(2, 10, 25, 25)]}, hosts)
        layer.STATE["segment_one"] = segment_one
        closed = [(item.number, item.slots) for item in layer.allocate_behind_the_local_gate(hosts, resources)]
        self.append_log(segment_one, placeholders.green_line("step-7", [], {}), STARTED)
        opened = [(item.number, item.slots) for item in layer.allocate_behind_the_local_gate(hosts, resources)]
        self.assertEqual((closed, opened), ([(1, 0), (2, 2)], [(1, 2), (2, 2)]))

    def test_running_rows_come_from_the_latest_attempt_without_an_exit_file_and_skip_carry(self) -> None:
        self.write_shard("attempt-1", "s12-row-15-h1", {"kind": "local", "step": "s12", "rows": [15]})
        self.write_shard("attempt-2", "s12-row-0-h1", {"kind": "local", "step": "s12", "rows": [0]})
        self.write_shard("attempt-2", "s11-row-269-h2", {"kind": "peer", "step": "s11", "rows": [269]}, finished=True)
        self.write_shard("attempt-2", "s12-carry", {"kind": "carry", "step": "s12", "rows": [23]})
        self.assertEqual(layer.running_segment_one_rows(self.path("dumps", "segment1-rows-shards")), {("s12", 0)})

    def test_waiting_rows_skip_done_rows_and_read_x_from_the_cross_host_root(self) -> None:
        segment_one = self.segment_one()
        judge.write_text(segment_one.sequence_log, STARTED + "\n")
        for step in ("s11", "s12"):
            for row in layer.one.STEP_ROWS[step]:
                if row != 15:
                    judge.write_text(self.path("dumps", "segment1-rows", f"row-{row}", "row-result.tsv"), "status\tok\n")
        self.assertEqual(layer.segment_one_rows_waiting(self.scratch, segment_one), ["s12:15", "sx:0"])

    def test_inputs_check_stops_on_a_changed_input_file(self) -> None:
        judge.write_text(self.path("repo", "a.py"), "a\n")
        layer.record_or_check_inputs(self.path("repo"), "dumps", ["a.py"], [("tree", "t")])
        layer.record_or_check_inputs(self.path("repo"), "dumps", ["a.py"], [("tree", "t")])
        judge.write_text(self.path("repo", "a.py"), "changed\n")
        with self.assertRaises(layer.one.Stopped):
            layer.record_or_check_inputs(self.path("repo"), "dumps", ["a.py"], [("tree", "t")])

    def test_run_values_check_stops_when_the_baseline_timing_moved(self) -> None:
        judge.write_text(self.path("repo", "a.py"), "a\n")
        layer.record_or_check_inputs(self.path("repo"), "dumps", ["a.py"], [("baseline_timing_h21", "x-r8-timing.out")])
        with self.assertRaises(layer.one.Stopped):
            layer.record_or_check_inputs(self.path("repo"), "dumps", ["a.py"], [("baseline_timing_h21", "x-attempt2-r8-timing.out")])

    def test_stop_file_is_a_stop_reason_once_it_exists(self) -> None:
        layer.STATE["segment_one"] = self.segment_one()
        before = layer.stop_reason(self.scratch)
        judge.write_text(self.segment_one().stop_file, "单元 step-5 红\n")
        self.assertEqual((before, "停派文件" in (layer.stop_reason(self.scratch) or "")), (None, True))

    def test_segment_one_cross_host_check_that_differs_is_a_stop_reason_and_one_that_agrees_is_not(self) -> None:
        segment_one = self.segment_one()
        layer.STATE["segment_one"] = segment_one
        product = os.path.join(self.scratch, segment_one.product("rows-cross-host-check"))
        judge.write_text(product, "E7RESULT name=verdict cross_host_identical=true row=0\n")
        agreeing = layer.stop_reason(self.scratch)
        judge.write_text(product, "E7RESULT name=verdict cross_host_identical=false row=0\n")
        self.assertEqual((agreeing, "S17" in (layer.stop_reason(self.scratch) or "")), (None, True))


class ContrastTests(TemporaryDirectoryCase):
    def test_touch_needs_the_row_to_have_run_and_reads_the_batch_of_the_recovery_report(self) -> None:
        self.assertFalse(contrast.touched({"ran": "false", "write_stream_differs": "true"}, "1"))
        self.assertTrue(contrast.touched({"ran": "true", "recovery_report_differs_batch_2": "true"}, "2"))
        self.assertFalse(contrast.touched({"ran": "true", "recovery_report_differs_batch_2": "true"}, "1"))

    def test_q2_set_is_the_union_of_red_and_not_red_rows(self) -> None:
        self.assertEqual(contrast.q2_of(contrast.fake_comparator({49: {}, 155: {}}, [155], [49], "yi")), ([49, 155], 1, "yi"))

    def test_category_change_is_reported_per_reading_and_batch(self) -> None:
        old = contrast.fake_comparator({3: {"category_original_reading_5_batch_2": "self_crash"}}, [], [], "bing")
        new = contrast.fake_comparator({3: {}}, [], [], "bing")
        changed = [line.split(" count=")[1] for line in contrast.q3_lines(old, new) if line.startswith("name=contrast_rows_category_changed")]
        self.assertEqual(changed, ["0 rows=-", "0 rows=-", "0 rows=-", "1 rows=3"])

    def test_ledger_hosts_read_only_the_whole_table_step_and_the_last_entry(self) -> None:
        ledger = judge.write_text(self.path("row-hosts.tsv"), "rows\t2\t1\tattempt-1\tlocal\nrows\t2\t2\tattempt-2\tfetched\ncross\t1\t2\tattempt-2\tfetched\n")
        self.assertEqual(contrast.ledger_hosts(ledger), {2: "2"})

    def test_substitution_line_appears_when_only_one_run_substituted(self) -> None:
        old = contrast.fake_comparator({7: {"substituted_from": "orig", "mutated_row_name": "g1"}}, [], [], "bing")
        lines = [line for line in contrast.q3_lines(old, contrast.fake_comparator({7: {}}, [], [], "bing")) if line.startswith("name=contrast_substitution")]
        self.assertEqual(lines, ["name=contrast_substitution role=contrast row=7 revision_6=g1 revision_8=- same=false"])

    def test_q3_denominators_are_paired_by_reading_and_batch(self) -> None:
        old = contrast.fake_comparator({1: {}, 2: {}}, [], [], "bing")
        new = contrast.fake_comparator({1: {}, 2: {}, 3: {}}, [], [], "bing")
        lines = [line for line in contrast.q3_lines(old, new) if line.startswith("name=contrast_q3_denominator")]
        self.assertEqual(lines[0], "name=contrast_q3_denominator role=contrast reading=registration batch=1 revision_6=1 revision_8=2 same=false")


if __name__ == "__main__":
    unittest.main()
