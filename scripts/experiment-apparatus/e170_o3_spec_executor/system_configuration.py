"""步 2：每盘读两个系统配置槽、自证、择槽；得出池的几何，或按条款整池拒。"""

from __future__ import annotations

from dataclasses import dataclass

import on_disk_layout as layout
import spec_gaps
from crash_image import SECTOR_BYTES, SparseCrashImage
from crc32c_castagnoli import crc32c

FIELDS = layout.SYSTEM_CONFIGURATION_FIELDS
SLOT_BYTES = layout.SYSTEM_CONFIGURATION_SLOT_BYTES
FIRST_VERSION_JOURNAL_RING_START_SLOT = 1024  # D22 已定项 16 第 1 句「与第一版常量（64、1024）不等一律整池拒」
FIRST_VERSION_ROOT_RING_BASE_SLOT = 64
MINIMUM_FIXED_STRUCTURE_SLOT_SPACING = 4096  # D22 已定项 2「固定结构槽距 ∈ [4096, 1 MiB − 4096]」
SLOTS_PER_REGION_RANGE = range(4, 17)  # layout 零节「S 住系统配置、区间 4..16」；I-7.13「R 与 S 越界…这一槽不可择」
FIRST_VERSION_REGION_COUNT = 3  # D22 已定项 2「区域数 R | 3」（R 的界 kb 没写，G15）
CLUSTER_SEGMENT_SLOTS = 64  # I-7.13「且是 CLUSTER_SEGMENT_SLOTS（64）的整数倍」
CHECKSUM_ALGORITHM_CRC32C = 1  # I-2.2：0 无效、1 CRC-32C，读到 0 或未登记的码一律拒绝挂载
FIRST_VERSION_INCOMPATIBLE_BITS = 1 << 1  # layout 一「incompat 位 1 = 1 …位 0 退役，其余全 0」
POOL_GEOMETRY_FIELD_NAMES = (
    "file_system_identity", "journal_ring_start_slot", "journal_ring_bytes", "journal_maximum_record_bytes",
    "journal_in_flight_record_limit", "journal_safety_factor", "root_ring_region_count", "root_ring_slots_per_region",
    "root_ring_prime_step", "root_ring_chunk_bytes", "root_ring_base_slot", "root_ring_region_device_identities",
    "unit_area_start_slot", "fixed_structure_slot_spacing", "physical_block_size_at_format",
)


@dataclass(frozen=True)
class SystemConfigurationSlot:
    device_identity: int
    slot_number: int
    byte_offset: int
    raw: bytes

    def unsigned(self, field_name: str) -> int:
        return layout.little_endian_unsigned(self.raw, FIELDS[field_name])

    def field_bytes(self, field_name: str) -> bytes:
        span = FIELDS[field_name]
        return self.raw[span.byte_offset:span.end_offset]


@dataclass(frozen=True)
class PoolSystemConfiguration:
    chosen_slot_by_device: dict[int, SystemConfigurationSlot]
    geometry_source: SystemConfigurationSlot

    def unsigned(self, field_name: str) -> int:
        return self.geometry_source.unsigned(field_name)

    def field_bytes(self, field_name: str) -> bytes:
        return self.geometry_source.field_bytes(field_name)


@dataclass(frozen=True)
class PoolRefusal:
    reason: str


def slot_is_self_verified(raw: bytes, gaps: spec_gaps.GapsHitInOneState) -> bool:
    """D22 已定项 21（整单元校验和 + 被实际检查的世代号）、D18 已定项 17（CRC32C 4 + 28 零，罩整槽 4096 含补齐，字段按零参与）。
    magic 的取值 kb 没写（G01），暂代读法不核 magic；字段偏移按行序（G02）。"""
    gaps.record_hit(spec_gaps.GAP_MAGIC_VALUES_NOT_IN_KB)
    gaps.record_hit(spec_gaps.GAP_FIELD_ORDER_IS_BYTE_ORDER_NOT_WRITTEN)
    if len(raw) != SLOT_BYTES:
        return False
    span = FIELDS["whole_slot_checksum"]
    stored = raw[span.byte_offset:span.end_offset]
    if any(stored[layout.SELF_CHECKSUM_VALUE_BYTES:]):
        return False  # D18 已定项 17「其余 28 字节恒 0，读者遇到非 0 一律判该结构损坏」
    covered = raw[:span.byte_offset] + bytes(layout.SELF_CHECKSUM_FIELD_BYTES) + raw[span.end_offset:]
    return crc32c(covered) == int.from_bytes(stored[:layout.SELF_CHECKSUM_VALUE_BYTES], "little")


def locate_self_verified_slots(image: SparseCrashImage, device_identity: int,
                               gaps: spec_gaps.GapsHitInOneState) -> list[SystemConfigurationSlot]:
    """D22 已定项 16：每盘恒 2 个槽，槽 i 的设备内偏移 = i × 固定结构槽距；槽距住槽里（D2 已定项 19）。"""
    slots = []
    slot_zero_raw = image.read_bytes(device_identity, 0, SLOT_BYTES)
    if slot_zero_raw is not None and slot_is_self_verified(slot_zero_raw, gaps):
        slot_zero = SystemConfigurationSlot(device_identity, 0, 0, slot_zero_raw)
        slots.append(slot_zero)
        spacing = slot_zero.unsigned("fixed_structure_slot_spacing")
        slot_one_raw = image.read_bytes(device_identity, spacing, SLOT_BYTES)
        if spacing >= MINIMUM_FIXED_STRUCTURE_SLOT_SPACING and slot_one_raw is not None \
                and slot_is_self_verified(slot_one_raw, gaps):
            slots.append(SystemConfigurationSlot(device_identity, 1, spacing, slot_one_raw))
        return slots
    # 槽 0 自证不过：槽 1 在哪 kb 没写（G03）。暂代读法：从 4096 起按扇区一档往上找到 1 MiB − 4096，
    # 取第一个自证过、且自述槽距等于它所在偏移的槽。全为零的扇区不可能是自证过的槽头，只看不全为零的扇区。
    gaps.record_hit(spec_gaps.GAP_WHERE_SYSTEM_CONFIGURATION_SLOT_ONE_LIES_WHEN_SLOT_ZERO_IS_UNVERIFIED)
    probe_end = FIRST_VERSION_ROOT_RING_BASE_SLOT * layout.PLACEMENT_SLOT_BYTES - SLOT_BYTES
    for sector_index in image.nonzero_sector_indices_in(device_identity, MINIMUM_FIXED_STRUCTURE_SLOT_SPACING, probe_end + 1):
        candidate_offset = sector_index * SECTOR_BYTES
        candidate_raw = image.read_bytes(device_identity, candidate_offset, SLOT_BYTES)
        if candidate_raw is None or not slot_is_self_verified(candidate_raw, gaps):
            continue
        candidate = SystemConfigurationSlot(device_identity, 1, candidate_offset, candidate_raw)
        if candidate.unsigned("fixed_structure_slot_spacing") == candidate_offset:
            slots.append(candidate)
            break
    return slots


def pool_level_refusal_of_slot(slot: SystemConfigurationSlot) -> str | None:
    """I-7.13（系统配置池级字段在读者收的范围里）与 D22 已定项 2、已定项 16 第 1 句：任一自证过的槽带越界值 ⇒ 整池拒。"""
    if slot.unsigned("checksum_algorithm") != CHECKSUM_ALGORITHM_CRC32C:
        return "checksum_algorithm_not_registered"  # I-2.2
    if slot.unsigned("encryption_type") != 0:
        return "encryption_type_not_off"
    root_ring_base_slot = slot.unsigned("root_ring_base_slot")
    journal_ring_start_slot = slot.unsigned("journal_ring_start_slot")
    if journal_ring_start_slot != FIRST_VERSION_JOURNAL_RING_START_SLOT:
        return "journal_ring_start_not_first_version_slot"
    if root_ring_base_slot != FIRST_VERSION_ROOT_RING_BASE_SLOT:
        return "root_ring_base_not_first_version_slot"
    spacing = slot.unsigned("fixed_structure_slot_spacing")
    if spacing < MINIMUM_FIXED_STRUCTURE_SLOT_SPACING or spacing + SLOT_BYTES > root_ring_base_slot * layout.PLACEMENT_SLOT_BYTES:
        return "fixed_structure_slot_spacing_out_of_range"
    physical_block_size = slot.unsigned("physical_block_size_at_format")
    if not layout.ROOT_RECORD_BYTES <= physical_block_size <= spacing:
        return "physical_block_size_outside_root_slot_bounds"
    ring_bytes = slot.unsigned("journal_ring_bytes")
    safety_factor = slot.unsigned("journal_safety_factor")
    record_bytes = slot.unsigned("journal_maximum_record_bytes")
    if safety_factor == 0 or record_bytes == 0 or ring_bytes // record_bytes // safety_factor < 1:
        return "journal_ring_bytes_outside_supported_range"
    unit_area_start_slot = slot.unsigned("unit_area_start_slot")
    if journal_ring_start_slot * layout.PLACEMENT_SLOT_BYTES + ring_bytes > unit_area_start_slot * layout.PLACEMENT_SLOT_BYTES:
        return "journal_ring_end_beyond_unit_area_start"
    slots_after_ring = journal_ring_start_slot + -(-ring_bytes // layout.PLACEMENT_SLOT_BYTES)
    if unit_area_start_slot != slots_after_ring or unit_area_start_slot % CLUSTER_SEGMENT_SLOTS != 0:
        return "unit_area_start_not_the_segment_aligned_slot_after_the_ring"
    return None


def slot_is_choosable(slot: SystemConfigurationSlot, gaps: spec_gaps.GapsHitInOneState) -> bool:
    """自证过之后还要「可择」：格式版本 1（D22 已定项 9；拒槽还是拒池两种读法，G13）、incompat 位认得（G14）、
    R 与 S 在界内（I-7.13「仍归「这一槽不可择」」；R 的界 kb 没写，G15）。"""
    choosable = True
    if slot.unsigned("format_version") != 1:
        gaps.record_hit(spec_gaps.GAP_SYSTEM_CONFIGURATION_FORMAT_VERSION_SLOT_OR_POOL_REFUSAL)
        choosable = False
    incompatible_bits = int.from_bytes(slot.field_bytes("feature_bits")[:32], "little")
    if incompatible_bits != FIRST_VERSION_INCOMPATIBLE_BITS:
        gaps.record_hit(spec_gaps.GAP_UNRECOGNISED_INCOMPATIBLE_FEATURE_BITS)
        choosable = False
    if slot.unsigned("root_ring_region_count") != FIRST_VERSION_REGION_COUNT:
        gaps.record_hit(spec_gaps.GAP_REGION_COUNT_BOUNDS)
        choosable = False
    if slot.unsigned("root_ring_slots_per_region") not in SLOTS_PER_REGION_RANGE:
        choosable = False
    return choosable


def choose_system_configuration(image: SparseCrashImage, device_identities: list[int],
                                gaps: spec_gaps.GapsHitInOneState) -> PoolSystemConfiguration | PoolRefusal:
    """择槽：逐盘取校验和过且世代号最大的（D22 已定项 16）。单盘两槽都不可择不拒池（I-8.5「单盘全废不判违例」），
    全池一槽都择不出才拒。"""
    chosen_slot_by_device: dict[int, SystemConfigurationSlot] = {}
    for device_identity in device_identities:
        verified_slots = locate_self_verified_slots(image, device_identity, gaps)
        for slot in verified_slots:
            refusal_reason = pool_level_refusal_of_slot(slot)
            if refusal_reason is not None:
                return PoolRefusal(f"system_configuration_value_refused:{refusal_reason}:device={device_identity}:slot={slot.slot_number}")
        choosable_slots = [slot for slot in verified_slots if slot_is_choosable(slot, gaps)]
        if not choosable_slots:
            continue
        largest_generation = max(slot.unsigned("slot_generation") for slot in choosable_slots)
        tied = [slot for slot in choosable_slots if slot.unsigned("slot_generation") == largest_generation]
        if len({slot.raw for slot in tied}) > 1:
            gaps.record_hit(spec_gaps.GAP_TWO_SYSTEM_CONFIGURATION_SLOTS_TIE_ON_GENERATION_WITH_DIFFERENT_CONTENT)
        chosen_slot_by_device[device_identity] = min(tied, key=lambda slot: slot.slot_number)
    if not chosen_slot_by_device:
        return PoolRefusal("no_choosable_system_configuration_on_any_device")
    chosen_slots = [chosen_slot_by_device[device] for device in sorted(chosen_slot_by_device)]
    if len({slot.unsigned("device_count") for slot in chosen_slots}) > 1:
        return PoolRefusal("system_configurations_disagree_on_device_count")  # I-7.15
    if any(len({slot.field_bytes(name) for slot in chosen_slots}) > 1 for name in POOL_GEOMETRY_FIELD_NAMES):
        gaps.record_hit(spec_gaps.GAP_DEVICES_CHOSE_SYSTEM_CONFIGURATIONS_WITH_DIFFERENT_POOL_GEOMETRY)
        return PoolRefusal("system_configurations_disagree_on_pool_geometry")  # G04 的暂代读法：整池拒
    return PoolSystemConfiguration(chosen_slot_by_device, chosen_slots[0])
