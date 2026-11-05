"""contrast_against_r1.py 的单测：跑前登记第五节 5.2 第二张表「Q1」那几格（手造 r1 形态与今天形态的片段，不碰真产物）、第八节判别力自证、
第九节 V1–V5 各自会红的那一格、作废 Z5 与分类表 sha256 的拒绝。跑法：在本目录 `python3 -m unittest test_contrast_against_r1`。"""

from __future__ import annotations

import os
import tempfile
import unittest

import contrast_against_r1 as contrast

CLASSES = contrast.read_cell_classes(open(contrast.CLASSES_PATH, encoding="utf-8").read())
S1, S2, S3 = contrast.STREAM_ONE, contrast.STREAM_TWO, contrast.STREAM_THREE


def p1_row(row: int, category: str = "counted", status: str = "ok", failed_attempts: str = "0") -> str:
    return (f"E7RESULT name=row row={row} status={status} device_exit=0 substituted_from=- failed_attempts={failed_attempts} mutated_row_name=m{row} "
            f"ran=true write_stream_differs=true call_outcomes_differ=false recovery_report_differs_batch_1=false recovery_report_differs_batch_2=false "
            f"ignored_pass_differs_batch_1=false ignored_pass_differs_batch_2=false call_panic=false judged_calls_before_the_first_panic=- "
            f"recovery_panic_states=0 category_registration_batch_1={category} category_registration_batch_2={category} "
            f"category_original_reading_5_batch_1={category} category_original_reading_5_batch_2={category}")


def q1_row(row: int, stream: str, batch: int = 1, o2_red: int = 0, crash_red: int = 0, o2_peak: int = 0) -> str:
    columns = " ".join(f"{column}_red={red} {column}_peak={peak} {column}_positive_segments=0 {column}_last=0 {column}_single_points=0"
                       for column, red, peak in (("O1-模型崩溃判官", crash_red, 0), ("O1-RefFS", 0, 0), ("O2", o2_red, o2_peak), ("O3甲", 0, 0),
                                                 ("O3乙", 0, 0)))
    return f"E7RESULT name=q1_row row={row} stream={stream} batch={batch} o1_comparison_red_calls=0 {columns}"


def p2_row(row: int, sha: str = "a" * 64) -> str:
    return (f"row row_number={row} row_name=m{row} lane=0 substituted_from=- line_matches_the_registration=true mutated_row_name=m{row} "
            f"failed_attempts=0 mutation_line_sha256={sha} copy_ns=1 compile_ns=2 compile_kind=cold status=ok device_exit=0 device_ns=3 o3_status=skipped_by_request")


def compare(pairs, segment=contrast.SEGMENT_STREAMS_ONE_TWO):
    return contrast.compare_segment(pairs, CLASSES, segment)


class Q1Controls(unittest.TestCase):
    def test_pinned_counts_changing_leave_no_decision_cell_changed(self):
        """5.2「Q1·没有一格变」：只改钉值格（计数 3 → 5、峰值变），判定格变了 0 格、钉值格差逐格列出（V1 会让它变成 ≥ 1 格）。"""
        comparison = compare([("P1", p1_row(15, failed_attempts="3"), p1_row(15, failed_attempts="5")),
                              ("P3", q1_row(15, S1, o2_red=3, o2_peak=1), q1_row(15, S1, o2_red=5, o2_peak=2))])
        self.assertEqual(comparison.decision_cells(), [])
        self.assertEqual(sorted(cell.name for cell in comparison.pinned_cells()), ["O2_peak", "failed_attempts"])
        self.assertEqual(contrast.self_check_problems(comparison), [])

    def test_a_category_turning_untouched_is_one_decision_cell(self):
        """5.2「Q1·有格变」：某行登记读法批 1 的类从 counted 变 untouched，判定格变了的格里有它（四种读法 × 批各一格）。"""
        comparison = compare([("P1", p1_row(15), p1_row(15, category="untouched"))])
        changed = {cell.name for cell in comparison.decision_cells()}
        self.assertIn("category_registration_batch_1", changed)
        self.assertEqual(len(changed), 4)

    def test_a_row_only_in_r1_is_a_presence_cell(self):
        """5.2「Q1·只在一边」与 V4：r1 有第 236 行、今天没有，那一行「在不在」一格变了。"""
        comparison = compare([("P1", "\n".join([p1_row(15), p1_row(236)]), p1_row(15))])
        presence = [cell for cell in comparison.decision_cells() if cell.name == contrast.PRESENCE_FIELD]
        self.assertEqual([(cell.key, cell.r1, cell.today) for cell in presence], [("row:236", "present", contrast.ABSENT_VALUE)])
        self.assertEqual(comparison.tallies[("P1", "row")].keys_only_r1, 1)

    def test_a_red_count_crossing_zero_is_one_decision_cell(self):
        """5.2「Q1·计数过 0」与 V2：O2_red 0 → 1，判定格变了恰 1 格（「是否大于 0」那一格）。"""
        comparison = compare([("P3", q1_row(15, S1, o2_red=0), q1_row(15, S1, o2_red=1))])
        self.assertEqual([(cell.name, cell.r1, cell.today) for cell in comparison.decision_cells()], [("O2_red", "0", "1")])

    def test_q1_row_cells_are_keyed_by_stream(self):
        """V3：同一行流一红、流二不红，r1 反过来——按（行, 流, 批）配键是 2 格，没有重键。"""
        r1 = "\n".join([q1_row(15, S1, crash_red=0), q1_row(15, S2, crash_red=2)])
        today = "\n".join([q1_row(15, S1, crash_red=2), q1_row(15, S2, crash_red=0)])
        comparison = compare([("P3", r1, today)])
        self.assertEqual(len(comparison.decision_cells()), 2)
        tally = comparison.tallies[("P3", "q1_row")]
        self.assertEqual((tally.duplicate_keys_r1, tally.duplicate_keys_today, tally.keys_both), (0, 0, 2))

    def test_a_verdict_word_is_a_decision_cell(self):
        """V5：手造 r1 `q2=yi`、今天 `q2=jia`，判定格 1 格。"""
        comparison = compare([("P1", "E7RESULT name=verdict q2=yi rows_read=281", "E7RESULT name=verdict q2=jia rows_read=278")])
        self.assertEqual([(cell.name, cell.cell_class) for cell in comparison.decision_cells()], [("q2", "decision")])
        self.assertEqual([cell.name for cell in comparison.pinned_cells()], ["rows_read"])

    def test_a_row_set_compares_as_a_set(self):
        """判定格③：行号集合按集合相等比，次序不同不算变。"""
        same = compare([("P1", "E7RESULT name=q2 rows_o1_red=155,161 verdict=yi", "E7RESULT name=q2 rows_o1_red=161,155 verdict=yi")])
        self.assertEqual(same.decision_cells(), [])
        differs = compare([("P1", "E7RESULT name=q2 rows_o1_red=155,161 verdict=yi", "E7RESULT name=q2 rows_o1_red=155 verdict=yi")])
        self.assertEqual([cell.name for cell in differs.decision_cells()], ["rows_o1_red"])

    def test_the_driver_product_compares_status_and_reports_the_line_digest_as_pinned(self):
        """P2/T2：status 是判定格，mutation_line_sha256 是钉值格，耗时与 O3 那几样不比、不算今天独有以外的格。"""
        comparison = compare([("P2", p2_row(15), p2_row(15, sha="b" * 64).replace("o3_status=skipped_by_request", "o3_status=ran"))])
        self.assertEqual(comparison.decision_cells(), [])
        self.assertEqual([cell.name for cell in comparison.pinned_cells()], ["mutation_line_sha256"])
        self.assertEqual(comparison.tallies[("P2", "row")].today_unique, 1)


class Segments(unittest.TestCase):
    def test_stream_three_rows_are_compared_only_in_the_second_segment(self):
        r1 = "\n".join([q1_row(15, S1), q1_row(15, S3, o2_red=0)])
        today = "\n".join([q1_row(15, S1), q1_row(15, S3, o2_red=4)])
        self.assertEqual(compare([("P3", r1, today)]).decision_cells(), [])
        second = compare([("P3", r1, today)], contrast.SEGMENT_STREAM_THREE)
        self.assertEqual([cell.key for cell in second.decision_cells()], [f"row:15,stream:{S3},batch:1"])
        self.assertEqual(contrast.self_check_problems(second), [])

    def test_the_p3_verdict_splits_q1_and_q3_between_the_segments(self):
        r1, today = "E7RESULT name=verdict q1=yi q3=jia q4_baseline=jia", "E7RESULT name=verdict q1=jia q3=yi q4_baseline=yi"
        self.assertEqual([cell.name for cell in compare([("P3", r1, today)]).decision_cells()], ["q3"])
        self.assertEqual([cell.name for cell in compare([("P3", r1, today)], contrast.SEGMENT_STREAM_THREE).decision_cells()], ["q1"])

    def test_a_decision_field_missing_today_is_left_unchecked(self):
        """F5：r1 有、今天这一种一条都没有的判定字段记「未比」，不算变了的格。"""
        comparison = compare([("P1", "E7RESULT name=q2 rows_o1_red=155 verdict=yi", "E7RESULT name=q2 verdict=yi")])
        self.assertEqual(comparison.decision_cells(), [])
        self.assertEqual(comparison.tallies[("P1", "q2")].unchecked_fields, ("rows_o1_red",))


class Refusals(unittest.TestCase):
    def test_the_classes_table_is_the_registered_one(self):
        self.assertEqual(contrast.sha256_of_file(contrast.CLASSES_PATH), contrast.CLASSES_SHA256)
        with self.assertRaises(ValueError):
            contrast.load_cell_classes(expected_sha256="0" * 64)

    def test_an_r1_product_other_than_the_registered_one_is_refused(self):
        with tempfile.TemporaryDirectory() as directory:
            relative = contrast.R1_PRODUCTS["P1"][0]
            os.makedirs(os.path.join(directory, os.path.dirname(relative)))
            with open(os.path.join(directory, relative), "w", encoding="utf-8") as handle:
                handle.write("E7RESULT name=done emitted=1\n")
            self.assertEqual(contrast.r1_product_problems(directory, {"P1": os.path.join(directory, relative)}), ["P1:sha256_or_lines"])
            self.assertEqual(contrast.r1_product_problems(directory, {"P1": os.path.join(directory, "elsewhere.out")}),
                             [f"P1:path={os.path.join(directory, 'elsewhere.out')}"])


if __name__ == "__main__":
    unittest.main()
