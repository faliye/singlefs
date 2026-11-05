"""`removal_effect.py` 的单测：登记第五节 5.2 撤项核算的阳性对照（乙臂·原登记、乙臂·r1 第二段、甲臂、干扰）、F11，第八节的两条判别力自证，
第九节 M1–M5。产物都是手造的同形行，不碰真产物；判定词照原比对器的函数现算（原目录按路径载入）。"""

from __future__ import annotations

import sys
import unittest

import removal_effect

SYSTEM_CONFIGURATION = "crates/singlefs-core/src/system_configuration.rs"
OTHER_FILE = "crates/singlefs-core/src/admission.rs"
CATEGORY_FIELDS = ("untouched", "only_in_the_ignored_pass", "compile_failed", "anchor_not_unique", "not_found", "self_crash", "timeout",
                   "device_stop", "wrapper_ended")
FUNCTIONS = removal_effect.load_original_registration_functions(removal_effect.repository_root())


def numbers(values) -> str:
    return ",".join(map(str, sorted(values))) or "none"


def original_product(categories: dict[int, str], quiet: list[int], extra: tuple[str, ...] = ()) -> list[removal_effect.ProductLine]:
    """原登记比对器同形的一份：每行一条 `name=row` 与两批 `row_matrix`（各列都不红），读法 1 点名的汇总行照类别现数。"""
    counted = sorted(number for number, category in categories.items() if category == "counted")
    verdict = "yes_a_row_is_quiet_on_every_oracle" if quiet else "no_such_row"
    lines = [f"E7RESULT name=row row={number} row_name=行_{number} category={category} substituted_from=- line_matches_the_registration=true"
             for number, category in sorted(categories.items())]
    lines += [f"E7RESULT name=row_matrix row={number} batch={batch} O1=quiet O2=quiet" for number in categories for batch in (1, 2)]
    lines += [f"E7RESULT name=q1_denominator rows_run={len(categories)} counted_D={len(counted)} "
              + " ".join(f"{key}={sum(category == key for category in categories.values())}" for key in CATEGORY_FIELDS)
              + " enough_to_judge_none_side=true",
              f"E7RESULT name=q1_o1_and_o2_both_quiet counted_rows={len(counted)} rows={len(quiet)} row_numbers={numbers(quiet)} o3=run",
              f"E7RESULT name=q1_o1_and_o2_both_quiet_batch_2 rows={len(quiet)} rows_that_differ_from_batch_1=none",
              f"E7RESULT name=q1_column_red_rows counted_rows={len(counted)} O1=0 O2=0",
              "E7RESULT name=untouched_but_red rows=0 row_numbers=none",
              f"E7RESULT name=q1_o3_coverage counted_rows={len(counted)} rows_with_no_strict_judgement=0 strict_reading_unreachable=false",
              f"E7RESULT name=q1_denominator_by_table table=step_8_table rows_run=0 counted_D=0 counted_row_numbers=none",
              f"E7RESULT name=q1_denominator_by_table table=step_4_expansion rows_run={len(categories)} counted_D={len(counted)} "
              f"counted_row_numbers={numbers(counted)}"]
    for label in ("q1_alpha", "q1_beta", "q1_gamma"):
        lines.append(f"E7RESULT name={label} counted_rows={len(counted)} all_quiet_rows={len(quiet)} row_numbers={numbers(quiet)} "
                     f"batch_2_all_quiet_rows={len(quiet)} rows_that_differ_from_batch_1=none verdict={verdict}")
    for label in ("q3_alpha", "q3_beta"):
        lines.append(f"E7RESULT name={label} caught_rows=0 none_held_by_a_single_oracle rows_whose_red_columns_differ_between_batch_1_and_2=none")
    return removal_effect.parse_product("\n".join(lines + list(extra)) + "\n")


def forty_counted_rows() -> dict[int, str]:
    return {number: "counted" for number in range(197, 237)}


def files_with_the_system_configuration_rows() -> dict[int, str]:
    files = {number: OTHER_FILE for number in range(1, 282)}
    files.update({number: SYSTEM_CONFIGURATION for number in (85, 86, 87, 235, 236, 237)})
    return files


def original_assessment(categories: dict[int, str], quiet: list[int], extra: tuple[str, ...] = ()) -> removal_effect.Assessment:
    return removal_effect.original_registration_assessment(original_product(categories, quiet, extra), frozenset({236}),
                                                           files_with_the_system_configuration_rows(), FUNCTIONS)


def cell(assessment: removal_effect.Assessment, line: str, key: str) -> tuple[str, str]:
    found = [item for item in assessment.cells if item.line == line and item.key == key]
    assert len(found) == 1, (line, key, found)
    return found[0].before, found[0].after


class OriginalRegistrationProduct(unittest.TestCase):
    def test_removing_the_only_quiet_row_flips_q1_alpha_to_no_such_row_and_judges_yi(self) -> None:
        assessment = original_assessment(forty_counted_rows(), [236])
        self.assertEqual(cell(assessment, "q1_alpha", "verdict"), ("yes_a_row_is_quiet_on_every_oracle", "no_such_row"))
        self.assertEqual(cell(assessment, "q1_denominator", "counted_D"), ("40", "39"))
        self.assertEqual(removal_effect.judgement_of(assessment), "yi")
        self.assertEqual(assessment.self_check_problems, [])

    def test_removing_a_counted_row_that_is_not_quiet_changes_pinned_values_only_and_judges_jia(self) -> None:
        assessment = original_assessment(forty_counted_rows(), [200])
        self.assertEqual(cell(assessment, "q1_denominator", "counted_D"), ("40", "39"))
        self.assertEqual(cell(assessment, "q1_alpha", "counted_rows"), ("40", "39"))
        self.assertEqual(cell(assessment, "q1_alpha", "verdict"), ("yes_a_row_is_quiet_on_every_oracle", "yes_a_row_is_quiet_on_every_oracle"))
        self.assertEqual(removal_effect.judgement_of(assessment), "jia")

    def test_the_removed_row_leaves_its_own_category_and_not_the_one_of_the_row_before_it(self) -> None:
        categories = forty_counted_rows()
        categories.update({235: "self_crash", 236: "untouched"})
        assessment = original_assessment(categories, [200])
        self.assertEqual(cell(assessment, "q1_denominator", "untouched"), ("1", "0"))
        self.assertEqual(cell(assessment, "q1_denominator", "self_crash"), ("1", "1"))
        self.assertEqual(cell(assessment, "q1_denominator", "rows_run"), ("40", "39"))
        self.assertEqual(assessment.self_check_problems, [])

    def test_a_substitution_on_another_row_of_the_same_file_makes_the_product_not_enough_to_judge(self) -> None:
        assessment = original_assessment(forty_counted_rows(), [200], ("E7RESULT name=substitution row=235 original_name=行_235 substituted_by=补位",))
        self.assertEqual(assessment.interference, [(235, "substitution")])
        self.assertEqual(removal_effect.judgement_of(assessment), "not_enough_f12")

    def test_a_product_without_the_removed_row_is_not_enough_to_judge(self) -> None:
        categories = {number: "counted" for number in range(197, 236)}
        assessment = original_assessment(categories, [200])
        self.assertEqual((assessment.removed_present, assessment.removed_absent), ([], [236]))
        self.assertEqual(removal_effect.judgement_of(assessment), "not_enough_f11")


def segment_two_product(set_rows: dict[int, bool]) -> list[removal_effect.ProductLine]:
    """r1 第二段比对器同形的一份：四格类别都是 untouched 的几行加一行 counted（第 1 行），Q2 集合照给的行。"""
    rows = {1: "counted", **{number: "untouched" for number in set_rows}}
    readings = ("registration", "original_reading_5")
    lines = [f"E7RESULT name=row row={number} status=ok substituted_from=- "
             + " ".join(f"category_{reading}_batch_{batch}={category}" for reading in readings for batch in (1, 2))
             for number, category in sorted(rows.items())]
    lines += [f"E7RESULT name=q2_set_row row={number} o1_red={str(red).lower()}" for number, red in sorted(set_rows.items())]
    red = [number for number, is_red in set_rows.items() if is_red]
    quiet = [number for number, is_red in set_rows.items() if not is_red]
    verdict = removal_effect_q2_verdict(len(set_rows), len(red))
    lines.append(f"E7RESULT name=q2 set_rows={len(set_rows)} o1_red_rows={len(red)} rows_o1_red={numbers(red) if red else '-'} "
                 f"rows_o1_not_red={numbers(quiet) if quiet else '-'} verdict={verdict}")
    for reading in readings:
        for batch in (1, 2):
            lines.append(f"E7RESULT name=q3_denominator reading={reading} batch={batch} rows={len(rows)} counted_D=1 counted=1 "
                         f"untouched={len(set_rows)}")
            lines.append(f"E7RESULT name=q3_denominator_class reading={reading} batch={batch} class=untouched rows={len(set_rows)} "
                         f"row_numbers={numbers(set_rows)}")
    lines.append(f"E7RESULT name=verdict q2={verdict} q2_set_rows={len(set_rows)} q2_o1_red_rows={len(red)} "
                 + " ".join(f"d_{reading}_batch_{batch}=1" for reading in readings for batch in (1, 2))
                 + f" row_125_counted_registration_batch_1=false rows_read={len(rows)} table_rows=281 rows_not_located=-")
    return removal_effect.parse_product("\n".join(lines) + "\n")


def removal_effect_q2_verdict(set_rows: int, red_rows: int) -> str:
    from segment_two_comparator import q2_verdict
    return q2_verdict(set_rows, red_rows)


class SegmentTwoProduct(unittest.TestCase):
    def test_removing_the_quiet_set_row_turns_q2_from_yi_to_jia_and_judges_yi(self) -> None:
        assessment = removal_effect.segment_two_assessment(segment_two_product({100: True, 236: False}), frozenset({236}),
                                                           files_with_the_system_configuration_rows(), removal_effect_q2_verdict)
        self.assertEqual(cell(assessment, "verdict", "q2"), ("yi", "jia"))
        self.assertEqual(cell(assessment, "q2", "set_rows"), ("2", "1"))
        self.assertEqual(cell(assessment, "verdict", "table_rows"), ("281", "280"))
        self.assertEqual(removal_effect.judgement_of(assessment), "yi")
        self.assertEqual(assessment.self_check_problems, [])


class SegmentOneStructure(unittest.TestCase):
    def test_the_sixteen_rows_hold_neither_row_236_nor_a_system_configuration_row(self) -> None:
        rows = [0, 15, 23, 42, 49, 110, 125, 155, 161, 162, 174, 242, 252, 269, 270, 271]
        self.assertEqual(removal_effect.segment_one_structure(rows, frozenset({236}), files_with_the_system_configuration_rows()), ([], [], "jia"))
        self.assertEqual(removal_effect.segment_one_structure(rows + [237], frozenset({236}), files_with_the_system_configuration_rows()),
                         ([], [237], "recompute"))


class OriginalModulesLoadedByPath(unittest.TestCase):
    def test_loading_the_original_comparator_leaves_this_directory_s_result_lines_in_place(self) -> None:
        import result_lines
        removal_effect.load_original_registration_functions(removal_effect.repository_root())
        self.assertIs(sys.modules["result_lines"], result_lines)
        self.assertEqual(FUNCTIONS[0]([], list(range(29)), False, True), "not_enough_rows")


class ProductNotTheRegisteredOne(unittest.TestCase):
    """A22 与作废 Z11：读的产物 sha256 或行数与登记的不同，整份作废、退 1，一格都不算。"""

    def test_a_product_whose_line_count_differs_from_the_registration_is_void(self) -> None:
        import os
        import tempfile
        with tempfile.TemporaryDirectory() as root:
            product, list_check_product, output = (os.path.join(root, name) for name in ("product.out", "list-check.out", "removal.out"))
            for path, text in ((product, "E7RESULT name=done emitted=1\n"), (list_check_product, "E7RESULT name=done emitted=1\n")):
                with open(path, "w", encoding="utf-8") as handle:
                    handle.write(text)
            import hashlib
            with open(product, "rb") as handle:
                digest = hashlib.sha256(handle.read()).hexdigest()
            exit_code = removal_effect.main(["--kind", "segment-two", "--product", product, "--expected-sha256", digest, "--expected-lines", "2",
                                             "--list-check-product", list_check_product, "--output", output])
            with open(output, encoding="utf-8") as handle:
                text = handle.read()
        self.assertEqual(exit_code, 1)
        self.assertIn("matches_the_registration=false", text)
        self.assertIn("E7RESULT name=verdict void_z11=true\n", text)


if __name__ == "__main__":
    unittest.main()
