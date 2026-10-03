"""E171 第二段比对器里 O3 那几列的单测（登记第九节 X16 起、第八节判别力自证）：Q1 的甲乙丙、批 2 不稳定、Q3 只靠 O3、Q2 的全放行与阴性底态、O3 阳性对照的读回、Q3-核的选行与比对。
手造数据，不读执行器、不读装置。每个互比旁边配一个钉绝对值的断言（例：甲乙分开的手造值、选出的行号写死）。

跑法（在本目录）：`python3 -m unittest test_o3_comparison`。
"""

from __future__ import annotations

import collections
import os
import tempfile
import unittest

import bad_images_report as images
import baseline_checks as checks
import collusion_matrix as matrix
import collusion_report as report
import comparator
from bad_images_report import BadImageRow, ImageJudgement
from result_lines import ResultLines
from run_directory import STREAM_NAMES
from test_collusion_matrix import DISAGREES, call, directory, labelled_call, run_whose_model_stopped_at, state
from test_o3_columns import write

QUIET = {name: ("in", "in") for name in STREAM_NAMES}


def o3_for(batch_one_pair: tuple[str, str], batch_two_pair: tuple[str, str] | None = None) -> dict[tuple[str, int], tuple[str, str]]:
    """状态 0（批 1）与状态 1（批 2）在两条流上各一个判定。"""
    verdicts = {}
    for name in STREAM_NAMES:
        verdicts[(name, 0)] = batch_one_pair
        verdicts[(name, 1)] = batch_two_pair or batch_one_pair
    return verdicts


def counted_row(number: int, o3: dict | None, pool: str = "green", crash: str = "green", model_word: str = "agrees",
                second_batch_pool: str = "green") -> report.RowAnalysis:
    run = directory([state(0, pool_checker=pool, crash_judge=crash), state(1, batch=2, pool_checker=second_batch_pool)], [call(0, model_word=model_word)])
    baseline = directory([state(0), state(1, batch=2)], [call(0)], writes="baseline")
    return report.analyse_row(number, f"r{number}", {"status": "ok", "device_exit": "0"}, baseline, run, {}, o3, "ran")


def render(function) -> list[str]:
    with tempfile.TemporaryDirectory() as root:
        lines = ResultLines(os.path.join(root, "out"))
        function(lines)
        lines.finish()
        with open(os.path.join(root, "out"), encoding="utf-8") as handle:
            return handle.read().splitlines()


class Q1WithO3Tests(unittest.TestCase):
    def test_a_row_quiet_on_o1_and_o2_is_caught_by_o3_only_when_o3_says_not_in(self) -> None:
        quiet_everywhere = counted_row(1, o3_for(("in", "in")))
        caught_by_o3 = counted_row(2, o3_for(("not_in", "not_in")))
        lines = render(lambda lines: report.q1_lines([quiet_everywhere, caught_by_o3], lines, True))
        alpha = next(line for line in lines if line.startswith("E7RESULT name=q1_alpha "))
        self.assertIn("all_quiet_rows=1 row_numbers=1 ", alpha)
        self.assertIn("verdict=yes_a_row_is_quiet_on_every_oracle", alpha)

    def test_a_row_only_the_provisional_reading_catches_is_quiet_for_the_strict_reading(self) -> None:
        row = counted_row(1, o3_for(("spec_gap", "not_in")))
        lines = render(lambda lines: report.q1_lines([row], lines, True))
        alpha = next(line for line in lines if line.startswith("E7RESULT name=q1_alpha "))
        beta = next(line for line in lines if line.startswith("E7RESULT name=q1_beta "))
        self.assertIn("all_quiet_rows=1 row_numbers=1", alpha)
        self.assertIn("all_quiet_rows=0 row_numbers=none", beta)

    def test_batch_two_is_judged_again_and_a_row_that_differs_is_listed(self) -> None:
        row = counted_row(1, o3_for(("in", "in"), ("not_in", "not_in")))
        lines = render(lambda lines: report.q1_lines([row], lines, True))
        alpha = next(line for line in lines if line.startswith("E7RESULT name=q1_alpha "))
        self.assertIn("all_quiet_rows=1", alpha)
        self.assertIn("batch_2_all_quiet_rows=0 rows_that_differ_from_batch_1=1", alpha)

    def test_the_verdict_word_follows_the_denominator_and_the_strict_coverage(self) -> None:
        counted = [object()] * 30
        self.assertEqual(report.reading_verdict([], counted, False, True), "no_such_row")
        self.assertEqual(report.reading_verdict([], counted[:29], False, True), "not_enough_rows")
        self.assertEqual(report.reading_verdict([], counted, True, True), "not_reported_f7_strict_o3_reaches_almost_no_state")
        self.assertEqual(report.reading_verdict([], counted, True, False), "no_such_row")
        self.assertEqual(report.reading_verdict([object()], counted[:1], True, True), "yes_a_row_is_quiet_on_every_oracle")

    def test_a_counted_row_whose_executor_failed_is_quiet_on_o3_and_listed_as_such(self) -> None:
        row = counted_row(1, {})
        row.o3_status = "executor_failed"
        self.assertTrue(matrix.all_quiet(row.run, report.Q1_ORACLES_STRICT, 1, row.o3))
        lines = render(lambda lines: report.row_lines(row, lines))
        self.assertTrue(any("name=row_o3 row=1 status=executor_failed" in line for line in lines))

    def test_a_row_without_o3_stays_unjudged_not_quiet(self) -> None:
        row = counted_row(1, None)
        self.assertIsNone(matrix.all_quiet(row.run, report.Q1_ORACLES_STRICT, 1, row.o3))
        lines = render(lambda lines: report.q1_lines([row], lines, False))
        self.assertTrue(any(line.startswith("E7RESULT name=q1_alpha status=not_run") for line in lines))


class Q3WithO3Tests(unittest.TestCase):
    def test_a_row_only_o3_catches_is_held_by_o3_alone_and_a_row_two_oracles_catch_is_not(self) -> None:
        only_o3 = counted_row(1, o3_for(("not_in", "not_in")))
        two = counted_row(2, o3_for(("not_in", "not_in")), pool="red", second_batch_pool="red")
        line = report.q3_line("q3_beta", [only_o3, two], False)
        self.assertIn("only_O3=1", line)
        self.assertNotIn("2", line.split("only_O3=")[1].split(" ")[0])
        self.assertIn("rows_whose_red_columns_differ_between_batch_1_and_2=none", line)

    def test_a_row_whose_red_columns_differ_between_the_batches_is_listed(self) -> None:
        row = counted_row(1, o3_for(("in", "in")), pool="red", second_batch_pool="green")
        self.assertIn("rows_whose_red_columns_differ_between_batch_1_and_2=1", report.q3_line("q3_alpha", [row], True))


def image_row(family: str, row_id: int, o2: str, strict: str | None, provisional: str | None, name: str = "P1-F", base: str = "F",
              expected: str | None = "not_in", stream: str = STREAM_NAMES[0]) -> BadImageRow:
    return BadImageRow(family, stream, row_id, name, base, o2, "I-1.1" if o2 == "red" else "-", "-", strict, provisional, expected)


class Q2WithO3Tests(unittest.TestCase):
    def lines_of(self, rows: list[BadImageRow], bases: dict | None = None) -> tuple[list[str], dict]:
        tally: dict[str, int] = {}
        lines = render(lambda output: images.bad_image_lines(rows, bases, output, tally))
        return lines, tally

    def test_the_all_quiet_readings_split_on_a_spec_gap_and_the_row_is_named(self) -> None:
        rows = [image_row("P", 0, "red", "not_in", "not_in"), image_row("P", 1, "green", "spec_gap", "not_in", name="P2-F"),
                image_row("B", 2, "green", "not_in", "not_in", name="B01"), image_row("K", 3, "green", "in", "in", name="I-4.8")]
        lines, tally = self.lines_of(rows)
        self.assertEqual((tally["q2_all_quiet_alpha"], tally["q2_all_quiet_beta"]), (2, 1))
        self.assertTrue(any("name=q2_all_quiet_row reading=alpha family=P stream=new_pool_file_creation row=1 " in line for line in lines))
        self.assertTrue(any("name=q2_all_quiet_row reading=alpha family=K " in line for line in lines))
        self.assertFalse(any("name=q2_all_quiet_row reading=beta family=P " in line for line in lines))

    def test_without_o3_judgements_the_all_quiet_count_is_not_reported(self) -> None:
        _, tally = self.lines_of([image_row("P", 0, "green", None, None)])
        self.assertNotIn("q2_all_quiet_alpha", tally)

    def test_a_p_row_judged_in_is_listed_for_f3_and_the_a2_count_follows_the_designers_expectations(self) -> None:
        rows = [image_row("P", 0, "red", "in", "in"), image_row("P", 1, "red", "not_in", "not_in", expected="in", name="P8-F")]
        _, tally = self.lines_of(rows)
        self.assertEqual((tally["f3_p_rows_judged_in_by_o3"], tally["a2_p_rows_not_expected_not_in"]), (1, 1))

    def test_geometry_compares_the_two_base_states_of_each_patch_kind(self) -> None:
        same = [image_row("P", 0, "red", "not_in", "not_in"), image_row("P", 1, "red", "not_in", "not_in", name="P1-Mk", base="Mk")]
        _, tally = self.lines_of(same)
        self.assertEqual(tally["q2_geometry_unstable_kinds"], 0)
        differing = [image_row("P", 0, "red", "not_in", "not_in"), image_row("P", 1, "red", "in", "in", name="P1-Mk", base="Mk")]
        lines, tally = self.lines_of(differing)
        self.assertEqual(tally["q2_geometry_unstable_kinds"], 1)
        self.assertTrue(any("name=q2_geometry stream=new_pool_file_creation kind=P1 " in line and "status=unstable" in line for line in lines))

    def test_negative_bases_count_o2_and_o3_reds_separately(self) -> None:
        rows = [image_row("P", 0, "green", None, None), image_row("P", 1, "green", None, None, name="P1-Mk", base="Mk")]
        bases = {(STREAM_NAMES[0], "F"): ImageJudgement("in", "in", True, True), (STREAM_NAMES[0], "Mk"): ImageJudgement("spec_gap", "not_in", True, True)}
        lines, tally = self.lines_of(rows, bases)
        self.assertEqual((tally["negative_bases_with_o2_red"], tally["negative_bases_with_o3_red"]), (0, 1))
        self.assertTrue(any("o3_judged_bases=2 o3_strict_red_bases=0 o3_provisional_red_bases=1" in line for line in lines))

    def test_the_judgements_are_joined_by_source_stream_and_row(self) -> None:
        rows = [image_row("P", 5, "red", None, None), image_row("K", 5, "red", None, None, name="I-1.1")]
        judgements = {("e170", STREAM_NAMES[0], 5, "-"): ImageJudgement("not_in", "in", True, True),
                      ("known_bad", STREAM_NAMES[0], 5, "-"): ImageJudgement("in", "not_in", True, True)}
        attached = images.attach_o3(rows, judgements)
        self.assertEqual([(row.o3_strict, row.o3_provisional) for row in attached], [("not_in", "in"), ("in", "not_in")])
        self.assertEqual(images.attach_o3(rows, None), rows)


class ComparatorO3Tests(unittest.TestCase):
    def test_the_o3_control_is_red_only_for_a_strict_not_in_on_a_matching_base(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            red, mismatch, absent = (os.path.join(root, name) for name in ("red", "mismatch", "absent"))
            write(red, "E7RESULT name=o3_control_verdict strict=not_in provisional=not_in base_digest_matches=true red_strict=true\n")
            write(mismatch, "E7RESULT name=o3_control_verdict strict=not_in provisional=not_in base_digest_matches=false red_strict=false\n")
            write(absent, "E7RESULT name=done emitted=0\n")
            self.assertEqual([comparator.o3_control_red(path) for path in (red, mismatch, absent)], [1, 0, 0])

    def test_a_control_for_o3_is_counted_and_the_unknown_oracle_name_is_refused(self) -> None:
        import collections
        with tempfile.TemporaryDirectory() as root:
            red, quiet = os.path.join(root, "red"), os.path.join(root, "quiet")
            write(red, "E7RESULT name=o3_control_verdict red_strict=true\n")
            write(quiet, "E7RESULT name=o3_control_verdict red_strict=false\n")
            tally = collections.Counter()
            lines = ResultLines(os.path.join(root, "out"))
            comparator.control_lines([f"O3:baseline:{red}", f"O3:empty_mutation:{quiet}"], lines, tally)
            lines.finish()
            self.assertEqual((tally["positive_controls_run"], tally["positive_controls_not_red"]), (2, 1))
            with open(os.path.join(root, "out"), encoding="utf-8") as handle:
                self.assertIn("name=positive_control oracle=O3 run=empty_mutation red_units=0 red=false", handle.read())

    def test_the_row_o3_is_read_only_when_the_driver_says_ran_and_the_output_is_complete(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            write(os.path.join(root, "o3.out"), "name=state stream=s state=1 strict=in provisional=in time_ns=1 worker=2\nname=complete\n")
            verdicts, reading = comparator.read_row_o3(root, {"o3_status": "ran"})
            self.assertEqual(verdicts, {("s", 1): ("in", "in")})
            self.assertTrue(reading.complete)
            self.assertEqual(comparator.read_row_o3(root, {"o3_status": "executor_failed"}), ({}, None))
            self.assertEqual(comparator.read_row_o3(root, {"o3_status": "skipped_by_request"}), (None, None))
            write(os.path.join(root, "half.out"), "")
        with tempfile.TemporaryDirectory() as root:
            write(os.path.join(root, "o3.out"), "name=state stream=s state=1 strict=in provisional=in time_ns=1 worker=2\n")
            self.assertEqual(comparator.read_row_o3(root, {"o3_status": "ran"})[0], {})

    def test_the_q3_core_takes_the_first_three_rows_of_the_first_slice_with_two_red_columns_else_the_first_three_with_one(self) -> None:
        import bad_implementation_table as table
        with tempfile.TemporaryDirectory() as root:
            selected = os.path.join(root, "selected.tsv")
            write(selected, "".join(f"{number}\t{1 if number <= 6 else 2}\tf.rs\t0\tname{number}\t000000000000\n" for number in range(1, 8)))
            self.assertEqual([row.slice_number for row in table.read_selected_rows(selected)], [1, 1, 1, 1, 1, 1, 2])
            o3_red = o3_for(("not_in", "not_in"))
            rows = [counted_row(1, o3_red, pool="red"), counted_row(2, o3_for(("in", "in"))), counted_row(3, o3_red, pool="red"),
                    counted_row(4, o3_red), counted_row(5, o3_red, pool="red"), counted_row(6, o3_red, pool="red"), counted_row(7, o3_red, pool="red")]
            self.assertEqual(comparator.q3_core_rows(rows, selected), [1, 3, 5])
            self.assertEqual(comparator.q3_core_rows([rows[0], rows[1], rows[2], rows[3]], selected), [1, 3, 4])

    def test_the_closed_run_differs_only_where_the_closed_column_is_and_a_no_executor_run_must_match_exactly(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            for kind, call_word, o3_text in (("full", "agrees", "A"), ("same", "agrees", "A"), ("o1_closed", "not_modelled", "A"), ("o3_other", "agrees", "B")):
                dump = os.path.join(root, kind, "row-7", "dump")
                for name in STREAM_NAMES:
                    base = os.path.join(dump, name)
                    write(os.path.join(base, "ops.tsv"), "0\t0\twrite\t0\t512\t0\t-\n")
                    write(os.path.join(base, "oracle-verdicts.tsv"), "0\t1\t0\t0\t0\tgreen\t-\tgreen\t-\ttrue\t1,3\tfile_read\tabc\t1\t1,3\tfile_read\tabc\t1\t-\n")
                    write(os.path.join(base, "call-outcomes.tsv"), f"0\tpublish\t0\t1\tok\t{call_word}\t-\t-\t-\n")
                    # 关 O1 那一趟模型不停（登记修订 4：`model_stopped_at_call` 是 O1 自己的状态，关 O1 时不比）
                    stopped = "none" if kind == "o1_closed" else "3"
                    write(os.path.join(base, "stream-status.tsv"), f"ending\tcompleted\nmodel_stopped_at_call\t{stopped}\ncalls\t4\n")
                write(os.path.join(root, kind, "row-7", "o3.out"), f"name=state stream=s state=2 strict=in provisional=in x={o3_text} time_ns=9 worker=1\n"
                      f"name=state stream=s state=1 strict=in provisional=in x={o3_text} time_ns=8 worker=2\n")
            import collections
            tally = collections.Counter()
            lines = render(lambda output: comparator.q3_core_lines([f"O1:{root}/full:{root}/o1_closed", f"O1:{root}/full:{root}/same", f"O1:{root}/full:{root}/o3_other",
                                                                    f"no_executor:{root}/full:{root}/o1_closed"], output, tally))
            by_run = [line for line in lines if "name=q3_core_row" in line]
            self.assertEqual(len(by_run), 4)
            self.assertIn("closing=O1 row=7 differences_outside_the_closed_column=0 o3_state_lines_differ=false", by_run[0])
            self.assertIn("closing=O1 row=7 differences_outside_the_closed_column=0 o3_state_lines_differ=false", by_run[1])
            self.assertIn("o3_state_lines_differ=true", by_run[2])
            self.assertIn("closing=no_executor row=7 differences_outside_the_closed_column=4 o3_state_lines_differ=not_applicable_no_executor_in_the_closed_run", by_run[3])
            self.assertEqual((tally["q3_core_runs"], tally["q3_core_runs_with_differences"]), (4, 2))
            with self.assertRaises(ValueError):
                comparator.q3_core_lines(["O9:a:b"], ResultLines(None), tally)

    def test_the_executor_state_lines_are_sorted_and_lose_the_time_and_the_process_number(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            path = os.path.join(root, "o3.out")
            write(path, "name=state stream=b state=2 strict=in time_ns=5 worker=9\nname=timing batch=1\nname=state stream=a state=10 strict=in time_ns=1 worker=2\n"
                  "name=state stream=a state=3 strict=in time_ns=7 worker=3\n")
            self.assertEqual(checks.o3_state_lines_without_timing(path),
                             ["name=state stream=a state=3 strict=in", "name=state stream=a state=10 strict=in", "name=state stream=b state=2 strict=in"])


class RegisteredTableLinesTests(unittest.TestCase):
    """A6、A7 不数 E171 装置自己的变异行（`E171 装置` 起头）：它们是测量工具、不是被测的坏实现；为证红而改核心的一行在 core 下，数进去会让挑法与计数对不上登记。"""

    def test_the_device_own_rows_do_not_move_the_selection_or_the_count(self) -> None:
        file = "crates/singlefs-core/src/mount.rs"
        with tempfile.TemporaryDirectory() as root:
            table_path, selected_path = os.path.join(root, "mutations.tsv"), os.path.join(root, "selected.tsv")
            core_rows = [f"core row {index}\t{file}\to\tr\t-p c\tt" for index in range(16)]
            write(table_path, "\n".join(core_rows + [f"E171 装置 取号恒拒\t{file}\to\tr\t-p c\tt"]) + "\n")
            write(selected_path, f"1\t1\t{file}\t0\tcore row 0\t000000000000\n2\t2\t{file}\t8\tcore row 8\t000000000000\n")
            lines = render(lambda lines: comparator.registered_table_lines(table_path, selected_path, lines))
        a6 = next(line for line in lines if line.startswith("E7RESULT name=a6 "))
        a7 = next(line for line in lines if line.startswith("E7RESULT name=a7 "))
        self.assertIn("step_8_rows=2 step_4_rows=4", a6)
        self.assertTrue(a6.endswith("step_8_names_equal_the_registered_table=true"))
        self.assertIn("implementation_rows=16 expected=844 files=1", a7)
        self.assertIn("E7RESULT name=a7_device_rows device_rows_left_out_of_a6_and_a7=1", lines)


class RowS5ReportTests(unittest.TestCase):
    """装置逐行 S5 的比对器一侧进结果行与判决行（代码轮第三轮判决 K4-c 甲、丙）：`name=row_s5` 一行、判决行两个计数。"""

    def row_with_calls(self, calls, stopped, states=None) -> report.RowAnalysis:
        run = run_whose_model_stopped_at(calls, states or [state(0), state(1, batch=2)], stopped)
        baseline = directory([state(0), state(1, batch=2)], [call(0)], writes="baseline")
        return report.analyse_row(1, "r1", {"status": "ok", "device_exit": "0"}, baseline, run, {}, None, "-")

    def test_the_row_s5_line_carries_both_counts_per_stream(self) -> None:
        silent = self.row_with_calls([labelled_call(0, "acquire", "err:X", "not_modelled")], 0)
        self.assertIn("E7RESULT name=row_s5 row=1 errored_calls_left_unjudged=1/1 o1_not_applicable_states_with_another_source=0/0",
                      render(lambda lines: report.row_lines(silent, lines)))
        other = self.row_with_calls([labelled_call(0, "publish", "err:Y", "agrees")], 0, [state(0, crash_judge="na"), state(1, batch=2)])
        self.assertIn("E7RESULT name=row_s5 row=1 errored_calls_left_unjudged=0/0 o1_not_applicable_states_with_another_source=1/1",
                      render(lambda lines: report.row_lines(other, lines)))

    def test_the_verdict_counts_rows_not_streams_and_a_recorded_red_is_not_counted(self) -> None:
        silent = self.row_with_calls([labelled_call(0, "acquire", "err:X", "not_modelled")], 0)
        recorded = self.row_with_calls([labelled_call(0, "acquire", "err:X", DISAGREES)], 0)
        other = self.row_with_calls([labelled_call(0, "publish", "err:Y", "agrees")], 0, [state(0, crash_judge="na"), state(1, batch=2)])
        tally = collections.Counter()
        comparator.row_tallies([silent, recorded, other], tally)
        self.assertEqual((tally["rows_with_an_errored_call_the_comparison_did_not_judge"], tally["rows_with_o1_not_applicable_from_another_source"]), (1, 1))
        # 第 108 行那一形（登记修订 4）：对拍判出不同、模型停下，之后的错误调用没比，不数；那一行 row_s5 两个数都是 0
        after_the_stop = self.row_with_calls([labelled_call(0, "publish", "ok", DISAGREES),
                                              labelled_call(1, "mount_writable", "err:Recovery", "skipped_after_the_model_stopped")], 0,
                                             [state(0, crash_judge="na"), state(1, batch=2)])
        self.assertIn("E7RESULT name=row_s5 row=1 errored_calls_left_unjudged=0/0 o1_not_applicable_states_with_another_source=0/0",
                      render(lambda lines: report.row_lines(after_the_stop, lines)))
        tally_after_the_stop = collections.Counter()
        comparator.row_tallies([after_the_stop], tally_after_the_stop)
        self.assertEqual((tally_after_the_stop["rows_with_an_errored_call_the_comparison_did_not_judge"],
                          tally_after_the_stop["rows_with_o1_not_applicable_from_another_source"]), (0, 0))
        self.assertIn("rows_with_an_errored_call_the_comparison_did_not_judge", comparator.VERDICT_KEYS)
        self.assertIn("rows_with_o1_not_applicable_from_another_source", comparator.VERDICT_KEYS)

    def test_a_compiled_row_without_the_drivers_rebuild_check_is_counted(self) -> None:
        """登记修订 5：编过的行（status=ok）没带驱动编完核过重编的那个键，是旧驱动跑的；编不过的行不算。"""
        baseline = directory([state(0), state(1, batch=2)], [call(0)], writes="baseline")
        run = directory([state(0), state(1, batch=2)], [call(0)])
        checked = report.analyse_row(1, "r1", {"status": "ok", "device_exit": "0", "packages_checked_rebuilt": "crates/singlefs-core"}, baseline, run, {}, None)
        unchecked = report.analyse_row(2, "r2", {"status": "ok", "device_exit": "0"}, baseline, run, {}, None)
        not_compiled = report.analyse_row(3, "r3", {"status": "compile_failed", "device_exit": "-"}, baseline, None, {}, None)
        tally = collections.Counter()
        comparator.row_tallies([checked, unchecked, not_compiled], tally)
        self.assertEqual(tally["rows_compiled_without_the_rebuild_check"], 1)
        self.assertIn("rows_compiled_without_the_rebuild_check", comparator.VERDICT_KEYS)


class F4ExpansionTests(unittest.TestCase):
    """F4 扩表（登记 5.2「扩表」）：步长 4 比步长 8 多出来的行按挑法脚本的输出顺序从 118 编号；比对器并两个输出根、分母按表分开报；驱动补位跳过两张表的名字。
    手造表：a.rs 16 行（步长 8 取下标 0、8，步长 4 取 0、4、8、12），b.rs 5 行（步长 8 取 0，步长 4 取 0、2），另加一行 E171 装置自己的（不算）与一行 format 的（不算）。"""

    FILE_A = "crates/singlefs-core/src/a.rs"
    FILE_B = "crates/singlefs-core/src/b.rs"

    def table_text(self) -> str:
        rows = [f"a{index}\t{self.FILE_A}\to\tr\t-p c\tt" for index in range(16)]
        rows.insert(5, f"E171 装置 取号恒拒\t{self.FILE_A}\to\tr\t-p c\tt")  # 数进去 a.rs 一组就是 17 行、步长 4 的下标全挪
        rows += [f"b{index}\t{self.FILE_B}\to\tr\t-p c\tt" for index in range(5)]
        rows.append("f0\tcrates/singlefs-format/src/lib.rs\to\tr\t-p c\tt")
        return "\n".join(rows) + "\n"

    def test_the_expansion_takes_the_step_four_rows_the_step_eight_table_left_out_in_the_script_order_from_118(self) -> None:
        import bad_implementation_table as table
        import expansion_table
        rows = expansion_table.expansion_rows(self.table_text())
        self.assertEqual([(row.number, row.slice_number, row.file, row.group_index, row.name) for row in rows],
                         [(118, 3, self.FILE_A, 4, "a4"), (119, 3, self.FILE_A, 12, "a12"), (120, 3, self.FILE_B, 2, "b2")])
        self.assertEqual(rows[0].line_digest_prefix, table.line_digest_prefix(f"a4\t{self.FILE_A}\to\tr\t-p c\tt"))
        with tempfile.TemporaryDirectory() as root:
            path = os.path.join(root, "expansion.tsv")
            write(path, "".join(expansion_table.selected_row_line(row) for row in rows))
            self.assertEqual(table.read_selected_rows(path), rows)

    def write_tables(self, root: str, expansion_lines: str) -> tuple[str, str, str]:
        table_path, selected_path, expansion_path = (os.path.join(root, name) for name in ("mutations.tsv", "selected.tsv", "expansion.tsv"))
        write(table_path, self.table_text())
        write(selected_path, f"1\t1\t{self.FILE_A}\t0\ta0\t000000000000\n2\t2\t{self.FILE_A}\t8\ta8\t000000000000\n3\t1\t{self.FILE_B}\t0\tb0\t000000000000\n")
        write(expansion_path, expansion_lines)
        return table_path, selected_path, expansion_path

    def test_the_comparator_checks_the_expansion_against_the_selection_script_row_by_row(self) -> None:
        import expansion_table
        rows = expansion_table.expansion_rows(self.table_text())
        good = "".join(expansion_table.selected_row_line(row) for row in rows)
        swapped = "".join(expansion_table.selected_row_line(row) for row in (rows[1], rows[0], rows[2]))
        with tempfile.TemporaryDirectory() as root:
            paths = self.write_tables(root, good)
            good_lines = render(lambda lines: comparator.expansion_table_lines(*paths, lines))
        with tempfile.TemporaryDirectory() as root:
            paths = self.write_tables(root, swapped)
            swapped_lines = render(lambda lines: comparator.expansion_table_lines(*paths, lines))
        self.assertEqual(good_lines, ["E7RESULT name=f4_expansion rows=3 expected=164 first_number=118 last_number=120 numbers_are_consecutive=true "
                                      "slice_3_rows=3 names_shared_with_the_117_rows=0 found_once_in_today_table=3 "
                                      "equal_to_the_selection_script_on_today_table=true", "E7RESULT name=done emitted=2"])
        self.assertIn("first_number=119 last_number=120 numbers_are_consecutive=false", swapped_lines[0])
        self.assertTrue(swapped_lines[0].endswith("equal_to_the_selection_script_on_today_table=false"))

    def test_two_output_roots_are_merged_by_row_number_and_a_row_in_both_is_refused(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            for relative in ("table/row-117", "table/row-1", "table/notes", "expansion/row-118", "again/row-117"):
                os.makedirs(os.path.join(root, relative))
            merged = comparator.row_directories([os.path.join(root, "table"), os.path.join(root, "expansion")])
            self.assertEqual([(number, os.path.relpath(path, root)) for number, path in merged],
                             [(1, "table/row-1"), (117, "table/row-117"), (118, "expansion/row-118")])
            with self.assertRaises(ValueError):
                comparator.row_directories([os.path.join(root, "table"), os.path.join(root, "again")])

    def test_the_denominator_is_reported_again_per_table(self) -> None:
        o3 = o3_for(("in", "in"))
        baseline = directory([state(0), state(1, batch=2)], [call(0)], writes="baseline")
        not_compiled = report.analyse_row(119, "r119", {"status": "compile_failed", "device_exit": "-"}, baseline, None, {}, None)
        rows = [counted_row(1, o3), counted_row(2, o3, pool="red"), counted_row(118, o3), not_compiled]
        self.assertEqual(render(lambda lines: comparator.q1_denominator_by_table_lines(rows, {118, 119, 120}, lines)),
                         ["E7RESULT name=q1_denominator_by_table table=step_8_table rows_run=2 counted_D=2 counted_row_numbers=1,2",
                          "E7RESULT name=q1_denominator_by_table table=step_4_expansion rows_run=2 counted_D=1 counted_row_numbers=118",
                          "E7RESULT name=done emitted=3"])

    def test_the_driver_skips_the_names_of_the_other_table_when_it_substitutes(self) -> None:
        import bad_implementation_table as table
        import row_driver
        with tempfile.TemporaryDirectory() as root:
            _, selected_path, expansion_path = self.write_tables(root, f"118\t3\t{self.FILE_A}\t4\ta4\t000000000000\n")
            names = row_driver.names_in_the_tables(table.read_selected_rows(expansion_path), [selected_path])
        self.assertEqual(names, {"a4", "a0", "a8", "b0"})
        group = [f"a{index}" for index in range(16)]
        self.assertEqual(table.substitute_for(group, "a7", names, set()), "a9")
        self.assertEqual(table.substitute_for(group, "a3", names, set()), "a5")


class CompileFailureSubstitutionComparatorTests(unittest.TestCase):
    """补位那一趟（登记第一节读法 5）：补位输出根的行只换掉原来编不过的同号行；结果行写换上的行名与补位自的原名；补位行另报一行、合计一行。"""

    def write_row(self, root: str, relative: str, fields: dict[str, str]) -> None:
        write(os.path.join(root, relative, "row-result.tsv"), "".join(f"{key}\t{value}\n" for key, value in fields.items()))

    def test_a_row_that_did_not_compile_is_replaced_and_every_other_case_is_refused(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            self.write_row(root, "table/row-3", {"row_name": "c 原", "status": "compile_failed"})
            self.write_row(root, "table/row-4", {"row_name": "d", "status": "ok"})
            self.write_row(root, "subst/row-3", {"row_name": "c 原", "status": "ok", "substituted_from": "c 原", "mutated_row_name": "c 换"})
            merged = comparator.row_directories([os.path.join(root, "table")])
            replaced_rows, replaced = comparator.rows_with_substitutes(merged, [os.path.join(root, "subst")])
            self.assertEqual([(number, os.path.relpath(path, root)) for number, path in replaced_rows], [(3, "subst/row-3"), (4, "table/row-4")])
            self.assertEqual([(number, os.path.relpath(old, root), os.path.relpath(new, root)) for number, old, new in replaced],
                             [(3, "table/row-3", "subst/row-3")])
            self.assertEqual(comparator.rows_with_substitutes(merged, []), (merged, []))
            for relative, fields in (("not-failed/row-4", {"row_name": "d", "status": "ok", "substituted_from": "d", "mutated_row_name": "e"}),
                                     ("unknown/row-9", {"row_name": "z", "status": "ok", "substituted_from": "z"}),
                                     ("other-name/row-3", {"row_name": "x", "status": "ok", "substituted_from": "x", "mutated_row_name": "y"}),
                                     ("compiled-now/row-3", {"row_name": "c 原", "status": "ok", "substituted_from": "-", "mutated_row_name": "c 原"})):
                self.write_row(root, relative, fields)
                with self.assertRaises(ValueError, msg=relative):
                    comparator.rows_with_substitutes(merged, [os.path.join(root, relative.split("/")[0])])
            self.write_row(root, "still-failing/row-3", {"row_name": "c 原", "status": "compile_failed", "substituted_from": "c 原", "mutated_row_name": "-"})
            self.assertEqual(len(comparator.rows_with_substitutes(merged, [os.path.join(root, "still-failing")])[1]), 1)

    def test_the_row_analyses_take_the_substitute_in_place_of_the_row_that_did_not_compile(self) -> None:
        from test_collusion_pipeline import minimal_run
        with tempfile.TemporaryDirectory() as root:
            minimal_run(os.path.join(root, "baseline"))
            self.write_row(root, "table/row-3", {"row_name": "c 原", "status": "compile_failed", "device_exit": "-"})
            self.write_row(root, "subst/row-3", {"row_name": "c 原", "status": "ok", "device_exit": "0", "substituted_from": "c 原",
                                                 "mutated_row_name": "c 换", "o3_status": "skipped_by_request"})
            minimal_run(os.path.join(root, "subst/row-3/dump"))
            plain = comparator.row_analyses([os.path.join(root, "table")], [], os.path.join(root, "baseline"), [])
            merged = comparator.row_analyses([os.path.join(root, "table")], [], os.path.join(root, "baseline"), [], [os.path.join(root, "subst")])
        self.assertEqual([(row.number, row.name, row.category) for row in plain], [(3, "c 原", "compile_failed")])
        self.assertEqual([(row.number, row.name, row.category) for row in merged], [(3, "c 换", "untouched")])

    def test_the_result_row_carries_the_substitute_name_and_each_substitution_is_reported(self) -> None:
        self.assertEqual(comparator.row_name_for_the_result({"row_name": "原", "substituted_from": "原", "mutated_row_name": "换"}, "表"), "换")
        self.assertEqual(comparator.row_name_for_the_result({"row_name": "原", "substituted_from": "-", "mutated_row_name": "原"}, "表"), "原")
        self.assertEqual(comparator.row_name_for_the_result({"substituted_from": "原", "mutated_row_name": "-"}, "表"), "表")
        with tempfile.TemporaryDirectory() as root:
            self.write_row(root, "table/row-3", {"row_name": "c 原", "status": "compile_failed"})
            self.write_row(root, "table/row-7", {"row_name": "g", "status": "compile_failed"})
            self.write_row(root, "subst/row-3", {"row_name": "c 原", "status": "ok", "substituted_from": "c 原", "mutated_row_name": "c 换",
                                                 "failed_attempts": "1", "failed_attempt_1": "compile_failed:c 原"})
            self.write_row(root, "subst/row-7", {"row_name": "g", "status": "compile_failed", "substituted_from": "g", "mutated_row_name": "-",
                                                 "failed_attempts": "2", "failed_attempt_1": "compile_failed:g", "failed_attempt_2": "anchor_not_unique:h"})
            _, replaced = comparator.rows_with_substitutes(comparator.row_directories([os.path.join(root, "table")]), [os.path.join(root, "subst")])
            baseline = directory([state(0), state(1, batch=2)], [call(0)], writes="baseline")
            analyses = [counted_row(3, o3_for(("in", "in"))),
                        report.analyse_row(7, "g", {"status": "compile_failed", "device_exit": "-"}, baseline, None, {}, None)]
            rendered = render(lambda lines: comparator.substitution_lines(replaced, analyses, lines))
        self.assertEqual(rendered, [
            "E7RESULT name=substitution row=3 original_name=c_原 original_status=compile_failed substituted_by=c_换 failed_attempts=1 "
            "attempts=compile_failed:c_原 category=counted",
            "E7RESULT name=substitution row=7 original_name=g original_status=compile_failed substituted_by=none failed_attempts=2 "
            "attempts=compile_failed:g|anchor_not_unique:h category=compile_failed",
            "E7RESULT name=substitution_summary rows_that_did_not_compile=2 substituted_rows=1 still_not_compiled=1 row_numbers=3,7",
            "E7RESULT name=done emitted=4"])


if __name__ == "__main__":
    unittest.main()
