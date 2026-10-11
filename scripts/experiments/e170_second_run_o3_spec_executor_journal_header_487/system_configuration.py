"""步 2：每盘读两个系统配置槽、自证、择槽；得出池的几何，或按条款整池拒。"""

from __future__ import annotations

from dataclasses import dataclass, field

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


# 槽 1 的偏移从哪取（layout 第 160 行、D22 已定项 16 读槽 1 的偏移那一句的三步，轨迹里记走了哪一步）
SLOT_ONE_ROUTE_OWN_SLOT_ZERO = "1"  # ① 这块盘的槽 0 可用：取它自述的槽距
SLOT_ONE_ROUTE_FIRST_USABLE_SLOT_ZERO_OF_THE_POOL = "2"  # ② 否则取池里按设备次序第一块槽 0 可用的盘自述的槽距
SLOT_ONE_ROUTE_EVERY_SPACING = "3"  # ③ 全池没有可用的槽 0：逐档读
SLOT_ONE_PROBE_FIRST_OFFSET = 4096  # layout 第 160 行「在 4096 到 1 MiB − 4096 之间每 512 字节一档」
SLOT_ONE_PROBE_LAST_OFFSET = FIRST_VERSION_ROOT_RING_BASE_SLOT * layout.PLACEMENT_SLOT_BYTES - SLOT_BYTES  # 1 MiB − 4096


@dataclass(frozen=True)
class SlotOneReading:
    """一块盘的槽 1 是怎么找的（单测 U2–U6 与 Q2-e 的路径计数读它）。"""

    route: str
    spacing_read: int | None  # ① ②：读槽 1 用的槽距；③ 没有
    collected_offsets: tuple[int, ...]  # ③：收到的槽的偏移（自证过、incompat 认得、自述槽距等于所在偏移）
    chosen_offset: int | None  # 这块盘最后择定的「槽 1」在哪（没有就 None）


@dataclass(frozen=True)
class PoolSystemConfiguration:
    chosen_slot_by_device: dict[int, SystemConfigurationSlot]
    geometry_source: SystemConfigurationSlot
    slot_one_reading_by_device: dict[int, SlotOneReading] = field(default_factory=dict)

    def unsigned(self, field_name: str) -> int:
        return self.geometry_source.unsigned(field_name)

    def field_bytes(self, field_name: str) -> bytes:
        return self.geometry_source.field_bytes(field_name)


@dataclass(frozen=True)
class PoolRefusal:
    reason: str


def slot_is_self_verified(raw: bytes) -> bool:
    """D22 已定项 21（整单元校验和 + 被实际检查的世代号）、D18 已定项 17（CRC32C 4 + 28 零，罩整槽 4096 含补齐，字段按零参与）。
    C1：自证过要核 magic = SFSB（.claude/kb/layout/01-first-txn.md 第 109 行；第 160 行「自证过（magic、整槽校验和、…都过）」），
    不等即这一份自证不过。字段偏移按行序（C2，第 158 行）。"""
    if len(raw) != SLOT_BYTES:
        return False
    if raw[:layout.MAGIC_BYTES] != layout.SYSTEM_CONFIGURATION_MAGIC:
        return False
    span = FIELDS["whole_slot_checksum"]
    stored = raw[span.byte_offset:span.end_offset]
    if any(stored[layout.SELF_CHECKSUM_VALUE_BYTES:]):
        return False  # D18 已定项 17「其余 28 字节恒 0，读者遇到非 0 一律判该结构损坏」
    covered = raw[:span.byte_offset] + bytes(layout.SELF_CHECKSUM_FIELD_BYTES) + raw[span.end_offset:]
    return crc32c(covered) == int.from_bytes(stored[:layout.SELF_CHECKSUM_VALUE_BYTES], "little")


def read_self_verified_slot(image: SparseCrashImage, device_identity: int, slot_number: int,
                            byte_offset: int) -> SystemConfigurationSlot | None:
    raw = image.read_bytes(device_identity, byte_offset, SLOT_BYTES)
    if raw is None or not slot_is_self_verified(raw):
        return None
    return SystemConfigurationSlot(device_identity, slot_number, byte_offset, raw)


def incompatible_bits_are_recognised(slot: SystemConfigurationSlot) -> bool:
    return int.from_bytes(slot.field_bytes("feature_bits")[:32], "little") == FIRST_VERSION_INCOMPATIBLE_BITS


def slot_zero_is_usable(slot_zero: SystemConfigurationSlot, gaps: spec_gaps.GapsHitInOneState) -> bool:
    """layout 第 160 行：「可用」的槽 = 自证过（magic、整槽校验和、格式版本、incompat 位、加密类型都过）且几何字段都在这一版读者收的范围里。
    池级字段越界的槽在调用之前已经整池拒了（I-7.13），到这里只剩格式版本、incompat 位、R 与 S 的界这几项判可不可择（G13、G14、G15 照旧记）。"""
    return slot_is_choosable(slot_zero, gaps)


def slot_one_by_trying_every_spacing(image: SparseCrashImage, device_identity: int,
                                     gaps: spec_gaps.GapsHitInOneState) -> tuple[list[SystemConfigurationSlot], PoolRefusal | None]:
    """C3 ③、C4（layout 第 160 行、D22 已定项 16）：全池没有可用的槽 0 时，在 4096 到 1 MiB − 4096 之间每 512 字节一档各读一槽，
    只收自证过、且自述的固定结构槽距等于所在偏移的槽，几档都收得到时取世代号最大的，世代号相同取偏移小的；
    读到自证过而池级字段越界的槽整池拒；incompat 位不认得的槽不可择、不核偏移。
    全为零的扇区不可能是自证过的槽头，只看不全为零的扇区，结果与逐档全读相同。返回 (收到的槽按偏移升序, 整池拒)。"""
    collected: list[SystemConfigurationSlot] = []
    for sector_index in image.nonzero_sector_indices_in(device_identity, SLOT_ONE_PROBE_FIRST_OFFSET, SLOT_ONE_PROBE_LAST_OFFSET + 1):
        candidate_offset = sector_index * SECTOR_BYTES
        candidate = read_self_verified_slot(image, device_identity, 1, candidate_offset)
        if candidate is None:
            continue
        if not incompatible_bits_are_recognised(candidate):
            gaps.record_hit(spec_gaps.GAP_UNRECOGNISED_INCOMPATIBLE_FEATURE_BITS)
            continue  # 不可择、不核偏移
        refusal_reason = pool_level_refusal_of_slot(candidate)
        offset_equals_spacing = candidate.unsigned("fixed_structure_slot_spacing") == candidate_offset
        if refusal_reason is not None:
            if not offset_equals_spacing:
                gaps.record_hit(spec_gaps.GAP_PROBED_SLOT_WITH_OUT_OF_RANGE_FIELD_AND_WRONG_SPACING_REFUSE_OR_IGNORE)
            return collected, PoolRefusal(f"system_configuration_value_refused:{refusal_reason}:device={device_identity}:offset={candidate_offset}")
        if offset_equals_spacing:
            collected.append(candidate)
    return collected, None


def choose_slot_one_among_collected(collected: list[SystemConfigurationSlot]) -> SystemConfigurationSlot | None:
    """③：几档都收得到时取世代号最大的，世代号相同取偏移小的。"""
    if not collected:
        return None
    largest_generation = max(slot.unsigned("slot_generation") for slot in collected)
    return min((slot for slot in collected if slot.unsigned("slot_generation") == largest_generation),
               key=lambda slot: slot.byte_offset)


def spacing_from_first_usable_slot_zero(usable_slot_zero_by_device: dict[int, SystemConfigurationSlot],
                                        device_identities: list[int], gaps: spec_gaps.GapsHitInOneState) -> int:
    """C3 ②：池里按设备次序第一块槽 0 可用的盘自述的槽距。「设备次序」kb 没定义（layout 第 160 行点名的是 `reader.devices()`，
    D19 已定项 11 与 I-2.5 只定了位置条目按设备身份升序）：暂代读法取盘身份升序；另一种读法取导出 manifest 里盘身份的列出次序，
    两种读法取到不同的槽距时记 G21。"""
    first_by_identity = min(usable_slot_zero_by_device)
    first_by_listing = next(device for device in device_identities if device in usable_slot_zero_by_device)
    spacing_by_identity = usable_slot_zero_by_device[first_by_identity].unsigned("fixed_structure_slot_spacing")
    spacing_by_listing = usable_slot_zero_by_device[first_by_listing].unsigned("fixed_structure_slot_spacing")
    if spacing_by_identity != spacing_by_listing:
        gaps.record_hit(spec_gaps.GAP_DEVICE_ORDER_OF_THE_POOL_FOR_THE_SECOND_SLOT_ONE_STEP)
    return spacing_by_identity


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
    全池一槽都择不出才拒。槽 1 的偏移按 C3 的三步取（layout 第 160 行、D22 已定项 16）：先逐盘读槽 0，再按 ① ② ③ 取第一个有的。"""
    slot_zero_by_device: dict[int, SystemConfigurationSlot] = {}
    usable_slot_zero_by_device: dict[int, SystemConfigurationSlot] = {}
    for device_identity in device_identities:
        slot_zero = read_self_verified_slot(image, device_identity, 0, 0)
        if slot_zero is None:
            continue
        refusal_reason = pool_level_refusal_of_slot(slot_zero)
        if refusal_reason is not None:
            return PoolRefusal(f"system_configuration_value_refused:{refusal_reason}:device={device_identity}:slot=0")
        slot_zero_by_device[device_identity] = slot_zero
        if slot_zero_is_usable(slot_zero, gaps):
            usable_slot_zero_by_device[device_identity] = slot_zero
    spacing_of_the_pool_for_the_second_step = (
        spacing_from_first_usable_slot_zero(usable_slot_zero_by_device, device_identities, gaps)
        if usable_slot_zero_by_device and len(usable_slot_zero_by_device) < len(device_identities) else None)
    chosen_slot_by_device: dict[int, SystemConfigurationSlot] = {}
    slot_one_reading_by_device: dict[int, SlotOneReading] = {}
    for device_identity in device_identities:
        verified_slots = [slot_zero_by_device[device_identity]] if device_identity in slot_zero_by_device else []
        if device_identity in usable_slot_zero_by_device:
            spacing = usable_slot_zero_by_device[device_identity].unsigned("fixed_structure_slot_spacing")
            slot_one = read_self_verified_slot(image, device_identity, 1, spacing)
            reading = SlotOneReading(SLOT_ONE_ROUTE_OWN_SLOT_ZERO, spacing, (), None if slot_one is None else spacing)
        elif spacing_of_the_pool_for_the_second_step is not None:
            spacing = spacing_of_the_pool_for_the_second_step
            slot_one = read_self_verified_slot(image, device_identity, 1, spacing)
            reading = SlotOneReading(SLOT_ONE_ROUTE_FIRST_USABLE_SLOT_ZERO_OF_THE_POOL, spacing, (),
                                     None if slot_one is None else spacing)
        else:
            collected, refusal = slot_one_by_trying_every_spacing(image, device_identity, gaps)
            if refusal is not None:
                return refusal
            slot_one = choose_slot_one_among_collected(collected)
            reading = SlotOneReading(SLOT_ONE_ROUTE_EVERY_SPACING, None, tuple(slot.byte_offset for slot in collected),
                                     None if slot_one is None else slot_one.byte_offset)
        slot_one_reading_by_device[device_identity] = reading
        if slot_one is not None:
            refusal_reason = pool_level_refusal_of_slot(slot_one)
            if refusal_reason is not None:
                return PoolRefusal(f"system_configuration_value_refused:{refusal_reason}:device={device_identity}:slot=1")
            verified_slots.append(slot_one)
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
    return PoolSystemConfiguration(chosen_slot_by_device, chosen_slots[0], slot_one_reading_by_device)
