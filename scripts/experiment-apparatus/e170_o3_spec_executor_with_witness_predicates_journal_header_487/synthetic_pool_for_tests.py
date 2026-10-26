"""单测用的小池：照 kb 的字节表（layout/01-first-txn.md 一、六、七节与 D22 已定项 7、D23 已定项 4 / 17）手写一条
「取号 → 暖机两次 → 新建一个文件」的写流，按登记 5.3 的格式落成导出目录。

它与执行器是同一个人照同一份 kb 写的，两边一起读错 kb 时单测照样绿；它证的是执行器的各步自洽、判得出改坏，
不证执行器与实现一致（那是 Q2，要第一段的真导出）。magic 逐字写 ASCII 字面（layout/01-first-txn.md 第 109、369、338 行：
SFSB、SFSR、SFSJ），不从执行器的 on_disk_layout 引，这样执行器那几个常量被改坏时单测能红。
"""

from __future__ import annotations

import hashlib
import os
from dataclasses import dataclass, field

import on_disk_layout as layout
from crc32c_castagnoli import crc32c

POOL_IDENTITY = bytes(range(0x40, 0x50))
DEVICES = (0, 1)
SECTOR = 512
SPACING = 4096
ROOT_RING_BASE_BYTES = 64 * 16384
CHUNK_BYTES = 1 << 20
PRIME_STEP = 3
REGION_COUNT = 3
SLOTS_PER_REGION = 8
REGION_DEVICES = (0, 1, 0)
JOURNAL_RING_START_BYTES = 1024 * 16384
JOURNAL_RING_BYTES = 805306368
RECORD_BYTES = 4096
RING_SLOT_COUNT = JOURNAL_RING_BYTES // RECORD_BYTES


def put_field(buffer: bytearray, span: layout.FieldSpan, value: int | bytes) -> None:
    encoded = value if isinstance(value, bytes) else value.to_bytes(span.byte_width, "little")
    assert len(encoded) == span.byte_width, span
    buffer[span.byte_offset:span.end_offset] = encoded


def seal_self_checksum(buffer: bytearray, span: layout.FieldSpan) -> None:
    buffer[span.byte_offset:span.end_offset] = bytes(span.byte_width)
    buffer[span.byte_offset:span.byte_offset + 4] = crc32c(bytes(buffer)).to_bytes(4, "little")


def system_configuration_slot(device: int, generation: int, tail: int, instance: int, rollback_floor: int = 0,
                              overrides: dict | None = None) -> bytes:
    values = {
        "magic": b"SFSB", "format_version": 1, "feature_bits": bytes([0x02]) + bytes(95),
        "file_system_identity": POOL_IDENTITY, "writer_identity": b"singlefs-rs".ljust(16, b"\0") + (1).to_bytes(4, "little"),
        "checksum_algorithm": 1, "own_device_number": device, "device_count": 2, "slot_generation": generation,
        "message_authentication_code_length": 16, "node_size": 16384, "unit_size": 32768, "placement_granularity": 16384,
        "location_entry_width": 14, "physical_block_size_at_format": SECTOR, "journal_ring_start_slot": 1024,
        "journal_ring_bytes": JOURNAL_RING_BYTES, "journal_maximum_record_bytes": RECORD_BYTES,
        "journal_in_flight_record_limit": 65536, "journal_worst_case_bytes": 65536 * 4096, "journal_safety_factor": 3,
        "root_ring_region_count": REGION_COUNT, "root_ring_slots_per_region": SLOTS_PER_REGION,
        "root_ring_prime_step": PRIME_STEP, "root_ring_chunk_bytes": CHUNK_BYTES, "root_ring_base_slot": 64,
        "root_ring_region_device_identities": b"".join(device_id.to_bytes(4, "little") for device_id in REGION_DEVICES),
        "stripe_width_upper_bound": 4, "group_size": 4, "unit_area_start_slot": 50176, "minimum_io_size_at_format": SECTOR,
        "fixed_structure_slot_spacing": SPACING, "checkpoint_time_threshold": 5, "dirty_bytes_threshold": 2 << 30,
        "journal_tail": tail, "journal_instance": instance, "rollback_floor": rollback_floor,
    }
    values.update(overrides or {})
    buffer = bytearray(layout.SYSTEM_CONFIGURATION_SLOT_BYTES)
    for field_name, value in values.items():
        put_field(buffer, layout.SYSTEM_CONFIGURATION_FIELDS[field_name], value)
    seal_self_checksum(buffer, layout.SYSTEM_CONFIGURATION_FIELDS["whole_slot_checksum"])
    return bytes(buffer)


def pointer_bytes(seed: int) -> bytes:
    """86 字节的指针（D19 已定项 11）；单测只要它逐版不同，内容取种子展开。"""
    return bytes((seed * 7 + index) % 251 for index in range(86)) if seed else bytes(86)


@dataclass(frozen=True)
class RootFields:
    instance: int
    txg: int
    tree_table_seed: int
    mapping_seed: int
    watermark: int
    rollback_floor: int = 0
    flags: int = 0
    # 格式补丁（登记 12.5 格四）：新根段多出的两条指针（D23 已定项 15）；默认值写出的根记录字节与补丁之前相同（实例表 99、分配记录树根全零）
    instance_table_seed: int = 99
    allocation_records_seed: int = 0


def root_record(fields: RootFields, pool_identity: bytes = POOL_IDENTITY, magic: bytes = b"SFSR") -> bytes:
    buffer = bytearray(SECTOR)
    spans = layout.ROOT_RECORD_FIELDS
    for field_name, value in (("magic", magic), ("file_system_identity", pool_identity), ("flags", fields.flags),
                              ("instance", fields.instance), ("checkpoint_txg", fields.txg),
                              ("tree_table_pointer", pointer_bytes(fields.tree_table_seed)),
                              ("tree_identifier_watermark", fields.watermark), ("rollback_floor", fields.rollback_floor),
                              ("instance_table_pointer", pointer_bytes(fields.instance_table_seed)),
                              ("allocation_records_root_pointer", pointer_bytes(fields.allocation_records_seed)),
                              ("central_mapping_root_pointer", pointer_bytes(fields.mapping_seed))):
        put_field(buffer, spans[field_name], value)
    seal_self_checksum(buffer, spans["self_checksum"])
    return bytes(buffer)


def root_slot_offset(region: int, slot_in_region: int) -> tuple[int, int]:
    return REGION_DEVICES[region], ROOT_RING_BASE_BYTES + region * PRIME_STEP * CHUNK_BYTES + slot_in_region * SPACING


def root_slot_of_txg(txg: int) -> tuple[int, int]:
    return root_slot_offset(txg % REGION_COUNT, (txg // REGION_COUNT) % SLOTS_PER_REGION)


def unit_bytes(seed: int, length: int) -> bytes:
    return bytes((seed * 31 + index * 13) % 256 for index in range(length))


def named_item(slot_number: int, unit_type_tag: int, content: bytes, txg: int,
               second_copy_content: bytes | None = None) -> bytes:
    """点名项 56 字节（D23 已定项 17）：两条位置条目按设备身份升序，校验和罩整单元。"""
    buffer = bytearray(layout.NAMED_ITEM_BYTES)
    for entry_index, device in enumerate(DEVICES):
        copy = content if device == 0 or second_copy_content is None else second_copy_content
        entry = device.to_bytes(4, "little") + slot_number.to_bytes(6, "little") + crc32c(copy).to_bytes(4, "little")
        start = 14 * entry_index
        buffer[start:start + 14] = entry
    buffer[28] = unit_type_tag
    buffer[29:37] = (65536).to_bytes(8, "little")
    buffer[37:45] = txg.to_bytes(8, "little")
    return bytes(buffer)


@dataclass(frozen=True)
class RecordFields:
    instance: int
    counter: int
    txg: int
    ordinal: int
    is_last_of_publish: bool
    transaction: int
    commit: int
    named_items: tuple[bytes, ...]
    new_root: RootFields
    back_chain: int


def journal_record(fields: RecordFields, pool_identity: bytes = POOL_IDENTITY, magic: bytes = b"SFSJ") -> bytes:
    buffer = bytearray(RECORD_BYTES)
    spans = layout.JOURNAL_RECORD_FIELDS
    item_array = b"".join(fields.named_items)
    for field_name, value in (("magic", magic), ("record_type", 1), ("record_flags", int(fields.is_last_of_publish)),
                              ("record_length", RECORD_BYTES), ("named_item_count", len(fields.named_items)),
                              ("jsn_instance", fields.instance), ("jsn_counter", fields.counter),
                              ("checkpoint_txg", fields.txg), ("transaction_number", fields.transaction),
                              ("commit_marker", fields.commit), ("ordinal_within_publish", fields.ordinal),
                              ("back_chain", fields.back_chain), ("payload_checksum", crc32c(item_array)),
                              ("new_root_tree_table_pointer", pointer_bytes(fields.new_root.tree_table_seed)),
                              ("new_root_central_mapping_root_pointer", pointer_bytes(fields.new_root.mapping_seed)),
                              ("new_root_tree_identifier_watermark", fields.new_root.watermark),
                              ("new_root_rollback_floor", fields.new_root.rollback_floor),
                              # 格式补丁（登记 12.5 格四）：新根段七个字段照 D23 已定项 15；卸载记号取根 flags 的位 0（D22 已定项 7）
                              ("new_root_instance_table_pointer", pointer_bytes(fields.new_root.instance_table_seed)),
                              ("new_root_allocation_records_root_pointer", pointer_bytes(fields.new_root.allocation_records_seed)),
                              ("new_root_unmount_marker", fields.new_root.flags & layout.ROOT_FLAGS_UNMOUNT_MARKER_BIT),
                              ("file_system_identity_low_eight_bytes", pool_identity[:8])):
        put_field(buffer, spans[field_name], value)
    buffer[layout.JOURNAL_HEADER_BYTES:layout.JOURNAL_HEADER_BYTES + len(item_array)] = item_array
    seal_self_checksum(buffer, spans["header_checksum"])
    return bytes(buffer)


# 格式补丁（登记 12.5，单测 U-F3 用）：补丁之前那一版的记录头（311 字节、新根段四个字段、fsid 287、MAC 295、点名项从 311 起），
# 照第 1 次重跑冻结的 on_disk_layout.py 那一张行表逐行抄；只用来造一条「按 311 格式写的记录」，读者照 487 读它必须自证不过。
JOURNAL_RECORD_FIELD_ROWS_OF_THE_311_BYTE_HEADER = [
    ("magic", 4), ("record_type", 2), ("algorithm_type", 1), ("record_flags", 1), ("record_length", 4),
    ("named_item_count", 4), ("jsn_instance", 4), ("jsn_counter", 6), ("checkpoint_txg", 8), ("nonce", 12),
    ("header_checksum", 32), ("transaction_number", 8), ("commit_marker", 1), ("ordinal_within_publish", 4),
    ("back_chain", 4), ("payload_checksum", 4),
    ("new_root_tree_table_pointer", 86), ("new_root_central_mapping_root_pointer", 86),
    ("new_root_tree_identifier_watermark", 8), ("new_root_rollback_floor", 8),
    ("file_system_identity_low_eight_bytes", 8), ("message_authentication_code", 16),
]
JOURNAL_HEADER_BYTES_OF_THE_311_BYTE_HEADER = 311


def journal_record_in_the_311_byte_header_format(fields: RecordFields, pool_identity: bytes = POOL_IDENTITY) -> bytes:
    spans = layout.spans_from_row_order(JOURNAL_RECORD_FIELD_ROWS_OF_THE_311_BYTE_HEADER)
    assert spans["message_authentication_code"].end_offset == JOURNAL_HEADER_BYTES_OF_THE_311_BYTE_HEADER
    buffer = bytearray(RECORD_BYTES)
    item_array = b"".join(fields.named_items)
    for field_name, value in (("magic", b"SFSJ"), ("record_type", 1), ("record_flags", int(fields.is_last_of_publish)),
                              ("record_length", RECORD_BYTES), ("named_item_count", len(fields.named_items)),
                              ("jsn_instance", fields.instance), ("jsn_counter", fields.counter),
                              ("checkpoint_txg", fields.txg), ("transaction_number", fields.transaction),
                              ("commit_marker", fields.commit), ("ordinal_within_publish", fields.ordinal),
                              ("back_chain", fields.back_chain), ("payload_checksum", crc32c(item_array)),
                              ("new_root_tree_table_pointer", pointer_bytes(fields.new_root.tree_table_seed)),
                              ("new_root_central_mapping_root_pointer", pointer_bytes(fields.new_root.mapping_seed)),
                              ("new_root_tree_identifier_watermark", fields.new_root.watermark),
                              ("new_root_rollback_floor", fields.new_root.rollback_floor),
                              ("file_system_identity_low_eight_bytes", pool_identity[:8])):
        put_field(buffer, spans[field_name], value)
    start = JOURNAL_HEADER_BYTES_OF_THE_311_BYTE_HEADER
    buffer[start:start + len(item_array)] = item_array
    seal_self_checksum(buffer, spans["header_checksum"])
    return bytes(buffer)


def chain_value_of(record: bytes) -> int:
    span = layout.JOURNAL_RECORD_FIELDS["header_checksum"]
    header = bytearray(record[:layout.JOURNAL_HEADER_BYTES])
    header[span.byte_offset:span.end_offset] = bytes(span.byte_width)
    return crc32c(bytes(header))


def journal_slot_offset(counter: int) -> int:
    return JOURNAL_RING_START_BYTES + ((counter - 1) % RING_SLOT_COUNT) * RECORD_BYTES


@dataclass
class SyntheticStream:
    base_sectors: dict[int, dict[int, bytes]] = field(default_factory=lambda: {device: {} for device in DEVICES})
    operations: list[tuple[int, str, int, int, bytes | None]] = field(default_factory=list)
    calls: list[tuple[str, int, int]] = field(default_factory=list)
    open_call: tuple[str, int] | None = None

    def seed_base(self, device: int, byte_offset: int, content: bytes) -> None:
        for step in range(0, len(content), SECTOR):
            piece = content[step:step + SECTOR].ljust(SECTOR, b"\0")
            if any(piece):
                self.base_sectors[device][(byte_offset + step) // SECTOR] = piece

    def write(self, device: int, byte_offset: int, content: bytes, forced_unit_access: bool = False) -> None:
        self.operations.append((device, "write_fua" if forced_unit_access else "write", byte_offset, len(content), content))

    def barrier_on_every_device(self) -> None:
        for device in DEVICES:
            self.operations.append((device, "barrier", 0, 0, None))

    def begin_call(self, label: str) -> None:
        self.open_call = (label, len(self.operations))

    def end_call(self) -> None:
        label, first = self.open_call
        self.calls.append((label, first, len(self.operations)))
        self.open_call = None


GENERATION_ZERO_ROOT = RootFields(0, 0, 1, 0, 65536)
DATA_UNIT_SLOT = 50180
NODE_UNIT_SLOT = 50240


def new_pool_file_creation_stream() -> tuple[SyntheticStream, dict]:
    """取号 → 暖机两次（txg 1、2）→ 新建文件（txg 3，点名一个码 1 数据单元与一个码 2 节点）。返回写流与各步的字节，单测改坏时用。"""
    stream = SyntheticStream()
    for device in DEVICES:
        for slot_offset in (0, SPACING):
            stream.seed_base(device, slot_offset, system_configuration_slot(device, 1, 0, 0))
    for region in range(REGION_COUNT):
        device, byte_offset = root_slot_offset(region, 0)
        stream.seed_base(device, byte_offset, root_record(GENERATION_ZERO_ROOT))
    stream.begin_call("acquire")
    for device in DEVICES:
        stream.write(device, 0, system_configuration_slot(device, 2, 0, 1))
    stream.barrier_on_every_device()
    stream.end_call()
    parts: dict = {"records": {}, "roots": {}}
    previous_record = None
    stream.begin_call("warm_up")
    for txg, slot_offset, generation in ((1, SPACING, 3), (2, 0, 4)):
        new_root = RootFields(1, txg, 1, 0, 65536)
        record = journal_record(RecordFields(1, txg, txg, 1, True, 0, 1, (), new_root,
                                             0 if previous_record is None else chain_value_of(previous_record)))
        for device in DEVICES:
            stream.write(device, journal_slot_offset(txg), record)
        stream.barrier_on_every_device()
        root_device, root_offset = root_slot_of_txg(txg)
        stream.write(root_device, root_offset, root_record(new_root), forced_unit_access=True)
        for device in DEVICES:
            stream.write(device, slot_offset, system_configuration_slot(device, generation, txg, 1))
        stream.barrier_on_every_device()
        parts["records"][txg] = record
        parts["roots"][txg] = new_root
        previous_record = record
    stream.end_call()
    stream.begin_call("publish")
    data_unit = unit_bytes(3, 32768)
    node_unit = unit_bytes(5, 16384)
    for device in DEVICES:
        stream.write(device, DATA_UNIT_SLOT * 16384, data_unit)
        stream.write(device, NODE_UNIT_SLOT * 16384, node_unit)
    stream.barrier_on_every_device()
    new_root = RootFields(1, 3, 3, 3, 65548)
    record = journal_record(RecordFields(1, 3, 3, 1, True, 1, 1,
                                         (named_item(DATA_UNIT_SLOT, 1, data_unit, 3), named_item(NODE_UNIT_SLOT, 2, node_unit, 3)),
                                         new_root, chain_value_of(previous_record)))
    for device in DEVICES:
        stream.write(device, journal_slot_offset(3), record)
    stream.barrier_on_every_device()
    root_device, root_offset = root_slot_of_txg(3)
    stream.write(root_device, root_offset, root_record(new_root), forced_unit_access=True)
    for device in DEVICES:
        stream.write(device, SPACING, system_configuration_slot(device, 5, 3, 1))
    stream.barrier_on_every_device()
    stream.end_call()
    parts["records"][3] = record
    parts["roots"][3] = new_root
    parts["data_unit"] = data_unit
    parts["node_unit"] = node_unit
    return stream, parts


def torn_bytes_by_halves(old: bytes, new: bytes) -> bytes:
    """D13 已定项 4：第三态里新旧不同的那一截前一半是新字节、后一半是旧字节（按字节撕）；一半取下整。"""
    differing = [index for index in range(len(new)) if old[index] != new[index]]
    if not differing:
        return new
    first, last = differing[0], differing[-1]
    split = first + (last - first + 1) // 2
    return new[:split] + old[split:]


def read_from_sectors(sectors: dict[int, bytes], byte_offset: int, length: int) -> bytes:
    first = byte_offset // SECTOR
    last = (byte_offset + length + SECTOR - 1) // SECTOR
    joined = b"".join(sectors.get(index, bytes(SECTOR)) for index in range(first, last))
    return joined[byte_offset - first * SECTOR:byte_offset - first * SECTOR + length]


def write_into_sectors(sectors: dict[int, bytes], byte_offset: int, content: bytes) -> None:
    assert byte_offset % SECTOR == 0 and len(content) % SECTOR == 0
    for step in range(0, len(content), SECTOR):
        sectors[(byte_offset + step) // SECTOR] = content[step:step + SECTOR]


def torn_writes_of(stream: SyntheticStream) -> dict[int, bytes]:
    """原地覆写取第三态：固定结构区（单元区之前）、长于一个扇区、罩住的范围里原来有东西（D13 已定项 4）。"""
    sectors = {device: dict(base) for device, base in stream.base_sectors.items()}
    torn = {}
    write_ordinal = 0
    for device, kind, byte_offset, length, payload in stream.operations:
        if kind == "barrier":
            continue
        old = read_from_sectors(sectors[device], byte_offset, length)
        if byte_offset < 50176 * 16384 and length > SECTOR and any(old):
            torn[write_ordinal] = torn_bytes_by_halves(old, payload)
        write_into_sectors(sectors[device], byte_offset, payload)
        write_ordinal += 1
    return torn


def image_digest_of(stream: SyntheticStream, torn: dict[int, bytes], landing_vector: str) -> str:
    sectors = {device: dict(base) for device, base in stream.base_sectors.items()}
    write_ordinal = 0
    for device, kind, byte_offset, length, payload in stream.operations:
        if kind == "barrier":
            continue
        landing = landing_vector[write_ordinal]
        if landing == "2":
            write_into_sectors(sectors[device], byte_offset, payload)
        elif landing == "1":
            write_into_sectors(sectors[device], byte_offset, torn[write_ordinal])
        write_ordinal += 1
    hasher = hashlib.sha256()
    for device in sorted(sectors):
        for sector_index in sorted(sectors[device]):
            content = sectors[device][sector_index]
            if any(content):
                hasher.update(device.to_bytes(4, "little") + sector_index.to_bytes(8, "little") + content)
    return hasher.hexdigest()


def write_count_of(stream: SyntheticStream) -> int:
    return sum(1 for operation in stream.operations if operation[1] != "barrier")


def write_dump(directory: str, stream: SyntheticStream, landing_vectors: list[str], stream_name: str = "synthetic") -> None:
    """按登记 5.3 的文件与列落盘；状态号从 0 起，层 0 序号与段号单测不用，填状态号与 0。"""
    os.makedirs(directory, exist_ok=True)
    torn = torn_writes_of(stream)
    files: dict[str, bytes] = {}
    files["base-sectors.bin"] = b"".join(device.to_bytes(4, "little") + sector.to_bytes(8, "little") + content
                                         for device in sorted(stream.base_sectors)
                                         for sector, content in sorted(stream.base_sectors[device].items()))
    payload = bytearray()
    operation_rows = ["stream_index\tdevice\tkind\toffset\tlength\tpayload_start\tsha256"]
    for stream_index, (device, kind, byte_offset, length, content) in enumerate(stream.operations):
        if content is None:
            operation_rows.append(f"{stream_index}\t{device}\t{kind}\t-\t-\t-\t-")
        else:
            operation_rows.append(f"{stream_index}\t{device}\t{kind}\t{byte_offset}\t{length}\t{len(payload)}\t{hashlib.sha256(content).hexdigest()}")
            payload += content
    files["ops.tsv"] = ("\n".join(operation_rows) + "\n").encode()
    files["ops-payload.bin"] = bytes(payload)
    files["calls.tsv"] = ("call\tlabel\tfirst_op\treturn_point\tresult\n" + "".join(
        f"{index}\t{label}\t{first}\t{returned}\tok\n" for index, (label, first, returned) in enumerate(stream.calls))).encode()
    torn_payload = bytearray()
    torn_rows = ["write\tstart\tlength\tsha256"]
    for write_ordinal, content in sorted(torn.items()):
        torn_rows.append(f"{write_ordinal}\t{len(torn_payload)}\t{len(content)}\t{hashlib.sha256(content).hexdigest()}")
        torn_payload += content
    files["torn.tsv"] = ("\n".join(torn_rows) + "\n").encode()
    files["torn-payload.bin"] = bytes(torn_payload)
    files["states.tsv"] = ("state\tlayer0\tsegment\tbatch\tlanding\tdigest\n" + "".join(
        f"{index}\t{index}\t0\t1\t{vector}\t{image_digest_of(stream, torn, vector)}\n"
        for index, vector in enumerate(landing_vectors))).encode()
    for file_name, content in files.items():
        with open(os.path.join(directory, file_name), "wb") as handle:
            handle.write(content)
    manifest = [("stream", stream_name), ("devices", ",".join(str(device) for device in DEVICES)),
                ("device_bytes", str(4 << 30)), ("sector_bytes", str(SECTOR)), ("ops", str(len(stream.operations))),
                ("writes", str(write_count_of(stream))), ("calls", str(len(stream.calls))),
                ("sampled_states", str(len(landing_vectors)))]
    manifest += [("sha256:" + file_name, hashlib.sha256(content).hexdigest()) for file_name, content in sorted(files.items())]
    with open(os.path.join(directory, "manifest.tsv"), "w", encoding="utf-8") as handle:
        handle.write("".join(f"{key}\t{value}\n" for key, value in manifest))


def new_pool_file_creation_stream_with_rotation_writes_rewritten(rewrite) -> tuple[SyntheticStream, dict]:
    """改五（E170 第 2 次重跑登记 5.2，单测 U-T3–U-T5 用）：照 new_pool_file_creation_stream 建同一条写流，再把每一次系统配置槽写
    （偏移 0 或槽距、4096 字节）交给 rewrite(device, slot_generation, content) 改写：返回新的 4096 字节，返回 None 不改。
    写进去的槽世代号认得出是哪一次写的：取号 2，暖机 txg 1 → 3、txg 2 → 4，新建文件 txg 3 → 5。原来那个函数不动。"""
    stream, parts = new_pool_file_creation_stream()
    rewritten_operations = []
    for device, kind, byte_offset, length, content in stream.operations:
        if kind != "barrier" and byte_offset in (0, SPACING) and length == layout.SYSTEM_CONFIGURATION_SLOT_BYTES:
            generation = layout.little_endian_unsigned(content, layout.SYSTEM_CONFIGURATION_FIELDS["slot_generation"])
            replacement = rewrite(device, generation, content)
            if replacement is not None:
                assert len(replacement) == layout.SYSTEM_CONFIGURATION_SLOT_BYTES
                content = replacement
        rewritten_operations.append((device, kind, byte_offset, length, content))
    stream.operations = rewritten_operations
    return stream, parts
