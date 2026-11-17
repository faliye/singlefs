"""步 9：由写流与调用返回点建版本表、算允许集合 A(c)（登记第一节读法 2、3）。

版本表 = 基镜像上的现行那一版（版本 0，执行器在基镜像上自己恢复得出）加录制流里每一次根槽写按流序各一版。
哪次写是根槽写、哪次是系统配置槽写，执行器按系统配置的几何由 (盘, 偏移) 自己认（登记 5.2 末段），不读导出给的种类。
一次发布的写：D16（发布语义） 已定项 7 的持久顺序「COW 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）→ 系统配置槽 → 屏障」——
根槽写之前、上一次发布之后的非系统配置写归这一次；根槽写之后、下一道屏障之前的系统配置槽写是这一次的轮换；
其余系统配置写（取号、抬 F 先写的那一次）不属于任何一次发布（D16 已定项 7「这一次系统配置写不属于任何一次发布」、D23 已定项 16）。
"""

from __future__ import annotations

import bisect
from dataclasses import dataclass

import on_disk_layout as layout
from crash_dump import OPERATION_KIND_BARRIER, OPERATION_KIND_WRITE_FORCED_UNIT_ACCESS, StreamDump
from crash_image import LANDED_NOT_PERSISTED, PreparedWrite
from root_ring import VersionProjection, projection_of_root_bytes, root_is_self_verified, root_slot_locations
from system_configuration import PoolSystemConfiguration


@dataclass(frozen=True)
class PublishedVersion:
    version_index: int
    projection: VersionProjection
    root_write_stream_index: int | None  # 版本 0 没有
    return_point: int | None  # 发出它的那次调用的返回点；版本 0 记 0；找不到调用记 None（永不算确认）
    call_label: str


@dataclass
class VersionTable:
    versions: list[PublishedVersion]
    owner_version_by_write_ordinal: list[int | None]
    write_stream_indices: list[int]
    anomalies: list[str]


def build_version_table(dump: StreamDump, prepared_writes: list[PreparedWrite], base_pool: PoolSystemConfiguration,
                        version_zero: VersionProjection) -> VersionTable:
    anomalies: list[str] = []
    root_slot_keys = {(location.device_identity, location.byte_offset) for location in root_slot_locations(base_pool)}
    spacing = base_pool.unsigned("fixed_structure_slot_spacing")
    system_configuration_offsets = {0, spacing}
    barrier_indices = [operation.stream_index for operation in dump.operations if operation.kind == OPERATION_KIND_BARRIER]
    versions = [PublishedVersion(0, version_zero, None, 0, "base_image")]
    for prepared in prepared_writes:
        if (prepared.device_identity, prepared.byte_offset) not in root_slot_keys:
            continue
        operation = dump.operations[prepared.stream_index]
        if operation.kind != OPERATION_KIND_WRITE_FORCED_UNIT_ACCESS:
            anomalies.append(f"root_slot_write_not_forced_unit_access stream_index={prepared.stream_index}")
        if not root_is_self_verified(prepared.new_content):
            anomalies.append(f"root_slot_write_payload_not_self_verified stream_index={prepared.stream_index}")
        owning_calls = [call for call in dump.calls
                        if call.first_operation_index <= prepared.stream_index < call.return_point]
        if len(owning_calls) != 1:
            anomalies.append(f"root_slot_write_in_{len(owning_calls)}_calls stream_index={prepared.stream_index}")
        return_point = owning_calls[0].return_point if len(owning_calls) == 1 else None
        label = owning_calls[0].label if len(owning_calls) == 1 else "no_call"
        versions.append(PublishedVersion(len(versions), projection_of_root_bytes(prepared.new_content),
                                         prepared.stream_index, return_point, label))
    root_write_indices = [version.root_write_stream_index for version in versions[1:]]
    owners: list[int | None] = []
    for prepared in prepared_writes:
        is_system_configuration_write = (prepared.byte_offset in system_configuration_offsets
                                         and prepared.byte_count == layout.SYSTEM_CONFIGURATION_SLOT_BYTES)
        if is_system_configuration_write:
            position = bisect.bisect_left(root_write_indices, prepared.stream_index) - 1
            if position < 0:
                owners.append(None)
                continue
            root_index = root_write_indices[position]
            barrier_between = bisect.bisect_left(barrier_indices, prepared.stream_index) - bisect.bisect_right(barrier_indices, root_index)
            owners.append(position + 1 if barrier_between == 0 else None)
        else:
            position = bisect.bisect_left(root_write_indices, prepared.stream_index)
            owners.append(position + 1 if position < len(root_write_indices) else None)
    return VersionTable(versions, owners, [prepared.stream_index for prepared in prepared_writes], anomalies)


@dataclass(frozen=True)
class AllowedVersions:
    largest_landed_write_stream_index: int | None  # m(c)
    acknowledged_version_index: int  # ack(c)
    begun_version_index: int  # begun(c)
    allowed: list[PublishedVersion]


def allowed_versions(table: VersionTable, landing_vector: str) -> AllowedVersions:
    """登记第一节读法 3：A(c) = 版本表里流序落在 [ack(c), begun(c)] 的各版（照字面，begun < ack 时为空）。"""
    landed_ordinals = [ordinal for ordinal, landing in enumerate(landing_vector) if landing != LANDED_NOT_PERSISTED]
    largest_index = max((table.write_stream_indices[ordinal] for ordinal in landed_ordinals), default=None)
    acknowledged = 0
    if largest_index is not None:
        for version in table.versions[1:]:
            if version.return_point is not None and version.return_point <= largest_index:
                acknowledged = version.version_index
    begun_candidates = [table.owner_version_by_write_ordinal[ordinal] for ordinal in landed_ordinals
                        if table.owner_version_by_write_ordinal[ordinal] is not None]
    begun = max(begun_candidates) if begun_candidates else acknowledged
    return AllowedVersions(largest_index, acknowledged, begun, table.versions[acknowledged:begun + 1])
