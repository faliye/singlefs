"""读第一段导出装置写的中性格式（登记 5.3），不读任何别的东西。

格式本身不带解析规则与布局分类：哪次写是根槽写、属于哪次发布，由执行器按系统配置的几何自己认（登记 5.2 末段）。
`impl-verdicts.tsv` 与 `bad-*.tsv` 这里一律不打开（登记 5.6：冻结之前写执行器的人不打开）。
"""

from __future__ import annotations

import hashlib
import os
import re
from dataclasses import dataclass

OPERATION_KIND_WRITE = "write"
OPERATION_KIND_WRITE_FORCED_UNIT_ACCESS = "write_fua"
OPERATION_KIND_WRITE_ZEROES = "write_zeroes"
OPERATION_KIND_BARRIER = "barrier"
WRITE_KINDS = (OPERATION_KIND_WRITE, OPERATION_KIND_WRITE_FORCED_UNIT_ACCESS, OPERATION_KIND_WRITE_ZEROES)
BASE_SECTOR_RECORD_BYTES = 4 + 8 + 512
FILES_THE_EXECUTOR_MAY_OPEN = ("manifest.tsv", "base-sectors.bin", "ops.tsv", "ops-payload.bin", "calls.tsv",
                               "torn.tsv", "torn-payload.bin", "states.tsv")


@dataclass(frozen=True)
class RecordedOperation:
    stream_index: int
    device_identity: int
    kind: str
    byte_offset: int
    byte_count: int
    payload: bytes | None  # 普通写与 FUA 写的字节；整段清零与屏障为 None


@dataclass(frozen=True)
class DriverCall:
    call_number: int
    label: str
    first_operation_index: int
    return_point: int  # 调用返回时录制流的长度
    returned_ok: bool


@dataclass(frozen=True)
class SampledState:
    state_number: int
    layer_zero_sequence: int
    segment_number: int
    batch_number: int
    landing_vector: str  # 每次写一个字符：0 没持久、1 撕裂、2 持久，按写序号
    image_digest: str


@dataclass
class StreamDump:
    stream_name: str
    manifest: dict[str, str]
    device_identities: list[int]
    sector_bytes: int
    base_sectors: dict[int, dict[int, bytes]]
    operations: list[RecordedOperation]
    write_operation_indices: list[int]  # 写序号 → 流序下标
    calls: list[DriverCall]
    torn_payload_by_write_ordinal: dict[int, bytes]
    states: list[SampledState]
    count_mismatches: list[str]  # 登记第十一节 S3
    checksum_mismatches: list[str]  # 登记第十一节 V8


def is_header_row(columns: list[str]) -> bool:
    return not columns[0].lstrip("-").isdigit()


def read_tab_separated_rows(path: str) -> list[list[str]]:
    rows = []
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            stripped = line.rstrip("\n")
            if not stripped or stripped.startswith("#"):
                continue
            columns = stripped.split("\t")
            if not rows and is_header_row(columns):
                continue
            rows.append(columns)
    return rows


def sha256_of_file(path: str) -> str:
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def load_stream_dump(stream_directory: str) -> StreamDump:
    manifest = {}
    with open(os.path.join(stream_directory, "manifest.tsv"), encoding="utf-8") as handle:
        for line in handle:
            stripped = line.rstrip("\n")
            if not stripped or stripped.startswith("#") or "\t" not in stripped:
                continue
            key, value = stripped.split("\t", 1)
            manifest[key] = value
    checksum_mismatches = []
    for file_name in FILES_THE_EXECUTOR_MAY_OPEN[1:]:
        recorded = manifest.get("sha256:" + file_name)
        actual = sha256_of_file(os.path.join(stream_directory, file_name))
        if recorded is None or recorded.lower() != actual:
            checksum_mismatches.append(f"{file_name} manifest={recorded} actual={actual}")
    device_identities = [int(token) for token in re.split(r"[ ,;]+", manifest["devices"].strip()) if token]
    sector_bytes = int(manifest["sector_bytes"])
    base_sectors: dict[int, dict[int, bytes]] = {device: {} for device in device_identities}
    with open(os.path.join(stream_directory, "base-sectors.bin"), "rb") as handle:
        raw = handle.read()
    assert len(raw) % BASE_SECTOR_RECORD_BYTES == 0, "base-sectors.bin 不是 524 字节一条"
    for record_start in range(0, len(raw), BASE_SECTOR_RECORD_BYTES):
        device = int.from_bytes(raw[record_start:record_start + 4], "little")
        sector = int.from_bytes(raw[record_start + 4:record_start + 12], "little")
        base_sectors.setdefault(device, {})[sector] = raw[record_start + 12:record_start + BASE_SECTOR_RECORD_BYTES]
    with open(os.path.join(stream_directory, "ops-payload.bin"), "rb") as handle:
        operation_payload = handle.read()
    operations = []
    write_operation_indices = []
    for columns in read_tab_separated_rows(os.path.join(stream_directory, "ops.tsv")):
        stream_index, device, kind, byte_offset, byte_count, payload_start, payload_sha256 = columns[:7]
        assert int(stream_index) == len(operations), "ops.tsv 的流序下标不是从 0 起逐一连续"
        payload = None
        if payload_start != "-":
            start = int(payload_start)
            payload = operation_payload[start:start + int(byte_count)]
            if hashlib.sha256(payload).hexdigest() != payload_sha256.lower():
                checksum_mismatches.append(f"ops.tsv row {stream_index} payload sha256")
        operation = RecordedOperation(int(stream_index), int(device), kind,
                                      0 if byte_offset == "-" else int(byte_offset),
                                      0 if byte_count == "-" else int(byte_count), payload)
        assert kind in WRITE_KINDS or kind == OPERATION_KIND_BARRIER, f"不认得的 op 种类 {kind}"
        if kind in WRITE_KINDS:
            write_operation_indices.append(operation.stream_index)
        operations.append(operation)
    calls = [DriverCall(int(columns[0]), columns[1], int(columns[2]), int(columns[3]), columns[4] == "ok")
             for columns in read_tab_separated_rows(os.path.join(stream_directory, "calls.tsv"))]
    with open(os.path.join(stream_directory, "torn-payload.bin"), "rb") as handle:
        torn_payload = handle.read()
    torn_payload_by_write_ordinal = {}
    for columns in read_tab_separated_rows(os.path.join(stream_directory, "torn.tsv")):
        write_ordinal, start, length, payload_sha256 = int(columns[0]), int(columns[1]), int(columns[2]), columns[3]
        torn_bytes = torn_payload[start:start + length]
        if hashlib.sha256(torn_bytes).hexdigest() != payload_sha256.lower():
            checksum_mismatches.append(f"torn.tsv write {write_ordinal} sha256")
        torn_payload_by_write_ordinal[write_ordinal] = torn_bytes
    states = [SampledState(int(columns[0]), int(columns[1]), int(columns[2]), int(columns[3]), columns[4], columns[5].lower())
              for columns in read_tab_separated_rows(os.path.join(stream_directory, "states.tsv"))]
    count_mismatches = []
    for manifest_key, counted in (("ops", len(operations)), ("writes", len(write_operation_indices)),
                                  ("calls", len(calls)), ("sampled_states", len(states))):
        if int(manifest.get(manifest_key, "-1")) != counted:
            count_mismatches.append(f"{manifest_key} manifest={manifest.get(manifest_key)} counted={counted}")
    for state in states:
        if len(state.landing_vector) != len(write_operation_indices):
            count_mismatches.append(f"state {state.state_number} landing vector length {len(state.landing_vector)}")
    return StreamDump(manifest.get("stream", os.path.basename(stream_directory)), manifest, device_identities,
                      sector_bytes, base_sectors, operations, write_operation_indices, calls,
                      torn_payload_by_write_ordinal, states, count_mismatches, checksum_mismatches)
