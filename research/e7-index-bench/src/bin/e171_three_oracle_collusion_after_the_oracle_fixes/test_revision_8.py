"""E171 第 1 次重跑第一段·修订 8 那一趟几份装置的单测（研究一侧变异表 research/mutations/e171_three_oracle_collusion_after_the_oracle_fixes_revision_8.tsv
经 mutation_driver.py --unit-test-module test_revision_8 跑）。每条用例一个场景、自己建自己删临时目录；只调纯函数与进程内的小件，不起装置、不编、不连别的台。"""
from __future__ import annotations

import json
import os
import shutil
import tempfile
import types
import unittest

import contrast_revision_6_against_revision_8 as contrast
import frozen_tree_lines_revision_8 as judge
import revision_8_placeholders as placeholders
import rows_orchestrator_revision_8 as rows
import segment_one_units_revision_8 as units


class TemporaryDirectoryCase(unittest.TestCase):
    def setUp(self) -> None:
        self.scratch = tempfile.mkdtemp(prefix="e171-r8-test-")

    def tearDown(self) -> None:
        shutil.rmtree(self.scratch, ignore_errors=True)

    def path(self, *parts: str) -> str:
        return os.path.join(self.scratch, *parts)


class FrozenTreeJudgeTests(TemporaryDirectoryCase):
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

    def test_dry_run_config_does_not_hold_at_start(self) -> None:
        self.assertFalse(placeholders.check_tree(self.scratch, {"dry_run": True, "tree": self.scratch}, "start").holds)


class UnitSchedulerTests(TemporaryDirectoryCase):
    def test_step_7_is_first_in_the_queue_even_when_another_unit_is_estimated_longer(self) -> None:
        longer = units.Unit("step-9", (), 10, 1, 99999)
        order = units.queue_order([longer, units.unit_table()["step-7"]])
        self.assertEqual([unit.name for unit in order], ["step-7", "step-9"])

    def test_downstream_of_step_7_includes_the_z1_unit_that_also_waits_for_step_9(self) -> None:
        self.assertEqual(units.downstream(units.unit_table(), {"step-7"}), {"step-7", "step-8", "step-9-z1"})

    def test_a_unit_does_not_fit_when_memory_runs_out_though_threads_suffice(self) -> None:
        table = units.unit_table()
        budget = units.Budget(30, 13 * units.GIB)
        self.assertFalse(units.fits(table["step-9"], [table["step-7"], table["step-3-o1-crash-judge"]], budget))

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


class ContrastTests(TemporaryDirectoryCase):
    def test_lines_of_the_same_kind_are_paired_by_their_identity_fields_not_by_order(self) -> None:
        old = ["E7RESULT name=q1_row row=15 stream=a batch=1 O2=1", "E7RESULT name=q1_row row=23 stream=a batch=1 O2=0"]
        new = ["E7RESULT name=q1_row row=23 stream=a batch=1 O2=0", "E7RESULT name=q1_row row=15 stream=a batch=1 O2=1"]
        self.assertFalse([cell for cell in contrast.contrast_cells(old, new) if cell.endswith(" same=false")])

    def test_s15_triggered_is_renamed_so_the_contrast_never_reads_as_a_stop(self) -> None:
        line = contrast.relabel_s15_line("E7RESULT name=verdict s15_four_problems=2 triggered=true", "6", "four")
        self.assertIn("differs_from_the_revision_6_prediction=true", line)
        self.assertNotIn(" triggered=", line)

    def test_row_time_takes_the_host_from_the_ledger(self) -> None:
        repo, revision_6, _ = contrast.fake_runs(self.scratch)
        hosts = {line.split(" row=")[1].split(" ")[0]: line.split(" host=")[1].split(" ")[0]
                 for line in contrast.row_time_lines(repo, revision_6) if line.startswith("name=contrast_row_time ")}
        self.assertEqual(hosts, {"0": "2", "15": "1"})


if __name__ == "__main__":
    unittest.main()
