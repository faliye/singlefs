"""E201（三oracle合谋今树整表） 归因（跑前登记 research/prompts/e201-preregistration.md 第一节读法 8、修订 1 第 1.3 条；第六节 Q1-归因、Q1-宽归因；第十节 F6）。

判定格变了的每一格（contrast_against_r1.py 产物里 `name=contrast_cell` 那几行），逐条按七种查，查中的全列；一种都不中、也没有查不了的记「归不到」，
一种都不中而有查不了的记「查不了」（F6，不算进归得到也不算进归不到）：
  G1 名单改了：那一行在 {33, 107, 236}（撤）或 {70, 141, 168}（改引）里。
  G2 坏实现的行文变了：T2 那一行的 mutation_line_sha256 与 P2 那一行的不同。
  G3 补位不同：P2 与 T2 那一行的 substituted_from、mutated_row_name 或 status 有一样不同。
  G4 落点所在的函数变了：今天锚点所在的那个 fn 单元，在参照树（提交 31e47115…）同文件、同限定名的单元指纹不同或不在；落点文件在 P0 那几份路径里的另标 reference_approximate。
  G5 装置算这一格的函数变了：device_function_map.tsv 对应到的函数，r1 的装置（f8a9ddaab 删第 3478–3484 行，sha256 等于 67e7a549… 记 exact，否则 approximate_device）
     与今天的装置之间同名 fn 单元的指纹不同。
  G6 那一列 oracle 的入口函数变了（只对列的格；O3 由冻结行定，不归这一种）。
  G7 研究一侧算这一格的函数变了：research_function_map.tsv 对应到的函数，r1 那一版按 E171 研究一侧 revision_8_research_side_of_the_revision_6_run.json 的 sha256
     在 git 历史里找，找不到记 research_reference_missing（查不了）。
集合字段（行号集合）那一格的行取两边集合的对称差；没有行的格（单条的行种、计数）G1–G4 不适用。
「今天的树」取计划那一步拷进计划目录的那一份（修订 1 第 1.3 条）；归到的文件在计划记的改动清单里的另标 uncommitted_at_submission（描述）。
宽归因（描述，不进判定）：「归不到」的格另报坏实现所在文件、装置入口 main 的闭包、那一列 oracle 入口的闭包里指纹变了的单元数（lib_function_units 的 rust_closure；算不出写 -）。

只读、写一份新产物（已存在就拒）。合并那一步由 full_table_run.py 在进程里调 attribute_cells；单测见 test_attribution.py。
"""

from __future__ import annotations

import dataclasses
import functools
import hashlib
import importlib.util
import json
import os
import subprocess
import sys

import e171_research_side
from contrast_against_r1 import DECISION_CLASSES, PRESENCE_FIELD, parse_row_set, read_product_entries
from result_lines import ResultLines, field

HERE = os.path.dirname(os.path.abspath(__file__))
REFERENCE_COMMIT = "31e47115471ba5502a4531d25dffcc921be36899"
R1_DEVICE_COMMIT = "f8a9ddaab"
R1_DEVICE_DELETED_LINES = (3478, 3484)
R1_DEVICE_SHA256 = "67e7a5498425ec3f27379ba8f0e5cd666faaeaac0817006d6938956d6419a156"
P0_COMMIT_PATH = ("04ceaa904^", "research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-frozen-tree.out")
P0_SHA256 = "b720e5d28dcaead6e7c7262fd82185856e869385dad30674bf9689cb658f24c6"
RESEARCH_REFERENCE_JSON = os.path.join(e171_research_side.E171_DIRECTORY, "revision_8_research_side_of_the_revision_6_run.json")
DEVICE_MAP_PATH = os.path.join(HERE, "device_function_map.tsv")
RESEARCH_MAP_PATH = os.path.join(HERE, "research_function_map.tsv")
DEVICE_MAP_SHA256 = "95b93eb4bf7be327215bbfcdf80bdaf890d6a36e3db1ea9be2edcc955cbce0b9"
RESEARCH_MAP_SHA256 = "9761c84f03d4295f28cf157279bf9e09010e0f0aee84049141d8e8f017d56a6a"
FUNCTION_UNITS_LIBRARY = os.path.join(e171_research_side.REPOSITORY_ROOT, "scripts", "gate", "lib_function_units.py")
WITHDRAWN_ROWS = frozenset({33, 107, 236})
REFERENCE_CHANGED_ROWS = frozenset({70, 141, 168})
TODAY_FILE_OF_A_RESEARCH_FILE = {"row_driver.py": os.path.relpath(os.path.join(HERE, "current_tree_row_driver.py"), e171_research_side.REPOSITORY_ROOT)}
NOT_APPLICABLE = "-"
# G6：每一列 oracle 的入口（crate 目录或文件, 函数名或前缀）。answer_ 那一项只取这一行两条流里出现过的调用名接上的那几个（answer_<调用名>）。
COLUMN_COMPARISON, COLUMN_CRASH_JUDGE, COLUMN_REFERENCE_FILESYSTEM, COLUMN_POOL_CHECKER = "O1-对拍", "O1-模型崩溃判官", "O1-RefFS", "O2"
COLUMN_O3_STRICT, COLUMN_O3_PROVISIONAL = "O3甲", "O3乙"
ORACLE_ENTRIES = {
    COLUMN_COMPARISON: (("crates/singlefs-verification/src/model.rs", ("judge_and_advance",)),
                        ("crates/singlefs-verification/src/model_comparison.rs", ("*observation*",))),
    COLUMN_CRASH_JUDGE: (("crates/singlefs-verification/src/model.rs", ("crash_recovery_disagreement", "committed_versions")),),
    COLUMN_REFERENCE_FILESYSTEM: (("crates/singlefs-reference-filesystem/src/lib.rs", ("submit", "allowed_set_for_a_crash_in", "crash_and_recover")),
                                  ("crates/singlefs-verification/src/reference_filesystem_translation.rs", ("events_of", "crash_moment_of_a_segment_closed_at"))),
    COLUMN_POOL_CHECKER: (("crates/singlefs-checker/src/", ("check_pool_image",)),),
}
ANSWER_PREFIX = "answer_"
MODEL_SOURCE = "crates/singlefs-verification/src/model.rs"
STATUS_ATTRIBUTED, STATUS_UNATTRIBUTABLE, STATUS_UNCHECKABLE = "attributed", "unattributable", "uncheckable"


@functools.lru_cache(maxsize=1)
def function_units():
    """scripts/gate/lib_function_units.py（按路径载入：它是门禁的库，不在 import 路径上）。"""
    spec = importlib.util.spec_from_file_location("lib_function_units_for_e201", FUNCTION_UNITS_LIBRARY)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


@dataclasses.dataclass(frozen=True)
class FunctionMap:
    """G5、G7 的对应表：组与（产物, 行种, 字段）→ 组名或函数。lookup 交回展开后的函数（`-` 交空元组 = 不适用；没登记交 None）。"""

    groups: dict[str, tuple[str, ...]]
    cells: dict[tuple[str, str, str], tuple[str, ...]]

    def lookup(self, product: str, kind: str, name: str) -> tuple[str, ...] | None:
        entries = self.cells.get((product, kind, name), self.cells.get((product, kind, "*")))
        if entries is None:
            return None
        expanded: list[str] = []
        for entry in entries:
            if entry == NOT_APPLICABLE:
                continue
            expanded.extend(self.groups.get(entry, (entry,)))
        return tuple(dict.fromkeys(expanded))


def read_function_map(text: str) -> FunctionMap:
    groups: dict[str, tuple[str, ...]] = {}
    cells: dict[tuple[str, str, str], tuple[str, ...]] = {}
    for number, line in enumerate(text.split("\n"), start=1):
        if not line or line.startswith("#"):
            continue
        columns = line.split("\t")
        if columns[0] == "group" and len(columns) == 3:
            groups[columns[1]] = tuple(columns[2].split(","))
        elif columns[0] == "cell" and len(columns) == 5:
            cells[(columns[1], columns[2], columns[3])] = tuple(columns[4].split(","))
        else:
            raise ValueError(f"对应表第 {number} 行认不得：{line!r}")
    return FunctionMap(groups, cells)


def load_function_map(path: str, expected_sha256: str) -> FunctionMap:
    with open(path, "rb") as handle:
        content = handle.read()
    actual = hashlib.sha256(content).hexdigest()
    if actual != expected_sha256:
        raise ValueError(f"对应表 {path} 的 sha256 是 {actual}，登记记的是 {expected_sha256}：表在登记之后被改过，不归因")
    return read_function_map(content.decode("utf-8"))


def crate_directory_of(relative_path: str) -> str:
    parts = relative_path.split("/")
    return "/".join(parts[:2]) if len(parts) > 2 and parts[0] == "crates" else os.path.dirname(relative_path)


def rust_units_of_text(root: str, relative_path: str, text: str) -> list:
    """一份 Rust 源文件切出来的 fn 单元（lib_function_units 的 RustFile；切不开交空表，调用方当「不在」）。"""
    library = function_units()
    rust_file = library.RustFile(root, relative_path, text, crate_directory_of(relative_path))
    return [unit for unit in rust_file.units if unit.kind == "fn"]


def last_segment(name: str) -> str:
    return name.split("::")[-1]


def name_matches(name: str, pattern: str) -> bool:
    """函数名的最后一段对得上：`*字*` 是含这个词，`前缀*` 是以它起头，别的逐字相同。"""
    short = last_segment(name)
    if pattern.startswith("*") and pattern.endswith("*"):
        return pattern.strip("*") in short
    if pattern.endswith("*"):
        return short.startswith(pattern[:-1])
    return short == pattern


def fingerprints_of_named(units: list, patterns: tuple[str, ...]) -> tuple[tuple[str, str], ...]:
    """这几个名字（按最后一段认）在一份文件的 fn 单元里的（限定名, 指纹），排好序；同名的几个一并收。"""
    return tuple(sorted((unit.name, unit.fingerprint) for unit in units if any(name_matches(unit.name, pattern) for pattern in patterns)))


def innermost_unit_at_line(units: list, line: int):
    """包住这一行的最里层 fn 单元；没有交 None。"""
    covering = [unit for unit in units if unit.first_line <= line <= unit.last_line]
    return min(covering, key=lambda unit: unit.last_line - unit.first_line) if covering else None


def python_fingerprints(root: str, relative_path: str) -> dict[str, str] | None:
    """一份 Python 文件里各函数（与类的方法）单元：{名字: 指纹}；文件不在交 None。"""
    if not os.path.isfile(os.path.join(root, relative_path)):
        return None
    module = function_units().PythonIndex(root).module(relative_path)
    return {unit.name: unit.fingerprint for unit in module.units if unit.kind != "import"}


def git_output(repository: str, *arguments: str) -> bytes:
    return subprocess.run(["git", "-C", repository, *arguments], check=True, capture_output=True).stdout


def r1_device_text(repository: str, device_path: str = e171_research_side.DEVICE_SOURCE) -> tuple[str, str]:
    """读法 8 G5 的 r1 装置：`git show f8a9ddaab:<装置>` 删第 3478–3484 行。交回 (全文, exact 或 approximate_device)。"""
    lines = git_output(repository, "show", f"{R1_DEVICE_COMMIT}:{device_path}").decode("utf-8").split("\n")
    first, last = R1_DEVICE_DELETED_LINES
    text = "\n".join(lines[:first - 1] + lines[last:])
    marker = "exact" if hashlib.sha256(text.encode("utf-8")).hexdigest() == R1_DEVICE_SHA256 else "approximate_device"
    return text, marker


def p0_paths(repository: str) -> frozenset[str]:
    """P0：只取它记的 git status 那几行（`name=frozen_tree key=git_status line=<两字状态>_<路径>`）的路径；sha256 对不上抛错（作废 Z5）。"""
    content = git_output(repository, "show", f"{P0_COMMIT_PATH[0]}:{P0_COMMIT_PATH[1]}")
    if hashlib.sha256(content).hexdigest() != P0_SHA256:
        raise ValueError("P0 的 sha256 与读法 2 记的不同（作废 Z5）")
    return paths_of_status_lines(content.decode("utf-8"))


def paths_of_status_lines(text: str) -> frozenset[str]:
    paths = set()
    for kind, fields in read_product_entries(text):
        if kind == "frozen_tree" and fields.get("key") == "git_status" and "line" in fields:
            paths.add(fields["line"][3:])
    return frozenset(paths)


def research_reference_text(repository: str, relative_path: str, expected_sha256: str) -> str | None:
    """r1 那一版研究一侧文件：在 git 历史里逐个改过它的提交找 sha256 等于登记那一版的内容；找不到交 None。"""
    commits = git_output(repository, "log", "--all", "--format=%H", "--", relative_path).decode("utf-8").split()
    for commit in commits:
        try:
            content = git_output(repository, "show", f"{commit}:{relative_path}")
        except subprocess.CalledProcessError:
            continue
        if hashlib.sha256(content).hexdigest() == expected_sha256:
            return content.decode("utf-8")
    return None


def research_reference_sha256s(path: str = RESEARCH_REFERENCE_JSON) -> dict[str, str]:
    """revision_8_research_side_of_the_revision_6_run.json 的 files：{文件名（不带目录）: sha256}。"""
    with open(path, encoding="utf-8") as handle:
        files = json.load(handle)["files"]
    return {os.path.basename(relative): digest for relative, digest in files.items()}


def python_fingerprints_of_text(scratch_root: str, name: str, text: str) -> dict[str, str]:
    """一段 Python 原文的函数单元指纹（写进草稿根下一份同名文件再切：lib_function_units 按文件读）。"""
    os.makedirs(scratch_root, exist_ok=True)
    path = os.path.join(scratch_root, name)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)
    fingerprints = python_fingerprints(scratch_root, name)
    assert fingerprints is not None
    return fingerprints


HIT, MISS, UNCHECKABLE, INAPPLICABLE = "hit", "miss", "uncheckable", "na"
KINDS_OF_ATTRIBUTION = ("g1", "g2", "g3", "g4", "g5", "g6", "g7")


@dataclasses.dataclass
class AttributionInputs:
    """归因要的全部输入（全量在合并那一步现取，单测手造）。"""

    reference_root: str
    today_root: str
    today_research_root: str
    r1_device: str
    device_marker: str
    today_device_path: str
    p0_paths: frozenset[str]
    uncommitted_paths: frozenset[str]
    p2_rows: dict[int, dict[str, str]]
    t2_rows: dict[int, dict[str, str]]
    today_mutations: dict[str, object]
    call_labels: dict[int, frozenset[str]]
    device_map: FunctionMap
    research_map: FunctionMap
    research_reference: dict[str, str | None]
    scratch_root: str


@dataclasses.dataclass(frozen=True)
class CellAttribution:
    """一格的七种结局（hit / miss / uncheckable / na）、合起来的结局与附注（reference_approximate、uncommitted_at_submission、approximate_device……）。"""

    outcomes: dict[str, str]
    status: str
    notes: tuple[str, ...]


def key_fields_of(key: str) -> dict[str, str]:
    return dict(part.split(":", 1) for part in key.split(",") if ":" in part) if key != "-" else {}


def rows_of_cell(cell: dict[str, str]) -> tuple[int, ...]:
    """这一格涉及哪几行：键里的 row / row_number；行号集合字段取两边的对称差；别的没有行。"""
    keys = key_fields_of(cell["key"])
    for name in ("row", "row_number"):
        if keys.get(name, "").isdigit():
            return (int(keys[name]),)
    if cell["class"] == "decision_set":
        difference = parse_row_set(cell["r1"]) ^ parse_row_set(cell["today"])
        return tuple(sorted(int(value) for value in difference if value.isdigit()))
    return ()


COMPARISON_FIELDS = ("O1-对拍", "o1_comparison_red_calls", "o1_red", "o1_comparison", "rows_o1_red", "rows_o1_not_red", "o1_red_rows", "q2_o1_red_rows")
STRICT_COLUMNS = (COLUMN_COMPARISON, COLUMN_CRASH_JUDGE, COLUMN_REFERENCE_FILESYSTEM, COLUMN_POOL_CHECKER, COLUMN_O3_STRICT)
PROVISIONAL_COLUMNS = (COLUMN_COMPARISON, COLUMN_CRASH_JUDGE, COLUMN_REFERENCE_FILESYSTEM, COLUMN_POOL_CHECKER, COLUMN_O3_PROVISIONAL)


def columns_of_cell(cell: dict[str, str]) -> tuple[str, ...]:
    """这一格是哪一列（几列）的格（G6 只对列的格）；不是列的格交空元组。"""
    name, kind = cell["field"], cell["kind"]
    if name in COMPARISON_FIELDS:
        return (COLUMN_COMPARISON,)
    for column in (COLUMN_CRASH_JUDGE, COLUMN_REFERENCE_FILESYSTEM, COLUMN_O3_STRICT, COLUMN_O3_PROVISIONAL):
        if name in (column, f"{column}_red"):
            return (column,)
    if name in ("O2", "O2_red"):
        return (COLUMN_POOL_CHECKER,)
    if (kind == "q1" and name in ("all_quiet_rows", "verdict")) or (kind == "verdict" and name == "q1"):
        return PROVISIONAL_COLUMNS if key_fields_of(cell["key"]).get("reading") == "provisional" else STRICT_COLUMNS
    if name == "untouched_rows_with_a_red_column":
        return STRICT_COLUMNS[:4]
    return ()


class Attributor:
    """按输入逐格归因；装置与各文件的单元切一次、缓存。"""

    def __init__(self, inputs: AttributionInputs):
        self.inputs = inputs
        self.rust_cache: dict[tuple[str, str], list | None] = {}

    def rust_units(self, root: str, relative_path: str) -> list | None:
        cache_key = (root, relative_path)
        if cache_key not in self.rust_cache:
            path = os.path.join(root, relative_path)
            if os.path.isfile(path):
                with open(path, encoding="utf-8") as handle:
                    self.rust_cache[cache_key] = rust_units_of_text(root, relative_path, handle.read())
            else:
                self.rust_cache[cache_key] = None
        return self.rust_cache[cache_key]

    def device_units(self, which: str) -> list:
        cache_key = ("<device>", which)
        if cache_key not in self.rust_cache:
            if which == "r1":
                self.rust_cache[cache_key] = rust_units_of_text(self.inputs.reference_root, self.inputs.today_device_path, self.inputs.r1_device)
            else:
                self.rust_cache[cache_key] = self.rust_units(self.inputs.today_root, self.inputs.today_device_path) or []
        return self.rust_cache[cache_key] or []

    def g1(self, rows: tuple[int, ...]) -> str:
        if not rows:
            return INAPPLICABLE
        return HIT if any(row in WITHDRAWN_ROWS or row in REFERENCE_CHANGED_ROWS for row in rows) else MISS

    def g2(self, rows: tuple[int, ...]) -> str:
        if not rows:
            return INAPPLICABLE
        return HIT if any(self.inputs.p2_rows.get(row, {}).get("mutation_line_sha256") != self.inputs.t2_rows.get(row, {}).get("mutation_line_sha256")
                          for row in rows) else MISS

    def g3(self, rows: tuple[int, ...]) -> str:
        if not rows:
            return INAPPLICABLE
        names = ("substituted_from", "mutated_row_name", "status")
        return HIT if any(self.inputs.p2_rows.get(row, {}).get(name) != self.inputs.t2_rows.get(row, {}).get(name)
                          for row in rows for name in names) else MISS

    def g4(self, rows: tuple[int, ...], notes: list[str]) -> str:
        """今天锚点所在的 fn 单元在参照树里同文件、同限定名的单元指纹不同或不在。行没有改坏用的变异（找不到、补不上）记不适用；今天切不出单元记查不了。"""
        outcomes = []
        for row in rows:
            mutation = self.inputs.today_mutations.get(self.inputs.t2_rows.get(row, {}).get("mutated_row_name", "-"))
            if mutation is None:
                outcomes.append(INAPPLICABLE)
                continue
            today_units = self.rust_units(self.inputs.today_root, mutation.file)
            with open(os.path.join(self.inputs.today_root, mutation.file), encoding="utf-8") as handle:
                text = handle.read()
            position = text.find(mutation.original_text())
            unit = innermost_unit_at_line(today_units or [], text[:position].count("\n") + 1) if position >= 0 else None
            if unit is None:
                outcomes.append(UNCHECKABLE)
                continue
            if mutation.file in self.inputs.p0_paths:
                notes.append(f"reference_approximate:{mutation.file}")
            if mutation.file in self.inputs.uncommitted_paths:
                notes.append(f"uncommitted_at_submission:{mutation.file}")
            reference_units = self.rust_units(self.inputs.reference_root, mutation.file) or []
            reference = [candidate.fingerprint for candidate in reference_units if candidate.name == unit.name]
            outcomes.append(HIT if reference != [unit.fingerprint] else MISS)
        return combine(outcomes)

    def g5(self, cell: dict[str, str], notes: list[str]) -> str:
        functions = self.inputs.device_map.lookup(cell["product"], cell["kind"], cell["field"])
        if functions is None:
            return UNCHECKABLE
        if not functions:
            return INAPPLICABLE
        if self.inputs.device_marker != "exact":
            notes.append(self.inputs.device_marker)
        before = fingerprints_of_named(self.device_units("r1"), functions)
        after = fingerprints_of_named(self.device_units("today"), functions)
        return HIT if before != after else MISS

    def entry_fingerprints(self, root: str, place: str, patterns: tuple[str, ...]) -> tuple[tuple[str, str, str], ...] | None:
        """一个入口（文件，或以 / 收尾的目录下全部 .rs）里这几个名字的 fn 单元：(文件, 限定名, 指纹)。文件不在交 None。"""
        if place.endswith("/"):
            directory = os.path.join(root, place)
            if not os.path.isdir(directory):
                return None
            files = sorted(os.path.relpath(os.path.join(base, name), root) for base, _, names in os.walk(directory) for name in names if name.endswith(".rs"))
        else:
            files = [place]
        found = []
        for relative_path in files:
            units = self.rust_units(root, relative_path)
            if units is None:
                return None
            found.extend((relative_path, name, fingerprint) for name, fingerprint in fingerprints_of_named(units, patterns))
        return tuple(sorted(found))

    def g6(self, cell: dict[str, str], rows: tuple[int, ...], notes: list[str]) -> str:
        columns = [column for column in columns_of_cell(cell) if column in ORACLE_ENTRIES]
        if not columns:
            return INAPPLICABLE
        outcomes = []
        for column in columns:
            for place, patterns in ORACLE_ENTRIES[column]:
                if column == COLUMN_COMPARISON and place == MODEL_SOURCE:
                    labels = sorted(set().union(*(self.inputs.call_labels.get(row, frozenset()) for row in rows))) if rows else []
                    if not labels:
                        notes.append("answer_functions_not_taken")
                    patterns = patterns + tuple(ANSWER_PREFIX + label for label in labels)
                before = self.entry_fingerprints(self.inputs.reference_root, place, patterns)
                after = self.entry_fingerprints(self.inputs.today_root, place, patterns)
                outcomes.append(HIT if before != after else MISS)
        return combine(outcomes)

    def g7(self, cell: dict[str, str], notes: list[str]) -> str:
        functions = self.inputs.research_map.lookup(cell["product"], cell["kind"], cell["field"])
        if functions is None:
            return UNCHECKABLE
        if not functions:
            return INAPPLICABLE
        outcomes = []
        by_file: dict[str, list[str]] = {}
        for entry in functions:
            file_name, _, function = entry.partition("::")
            by_file.setdefault(file_name, []).append(function)
        for file_name, names in sorted(by_file.items()):
            reference_text = self.inputs.research_reference.get(file_name)
            if reference_text is None:
                notes.append(f"research_reference_missing:{file_name}")
                outcomes.append(UNCHECKABLE)
                continue
            today_relative = TODAY_FILE_OF_A_RESEARCH_FILE.get(file_name, os.path.join(e171_research_side.E171_RELATIVE, file_name))
            with open(os.path.join(self.inputs.today_research_root, today_relative), encoding="utf-8") as handle:
                today_text = handle.read()
            before = python_fingerprints_of_text(os.path.join(self.inputs.scratch_root, "reference"), file_name, reference_text)
            after = python_fingerprints_of_text(os.path.join(self.inputs.scratch_root, "today"), file_name, today_text)
            outcomes.append(HIT if any(before.get(name) != after.get(name) for name in names) else MISS)
        return combine(outcomes)

    def attribute(self, cell: dict[str, str]) -> CellAttribution:
        rows = rows_of_cell(cell)
        notes: list[str] = []
        outcomes = {"g1": self.g1(rows), "g2": self.g2(rows), "g3": self.g3(rows), "g4": self.g4(rows, notes) if rows else INAPPLICABLE,
                    "g5": self.g5(cell, notes), "g6": self.g6(cell, rows, notes), "g7": self.g7(cell, notes)}
        if HIT in outcomes.values():
            status = STATUS_ATTRIBUTED
        elif UNCHECKABLE in outcomes.values():
            status = STATUS_UNCHECKABLE
        else:
            status = STATUS_UNATTRIBUTABLE
        return CellAttribution(outcomes, status, tuple(dict.fromkeys(notes)))


def combine(outcomes: list[str]) -> str:
    """几处（几行、几个入口、几份文件）合成一种的结局：有一处中就中；没中而有查不了的记查不了；全不适用记不适用。"""
    if HIT in outcomes:
        return HIT
    if UNCHECKABLE in outcomes:
        return UNCHECKABLE
    if outcomes and all(outcome == INAPPLICABLE for outcome in outcomes):
        return INAPPLICABLE
    return MISS if outcomes else INAPPLICABLE


def changed_cells_of(contrast_text: str) -> list[dict[str, str]]:
    """contrast_against_r1.py 产物里判定格变了的那几格（`name=contrast_cell`）。"""
    return [fields for kind, fields in read_product_entries(contrast_text) if kind == "contrast_cell" and fields.get("class") in DECISION_CLASSES]


def closure_members(root: str, entry: str) -> dict[tuple[str, str], str] | None:
    """lib_function_units 的 rust_closure：入口闭包里各单元 {(文件, 种类与限定名): 指纹}；算不出（入口不在、拒算、切不开）交 None。"""
    try:
        result = function_units().rust_closure(root, entry)
    except Exception:  # 宽归因只描述：哪一种失败都只记「算不出」，不让归因整趟停
        return None
    members = {}
    for line in result.members:
        kind, name, place, fingerprint = line.split("\t")
        members[(place.split(":")[0], f"{kind} {name}")] = fingerprint
    return members


def changed_unit_count(before: dict | None, after: dict | None) -> str:
    if before is None or after is None:
        return NOT_APPLICABLE
    return str(sum(1 for key, fingerprint in after.items() if before.get(key) != fingerprint))


def wide_attribution(attributor: Attributor, cell: dict[str, str]) -> dict[str, str]:
    """宽归因（描述）：坏实现所在文件里指纹变了的 fn 单元数、装置入口 main 的闭包、这一格那几列 oracle 入口的闭包里指纹变了的单元数。"""
    inputs = attributor.inputs
    counts = {"mutated_file_changed_units": NOT_APPLICABLE, "device_main_closure_changed_units": NOT_APPLICABLE,
              "oracle_closure_changed_units": NOT_APPLICABLE}
    rows = rows_of_cell(cell)
    files = sorted({mutation.file for row in rows
                    for mutation in [inputs.today_mutations.get(inputs.t2_rows.get(row, {}).get("mutated_row_name", "-"))] if mutation is not None})
    if files:
        total = 0
        for relative_path in files:
            before = {unit.name: unit.fingerprint for unit in attributor.rust_units(inputs.reference_root, relative_path) or []}
            total += sum(1 for unit in attributor.rust_units(inputs.today_root, relative_path) or [] if before.get(unit.name) != unit.fingerprint)
        counts["mutated_file_changed_units"] = str(total)
    entry = f"{inputs.today_device_path}::main"
    counts["device_main_closure_changed_units"] = changed_unit_count(closure_members(inputs.reference_root, entry), closure_members(inputs.today_root, entry))
    oracle_counts = []
    for column in columns_of_cell(cell):
        for place, patterns in ORACLE_ENTRIES.get(column, ()):
            if place.endswith("/") or any("*" in pattern for pattern in patterns):
                continue
            for pattern in patterns:
                today_units = [unit for unit in attributor.rust_units(inputs.today_root, place) or [] if last_segment(unit.name) == pattern]
                for unit in today_units:
                    entry = f"{place}::{unit.name}"
                    oracle_counts.append(changed_unit_count(closure_members(inputs.reference_root, entry), closure_members(inputs.today_root, entry)))
    if oracle_counts:
        counts["oracle_closure_changed_units"] = ",".join(oracle_counts)
    return counts


def q1_verdict(changed: int, unattributable: int, uncheckable: int) -> str:
    """第六节 Q1-判定格差与 Q1-归因：c = 0 没有一格变；u ≥ 1 有的格归不到；u = 0 而 q ≥ 1 不够判（F6）；c ≥ 1 且 u = q = 0 每一格都归得到。"""
    if changed == 0:
        return "no_cell_changed"
    if unattributable:
        return "some_cells_unattributable"
    if uncheckable:
        return "not_enough_f6"
    return "every_cell_attributed"


def attribute_cells(lines: ResultLines, contrast_text: str, inputs: AttributionInputs, segment: str, head: str) -> str:
    """逐格归因、宽归因、按批合计；交回批 1 那一份的判定（Q1）。"""
    attributor = Attributor(inputs)
    lines.result(f"name=attribution_context segment={segment} head={field(head)} uncommitted_paths={len(inputs.uncommitted_paths)} "
                 f"reference_commit={REFERENCE_COMMIT} device={inputs.device_marker} p0_paths={len(inputs.p0_paths)} "
                 f"research_reference_missing={field(','.join(sorted(name for name, text in inputs.research_reference.items() if text is None)) or '-')}")
    tallies = {"1": [0, 0, 0], "2": [0, 0, 0]}
    for cell in changed_cells_of(contrast_text):
        result = attributor.attribute(cell)
        batch = "2" if cell.get("batch") == "2" else "1"
        tallies[batch][0] += 1
        tallies[batch][1] += result.status == STATUS_UNATTRIBUTABLE
        tallies[batch][2] += result.status == STATUS_UNCHECKABLE
        lines.result(f"name=attribution_cell segment={segment} product={cell['product']} kind={cell['kind']} key={field(cell['key'])} field={field(cell['field'])} "
                     f"batch={batch} status={result.status} " + " ".join(f"{name}={result.outcomes[name]}" for name in KINDS_OF_ATTRIBUTION)
                     + f" notes={field(','.join(result.notes) or '-')}")
        if result.status == STATUS_UNATTRIBUTABLE:
            counts = wide_attribution(attributor, cell)
            lines.result(f"name=wide_attribution segment={segment} product={cell['product']} kind={cell['kind']} key={field(cell['key'])} field={field(cell['field'])} "
                         + " ".join(f"{name}={field(value)}" for name, value in counts.items()))
    decisive = ""
    for batch, (changed, unattributable, uncheckable) in sorted(tallies.items()):
        verdict = q1_verdict(changed, unattributable, uncheckable)
        lines.result(f"name=attribution_total segment={segment} batch={batch} c={changed} u={unattributable} q={uncheckable} verdict={verdict}")
        if batch == "1":
            decisive = verdict
    return decisive


def extract_reference_tree(repository: str, destination: str) -> None:
    """参照树：`git archive 31e47115… crates Cargo.toml Cargo.lock` 落到 destination（新目录）。"""
    os.makedirs(destination, exist_ok=False)
    archive = subprocess.run(["git", "-C", repository, "archive", REFERENCE_COMMIT, "crates", "Cargo.toml", "Cargo.lock"],
                             check=True, capture_output=True).stdout
    subprocess.run(["tar", "-x", "-C", destination], input=archive, check=True)


def driver_rows_of(text: str) -> dict[int, dict[str, str]]:
    return {int(fields["row_number"]): fields for kind, fields in read_product_entries(text) if kind == "row" and fields.get("row_number", "").isdigit()}


def call_labels_of_rows(rows_root: str, numbers) -> dict[int, frozenset[str]]:
    """今天每行流一、流二里出现过的调用名（G6 的 answer_<调用名>）：读行目录里导出的 calls 表；没有导出的行不收。"""
    from run_directory import TOUCH_STREAM_NAMES, load_run_directory
    labels = {}
    for number in numbers:
        dump = os.path.join(rows_root, f"row-{number}", "dump")
        if not os.path.isdir(dump):
            continue
        run = load_run_directory(dump)
        labels[number] = frozenset(call.label for name in TOUCH_STREAM_NAMES if run.streams[name].present() for call in run.streams[name].calls)
    return labels


def collect_inputs(repository: str, today_root: str, reference_root: str, p2_text: str, t2_text: str, rows_root: str,
                   uncommitted_paths: frozenset[str], scratch_root: str) -> AttributionInputs:
    """合并那一步现取归因的全部输入（两张对应表先核 sha256）。"""
    from bad_implementation_table import read_mutation_rows
    r1_device, marker = r1_device_text(repository)
    t2_rows = driver_rows_of(t2_text)
    references = {}
    for name, digest in research_reference_sha256s().items():
        references[name] = research_reference_text(repository, os.path.join(e171_research_side.E171_RELATIVE, name), digest)
    for line in read_function_map(open(RESEARCH_MAP_PATH, encoding="utf-8").read()).groups.values():
        for entry in line:
            references.setdefault(entry.partition("::")[0], None)
    return AttributionInputs(
        reference_root=reference_root, today_root=today_root, today_research_root=repository, r1_device=r1_device, device_marker=marker,
        today_device_path=e171_research_side.DEVICE_SOURCE, p0_paths=p0_paths(repository), uncommitted_paths=uncommitted_paths,
        p2_rows=driver_rows_of(p2_text), t2_rows=t2_rows,
        today_mutations={row.name: row for row in read_mutation_rows(os.path.join(today_root, "crates", "mutations.tsv"))},
        call_labels=call_labels_of_rows(rows_root, sorted(t2_rows)),
        device_map=load_function_map(DEVICE_MAP_PATH, DEVICE_MAP_SHA256), research_map=load_function_map(RESEARCH_MAP_PATH, RESEARCH_MAP_SHA256),
        research_reference=references, scratch_root=scratch_root)
