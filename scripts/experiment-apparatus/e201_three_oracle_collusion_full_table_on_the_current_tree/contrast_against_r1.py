"""E201（三oracle合谋今树整表） 与 r1 逐格比（跑前登记 research/prompts/e201-preregistration.md 第一节读法 7；第六节 Q1-配键、Q1-判定格差、Q1-钉值格差、
Q1-今天独有；第七节 A3、A5；第十节 F5；第十一节 Z5）。

两边：r1 留在 git 里的三份产物 P1（第二段比对器）、P2（第二段逐行驱动合成产物）、P3（第一段比对器），与今天这一趟合并那一步出的 T1、T2、T3（同形）。
配键与分类照本目录 `cell_classes.tsv`（读法 7 那张表，sha256 钉在 CLASSES_SHA256，跑前登记修订 2 记的就是这个值；对不上退 2、不比）。
一格变了 = 同一个键、同一个判定格两边取值不同（集合按集合相等比，红的计数按「是否大于 0」比）；只在一边的键算那一行「在不在」一格变了。
钉值格的差另报，不进 Q1 的判定；今天有而 r1 没有的格记今天独有，照报。r1 有、今天没有的判定字段或判定行种记「未比」（F5）。

两段（读法 11、修订 1 第 1.2 条）：`--segment streams_1_2` 比 P1/T1、P2/T2 与 P3/T3 里流一、流二那一半；`--segment stream_three` 只比 P3/T3 里流三那几格
（`q1` 的 `streams=three_streams`、流三的 `q1_row`、`q2_named_row` 的 `streams=stream_3` 与流三的调用、判决行的 `q1`）。

只读、写一份新产物（已存在就拒）。跑法（在本目录，整条命令交 `task-server/task submit --`；合并那一步由 full_table_run.py 在进程里调 compare_segment）：
  python3 contrast_against_r1.py --segment streams_1_2 --p1 <P1> --p2 <P2> --p3 <P3> --t1 <T1> --t2 <T2> --t3 <T3> --output <新文件>
  python3 contrast_against_r1.py --segment stream_three --p3 <P3> --t3 <T3> --output <新文件>
退出码：0 比完（判定格变没变都是 0，判定在产物里）；1 自检不过（A5：键数与另数的行数对不上、格数加不齐）；2 用法错、分类表或 r1 产物核不过（Z5）、r1 有没归类的字段。
"""

from __future__ import annotations

import argparse
import dataclasses
import hashlib
import os
import sys

import e171_research_side  # noqa: F401  （把 E171 研究一侧放进 import 路径）
from result_lines import ResultLines, field

HERE = os.path.dirname(os.path.abspath(__file__))
CLASSES_PATH = os.path.join(HERE, "cell_classes.tsv")
CLASSES_SHA256 = "4c053fa2a7325ec16a7983121231ade70ca5269547baff922654162aca93d39a"
# 读法 2：r1 留在 git 里的三份产物（相对仓根）、sha256 与行数（作废 Z5）
R1_PRODUCTS = {
    "P1": ("research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-comparator-r6.out",
           "d0bc79ca3f580774cfd4bad2fd672006b98c8a5d02063c48fb9915cb246a8549", 451),
    "P2": ("research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-rows-driver-r6.out",
           "59c89b96c5bef3151c8b4885d98f8fe4db680a300106a9de400af4188cf74e7c", 565),
    "P3": ("research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3-2026-10-07.out",
           "40ee800a8681caf503c05131fbd05fcdba7a9514a38178660c55195330b1cabc", 811),
}
SEGMENT_STREAMS_ONE_TWO = "streams_1_2"
SEGMENT_STREAM_THREE = "stream_three"
SEGMENTS = (SEGMENT_STREAMS_ONE_TWO, SEGMENT_STREAM_THREE)
STREAM_ONE, STREAM_TWO, STREAM_THREE = "new_pool_file_creation", "fixed_script_through_e_then_unmount", "first_file_unlinked_then_inodes_across_one_leaf"
CLASS_KEY, CLASS_DECISION, CLASS_DECISION_SET, CLASS_DECISION_POSITIVE = "key", "decision", "decision_set", "decision_positive"
CLASS_PINNED, CLASS_NOT_COMPARED, CLASS_TODAY_ONLY = "pinned", "not_compared", "today_only"
DECISION_CLASSES = (CLASS_DECISION, CLASS_DECISION_SET, CLASS_DECISION_POSITIVE)
CLASSES = (CLASS_KEY, *DECISION_CLASSES, CLASS_PINNED, CLASS_NOT_COMPARED, CLASS_TODAY_ONLY)
PRESENCE_FIELD = "<present>"
ABSENT_VALUE = "<absent>"
EMPTY_SET_WORD = "-"
OUTCOME_SAME, OUTCOME_CHANGED = "same", "changed"
DRIVER_ROW_KIND = "row"
RESULT_PREFIX = "E7RESULT name="
DRIVER_ROW_PREFIX = "row "


@dataclasses.dataclass(frozen=True)
class CellClasses:
    """读法 7 那张表读进来：{(产物, 行种): {字段: 类}}；字段写 * 的是这一种里没单列的字段的类；键字段按表里的先后。"""

    by_kind: dict[tuple[str, str], dict[str, str]]
    keys: dict[tuple[str, str], tuple[str, ...]]

    def class_of(self, product: str, kind: str, name: str) -> str | None:
        fields = self.by_kind.get((product, kind))
        if fields is None:
            return None
        return fields.get(name, fields.get("*"))

    def kind_is_compared(self, product: str, kind: str) -> bool:
        fields = self.by_kind.get((product, kind))
        return fields is not None and fields != {"*": CLASS_NOT_COMPARED}


def read_cell_classes(text: str) -> CellClasses:
    by_kind: dict[tuple[str, str], dict[str, str]] = {}
    keys: dict[tuple[str, str], list[str]] = {}
    for number, line in enumerate(text.split("\n"), start=1):
        if not line or line.startswith("#"):
            continue
        columns = line.split("\t")
        if len(columns) != 5 or columns[3] not in CLASSES:
            raise ValueError(f"分类表第 {number} 行认不得：{line!r}")
        product, kind, name, cell_class, _reason = columns
        fields = by_kind.setdefault((product, kind), {})
        if name in fields:
            raise ValueError(f"分类表第 {number} 行重复登记 {product} {kind} {name}")
        fields[name] = cell_class
        if cell_class == CLASS_KEY:
            keys.setdefault((product, kind), []).append(name)
    return CellClasses(by_kind, {pair: tuple(names) for pair, names in keys.items()})


def sha256_of_file(path: str) -> str:
    with open(path, "rb") as handle:
        return hashlib.sha256(handle.read()).hexdigest()


def load_cell_classes(path: str = CLASSES_PATH, expected_sha256: str = CLASSES_SHA256) -> CellClasses:
    """读分类表，先核它的 sha256 等于登记修订 2 记的那一个（读法 7「记 sha256、之后不改」）。"""
    actual = sha256_of_file(path)
    if actual != expected_sha256:
        raise ValueError(f"分类表 {path} 的 sha256 是 {actual}，登记记的是 {expected_sha256}：表在登记之后被改过，不比")
    with open(path, encoding="utf-8") as handle:
        return read_cell_classes(handle.read())


def line_fields(tokens: list[str]) -> dict[str, str]:
    return {name: value for name, _, value in (token.partition("=") for token in tokens if "=" in token)}


def read_product_entries(text: str) -> list[tuple[str, dict[str, str]]]:
    """一份产物里的条目：比对器的 `E7RESULT name=<行种> …` 行，与逐行驱动合成产物的 `row row_number=…` 行（行种记 row）。别的行不收。"""
    entries = []
    for raw in text.split("\n"):
        if raw.startswith(RESULT_PREFIX):
            tokens = raw[len("E7RESULT "):].split(" ")
            fields = line_fields(tokens[1:])
            entries.append((tokens[0][len("name="):], fields))
        elif raw.startswith(DRIVER_ROW_PREFIX):
            entries.append((DRIVER_ROW_KIND, line_fields(raw.split(" ")[1:])))
    return entries


# A5 的另一份数法认这一段的行用的子串（行尾补一个空格再找）：P3 里没列的（行种, 段）整种都算，P1、P2 整份都算
SEGMENT_MARKERS = {
    ("q1", SEGMENT_STREAMS_ONE_TWO): (" streams=two_streams ",), ("q1", SEGMENT_STREAM_THREE): (" streams=three_streams ",),
    ("q1_row", SEGMENT_STREAMS_ONE_TWO): (f" stream={STREAM_ONE} ", f" stream={STREAM_TWO} "),
    ("q1_row", SEGMENT_STREAM_THREE): (f" stream={STREAM_THREE} ",),
    ("q2_named_row_call", SEGMENT_STREAMS_ONE_TWO): (f" stream={STREAM_ONE} ", f" stream={STREAM_TWO} "),
    ("q2_named_row_call", SEGMENT_STREAM_THREE): (f" stream={STREAM_THREE} ",),
    ("q2_named_row", SEGMENT_STREAMS_ONE_TWO): (" streams=streams_1_2 ", " ran=false "),
    ("q2_named_row", SEGMENT_STREAM_THREE): (" streams=stream_3 ",),
    ("verdict", SEGMENT_STREAMS_ONE_TWO): (" q3=",), ("verdict", SEGMENT_STREAM_THREE): (" q1=",),
}


def count_lines_of_kind(text: str, kind: str, driver_rows: bool, segment: str, product: str) -> int:
    """A5 的另一份数法：按行首前缀与这一段的子串数（与 read_product_entries、segment_view 的切分不共用代码）。"""
    prefix = DRIVER_ROW_PREFIX if driver_rows else f"{RESULT_PREFIX}{kind} "
    markers = SEGMENT_MARKERS.get((kind, segment), ()) if product == "P3" else ()
    return sum(1 for raw in text.splitlines()
               if (raw + " ").startswith(prefix) and (not markers or any(marker in raw + " " for marker in markers)))


def segment_view(product: str, kind: str, fields: dict[str, str], segment: str) -> dict[str, str] | None:
    """这一条在这一段里比哪几个字段（读法 7、11，修订 1 第 1.2 条）：交回要比的那几个字段，整条不属这一段交 None。"""
    if product in ("P1", "P2"):
        return dict(fields) if segment == SEGMENT_STREAMS_ONE_TWO else None
    first_segment = segment == SEGMENT_STREAMS_ONE_TWO
    if kind == "q1":
        return dict(fields) if fields.get("streams") == ("two_streams" if first_segment else "three_streams") else None
    if kind in ("q1_row", "q2_named_row_call"):
        on_stream_three = fields.get("stream") == STREAM_THREE
        return dict(fields) if on_stream_three != first_segment else None
    if kind == "q2_named_row":
        streams = fields.get("streams")
        if streams is None:  # ran=false 那一种没有 streams：只在流一、流二那一段比一次
            return dict(fields) if first_segment else None
        return dict(fields) if (streams == "stream_3") != first_segment else None
    if kind == "q3":
        return dict(fields) if first_segment else None
    if kind == "verdict":  # P3 判决行的 q1 是「三流、严格、批 1」那一圈的判定，q3 只看流一、流二
        kept = {name: value for name, value in fields.items() if name == ("q3" if first_segment else "q1")}
        return kept or None
    return dict(fields)


@dataclasses.dataclass(frozen=True)
class Cell:
    """一格：产物（P1/P2/P3，比的是它与 T1/T2/T3）、行种、键（`字段:值` 逗号连起来，单条的行种写 -）、字段（键在不在那一格写 <present>）、类、批、两边的值、结局。"""

    product: str
    kind: str
    key: str
    name: str
    cell_class: str
    batch: str
    r1: str
    today: str
    outcome: str


@dataclasses.dataclass
class KindTally:
    """一种行种的计数（第六节 Q1-配键；A5 的自检两份数）。"""

    keys_r1: int = 0
    keys_today: int = 0
    keys_both: int = 0
    keys_only_r1: int = 0
    keys_only_today: int = 0
    decision_same: int = 0
    decision_changed: int = 0
    pinned_same: int = 0
    pinned_changed: int = 0
    today_unique: int = 0
    duplicate_keys_r1: int = 0
    duplicate_keys_today: int = 0
    unchecked_fields: tuple[str, ...] = ()


def parse_row_set(value: str) -> frozenset[str]:
    return frozenset() if value in (EMPTY_SET_WORD, "") else frozenset(value.split(","))


def is_positive(value: str) -> str:
    """红的计数取「是否大于 0」（读法 7 判定格④）；不是整数的值（缺、-）原样当一个词比。"""
    return str(int(value) > 0).lower() if value.lstrip("-").isdigit() else value


def decision_values_differ(cell_class: str, r1_value: str, today_value: str) -> bool:
    if cell_class == CLASS_DECISION_SET:
        return parse_row_set(r1_value) != parse_row_set(today_value)
    if cell_class == CLASS_DECISION_POSITIVE:
        return is_positive(r1_value) != is_positive(today_value)
    return r1_value != today_value


def batch_of(key_fields: dict[str, str], name: str) -> str:
    """这一格属哪一批（第八节几何敏感性：批 1 定判、批 2 并列）：键里有 batch 取它，字段名带 batch_2 记 2，其余记 1。"""
    if "batch" in key_fields:
        return key_fields["batch"]
    return "2" if "batch_2" in name else "1"


def key_text(key_names: tuple[str, ...], fields: dict[str, str]) -> str:
    return ",".join(f"{name}:{fields.get(name, ABSENT_VALUE)}" for name in key_names) or "-"


def index_by_key(entries: list[dict[str, str]], key_names: tuple[str, ...]) -> tuple[dict[str, dict[str, str]], int]:
    """按键收条目；同一把键第二次出现记一次重键（只留第一条）。交回 ({键: 字段}, 重键数)。"""
    indexed: dict[str, dict[str, str]] = {}
    duplicates = 0
    for fields in entries:
        key = key_text(key_names, fields)
        if key in indexed:
            duplicates += 1
            continue
        indexed[key] = fields
    return indexed, duplicates


def compare_kind(product: str, kind: str, r1_entries: list[dict[str, str]], today_entries: list[dict[str, str]],
                 classes: CellClasses) -> tuple[list[Cell], KindTally, list[str]]:
    """一种行种逐键逐字段比。交回 (变了的格与钉值格的差, 计数, r1 有而分类表没归类的字段)。
    r1 有、今天没有的判定字段（今天这一种有条目、却一条都没有这个字段）记进 unchecked_fields、那几格不比（F5）。"""
    key_names = classes.keys.get((product, kind), ())
    r1_by_key, r1_duplicates = index_by_key(r1_entries, key_names)
    today_by_key, today_duplicates = index_by_key(today_entries, key_names)
    tally = KindTally(keys_r1=len(r1_by_key), keys_today=len(today_by_key), duplicate_keys_r1=r1_duplicates,
                      duplicate_keys_today=today_duplicates)
    r1_schema = {name for fields in r1_entries for name in fields} - set(key_names)
    today_schema = {name for fields in today_entries for name in fields} - set(key_names)
    unclassified = sorted(name for name in r1_schema if classes.class_of(product, kind, name) is None)
    unchecked = tuple(sorted(name for name in r1_schema - today_schema
                             if classes.class_of(product, kind, name) in DECISION_CLASSES))
    tally.unchecked_fields = unchecked
    cells: list[Cell] = []
    for key in sorted(set(r1_by_key) | set(today_by_key)):
        r1_fields, today_fields = r1_by_key.get(key), today_by_key.get(key)
        if r1_fields is None or today_fields is None:
            present = today_fields if today_fields is not None else r1_fields
            assert present is not None
            if r1_fields is None:
                tally.keys_only_today += 1
            else:
                tally.keys_only_r1 += 1
            tally.decision_changed += 1
            cells.append(Cell(product, kind, key, PRESENCE_FIELD, CLASS_DECISION, batch_of(present, ""),
                              "present" if r1_fields is not None else ABSENT_VALUE, "present" if today_fields is not None else ABSENT_VALUE,
                              OUTCOME_CHANGED))
            continue
        tally.keys_both += 1
        for name in sorted((set(r1_fields) | set(today_fields)) - set(key_names)):
            cell_class = classes.class_of(product, kind, name)
            if cell_class is None or cell_class == CLASS_TODAY_ONLY:
                if name in today_fields:
                    tally.today_unique += 1
                continue
            if cell_class == CLASS_NOT_COMPARED or name in unchecked:
                continue
            r1_value, today_value = r1_fields.get(name, ABSENT_VALUE), today_fields.get(name, ABSENT_VALUE)
            if cell_class in DECISION_CLASSES:
                differs = decision_values_differ(cell_class, r1_value, today_value)
                if differs:
                    tally.decision_changed += 1
                else:
                    tally.decision_same += 1
            else:
                differs = r1_value != today_value
                if differs:
                    tally.pinned_changed += 1
                else:
                    tally.pinned_same += 1
            if differs:
                cells.append(Cell(product, kind, key, name, cell_class, batch_of({**r1_fields, **today_fields}, name), r1_value, today_value,
                                  OUTCOME_CHANGED))
    return cells, tally, unclassified


@dataclasses.dataclass
class SegmentComparison:
    """一段的逐格比：变了的格（判定格与钉值格）、每种行种的计数、未比的判定行种（F5）、A5 的另一份行数。"""

    cells: list[Cell]
    tallies: dict[tuple[str, str], KindTally]
    unchecked_kinds: list[tuple[str, str]]
    unclassified: list[str]
    grep_counts: dict[tuple[str, str], tuple[int, int]]
    today_only_kinds: dict[tuple[str, str], int]

    def decision_cells(self) -> list[Cell]:
        return [cell for cell in self.cells if cell.cell_class in DECISION_CLASSES]

    def pinned_cells(self) -> list[Cell]:
        return [cell for cell in self.cells if cell.cell_class == CLASS_PINNED]


def compare_product(product: str, r1_text: str, today_text: str, classes: CellClasses, segment: str, comparison: SegmentComparison) -> None:
    """一对产物（P 与 T）在一段里逐种比，结果并进 comparison。"""
    driver_rows = product == "P2"
    grouped: dict[str, tuple[list[dict[str, str]], list[dict[str, str]]]] = {}
    for side, text in ((0, r1_text), (1, today_text)):
        for kind, fields in read_product_entries(text):
            viewed = segment_view(product, kind, fields, segment)
            if viewed is not None:
                grouped.setdefault(kind, ([], []))[side].append(viewed)
    for kind in sorted(grouped):
        r1_entries, today_entries = grouped[kind]
        if (product, kind) not in classes.by_kind:
            if r1_entries:
                comparison.unclassified.append(f"{product}:{kind}:*")
            else:
                comparison.today_only_kinds[(product, kind)] = len(today_entries)
            continue
        if not classes.kind_is_compared(product, kind):
            continue
        if r1_entries and not today_entries:
            if any(classes.class_of(product, kind, name) in DECISION_CLASSES for fields in r1_entries for name in fields):
                comparison.unchecked_kinds.append((product, kind))
            continue
        cells, tally, unclassified = compare_kind(product, kind, r1_entries, today_entries, classes)
        comparison.cells.extend(cells)
        comparison.tallies[(product, kind)] = tally
        comparison.unclassified.extend(f"{product}:{kind}:{name}" for name in unclassified)
        comparison.grep_counts[(product, kind)] = (count_lines_of_kind(r1_text, kind, driver_rows, segment, product),
                                                   count_lines_of_kind(today_text, kind, driver_rows, segment, product))


def compare_segment(pairs: list[tuple[str, str, str]], classes: CellClasses, segment: str) -> SegmentComparison:
    """几对产物（产物名, r1 那一份的全文, 今天那一份的全文）在一段里逐格比。"""
    comparison = SegmentComparison([], {}, [], [], {}, {})
    for product, r1_text, today_text in pairs:
        compare_product(product, r1_text, today_text, classes, segment, comparison)
    return comparison


def self_check_problems(comparison: SegmentComparison) -> list[str]:
    """A5：每种行种，比出来的键数（加重键）等于另按行首数的行数；每种「相同 + 变了 + 只在一边」等于另数的格总数。"""
    problems = []
    for pair, tally in sorted(comparison.tallies.items()):
        grep_r1, grep_today = comparison.grep_counts[pair]
        if tally.keys_r1 + tally.duplicate_keys_r1 != grep_r1 or tally.keys_today + tally.duplicate_keys_today != grep_today:
            problems.append(f"{pair[0]}:{pair[1]}:keys_r1={tally.keys_r1}+{tally.duplicate_keys_r1}/grep={grep_r1},"
                            f"keys_today={tally.keys_today}+{tally.duplicate_keys_today}/grep={grep_today}")
        compared = tally.decision_same + tally.decision_changed + tally.pinned_same + tally.pinned_changed
        expected = tally.keys_only_r1 + tally.keys_only_today + sum(
            1 for cell in comparison.cells if (cell.product, cell.kind) == pair and cell.name != PRESENCE_FIELD) \
            + tally.decision_same + tally.pinned_same
        if compared != expected:
            problems.append(f"{pair[0]}:{pair[1]}:cells={compared}/expected={expected}")
    return problems


def r1_product_problems(repository_root: str, paths: dict[str, str]) -> list[str]:
    """作废 Z5：给的 r1 产物路径要是读法 2 那几份（相对仓根同一路径），sha256 与行数等于登记记的。"""
    problems = []
    for product, path in sorted(paths.items()):
        expected_path, expected_sha256, expected_lines = R1_PRODUCTS[product]
        if os.path.abspath(path) != os.path.abspath(os.path.join(repository_root, expected_path)):
            problems.append(f"{product}:path={path}")
            continue
        with open(path, "rb") as handle:
            content = handle.read()
        if hashlib.sha256(content).hexdigest() != expected_sha256 or content.count(b"\n") != expected_lines:
            problems.append(f"{product}:sha256_or_lines")
    return problems


def write_comparison_lines(lines: ResultLines, comparison: SegmentComparison, segment: str) -> int:
    """逐格（变了的判定格、变了的钉值格）、逐种计数、未比与今天独有、合计。交回判定格变了的格数 c（批 1 那一批）。"""
    for cell in comparison.cells:
        line_name = "contrast_cell" if cell.cell_class in DECISION_CLASSES else "contrast_pinned_cell"
        lines.result(f"name={line_name} segment={segment} product={cell.product} kind={cell.kind} key={field(cell.key)} field={field(cell.name)} "
                     f"class={cell.cell_class} batch={cell.batch} r1={field(cell.r1)} today={field(cell.today)} outcome={cell.outcome}")
    for (product, kind), tally in sorted(comparison.tallies.items()):
        grep_r1, grep_today = comparison.grep_counts[(product, kind)]
        lines.result(f"name=contrast_kind segment={segment} product={product} kind={kind} keys_r1={tally.keys_r1} keys_today={tally.keys_today} "
                     f"keys_both={tally.keys_both} keys_only_r1={tally.keys_only_r1} keys_only_today={tally.keys_only_today} "
                     f"duplicate_keys_r1={tally.duplicate_keys_r1} duplicate_keys_today={tally.duplicate_keys_today} grep_r1={grep_r1} grep_today={grep_today} "
                     f"decision_same={tally.decision_same} decision_changed={tally.decision_changed} pinned_same={tally.pinned_same} "
                     f"pinned_changed={tally.pinned_changed} today_unique={tally.today_unique} "
                     f"unchecked_fields={field(','.join(tally.unchecked_fields) or '-')}")
    for product, kind in comparison.unchecked_kinds:
        lines.result(f"name=contrast_unchecked_kind segment={segment} product={product} kind={kind} clause=F5")
    for (product, kind), count in sorted(comparison.today_only_kinds.items()):
        lines.result(f"name=contrast_today_only_kind segment={segment} product={product} kind={kind} entries={count}")
    decision = comparison.decision_cells()
    batch_one = sum(cell.batch != "2" for cell in decision)
    batch_two = sum(cell.batch == "2" for cell in decision)
    unchecked_fields = sum(len(tally.unchecked_fields) for tally in comparison.tallies.values())
    lines.result(f"name=contrast_total segment={segment} decision_changed_batch_1={batch_one} decision_changed_batch_2={batch_two} "
                 f"pinned_changed={len(comparison.pinned_cells())} unchecked_kinds={len(comparison.unchecked_kinds)} unchecked_fields={unchecked_fields} "
                 f"today_unique={sum(tally.today_unique for tally in comparison.tallies.values()) + sum(comparison.today_only_kinds.values())}")
    return batch_one


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E201 与 r1 逐格比")
    parser.add_argument("--segment", required=True, choices=SEGMENTS)
    for name in ("p1", "p2", "p3", "t1", "t2", "t3"):
        parser.add_argument(f"--{name}")
    parser.add_argument("--repository-root", default=e171_research_side.REPOSITORY_ROOT)
    parser.add_argument("--output", required=True)
    arguments = parser.parse_args(argument_list)
    products = ("P1", "P2", "P3") if arguments.segment == SEGMENT_STREAMS_ONE_TWO else ("P3",)
    missing = [name for product in products for name in (product.lower(), "t" + product[1]) if not getattr(arguments, name)]
    if missing:
        parser.error("这一段要给：" + "、".join(f"--{name}" for name in missing))
    problems = r1_product_problems(arguments.repository_root, {product: getattr(arguments, product.lower()) for product in products})
    if problems:
        print("r1 产物不是读法 2 登记的那几份（作废 Z5）：" + "，".join(problems), file=sys.stderr)
        return 2
    try:
        classes = load_cell_classes()
    except ValueError as error:
        print(str(error), file=sys.stderr)
        return 2
    pairs = []
    for product in products:
        with open(getattr(arguments, product.lower()), encoding="utf-8") as r1, open(getattr(arguments, "t" + product[1]), encoding="utf-8") as today:
            pairs.append((product, r1.read(), today.read()))
    comparison = compare_segment(pairs, classes, arguments.segment)
    if comparison.unclassified:
        print("r1 有分类表没归类的（行种, 字段），表不全、不比：" + "，".join(comparison.unclassified), file=sys.stderr)
        return 2
    lines = ResultLines(arguments.output)
    lines.result(f"name=config script=contrast_against_r1 segment={arguments.segment} classes_sha256={CLASSES_SHA256} "
                 + " ".join(f"{product.lower()}={field(os.path.basename(getattr(arguments, product.lower())))}" for product in products))
    write_comparison_lines(lines, comparison, arguments.segment)
    problems = self_check_problems(comparison)
    lines.result(f"name=contrast_self_check segment={arguments.segment} problems={len(problems)} detail={field(','.join(problems) or '-')}")
    lines.finish()
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
