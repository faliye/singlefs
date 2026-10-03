"""盘上字段的偏移与宽度，只取自 kb 文字；每一组旁写出处（`文件@小节标题`，标题逐字）。

整数一律小端：.claude/kb/decisions/22-单元原子性怎么合成.md@已定项 13：盘上定长字段一律小端，枚举取值各有登记位，位图位序统一。
"""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class FieldSpan:
    byte_offset: int
    byte_width: int

    @property
    def end_offset(self) -> int:
        return self.byte_offset + self.byte_width


def spans_from_row_order(rows: list[tuple[str, int]]) -> dict[str, FieldSpan]:
    """行序即盘上顺序、偏移按行累加。
    依据（C2，欠一句表 G02 已由 kb 补）：.claude/kb/layout/01-first-txn.md 第 158 行（系统配置字段表）、第 216 行（位置条目）、
    第 359 行（journal 记录头）、第 365 行（根记录）四处都写明「行序即盘上顺序，偏移按行累加」；模块末尾对这些行列出的每一个偏移逐个断言。"""
    spans = {}
    running_offset = 0
    for field_name, byte_width in rows:
        assert field_name not in spans, f"字段名重复：{field_name}"
        spans[field_name] = FieldSpan(running_offset, byte_width)
        running_offset += byte_width
    return spans


def little_endian_unsigned(raw: bytes, span: FieldSpan) -> int:
    return int.from_bytes(raw[span.byte_offset:span.end_offset], "little")


# ---- 系统配置槽 ----
# .claude/kb/layout/01-first-txn.md@一、mkfs 已经种下的（新池新建文件之前就在盘上）——「系统配置字段表」46 行、489 字节；
# .claude/kb/decisions/22-单元原子性怎么合成.md@已定项 9：系统配置的字段表——字段表就是那一张，住 4096 字节的槽。
SYSTEM_CONFIGURATION_SLOT_BYTES = 4096
SYSTEM_CONFIGURATION_FIELD_ROWS = [
    ("magic", 4), ("format_version", 2), ("feature_bits", 96), ("file_system_identity", 16),
    ("writer_identity", 20), ("checksum_algorithm", 1), ("own_device_number", 4), ("device_count", 4),
    ("slot_generation", 8), ("whole_slot_checksum", 32),
    ("own_message_authentication_code", 16), ("nonce_watermark", 12), ("key_derivation_function", 4),
    ("encryption_type", 1), ("message_authentication_code_length", 1), ("master_key_slot", 80),
    ("node_size", 4), ("unit_size", 4), ("placement_granularity", 4), ("location_entry_width", 4),
    ("physical_block_size_at_format", 4), ("extension_point_count", 4),
    ("journal_ring_start_slot", 8), ("journal_ring_bytes", 8), ("journal_maximum_record_bytes", 4),
    ("journal_in_flight_record_limit", 4), ("journal_worst_case_bytes", 8), ("journal_safety_factor", 4),
    ("root_ring_region_count", 1), ("root_ring_slots_per_region", 1), ("root_ring_prime_step", 4),
    ("root_ring_chunk_bytes", 4), ("root_ring_base_slot", 8), ("root_ring_region_device_identities", 12),
    ("stripe_width_upper_bound", 1), ("group_size", 1), ("mapping_origin", 24), ("unit_area_start_slot", 8),
    ("minimum_io_size_at_format", 4), ("fixed_structure_slot_spacing", 4),
    ("checkpoint_time_threshold", 4), ("dirty_bytes_threshold", 8), ("defragmentation_watermarks", 24),
    ("journal_tail", 8), ("journal_instance", 4), ("rollback_floor", 8),
]
SYSTEM_CONFIGURATION_FIELDS = spans_from_row_order(SYSTEM_CONFIGURATION_FIELD_ROWS)
SYSTEM_CONFIGURATION_CONTENT_BYTES = 489

# ---- 根记录 ----
# .claude/kb/decisions/22-单元原子性怎么合成.md@已定项 7：根记录的字段表——「行序即盘上顺序，偏移按行累加」，偏移逐个写明。
ROOT_RECORD_FIELD_ROWS = [
    ("magic", 4), ("file_system_identity", 16), ("flags", 4), ("instance", 4), ("checkpoint_txg", 8),
    ("tree_table_pointer", 86), ("tree_identifier_watermark", 8), ("rollback_floor", 8),
    ("self_checksum", 32), ("instance_table_pointer", 86), ("central_mapping_root_pointer", 86),
    ("allocation_records_root_pointer", 86), ("algorithm_type", 1), ("nonce", 12), ("message_authentication_code", 16),
]
ROOT_RECORD_FIELDS = spans_from_row_order(ROOT_RECORD_FIELD_ROWS)
ROOT_RECORD_BYTES = 457
ROOT_FLAGS_UNMOUNT_MARKER_BIT = 0x1

# ---- journal 记录 ----
# .claude/kb/decisions/23-journal的角色与格式.md@已定项 4：记录头约束在单个原子单元内，头 311 字节——
# 十个字段按列举序（G02），其后「事务号 78、提交标记 86、本次发布内序号 87、反向链 91、载荷校验和 95、新根段 99、fsid 287、MAC 295、头末 311」。
JOURNAL_RECORD_FIELD_ROWS = [
    ("magic", 4), ("record_type", 2), ("algorithm_type", 1), ("record_flags", 1), ("record_length", 4),
    ("named_item_count", 4), ("jsn_instance", 4), ("jsn_counter", 6), ("checkpoint_txg", 8), ("nonce", 12),
    ("header_checksum", 32), ("transaction_number", 8), ("commit_marker", 1), ("ordinal_within_publish", 4),
    ("back_chain", 4), ("payload_checksum", 4),
    ("new_root_tree_table_pointer", 86), ("new_root_central_mapping_root_pointer", 86),
    ("new_root_tree_identifier_watermark", 8), ("new_root_rollback_floor", 8),
    ("file_system_identity_low_eight_bytes", 8), ("message_authentication_code", 16),
]
JOURNAL_RECORD_FIELDS = spans_from_row_order(JOURNAL_RECORD_FIELD_ROWS)
JOURNAL_HEADER_BYTES = 311
JOURNAL_RECORD_TYPE_INVALID = 0
JOURNAL_RECORD_TYPE_ORDINARY = 1
JOURNAL_RECORD_FLAG_LAST_OF_PUBLISH = 0x1

# 点名项：.claude/kb/decisions/23-journal的角色与格式.md@已定项 17：点名项 56 字节的字段表（偏移逐个写明）。
NAMED_ITEM_BYTES = 56
NAMED_ITEM_FIELDS = {
    "first_location_entry": FieldSpan(0, 14), "second_location_entry": FieldSpan(14, 14),
    "unit_type_tag": FieldSpan(28, 1), "birth_tree": FieldSpan(29, 8), "birth_txg": FieldSpan(37, 8),
    "key_tail": FieldSpan(45, 10), "flags": FieldSpan(55, 1),
}
# 位置条目：.claude/kb/decisions/19-块指针的结构与宽度预算.md@已定项 4：位置条目 14 字节（按列举序，G02）。
LOCATION_ENTRY_FIELDS = spans_from_row_order([("device_identity", 4), ("slot_number", 6), ("ciphertext_checksum", 4)])
PLACEMENT_SLOT_BYTES = 16384

# 单元大小按类型标签：.claude/kb/decisions/18-块里携带什么信息.md@已定项 11：单元类型标签的值域是一张登记表，打包记录单元是独立的一类。
UNIT_BYTES_BY_TYPE_TAG = {1: 32768, 2: 16384, 3: 32768}

# ---- 自证结构的校验和字段 ----
# .claude/kb/decisions/18-块里携带什么信息.md@已定项 17：32 字节头校验和字段放 CRC32C 4 字节 + 28 字节零；自证结构的校验和覆盖整个槽含补齐
SELF_CHECKSUM_FIELD_BYTES = 32
SELF_CHECKSUM_VALUE_BYTES = 4

# ---- magic（C1：欠一句表 G01 已由 kb 补）----
# 取值逐字取自 .claude/kb/layout/01-first-txn.md 的 magic 行（「4 字节 ASCII，从偏移 0 起按此顺序排，字节串，不涉字节序」）：
# 系统配置第 109 行 SFSB = 53 46 53 42；根记录第 369 行 SFSR = 53 46 53 52；journal 记录第 338 行 SFSJ = 53 46 53 4A。
# 单元头 SFSU = 53 46 53 55（同文件第 175 行）：步 7 验点名单元的条款（D23 已定项 14 第四条、D16 已定项 7）只写「两份读得出、校验和对」，
# 没有一句把 magic 列进这一步，所以步 7 不核 SFSU（记进 Q1-f，不新开欠一句）。
SYSTEM_CONFIGURATION_MAGIC = bytes.fromhex("53465342")
ROOT_RECORD_MAGIC = bytes.fromhex("53465352")
JOURNAL_RECORD_MAGIC = bytes.fromhex("5346534a")
MAGIC_BYTES = 4


def assert_kb_anchors_hold() -> None:
    """G02 的暂代读法与 kb 写明的每一个偏移逐个对；对不上说明行序读法错了，不往下跑。"""
    system_configuration = SYSTEM_CONFIGURATION_FIELDS
    assert system_configuration["journal_ring_start_slot"].byte_offset == 325  # D22 已定项 16 第 1 句「偏移 325」
    assert system_configuration["root_ring_base_slot"].byte_offset == 371  # 同一句「偏移 371」
    assert system_configuration["unit_area_start_slot"].byte_offset == 417  # I-7.13「单元区起始槽号（偏移 417）」
    assert system_configuration["rollback_floor"].byte_offset == 481  # D22 已定项 9「偏移 481 起」
    assert system_configuration["rollback_floor"].end_offset == SYSTEM_CONFIGURATION_CONTENT_BYTES
    assert len(SYSTEM_CONFIGURATION_FIELD_ROWS) == 46  # D22 已定项 9「46 行」
    root = ROOT_RECORD_FIELDS
    written_root_offsets = {"magic": 0, "file_system_identity": 4, "flags": 20, "instance": 24, "checkpoint_txg": 28,
                            "tree_table_pointer": 36, "tree_identifier_watermark": 122, "rollback_floor": 130,
                            "self_checksum": 138, "instance_table_pointer": 170, "central_mapping_root_pointer": 256,
                            "allocation_records_root_pointer": 342, "algorithm_type": 428, "nonce": 429,
                            "message_authentication_code": 441}
    for field_name, written_offset in written_root_offsets.items():
        assert root[field_name].byte_offset == written_offset, field_name
    assert root["message_authentication_code"].end_offset == ROOT_RECORD_BYTES
    journal = JOURNAL_RECORD_FIELDS
    assert journal["header_checksum"].byte_offset == 46  # D23 已定项 17「头部校验和 [46, 50)」
    written_journal_offsets = {"transaction_number": 78, "commit_marker": 86, "ordinal_within_publish": 87,
                               "back_chain": 91, "payload_checksum": 95, "new_root_tree_table_pointer": 99,
                               "file_system_identity_low_eight_bytes": 287, "message_authentication_code": 295}
    for field_name, written_offset in written_journal_offsets.items():
        assert journal[field_name].byte_offset == written_offset, field_name
    assert journal["message_authentication_code"].end_offset == JOURNAL_HEADER_BYTES
    assert journal["new_root_rollback_floor"].end_offset == 287  # 新根段 188 = 86 + 86 + 8 + 8，D23 已定项 15


# C2：layout 第 158、216、359 行（与第 365 行指到的 D22 已定项 7 那一句）列出的每一个偏移，载入时逐个断言。
# 下面的数逐个抄自那几行的原文，不由 FIELD_ROWS 现算；对不上说明行序读法或某一行的宽写错，不往下跑。
SYSTEM_CONFIGURATION_OFFSETS_LAYOUT_LINE_158 = {
    "magic": 0, "format_version": 4, "feature_bits": 6, "file_system_identity": 102, "writer_identity": 118,
    "checksum_algorithm": 138, "own_device_number": 139, "device_count": 143, "slot_generation": 147,
    "whole_slot_checksum": 155, "own_message_authentication_code": 187, "nonce_watermark": 203,
    "key_derivation_function": 215, "encryption_type": 219, "message_authentication_code_length": 220, "master_key_slot": 221,
    "node_size": 301, "unit_size": 305, "placement_granularity": 309, "location_entry_width": 313,
    "physical_block_size_at_format": 317, "extension_point_count": 321, "journal_ring_start_slot": 325,
    "journal_ring_bytes": 333, "journal_maximum_record_bytes": 341, "journal_in_flight_record_limit": 345,
    "journal_worst_case_bytes": 349, "journal_safety_factor": 357, "root_ring_region_count": 361,
    "root_ring_slots_per_region": 362, "root_ring_prime_step": 363, "root_ring_chunk_bytes": 367, "root_ring_base_slot": 371,
    "root_ring_region_device_identities": 379, "stripe_width_upper_bound": 391, "group_size": 392, "mapping_origin": 393,
    "unit_area_start_slot": 417, "minimum_io_size_at_format": 425, "fixed_structure_slot_spacing": 429,
    "checkpoint_time_threshold": 433, "dirty_bytes_threshold": 437, "defragmentation_watermarks": 445,
    "journal_tail": 469, "journal_instance": 477, "rollback_floor": 481,
}
SYSTEM_CONFIGURATION_DEFRAGMENTATION_WATERMARK_OFFSETS_LAYOUT_LINE_158 = (445, 453, 461)  # 低水位、高水位、停止线各 8 字节
SYSTEM_CONFIGURATION_REGION_DEVICE_IDENTITY_OFFSETS_LAYOUT_LINE_158 = (379, 383, 387)  # 每区 4 字节
JOURNAL_RECORD_OFFSETS_LAYOUT_LINE_359 = {
    "magic": 0, "record_type": 4, "algorithm_type": 6, "record_flags": 7, "record_length": 8, "named_item_count": 12,
    "jsn_instance": 16, "jsn_counter": 20, "checkpoint_txg": 26, "nonce": 34, "header_checksum": 46,
    "transaction_number": 78, "commit_marker": 86, "ordinal_within_publish": 87, "back_chain": 91, "payload_checksum": 95,
    "new_root_tree_table_pointer": 99, "new_root_central_mapping_root_pointer": 185,
    "new_root_tree_identifier_watermark": 271, "new_root_rollback_floor": 279,
    "file_system_identity_low_eight_bytes": 287, "message_authentication_code": 295,
}
ROOT_RECORD_OFFSETS_D22_ITEM_7_AS_REFERRED_BY_LAYOUT_LINE_365 = {
    "magic": 0, "file_system_identity": 4, "flags": 20, "instance": 24, "checkpoint_txg": 28, "tree_table_pointer": 36,
    "tree_identifier_watermark": 122, "rollback_floor": 130, "self_checksum": 138, "instance_table_pointer": 170,
    "central_mapping_root_pointer": 256, "allocation_records_root_pointer": 342, "algorithm_type": 428, "nonce": 429,
    "message_authentication_code": 441,
}
LOCATION_ENTRY_OFFSETS_LAYOUT_LINE_216 = {"device_identity": 0, "slot_number": 4, "ciphertext_checksum": 10}
LOCATION_ENTRY_START_IN_A_POINTER_LAYOUT_LINE_216 = (50, 64)  # 头部 50 + 14
LOCATION_ENTRY_START_IN_A_NAMED_ITEM_LAYOUT_LINE_216 = (0, 14)


def assert_every_offset_the_layout_lines_list_holds() -> None:
    for field_name, listed_offset in SYSTEM_CONFIGURATION_OFFSETS_LAYOUT_LINE_158.items():
        assert SYSTEM_CONFIGURATION_FIELDS[field_name].byte_offset == listed_offset, f"系统配置 {field_name}"
    assert set(SYSTEM_CONFIGURATION_OFFSETS_LAYOUT_LINE_158) == set(SYSTEM_CONFIGURATION_FIELDS), \
        "第 158 行列出的每个字段（整理三水位与逐区域设备身份在表里各是一个字段，下面另核它们的子偏移）与表里的字段一一对应"
    first_watermark = SYSTEM_CONFIGURATION_FIELDS["defragmentation_watermarks"]
    assert (first_watermark.byte_offset, first_watermark.byte_offset + 8, first_watermark.byte_offset + 16) \
        == SYSTEM_CONFIGURATION_DEFRAGMENTATION_WATERMARK_OFFSETS_LAYOUT_LINE_158 and first_watermark.byte_width == 24
    first_identity = SYSTEM_CONFIGURATION_FIELDS["root_ring_region_device_identities"]
    assert tuple(first_identity.byte_offset + 4 * region for region in range(3)) \
        == SYSTEM_CONFIGURATION_REGION_DEVICE_IDENTITY_OFFSETS_LAYOUT_LINE_158 and first_identity.byte_width == 12
    for field_name, listed_offset in JOURNAL_RECORD_OFFSETS_LAYOUT_LINE_359.items():
        assert JOURNAL_RECORD_FIELDS[field_name].byte_offset == listed_offset, f"journal 记录头 {field_name}"
    assert JOURNAL_RECORD_FIELDS["jsn_instance"].byte_width == 4 and JOURNAL_RECORD_FIELDS["jsn_counter"].byte_width == 6
    first_ten_fields_end = JOURNAL_RECORD_FIELDS["header_checksum"].end_offset
    assert first_ten_fields_end == 78, "前十个字段是 magic 到头校验和，合计 78（第 359 行）"
    for field_name, listed_offset in ROOT_RECORD_OFFSETS_D22_ITEM_7_AS_REFERRED_BY_LAYOUT_LINE_365.items():
        assert ROOT_RECORD_FIELDS[field_name].byte_offset == listed_offset, f"根记录 {field_name}"
    for field_name, listed_offset in LOCATION_ENTRY_OFFSETS_LAYOUT_LINE_216.items():
        assert LOCATION_ENTRY_FIELDS[field_name].byte_offset == listed_offset, f"位置条目 {field_name}"
    assert (LOCATION_ENTRY_FIELDS["device_identity"].byte_width, LOCATION_ENTRY_FIELDS["slot_number"].byte_width,
            LOCATION_ENTRY_FIELDS["ciphertext_checksum"].byte_width) == (4, 6, 4)
    entry_bytes = LOCATION_ENTRY_FIELDS["ciphertext_checksum"].end_offset
    assert entry_bytes == 14 and tuple(50 + entry_bytes * copy for copy in range(2)) == LOCATION_ENTRY_START_IN_A_POINTER_LAYOUT_LINE_216
    assert tuple(entry_bytes * copy for copy in range(2)) == LOCATION_ENTRY_START_IN_A_NAMED_ITEM_LAYOUT_LINE_216
    assert NAMED_ITEM_FIELDS["first_location_entry"].byte_offset == LOCATION_ENTRY_START_IN_A_NAMED_ITEM_LAYOUT_LINE_216[0]
    assert NAMED_ITEM_FIELDS["second_location_entry"].byte_offset == LOCATION_ENTRY_START_IN_A_NAMED_ITEM_LAYOUT_LINE_216[1]
    for magic_name, magic_value, listed_hex in (("系统配置", SYSTEM_CONFIGURATION_MAGIC, "53465342"), ("根记录", ROOT_RECORD_MAGIC, "53465352"),
                                                ("journal 记录", JOURNAL_RECORD_MAGIC, "5346534a")):
        assert len(magic_value) == MAGIC_BYTES and magic_value.hex() == listed_hex, f"{magic_name} magic"
    assert (SYSTEM_CONFIGURATION_FIELDS["magic"].byte_offset, ROOT_RECORD_FIELDS["magic"].byte_offset,
            JOURNAL_RECORD_FIELDS["magic"].byte_offset) == (0, 0, 0), "三种自证结构的 magic 都从偏移 0 起"


assert_kb_anchors_hold()
assert_every_offset_the_layout_lines_list_holds()
