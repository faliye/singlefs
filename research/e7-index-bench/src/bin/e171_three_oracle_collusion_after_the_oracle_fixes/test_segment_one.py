"""E171 第 1 次重跑第一段比对器的单测（登记第五节 5.2 末段「比对器单测里手造矩阵」、第九节 V13–V16、第八节判别力自证）。

跑法：`python3 -B -m unittest test_segment_one`（在本目录）。只用手造的流与调用，不读仓里任何产物。
"""

from __future__ import annotations

import hashlib
import os
import tempfile
import unittest

import collusion_matrix as matrix
import segment_one_comparator as comparator
from result_lines import ResultLines
from run_directory import STREAM_NAMES, CallRow, ProbeRow, RecoverySummary, ReturnValueRow, RunDirectory, StateRow, StreamRun

SUMMARY = RecoverySummary("1,3", "file_read", "-", 0)


def state(number: int, *, crash_judge: str = "green", reference_filesystem: str = "green", pool_checker: str = "green",
          batch: int = 1, segment: int = 0, touched_call: int | None = 2, landing: str = "2") -> StateRow:
    return StateRow(number, number, segment, batch, landing, "d", touched_call, crash_judge, "-", pool_checker, "-", True, SUMMARY, SUMMARY, "-",
                    reference_filesystem, "-", "after_every_step", "S_0", "-")


def call(index: int, label: str = "publish", outcome: str = "ok", model_word: str = "agrees") -> CallRow:
    return CallRow(index, label, 0, 0, outcome, model_word, "-", "-", "-", "-", "-")


def stream(name: str, states: list[StateRow] | None = None, calls: list[CallRow] | None = None,
           return_values: list[ReturnValueRow] | None = None, probes: list[ProbeRow] | None = None) -> StreamRun:
    run = StreamRun(name=name, exported=True, states=states or [state(0)], calls=calls or [call(0, "acquire", model_word="not_modelled")],
                    return_values=return_values or [], probes=probes or [], status={"ending": "completed", "model_stopped_at_call": "none"})
    run.write_stream_sha256 = {"ops.tsv": "a"}
    return run


def directory(**by_stream: StreamRun) -> RunDirectory:
    return RunDirectory("手造", {name: by_stream.get(name, stream(name)) for name in STREAM_NAMES})


class OneQuietOracle(unittest.TestCase):
    """读法 1：O1 = 三个子列任一个红。一行只有 RefFS 红：O1 算红、这一行没有「三条全放行」（V13：漏掉 RefFS 子列，这里红）。"""

    def test_a_row_red_only_on_the_reference_filesystem_is_not_all_quiet(self) -> None:
        run = directory(new_pool_file_creation=stream(STREAM_NAMES[0], states=[state(0, reference_filesystem="red")]))
        self.assertTrue(matrix.column_is_red(run, matrix.SUBCOLUMN_REFERENCE_FILESYSTEM, 1, None))
        self.assertTrue(matrix.column_is_red(run, matrix.ORACLE_ONE, 1, None))
        self.assertIs(matrix.all_quiet(run, (matrix.ORACLE_ONE, matrix.ORACLE_TWO), 1, None), False)

    def test_a_row_red_nowhere_is_all_quiet_and_an_abstention_is_not_red(self) -> None:
        run = directory(new_pool_file_creation=stream(STREAM_NAMES[0], states=[state(0, reference_filesystem="abstained")]))
        self.assertIs(matrix.all_quiet(run, (matrix.ORACLE_ONE, matrix.ORACLE_TWO), 1, None), True)
        self.assertIsNone(matrix.all_quiet(run, (matrix.ORACLE_ONE, matrix.ORACLE_THREE_STRICT), 1, None), "O3 没跑记 None")

    def test_a_red_only_in_the_third_stream_drops_out_of_the_two_stream_reading(self) -> None:
        run = directory(first_file_unlinked_then_inodes_across_one_leaf=stream(STREAM_NAMES[2], states=[state(0, pool_checker="red")]))
        self.assertTrue(matrix.column_is_red(run, matrix.ORACLE_TWO, 1, None))
        self.assertFalse(matrix.column_is_red(run, matrix.ORACLE_TWO, 1, None, STREAM_NAMES[:2]))


class ReturnValues(unittest.TestCase):
    """读法 4：交回值顶层字段或内存对象与基线不同的调用进 Q2 的集合；胶水交给模型的观测同不同另报（V14：只看内存对象，这里红）。"""

    def test_a_call_whose_return_value_field_differs_and_whose_observation_is_the_same_is_a_difference(self) -> None:
        base_value = ReturnValueRow(2, "publish", "w", {"root": "r", "units": "u"}, {"allocator": "a"}, "o")
        row_value = ReturnValueRow(2, "publish", "w2", {"root": "r", "units": "u2"}, {"allocator": "a"}, "o")
        baseline = directory(fixed_script_through_e_then_unmount=stream(STREAM_NAMES[1], return_values=[base_value]))
        row = directory(fixed_script_through_e_then_unmount=stream(STREAM_NAMES[1], return_values=[row_value]))
        differences = matrix.return_value_differences(baseline, row)
        self.assertEqual([(difference.call_index, difference.differing_fields, difference.differing_objects, difference.observation_differs)
                          for difference in differences], [(2, ("units",), (), False)])
        self.assertEqual(matrix.red_calls(row)[STREAM_NAMES[1]], 0, "这一行 O1-对拍绿：Q2 判丙的那一形")

    def test_an_object_that_is_unstable_across_two_baselines_is_left_out(self) -> None:
        first = directory(new_pool_file_creation=stream(STREAM_NAMES[0], return_values=[ReturnValueRow(0, "acquire", "w", {}, {"allocator": "a"}, "-")]))
        second = directory(new_pool_file_creation=stream(STREAM_NAMES[0], return_values=[ReturnValueRow(0, "acquire", "w", {}, {"allocator": "b"}, "-")]))
        unstable = matrix.unstable_memory_objects(first, second)
        self.assertEqual(unstable, frozenset({"allocator"}))
        self.assertEqual(matrix.return_value_differences(first, second, unstable), [])


class SelfCrash(unittest.TestCase):
    """读法 5：「可判的调用」只数 O1-对拍 `agrees` / `disagrees`（V15：把 `not_modelled` 也数进去，这里红）；判别力自证：门槛挪到 2，c = 1 由计入转不计入。"""

    def calls_panicking_at(self, index: int) -> list[CallRow]:
        calls = [call(0, "acquire", model_word="not_modelled"), call(1, "warm_up", model_word="not_modelled"), call(2), call(3)]
        calls[index] = call(index, calls[index].label, "panic", "implementation_crashed_not_compared")
        return calls[:index + 1]

    def test_a_panic_in_the_publish_after_the_acquire_and_the_warm_up_has_zero_judged_calls(self) -> None:
        run = directory(fixed_script_through_e_then_unmount=stream(STREAM_NAMES[1], calls=self.calls_panicking_at(2)))
        self.assertEqual(matrix.judged_calls_before_the_first_panic(run), 0)
        self.assertTrue(matrix.counts_as_a_self_crash(run))

    def test_the_panic_control_has_one_judged_call_and_a_threshold_of_two_flips_it(self) -> None:
        run = directory(fixed_script_through_e_then_unmount=stream(STREAM_NAMES[1], calls=self.calls_panicking_at(3)))
        self.assertEqual(matrix.judged_calls_before_the_first_panic(run), 1)
        self.assertFalse(matrix.counts_as_a_self_crash(run))
        self.assertTrue(matrix.counts_as_a_self_crash(run, minimum=2))

    def test_a_stream_without_a_panic_has_no_count(self) -> None:
        self.assertIsNone(matrix.judged_calls_before_the_first_panic(directory()))


def probe(counter_branch: str, second_crop: str, *, mount: str = "refused", undecidable: bool = False, on_the_list: str = "-") -> ProbeRow:
    return ProbeRow(1, 0, 0, undecidable, counter_branch, 4, "1,3", mount, "NewerStateStillUnreadableAfterOneReread", on_the_list, second_crop)


class WitnessProbe(unittest.TestCase):
    """读法 8 / 第六节 Q4：（ii）态全判不合法判乙、有一态判合法判丙（V16：只看（ii）有没有，这里红）；（i）不进候选；Q4-对账两数。"""

    def test_a_counter_branch_state_judged_illegal_is_yi_and_one_judged_legal_is_bing(self) -> None:
        illegal = directory(new_pool_file_creation=stream(STREAM_NAMES[0], probes=[probe("AboveTheLastRecordOfTheSelectedVersion", "illegal:x")]))
        counts = matrix.probe_counts(illegal, STREAM_NAMES[0], 1)
        self.assertEqual(matrix.q4_verdict(counts.counter_branch, counts.counter_branch_judged_legal), "yi")
        legal = directory(new_pool_file_creation=stream(STREAM_NAMES[0], probes=[probe("NeitherRecordReadable", "legal")]))
        counts = matrix.probe_counts(legal, STREAM_NAMES[0], 1)
        self.assertEqual(matrix.q4_verdict(counts.counter_branch, counts.counter_branch_judged_legal), "bing")
        self.assertEqual(matrix.q4_verdict(0, 0), "jia")

    def test_the_undecidable_branch_does_not_enter_the_candidates_and_the_reconciliation_counts_are_counted(self) -> None:
        run = directory(new_pool_file_creation=stream(STREAM_NAMES[0], probes=[
            probe("false", "legal", undecidable=True, on_the_list=matrix.WITNESS_REASON_ON_THE_LIST),
            probe("false", "-", mount="mounted"),
            probe("false", "legal", on_the_list=matrix.WITNESS_REASON_ON_THE_LIST),
            probe("AboveTheLastRecordOfTheSelectedVersion", "-", mount="mounted"),
        ]))
        counts = matrix.probe_counts(run, STREAM_NAMES[0], 1)
        self.assertEqual((counts.counter_branch, counts.undecidable_branch), (1, 1))
        self.assertEqual((counts.witness_refusals_without_either_branch, counts.counter_branch_mounted), (1, 1))


class Trajectories(unittest.TestCase):
    """第八节：单点红（一段只红一个态）；判别力自证：「两个态才算红」时单点红的格由红转不红。"""

    def test_a_segment_red_in_one_state_is_a_single_point_and_a_threshold_of_two_drops_it(self) -> None:
        states = [state(0, segment=1, crash_judge="red"), state(1, segment=1), state(2, segment=4, crash_judge="red"),
                  state(3, segment=4, crash_judge="red")]
        run = stream(STREAM_NAMES[1], states=states)
        path = matrix.trajectory(run, STREAM_NAMES[1], matrix.SUBCOLUMN_CRASH_JUDGE, 1, None)
        self.assertEqual((path.peak, path.positive_segments, path.last_segment_value), (2, 2, 2))
        self.assertEqual(path.single_point_segments, (1,))
        self.assertEqual(matrix.trajectory(run, STREAM_NAMES[1], matrix.SUBCOLUMN_CRASH_JUDGE, 1, None, 2).single_point_segments, (4,))


class ConfirmedLoss(unittest.TestCase):
    """第六节 Q5：任一子列红判甲、两者都绿判乙、全弃权走 F8、L = 0 走 F5；判别力自证：L 门槛挪到实测 L + 1，Q5 由够判转 F5。"""

    def test_the_q5_verdicts_follow_the_registered_rules(self) -> None:
        self.assertEqual(matrix.q5_verdict(9, 9, 0, 0, 0), "jia")
        self.assertEqual(matrix.q5_verdict(9, 0, 0, 3, 0), "yi")
        self.assertEqual(matrix.q5_verdict(9, 0, 0, 0, 9), "not_enough_f8")
        self.assertEqual(matrix.q5_verdict(0, 0, 0, 0, 0), "not_enough_f5")
        self.assertEqual(matrix.q5_verdict(9, 9, 0, 0, 0, minimum_lost=10), "not_enough_f5")

    def test_no_red_and_no_both_green_over_abstentions_and_judge_panics_is_f8(self) -> None:
        # 修订 5 第 5.1 节第 7 条：L 个态上没有一态红、也没有一态两者都绿（全是弃权或 judge_panic）照 F8 记不够判
        self.assertEqual(matrix.q5_verdict(9, 0, 0, 0, 4), "not_enough_f8", "5 态 RefFS judge_panic、4 态弃权")
        self.assertEqual(matrix.q5_verdict(9, 0, 0, 0, 0), "not_enough_f8", "9 态全是 judge_panic")

    def test_every_abstention_comes_before_a_model_red(self) -> None:
        # 第一段比对器原有的次序：RefFS 全弃权先走 F8，即使模型崩溃判官有红
        self.assertEqual(matrix.q5_verdict(9, 0, 3, 0, 9), "not_enough_f8")

    def test_the_comparator_reads_the_q5_line_and_a_main_point_that_did_not_run_is_not_enough(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            product = os.path.join(scratch, "q5.out")
            with open(product, "w", encoding="utf-8") as handle:
                handle.write("E7RESULT name=q5 sampling_point=reverse states=72 lost_the_confirmed_version=9 reference_filesystem_red=9 "
                             "model_crash_judge_red=0 both_green=0 reference_filesystem_abstained=0 not_lost_o1_red=0 a10_states_equal_the_product=true\n")
            not_run = os.path.join(scratch, "q5-main.out")
            with open(not_run, "w", encoding="utf-8") as handle:
                handle.write("E7RESULT name=q5 sampling_point=main not_run=too_many_states states=1207959552\n")
            lines = ResultLines(os.path.join(scratch, "out"))
            self.assertEqual(comparator.q5_lines(lines, product, "reverse"), "jia")
            self.assertEqual(comparator.q5_lines(lines, not_run, "main"), "not_enough")
            lines.finish()


class SegmentClosing(unittest.TestCase):
    """比对器自己按 `SegmentClosingRule` 重切段（A11 的现算那一份）：屏障在每块有写的盘都放行之后才关段；FUA 放行它自己那块盘，
    别的盘上没有没放行的写时它自己就关段（下标 4），流尾没关上的那一段是 None。"""

    def test_a_barrier_on_one_device_does_not_close_while_another_device_has_unreleased_writes(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            rows = ["0\t0\twrite\t0\t512\t0\t-", "1\t1\twrite\t0\t512\t512\t-", "2\t0\tbarrier\t0\t0\t-\t-", "3\t1\tbarrier\t0\t0\t-\t-",
                    "4\t0\twrite_fua\t0\t512\t1024\t-", "5\t1\twrite\t0\t512\t1536\t-"]
            with open(os.path.join(scratch, "ops.tsv"), "w", encoding="utf-8") as handle:
                handle.write("\n".join(rows) + "\n")
            self.assertEqual(comparator.segment_closing_indexes(scratch), [3, 4, None])
            self.assertEqual(comparator.write_stream_indexes(scratch), [0, 1, 4, 5])


def touched_call_one_write_at_a_time(landing: str, first_operations: list[int], write_indexes: list[int]) -> int | None:
    """A11 原来的逐个写取最大值的写法（比对器改成「最后一个落了的写」之前那一份），留在这里当对拍的参照。"""
    return max((max((call for call, first in enumerate(first_operations) if first <= write_indexes[write]), default=-1)
                for write, digit in enumerate(landing) if digit != "0"), default=None)


class TouchedCallOfALanding(unittest.TestCase):
    """A11 的现算那一份：「最后一个落了的写所在的调用」与逐个写取最大值逐态同值（调用的第一条 op 不按序也一样）；写的下标不递增就抛错。"""

    def test_the_last_landed_write_gives_the_same_call_as_the_maximum_over_every_landed_write(self) -> None:
        write_indexes = [1, 2, 4, 6, 7, 9, 11]
        for first_operations in ([0, 3, 5, 8], [0, 8, 3, 5], [2, 2, 10], [5]):
            call_of_write = comparator.calls_reaching_each_write(first_operations, write_indexes)
            for landing in ("0000000", "1000000", "0000002", "2020000", "0100100", "2222222", "0000010", "0001000"):
                self.assertEqual(comparator.last_touched_call_of_a_landing(landing, call_of_write),
                                 touched_call_one_write_at_a_time(landing, first_operations, write_indexes), (first_operations, landing))

    def test_write_indexes_that_do_not_increase_are_refused(self) -> None:
        with self.assertRaises(ValueError):
            comparator.calls_reaching_each_write([0, 3], [1, 4, 4])


class OracleVerdictColumns(unittest.TestCase):
    """修订 5 第 5.1 节第 5 条在 `oracle-verdicts.tsv` 末尾加了 `piece_panics`：25 列照读，少一列（修订 5 之前的 24 列）拒读。"""

    def write_pair(self, scratch: str, verdict_columns: int) -> tuple[str, str]:
        states, verdicts = os.path.join(scratch, "states.tsv"), os.path.join(scratch, "oracle-verdicts.tsv")
        with open(states, "w", encoding="utf-8") as handle:
            handle.write("0\t0\t0\t1\t20\td\n")
        row = ["0", "1", "0", "0", "-", "green", "-", "green", "-", "true", "0,0", "no_file", "-", "0", "0,0", "no_file", "-", "0", "-",
               "green", "-", "call_0", "S_0", "-", "consulted_recovery"][:verdict_columns]
        with open(verdicts, "w", encoding="utf-8") as handle:
            handle.write("\t".join(row) + "\n")
        return states, verdicts

    def test_twenty_five_columns_are_read_and_twenty_four_are_refused(self) -> None:
        from run_directory import read_states
        with tempfile.TemporaryDirectory() as scratch:
            read = read_states(*self.write_pair(scratch, 25))
            self.assertEqual((read[0].recovery_panic, read[0].piece_panics), ("-", "consulted_recovery"))
            with self.assertRaises(ValueError):
                read_states(*self.write_pair(scratch, 24))


class OnlyThisRun(unittest.TestCase):
    """修订 8 第 8.5 节第 1 条：给到另一趟的产物前缀或导出根就退出、不写产物。"""

    def test_a_product_of_another_run_is_refused_before_anything_is_written(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            dumps = os.path.join(scratch, "revision-6")
            os.makedirs(os.path.join(dumps, "baseline-run1"))
            other_dumps = os.path.join(scratch, "revision-8", "baseline-run1")
            prefix, suffix = "e171-r1-x-2026-10-05-s1-", "-r6-r3.out"
            self.assertEqual(comparator.inputs_outside_this_run(dumps, prefix, suffix, [os.path.join(dumps, "baseline-run1")],
                                                                [os.path.join(scratch, prefix + "q5-main-r6-r3.out")]), [])
            off_product = os.path.join(scratch, prefix + "q5-main-r8.out")
            self.assertEqual(comparator.inputs_outside_this_run(dumps, prefix, suffix, [other_dumps], [off_product]), [other_dumps, off_product])
            output = os.path.join(scratch, "comparator.out")
            exit_code = comparator.main(["--baseline", os.path.join(dumps, "baseline-run1"), "--q5-main", off_product, "--output", output,
                                         "--dumps-root", dumps, "--product-prefix", prefix, f"--product-suffix={suffix}"])
            self.assertEqual(exit_code, 2)
            self.assertFalse(os.path.exists(output))


class ThirdStreamCoverage(unittest.TestCase):
    """第六节「流三覆盖（O3）」与 F7：流三批 1、批 2 上 O3 严格一态都没给出「在 / 不在」记 F7；批 3 的态不算；O3 没跑记 not_run。"""

    def test_f7_follows_the_states_the_strict_reading_decided(self) -> None:
        third = STREAM_NAMES[2]
        baseline = directory(first_file_unlinked_then_inodes_across_one_leaf=stream(third, states=[state(0), state(1, batch=2), state(2, batch=3)]))
        with tempfile.TemporaryDirectory() as scratch:
            lines = ResultLines(os.path.join(scratch, "out"))
            self.assertEqual(comparator.o3_third_stream_coverage(lines, baseline, {(third, 2): ("in", "in")}), "true")
            self.assertEqual(comparator.o3_third_stream_coverage(lines, baseline, {(third, 1): ("not_in", "in")}), "false")
            self.assertEqual(comparator.o3_third_stream_coverage(lines, baseline, None), "not_run")
            lines.finish()


class EmptyMutation(unittest.TestCase):
    """第七节 A1 的空变异那一半：与基线相同的那一行 0 处红、未触达；一态红记 1 处；写流摘要不同记触达。"""

    def test_the_empty_mutation_counts_reds_and_touch_against_the_baseline(self) -> None:
        baseline = directory()
        changed_write_stream = stream(STREAM_NAMES[1])
        changed_write_stream.write_stream_sha256 = {"ops.tsv": "b"}
        with tempfile.TemporaryDirectory() as scratch:
            lines = ResultLines(os.path.join(scratch, "out"))
            self.assertEqual(comparator.empty_mutation_a1(lines, baseline, directory(), None), (0, False))
            red = directory(new_pool_file_creation=stream(STREAM_NAMES[0], states=[state(0, pool_checker="red")]))
            self.assertEqual(comparator.empty_mutation_a1(lines, baseline, red, None), (1, False))
            self.assertEqual(comparator.empty_mutation_a1(lines, baseline, directory(fixed_script_through_e_then_unmount=changed_write_stream), None),
                             (0, True))
            lines.finish()


def result_lines_in(path: str) -> list[dict[str, str]]:
    from run_directory import parse_result_line
    with open(path, encoding="utf-8") as handle:
        return [fields for fields in (parse_result_line(line.rstrip("\n")) for line in handle) if fields is not None]


ILLEGAL_WITH_A_CONDITION = "illegal:witness_criterion_true_without_read_faults:RecordAtTheWitnessedCounterNewerThanTheSelectedVersion"


def witness_probe(state_row: StateRow, counter_branch: str, second_crop: str, *, undecidable: bool = False) -> ProbeRow:
    return ProbeRow(state_row.batch, state_row.ordinal, state_row.state_number, undecidable, counter_branch, 3, "1,2", "refused",
                    "NewerStateStillUnreadableAfterOneReread", matrix.WITNESS_REASON_ON_THE_LIST, second_crop)


class CountSeriesSummary(unittest.TestCase):
    """第八节与「端点不是轨迹」：峰值取第一次出现的格、为正的格数不数 0 格、末格是格号最大的那一格（为 0 照报 0，不退到最后一个为正的格）。"""

    def test_peak_positive_cells_and_last_cell_of_a_series_whose_last_cell_is_zero(self) -> None:
        series = matrix.count_series(((3, 1), (0, 0), (1, 3), (2, 3), (4, 0)))
        self.assertEqual((series.peak, series.peak_at, series.positive_cells, series.last_cell_value), (3, 1, 3, 0))
        self.assertEqual(matrix.positive_cells_text(((3, 1), (0, 0), (1, 3), (2, 3), (4, 0))), "1:3,2:3,3:1")

    def test_an_all_zero_or_empty_series_has_no_peak_cell(self) -> None:
        self.assertEqual(matrix.count_series(((0, 0), (1, 0))), matrix.CountSeries(0, None, 0, 0))
        self.assertEqual(matrix.count_series(()), matrix.CountSeries(0, None, 0, 0))
        self.assertEqual(matrix.positive_cells_text(((0, 0),)), "-")


class CounterBranchStateList(unittest.TestCase):
    """第六节 Q4-（ii）逐态清单：只列（ii）为真的态（（i）判不出那一支与都不为真的不列），按批分开；落地向量摘要是落地串的 sha256；
    第二截判定拆成判定与没满足的条件（条件里还带冒号）；段接回发出它的调用与第几次建 inode。"""

    def run_with_probes(self) -> RunDirectory:
        states = [state(0, segment=0, landing="20"), state(1, segment=1, landing="22"), state(2, segment=1), state(3, segment=2),
                  state(4, segment=2, batch=2, landing="21")]
        probes = [witness_probe(states[0], "RecordAtTheWitnessedCounterIsNewer", ILLEGAL_WITH_A_CONDITION),
                  witness_probe(states[1], "NeitherRecordReadable", "legal"),
                  witness_probe(states[2], "false", "legal", undecidable=True), witness_probe(states[3], "false", "-"),
                  witness_probe(states[4], "AboveTheLastRecordOfTheSelectedVersion", "illegal:x")]
        return directory(fixed_script_through_e_then_unmount=stream(STREAM_NAMES[1], states=states, probes=probes))

    def test_only_counter_branch_states_are_listed_with_their_fields(self) -> None:
        calls = comparator.SegmentCalls((1, 2, 3), {2: 1, 3: 2}, 2)
        with tempfile.TemporaryDirectory() as scratch:
            output = os.path.join(scratch, "out")
            lines = ResultLines(output)
            listed = comparator.q4_counter_branch_state_lines(lines, {"row_242": self.run_with_probes()}, segment_calls_of=lambda run, name: calls)
            lines.finish()
            found = [fields for fields in result_lines_in(output) if fields["name"] == "q4_counter_branch_state"]
        self.assertEqual(listed, 3)
        self.assertEqual([(fields["batch"], fields["state"], fields["segment"], fields["call_of_the_segment"], fields["inode_creation"],
                           fields["second_crop"], fields["unmet_condition"], fields["counter_sentence"]) for fields in found],
                         [("1", "0", "0", "1", "-", "illegal", "witness_criterion_true_without_read_faults:RecordAtTheWitnessedCounterNewerThanTheSelectedVersion",
                           "RecordAtTheWitnessedCounterIsNewer"),
                          ("1", "1", "1", "2", "1", "legal", "-", "NeitherRecordReadable"),
                          ("2", "4", "2", "3", "2", "illegal", "x", "AboveTheLastRecordOfTheSelectedVersion")])
        self.assertEqual([fields["landing_sha256"] for fields in found],
                         [hashlib.sha256(landing.encode("utf-8")).hexdigest() for landing in ("20", "22", "21")])
        self.assertEqual({(fields["trip"], fields["witnessed_counter"], fields["selected_version"], fields["mount"]) for fields in found},
                         {("row_242", "3", "1,2", "refused")})

    def test_a_probe_that_does_not_match_its_state_is_refused(self) -> None:
        run = self.run_with_probes()
        stray = ProbeRow(1, 99, 0, False, "NeitherRecordReadable", 3, "1,2", "refused", "-", "-", "legal")
        run.streams[STREAM_NAMES[1]].probes.append(stray)
        with self.assertRaises(ValueError):
            matrix.counter_branch_states(run.streams[STREAM_NAMES[1]], 1)

    def test_the_second_crop_splits_at_the_first_colon(self) -> None:
        self.assertEqual(matrix.second_crop_verdict_and_unmet_condition(ILLEGAL_WITH_A_CONDITION),
                         ("illegal", "witness_criterion_true_without_read_faults:RecordAtTheWitnessedCounterNewerThanTheSelectedVersion"))
        self.assertEqual(matrix.second_crop_verdict_and_unmet_condition("legal"), ("legal", "-"))
        self.assertEqual(matrix.second_crop_verdict_and_unmet_condition("-"), ("-", "-"))


class WitnessTrajectories(unittest.TestCase):
    """第八节 Q4：按段号排开（ii）态数与其中判合法的态数；流三另按第几次建 inode 排开，不在建 inode 调用里的段与全持久态那一段单列。"""

    def third_stream(self) -> StreamRun:
        states = [state(0, segment=0), state(1, segment=1), state(2, segment=1), state(3, segment=2), state(4, segment=3)]
        probes = [witness_probe(states[0], "RecordAtTheWitnessedCounterIsNewer", "illegal:x"),
                  witness_probe(states[1], "NeitherRecordReadable", "legal"), witness_probe(states[2], "NeitherRecordReadable", "illegal:y"),
                  witness_probe(states[3], "false", "-"), witness_probe(states[4], "AboveTheLastRecordOfTheSelectedVersion", "legal")]
        return stream(STREAM_NAMES[2], states=states, probes=probes)

    def test_counts_by_segment_keep_the_zero_segments_and_count_only_legal_ones_as_legal(self) -> None:
        counter, legal = matrix.counter_branch_by_segment(self.third_stream(), 1)
        self.assertEqual(counter, ((0, 1), (1, 2), (2, 0), (3, 1)))
        self.assertEqual(legal, ((0, 0), (1, 1), (2, 0), (3, 1)))

    def test_the_trajectory_lines_by_segment_and_by_inode_creation(self) -> None:
        # 段 0 由第 0 次调用（取号）发出、段 1 由第 2 次（第 1 次建 inode）、段 2 由第 3 次（第 2 次建 inode）；段 3 等于段数，是全持久态
        calls = comparator.SegmentCalls((0, 2, 3), {2: 1, 3: 2}, 2)
        run = directory(first_file_unlinked_then_inodes_across_one_leaf=self.third_stream())
        with tempfile.TemporaryDirectory() as scratch:
            output = os.path.join(scratch, "out")
            lines = ResultLines(output)
            comparator.q4_trajectory_lines(lines, {"row_269": run}, segment_calls_of=lambda run, name: calls)
            lines.finish()
            found = result_lines_in(output)
        by_segment = next(fields for fields in found if fields["name"] == "q4_trajectory" and fields["stream"] == STREAM_NAMES[2] and fields["batch"] == "1")
        self.assertEqual({key: by_segment[key] for key in ("segments", "first_segment", "last_segment", "counter_branch_by_cell", "counter_branch_peak",
                                                           "counter_branch_peak_at", "counter_branch_positive_cells", "counter_branch_last",
                                                           "judged_legal_by_cell", "judged_legal_last")},
                         {"segments": "4", "first_segment": "0", "last_segment": "3", "counter_branch_by_cell": "0:1,1:2,3:1", "counter_branch_peak": "2",
                          "counter_branch_peak_at": "1", "counter_branch_positive_cells": "3", "counter_branch_last": "1",
                          "judged_legal_by_cell": "1:1,3:1", "judged_legal_last": "1"})
        by_creation = next(fields for fields in found if fields["name"] == "q4_trajectory_by_inode_creation" and fields["batch"] == "1")
        self.assertEqual({key: by_creation[key] for key in ("create_inode_calls", "counter_branch_outside_create_inode_calls", "counter_branch_after_every_call",
                                                            "judged_legal_after_every_call", "counter_branch_by_cell", "counter_branch_last",
                                                            "judged_legal_by_cell", "judged_legal_positive_cells")},
                         {"create_inode_calls": "2", "counter_branch_outside_create_inode_calls": "1", "counter_branch_after_every_call": "1",
                          "judged_legal_after_every_call": "1", "counter_branch_by_cell": "1:2", "counter_branch_last": "0",
                          "judged_legal_by_cell": "1:1", "judged_legal_positive_cells": "1"})

    def test_a_segment_beyond_the_full_state_is_refused(self) -> None:
        calls = comparator.SegmentCalls((0, 2), {2: 1}, 1)
        self.assertIsNone(comparator.call_of_a_segment(calls, 2))
        with self.assertRaises(ValueError):
            comparator.call_of_a_segment(calls, 3)


class SegmentCallsOfAnExport(unittest.TestCase):
    """一段的调用取这一段最后一个写所在的调用（段 1 跨第 1、2 次调用，取第 2 次）；FUA 自己关段；流尾没关上的那一段取最后一个写；
    建 inode 的调用从 1 数。"""

    def test_calls_and_inode_creations_of_each_segment(self) -> None:
        ops = ["0\t0\twrite\t0\t512\t0\t-", "1\t0\tbarrier\t0\t0\t-\t-", "2\t0\twrite\t0\t512\t512\t-", "3\t0\twrite\t0\t512\t1024\t-",
               "4\t0\tbarrier\t0\t0\t-\t-", "5\t0\twrite_fua\t0\t512\t1536\t-", "6\t0\twrite\t0\t512\t2048\t-"]
        calls = ["0\tacquire\t0\t2\tok", "1\tpublish\t2\t3\tok", "2\tcreate_inode\t3\t5\tok", "3\tcreate_inode\t5\t6\tok", "4\tcreate_inode\t6\t7\tok"]
        with tempfile.TemporaryDirectory() as scratch:
            stream_directory = os.path.join(scratch, STREAM_NAMES[2])
            os.makedirs(stream_directory)
            for name, rows in (("ops.tsv", ops), ("calls.tsv", calls)):
                with open(os.path.join(stream_directory, name), "w", encoding="utf-8") as handle:
                    handle.write("\n".join(rows) + "\n")
            found = comparator.segment_calls_from_the_export(RunDirectory(scratch, {}), STREAM_NAMES[2])
        self.assertEqual(found, comparator.SegmentCalls((0, 2, 3, 4), {2: 1, 3: 2, 4: 3}, 3))


def q5_product(scratch: str, name: str, reference_filesystem_red: int, extra: str = "") -> str:
    path = os.path.join(scratch, name)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(f"E7RESULT name=q5 sampling_point=main_x states=8 last_segment_writes=3 lost_the_confirmed_version=6 "
                     f"reference_filesystem_red={reference_filesystem_red} model_crash_judge_red=1 both_green=0 reference_filesystem_abstained=0\n"
                     "E7RESULT name=q5_trajectory sampling_point=main_x writes_landed_in_the_last_segment=0 lost_states=2 reference_filesystem_red=2 model_crash_judge_red=0\n"
                     "E7RESULT name=q5_trajectory sampling_point=other writes_landed_in_the_last_segment=3 lost_states=7 reference_filesystem_red=7 model_crash_judge_red=7\n"
                     "E7RESULT name=q5_trajectory sampling_point=main_x writes_landed_in_the_last_segment=1 lost_states=4 reference_filesystem_red=3 model_crash_judge_red=1\n"
                     + extra)
    return path


class ConfirmedLossTrajectory(unittest.TestCase):
    """第八节 Q5：格补成第 0 至末段写数（没有 L 态的格记 0，末格是末段全落那一格）；只取同一取样点的格；各格之和与 `name=q5` 行核一遍。"""

    def lines_of(self, product: str) -> list[dict[str, str]]:
        with tempfile.TemporaryDirectory() as scratch:
            output = os.path.join(scratch, "out")
            lines = ResultLines(output)
            try:
                comparator.q5_trajectory_lines(lines, product, "main")
            finally:
                lines.finish()
            return result_lines_in(output)

    def test_cells_are_filled_up_to_the_last_segment_writes_and_add_up(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            found = self.lines_of(q5_product(scratch, "q5.out", 5))
        self.assertEqual([(fields["writes_landed_in_the_last_segment"], fields["lost_states"], fields["reference_filesystem_red"], fields["model_crash_judge_red"])
                          for fields in found if fields["name"] == "q5_trajectory_cell"],
                         [("0", "2", "2", "0"), ("1", "4", "3", "1"), ("2", "0", "0", "0"), ("3", "0", "0", "0")])
        summary = next(fields for fields in found if fields["name"] == "q5_trajectory")
        self.assertEqual({key: summary[key] for key in ("cells", "cells_add_up_to_the_q5_line", "reference_filesystem_red_by_cell", "reference_filesystem_red_peak",
                                                         "reference_filesystem_red_peak_at", "reference_filesystem_red_positive_cells", "reference_filesystem_red_last",
                                                         "model_crash_judge_red_by_cell", "model_crash_judge_red_last")},
                         {"cells": "4", "cells_add_up_to_the_q5_line": "true", "reference_filesystem_red_by_cell": "0:2,1:3", "reference_filesystem_red_peak": "3",
                          "reference_filesystem_red_peak_at": "1", "reference_filesystem_red_positive_cells": "2", "reference_filesystem_red_last": "0",
                          "model_crash_judge_red_by_cell": "1:1", "model_crash_judge_red_last": "0"})

    def test_cells_that_do_not_add_up_and_a_point_that_did_not_run_are_reported(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            mismatched = self.lines_of(q5_product(scratch, "q5.out", 6))
            not_run_path = os.path.join(scratch, "not-run.out")
            with open(not_run_path, "w", encoding="utf-8") as handle:
                handle.write("E7RESULT name=q5 sampling_point=main not_run=too_many_states states=1207959552\n")
            not_run = self.lines_of(not_run_path)
            duplicate = q5_product(scratch, "dup.out", 5, "E7RESULT name=q5_trajectory sampling_point=main_x writes_landed_in_the_last_segment=1 "
                                                          "lost_states=1 reference_filesystem_red=1 model_crash_judge_red=0\n")
            with self.assertRaises(ValueError):
                self.lines_of(duplicate)
        self.assertEqual(next(fields for fields in mismatched if fields["name"] == "q5_trajectory")["cells_add_up_to_the_q5_line"], "false")
        self.assertEqual([(fields["name"], fields.get("ran")) for fields in not_run if fields["name"] != "done"], [("q5_trajectory", "false")])


if __name__ == "__main__":
    unittest.main()
