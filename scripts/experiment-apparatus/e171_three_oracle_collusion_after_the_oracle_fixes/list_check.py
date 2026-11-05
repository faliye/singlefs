"""E171（三oracle合谋测试） 第 2 次重跑（登记 research/prompts/e171-r2-prereg.md）的名单核对：两份坏实现名单按变异名逐项取行（读法 5）、
取不到的照读法 3 分甲乙丙、核改后的名单（第六节「名单改后」、作废 Z12）、代码里现算的行集（「钉值·名单」）与今天的树上每一行取不取得到（「现状·可取行」），
以及第七节 K1–K3。

取法与逐行驱动同一份：`bad_implementation_table.read_mutation_rows`（跳过 `#` 起头的行、只收恰 6 列的行），名单第 5 列逐字比变异表第 1 列，数命中次数；
命中 0 次是「取不到」，2 次以上是「名字不唯一」（另列）。改之前的名单从开工那一刻的提交（`--reference-commit`，登记修订记的 HEAD）里取，
改之后的名单读工作区；变异表读工作区（HEAD 加暂存区加未暂存），sha256 写进产物（S21）。

读法 3 的三类（登记第十二节修订写了「r1 用的那张表那一行」的取法）：
- 甲：名字在开工那一刻提交的变异表里有、工作区表里没有，且删兼容的记录里写着这个坏实现（名字最后一个「：」之后那一截原样出现在记录里）。
- 乙：对照行（下面）的（文件，原文，替换文）三样在今天的表里逐字相同的恰一行；不止一行时取名字等于旧名照术语改名表换过之后的那一行，恰一行才算。
- 丙：三样在今天的表里一行都没有。
- 未分类：别的情形（对照行找不到、三样相同的不止一行又按改名挑不出一行）。
对照行：r1 第二段逐行驱动产物里这一行没补位、照名字取到并改坏的（`substituted_from=-`、`mutated_row_name` 等于 `row_name`），取它记的
`mutation_line_sha256` 在变异表历史各版本里那一行；别的（r1 里取不到、编不过换了补位）取登记行：名字相同、行文 sha256 前 12 位等于名单第 6 列的那一行。

只读：仓（git 只读命令）、r1 逐行驱动产物、术语改名表、删兼容的记录；写一份新产物（已存在就拒绝）。

跑法（在本目录）：
  python3 list_check.py --repository <仓根> --reference-commit <提交> --r1-rows-driver-product <产物> --output <新文件>
退出码：0 核完且名单改后那一格过；1 名单改后、钉值·名单或两份名单的读数有一样不过（Z12）；2 用法错。
"""

from __future__ import annotations

import argparse
import hashlib
import os
import re
import subprocess
import sys
from dataclasses import dataclass

from bad_implementation_table import MutationRow, SelectedRow, anchor_hit_count, line_digest_prefix, read_selected_rows
from result_lines import ResultLines, field

HERE = os.path.dirname(os.path.abspath(__file__))
ORIGINAL_DIRECTORY = "scripts/experiment-apparatus/e171_three_oracle_collusion"
THIS_DIRECTORY = "scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes"
LIST_DIRECTORIES = (ORIGINAL_DIRECTORY, THIS_DIRECTORY)
# 改名之前与之后的名单文件（登记第五节 5.4 第 2 条：去掉项数、按内容起名）；两张表按整表编号并成一张
STEP_8_LIST_BEFORE, STEP_4_ONLY_LIST_BEFORE = "bad-implementations-117.tsv", "bad-implementations-expansion-164.tsv"
STEP_8_LIST, STEP_4_ONLY_LIST = "bad-implementations-step-8.tsv", "bad-implementations-step-4-only.tsv"
LISTS_BEFORE = (STEP_8_LIST_BEFORE, STEP_4_ONLY_LIST_BEFORE)
LISTS_AFTER = (STEP_8_LIST, STEP_4_ONLY_LIST)
MUTATION_TABLE = "crates/mutations.tsv"
MUTATION_TABLE_COLUMNS = 6
TERM_RENAMES = ".claude/kb/term-renames.md"
TERM_RENAMES_MARK = "<!-- term-renames:table -->"
COMPATIBILITY_RECORD = "records/2026-10-31-删兼容旧版本代码.md"
# K1：退役位 0 的三个标识符（登记第七节 K1）
RETIRED_IDENTIFIERS = ("INCOMPAT_RETIRED", "retired_layout_bit", "old_reader_supported_bits")

CLASS_OLD_FORMAT = "jia"
CLASS_RENAMED = "yi"
CLASS_GONE = "bing"
CLASS_UNCLASSIFIED = "unclassified"
REFERENCE_FROM_R1 = "r1_line"
REFERENCE_FROM_REGISTRATION = "registration_line"
REFERENCE_NOT_FOUND = "not_found"


@dataclass(frozen=True)
class TableLine:
    """变异表的一行：六列与原样行文（行文 sha256 全长与逐行驱动记的 `mutation_line_sha256` 同一算法）。"""

    name: str
    file: str
    original: str
    replacement: str
    raw_line: str

    def triple(self) -> tuple[str, str, str]:
        return (self.file, self.original, self.replacement)

    def sha256(self) -> str:
        return hashlib.sha256(self.raw_line.encode("utf-8")).hexdigest()


@dataclass(frozen=True)
class R1Row:
    """r1 第二段逐行驱动产物里一行的定位记载。"""

    located_without_substitution: bool
    mutation_line_sha256: str
    status: str


@dataclass(frozen=True)
class Classification:
    row: SelectedRow
    klass: str
    reference_kind: str
    reference_line: TableLine | None
    reference_found_in: str
    in_reference_table: int
    in_worktree_table: int
    named_in_the_record: bool
    triple_matches: tuple[str, ...]
    anchor_matches: tuple[str, ...]
    renamed_name: str
    chosen_name: str


def table_lines_from_text(text: str) -> list[TableLine]:
    """与 `bad_implementation_table.read_mutation_rows` 同一读法：跳过 `#` 起头的行、只收恰 6 列的行。"""
    found = []
    for line in text.split("\n"):
        if line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) != MUTATION_TABLE_COLUMNS:
            continue
        found.append(TableLine(fields[0], fields[1], fields[2], fields[3], line))
    return found


def name_hits(table: list[TableLine], name: str) -> int:
    return sum(line.name == name for line in table)


def selected_rows_from_text(text: str) -> list[SelectedRow]:
    rows = []
    for line in text.split("\n"):
        fields = line.split("\t")
        if len(fields) != 6:
            continue
        number, slice_number, file, group_index, name, digest_prefix = fields
        rows.append(SelectedRow(int(number), int(slice_number), file, int(group_index), name, digest_prefix))
    return rows


TABLE_CELL_SEPARATOR = re.compile(r"(?<!\\)\|")


def term_renames_from_text(text: str) -> list[tuple[str, str, str]]:
    """术语改名表（旧, 新, 匹配）按表的次序；切格照 `scripts/kb/sweep-term.py` 的 `table_cells`：没转义的竖线切、去两头空格与反引号、`\\|` 还原。"""
    if TERM_RENAMES_MARK not in text:
        return []
    pairs = []
    for line in text[text.index(TERM_RENAMES_MARK):].split("\n"):
        if not line.startswith("|"):
            continue
        parts = TABLE_CELL_SEPARATOR.split(line.strip())
        if parts and not parts[0].strip():
            parts = parts[1:]
        if parts and not parts[-1].strip():
            parts = parts[:-1]
        cells = [part.strip().strip("`").replace("\\|", "|") for part in parts]
        if len(cells) < 3 or cells[0] in ("", "旧") or set(cells[0]) <= set("-: "):
            continue
        pairs.append((cells[0], cells[1], cells[2]))
    return pairs


def renamed(name: str, renames: list[tuple[str, str, str]]) -> str:
    """旧名照改名表逐条换过之后的名字：整串照字面换，词边界加 `\\b`，正则照原式换。"""
    text = name
    for old, new, how in renames:
        if how == "整串":
            text = text.replace(old, new)
        elif how == "词边界":
            text = re.sub(rf"\b{re.escape(old)}\b", new.replace("\\", "\\\\"), text)
        elif how == "正则":
            text = re.sub(old, new.replace("\\", "\\\\"), text)
    return text


def named_in_the_record(name: str, record_text: str) -> bool:
    """删兼容的记录写着这个坏实现：名字最后一个「：」之后那一截（坏实现本身的说法）原样出现在记录里。"""
    tail = name.rsplit("：", 1)[-1]
    return bool(tail) and tail in record_text


def r1_rows_from_text(text: str) -> dict[int, R1Row]:
    """r1 第二段逐行驱动合成产物的 `row row_number=N …` 行（值里的空白已换成下划线，名字比较前同样换）。"""
    found = {}
    for line in text.split("\n"):
        if not line.startswith("row row_number="):
            continue
        fields = dict(token.split("=", 1) for token in line.split(" ")[1:] if "=" in token)
        located = fields.get("substituted_from") == "-" and fields.get("mutated_row_name") == fields.get("row_name")
        found[int(fields["row_number"])] = R1Row(located, fields.get("mutation_line_sha256", "-"), fields.get("status", "-"))
    return found


def reference_line(row: SelectedRow, r1_row: R1Row | None, history: list[tuple[str, list[TableLine]]]) -> tuple[str, TableLine | None, str]:
    """对照行（文件头「对照行」）：r1 照名字取到、没补位的取它记的行文 sha256；别的取登记行（名字相同、行文 sha256 前 12 位等于名单第 6 列）。
    交回 (取法, 行, 在哪一版里找到的)。"""
    if r1_row is not None and r1_row.located_without_substitution:
        for label, lines in history:
            for line in lines:
                if line.sha256() == r1_row.mutation_line_sha256:
                    return REFERENCE_FROM_R1, line, label
        return REFERENCE_NOT_FOUND, None, "-"
    for label, lines in history:
        for line in lines:
            if line.name == row.name and line_digest_prefix(line.raw_line) == row.line_digest_prefix:
                return REFERENCE_FROM_REGISTRATION, line, label
    return REFERENCE_NOT_FOUND, None, "-"


def classify(row: SelectedRow, reference_table: list[TableLine], worktree_table: list[TableLine], record_text: str,
             renames: list[tuple[str, str, str]], r1_row: R1Row | None, history: list[tuple[str, list[TableLine]]]) -> Classification:
    """读法 3：一项取不到的名字归甲、乙、丙或未分类。"""
    in_reference, in_worktree = name_hits(reference_table, row.name), name_hits(worktree_table, row.name)
    in_record = named_in_the_record(row.name, record_text)
    kind, line, found_in = reference_line(row, r1_row, history)
    triples = tuple(today.name for today in worktree_table if line is not None and today.triple() == line.triple())
    anchors = tuple(today.name for today in worktree_table if line is not None and (today.file, today.original) == (line.file, line.original))
    new_name = renamed(row.name, renames)

    def result(klass: str, chosen: str) -> Classification:
        return Classification(row, klass, kind, line, found_in, in_reference, in_worktree, in_record, triples, anchors, new_name, chosen)

    if in_reference >= 1 and in_worktree == 0 and in_record:
        return result(CLASS_OLD_FORMAT, "-")
    if line is None:
        return result(CLASS_UNCLASSIFIED, "-")
    if len(triples) == 1:
        return result(CLASS_RENAMED, triples[0])
    if not triples:
        return result(CLASS_GONE, "-")
    by_rename = [name for name in triples if name == new_name]
    if len(by_rename) == 1:
        return result(CLASS_RENAMED, by_rename[0])
    return result(CLASS_UNCLASSIFIED, "-")


def expected_after_lists(before: dict[str, list[SelectedRow]], classified: dict[int, Classification]) -> dict[str, str]:
    """改之后两张名单应有的全文（改名之后的文件名 → 文本）：甲丙两类撤掉、整表编号不重编，乙类只换第 5 列，未分类的照留。"""
    texts = {}
    for before_name, after_name in zip(LISTS_BEFORE, LISTS_AFTER):
        lines = []
        for row in before[before_name]:
            entry = classified.get(row.number)
            if entry is not None and entry.klass in (CLASS_OLD_FORMAT, CLASS_GONE):
                continue
            name = entry.chosen_name if entry is not None and entry.klass == CLASS_RENAMED else row.name
            lines.append("\t".join((str(row.number), str(row.slice_number), row.file, str(row.group_index), name, row.line_digest_prefix)))
        texts[after_name] = "".join(line + "\n" for line in lines)
    return texts


def whole_table_row_numbers(directory: str = HERE) -> tuple[int, ...]:
    """整表行集：改名之后两张名单第一列之并，升序（登记读法 4：撤掉的行留空号，行集不再写死成 1–281）。
    第二段比对器的 `TABLE_ROWS` 与 `count_reusable_units.py` 的 `WHOLE_TABLE_ROWS` 都从这里现算。"""
    numbers: list[int] = []
    for name in LISTS_AFTER:
        numbers += [row.number for row in read_selected_rows(os.path.join(directory, name))]
    return tuple(sorted(numbers))


def git_text(repository: str, *arguments: str) -> str:
    completed = subprocess.run(["git", "-C", repository, *arguments], check=True, capture_output=True)
    return completed.stdout.decode("utf-8")


def history_tables(repository: str, reference_commit: str) -> list[tuple[str, list[TableLine]]]:
    """变异表在开工那一刻的提交及之前改过它的每一版（新的在前），找对照行用。"""
    commits = git_text(repository, "log", "--format=%H", reference_commit, "--", MUTATION_TABLE).split()
    return [(commit[:12], table_lines_from_text(git_text(repository, "show", f"{commit}:{MUTATION_TABLE}"))) for commit in commits]


def read_text(path: str) -> str:
    with open(path, encoding="utf-8") as handle:
        return handle.read()


def sha256_of_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def lookup_lines(lines: ResultLines, label: str, rows: list[SelectedRow], table: list[TableLine]) -> tuple[list[SelectedRow], list[SelectedRow]]:
    """Q2-逐项：一份名单（两张表并起来）逐项按名字数命中，交回 (命中 0 次的, 命中 2 次以上的)。"""
    not_found = [row for row in rows if name_hits(table, row.name) == 0]
    ambiguous = [row for row in rows if name_hits(table, row.name) >= 2]
    lines.result(f"name=q2_lookup directory={label} items={len(rows)} not_found={len(not_found)} "
                 f"not_found_rows={','.join(str(row.number) for row in not_found) or 'none'} ambiguous={len(ambiguous)} "
                 f"ambiguous_rows={','.join(str(row.number) for row in ambiguous) or 'none'}")
    for row in not_found:
        lines.result(f"name=q2_not_found directory={label} row={row.number} file={row.file} row_name={field(row.name)}")
    return not_found, ambiguous


def classification_line(entry: Classification) -> str:
    line = entry.reference_line
    return (f"name=classification row={entry.row.number} class={entry.klass} file={entry.row.file} row_name={field(entry.row.name)} "
            f"in_reference_table={entry.in_reference_table} in_worktree_table={entry.in_worktree_table} "
            f"named_in_the_record={str(entry.named_in_the_record).lower()} reference={entry.reference_kind} "
            f"reference_line_sha256={line.sha256() if line else '-'} reference_found_in={entry.reference_found_in} "
            f"triple_matches_today={len(entry.triple_matches)} anchor_matches_today={len(entry.anchor_matches)} "
            f"triple_match_names={field('|'.join(entry.triple_matches)) or '-'} renamed_name={field(entry.renamed_name)} "
            f"chosen_name={field(entry.chosen_name)}")


def numbers_text(numbers) -> str:
    return ",".join(map(str, sorted(numbers))) or "none"


def after_lines(lines: ResultLines, repository: str, before: dict[str, list[SelectedRow]], classified: dict[int, Classification],
                worktree_table: list[TableLine]) -> bool:
    """名单改后（第六节）：两个目录逐字节相同、每张表等于应有全文、行集 = 改之前减甲丙、编号不重、未分类之外每一项在今天的表里恰命中 1 次。"""
    expected = expected_after_lists(before, classified)
    texts = {}
    for directory in LIST_DIRECTORIES:
        for name in LISTS_AFTER:
            path = os.path.join(repository, directory, name)
            texts[(directory, name)] = read_text(path) if os.path.exists(path) else None
            lines.result(f"name=after_list directory={directory} list={name} present={str(texts[(directory, name)] is not None).lower()} "
                         f"rows={len(selected_rows_from_text(texts[(directory, name)] or ''))} "
                         f"sha256={sha256_of_text(texts[(directory, name)]) if texts[(directory, name)] is not None else '-'} "
                         f"equals_the_expected_text={str(texts[(directory, name)] == expected[name]).lower()}")
    identical = all(texts[(ORIGINAL_DIRECTORY, name)] == texts[(THIS_DIRECTORY, name)] for name in LISTS_AFTER)
    after_rows = [row for name in LISTS_AFTER for row in selected_rows_from_text(texts[(THIS_DIRECTORY, name)] or "")]
    numbers = [row.number for row in after_rows]
    withdrawn = {number for number, entry in classified.items() if entry.klass in (CLASS_OLD_FORMAT, CLASS_GONE)}
    retargeted = {number for number, entry in classified.items() if entry.klass == CLASS_RENAMED}
    unclassified = {number for number, entry in classified.items() if entry.klass == CLASS_UNCLASSIFIED}
    before_numbers = {row.number for rows in before.values() for row in rows}
    set_ok = set(numbers) == before_numbers - withdrawn
    unique = len(numbers) == len(set(numbers))
    not_once = [row.number for row in after_rows if row.number not in unclassified and name_hits(worktree_table, row.name) != 1]
    retargeted_ok = all(next((row.name for row in after_rows if row.number == number), None) == classified[number].chosen_name
                        for number in retargeted)
    passed = (identical and set_ok and unique and not not_once and retargeted_ok
              and all(texts[(directory, name)] == expected[name] for directory in LIST_DIRECTORIES for name in LISTS_AFTER))
    lines.result(f"name=after_lists directories_identical={str(identical).lower()} rows={len(numbers)} withdrawn_rows={numbers_text(withdrawn)} "
                 f"retargeted_rows={numbers_text(retargeted)} unclassified_rows={numbers_text(unclassified)} "
                 f"row_set_equals_before_minus_withdrawn={str(set_ok).lower()} numbers_unique={str(unique).lower()} "
                 f"retargeted_names_are_the_chosen_ones={str(retargeted_ok).lower()} items_not_hit_once={numbers_text(not_once)} "
                 f"passed={str(passed).lower()}")
    for number in sorted(retargeted):
        lines.result(f"name=retargeted row={number} old_name={field(classified[number].row.name)} new_name={field(classified[number].chosen_name)}")
    return passed


def pinned_value_line(lines: ResultLines) -> bool:
    """钉值·名单：代码里原来写死行集与项数的两处（第二段比对器、复用计数）改成现算之后各自现算出的值，要等于改后名单的行数。"""
    expected = len(whole_table_row_numbers())
    found = {}
    for label, module_name, attribute in (("segment_two_comparator_table_rows", "segment_two_comparator", "TABLE_ROWS"),
                                          ("count_reusable_units_whole_table_rows", "count_reusable_units", "WHOLE_TABLE_ROWS")):
        try:
            value = getattr(__import__(module_name), attribute)
            found[label] = str(value if isinstance(value, int) else len(value))
        except Exception as error:  # 载入失败照实写进产物，判不过（Z12），不吞
            found[label] = f"import_failed:{field(type(error).__name__)}"
    equal = all(value == str(expected) for value in found.values())
    lines.result(f"name=pinned_values list_rows={expected} " + " ".join(f"{key}={value}" for key, value in found.items())
                 + f" equal={str(equal).lower()}")
    return equal


def current_row_lines(lines: ResultLines, repository: str, worktree_table: list[TableLine]) -> None:
    """现状·可取行：改后名单每一行在工作区按名字取几次、锚点（原文，`\\n` 还原）在源文件里命中几次；不是恰 1 次的逐行列，不编不跑。"""
    rows = [row for name in LISTS_AFTER for row in read_selected_rows(os.path.join(repository, THIS_DIRECTORY, name))]
    sources: dict[str, str | None] = {}
    not_once = 0
    for row in rows:
        hits = [line for line in worktree_table if line.name == row.name]
        anchor = "-"
        if len(hits) == 1:
            line = hits[0]
            if line.file not in sources:
                path = os.path.join(repository, line.file)
                sources[line.file] = read_text(path) if os.path.exists(path) else None
            source = sources[line.file]
            anchor = "file_missing" if source is None else str(anchor_hit_count(
                source, MutationRow(line.name, line.file, line.original, line.replacement, "", "", line.raw_line)))
        if len(hits) != 1 or anchor != "1":
            not_once += 1
            lines.result(f"name=current_row row={row.number} file={row.file} name_hits={len(hits)} anchor_hits={anchor} row_name={field(row.name)}")
    lines.result(f"name=current_rows rows={len(rows)} rows_not_hit_once_by_name_or_anchor={not_once}")


def retired_bit_lines(lines: ResultLines, repository: str, reference_table: list[TableLine], worktree_table: list[TableLine],
                      old_format_names: list[str]) -> None:
    """第七节 K1–K3：退役位 0 的三个标识符在 crates/ 下的 .rs 里 0 处；甲类每一项的原文在今天的源文件里 0 次；名字在开工那一刻的表里恰 1 次、工作区 0 次。"""
    identifier_hits = 0
    for root, _, names in os.walk(os.path.join(repository, "crates")):
        for name in names:
            if name.endswith(".rs"):
                text = read_text(os.path.join(root, name))
                identifier_hits += sum(text.count(identifier) for identifier in RETIRED_IDENTIFIERS)
    lines.result(f"name=k1_identifiers retired_identifier_hits_in_crates={identifier_hits}")
    for name in old_format_names:
        reference = [line for line in reference_table if line.name == name]
        source_hits = "-"
        if len(reference) == 1:
            path = os.path.join(repository, reference[0].file)
            source_hits = str(read_text(path).count(reference[0].original.replace("\\n", "\n"))) if os.path.exists(path) else "file_missing"
        lines.result(f"name=k1_k3_old_format_row row_name={field(name)} in_reference_table={len(reference)} "
                     f"in_worktree_table={name_hits(worktree_table, name)} original_text_hits_in_source={source_hits}")


QUESTION_ROW = 236  # 问题单 Q1 点名的那一项（扩表第 119 行），Q2 问的是它之外还有没有取不到的


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E171 第 2 次重跑的名单核对（登记第六节 Q2、名单改后、钉值·名单、现状·可取行，第七节 K1–K3）")
    parser.add_argument("--repository", required=True, help="仓根")
    parser.add_argument("--reference-commit", required=True, help="开工那一刻的提交（改之前的名单与 HEAD 那一版变异表从这里取）")
    parser.add_argument("--r1-rows-driver-product", required=True, help="r1 第二段逐行驱动合成产物（对照行的 mutation_line_sha256）")
    parser.add_argument("--output", required=True)
    arguments = parser.parse_args(argument_list)
    repository = os.path.abspath(arguments.repository)
    worktree_text = read_text(os.path.join(repository, MUTATION_TABLE))
    worktree_table = table_lines_from_text(worktree_text)
    reference_text = git_text(repository, "show", f"{arguments.reference_commit}:{MUTATION_TABLE}")
    reference_table = table_lines_from_text(reference_text)
    record_text = read_text(os.path.join(repository, COMPATIBILITY_RECORD))
    renames_text = read_text(os.path.join(repository, TERM_RENAMES))
    r1_text = read_text(arguments.r1_rows_driver_product)
    lines = ResultLines(arguments.output)
    lines.result(f"name=config reference_commit={arguments.reference_commit} worktree_mutation_table_sha256={sha256_of_text(worktree_text)} "
                 f"reference_mutation_table_sha256={sha256_of_text(reference_text)} record_sha256={sha256_of_text(record_text)} "
                 f"term_renames_sha256={sha256_of_text(renames_text)} r1_rows_driver_product={os.path.basename(arguments.r1_rows_driver_product)} "
                 f"r1_rows_driver_product_sha256={sha256_of_text(r1_text)}")
    before_by_directory: dict[str, dict[str, list[SelectedRow]]] = {}
    before_texts = {}
    for directory in LIST_DIRECTORIES:
        before_by_directory[directory] = {}
        for name in LISTS_BEFORE:
            text = git_text(repository, "show", f"{arguments.reference_commit}:{directory}/{name}")
            before_texts[(directory, name)] = text
            before_by_directory[directory][name] = selected_rows_from_text(text)
            lines.result(f"name=before_list directory={directory} list={name} rows={len(before_by_directory[directory][name])} "
                         f"sha256={sha256_of_text(text)}")
    lookups = {}
    for directory in LIST_DIRECTORIES:
        rows = [row for name in LISTS_BEFORE for row in before_by_directory[directory][name]]
        not_found, ambiguous = lookup_lines(lines, directory, rows, worktree_table)
        lookups[directory] = ([row.number for row in not_found], [row.number for row in ambiguous])
    directories_agree = (lookups[ORIGINAL_DIRECTORY] == lookups[THIS_DIRECTORY]
                         and all(before_texts[(ORIGINAL_DIRECTORY, name)] == before_texts[(THIS_DIRECTORY, name)] for name in LISTS_BEFORE))
    not_found_rows = lookups[THIS_DIRECTORY][0]
    besides = [number for number in not_found_rows if number != QUESTION_ROW]
    lines.result(f"name=q2 directories_agree={str(directories_agree).lower()} not_found_rows={numbers_text(not_found_rows)} "
                 f"not_found_besides_row_{QUESTION_ROW}={len(besides)} verdict={'yes' if besides else 'no'}")
    renames = term_renames_from_text(renames_text)
    r1_rows = r1_rows_from_text(r1_text)
    history = history_tables(repository, arguments.reference_commit)
    before = before_by_directory[THIS_DIRECTORY]
    by_number = {row.number: row for rows in before.values() for row in rows}
    classified = {number: classify(by_number[number], reference_table, worktree_table, record_text, renames, r1_rows.get(number), history)
                  for number in not_found_rows}
    for number in sorted(classified):
        lines.result(classification_line(classified[number]))
    def rows_of_class(klass: str) -> str:
        return numbers_text(number for number, entry in classified.items() if entry.klass == klass)

    lines.result(f"name=disposition withdrawn_old_format={rows_of_class(CLASS_OLD_FORMAT)} withdrawn_gone={rows_of_class(CLASS_GONE)} "
                 f"retargeted={rows_of_class(CLASS_RENAMED)} unclassified={rows_of_class(CLASS_UNCLASSIFIED)}")
    after_passed = after_lines(lines, repository, before, classified, worktree_table)
    pinned_equal = pinned_value_line(lines)
    current_row_lines(lines, repository, worktree_table)
    retired_bit_lines(lines, repository, reference_table, worktree_table,
                      [entry.row.name for entry in classified.values() if entry.klass == CLASS_OLD_FORMAT])
    final_worktree_sha256 = sha256_of_text(read_text(os.path.join(repository, MUTATION_TABLE)))
    table_unchanged = final_worktree_sha256 == sha256_of_text(worktree_text)
    unclassified = sum(entry.klass == CLASS_UNCLASSIFIED for entry in classified.values())
    lines.result(f"name=verdict q2={'yes' if besides else 'no'} directories_agree={str(directories_agree).lower()} unclassified={unclassified} "
                 f"after_lists_passed={str(after_passed).lower()} pinned_values_equal={str(pinned_equal).lower()} "
                 f"mutation_table_unchanged_during_the_run={str(table_unchanged).lower()}")
    lines.finish()
    return 0 if directories_agree and after_passed and pinned_equal and table_unchanged else 1


if __name__ == "__main__":
    sys.path.insert(0, HERE)
    sys.exit(main(sys.argv[1:]))
