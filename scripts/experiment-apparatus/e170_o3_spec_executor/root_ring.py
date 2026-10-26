"""步 3、步 4：由系统配置得根环几何，读全部根槽、逐个自证，再在有效者里择新。

.claude/kb/decisions/22-单元原子性怎么合成.md@已定项 24：根落在同一类，且两类布局需要的是同一种机制——
「必须先逐个验证全部候选、再在有效者中按代号择新」。
"""

from __future__ import annotations

from dataclasses import dataclass

import on_disk_layout as layout
import spec_gaps
from crash_image import SparseCrashImage
from crc32c_castagnoli import crc32c
from system_configuration import PoolSystemConfiguration

FIELDS = layout.ROOT_RECORD_FIELDS


@dataclass(frozen=True)
class VersionProjection:
    """登记第一节读法 2：一版 = (实例代号, checkpoint_txg, 树表单元指针 86, 中央映射树根指针 86, 树 ID 水位, 回退下界 F)。
    卸载记号与根记录其余字段不进投影（D13 已定项 1 M2「其余字段等于所选根，卸载记号除外」）。"""

    instance: int
    checkpoint_txg: int
    tree_table_pointer: bytes
    central_mapping_root_pointer: bytes
    tree_identifier_watermark: int
    rollback_floor: int

    def short_text(self) -> str:
        return f"({self.instance},{self.checkpoint_txg},wm={self.tree_identifier_watermark},floor={self.rollback_floor})"


@dataclass(frozen=True)
class RootSlotLocation:
    region: int
    slot_in_region: int
    device_identity: int
    byte_offset: int


@dataclass(frozen=True)
class SelfVerifiedRoot:
    location: RootSlotLocation
    raw: bytes

    def unsigned(self, field_name: str) -> int:
        return layout.little_endian_unsigned(self.raw, FIELDS[field_name])

    def field_bytes(self, field_name: str) -> bytes:
        span = FIELDS[field_name]
        return self.raw[span.byte_offset:span.end_offset]

    @property
    def choice_key(self) -> tuple[int, int]:
        # D22 已定项 7 字段表「实例代号」一行：择新在 checkpoint_txg 平局时按它高者赢；D23 已定项 14 注 2「择根照旧 txg 为主」
        return (self.unsigned("checkpoint_txg"), self.unsigned("instance"))

    def projection(self) -> VersionProjection:
        return projection_of_root_bytes(self.raw)


def projection_of_root_bytes(raw: bytes) -> VersionProjection:
    def unsigned(field_name: str) -> int:
        return layout.little_endian_unsigned(raw, FIELDS[field_name])

    def field_bytes(field_name: str) -> bytes:
        span = FIELDS[field_name]
        return raw[span.byte_offset:span.end_offset]

    return VersionProjection(unsigned("instance"), unsigned("checkpoint_txg"), field_bytes("tree_table_pointer"),
                             field_bytes("central_mapping_root_pointer"), unsigned("tree_identifier_watermark"),
                             unsigned("rollback_floor"))


def root_slot_locations(pool: PoolSystemConfiguration) -> list[RootSlotLocation]:
    """D22 已定项 16：区域 r 的起点 = 基址 + r × P × chunk，基址 = 根环起点字段 × 16384；根槽序号 j 的偏移 = 区域起点 + j × 固定结构槽距；
    区域归属取系统配置「根环逐区域设备身份」（D2 已定项 7，每区域 4 字节）。"""
    base_bytes = pool.unsigned("root_ring_base_slot") * layout.PLACEMENT_SLOT_BYTES
    prime_step = pool.unsigned("root_ring_prime_step")
    chunk_bytes = pool.unsigned("root_ring_chunk_bytes")
    spacing = pool.unsigned("fixed_structure_slot_spacing")
    region_identities = pool.field_bytes("root_ring_region_device_identities")
    locations = []
    for region in range(pool.unsigned("root_ring_region_count")):
        device_identity = int.from_bytes(region_identities[region * 4:(region + 1) * 4], "little")
        region_start = base_bytes + region * prime_step * chunk_bytes
        for slot_in_region in range(pool.unsigned("root_ring_slots_per_region")):
            locations.append(RootSlotLocation(region, slot_in_region, device_identity, region_start + slot_in_region * spacing))
    return locations


def root_slot_bytes(pool: PoolSystemConfiguration, probed_sector_bytes: int, gaps: spec_gaps.GapsHitInOneState) -> int:
    """D22 已定项 2：根槽宽 = 挂载时探测的 physical_block_size；执行器探不了设备，取系统配置记的 mkfs 时取值（G18）。"""
    recorded = pool.unsigned("physical_block_size_at_format")
    if recorded != probed_sector_bytes:
        gaps.record_hit(spec_gaps.GAP_ROOT_SLOT_WIDTH_RECORDED_VERSUS_PROBED)
    return recorded


def root_is_self_verified(raw: bytes, gaps: spec_gaps.GapsHitInOneState) -> bool:
    """D22 已定项 24（整单元校验和 + 代号）；D18 已定项 17（罩整槽含补齐、字段按零参与、其余 28 字节恒 0）；
    D22 已定项 7 flags 一行（其余位第一版恒 0、非 0 拒收）。magic 取值 kb 没写（G01），不核。"""
    gaps.record_hit(spec_gaps.GAP_MAGIC_VALUES_NOT_IN_KB)
    if len(raw) < layout.ROOT_RECORD_BYTES:
        return False
    span = FIELDS["self_checksum"]
    stored = raw[span.byte_offset:span.end_offset]
    if any(stored[layout.SELF_CHECKSUM_VALUE_BYTES:]):
        return False
    covered = raw[:span.byte_offset] + bytes(layout.SELF_CHECKSUM_FIELD_BYTES) + raw[span.end_offset:]
    if crc32c(covered) != int.from_bytes(stored[:layout.SELF_CHECKSUM_VALUE_BYTES], "little"):
        return False
    return layout.little_endian_unsigned(raw, FIELDS["flags"]) & ~layout.ROOT_FLAGS_UNMOUNT_MARKER_BIT == 0


@dataclass(frozen=True)
class RootRingReading:
    self_verified_roots: list[SelfVerifiedRoot]
    largest_txg_in_any_nonzero_root_slot: int | None  # Q2-e 第二项要的「盘上 txg 最大那条」，不论自证过没有


def read_root_ring(image: SparseCrashImage, pool: PoolSystemConfiguration, probed_sector_bytes: int,
                   gaps: spec_gaps.GapsHitInOneState) -> RootRingReading:
    slot_bytes = root_slot_bytes(pool, probed_sector_bytes, gaps)
    verified = []
    largest_raw_txg = None
    for location in root_slot_locations(pool):
        raw = image.read_bytes(location.device_identity, location.byte_offset, slot_bytes)
        if raw is None or not any(raw):
            continue  # 全为零的槽自证不过（A10：512 个零字节的 CRC-32C ≠ 0）
        raw_txg = layout.little_endian_unsigned(raw, FIELDS["checkpoint_txg"])
        largest_raw_txg = raw_txg if largest_raw_txg is None else max(largest_raw_txg, raw_txg)
        if root_is_self_verified(raw, gaps):
            verified.append(SelfVerifiedRoot(location, raw))
    return RootRingReading(verified, largest_raw_txg)


def root_sits_in_the_slot_its_txg_gives(root: SelfVerifiedRoot, pool: PoolSystemConfiguration) -> bool:
    """I-7.17（根槽位置等于 txg 算出的位置）的字面：txg ≥ 1 时区域 = txg mod R、槽 = (txg div R) mod S；txg = 0 只判槽 0。"""
    txg = root.unsigned("checkpoint_txg")
    region_count = pool.unsigned("root_ring_region_count")
    slots_per_region = pool.unsigned("root_ring_slots_per_region")
    if txg == 0:
        return root.location.slot_in_region == 0
    return root.location.region == txg % region_count and root.location.slot_in_region == (txg // region_count) % slots_per_region


def pick_largest(candidates: list[SelfVerifiedRoot]) -> SelfVerifiedRoot | None:
    if not candidates:
        return None
    largest_key = max(candidate.choice_key for candidate in candidates)
    tied = [candidate for candidate in candidates if candidate.choice_key == largest_key]
    return min(tied, key=lambda candidate: (candidate.location.region, candidate.location.slot_in_region))


def choose_root(reading: RootRingReading, pool: PoolSystemConfiguration,
                gaps: spec_gaps.GapsHitInOneState) -> SelfVerifiedRoot | None:
    """步 4。暂代读法：不核根所在槽位（G05 取 D22 已定项 24 的字面）、fsid 与本池不同的根不算候选（G06）；
    两种读法择到的根不同时记撞上。最大键上两条根投影不同时取 (区域, 槽) 最小的那条（G16）。"""
    pool_identity = pool.field_bytes("file_system_identity")
    own_pool_roots = [root for root in reading.self_verified_roots if root.field_bytes("file_system_identity") == pool_identity]
    provisional_choice = pick_largest(own_pool_roots)
    with_slot_check = pick_largest([root for root in own_pool_roots if root_sits_in_the_slot_its_txg_gives(root, pool)])
    if with_slot_check != provisional_choice:
        gaps.record_hit(spec_gaps.GAP_READER_CHECKS_ROOT_SLOT_AGAINST_ITS_TXG)
    without_identity_check = pick_largest(reading.self_verified_roots)
    if without_identity_check != provisional_choice:
        gaps.record_hit(spec_gaps.GAP_ROOT_WITH_FOREIGN_FILE_SYSTEM_IDENTITY)
    if provisional_choice is not None:
        tied = [root for root in own_pool_roots if root.choice_key == provisional_choice.choice_key]
        if len({root.projection() for root in tied}) > 1:
            gaps.record_hit(spec_gaps.GAP_TWO_ROOTS_SHARE_THE_LARGEST_KEY_WITH_DIFFERENT_PROJECTIONS)
    return provisional_choice
