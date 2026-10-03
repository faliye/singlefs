"""E170 第 2 次重跑第二段：参照谓词 W_ref、T_ref（`research/prompts/e170-r2-prereg.md` 5.4）。

照登记第一节读法 2、3 与第二节抄的 kb 句子另写一份，不打开 v2 的 `witness_predicates.py` 与 `executor_core.py`、`executor_main.py` 相对 v1 的改动。
系统配置槽的自证、journal tail、记录头的自证与字段、两块盘各一份地扫环、从录制流认 ack 那次发布的轮换写取 τ，都在这一份里自己写；
许共用的只有三样（5.4）：建镜像（`crash_image.build_crash_image`，它要 `crash_dump.load_stream_dump` 读出的流；建完的镜像摘要与 `states.tsv`
或坏状态表里导出装置算的那一个比）、CRC-32C（`crc32c_castagnoli.crc32c`，第七节 B8–B10）、L(c) 与 ack(c)（从 v2 判定行的 `landed=`、`ack=` 读）。

跑法（在本目录，经 capped.sh 与 run-with-memory-cap.sh）：
  python3 witness_reference.py states --dumps <导出目录> --v2-output <v2 的 executor_main 产物> --input-label <名> --output <新文件>
  python3 witness_reference.py images --dumps <含 bad-*.tsv 的导出目录> [--known-bad-dir <目录>] --v2-output <v2 的坏镜像判定产物>
      --input-label <名> --output <新文件>
文件名以 witness_reference 起头，不进 v2 冻结的 sha256（登记 5.6）。
"""

from __future__ import annotations

import argparse
import os
import re
import sys
from dataclasses import dataclass

sys.dont_write_bytecode = True

import crc32c_castagnoli  # noqa: E402  共用之二：CRC-32C
from crash_dump import load_stream_dump  # noqa: E402  建镜像要的流（共用之一的输入）
from crash_image import SparseCrashImage, build_crash_image, prepare_writes  # noqa: E402  共用之一：建镜像

# ---- 系统配置槽：`.claude/kb/layout/01-first-txn.md` 第 158 行（字段偏移）、第 160 行（槽位与「自证过」）、第 109–111 行（magic、格式版本、incompat 位） ----
SYSTEM_CONFIGURATION_SLOT_BYTES = 4096  # layout 第 96 行：槽宽 = 格式常量 4096
SYSTEM_CONFIGURATION_MAGIC = b"SFSB"  # layout 第 109 行：4 字节 ASCII SFSB
SYSTEM_CONFIGURATION_FORMAT_VERSION_OFFSET = 4  # 第 158 行「格式版本 4」，宽 2（第 110 行）
SYSTEM_CONFIGURATION_SUPPORTED_FORMAT_VERSION = 1  # 第 110 行：格式版本 1
SYSTEM_CONFIGURATION_INCOMPAT_BITMAP_OFFSET = 6  # 第 158 行「feature bits 6」；incompat 是三个 256 位里的第一个（第 111 行）
SYSTEM_CONFIGURATION_INCOMPAT_BITMAP_BYTES = 32  # 256 位
SYSTEM_CONFIGURATION_KNOWN_INCOMPAT_BITS = frozenset({1})  # 第 111 行：incompat 位 1 = 1，位 0 退役，其余全 0
SYSTEM_CONFIGURATION_SLOT_GENERATION_OFFSET = 147  # 第 158 行「槽世代号 147」，宽 8
SYSTEM_CONFIGURATION_CHECKSUM_OFFSET = 155  # 第 158 行「整槽校验和 155」
SYSTEM_CONFIGURATION_CHECKSUM_FIELD_BYTES = 32  # 第 118 行：CRC32C 4 + 28 零，覆盖整槽；字段按零参与（第 99 行）
SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFFSET = 219  # 第 158 行「加密类型 219」，宽 1
SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_UNENCRYPTED = 0  # 第 122 行：0（未加密）
SYSTEM_CONFIGURATION_JOURNAL_RING_START_SLOT_OFFSET = 325  # 第 158 行；16 KiB 槽号（第 131 行）
SYSTEM_CONFIGURATION_JOURNAL_RING_LENGTH_OFFSET = 333  # 第 158 行；字节（第 132 行）
SYSTEM_CONFIGURATION_FIXED_STRUCTURE_SLOT_SPACING_OFFSET = 429  # 第 160 行：偏移 429 起那 4 字节，小端
SYSTEM_CONFIGURATION_JOURNAL_TAIL_OFFSET = 469  # 第 158 行「journal tail 469」，宽 8（第 152 行）
JOURNAL_RING_START_SLOT_BYTES = 16384  # 第 131 行：journal 环起点按 16 KiB 槽号记
SLOT_ONE_EVERY_SPACING_FIRST_OFFSET = 4096  # 第 160 行 ③：4096 到 1 MiB − 4096 之间每 512 字节一档
SLOT_ONE_EVERY_SPACING_LAST_OFFSET = (1 << 20) - 4096
SLOT_ONE_EVERY_SPACING_STEP = 512

# ---- journal 记录头：layout 第 359 行（311 格式的偏移，本实验的输入都是 311 格式的冻结导出，登记读法 7）、D23 已定项 4、13、17 ----
JOURNAL_RECORD_BYTES = 4096  # D23 已定项 12；layout 第 319 行
JOURNAL_RECORD_MAGIC = b"SFSJ"  # layout 第 338 行
JOURNAL_RECORD_TYPE_OFFSET = 4  # 宽 2；1 = 普通记录（第 339 行）
JOURNAL_RECORD_TYPE_ORDINARY = 1
JOURNAL_RECORD_FLAGS_OFFSET = 7  # 宽 1；位 0 = 本次发布末条（第 341 行；D23 已定项 4：其余位写 0，读到非 0 当损坏）
JOURNAL_RECORD_LAST_OF_PUBLISH_FLAG = 0x01
JOURNAL_RECORD_LENGTH_OFFSET = 8  # 宽 4；4096（第 342 行）
JOURNAL_RECORD_NAMED_ITEM_COUNT_OFFSET = 12  # 宽 4（第 343 行）
JOURNAL_RECORD_INSTANCE_OFFSET = 16  # jsn：实例代号 4 字节在 16
JOURNAL_RECORD_COUNTER_OFFSET = 20  # 计数器 6 字节在 20（第 359 行）
JOURNAL_RECORD_COUNTER_BYTES = 6
JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET = 26  # 宽 8
JOURNAL_RECORD_HEADER_CHECKSUM_OFFSET = 46  # 32 字节：CRC32C 4 + 28 零，覆盖整条 [0, 4096)、自身按零参与（D23 已定项 13）
JOURNAL_RECORD_HEADER_CHECKSUM_FIELD_BYTES = 32
JOURNAL_RECORD_PAYLOAD_CHECKSUM_OFFSET = 95  # 宽 4，罩点名项数组（D23 已定项 13）
JOURNAL_RECORD_HEADER_BYTES_311_FORMAT = 311  # layout 第 359 行「到 311 为止」；点名项区从这里起
JOURNAL_RECORD_NAMED_ITEM_BYTES = 56  # D23 已定项 17；layout 第 361 行
JOURNAL_RECORD_MOST_NAMED_ITEMS_311_FORMAT = (JOURNAL_RECORD_BYTES - JOURNAL_RECORD_HEADER_BYTES_311_FORMAT) // JOURNAL_RECORD_NAMED_ITEM_BYTES

# ---- 录制流里的根槽写：根记录 magic（layout 第 369 行，七「发布」一节根记录字段表的 magic 行；版本表 = 基镜像那一版加每一次根槽写按流序各一版，原登记读法 3） ----
ROOT_RECORD_MAGIC = b"SFSR"
WRITE_KINDS = ("write", "write_fua", "write_zeroes")
BARRIER_KIND = "barrier"

assert JOURNAL_RECORD_MOST_NAMED_ITEMS_311_FORMAT == 67, "layout 第 329 行：4096 − 311 = 3785，一条记录最多装 67 项"


def unsigned_little_endian(content: bytes, offset: int, width: int) -> int:
    return int.from_bytes(content[offset:offset + width], "little")


def checksum_field_holds(content: bytes, field_offset: int, field_bytes: int) -> bool:
    """D18 已定项 17 与 D23 已定项 13 的写法：CRC32C 4 字节（小端）+ 28 字节零；CRC 覆盖整个槽，那 32 字节按零参与。"""
    stored = unsigned_little_endian(content, field_offset, 4)
    padding_is_zero = content[field_offset + 4:field_offset + field_bytes] == bytes(field_bytes - 4)
    zeroed = content[:field_offset] + bytes(field_bytes) + content[field_offset + field_bytes:]
    return padding_is_zero and crc32c_castagnoli.crc32c(zeroed) == stored


@dataclass(frozen=True)
class SelfVerifiedSystemConfigurationSlot:
    device_identity: int
    byte_offset: int
    slot_generation: int
    journal_tail: int
    fixed_structure_slot_spacing: int
    journal_ring_start_byte: int
    journal_ring_length_bytes: int


def system_configuration_slot_if_self_verified(image: SparseCrashImage, device_identity: int,
                                               byte_offset: int) -> SelfVerifiedSystemConfigurationSlot | None:
    """layout 第 160 行「自证过」的五样：magic、整槽校验和、格式版本、incompat 位、加密类型都过。几何字段的范围不在「自证过」里。"""
    content = image.read_bytes(device_identity, byte_offset, SYSTEM_CONFIGURATION_SLOT_BYTES)
    if content is None or len(content) != SYSTEM_CONFIGURATION_SLOT_BYTES:
        return None
    if content[:4] != SYSTEM_CONFIGURATION_MAGIC:
        return None
    if not checksum_field_holds(content, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET, SYSTEM_CONFIGURATION_CHECKSUM_FIELD_BYTES):
        return None
    if unsigned_little_endian(content, SYSTEM_CONFIGURATION_FORMAT_VERSION_OFFSET, 2) != SYSTEM_CONFIGURATION_SUPPORTED_FORMAT_VERSION:
        return None
    incompat = content[SYSTEM_CONFIGURATION_INCOMPAT_BITMAP_OFFSET:SYSTEM_CONFIGURATION_INCOMPAT_BITMAP_OFFSET + SYSTEM_CONFIGURATION_INCOMPAT_BITMAP_BYTES]
    set_bits = {byte_index * 8 + bit for byte_index, value in enumerate(incompat) for bit in range(8) if value >> bit & 1}
    if not set_bits <= SYSTEM_CONFIGURATION_KNOWN_INCOMPAT_BITS:
        return None
    if content[SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFFSET] != SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_UNENCRYPTED:
        return None
    return SelfVerifiedSystemConfigurationSlot(
        device_identity, byte_offset,
        unsigned_little_endian(content, SYSTEM_CONFIGURATION_SLOT_GENERATION_OFFSET, 8),
        unsigned_little_endian(content, SYSTEM_CONFIGURATION_JOURNAL_TAIL_OFFSET, 8),
        unsigned_little_endian(content, SYSTEM_CONFIGURATION_FIXED_STRUCTURE_SLOT_SPACING_OFFSET, 4),
        unsigned_little_endian(content, SYSTEM_CONFIGURATION_JOURNAL_RING_START_SLOT_OFFSET, 8) * JOURNAL_RING_START_SLOT_BYTES,
        unsigned_little_endian(content, SYSTEM_CONFIGURATION_JOURNAL_RING_LENGTH_OFFSET, 8))


def spacing_is_one_the_reader_accepts(spacing: int) -> bool:
    """「可用」= 自证过且几何字段在读者收的范围里（layout 第 160 行）。这一份只核槽距一样：落在 ③ 的逐档范围里、按 512 对齐。
    其余几何字段的范围这里不核，写在交回里（与 v2 若有不同，进 Q2-c 逐态列）。"""
    return (SLOT_ONE_EVERY_SPACING_FIRST_OFFSET <= spacing <= SLOT_ONE_EVERY_SPACING_LAST_OFFSET
            and spacing % SLOT_ONE_EVERY_SPACING_STEP == 0)


@dataclass(frozen=True)
class PoolSystemConfigurationSlots:
    """每块盘两槽里自证过的那几份（槽 0 在偏移 0；槽 1 的偏移照 layout 第 160 行 ① ② ③ 取），以及槽 1 走的哪一步。"""

    self_verified_by_device: dict[int, list[SelfVerifiedSystemConfigurationSlot]]
    slot_one_route_by_device: dict[int, str]


def read_pool_system_configuration_slots(image: SparseCrashImage, device_identities: list[int]) -> PoolSystemConfigurationSlots:
    slot_zero_by_device = {device: system_configuration_slot_if_self_verified(image, device, 0) for device in device_identities}
    usable_slot_zero = {device: slot for device, slot in slot_zero_by_device.items()
                        if slot is not None and spacing_is_one_the_reader_accepts(slot.fixed_structure_slot_spacing)}
    first_usable_slot_zero_of_the_pool = next((usable_slot_zero[device] for device in sorted(device_identities) if device in usable_slot_zero), None)
    self_verified_by_device: dict[int, list[SelfVerifiedSystemConfigurationSlot]] = {}
    route_by_device: dict[int, str] = {}
    for device in device_identities:
        slots = [slot_zero_by_device[device]] if slot_zero_by_device[device] is not None else []
        if device in usable_slot_zero:
            route_by_device[device] = "own_slot_zero"
            slot_one = system_configuration_slot_if_self_verified(image, device, usable_slot_zero[device].fixed_structure_slot_spacing)
        elif first_usable_slot_zero_of_the_pool is not None:
            route_by_device[device] = "first_usable_slot_zero_of_the_pool"
            slot_one = system_configuration_slot_if_self_verified(image, device, first_usable_slot_zero_of_the_pool.fixed_structure_slot_spacing)
        else:
            route_by_device[device] = "every_spacing"
            collected = []
            for offset in range(SLOT_ONE_EVERY_SPACING_FIRST_OFFSET, SLOT_ONE_EVERY_SPACING_LAST_OFFSET + 1, SLOT_ONE_EVERY_SPACING_STEP):
                candidate = system_configuration_slot_if_self_verified(image, device, offset)
                if candidate is not None and candidate.fixed_structure_slot_spacing == offset:
                    collected.append(candidate)
            # 几档都收得到时取世代号最大的，世代号相同取偏移小的（layout 第 160 行 ③）
            slot_one = min(collected, key=lambda slot: (-slot.slot_generation, slot.byte_offset)) if collected else None
        if slot_one is not None:
            slots.append(slot_one)
        self_verified_by_device[device] = slots
    return PoolSystemConfigurationSlots(self_verified_by_device, route_by_device)


def witness_counter_of(slots: PoolSystemConfigurationSlots) -> int:
    """c_见证 = 池里每块盘两槽中全部自证过的系统配置槽的 journal tail（偏移 469）的最大值；一份都没有时 0（登记读法 2）。"""
    tails = [slot.journal_tail for device_slots in slots.self_verified_by_device.values() for slot in device_slots]
    return max(tails, default=0)


@dataclass(frozen=True)
class SelfVerifiedJournalRecord:
    device_identity: int
    ring_byte_offset: int
    record_type: int
    instance: int
    counter: int
    checkpoint_txg: int
    is_last_of_publish: bool


def journal_record_if_self_verified(content: bytes, device_identity: int, ring_byte_offset: int) -> SelfVerifiedJournalRecord | None:
    """D23 已定项 13：任一校验和不过就是该记录无效；已定项 26：自述长度先做边界检查再算校验和；已定项 4：记录标志位 0 之外读到非 0 当损坏。
    「记录在」= 任一份自证过（D23 已定项 14 两份镜像那一段），由调用方按盘各扫一遍再合起来。"""
    if content[:4] != JOURNAL_RECORD_MAGIC:
        return None
    if unsigned_little_endian(content, JOURNAL_RECORD_LENGTH_OFFSET, 4) != JOURNAL_RECORD_BYTES:
        return None
    named_item_count = unsigned_little_endian(content, JOURNAL_RECORD_NAMED_ITEM_COUNT_OFFSET, 4)
    if named_item_count > JOURNAL_RECORD_MOST_NAMED_ITEMS_311_FORMAT:
        return None
    if not checksum_field_holds(content, JOURNAL_RECORD_HEADER_CHECKSUM_OFFSET, JOURNAL_RECORD_HEADER_CHECKSUM_FIELD_BYTES):
        return None
    named_items = content[JOURNAL_RECORD_HEADER_BYTES_311_FORMAT:
                          JOURNAL_RECORD_HEADER_BYTES_311_FORMAT + named_item_count * JOURNAL_RECORD_NAMED_ITEM_BYTES]
    if crc32c_castagnoli.crc32c(named_items) != unsigned_little_endian(content, JOURNAL_RECORD_PAYLOAD_CHECKSUM_OFFSET, 4):
        return None
    flags = content[JOURNAL_RECORD_FLAGS_OFFSET]
    if flags & ~JOURNAL_RECORD_LAST_OF_PUBLISH_FLAG & 0xFF:
        return None
    record_type = unsigned_little_endian(content, JOURNAL_RECORD_TYPE_OFFSET, 2)
    if record_type != JOURNAL_RECORD_TYPE_ORDINARY:
        return None
    return SelfVerifiedJournalRecord(
        device_identity, ring_byte_offset, record_type,
        unsigned_little_endian(content, JOURNAL_RECORD_INSTANCE_OFFSET, 4),
        unsigned_little_endian(content, JOURNAL_RECORD_COUNTER_OFFSET, JOURNAL_RECORD_COUNTER_BYTES),
        unsigned_little_endian(content, JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET, 8),
        bool(flags & JOURNAL_RECORD_LAST_OF_PUBLISH_FLAG))


def journal_ring_geometry(slots: PoolSystemConfigurationSlots) -> tuple[int, int] | None:
    """环的几何取全池自证过的槽里世代号最大的那一份（同一个池各槽写的是同一组 mkfs 参数）；一份都没有时 None。"""
    every_slot = [slot for device_slots in slots.self_verified_by_device.values() for slot in device_slots]
    if not every_slot:
        return None
    newest = max(every_slot, key=lambda slot: (slot.slot_generation, -slot.device_identity, -slot.byte_offset))
    return newest.journal_ring_start_byte, newest.journal_ring_length_bytes


def scan_journal_ring_on_every_device(image: SparseCrashImage, device_identities: list[int],
                                      geometry: tuple[int, int]) -> list[SelfVerifiedJournalRecord]:
    """两块盘各一份地扫环：只看首扇区不全为零的 4096 字节槽（全零的槽没有 magic，自证不过）。"""
    ring_start, ring_length = geometry
    found = []
    for device in device_identities:
        candidate_ring_offsets = sorted({(sector * 512 - ring_start) // JOURNAL_RECORD_BYTES * JOURNAL_RECORD_BYTES
                                         for sector in image.nonzero_sector_indices_in(device, ring_start, ring_start + ring_length)})
        for ring_offset in candidate_ring_offsets:
            content = image.read_bytes(device, ring_start + ring_offset, JOURNAL_RECORD_BYTES)
            record = None if content is None else journal_record_if_self_verified(content, device, ring_offset)
            if record is not None:
                found.append(record)
    return found


@dataclass(frozen=True)
class WitnessReference:
    evaluated: bool
    witness_counter: int
    branch: str  # nothing / last_record / record_at_counter / both_unreadable / not_evaluated
    compared: str  # last_record：比的那个计数器；record_at_counter：那条记录的 (实例代号, checkpoint_txg)；其余 "-"
    witness_true: bool | None
    candidates_in_branch: int  # 那一支读得出的候选记录份数（两块盘各算一份），kb 那一段写的是单数，几份时这里如实报


def witness_reference(landed: tuple[int, int] | None, witness_counter: int,
                      records: list[SelfVerifiedJournalRecord]) -> WitnessReference:
    """登记读法 2（D23 已定项 14「可写挂载读到的更新状态读不出」那一段的计数那一支）。landed = L(c) 的 (实例代号, checkpoint_txg)，
    L(c) = 无时不算。比较 (实例代号, checkpoint_txg) 按实例代号为主（D23 已定项 14 注 2：记录水位按实例代号为主比）。"""
    if landed is None:
        return WitnessReference(False, witness_counter, "not_evaluated", "-", None, 0)
    if witness_counter == 0:
        return WitnessReference(True, witness_counter, "nothing", "-", False, 0)
    last_records = [record for record in records
                    if (record.instance, record.checkpoint_txg) == landed and record.is_last_of_publish]
    if last_records:
        compared_counter = max(record.counter for record in last_records)
        return WitnessReference(True, witness_counter, "last_record", str(compared_counter),
                                witness_counter > compared_counter, len(last_records))
    records_at_counter = [record for record in records if record.counter == witness_counter]
    if records_at_counter:
        newest_key = max((record.instance, record.checkpoint_txg) for record in records_at_counter)
        return WitnessReference(True, witness_counter, "record_at_counter", f"{newest_key[0]},{newest_key[1]}",
                                newest_key > landed, len(records_at_counter))
    return WitnessReference(True, witness_counter, "both_unreadable", "-", True, 0)


@dataclass(frozen=True)
class RecordedWrite:
    stream_index: int
    device_identity: int
    kind: str
    byte_offset: int
    payload: bytes | None


def read_recorded_stream(stream_directory: str) -> list[RecordedWrite]:
    """自己读 `ops.tsv` 与 `ops-payload.bin`（不经 crash_dump）：列 = 流序下标、盘身份、种类、盘内偏移、字节数、载荷起点、载荷 sha256。"""
    with open(os.path.join(stream_directory, "ops-payload.bin"), "rb") as handle:
        payload_blob = handle.read()
    operations = []
    with open(os.path.join(stream_directory, "ops.tsv"), encoding="utf-8") as handle:
        for line in handle:
            columns = line.rstrip("\n").split("\t")
            if len(columns) < 7 or not columns[0].isdigit():
                continue
            stream_index, device, kind, byte_offset, byte_count, payload_start = columns[:6]
            payload = None if payload_start == "-" else payload_blob[int(payload_start):int(payload_start) + int(byte_count)]
            operations.append(RecordedWrite(int(stream_index), int(device), kind, int(byte_offset) if byte_offset != "-" else 0, payload))
    assert [operation.stream_index for operation in operations] == list(range(len(operations))), "ops.tsv 的流序下标不是从 0 起逐一连续"
    return operations


@dataclass(frozen=True)
class PublishRotation:
    """版本 k（k ≥ 1）= 录制流里第 k 次根槽写（版本表的定义，原登记读法 3）。τ_k = 那次根槽写之后、其后第一道屏障之前，
    落在系统配置槽偏移上、整槽自证过的那几次写所带 journal tail 的最大值（登记读法 3，D16 已定项 7 的持久顺序）。"""

    version_index: int
    root_write_stream_index: int
    rotation_write_stream_indices: tuple[int, ...]
    self_verified_rotation_tails: tuple[int, ...]
    rotation_tail: int | None
    last_record_counter: int | None  # 这次发布写进流里、带末条标志、自证过的记录的计数器（Q3-f 用；几份取最大）
    rotation_offsets_seen_by_fallback: int  # 槽偏移靠写进去那一槽自述的槽距认的次数（基镜像上那块盘没有自证过的槽 0 时）


def system_configuration_offsets_by_device(base_image: SparseCrashImage, device_identities: list[int]) -> dict[int, set[int]]:
    """槽 0 在偏移 0；槽 1 在本盘固定结构槽距（layout 第 160 行）——槽距取基镜像上这块盘自证过的槽 0 自述的，没有就取池里第一块有的。"""
    slot_zero = {device: system_configuration_slot_if_self_verified(base_image, device, 0) for device in device_identities}
    first_of_pool = next((slot_zero[device] for device in sorted(device_identities) if slot_zero[device] is not None), None)
    offsets = {}
    for device in device_identities:
        source = slot_zero[device] or first_of_pool
        offsets[device] = {0} | ({source.fixed_structure_slot_spacing} if source is not None else set())
    return offsets


def journal_record_in_written_payload(payload: bytes | None, device_identity: int) -> SelfVerifiedJournalRecord | None:
    if payload is None or len(payload) != JOURNAL_RECORD_BYTES:
        return None
    return journal_record_if_self_verified(payload, device_identity, -1)


def publish_rotations(operations: list[RecordedWrite], base_image: SparseCrashImage,
                      device_identities: list[int]) -> list[PublishRotation]:
    slot_offsets = system_configuration_offsets_by_device(base_image, device_identities)
    root_write_indices = [operation.stream_index for operation in operations
                          if operation.kind in WRITE_KINDS and operation.payload is not None and operation.payload[:4] == ROOT_RECORD_MAGIC]
    rotations = []
    for version_index, root_write_index in enumerate(root_write_indices, start=1):
        rotation_indices, tails, fallback = [], [], 0
        for operation in operations[root_write_index + 1:]:
            if operation.kind == BARRIER_KIND:
                break
            if operation.kind not in WRITE_KINDS or operation.payload is None or len(operation.payload) != SYSTEM_CONFIGURATION_SLOT_BYTES:
                continue
            image_of_one_slot = SparseCrashImage({operation.device_identity: {}})
            image_of_one_slot.write_bytes(operation.device_identity, operation.byte_offset, operation.payload)
            slot = system_configuration_slot_if_self_verified(image_of_one_slot, operation.device_identity, operation.byte_offset)
            on_a_slot_offset = operation.byte_offset in slot_offsets.get(operation.device_identity, {0})
            if not on_a_slot_offset and slot is not None and slot.fixed_structure_slot_spacing == operation.byte_offset:
                on_a_slot_offset, fallback = True, fallback + 1
            if not on_a_slot_offset:
                continue
            rotation_indices.append(operation.stream_index)
            if slot is not None:
                tails.append(slot.journal_tail)
        previous_root_write = root_write_indices[version_index - 2] if version_index >= 2 else -1
        last_counters = [record.counter for operation in operations[previous_root_write + 1:root_write_index]
                         if operation.kind in WRITE_KINDS
                         for record in [journal_record_in_written_payload(operation.payload, operation.device_identity)]
                         if record is not None and record.is_last_of_publish]
        rotations.append(PublishRotation(version_index, root_write_index, tuple(rotation_indices), tuple(tails),
                                         max(tails, default=None), max(last_counters, default=None), fallback))
    return rotations


@dataclass(frozen=True)
class TailCheckReference:
    value: str  # true / false / not_evaluated
    reason: str
    rotation_tail: int | None


def tail_check_reference(landed: tuple[int, int] | None, acknowledged_version_index: int | None, witness_counter: int,
                         rotations_by_version: dict[int, PublishRotation]) -> TailCheckReference:
    """登记读法 3：T =「c_见证 ≥ τ(c)」；ack(c) 是版本 0、或那次发布的轮换写一份自证过的都没有 ⇒ 不判；读法 4：L(c) = 无的态不算。"""
    if landed is None:
        return TailCheckReference("not_evaluated", "landed_none", None)
    if acknowledged_version_index is None or acknowledged_version_index == 0:
        return TailCheckReference("not_evaluated", "ack_is_version_zero", None)
    rotation = rotations_by_version.get(acknowledged_version_index)
    if rotation is None:
        return TailCheckReference("not_evaluated", "ack_version_has_no_root_write_in_the_stream", None)
    if rotation.rotation_tail is None:
        return TailCheckReference("not_evaluated", "no_self_verified_rotation_write", None)
    return TailCheckReference(str(witness_counter >= rotation.rotation_tail).lower(), "evaluated", rotation.rotation_tail)


# ---- 驱动：读 v2 判定行里的 L(c)、ack(c)（共用之三），逐态建镜像、算 W_ref 与 T_ref，打一行 ----

FIELD_PATTERN = re.compile(r"(\w+)=(\S*)")
LANDED_PAIR_PATTERN = re.compile(r"^\((\d+),(\d+)")
STREAM_NAMES = ("new_pool_file_creation", "fixed_script_through_e_then_unmount")


def fields_of(line: str) -> dict[str, str]:
    body = line.rstrip("\n").removeprefix("E7RESULT ")
    return {key: value for key, value in FIELD_PATTERN.findall(body)}


def landed_pair(landed_text: str) -> tuple[int, int] | None:
    match = LANDED_PAIR_PATTERN.match(landed_text)
    return None if match is None else (int(match.group(1)), int(match.group(2)))


def acknowledged_index(ack_text: str) -> int | None:
    return int(ack_text) if ack_text.isdigit() else None


@dataclass
class StreamMaterial:
    dump: object
    prepared_writes: list
    rotations_by_version: dict[int, PublishRotation]


def load_stream_material(stream_directory: str) -> StreamMaterial:
    dump = load_stream_dump(stream_directory)
    base_image = build_crash_image(dump, prepare_writes(dump), "0" * len(dump.write_operation_indices))
    rotations = publish_rotations(read_recorded_stream(stream_directory), base_image, dump.device_identities)
    return StreamMaterial(dump, prepare_writes(dump), {rotation.version_index: rotation for rotation in rotations})


def reference_line(input_label: str, identity: str, v2_fields: dict[str, str], image: SparseCrashImage,
                   digest_matches: bool, material: StreamMaterial) -> str:
    device_identities = material.dump.device_identities
    landed = landed_pair(v2_fields.get("landed", "none"))
    acknowledged = acknowledged_index(v2_fields.get("ack", "-"))
    slots = read_pool_system_configuration_slots(image, device_identities)
    witness_counter = witness_counter_of(slots)
    geometry = journal_ring_geometry(slots)
    records = [] if geometry is None else scan_journal_ring_on_every_device(image, device_identities, geometry)
    witness = witness_reference(landed, witness_counter, records)
    tail = tail_check_reference(landed, acknowledged, witness_counter, material.rotations_by_version)
    relation = "-" if tail.rotation_tail is None else ("eq" if witness_counter == tail.rotation_tail else
                                                       "gt" if witness_counter > tail.rotation_tail else "lt")
    fewer_than_two = sum(len(device_slots) < 2 for device_slots in slots.self_verified_by_device.values())
    routes = ",".join(f"d{device}:{slots.slot_one_route_by_device[device]}" for device in sorted(slots.slot_one_route_by_device))
    witness_text = "-" if witness.witness_true is None else str(witness.witness_true).lower()
    return (f"name=witness_reference input={input_label} {identity} digest={'match' if digest_matches else 'mismatch'} "
            f"landed={v2_fields.get('landed', '-')} ack={v2_fields.get('ack', '-')} w_ref_evaluated={str(witness.evaluated).lower()} "
            f"c_witness={witness_counter} w_branch={witness.branch} w_compared={witness.compared} w_ref={witness_text} "
            f"w_candidates={witness.candidates_in_branch} t_ref={tail.value} t_reason={tail.reason} "
            f"tau={'-' if tail.rotation_tail is None else tail.rotation_tail} c_witness_vs_tau={relation} "
            f"disks_with_fewer_than_two_self_verified_slots={fewer_than_two} slot_one_routes={routes} "
            f"records_self_verified={len(records)} ring_geometry={'none' if geometry is None else 'found'}")


def rotation_lines(input_label: str, stream_name: str, material: StreamMaterial, v2_root_writes: dict[int, int], emit) -> None:
    for version_index, rotation in sorted(material.rotations_by_version.items()):
        tails = rotation.self_verified_rotation_tails
        emit(f"name=reference_rotation input={input_label} stream={stream_name} version={version_index} "
             f"root_write={rotation.root_write_stream_index} rotation_writes={','.join(map(str, rotation.rotation_write_stream_indices)) or '-'} "
             f"self_verified_tails={','.join(map(str, tails)) or '-'} tau={'-' if rotation.rotation_tail is None else rotation.rotation_tail} "
             f"last_record_counter={'-' if rotation.last_record_counter is None else rotation.last_record_counter} "
             f"tau_differs_from_last_record_counter={str(rotation.rotation_tail is not None and rotation.last_record_counter is not None and rotation.rotation_tail != rotation.last_record_counter).lower()} "
             f"tails_differ_across_rotation_writes={str(len(set(tails)) > 1).lower()} offsets_by_fallback={rotation.rotation_offsets_seen_by_fallback}")
    if v2_root_writes:
        mine = {version: rotation.root_write_stream_index for version, rotation in material.rotations_by_version.items()}
        differing = sorted(version for version in set(mine) | set(v2_root_writes) if mine.get(version) != v2_root_writes.get(version))
        emit(f"name=reference_root_write_check input={input_label} stream={stream_name} reference_root_writes={len(mine)} "
             f"v2_version_lines_with_root_write={len(v2_root_writes)} differing_versions={','.join(map(str, differing)) or 'zero'}")


class ResultFile:
    def __init__(self, path: str):
        self.handle = open(path, "x", encoding="utf-8")
        self.emitted = 0

    def emit(self, line: str) -> None:
        self.emitted += 1
        self.handle.write(line + "\n")

    def finish(self) -> None:
        self.emitted += 1
        self.handle.write(f"name=done emitted={self.emitted}\n")
        self.handle.close()


def read_v2_lines(path: str, wanted_name: str) -> list[dict[str, str]]:
    with open(path, encoding="utf-8") as handle:
        return [fields_of(line) for line in handle if line.removeprefix("E7RESULT ").startswith(f"name={wanted_name} ")]


def summary_line(input_label: str, emitted_reference_lines: list[str]) -> str:
    parsed = [fields_of(line) for line in emitted_reference_lines]
    return (f"name=reference_summary input={input_label} states={len(parsed)} "
            f"digest_mismatches={sum(fields['digest'] != 'match' for fields in parsed)} "
            f"w_ref_evaluated={sum(fields['w_ref_evaluated'] == 'true' for fields in parsed)} "
            f"w_ref_true={sum(fields['w_ref'] == 'true' for fields in parsed)} "
            f"t_ref_true={sum(fields['t_ref'] == 'true' for fields in parsed)} t_ref_false={sum(fields['t_ref'] == 'false' for fields in parsed)} "
            f"t_ref_not_evaluated={sum(fields['t_ref'] == 'not_evaluated' for fields in parsed)} "
            f"states_with_a_disk_with_fewer_than_two_self_verified_slots={sum(fields['disks_with_fewer_than_two_self_verified_slots'] != '0' for fields in parsed)}")


def run_states(arguments: argparse.Namespace) -> int:
    output = ResultFile(arguments.output)
    output.emit(f"name=config mode=states input={arguments.input_label} dumps={arguments.dumps} v2_output={os.path.basename(arguments.v2_output)} "
                f"crc32c_self_check={'pass' if not crc32c_castagnoli.LOAD_TIME_DISAGREEMENTS else 'fail'}")
    state_fields = read_v2_lines(arguments.v2_output, "state")
    version_fields = read_v2_lines(arguments.v2_output, "version")
    emitted = []
    for stream_name in STREAM_NAMES:
        wanted = sorted((fields for fields in state_fields if fields["stream"] == stream_name), key=lambda fields: int(fields["state"]))
        if not wanted:
            continue
        material = load_stream_material(os.path.join(arguments.dumps, stream_name))
        v2_root_writes = {int(fields["index"]): int(fields["root_write"]) for fields in version_fields
                          if fields["stream"] == stream_name and fields.get("root_write", "-").isdigit()}
        rotation_lines(arguments.input_label, stream_name, material, v2_root_writes, output.emit)
        states_by_number = {state.state_number: state for state in material.dump.states}
        for fields in wanted:
            state = states_by_number[int(fields["state"])]
            image = build_crash_image(material.dump, material.prepared_writes, state.landing_vector)
            identity = (f"source=state stream={stream_name} state={state.state_number} batch={state.batch_number} "
                        f"segment={state.segment_number} layer0={state.layer_zero_sequence}")
            line = reference_line(arguments.input_label, identity, fields, image, image.digest() == state.image_digest, material)
            emitted.append(line)
            output.emit(line)
    output.emit(summary_line(arguments.input_label, emitted))
    output.finish()
    return 0


def read_patch_rows(states_path: str, patches_path: str) -> dict[int, tuple[str, str, str, tuple[tuple[int, int, bytes], ...]]]:
    """坏状态表自己读：行号 → (底态落地向量, 底态摘要, 补丁之后的摘要, 补丁 (盘身份, 盘内偏移, 字节))。"""
    with open(patches_path, "rb") as handle:
        blob = handle.read()
    rows = {}
    with open(states_path, encoding="utf-8") as handle:
        for line in handle:
            columns = line.rstrip("\n").split("\t")
            if len(columns) < 5 or not columns[0].isdigit():
                continue
            patches = []
            for entry in columns[4].split(","):
                device, offset, start, length = (int(token) for token in entry.split(":"))
                patches.append((device, offset, blob[start:start + length]))
            rows[int(columns[0])] = (columns[1], columns[2].lower(), columns[3].lower(), tuple(patches))
    return rows


def run_images(arguments: argparse.Namespace) -> int:
    output = ResultFile(arguments.output)
    output.emit(f"name=config mode=images input={arguments.input_label} dumps={arguments.dumps} known_bad_dir={arguments.known_bad_dir or '-'} "
                f"v2_output={os.path.basename(arguments.v2_output)} crc32c_self_check={'pass' if not crc32c_castagnoli.LOAD_TIME_DISAGREEMENTS else 'fail'}")
    image_fields = read_v2_lines(arguments.v2_output, "image_judgement")
    materials, patch_rows, emitted = {}, {}, []
    for stream_name in STREAM_NAMES:
        stream_directory = os.path.join(arguments.dumps, stream_name)
        materials[stream_name] = load_stream_material(stream_directory)
        rotation_lines(arguments.input_label, stream_name, materials[stream_name], {}, output.emit)
        patch_rows[("e170", stream_name)] = read_patch_rows(os.path.join(stream_directory, "bad-states.tsv"),
                                                            os.path.join(stream_directory, "bad-patches.bin"))
    if arguments.known_bad_dir:
        patch_rows[("known_bad", STREAM_NAMES[0])] = read_patch_rows(os.path.join(arguments.known_bad_dir, "known-bad-states.tsv"),
                                                                     os.path.join(arguments.known_bad_dir, "known-bad-patches.bin"))
    for fields in image_fields:
        source, stream_name, row_id = fields["source"], fields["stream"], int(fields["state"])
        material = materials[stream_name]
        if source == "base":
            landing_vector, base_digest, _, _ = patch_rows[("e170", stream_name)][row_id]
            patches, expected_digest = (), base_digest
        else:
            landing_vector, _, expected_digest, patches = patch_rows[(source, stream_name)][row_id]
        image = build_crash_image(material.dump, material.prepared_writes, landing_vector)
        for device, offset, payload in patches:
            image.write_bytes(device, offset, payload)
        identity = f"source={source} stream={stream_name} state={row_id} label={fields.get('label', '-')}"
        line = reference_line(arguments.input_label, identity, fields, image, image.digest() == expected_digest, material)
        emitted.append(line)
        output.emit(line)
    output.emit(summary_line(arguments.input_label, emitted))
    output.finish()
    return 0


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E170 第 2 次重跑第二段：参照谓词 W_ref、T_ref")
    parser.add_argument("mode", choices=["states", "images"])
    parser.add_argument("--dumps", required=True)
    parser.add_argument("--known-bad-dir")
    parser.add_argument("--v2-output", required=True)
    parser.add_argument("--input-label", required=True)
    parser.add_argument("--output", required=True)
    arguments = parser.parse_args(argument_list)
    return run_states(arguments) if arguments.mode == "states" else run_images(arguments)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
