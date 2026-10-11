"""E171 坏实现表（登记 5.2、5.9）：从 `crates/mutations.tsv` 按名字取行、核锚点、施加替换、补位。

挑法本身（哪 117 行）在 `select_bad_implementations.py`（登记 5.2，sha256 4032a0e0…）与登记 5.9 的表里；这里的 `bad-implementations-117.tsv`
是那张表照名字抄出来的（# / 刀 / 文件 / 组内下标 / 变异名 / 行文 sha256 前 12 位），开工时表若已变照名字取行、不重挑（登记 5.2「行文变了」）。
"""

from __future__ import annotations

import hashlib
from dataclasses import dataclass

MUTATION_TABLE_COLUMNS = 6
OWN_INSTRUMENT_ROW_PREFIX = "E171 装置"
MAXIMUM_SUBSTITUTIONS_PER_ROW = 3
CORE_SOURCE_PREFIX = "crates/singlefs-core/src/"
REPLAY_PREFIX_SOURCE = "crates/singlefs-replay-prefix/src/lib.rs"

STATUS_LOCATED = "ok"
STATUS_NOT_FOUND = "not_found"
STATUS_AMBIGUOUS_NAME = "ambiguous_name"
STATUS_ANCHOR_NOT_UNIQUE = "anchor_not_unique"


@dataclass(frozen=True)
class MutationRow:
    """`crates/mutations.tsv` 的一行：变异名、文件、原文、替换文、cargo test 的参数、必须红的测试名（原文与替换文里的 `\\n` 表示换行）。"""

    name: str
    file: str
    original: str
    replacement: str
    cargo_arguments: str
    test_name: str
    raw_line: str

    def line_digest_prefix(self) -> str:
        return line_digest_prefix(self.raw_line)

    def original_text(self) -> str:
        return self.original.replace("\\n", "\n")

    def replacement_text(self) -> str:
        return self.replacement.replace("\\n", "\n")


@dataclass(frozen=True)
class SelectedRow:
    """登记 5.9 表里的一行：序号、刀（1 第一刀、2 其余）、文件、组内下标、变异名、行文 sha256 前 12 位。"""

    number: int
    slice_number: int
    file: str
    group_index: int
    name: str
    line_digest_prefix: str


@dataclass(frozen=True)
class Located:
    status: str
    row: MutationRow | None
    line_matches_the_registration: bool


def line_digest_prefix(raw_line: str) -> str:
    return hashlib.sha256(raw_line.encode("utf-8")).hexdigest()[:12]


def is_implementation_file(path: str) -> bool:
    """挑法脚本的筛法（登记 5.2）：core 的源文件、或 replay-prefix 的 lib.rs；singlefs-format 与验证一侧的行不算坏实现。"""
    return path.startswith(CORE_SOURCE_PREFIX) or path == REPLAY_PREFIX_SOURCE


def read_mutation_rows(path: str) -> list[MutationRow]:
    rows = []
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            raw_line = line.rstrip("\n")
            if raw_line.startswith("#"):
                continue
            fields = raw_line.split("\t")
            if len(fields) != MUTATION_TABLE_COLUMNS:
                continue
            rows.append(MutationRow(*fields, raw_line=raw_line))
    return rows


def without_the_rows_of_the_device_itself(table_text: str) -> tuple[str, int]:
    """登记 5.2 的挑法与 A6、A7 数的是「改实现」的行，也就是被测的坏实现。E171 装置自己的变异行（名字以 `E171 装置` 起头，追加在 `crates/mutations.tsv` 末尾）
    是本实验的测量工具、不是被测对象：改装置的行不在 core 下，为证红而改核心的一行（K4-c 造法）在 core 下，它一加，挑法对那个文件的一组就多一行、
    整组的下标都挪，A6 与 A7 就对不上登记。交回去掉这些行之后的表文本与去掉的行数。"""
    kept = []
    removed = 0
    for line in table_text.split("\n"):
        if line.startswith(OWN_INSTRUMENT_ROW_PREFIX):
            removed += 1
        else:
            kept.append(line)
    return "\n".join(kept), removed


def read_selected_rows(path: str) -> list[SelectedRow]:
    rows = []
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            fields = line.rstrip("\n").split("\t")
            if len(fields) != 6:
                continue
            number, slice_number, file, group_index, name, digest_prefix = fields
            rows.append(SelectedRow(int(number), int(slice_number), file, int(group_index), name, digest_prefix))
    return rows


def locate_by_name(selected: SelectedRow, table: list[MutationRow]) -> Located:
    """照名字在今天的表里取这一行：找不到、名字不唯一都不跑（补位）；行文与登记 5.9 的 sha256 不同照今天的行跑、另记。"""
    hits = [row for row in table if row.name == selected.name]
    if not hits:
        return Located(STATUS_NOT_FOUND, None, False)
    if len(hits) > 1:
        return Located(STATUS_AMBIGUOUS_NAME, None, False)
    row = hits[0]
    return Located(STATUS_LOCATED, row, row.line_digest_prefix() == selected.line_digest_prefix)


def anchor_hit_count(source_text: str, row: MutationRow) -> int:
    return source_text.count(row.original_text())


def apply_mutation(source_text: str, row: MutationRow) -> str:
    """原文在文件里必须恰好命中一次（`crates/mutations.tsv` 表头），不是就拒绝，不静默失效。"""
    hits = anchor_hit_count(source_text, row)
    if hits != 1:
        raise ValueError(f"锚点在 {row.file} 里命中 {hits} 次（要恰好 1 次）：{row.name}")
    return source_text.replace(row.original_text(), row.replacement_text(), 1)


def empty_mutation(row: MutationRow) -> MutationRow:
    """空变异（登记 5.6 阴性）：拿表里第一行、把替换文换成原文本身，走完整条逐行驱动。"""
    return MutationRow(
        name=f"空变异（{row.name}）",
        file=row.file,
        original=row.original,
        replacement=row.original,
        cargo_arguments=row.cargo_arguments,
        test_name=row.test_name,
        raw_line=row.raw_line,
    )


def group_names_of_file(table: list[MutationRow], file: str) -> list[str]:
    """一个文件的组：表里文件列等于它的、算坏实现的行，保持表里的先后。E171 装置自己的变异行不在组里（与挑法同一个组，登记修订 3：
    为证红而改核心的那一行在 `transaction.rs` 下，留在组里补位就可能换到测量工具自己的行上）。"""
    if not is_implementation_file(file):
        return []
    return [row.name for row in table if row.file == file and not row.name.startswith(OWN_INSTRUMENT_ROW_PREFIX)]


def substitute_for(
    group_names: list[str],
    failed_name: str,
    names_in_the_table_of_rows: set[str],
    names_already_substituted: set[str],
) -> str | None:
    """补位（登记 5.2）：一行编不过、锚点不唯一、按名字找不到，换同一文件组内下标 j + 1（到组尾回到 0）、不在本表里的那一行。
    `names_in_the_table_of_rows` 是这一次跑的表与别的表（117 行与扩表）的变异名，`names_already_substituted` 是这一次跑里别的补位已经用掉的；都跳过。找不到返回 None。"""
    if failed_name not in group_names:
        return None
    count = len(group_names)
    start = group_names.index(failed_name)
    for step in range(1, count):
        candidate = group_names[(start + step) % count]
        if candidate in names_in_the_table_of_rows or candidate in names_already_substituted:
            continue
        return candidate
    return None
