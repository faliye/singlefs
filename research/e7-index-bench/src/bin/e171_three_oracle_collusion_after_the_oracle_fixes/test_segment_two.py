"""E171 第 1 次重跑第二段比对器的单测（登记第六节 Q2-集合、Q2-判、Q2-观测、Q3-分母，第八节「Q3 的分母」两批，作废 Z6）。

跑法：`python3 -B -m unittest test_segment_two`（在本目录）。只用手造的流、调用与临时目录，不读仓里任何产物。
手造流的帮手借第一段单测的（`test_segment_one.state` 等），只取函数、不取它的用例类。
"""

from __future__ import annotations

import dataclasses
import os
import tempfile
import unittest

import collusion_matrix as matrix
import segment_two_comparator as comparator
from run_directory import STREAM_NAMES, RecoverySummary, ReturnValueRow
from test_segment_one import call, directory, state, stream

FIRST, SECOND = STREAM_NAMES[0], STREAM_NAMES[1]
OTHER_SUMMARY = RecoverySummary("1,4", "file_read", "-", 0)


def two_batches(consulted_first: RecoverySummary | None = None, consulted_second: RecoverySummary | None = None) -> list:
    """流上两个取样态：批 1 一态、批 2 一态；可分别换掉看 journal 那一遍的恢复报告。"""
    first, second = state(0, batch=1), state(1, batch=2)
    if consulted_first is not None:
        first = dataclasses.replace(first, consulted=consulted_first)
    if consulted_second is not None:
        second = dataclasses.replace(second, consulted=consulted_second)
    return [first, second]


def run_with(**first_stream: object):
    return directory(new_pool_file_creation=stream(FIRST, **first_stream))


class TouchOnEachBatch(unittest.TestCase):
    """第八节「Q3 的分母」：批 1 定触达与批 2 定触达；批 1 与 `matrix.touch_report` 同值（换批靠只留那一批的态、改记批 1）。"""

    def test_the_first_batch_agrees_with_the_matrix_and_a_second_batch_difference_touches_only_the_second_batch(self) -> None:
        baseline = run_with(states=two_batches())
        row = run_with(states=two_batches(consulted_second=OTHER_SUMMARY))
        self.assertEqual(comparator.touch_report_on_the_batch(baseline, row, 1), matrix.touch_report(baseline, row))
        self.assertFalse(comparator.touch_report_on_the_batch(baseline, row, 1).touched())
        self.assertTrue(comparator.touch_report_on_the_batch(baseline, row, 2).touched())

    def test_a_first_batch_difference_touches_only_the_first_batch(self) -> None:
        baseline = run_with(states=two_batches())
        row = run_with(states=two_batches(consulted_first=OTHER_SUMMARY))
        self.assertTrue(comparator.touch_report_on_the_batch(baseline, row, 1).touched())
        self.assertFalse(comparator.touch_report_on_the_batch(baseline, row, 2).touched())

    def test_a_different_write_stream_touches_both_batches(self) -> None:
        baseline = run_with(states=two_batches())
        row = run_with(states=two_batches())
        row.streams[FIRST].write_stream_sha256 = {"ops.tsv": "b"}
        self.assertEqual([comparator.touch_report_on_the_batch(baseline, row, batch).touched() for batch in (1, 2)], [True, True])


class CategoriesOfARow(unittest.TestCase):
    """读法 5：驱动定位失败、装置那一步的结局各一类；跑成了的按自崩、触达、只在不看 journal 那一遍不同、未触达分。"""

    def test_the_driver_and_device_outcomes_map_to_their_own_categories(self) -> None:
        cases = {("not_found", "-"): "not_found", ("compile_failed", "-"): "compile_failed", ("ok", "253"): "timeout",
                 ("ok", "251"): "wrapper_ended", ("ok", "3"): "device_stop", ("ok", "7"): "device_exit_7", ("ok", "0"): None}
        for (status, exit_code), expected in cases.items():
            self.assertEqual(comparator.not_ran_category({"status": status, "device_exit": exit_code}), expected, (status, exit_code))

    def test_a_run_row_is_a_self_crash_then_counted_then_ignored_only_then_untouched(self) -> None:
        touched = matrix.TouchReport(True, False, False, False)
        ignored_only = matrix.TouchReport(False, False, False, True)
        quiet = matrix.TouchReport(False, False, False, False)
        self.assertEqual(comparator.ran_category(touched, True), "self_crash")
        self.assertEqual(comparator.ran_category(touched, False), "counted")
        self.assertEqual(comparator.ran_category(ignored_only, False), "only_in_the_ignored_pass")
        self.assertEqual(comparator.ran_category(quiet, False), "untouched")


class SelfCrashReadings(unittest.TestCase):
    """读法 5 两种算法：第 1 次重跑只把「调用 panic 之前可判的调用为 0」记自崩；原登记把调用 panic 与态上的恢复 panic 一律记自崩。"""

    def panicking_at(self, index: int) -> list:
        calls = [call(0, "acquire", model_word="not_modelled"), call(1, "warm_up", model_word="not_modelled"), call(2), call(3)]
        calls[index] = call(index, calls[index].label, "panic", "implementation_crashed_not_compared")
        return calls[:index + 1]

    def test_a_panic_after_one_judged_call_is_counted_by_the_registration_and_a_self_crash_by_the_original_reading(self) -> None:
        baseline = directory(fixed_script_through_e_then_unmount=stream(SECOND, calls=self.panicking_at(3)[:3]))
        row_run = directory(fixed_script_through_e_then_unmount=stream(SECOND, calls=self.panicking_at(3)))
        row = comparator.reading_of(125, {"status": "ok", "device_exit": "0"}, row_run, baseline)
        self.assertEqual(row.category("registration", 1), "counted")
        self.assertEqual(row.category("original_reading_5", 1), "self_crash")
        self.assertEqual(comparator.self_crash_place(row), f"{SECOND}:call_3:judged_1")

    def test_a_panic_with_no_judged_call_before_it_is_a_self_crash_by_both_readings(self) -> None:
        row_run = directory(fixed_script_through_e_then_unmount=stream(SECOND, calls=self.panicking_at(2)))
        row = comparator.reading_of(6, {"status": "ok", "device_exit": "0"}, row_run, directory())
        self.assertEqual([row.category(reading, 1) for reading in comparator.READINGS], ["self_crash", "self_crash"])

    def test_a_recovery_panic_on_a_state_alone_is_a_self_crash_only_by_the_original_reading(self) -> None:
        states = [dataclasses.replace(state(0), recovery_panic="assertion_failed")]
        row_run = run_with(states=states)
        baseline = run_with(states=[state(0)])
        self.assertEqual(comparator.recovery_panic_states(row_run), 1)
        self.assertTrue(comparator.is_a_self_crash_by_the_original_reading_5(row_run))
        self.assertFalse(matrix.counts_as_a_self_crash(row_run))
        row = comparator.reading_of(9, {"status": "ok", "device_exit": "0"}, row_run, baseline)
        self.assertEqual(row.category("registration", 1), "untouched")
        self.assertEqual(comparator.self_crash_place(row), "recovery_panic_states_1")


def stream_line(name: str, *, exported: str = "false", why: str = "pipeline_panicked:assertion", calls: str = "0") -> dict[str, str]:
    return {"stream": name, "exported": exported, "not_exported_why": why, "calls": calls}


class PanicBeforeTheFirstCall(unittest.TestCase):
    """流一、流二都在 mkfs 那一步 panic、一次调用没发：没有导出，两种读法都记自崩；有一条导出了或发过调用，就不是这一种（记 `dump_missing`）。"""

    def test_both_streams_panicking_before_any_call_is_a_self_crash(self) -> None:
        lines = [stream_line(FIRST), stream_line(SECOND)]
        self.assertTrue(comparator.panicked_before_the_first_call(lines))
        row = comparator.reading_of(85, {"status": "ok", "device_exit": "0"}, None, directory(), lines)
        self.assertEqual([row.category(reading, batch) for reading in comparator.READINGS for batch in comparator.BATCHES], ["self_crash"] * 4)
        self.assertEqual(comparator.self_crash_place(row), "before_the_first_call")
        self.assertFalse(row.not_located())

    def test_a_stream_that_sent_a_call_or_was_exported_is_not_a_panic_before_the_first_call(self) -> None:
        self.assertFalse(comparator.panicked_before_the_first_call([stream_line(FIRST), stream_line(SECOND, calls="1")]))
        self.assertFalse(comparator.panicked_before_the_first_call([stream_line(FIRST), stream_line(SECOND, exported="true", why="-")]))
        self.assertFalse(comparator.panicked_before_the_first_call([stream_line(FIRST)]))
        row = comparator.reading_of(85, {"status": "ok", "device_exit": "0"}, None, directory(), [stream_line(FIRST)])
        self.assertEqual(row.category("registration", 1), "dump_missing")


def reading(number: int, category_inputs: str) -> comparator.RowReading:
    """手造一行：`counted`、`untouched`、`first_batch_only`（只在批 1 恢复报告上不同）、`self_crash`、`not_found`。"""
    if category_inputs == "not_found":
        return comparator.RowReading(number, {"status": "not_found"}, "not_found", None, {}, {})
    touched = matrix.TouchReport(True, False, False, False)
    quiet = matrix.TouchReport(False, False, False, False)
    by_batch = {"counted": (touched, touched), "untouched": (quiet, quiet), "first_batch_only": (matrix.TouchReport(False, False, True, False), quiet),
                "self_crash": (touched, touched)}[category_inputs]
    crash = category_inputs == "self_crash"
    return comparator.RowReading(number, {"status": "ok", "device_exit": "0"}, None, directory(), dict(zip((1, 2), by_batch)),
                                 {"registration": crash, "original_reading_5": crash})


class Denominator(unittest.TestCase):
    """Q3-分母 D = 触达且不在读法 5 那几类里的行数；各类逐行；批 2 定触达时只在批 1 恢复报告上不同的那一行掉出分母。"""

    def test_the_denominator_counts_only_touched_rows_outside_the_reading_5_classes(self) -> None:
        rows = [reading(1, "counted"), reading(2, "untouched"), reading(3, "first_batch_only"), reading(4, "self_crash"), reading(5, "not_found")]
        self.assertEqual(comparator.denominator(rows, "registration", 1), 2)
        self.assertEqual(comparator.denominator(rows, "registration", 2), 1)
        self.assertEqual(comparator.rows_by_category(rows, "registration", 1),
                         {"counted": [1, 3], "untouched": [2], "self_crash": [4], "not_found": [5]})
        self.assertEqual(comparator.rows_by_category(rows, "registration", 2),
                         {"counted": [1], "untouched": [2, 3], "self_crash": [4], "not_found": [5]})


def value(index: int, *, fields: dict[str, str] | None = None, objects: dict[str, str] | None = None, observation: str = "o") -> ReturnValueRow:
    return ReturnValueRow(index, "publish", "whole", fields if fields is not None else {"writes": "w"},
                          objects if objects is not None else {"allocator": "a"}, observation)


class Q2Set(unittest.TestCase):
    """Q2-集合：未触达（批 1）而交回值或内存对象与基线不同的行；触达的行不进；Q2-判 甲 / 乙 / 丙 / F4。"""

    def setUp(self) -> None:
        self.baseline = directory(fixed_script_through_e_then_unmount=stream(SECOND, return_values=[value(2)]))

    def row(self, number: int, return_value: ReturnValueRow, *, model_word: str = "agrees", touched: bool = False,
            touched_on_the_first_batch_only: bool = False) -> comparator.RowReading:
        run = directory(fixed_script_through_e_then_unmount=stream(SECOND, return_values=[return_value], calls=[call(2, model_word=model_word)]))
        second = matrix.TouchReport(touched, False, False, False)
        first = matrix.TouchReport(touched, False, touched_on_the_first_batch_only, False)
        return comparator.RowReading(number, {"status": "ok", "device_exit": "0"}, None, run, {1: first, 2: second},
                                     {"registration": False, "original_reading_5": False})

    def test_an_untouched_row_with_a_different_field_is_in_the_set_and_a_touched_one_is_not(self) -> None:
        rows = [self.row(1, value(2, fields={"writes": "x"})), self.row(2, value(2, fields={"writes": "x"}), touched=True), self.row(3, value(2)),
                self.row(4, value(2, fields={"writes": "x"}), touched_on_the_first_batch_only=True)]
        found = comparator.q2_set(rows, self.baseline, frozenset())
        self.assertEqual([(item.number, item.differences[0].differing_fields, item.comparison_red_calls) for item in found], [(1, ("writes",), 0)],
                         "第 4 行只在批 1 的恢复报告上不同：批 1 定触达（第五节 5.1），它是触达的、不进集合")

    def test_a_memory_object_difference_alone_is_in_the_set_unless_the_object_is_unstable(self) -> None:
        rows = [self.row(4, value(2, objects={"allocator": "b"}))]
        self.assertEqual([item.differences[0].differing_objects for item in comparator.q2_set(rows, self.baseline, frozenset())], [("allocator",)])
        self.assertEqual(comparator.q2_set(rows, self.baseline, frozenset({"allocator"})), [])

    def test_the_comparison_red_and_the_observation_split_are_counted_per_row(self) -> None:
        found = comparator.q2_set([self.row(5, value(2, fields={"writes": "x"}, observation="p"), model_word="disagrees:写数")], self.baseline, frozenset())
        self.assertEqual((found[0].comparison_red_calls, found[0].observation_differs_calls()), (1, 1))

    def test_the_verdict_is_jia_when_every_row_is_red_yi_when_some_bing_when_none_and_f4_when_the_set_is_empty(self) -> None:
        self.assertEqual([comparator.q2_verdict(2, 2), comparator.q2_verdict(2, 1), comparator.q2_verdict(2, 0), comparator.q2_verdict(0, 0)],
                         ["jia", "yi", "bing", "not_enough_f4"])


class InputsOfThisRun(unittest.TestCase):
    """导出目录在给的任一导出根之下就算这一趟的；合成产物名要带前缀与后缀。"""

    def test_a_directory_under_either_root_is_inside_and_one_under_neither_is_outside(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            first, second, third = (os.path.join(root, name) for name in ("a", "b", "c"))
            for path in (first, second, third):
                os.makedirs(os.path.join(path, "x"))
            outside = comparator.inputs_outside_this_run([first, second], "p-", "-r6.out", [os.path.join(first, "x"), os.path.join(second, "x"),
                                                                                             os.path.join(third, "x")], ["p-rows-r6.out", "q-rows-r6.out"])
            self.assertEqual(outside, [os.path.join(third, "x"), "q-rows-r6.out"])


def write_lines(path: str, rows: list[list[str]]) -> None:
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.writelines("\t".join(row) + "\n" for row in rows)


def write_stream(root: str, name: str, *, ops_text: str = "0\t0\twrite\n", fields: dict[int, str] | None = None, objects: dict[int, str] | None = None,
                 observations: dict[int, str] | None = None, words: dict[int, str] | None = None, panic_at: int | None = None,
                 first_batch_root: str = "1,3") -> None:
    """一条流的导出：三次调用（取号、暖机、发布）、每次的交回值与内存对象、批 1 与批 2 各一态、写流四个文件。键是调用号。"""
    directory_path = os.path.join(root, name)
    labels = ("acquire", "warm_up", "publish")
    outcomes = ["panic" if index == panic_at else "ok" for index in range(3)]
    model_words = [(words or {}).get(index, "agrees" if index == 2 else "not_modelled") for index in range(3)]
    if panic_at is not None:
        model_words[panic_at] = "implementation_crashed_not_compared"
    write_lines(os.path.join(directory_path, "stream-status.tsv"), [["ending", "completed"], ["model_stopped_at_call", "none"]])
    write_lines(os.path.join(directory_path, "call-outcomes.tsv"),
                [[str(index), labels[index], "0", "0", outcomes[index], model_words[index], "-", "-", "-", "-", "-"] for index in range(3)])
    write_lines(os.path.join(directory_path, "call-return-values.tsv"),
                [[name, str(index), labels[index], "whole", f"writes:{(fields or {}).get(index, 'w')}",
                  f"allocator:a,scratch:{(objects or {}).get(index, 's')}", (observations or {}).get(index, "o")]
                 for index in range(3) if index != panic_at])
    write_lines(os.path.join(directory_path, "states.tsv"), [["0", "0", "0", "1", "2", "d"], ["1", "1", "0", "2", "2", "d"]])
    verdict_tail = ["-", "green", "-", "after_every_step", "S_0", "-", "-"]
    write_lines(os.path.join(directory_path, "oracle-verdicts.tsv"),
                [[number, batch, number, "0", "2", "green", "-", "green", "-", "true", root_text, "file_read", "-", "0", "1,3", "file_read", "-", "0"]
                 + verdict_tail for number, batch, root_text in (("0", "1", first_batch_root), ("1", "2", "1,3"))])
    for file_name, text in (("ops.tsv", ops_text), ("calls.tsv", "0\tacquire\n"), ("ops-payload.bin", "p"), ("base-sectors.bin", "b")):
        with open(os.path.join(directory_path, file_name), "w", encoding="utf-8") as handle:
            handle.write(text)


def write_row(rows_root: str, number: int, *, status: str = "ok", rebuild_checked: bool = True, **by_stream: dict) -> None:
    row_directory = os.path.join(rows_root, f"row-{number}")
    fields = [["row_number", str(number)], ["row_name", f"行 {number}"], ["status", status], ["device_exit", "0" if status == "ok" else "-"],
              ["substituted_from", "-"], ["failed_attempts", "0" if status == "ok" else "1"], ["mutated_row_name", f"行 {number}"]]
    if rebuild_checked and status == "ok":
        fields.append(["packages_checked_rebuilt", "crates/singlefs-core"])
    write_lines(os.path.join(row_directory, "row-result.tsv"), fields)
    if status == "ok":
        for name in (FIRST, SECOND):
            write_stream(os.path.join(row_directory, "dump"), name, **by_stream.get(name, {}))


EXPECTED_VERDICT = ("E7RESULT name=verdict q2=yi q2_set_rows=2 q2_o1_red_rows=1 d_registration_batch_1=2 d_registration_batch_2=1 "
                    "d_original_reading_5_batch_1=2 d_original_reading_5_batch_2=1 row_125_counted_registration_batch_1=false rows_read=8 table_rows=281 "
                    "rows_not_located=4 unstable_memory_objects=1 stability_baseline_files_differing=1 rows_compiled_without_the_rebuild_check=1 "
                    "rows_with_an_errored_call_the_comparison_did_not_judge=0 rows_with_o1_not_applicable_from_another_source=0 "
                    "untouched_rows_with_a_red_column=1 rows_whose_directory_differs_from_the_driver_product=1")


class WholeTableOnDisk(unittest.TestCase):
    """手造整表 8 行落盘、跑 `main`：第 1 行交回值字段与观测都不同、O1-对拍绿（进集合、不红）；第 2 行字段不同、观测相同、O1-对拍红
    （红按 O1-对拍数、不按观测数）；第 3 行写流不同（触达）；
    第 4 行找不到；第 5 行只在批 1 恢复报告上不同；第 6 行流二在发布那一次 panic（之前可判 0 次）；第 7 行只有基线两次就不稳的对象不同；
    第 8 行与基线相同、没带重编核对的键。合成产物里第 3 行的退出码记成 253（与行目录对不上）。判决行整行钉死。"""

    def test_the_verdict_line_and_the_q2_and_q3_lines_are_exactly_the_hand_counted_ones(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            dumps, rows_root = os.path.join(root, "dumps"), os.path.join(root, "dumps", "rows")
            for name in (FIRST, SECOND):
                write_stream(os.path.join(dumps, "baseline"), name)
                write_stream(os.path.join(dumps, "stability"), name, objects={0: "t"} if name == FIRST else None)
            write_row(rows_root, 1, **{SECOND: {"fields": {2: "x"}, "observations": {2: "p"}}})
            write_row(rows_root, 2, **{SECOND: {"fields": {2: "x"}, "words": {2: "disagrees:写数"}}})
            write_row(rows_root, 3, **{FIRST: {"ops_text": "0\t0\twrite_fua\n"}})
            write_row(rows_root, 4, status="not_found")
            write_row(rows_root, 5, **{FIRST: {"first_batch_root": "1,4"}})
            write_row(rows_root, 6, **{SECOND: {"panic_at": 2}})
            write_row(rows_root, 7, **{FIRST: {"objects": {0: "t"}}})
            write_row(rows_root, 8, rebuild_checked=False)
            product = os.path.join(root, "p-s2-rows-driver-r6.out")
            with open(product, "w", encoding="utf-8") as handle:
                handle.write("name=row_time_limit seconds=1800\nsnapshot_digest=abc files=1 rows=8\n")
                handle.writelines(f"row row_number={number} status={'not_found' if number == 4 else 'ok'} "
                                  f"device_exit={'-' if number == 4 else ('253' if number == 3 else '0')}\n" for number in range(1, 9))
                handle.write("table lanes=1 rows=8\n")
            output = os.path.join(root, "comparator.out")
            exit_code = comparator.main(["--baseline", os.path.join(dumps, "baseline"), "--stability-baseline", os.path.join(dumps, "stability"),
                                         "--rows-root", rows_root, "--driver-product", product, "--dumps-root", dumps, "--product-prefix", "p-s2-",
                                         "--product-suffix=-r6.out", "--output", output])
            self.assertEqual(exit_code, 0)
            with open(output, encoding="utf-8") as handle:
                lines = handle.read().splitlines()
        self.assertIn(EXPECTED_VERDICT, lines)
        self.assertIn("E7RESULT name=q2 set_rows=2 o1_red_rows=1 rows_o1_red=2 rows_o1_not_red=1 verdict=yi", lines)
        self.assertIn("E7RESULT name=q2_observation set_rows=2 calls_with_a_difference=2 calls_whose_observation_differs=1 calls_whose_observation_is_the_same=1", lines)
        self.assertIn(f"E7RESULT name=q2_set_call row=1 stream={SECOND} call=2 label=publish fields=writes objects=- observation_differs=true o1_comparison=agrees", lines)
        self.assertIn(f"E7RESULT name=q2_set_call row=2 stream={SECOND} call=2 label=publish fields=writes objects=- observation_differs=false "
                      "o1_comparison=disagrees:写数", lines)
        self.assertIn("E7RESULT name=q3_denominator reading=registration batch=1 rows=8 counted_D=2 counted=2 untouched=4 self_crash=1 not_found=1", lines)
        self.assertIn("E7RESULT name=q3_denominator_class reading=registration batch=2 class=untouched rows=5 row_numbers=1,2,5,7,8", lines)
        self.assertIn(f"E7RESULT name=q3_self_crash_row row=6 where={SECOND}:call_2:judged_0 registration=self_crash original_reading_5=self_crash", lines)
        self.assertIn("E7RESULT name=unstable_memory_objects stability_baselines=1 names=scratch", lines)
        self.assertIn("E7RESULT name=driver_product_rows rows_in_the_product=8 rows_in_the_root=8 rows_whose_status_or_exit_differs=3", lines)
        self.assertEqual(lines[-1], f"E7RESULT name=done emitted={len(lines)}")
