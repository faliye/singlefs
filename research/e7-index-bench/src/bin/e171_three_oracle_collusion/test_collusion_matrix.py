"""E171 比对器矩阵部分的单测（登记第九节 X1–X3、X7、X8、X11、X12，第八节判别力自证）：手造数据，不读盘。

跑法（在本目录）：`python3 -m unittest test_collusion_matrix`。每个断言都配了一条钉绝对值的：O3 的两个判定在手造矩阵里各给一个会让甲乙分开的值，
这样「甲乙对调」「欠一句当红」这类弄坏不会被「两列一起错」藏住。
"""

from __future__ import annotations

import unittest

import collusion_matrix as matrix
from collusion_report import (CATEGORY_DEVICE_STOP, Q1_ORACLES_STRICT, classify_row, untouched_but_red_rows, RowAnalysis)
from run_directory import (CallRow, RecoverySummary, RunDirectory, StateRow, StreamRun, STREAM_NAMES)

NO_FILE_SUMMARY = RecoverySummary("1,3", "file_read", "abc", 1)


def state(number: int, segment: int = 0, batch: int = 1, crash_judge: str = "green", pool_checker: str = "green",
          consulted: RecoverySummary = NO_FILE_SUMMARY, ignored: RecoverySummary = NO_FILE_SUMMARY, items: str = "-") -> StateRow:
    return StateRow(state_number=number, ordinal=number, segment=segment, batch=batch, landing="2", digest="d", touched_call=0,
                    crash_judge=crash_judge, crash_judge_detail="-", pool_checker=pool_checker, pool_checker_items="-",
                    pool_checker_matches_the_judgement=True, consulted=consulted, ignored=ignored, layer0_red_items=items)


def call(index: int, outcome: str = "ok", model_word: str = "agrees") -> CallRow:
    return CallRow(index, "publish", index * 10, index * 10 + 10, outcome, model_word, "-", "-", "-")


def stream(name: str, states: list[StateRow], calls: list[CallRow], writes: str = "w") -> StreamRun:
    return StreamRun(name=name, exported=True, states=states, calls=calls, status={"ending": "completed"},
                     write_stream_sha256={"ops.tsv": writes})


def directory(states: list[StateRow], calls: list[CallRow], writes: str = "w") -> RunDirectory:
    return RunDirectory("x", {name: stream(name, list(states), list(calls), writes) for name in STREAM_NAMES})


class TouchTests(unittest.TestCase):
    def test_identical_runs_are_untouched(self) -> None:
        baseline, row = directory([state(0)], [call(0)]), directory([state(0)], [call(0)])
        report = matrix.touch_report(baseline, row)
        self.assertFalse(report.touched())
        self.assertFalse(report.only_in_the_ignored_pass())

    def test_a_different_write_stream_alone_touches(self) -> None:
        report = matrix.touch_report(directory([state(0)], [call(0)]), directory([state(0)], [call(0)], writes="other"))
        self.assertTrue(report.write_stream_differs)
        self.assertTrue(report.touched())

    def test_a_different_call_outcome_with_the_same_write_stream_touches(self) -> None:
        report = matrix.touch_report(directory([state(0)], [call(0)]), directory([state(0)], [call(0, outcome="err:NoSpace")]))
        self.assertFalse(report.write_stream_differs)
        self.assertTrue(report.call_outcomes_differ)
        self.assertTrue(report.touched())

    def test_a_different_recovery_report_with_the_same_write_stream_and_outcomes_touches(self) -> None:
        other = RecoverySummary("1,3", "file_read", "def", 1)
        report = matrix.touch_report(directory([state(0)], [call(0)]), directory([state(0, consulted=other)], [call(0)]))
        self.assertFalse(report.write_stream_differs or report.call_outcomes_differ)
        self.assertTrue(report.recovery_report_differs)
        self.assertTrue(report.touched())

    def test_a_difference_only_in_the_pass_that_ignores_the_journal_is_not_a_touch(self) -> None:
        other = RecoverySummary("0,0", "no_file", "-", 0)
        report = matrix.touch_report(directory([state(0)], [call(0)]), directory([state(0, ignored=other)], [call(0)]))
        self.assertFalse(report.touched())
        self.assertTrue(report.only_in_the_ignored_pass())


class ColumnTests(unittest.TestCase):
    def test_o3_columns_are_none_when_o3_did_not_run(self) -> None:
        run = directory([state(0)], [call(0)])
        self.assertIsNone(matrix.column_is_red(run, matrix.ORACLE_THREE_STRICT, 1, None))
        self.assertIsNone(matrix.column_is_red(run, matrix.ORACLE_THREE_PROVISIONAL, 1, None))
        self.assertFalse(matrix.column_is_red(run, matrix.ORACLE_ONE, 1, None))

    def test_a_spec_gap_is_an_abstention_for_the_strict_reading_and_not_for_the_provisional_one(self) -> None:
        run = directory([state(0)], [call(0)])
        gap_only_strict = {(name, 0): ("spec_gap", "not_in") for name in STREAM_NAMES}
        self.assertFalse(matrix.column_is_red(run, matrix.ORACLE_THREE_STRICT, 1, gap_only_strict))
        self.assertTrue(matrix.column_is_red(run, matrix.ORACLE_THREE_PROVISIONAL, 1, gap_only_strict))
        flipped = {(name, 0): ("not_in", "in") for name in STREAM_NAMES}
        self.assertTrue(matrix.column_is_red(run, matrix.ORACLE_THREE_STRICT, 1, flipped))
        self.assertFalse(matrix.column_is_red(run, matrix.ORACLE_THREE_PROVISIONAL, 1, flipped))

    def test_o1_is_red_when_either_the_comparison_or_the_crash_judge_is_red(self) -> None:
        comparison_red = directory([state(0)], [call(0, model_word="disagrees:RefusedWhenModelRequiresSuccess")])
        crash_red = directory([state(0, crash_judge="red")], [call(0)])
        quiet = directory([state(0, crash_judge="na")], [call(0)])
        for run, expected in ((comparison_red, True), (crash_red, True), (quiet, False)):
            self.assertEqual(matrix.column_is_red(run, matrix.ORACLE_ONE, 1, None), expected)
        self.assertTrue(matrix.column_is_red(comparison_red, matrix.ORACLE_COMPARISON, 1, None))
        self.assertFalse(matrix.column_is_red(comparison_red, matrix.ORACLE_CRASH_JUDGE, 1, None))
        self.assertEqual(matrix.crash_judge_not_applicable_counts(quiet, 1), {name: 1 for name in STREAM_NAMES})

    def test_q1_all_quiet_needs_the_third_oracle_and_names_all_three(self) -> None:
        run = directory([state(0)], [call(0)])
        o3_red = {(name, 0): ("not_in", "not_in") for name in STREAM_NAMES}
        self.assertEqual(Q1_ORACLES_STRICT, (matrix.ORACLE_ONE, matrix.ORACLE_TWO, matrix.ORACLE_THREE_STRICT))
        self.assertFalse(matrix.all_quiet(run, Q1_ORACLES_STRICT, 1, o3_red))
        o3_quiet = {(name, 0): ("in", "in") for name in STREAM_NAMES}
        self.assertTrue(matrix.all_quiet(run, Q1_ORACLES_STRICT, 1, o3_quiet))
        self.assertIsNone(matrix.all_quiet(run, Q1_ORACLES_STRICT, 1, None))
        self.assertFalse(matrix.all_quiet(directory([state(0, pool_checker="red")], [call(0)]), Q1_ORACLES_STRICT, 1, o3_quiet))

    def test_the_number_of_judged_states_leaves_spec_gaps_and_errors_out(self) -> None:
        run = directory([state(0), state(1), state(2)], [call(0)])
        verdicts = {(name, number): pair for name in STREAM_NAMES
                    for number, pair in ((0, ("in", "in")), (1, ("spec_gap", "in")), (2, ("error", "error")))}
        self.assertEqual(matrix.o3_judged_state_count(run, verdicts, 0), 2)
        self.assertEqual(matrix.o3_judged_state_count(run, verdicts, 1), 4)
        self.assertIsNone(matrix.o3_judged_state_count(run, None, 0))


class ClosingOneOracleTests(unittest.TestCase):
    def test_a_row_only_o2_catches_is_held_by_o2_alone(self) -> None:
        self.assertEqual(matrix.oracles_that_alone_hold_the_row({"O2"}), ["O2"])
        self.assertEqual(matrix.q3_closing_report({"O2"}), {"O1": True, "O2": False, "O3": True})

    def test_a_row_two_oracles_catch_survives_closing_any_one(self) -> None:
        self.assertEqual(matrix.oracles_that_alone_hold_the_row({"O1", "O2"}), [])
        self.assertEqual(matrix.q3_closing_report({"O1", "O2"}), {"O1": True, "O2": True, "O3": True})

    def test_a_row_nothing_catches_has_no_closing_report(self) -> None:
        self.assertEqual(matrix.q3_closing_report(set()), {})
        self.assertEqual(matrix.oracles_that_alone_hold_the_row(set()), [])


class ThresholdTests(unittest.TestCase):
    def test_the_coverage_threshold_is_thirty_counted_rows(self) -> None:
        self.assertEqual(matrix.MINIMUM_COUNTED_ROWS, 30)
        self.assertTrue(matrix.coverage_enough_to_judge(30))
        self.assertFalse(matrix.coverage_enough_to_judge(29))

    def test_moving_the_threshold_between_d_and_d_plus_one_flips_the_none_side_from_enough_to_not_enough(self) -> None:
        counted = 37
        self.assertTrue(matrix.coverage_enough_to_judge(counted, minimum=counted))
        self.assertFalse(matrix.coverage_enough_to_judge(counted, minimum=counted + 1))

    def test_the_strict_reading_is_unreachable_from_ninety_percent_of_rows_without_a_judgement(self) -> None:
        self.assertTrue(matrix.strict_reading_is_unreachable(27, 30))
        self.assertFalse(matrix.strict_reading_is_unreachable(26, 30))
        self.assertFalse(matrix.strict_reading_is_unreachable(0, 0))

    def test_a_single_point_is_one_red_state_and_moving_it_to_two_flips_the_flag(self) -> None:
        red_by_segment = ((0, 0), (1, 1), (2, 2), (3, 1))
        self.assertEqual(matrix.single_point_segments_of(red_by_segment), (1, 3))
        self.assertEqual(matrix.single_point_segments_of(red_by_segment, red_states_that_make_a_single_point=2), (2,))


class TrajectoryTests(unittest.TestCase):
    def test_peak_positive_segments_last_value_and_first_red(self) -> None:
        states = [state(0, segment=0), state(1, segment=1, pool_checker="red"), state(2, segment=2, pool_checker="red"),
                  state(3, segment=2, pool_checker="red"), state(4, segment=3)]
        path = matrix.trajectory(stream(STREAM_NAMES[0], states, [call(0)]), STREAM_NAMES[0], matrix.ORACLE_TWO, 1, None)
        self.assertEqual(path.red_by_segment, ((0, 0), (1, 1), (2, 2), (3, 0)))
        self.assertEqual((path.peak, path.positive_segments, path.last_segment_value), (2, 2, 0))
        self.assertEqual((path.first_red_segment, path.single_point_segments, path.total()), (1, (1,), 3))


class ClassificationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.baseline = directory([state(0)], [call(0)])

    def test_every_category_has_a_distinguishing_input(self) -> None:
        touched = matrix.touch_report(self.baseline, directory([state(0)], [call(0)], writes="other"))
        same = matrix.touch_report(self.baseline, self.baseline)
        crash = {"new_pool_file_creation": {"not_exported_why": "implementation_crashed_in_a_call:x"}}
        self.assertEqual(classify_row({"status": "compile_failed"}, None, None, {}), "compile_failed")
        self.assertEqual(classify_row({"status": "anchor_not_unique"}, None, None, {}), "anchor_not_unique")
        self.assertEqual(classify_row({"status": "ok", "device_exit": "253"}, self.baseline, same, {}), "timeout")
        self.assertEqual(classify_row({"status": "ok", "device_exit": "252"}, self.baseline, same, {}), "wrapper_ended")
        self.assertEqual(classify_row({"status": "ok", "device_exit": "0"}, self.baseline, touched, crash), "self_crash")
        self.assertEqual(classify_row({"status": "ok", "device_exit": "3"}, self.baseline, touched, {}), CATEGORY_DEVICE_STOP)
        self.assertEqual(classify_row({"status": "ok", "device_exit": "0"}, self.baseline, touched, {}), "counted")
        self.assertEqual(classify_row({"status": "ok", "device_exit": "0"}, self.baseline, same, {}), "untouched")

    def test_an_untouched_row_that_some_oracle_reddened_is_listed(self) -> None:
        red_run = directory([state(0, pool_checker="red")], [call(0)])
        flagged = RowAnalysis(1, "r", {}, red_run, "untouched", matrix.touch_report(self.baseline, self.baseline), None)
        quiet = RowAnalysis(2, "q", {}, self.baseline, "untouched", matrix.touch_report(self.baseline, self.baseline), None)
        self.assertEqual([row.number for row in untouched_but_red_rows([flagged, quiet])], [1])


def labelled_call(index: int, label: str, outcome: str = "ok", model_word: str = "agrees") -> CallRow:
    return CallRow(index, label, index * 10, index * 10 + 10, outcome, model_word, "-", "-", "-")


def run_whose_model_stopped_at(calls: list[CallRow], states: list[StateRow], stopped: int | None) -> RunDirectory:
    status = {"ending": "stopped_by_an_error", "model_stopped_at_call": "none" if stopped is None else str(stopped)}
    return RunDirectory("x", {name: StreamRun(name=name, exported=True, states=list(states), calls=list(calls), status=dict(status))
                              for name in STREAM_NAMES})


DISAGREES = "disagrees:模型说该成、实现拒了"


class RowS5MirrorTests(unittest.TestCase):
    """装置逐行那一趟 S5 的比对器一侧（代码轮第三轮判决 K4-c 甲、丙）：返回了错误而对拍没问模型的调用、与不适用来源不合规的态，各自数出来，期望 0。"""

    def test_an_errored_acquire_left_in_not_modelled_is_counted_and_a_recorded_red_is_not(self) -> None:
        silent = run_whose_model_stopped_at([labelled_call(0, "acquire", "err:X", "not_modelled")], [], 0)
        self.assertEqual(matrix.errored_calls_left_unjudged(silent), {name: 1 for name in STREAM_NAMES})
        recorded = run_whose_model_stopped_at([labelled_call(0, "acquire", "err:X", DISAGREES)], [], 0)
        self.assertEqual(matrix.errored_calls_left_unjudged(recorded), {name: 0 for name in STREAM_NAMES})

    def test_a_call_after_the_stop_on_a_disagreement_that_errored_is_not_counted(self) -> None:
        """第 108 行那一形（登记修订 4）：对拍在发布上判出不同、模型停下，之后的可写挂载出错没比，是读法 6 的合规来源，数 0。"""
        run = run_whose_model_stopped_at([labelled_call(0, "publish", "ok", DISAGREES),
                                          labelled_call(1, "mount_writable", "err:Recovery", "skipped_after_the_model_stopped")], [], 0)
        self.assertEqual(matrix.errored_calls_left_unjudged(run), {name: 0 for name in STREAM_NAMES})

    def test_an_errored_call_left_in_not_modelled_after_a_good_call_is_still_counted(self) -> None:
        run = run_whose_model_stopped_at([labelled_call(0, "publish", "ok", "agrees"),
                                          labelled_call(1, "mount_writable", "err:Recovery", "not_modelled")], [], 1)
        self.assertEqual(matrix.errored_calls_left_unjudged(run), {name: 1 for name in STREAM_NAMES})

    def test_the_unmount_a_panic_and_a_good_acquire_are_not_counted(self) -> None:
        calls = [labelled_call(0, "acquire", "ok", "not_modelled"), labelled_call(1, "unmount", "err:Z", "not_modelled"),
                 labelled_call(2, "publish", "panic", "not_modelled")]
        self.assertEqual(matrix.errored_calls_left_unjudged(run_whose_model_stopped_at(calls, [], 1)), {name: 0 for name in STREAM_NAMES})

    def test_not_applicable_states_count_only_when_the_stop_is_not_the_unmount_or_a_disagreement(self) -> None:
        states = [state(0, crash_judge="na"), state(1, crash_judge="na", batch=2), state(2)]
        every_stream = lambda number: {name: number for name in STREAM_NAMES}  # noqa: E731
        agreed_refusal = [labelled_call(0, "publish", "err:Y", "agrees")]
        self.assertEqual(matrix.not_applicable_states_with_another_source(run_whose_model_stopped_at(agreed_refusal, states, 0)), every_stream(2))
        silent_acquire = [labelled_call(0, "acquire", "err:X", "not_modelled")]
        self.assertEqual(matrix.not_applicable_states_with_another_source(run_whose_model_stopped_at(silent_acquire, states, 0)), every_stream(2))
        recorded = [labelled_call(0, "acquire", "err:X", DISAGREES)]
        self.assertEqual(matrix.not_applicable_states_with_another_source(run_whose_model_stopped_at(recorded, states, 0)), every_stream(0))
        at_the_unmount = [labelled_call(0, "publish"), labelled_call(1, "unmount", "ok", "not_modelled")]
        self.assertEqual(matrix.not_applicable_states_with_another_source(run_whose_model_stopped_at(at_the_unmount, states, 1)), every_stream(0))
        self.assertEqual(matrix.not_applicable_states_with_another_source(run_whose_model_stopped_at(at_the_unmount, states, None)), every_stream(0))
        self.assertEqual(matrix.not_applicable_states_with_another_source(run_whose_model_stopped_at(at_the_unmount, states, 7)), every_stream(2))


if __name__ == "__main__":
    unittest.main()
