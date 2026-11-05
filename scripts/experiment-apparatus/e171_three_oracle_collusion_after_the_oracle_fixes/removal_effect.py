"""E171（三oracle合谋测试） 第 2 次重跑（登记 research/prompts/e171-r2-prereg.md）的撤项核算：从已落盘的读数里减掉被撤的行，判定词用原比对器的函数现算，
逐格写撤之前、撤之后、变没变（第六节「撤项·原登记」「撤项·r1 第二段」「撤项·r1 第一段」「撤项·干扰」，第十节 F11、F12、F14，第七节 A22、A25）。

被撤的行集 R 取自名单核对的产物（`list_check.py` 的 `name=classification` 行），两种各算一遍：只甲类（问题单 Q1 判它）、甲丙两类全部（描述）。
判定词（读法 2）：原登记比对器的 `q1_alpha` / `q1_beta` / `q1_gamma` 的 `verdict=`，r1 第二段的 `q2=`、`verdict=` 与 `row_125_counted_registration_batch_1=`；
其余取值是数、行号列表或真假的格都是钉值。判定词不另写判法：原登记那一份按路径载入原目录的 `collusion_report.reading_verdict`、
`collusion_matrix.coverage_enough_to_judge` 与 `strict_reading_is_unreachable`，r1 第二段那一份用本目录 `segment_two_comparator.q2_verdict`。
一份产物对某个 R 判：读不到被撤行（F11）记 `not_enough_f11`；与被撤行同文件的别的行在那一趟里有过补位或定位失败（F12）记 `not_enough_f12`；
否则判定词有一格变了记 `yi`，全同记 `jia`。r1 第一段只做结构核对（它只跑 16 行）：那 16 行里有被撤的行或与被撤行同文件的行记 `recompute`，没有记 `jia`。

只读：一份比对器产物、名单核对产物、改后的两张名单；写一份新产物（已存在就拒绝）。读之前核产物的 sha256 与行数（A22），对不上整份作废（Z11）、退 1。

跑法（在本目录）：
  python3 removal_effect.py --kind original-registration|segment-two --product <产物> --expected-sha256 <值> --expected-lines <行数>
      --list-check-product <名单核对产物> --output <新文件>
r1 第一段的结构核对每一趟都做（结果行 `name=segment_one_structure`），没有只做一半的用法。
退出码：0 核完；1 产物对不上（Z11）或自检（A25）不过；2 用法错。
"""

from __future__ import annotations

import argparse
import hashlib
import importlib
import os
import sys
from dataclasses import dataclass

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import list_check  # noqa: E402
from bad_implementation_table import read_selected_rows  # noqa: E402
from result_lines import ResultLines, field  # noqa: E402

KIND_ORIGINAL_REGISTRATION = "original-registration"
KIND_SEGMENT_TWO = "segment-two"
WITHDRAWN_OLD_FORMAT = "old_format_only"
WITHDRAWN_OLD_FORMAT_AND_GONE = "old_format_and_gone"
VERDICT_WORD = "verdict_word"
PINNED_VALUE = "pinned_value"
UNKNOWN = "unknown"
JUDGEMENT_SAME_WORDS = "jia"
JUDGEMENT_A_WORD_CHANGED = "yi"
JUDGEMENT_MISSING_ROWS = "not_enough_f11"
JUDGEMENT_INTERFERENCE = "not_enough_f12"
COUNTED = "counted"
# 原登记 5.9 的表编到 117，扩表从 118 起（原目录 expansion_table.FIRST_EXPANSION_ROW_NUMBER）
FIRST_EXPANSION_ROW_NUMBER = 118
STEP_8_TABLE, STEP_4_EXPANSION = "step_8_table", "step_4_expansion"
# 原比对器那几份模块与本目录同名，按路径载入时先让开、载完还原
ORIGINAL_MODULES = ("collusion_report", "collusion_matrix", "result_lines", "run_directory")
FAILED_LOCATION_CATEGORIES = ("compile_failed", "not_found", "anchor_not_unique", "ambiguous_name")


@dataclass(frozen=True)
class ProductLine:
    name: str
    fields: dict[str, str]


@dataclass(frozen=True)
class Cell:
    """一格：哪一行（行名加区分它的字段）、哪个字段、撤之前、撤之后、判定词还是钉值。"""

    line: str
    key: str
    before: str
    after: str
    kind: str

    def changed(self) -> bool:
        return self.before != self.after


@dataclass(frozen=True)
class Assessment:
    cells: list[Cell]
    removed_present: list[int]
    removed_absent: list[int]
    interference: list[tuple[int, str]]
    self_check_problems: list[str]


def repository_root() -> str:
    """变异驱动在草稿目录的副本里跑单测时由 `E171_REPOSITORY_ROOT` 给真的仓根；平常从本文件的位置推。"""
    return os.environ.get("E171_REPOSITORY_ROOT") or os.path.abspath(os.path.join(HERE, "../../.."))


def parse_product(text: str) -> list[ProductLine]:
    found = []
    for line in text.split("\n"):
        if not line.startswith("E7RESULT "):
            continue
        tokens = [token.split("=", 1) for token in line.split(" ")[1:] if "=" in token]
        fields = dict(tokens)
        found.append(ProductLine(fields.get("name", "-"), fields))
    return found


def numbers_of(value: str) -> list[int]:
    if value in ("", "-", "none"):
        return []
    return [int(number) for number in value.split(",")]


def numbers_value(numbers, empty: str = "none") -> str:
    return ",".join(map(str, sorted(numbers))) or empty


def withdrawn_sets(list_check_text: str) -> tuple[dict[str, frozenset[int]], dict[int, str]]:
    """名单核对产物里的分类：只甲类、甲丙两类全部；另交回被撤各行的文件。"""
    by_class: dict[str, set[int]] = {}
    files = {}
    for line in parse_product(list_check_text):
        if line.name != "classification":
            continue
        number = int(line.fields["row"])
        by_class.setdefault(line.fields["class"], set()).add(number)
        files[number] = line.fields["file"]
    old_format = frozenset(by_class.get(list_check.CLASS_OLD_FORMAT, set()))
    gone = frozenset(by_class.get(list_check.CLASS_GONE, set()))
    return {WITHDRAWN_OLD_FORMAT: old_format, WITHDRAWN_OLD_FORMAT_AND_GONE: old_format | gone}, files


def files_of_the_rows(directory: str = HERE) -> dict[int, str]:
    """改后两张名单每一行的文件（被撤的行不在里面，由名单核对产物补）。"""
    return {row.number: row.file for name in list_check.LISTS_AFTER for row in read_selected_rows(os.path.join(directory, name))}


def load_original_registration_functions(repository: str):
    """原目录的 `reading_verdict`、`coverage_enough_to_judge`、`strict_reading_is_unreachable`（按路径载入，不另写）。"""
    directory = os.path.join(repository, list_check.ORIGINAL_DIRECTORY)
    saved = {name: sys.modules.pop(name) for name in ORIGINAL_MODULES if name in sys.modules}
    sys.path.insert(0, directory)
    try:
        report = importlib.import_module("collusion_report")
        matrix = sys.modules["collusion_matrix"]
        return report.reading_verdict, matrix.coverage_enough_to_judge, matrix.strict_reading_is_unreachable
    finally:
        sys.path.remove(directory)
        for name in ORIGINAL_MODULES:
            sys.modules.pop(name, None)
        sys.modules.update(saved)


def single_lines(product: list[ProductLine]) -> dict[str, dict[str, str]]:
    return {line.name: line.fields for line in product}


def count_cell(cells: list[Cell], line: str, fields: dict[str, str], key: str, removed_count: int) -> int:
    after = int(fields[key]) - removed_count
    cells.append(Cell(line, key, fields[key], str(after), PINNED_VALUE))
    return after


def list_cell(cells: list[Cell], line: str, fields: dict[str, str], key: str, removed: frozenset[int], empty: str = "none") -> list[int]:
    after = [number for number in numbers_of(fields[key]) if number not in removed]
    cells.append(Cell(line, key, fields[key], numbers_value(after, empty), PINNED_VALUE))
    return after


def interference_rows(removed: frozenset[int], files: dict[int, str], failed_rows: dict[int, str]) -> list[tuple[int, str]]:
    """读法 6：与被撤行同一文件、在那一趟里有过补位或定位失败的别的行（F12）。"""
    removed_files = {files[number] for number in removed if number in files}
    return sorted((number, record) for number, record in failed_rows.items()
                  if number not in removed and files.get(number) in removed_files)


def original_registration_assessment(product: list[ProductLine], removed: frozenset[int], files: dict[int, str], functions) -> Assessment:
    """原登记比对器那一份（F4 扩表、加补位）：读法 1 点名的各格撤之前与撤之后。"""
    reading_verdict, coverage_enough_to_judge, strict_reading_is_unreachable = functions
    rows = {int(line.fields["row"]): line.fields for line in product if line.name == "row"}
    matrix_rows = {(int(line.fields["row"]), line.fields["batch"]): line.fields for line in product if line.name == "row_matrix"}
    q3_rows = {(line.fields["reading"], int(line.fields["row"])) for line in product if line.name == "q3_row"}
    failed_rows = {number: fields["category"] for number, fields in rows.items() if fields["category"] in FAILED_LOCATION_CATEGORIES}
    failed_rows.update({number: "substituted" for number, fields in rows.items() if fields.get("substituted_from", "-") != "-"})
    failed_rows.update({int(line.fields["row"]): "substitution" for line in product if line.name == "substitution"})
    present = sorted(number for number in removed if number in rows)
    absent = sorted(number for number in removed if number not in rows)
    counted_removed = {number for number in present if rows[number]["category"] == COUNTED}
    single = single_lines(product)
    cells: list[Cell] = []
    problems: list[str] = []
    denominator = single["q1_denominator"]
    rows_after = count_cell(cells, "q1_denominator", denominator, "rows_run", len(present))
    counted_after = count_cell(cells, "q1_denominator", denominator, "counted_D", len(counted_removed))
    categories = [key for key in denominator if key not in ("name", "rows_run", "counted_D", "enough_to_judge_none_side")]
    category_after = {key: count_cell(cells, "q1_denominator", denominator, key, sum(rows[number]["category"] == key for number in present))
                      for key in categories}
    cells.append(Cell("q1_denominator", "enough_to_judge_none_side", denominator["enough_to_judge_none_side"],
                      str(coverage_enough_to_judge(counted_after)).lower(), PINNED_VALUE))
    if int(denominator["counted_D"]) + sum(int(denominator[key]) for key in categories) != int(denominator["rows_run"]):
        problems.append("q1_denominator_before_categories_do_not_add_up")
    if counted_after + sum(category_after.values()) != rows_after:
        problems.append("q1_denominator_after_categories_do_not_add_up")
    both_quiet = single["q1_o1_and_o2_both_quiet"]
    count_cell(cells, "q1_o1_and_o2_both_quiet", both_quiet, "counted_rows", len(counted_removed))
    quiet_after = list_cell(cells, "q1_o1_and_o2_both_quiet", both_quiet, "row_numbers", removed)
    cells.append(Cell("q1_o1_and_o2_both_quiet", "rows", both_quiet["rows"], str(len(quiet_after)), PINNED_VALUE))
    second = single["q1_o1_and_o2_both_quiet_batch_2"]
    second_set = set(numbers_of(both_quiet["row_numbers"])) ^ set(numbers_of(second["rows_that_differ_from_batch_1"]))
    cells.append(Cell("q1_o1_and_o2_both_quiet_batch_2", "rows", second["rows"], str(len(second_set - removed)), PINNED_VALUE))
    list_cell(cells, "q1_o1_and_o2_both_quiet_batch_2", second, "rows_that_differ_from_batch_1", removed)
    columns = single["q1_column_red_rows"]
    count_cell(cells, "q1_column_red_rows", columns, "counted_rows", len(counted_removed))
    for key in [key for key in columns if key not in ("name", "counted_rows")]:
        count_cell(cells, "q1_column_red_rows", columns, key,
                   sum(matrix_rows.get((number, "1"), {}).get(key) == "red" for number in counted_removed))
    flagged = single["untouched_but_red"]
    flagged_after = list_cell(cells, "untouched_but_red", flagged, "row_numbers", removed)
    cells.append(Cell("untouched_but_red", "rows", flagged["rows"], str(len(flagged_after)), PINNED_VALUE))
    coverage = single["q1_o3_coverage"]
    zero_before = int(coverage["rows_with_no_strict_judgement"])
    count_cell(cells, "q1_o3_coverage", coverage, "counted_rows", len(counted_removed))
    zero_after = None if counted_removed and zero_before > 0 else zero_before
    if zero_after is None:
        problems.append("q1_o3_coverage_zero_coverage_of_a_removed_counted_row_is_not_in_the_product")
    unreachable_after = None if zero_after is None else strict_reading_is_unreachable(zero_after, counted_after)
    cells.append(Cell("q1_o3_coverage", "rows_with_no_strict_judgement", coverage["rows_with_no_strict_judgement"],
                      UNKNOWN if zero_after is None else str(zero_after), PINNED_VALUE))
    cells.append(Cell("q1_o3_coverage", "strict_reading_unreachable", coverage["strict_reading_unreachable"],
                      UNKNOWN if unreachable_after is None else str(unreachable_after).lower(), PINNED_VALUE))
    unreachable_before = coverage["strict_reading_unreachable"] == "true"
    for label in ("q1_alpha", "q1_beta", "q1_gamma"):
        reading = single[label]
        quiet_before = numbers_of(reading["row_numbers"])
        count_cell(cells, label, reading, "counted_rows", len(counted_removed))
        quiet = list_cell(cells, label, reading, "row_numbers", removed)
        cells.append(Cell(label, "all_quiet_rows", reading["all_quiet_rows"], str(len(quiet)), PINNED_VALUE))
        batch_two = set(quiet_before) ^ set(numbers_of(reading["rows_that_differ_from_batch_1"]))
        cells.append(Cell(label, "batch_2_all_quiet_rows", reading["batch_2_all_quiet_rows"], str(len(batch_two - removed)), PINNED_VALUE))
        list_cell(cells, label, reading, "rows_that_differ_from_batch_1", removed)
        recomputed_before = reading_verdict(quiet_before, list(range(int(reading["counted_rows"]))), unreachable_before, label == "q1_alpha")
        if recomputed_before != reading["verdict"]:
            problems.append(f"{label}_verdict_recomputed_before_differs_from_the_product")
        after = UNKNOWN if unreachable_after is None else reading_verdict(quiet, list(range(counted_after)), unreachable_after, label == "q1_alpha")
        cells.append(Cell(label, "verdict", reading["verdict"], after, VERDICT_WORD))
    for label in ("q3_alpha", "q3_beta"):
        reading = single[label]
        count_cell(cells, label, reading, "caught_rows", sum((label, number) in q3_rows for number in counted_removed))
        for key in [key for key in reading if key.startswith("only_") or key == "rows_whose_red_columns_differ_between_batch_1_and_2"]:
            list_cell(cells, label, reading, key, removed)
    for line in product:
        if line.name != "q1_denominator_by_table":
            continue
        table = line.fields["table"]
        in_table = [number for number in present if (number < FIRST_EXPANSION_ROW_NUMBER) == (table == STEP_8_TABLE)]
        name = f"q1_denominator_by_table.{table}"
        count_cell(cells, name, line.fields, "rows_run", len(in_table))
        count_cell(cells, name, line.fields, "counted_D", sum(number in counted_removed for number in in_table))
        list_cell(cells, name, line.fields, "counted_row_numbers", removed)
    return Assessment(cells, present, absent, interference_rows(removed, files, failed_rows), problems)


def segment_two_assessment(product: list[ProductLine], removed: frozenset[int], files: dict[int, str], q2_verdict) -> Assessment:
    """r1 第二段比对器那一份：判决行、Q2 行、Q3 分母与各类逐行（两种读法 × 两批）撤之前与撤之后。"""
    rows = {int(line.fields["row"]): line.fields for line in product if line.name == "row"}
    set_rows = {int(line.fields["row"]): line.fields["o1_red"] == "true" for line in product if line.name == "q2_set_row"}
    failed_rows = {number: fields["status"] for number, fields in rows.items() if fields.get("status", "ok") != "ok"}
    failed_rows.update({number: "substituted" for number, fields in rows.items() if fields.get("substituted_from", "-") != "-"})
    failed_rows.update({int(line.fields["row"]): line.name for line in product if line.name in ("substitution", "not_located_row")})
    present = sorted(number for number in removed if number in rows)
    absent = sorted(number for number in removed if number not in rows)
    single = single_lines(product)
    cells: list[Cell] = []
    problems: list[str] = []

    def category_of(number: int, reading: str, batch: str) -> str:
        return rows[number].get(f"category_{reading}_batch_{batch}", "-")

    set_after = [number for number in set_rows if number not in removed]
    red_after = [number for number in set_after if set_rows[number]]
    question = single["q2"]
    if q2_verdict(len(set_rows), sum(set_rows.values())) != question["verdict"]:
        problems.append("q2_verdict_recomputed_before_differs_from_the_product")
    count_cell(cells, "q2", question, "set_rows", sum(number in removed for number in set_rows))
    count_cell(cells, "q2", question, "o1_red_rows", sum(number in removed and red for number, red in set_rows.items()))
    list_cell(cells, "q2", question, "rows_o1_red", removed, "-")
    list_cell(cells, "q2", question, "rows_o1_not_red", removed, "-")
    cells.append(Cell("q2", "verdict", question["verdict"], q2_verdict(len(set_after), len(red_after)), VERDICT_WORD))
    for line in product:
        if line.name == "q3_denominator":
            reading, batch = line.fields["reading"], line.fields["batch"]
            name = f"q3_denominator.{reading}.batch_{batch}"
            rows_after = count_cell(cells, name, line.fields, "rows", len(present))
            classes = [key for key in line.fields if key not in ("name", "reading", "batch", "rows", "counted_D")]
            count_cell(cells, name, line.fields, "counted_D", sum(category_of(number, reading, batch) == COUNTED for number in present))
            after = {key: count_cell(cells, name, line.fields, key, sum(category_of(number, reading, batch) == key for number in present))
                     for key in classes}
            if sum(int(line.fields[key]) for key in classes) != int(line.fields["rows"]):
                problems.append(f"{name}_before_classes_do_not_add_up")
            if sum(after.values()) != rows_after:
                problems.append(f"{name}_after_classes_do_not_add_up")
        elif line.name == "q3_denominator_class":
            name = f"q3_denominator_class.{line.fields['reading']}.batch_{line.fields['batch']}.{line.fields['class']}"
            numbers = list_cell(cells, name, line.fields, "row_numbers", removed, "-")
            cells.append(Cell(name, "rows", line.fields["rows"], str(len(numbers)), PINNED_VALUE))
    verdict = single["verdict"]
    cells.append(Cell("verdict", "q2", verdict["q2"], q2_verdict(len(set_after), len(red_after)), VERDICT_WORD))
    count_cell(cells, "verdict", verdict, "q2_set_rows", sum(number in removed for number in set_rows))
    count_cell(cells, "verdict", verdict, "q2_o1_red_rows", sum(number in removed and red for number, red in set_rows.items()))
    for key in [key for key in verdict if key.startswith("d_")]:
        reading, batch = key[len("d_"):].rsplit("_batch_", 1)
        count_cell(cells, "verdict", verdict, key, sum(category_of(number, reading, batch) == COUNTED for number in present))
    row_125_after = "false" if 125 in removed else verdict["row_125_counted_registration_batch_1"]
    cells.append(Cell("verdict", "row_125_counted_registration_batch_1", verdict["row_125_counted_registration_batch_1"], row_125_after, VERDICT_WORD))
    count_cell(cells, "verdict", verdict, "rows_read", len(present))
    count_cell(cells, "verdict", verdict, "table_rows", len(removed))
    list_cell(cells, "verdict", verdict, "rows_not_located", removed, "-")
    return Assessment(cells, present, absent, interference_rows(removed, files, failed_rows), problems)


def segment_one_rows() -> list[int]:
    """r1 第一段跑过的行：原跑法第 11、12 步的 --rows 之并（含空变异 row-0）与比对器点名的各行（登记第七节 A23）。"""
    import rows_orchestrator
    import segment_one_comparator
    numbers = {number for rows in rows_orchestrator.STEP_ROWS.values() for number in rows}
    numbers |= set(segment_one_comparator.Q1_ROWS) | set(segment_one_comparator.Q2_NAMED_ROWS) | set(segment_one_comparator.Q4_ROWS)
    numbers.add(segment_one_comparator.Q3_ROW)
    return sorted(numbers)


def segment_one_structure(segment_rows: list[int], removed: frozenset[int], files: dict[int, str]) -> tuple[list[int], list[int], str]:
    """结构核对：第一段的行里有没有被撤的行；与被撤行同文件的有哪几行（空变异 row-0 取表第一行的文件）。交回 (在 R 里的, 同文件的, 判定)。"""
    removed_files = {files[number] for number in removed if number in files}
    in_removed = [number for number in segment_rows if number in removed]
    same_file = [number for number in segment_rows if files.get(number if number else 1) in removed_files]
    return in_removed, same_file, ("recompute" if in_removed or same_file else JUDGEMENT_SAME_WORDS)


def judgement_of(assessment: Assessment) -> str:
    if assessment.removed_absent:
        return JUDGEMENT_MISSING_ROWS
    if assessment.interference:
        return JUDGEMENT_INTERFERENCE
    if any(cell.kind == VERDICT_WORD and cell.after == UNKNOWN for cell in assessment.cells):
        return JUDGEMENT_MISSING_ROWS
    return JUDGEMENT_A_WORD_CHANGED if any(cell.kind == VERDICT_WORD and cell.changed() for cell in assessment.cells) else JUDGEMENT_SAME_WORDS


def assessment_lines(lines: ResultLines, label: str, assessment: Assessment) -> str:
    """一个 R 的全部格：被撤行在不在、干扰行、逐格撤前撤后、自检（A25）、这一份对这个 R 的判定。交回判定。"""
    lines.result(f"name=removed_rows r={label} present={numbers_value(assessment.removed_present)} absent={numbers_value(assessment.removed_absent)}")
    for number, record in assessment.interference:
        lines.result(f"name=interference_row r={label} row={number} record={field(record)}")
    for cell in assessment.cells:
        lines.result(f"name=cell r={label} line={field(cell.line)} key={field(cell.key)} kind={cell.kind} before={field(cell.before)} "
                     f"after={field(cell.after)} changed={str(cell.changed()).lower()}")
    not_subset = [f"{cell.line}.{cell.key}" for cell in assessment.cells
                  if cell.kind == PINNED_VALUE and "," in cell.after + cell.before
                  and not set(numbers_of(cell.after)) <= set(numbers_of(cell.before))]
    problems = assessment.self_check_problems + [f"{name}_after_is_not_a_subset" for name in not_subset]
    lines.result(f"name=self_check r={label} problems={field(','.join(problems)) or 'none'}")
    judgement = judgement_of(assessment)
    words = [cell for cell in assessment.cells if cell.kind == VERDICT_WORD]
    pinned = [cell for cell in assessment.cells if cell.kind == PINNED_VALUE]
    lines.result(f"name=removal_verdict r={label} verdict_words={len(words)} verdict_words_changed={sum(cell.changed() for cell in words)} "
                 f"pinned_values={len(pinned)} pinned_values_changed={sum(cell.changed() for cell in pinned)} "
                 f"interference_rows={numbers_value(number for number, _ in assessment.interference)} judgement={judgement}")
    return judgement if not problems else "self_check_failed"


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E171 第 2 次重跑的撤项核算（登记第六节撤项各行）")
    parser.add_argument("--kind", required=True, choices=(KIND_ORIGINAL_REGISTRATION, KIND_SEGMENT_TWO))
    parser.add_argument("--product", required=True)
    parser.add_argument("--expected-sha256", required=True, help="登记第十一节比对对象表里这一份的 sha256（A22）")
    parser.add_argument("--expected-lines", required=True, type=int)
    parser.add_argument("--list-check-product", required=True)
    parser.add_argument("--output", required=True)
    arguments = parser.parse_args(argument_list)
    with open(arguments.product, "rb") as handle:
        product_bytes = handle.read()
    with open(arguments.list_check_product, encoding="utf-8") as handle:
        list_check_text = handle.read()
    product_sha256 = hashlib.sha256(product_bytes).hexdigest()
    product_lines = product_bytes.count(b"\n")
    matches = product_sha256 == arguments.expected_sha256 and product_lines == arguments.expected_lines
    lines = ResultLines(arguments.output)
    lines.result(f"name=config kind={arguments.kind} product={os.path.basename(arguments.product)} product_sha256={product_sha256} "
                 f"product_lines={product_lines} expected_sha256={arguments.expected_sha256} expected_lines={arguments.expected_lines} "
                 f"matches_the_registration={str(matches).lower()} list_check_product={os.path.basename(arguments.list_check_product)} "
                 f"list_check_product_sha256={hashlib.sha256(list_check_text.encode('utf-8')).hexdigest()}")
    if not matches:
        lines.result("name=verdict void_z11=true")
        lines.finish()
        return 1
    product = parse_product(product_bytes.decode("utf-8"))
    withdrawn, withdrawn_files = withdrawn_sets(list_check_text)
    files = {**files_of_the_rows(), **withdrawn_files}
    if arguments.kind == KIND_ORIGINAL_REGISTRATION:
        functions = load_original_registration_functions(repository_root())

        def assess(removed: frozenset[int]) -> Assessment:
            return original_registration_assessment(product, removed, files, functions)
    else:
        from segment_two_comparator import q2_verdict

        def assess(removed: frozenset[int]) -> Assessment:
            return segment_two_assessment(product, removed, files, q2_verdict)
    judgements = {}
    for label in (WITHDRAWN_OLD_FORMAT, WITHDRAWN_OLD_FORMAT_AND_GONE):
        lines.result(f"name=withdrawn r={label} rows={numbers_value(withdrawn[label])}")
        judgements[label] = assessment_lines(lines, label, assess(withdrawn[label]))
    segment_one = "-"
    segment_rows = segment_one_rows()
    for label in (WITHDRAWN_OLD_FORMAT, WITHDRAWN_OLD_FORMAT_AND_GONE):
        in_removed, same_file, judgement = segment_one_structure(segment_rows, withdrawn[label], files)
        judgement = judgement if label == WITHDRAWN_OLD_FORMAT else "described_only"
        lines.result(f"name=segment_one_structure r={label} segment_one_rows={numbers_value(segment_rows)} rows_in_r={numbers_value(in_removed)} "
                     f"same_file_rows={numbers_value(same_file)} judgement={judgement}")
        if label == WITHDRAWN_OLD_FORMAT:
            segment_one = judgement
    failed = any(judgement == "self_check_failed" for judgement in judgements.values())
    lines.result(f"name=verdict void_z11=false q1_judgement={judgements[WITHDRAWN_OLD_FORMAT]} "
                 f"described_judgement_old_format_and_gone={judgements[WITHDRAWN_OLD_FORMAT_AND_GONE]} segment_one_structure={segment_one} "
                 f"self_check_passed={str(not failed).lower()}")
    lines.finish()
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
