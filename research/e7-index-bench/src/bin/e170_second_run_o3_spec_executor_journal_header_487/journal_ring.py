"""步 5：扫 journal 环——两盘各一份，逐槽自证；任一份自证过即在（D23 已定项 14「两份镜像何时算「记录在」」）。

.claude/kb/decisions/23-journal的角色与格式.md@已定项 3：tail 住系统配置槽，改 jbd2 两处——恢复不许先信 tail，先全环扫描、逐条验证。
全为零的槽自证不过（A9：4096 个零字节的 CRC-32C ≠ 0），所以只看有不全为零扇区的槽，结果与逐槽全扫相同。
"""

from __future__ import annotations

from dataclasses import dataclass

import on_disk_layout as layout
import spec_gaps
from crash_image import SECTOR_BYTES, SparseCrashImage
from crc32c_castagnoli import crc32c
from system_configuration import PoolSystemConfiguration

FIELDS = layout.JOURNAL_RECORD_FIELDS
HEADER_CHECKSUM = FIELDS["header_checksum"]


@dataclass(frozen=True)
class LocationEntry:
    device_identity: int
    slot_number: int
    ciphertext_checksum: int


@dataclass(frozen=True)
class NamedItem:
    raw: bytes

    def location_entries(self) -> list[LocationEntry]:
        entries = []
        for entry_name in ("first_location_entry", "second_location_entry"):
            entry_span = layout.NAMED_ITEM_FIELDS[entry_name]
            entry_raw = self.raw[entry_span.byte_offset:entry_span.end_offset]
            entries.append(LocationEntry(
                layout.little_endian_unsigned(entry_raw, layout.LOCATION_ENTRY_FIELDS["device_identity"]),
                layout.little_endian_unsigned(entry_raw, layout.LOCATION_ENTRY_FIELDS["slot_number"]),
                layout.little_endian_unsigned(entry_raw, layout.LOCATION_ENTRY_FIELDS["ciphertext_checksum"])))
        return entries

    @property
    def unit_type_tag(self) -> int:
        return layout.little_endian_unsigned(self.raw, layout.NAMED_ITEM_FIELDS["unit_type_tag"])


@dataclass(frozen=True)
class JournalRecord:
    ring_slot_index: int
    raw: bytes

    def unsigned(self, field_name: str) -> int:
        return layout.little_endian_unsigned(self.raw, FIELDS[field_name])

    def field_bytes(self, field_name: str) -> bytes:
        span = FIELDS[field_name]
        return self.raw[span.byte_offset:span.end_offset]

    @property
    def jsn(self) -> tuple[int, int]:
        return (self.unsigned("jsn_instance"), self.unsigned("jsn_counter"))

    @property
    def watermark(self) -> tuple[int, int]:
        # D23 已定项 14 注 2：记录水位 (实例代号, checkpoint_txg) 按实例代号为主比
        return (self.unsigned("jsn_instance"), self.unsigned("checkpoint_txg"))

    @property
    def is_last_of_publish(self) -> bool:
        return bool(self.unsigned("record_flags") & layout.JOURNAL_RECORD_FLAG_LAST_OF_PUBLISH)

    def named_items(self) -> list[NamedItem]:
        start = layout.JOURNAL_HEADER_BYTES
        return [NamedItem(self.raw[start + index * layout.NAMED_ITEM_BYTES:start + (index + 1) * layout.NAMED_ITEM_BYTES])
                for index in range(self.unsigned("named_item_count"))]

    def header_with_checksum_zeroed(self) -> bytes:
        # D23 已定项 19 ②：反向链 = CRC32C(前一条记录的头，header_csum 那 32 字节按零参与)；D23 已定项 10「覆盖前一条记录的整个记录头」，
        # 头宽跟着常量走（格式补丁，登记 12.5 格三：D23 已定项 4 正文 487；I-8.6「311 字节头」按读法 11 读成腐烂）
        header = self.raw[:layout.JOURNAL_HEADER_BYTES]
        return header[:HEADER_CHECKSUM.byte_offset] + bytes(HEADER_CHECKSUM.byte_width) + header[HEADER_CHECKSUM.end_offset:]


RECORD_VALID = "valid"
RECORD_CORRUPT = "corrupt"
RECORD_FOREIGN = "foreign_file_system"
RECORD_UNKNOWN_TYPE = "unknown_record_type"


def verify_record(raw: bytes, record_bytes: int, pool_identity_low_eight_bytes: bytes,
                  gaps: spec_gaps.GapsHitInOneState) -> str:
    # C1：自证过要核 magic = SFSJ（.claude/kb/layout/01-first-txn.md 第 338 行），不等即这一份自证不过。字段偏移按行序（C2，第 359 行）。
    if raw[:layout.MAGIC_BYTES] != layout.JOURNAL_RECORD_MAGIC:
        return RECORD_CORRUPT
    # D23 已定项 26 第 2 条：自述长度先做边界检查再用于校验和；记录 4 KiB 定长（已定项 12）。
    if layout.little_endian_unsigned(raw, FIELDS["record_length"]) != record_bytes:
        return RECORD_CORRUPT
    named_item_count = layout.little_endian_unsigned(raw, FIELDS["named_item_count"])
    if layout.JOURNAL_HEADER_BYTES + named_item_count * layout.NAMED_ITEM_BYTES > record_bytes:
        return RECORD_CORRUPT
    # D23 已定项 13：header_csum 罩整条记录 [0, 4096)，自身按零参与；payload_csum 罩点名项数组；任一不过即该记录无效。
    stored = raw[HEADER_CHECKSUM.byte_offset:HEADER_CHECKSUM.end_offset]
    if any(stored[layout.SELF_CHECKSUM_VALUE_BYTES:]):
        return RECORD_CORRUPT
    covered = raw[:HEADER_CHECKSUM.byte_offset] + bytes(HEADER_CHECKSUM.byte_width) + raw[HEADER_CHECKSUM.end_offset:record_bytes]
    if crc32c(covered) != int.from_bytes(stored[:layout.SELF_CHECKSUM_VALUE_BYTES], "little"):
        return RECORD_CORRUPT
    item_array = raw[layout.JOURNAL_HEADER_BYTES:layout.JOURNAL_HEADER_BYTES + named_item_count * layout.NAMED_ITEM_BYTES]
    if crc32c(item_array) != layout.little_endian_unsigned(raw, FIELDS["payload_checksum"]):
        return RECORD_CORRUPT
    # D23 已定项 1：类型 0 无效（判损坏）、1 普通记录；未知类型按 A 条拒绝挂载。
    record_type = layout.little_endian_unsigned(raw, FIELDS["record_type"])
    if record_type == layout.JOURNAL_RECORD_TYPE_INVALID:
        return RECORD_CORRUPT
    if record_type != layout.JOURNAL_RECORD_TYPE_ORDINARY:
        return RECORD_UNKNOWN_TYPE
    # D23 已定项 4：记录标志位 0 = 本次发布末条，其余位写 0、读到非 0 当损坏；序号 0 当损坏。
    if layout.little_endian_unsigned(raw, FIELDS["record_flags"]) & ~layout.JOURNAL_RECORD_FLAG_LAST_OF_PUBLISH:
        return RECORD_CORRUPT
    if layout.little_endian_unsigned(raw, FIELDS["ordinal_within_publish"]) == 0:
        return RECORD_CORRUPT
    if layout.little_endian_unsigned(raw, FIELDS["commit_marker"]) not in (0, 1):
        gaps.record_hit(spec_gaps.GAP_COMMIT_MARKER_BYTE_OUTSIDE_ZERO_AND_ONE)
        return RECORD_CORRUPT
    # D23 已定项 4：fsid 8（系统配置 fsid 的低 8 字节）——全环扫描靠它挡住上一个文件系统留下的记录。
    if raw[FIELDS["file_system_identity_low_eight_bytes"].byte_offset:FIELDS["file_system_identity_low_eight_bytes"].end_offset] \
            != pool_identity_low_eight_bytes:
        return RECORD_FOREIGN
    # 格式补丁（登记 12.5 格三、格五，读法 11 最后一条）：新根段的卸载记号 4 字节（D23 已定项 15）除位 0 外的位读到非 0 怎么判，kb 没写（G23）；
    # 暂代读法照 D23 已定项 4 记录标志「其余位写 0，读到非 0 当损坏」与 D22 已定项 7 flags「其余位第一版恒 0、非 0 拒收」同一用意：当该记录损坏。
    if layout.little_endian_unsigned(raw, FIELDS["new_root_unmount_marker"]) & ~layout.ROOT_FLAGS_UNMOUNT_MARKER_BIT:
        gaps.record_hit(spec_gaps.GAP_UNMOUNT_MARKER_BITS_OTHER_THAN_BIT_ZERO_IN_THE_NEW_ROOT_SEGMENT)
        return RECORD_CORRUPT
    return RECORD_VALID


@dataclass(frozen=True)
class RingScan:
    records_by_counter: dict[int, JournalRecord]  # 每个计数器至多一条（落在自己槽位上、任一份自证过）
    valid_records_by_device: dict[int, list[JournalRecord]]  # A3 用：逐盘的自证记录
    ring_slot_count: int
    unknown_record_type_seen: bool


def scan_journal_ring(image: SparseCrashImage, pool: PoolSystemConfiguration,
                      gaps: spec_gaps.GapsHitInOneState) -> RingScan:
    """D22 已定项 8 第 2 条：journal 环在两块盘上互为镜像，设备内偏移相同；环起点与环长住系统配置（D23 已定项 2、已定项 18）。"""
    ring_start = pool.unsigned("journal_ring_start_slot") * layout.PLACEMENT_SLOT_BYTES
    record_bytes = pool.unsigned("journal_maximum_record_bytes")
    ring_slot_count = pool.unsigned("journal_ring_bytes") // record_bytes
    # 「低 8 字节」按小端读成 fsid 的前 8 个字节（D22 已定项 13 一律小端）。
    pool_identity_low_eight_bytes = pool.field_bytes("file_system_identity")[:8]
    valid_records_by_device: dict[int, list[JournalRecord]] = {}
    valid_by_slot: dict[int, list[JournalRecord]] = {}
    unknown_record_type_seen = False
    for device_identity in image.device_identities():
        touched_slots = sorted({(sector_index * SECTOR_BYTES - ring_start) // record_bytes for sector_index in
                                image.nonzero_sector_indices_in(device_identity, ring_start, ring_start + ring_slot_count * record_bytes)})
        device_records = []
        for ring_slot_index in touched_slots:
            raw = image.read_bytes(device_identity, ring_start + ring_slot_index * record_bytes, record_bytes)
            verdict = verify_record(raw, record_bytes, pool_identity_low_eight_bytes, gaps)
            if verdict == RECORD_UNKNOWN_TYPE:
                unknown_record_type_seen = True
            if verdict != RECORD_VALID:
                continue
            record = JournalRecord(ring_slot_index, raw)
            device_records.append(record)
            valid_by_slot.setdefault(ring_slot_index, []).append(record)
        valid_records_by_device[device_identity] = device_records
    records_by_counter: dict[int, JournalRecord] = {}
    for ring_slot_index, copies in sorted(valid_by_slot.items()):
        if len({copy.raw for copy in copies}) > 1:
            gaps.record_hit(spec_gaps.GAP_TWO_SELF_VERIFIED_COPIES_OF_ONE_RING_SLOT_DIFFER)
            continue  # G07 暂代读法：这一槽当读不出
        record = copies[0]
        counter = record.unsigned("jsn_counter")
        # D23 已定项 18：记录 n 落在环内偏移 (计数器 − 1) mod 槽数 × 4096。落错槽的记录收不收，kb 没写（G08），暂代读法不收。
        if counter == 0 or (counter - 1) % ring_slot_count != ring_slot_index:
            gaps.record_hit(spec_gaps.GAP_RECORD_SITS_OUTSIDE_THE_SLOT_ITS_COUNTER_GIVES)
            continue
        records_by_counter[counter] = record
    return RingScan(records_by_counter, valid_records_by_device, ring_slot_count, unknown_record_type_seen)
