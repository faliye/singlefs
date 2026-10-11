"""E170 第 2 次重跑第二段 Q5 那一半：Q5-d 与 Q4-fmt-487（`research/prompts/e170-r2-prereg.md` 12.7、12.9）。

Q5-d：v2-487 的 `on_disk_layout.py` 里 journal 记录头每个字段的偏移与宽、`JOURNAL_HEADER_BYTES`、新根段宽、每条点名项上限、点名项宽，
逐个对 12.9 列的实现那几处：`crates/singlefs-format/src/lib.rs` 的格式常量，`crates/singlefs-core/src/journal.rs` 里 `JournalRecord::to_bytes`
的写序（第 244–259 行一带）。写序按下面这张表逐句在实现源码里找（每句在函数体里恰好命中一行、行号递增），宽度取写法本身
（`put_u8` 1、`put_u16` 2、`put_u32` 4、`put_u64` 8、`put_six_byte_unsigned` 6、`skip(n)` n、常量宽度取 lib.rs 的值），偏移按写序累加；
新根段四个指针的宽度由实现自己那一句断言（段宽 = `JOURNAL_NEW_ROOT_SEGMENT_BYTES`）解出，再与 `NODE_POINTER_BYTES` 比。
Q4-fmt-487：v2-487 的 `JOURNAL_HEADER_BYTES`、D23 已定项 4 正文「头 **N 字节**」、实现的 `JOURNAL_HEADER_BYTES` 三个数并排；layout 第 357 行另列。
跑法：python3 comparator_q5_format_constants.py --executor-directory <v2-487 目录> --tree <快照根> --working-tree <仓根> --output <新文件>
文件名以 comparator 起头，不进冻结。
"""

from __future__ import annotations

import argparse
import hashlib
import os
import re
import sys

sys.dont_write_bytecode = True

LIBRARY_FILE = "crates/singlefs-format/src/lib.rs"
JOURNAL_FILE = "crates/singlefs-core/src/journal.rs"
DECISION_FILE = ".claude/kb/decisions/23-journal的角色与格式.md"
LAYOUT_FILE = ".claude/kb/layout/01-first-txn.md"
LAYOUT_LINE_WITH_THE_OLD_TOTAL = 357
FORMAT_CONSTANT_NAMES = ("JOURNAL_HEADER_TEN_FIELD_BYTES", "JOURNAL_NEW_ROOT_SEGMENT_BYTES", "JOURNAL_HEADER_BYTES", "JOURNAL_NAMED_ENTRY_BYTES",
                         "JOURNAL_NAMED_ENTRIES_PER_RECORD", "JOURNAL_RECORD_BYTES", "WIDE_CHECKSUM_BYTES", "NODE_POINTER_BYTES")
POINTER_WIDTH = "pointer"  # 由段宽断言解出

# (v2-487 的字段名, 实现 to_bytes 里那一句的开头, 宽度：整数 / lib.rs 常量名 / POINTER_WIDTH)
WRITE_ORDER = [
    ("magic", "writer.put(&JOURNAL_MAGIC);", 4),
    ("record_type", "writer.put_u16(JOURNAL_RECORD_TYPE_ORDINARY);", 2),
    ("algorithm_type", "writer.put_u8(0); // 算法类型", 1),
    ("record_flags", "writer.put_u8(self.place_in_publish.record_flags_byte());", 1),
    ("record_length", "writer.put_u32(u32::try_from(JOURNAL_RECORD_BYTES)", 4),
    ("named_item_count", "writer.put_u32(u32::try_from(self.named.len())", 4),
    ("jsn_instance", "writer.put_u32(self.instance.0);", 4),
    ("jsn_counter", "writer.put_six_byte_unsigned(self.counter);", 6),
    ("checkpoint_txg", "writer.put_u64(self.checkpoint_txg.0);", 8),
    ("nonce", "writer.skip(12); // nonce", 12),
    ("header_checksum", "writer.skip(usize::try_from(WIDE_CHECKSUM_BYTES)", "WIDE_CHECKSUM_BYTES"),
    ("transaction_number", "writer.put_u64(self.transaction);", 8),
    ("commit_marker", "writer.put_u8(u8::from(self.is_commit));", 1),
    ("ordinal_within_publish", "writer.put_u32(self.ordinal_within_publish.0);", 4),
    ("back_chain", "writer.put_u32(self.back_chain);", 4),
    ("payload_checksum", "writer.skip(4);", 4),
    ("new_root_tree_table_pointer", "self.new_tree_table.write_to(&mut writer);", POINTER_WIDTH),
    ("new_root_central_mapping_root_pointer", "self.new_mapping_root.write_to(&mut writer);", POINTER_WIDTH),
    ("new_root_tree_identifier_watermark", "writer.put_u64(self.new_tree_identifier_watermark);", 8),
    ("new_root_rollback_floor", "writer.put_u64(self.new_rollback_floor.0);", 8),
    ("new_root_instance_table_pointer", "self.new_instance_table.write_to(&mut writer);", POINTER_WIDTH),
    ("new_root_allocation_records_root_pointer", "self.new_allocation_record_tree_root.write_to(&mut writer);", POINTER_WIDTH),
    ("new_root_unmount_marker", "writer.put_u32(self.new_unmount_marker.flags_bits());", 4),
    ("file_system_identity_low_eight_bytes", "writer.put_u64(self.filesystem_identifier);", 8),
    ("message_authentication_code", "writer.skip(16); // MAC", 16),
]
NEW_ROOT_SEGMENT_FIELDS = [name for name, _, _ in WRITE_ORDER if name.startswith("new_root_")]
FUNCTION_START = "pub fn to_bytes(&self) -> Vec<u8> {"
FUNCTION_HEADER_END = 'writer.assert_position(JOURNAL_HEADER_BYTES, "记录头");'


def sha256_of(path: str) -> str:
    with open(path, "rb") as handle:
        return hashlib.sha256(handle.read()).hexdigest()


def library_constants(text: str) -> dict[str, tuple[int, int]]:
    """lib.rs 里 `pub const NAME: u64 = N;` 的值与行号；每个名字必须恰好一处。"""
    found: dict[str, tuple[int, int]] = {}
    for line_number, line in enumerate(text.split("\n"), start=1):
        match = re.match(r"pub const ([A-Z0-9_]+): u64 = (\d+);", line)
        if match and match.group(1) in FORMAT_CONSTANT_NAMES:
            assert match.group(1) not in found, f"{match.group(1)} 在 lib.rs 里不止一处"
            found[match.group(1)] = (int(match.group(2)), line_number)
    missing = set(FORMAT_CONSTANT_NAMES) - set(found)
    assert not missing, f"lib.rs 里没找到 {sorted(missing)}"
    return found


def write_order_in_the_implementation(text: str, constants: dict[str, tuple[int, int]]) -> list[tuple[str, int, int, int]]:
    """交回 [(字段, 实现那一句的行号, 偏移, 宽)]；每句在 to_bytes 的头部那一段里恰好命中一行、行号递增。"""
    lines = text.split("\n")
    starts = [index for index, line in enumerate(lines) if line.strip() == FUNCTION_START]
    assert len(starts) == 1, f"to_bytes 起头命中 {len(starts)} 次"
    end = next(index for index in range(starts[0], len(lines)) if lines[index].strip() == FUNCTION_HEADER_END)
    body = list(range(starts[0] + 1, end))
    resolved = []
    previous = starts[0]
    for field, statement, width in WRITE_ORDER:
        hits = [index for index in body if lines[index].strip().startswith(statement)]
        assert len(hits) == 1, f"{field}：「{statement}」在 to_bytes 里命中 {len(hits)} 次"
        assert hits[0] > previous, f"{field}：写序不递增"
        previous = hits[0]
        resolved.append((field, hits[0] + 1, width))
    fixed_segment = sum(width for field, _, width in resolved if field in NEW_ROOT_SEGMENT_FIELDS and width != POINTER_WIDTH)
    pointer_count = sum(1 for field, _, width in resolved if field in NEW_ROOT_SEGMENT_FIELDS and width == POINTER_WIDTH)
    pointer_width, remainder = divmod(constants["JOURNAL_NEW_ROOT_SEGMENT_BYTES"][0] - fixed_segment, pointer_count)
    assert remainder == 0, "新根段宽减去定宽字段之后不能被指针数整除"
    offsets = []
    running = 0
    for field, line_number, width in resolved:
        numeric = pointer_width if width == POINTER_WIDTH else constants[width][0] if isinstance(width, str) else width
        offsets.append((field, line_number, running, numeric))
        running += numeric
    return offsets


def decision_item_four_header_bytes(text: str) -> tuple[int, int]:
    """D23 已定项 4 那一节以「**定案**」起头那一行里的「头 **N 字节**」：交回 (N, 行号)；恰好一处。
    同一节里 `format-const` 注释行列着旧值（stale=…），不是正文，不取。"""
    lines = text.split("\n")
    start = next(index for index, line in enumerate(lines) if line.startswith("#### 已定项 4："))
    end = next(index for index in range(start + 1, len(lines)) if lines[index].startswith("#### "))
    hits = [(int(match.group(1)), index + 1) for index in range(start, end) if lines[index].startswith("**定案**")
            for match in re.finditer(r"头 \*\*(\d+) 字节\*\*", lines[index])]
    assert len(hits) == 1, f"已定项 4 定案句命中 {len(hits)} 处"
    return hits[0]


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E170 第 2 次重跑 Q5-d 与 Q4-fmt-487")
    parser.add_argument("--executor-directory", required=True)
    parser.add_argument("--tree", required=True, help="今天的树的快照根（Z 族从它编出来）")
    parser.add_argument("--working-tree", required=True, help="仓根（kb 与工作区的 crates/）")
    parser.add_argument("--output", required=True)
    arguments = parser.parse_args(argument_list)
    sys.path.insert(0, os.path.abspath(arguments.executor_directory))
    import on_disk_layout as layout
    output = open(arguments.output, "x", encoding="utf-8")
    emitted = 0

    def emit(line: str) -> None:
        nonlocal emitted
        emitted += 1
        output.write(line + "\n")

    for relative in (LIBRARY_FILE, JOURNAL_FILE):
        snapshot, working = sha256_of(os.path.join(arguments.tree, relative)), sha256_of(os.path.join(arguments.working_tree, relative))
        emit(f"name=implementation_source file={relative} snapshot_sha256={snapshot} working_tree_sha256={working} same={str(snapshot == working).lower()}")
    with open(os.path.join(arguments.tree, LIBRARY_FILE), encoding="utf-8") as handle:
        constants = library_constants(handle.read())
    for name in FORMAT_CONSTANT_NAMES:
        emit(f"name=implementation_constant constant={name} value={constants[name][0]} line={constants[name][1]} file={LIBRARY_FILE}")
    with open(os.path.join(arguments.tree, JOURNAL_FILE), encoding="utf-8") as handle:
        implementation = write_order_in_the_implementation(handle.read(), constants)
    differing = 0
    executor_fields = layout.JOURNAL_RECORD_FIELDS
    for field, line_number, offset, width in implementation:
        span = executor_fields.get(field)
        same = span is not None and (span.byte_offset, span.byte_width) == (offset, width)
        differing += 0 if same else 1
        executor_text = "-" if span is None else f"{span.byte_offset}+{span.byte_width}"
        emit(f"name=q5d_field field={field} implementation={offset}+{width} implementation_line={line_number} "
             f"executor={executor_text} same={str(same).lower()}")
    only_in_the_executor = sorted(set(executor_fields) - {field for field, _, _, _ in implementation})
    differing += len(only_in_the_executor)
    emit(f"name=q5d_fields_only_in_the_executor fields={','.join(only_in_the_executor) or '-'}")
    implementation_end = implementation[-1][2] + implementation[-1][3]
    new_root_start = next(offset for field, _, offset, _ in implementation if field == "new_root_tree_table_pointer")
    new_root_end = next(offset + width for field, _, offset, width in implementation if field == "new_root_unmount_marker")
    ten_field_end = next(offset + width for field, _, offset, width in implementation if field == "header_checksum")
    pairs = [
        ("journal_header_bytes", layout.JOURNAL_HEADER_BYTES, constants["JOURNAL_HEADER_BYTES"][0]),
        ("journal_header_bytes_by_write_order", layout.JOURNAL_HEADER_BYTES, implementation_end),
        ("new_root_segment_bytes", layout.JOURNAL_NEW_ROOT_SEGMENT_BYTES, constants["JOURNAL_NEW_ROOT_SEGMENT_BYTES"][0]),
        ("new_root_segment_bytes_by_write_order", layout.JOURNAL_NEW_ROOT_SEGMENT_BYTES, new_root_end - new_root_start),
        ("ten_field_bytes", executor_fields["header_checksum"].end_offset, constants["JOURNAL_HEADER_TEN_FIELD_BYTES"][0]),
        ("ten_field_bytes_by_write_order", executor_fields["header_checksum"].end_offset, ten_field_end),
        ("named_items_per_record", layout.NAMED_ITEMS_IN_A_4096_BYTE_RECORD_AT_MOST, constants["JOURNAL_NAMED_ENTRIES_PER_RECORD"][0]),
        ("named_item_bytes", layout.NAMED_ITEM_BYTES, constants["JOURNAL_NAMED_ENTRY_BYTES"][0]),
        ("new_root_pointer_bytes", executor_fields["new_root_tree_table_pointer"].byte_width, constants["NODE_POINTER_BYTES"][0]),
        ("header_checksum_bytes", executor_fields["header_checksum"].byte_width, constants["WIDE_CHECKSUM_BYTES"][0]),
    ]
    for item, executor_value, implementation_value in pairs:
        same = executor_value == implementation_value
        differing += 0 if same else 1
        emit(f"name=q5d_constant item={item} executor={executor_value} implementation={implementation_value} same={str(same).lower()}")
    emit(f"name=q5d_verdict differing={differing} compared={len(implementation) + 1 + len(pairs)} stop_s4={str(differing > 0).lower()}")
    with open(os.path.join(arguments.working_tree, DECISION_FILE), encoding="utf-8") as handle:
        decision_value, decision_line = decision_item_four_header_bytes(handle.read())
    with open(os.path.join(arguments.working_tree, LAYOUT_FILE), encoding="utf-8") as handle:
        layout_line_text = handle.read().split("\n")[LAYOUT_LINE_WITH_THE_OLD_TOTAL - 1]
    layout_match = re.search(r"合计 \*\*(\d+) 字节\*\*", layout_line_text)
    three = (layout.JOURNAL_HEADER_BYTES, decision_value, constants["JOURNAL_HEADER_BYTES"][0])
    emit(f"name=q4_fmt_487 executor_v2_487={three[0]} decision_23_item_4={three[1]} decision_line={decision_line} "
         f"implementation={three[2]} implementation_line={constants['JOURNAL_HEADER_BYTES'][1]} all_three_equal={str(len(set(three)) == 1).lower()} "
         f"layout_line_{LAYOUT_LINE_WITH_THE_OLD_TOTAL}={layout_match.group(1) if layout_match else 'not_found'}")
    emitted += 1
    output.write(f"name=done emitted={emitted}\n")
    output.close()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
