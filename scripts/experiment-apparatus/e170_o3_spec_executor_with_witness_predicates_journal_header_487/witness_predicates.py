"""改一（E170 第 2 次重跑登记 5.2）：谓词 W（登记读法 2）、τ 与谓词 T（登记读法 3）。

W 的依据：.claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换
——「可写挂载读到的更新状态读不出」那一段：「不是判不出时取全部自证过的槽 journal tail 的最大值 c_见证，0 ⇒ 不判；所选那一版那次发布带末条标志的
记录读得出（任一份）⇒ c_见证 > 它的计数器即为真；读不出而计数器等于 c_见证 的记录读得出 ⇒ 它的 (实例代号, checkpoint_txg) 大于所选那一版即为真；
两条都读不出按真」。「判不出」那一支（判不出取 R2：.claude/kb/decisions/22-单元原子性怎么合成.md@已定项 8：固定结构的放置 射程「判不出 ⟺ 有槽设备报读错，
且没有一块盘两槽都自证过」）要有设备报读错；O3 的输入里没有读故障，这一支在每一态上都不成立，不判（登记读法 2）。重读一次（R = 1）在无读故障的镜像上
读到同样的字节，W 取第一遍的值。
T 的依据：.claude/kb/decisions/16-发布语义.md@已定项 7：发布的持久顺序（「根槽（FUA）→ 系统配置槽 → 屏障；fsync 等系统配置轮换持久之后才返回」）；
.claude/kb/decisions/23-journal的角色与格式.md@已定项 18：在飞记录数上限的取值与语义，tail 字段的编码（tail 存 jsn 的 48 位计数器）；
.claude/kb/layout/01-first-txn.md 第 152 行（tail：mkfs 0，暖机后 1、2，新池新建文件发布后 3）、第 158 行（journal tail 在偏移 469）；
ack(c) 取 v1 的 version_table.allowed_versions 算出的那一个（原登记读法 3），不另算。

读系统配置槽、扫记录、认自证都调 v1 已有的函数（system_configuration、journal_ring、replay_prefix），不改它们。
只在 v1 给出了落点 L(c) ≠ 无 的态上算（登记读法 2、4）；L(c) = 无的态 W、T 都记 not_evaluated。
"""

from __future__ import annotations

from dataclasses import dataclass

import on_disk_layout as layout
from crash_image import SparseCrashImage
from journal_ring import JournalRecord, RingScan
from replay_prefix import flagged_record_is_followed_within_its_publish
from root_ring import VersionProjection
from system_configuration import PoolSystemConfiguration, SystemConfigurationSlot, incompatible_bits_are_recognised, \
    read_self_verified_slot, slot_is_self_verified

BRANCH_NOTHING = "nothing"
BRANCH_LAST_RECORD = "last_record"
BRANCH_RECORD_AT_COUNTER = "record_at_counter"
BRANCH_BOTH_UNREADABLE = "both_unreadable"
BRANCH_NOT_EVALUATED = "not_evaluated"
TAIL_CHECK_TRUE = "true"
TAIL_CHECK_FALSE = "false"
TAIL_CHECK_NOT_EVALUATED = "not_evaluated"
FIRST_VERSION_FORMAT_VERSION = 1  # D22 已定项 9「格式版本不是 1，一律拒收那一槽」
ENCRYPTION_TYPE_UNENCRYPTED = 0  # layout/01-first-txn.md 第 122 行「加密类型 … 0（未加密）」


def slot_passes_every_check_of_self_verification(slot: SystemConfigurationSlot) -> bool:
    """layout/01-first-txn.md 第 160 行：「自证过（magic、整槽校验和、格式版本、incompat 位、加密类型都过）」。
    magic 与整槽校验和在调用之前已由 v1 的 read_self_verified_slot / slot_is_self_verified 核过；这里核其余三样：
    格式版本 1、incompat 位是第一版那一组（v1 的 incompatible_bits_are_recognised）、加密类型 0。"""
    return (slot.unsigned("format_version") == FIRST_VERSION_FORMAT_VERSION and incompatible_bits_are_recognised(slot)
            and slot.unsigned("encryption_type") == ENCRYPTION_TYPE_UNENCRYPTED)


def self_verified_system_configuration_slots_of_every_device(image: SparseCrashImage,
                                                             pool: PoolSystemConfiguration) -> list[SystemConfigurationSlot]:
    """D23 已定项 14「读池里每块盘两槽的系统配置」：槽 0 在设备内偏移 0；槽 1 在 v1 择系统配置时照 ① ② ③ 定下的偏移
    （layout/01-first-txn.md 第 160 行；① ② 是读槽 1 用的槽距，③ 是逐档择定的那一槽，一槽都没收到就只有槽 0）。"""
    slots = []
    for witnessed_device in sorted(pool.slot_one_reading_by_device):
        reading = pool.slot_one_reading_by_device[witnessed_device]
        slot_one_offset = reading.chosen_offset if reading.chosen_offset is not None else reading.spacing_read
        offsets_by_slot_number = [(0, 0)] + ([] if slot_one_offset is None else [(1, slot_one_offset)])
        for slot_number, byte_offset in offsets_by_slot_number:
            slot = read_self_verified_slot(image, witnessed_device, slot_number, byte_offset)
            if slot is not None and slot_passes_every_check_of_self_verification(slot):
                slots.append(slot)
    return slots


def witness_value_of(slots: list[SystemConfigurationSlot]) -> int:
    """c_见证 = 全部自证过的槽 journal tail（偏移 469，layout 第 158 行）的最大值；一份都没有时取 0（「0 ⇒ 不判」那一支）。"""
    return max((slot.unsigned("journal_tail") for slot in slots), default=0)


def instance_major_key(instance: int, checkpoint_txg: int) -> tuple[int, int]:
    """D23 已定项 14 注 2：「记录水位 (实例代号, checkpoint_txg) 按实例代号为主比，与 jsn 同序」。
    W 第二支比的就是一条记录的 (实例代号, checkpoint_txg) 与所选那一版的，照这一句按实例代号为主比。"""
    return (instance, checkpoint_txg)


@dataclass(frozen=True)
class WitnessBranchOutcome:
    branch: str
    compared: str  # last_record：比的计数器；record_at_counter：那条记录的 (实例代号,checkpoint_txg)；其余 -
    witness_true: bool


def record_is_readable(record: JournalRecord, scan: RingScan) -> bool:
    """「读得出（任一份）」= 环里任一份自证过、落在自己的槽上（v1 的 RingScan.records_by_counter），且不是 D23 已定项 4 第四格
    「带末条标志的那条之后还有同发布的记录 ⇒ 当损坏」的那一条（v1 的 flagged_record_is_followed_within_its_publish）。"""
    return not flagged_record_is_followed_within_its_publish(record, scan)


def witness_predicate(witness_counter: int, landed: VersionProjection, scan: RingScan) -> WitnessBranchOutcome:
    if witness_counter == 0:
        return WitnessBranchOutcome(BRANCH_NOTHING, "-", False)  # 「0 ⇒ 不判」：W 假
    selected_key = instance_major_key(landed.instance, landed.checkpoint_txg)
    flagged_records = [record for record in scan.records_by_counter.values()
                       if record.is_last_of_publish and instance_major_key(*record.watermark) == selected_key
                       and record_is_readable(record, scan)]
    if flagged_records:
        # 同一 (实例代号, checkpoint_txg) 两条带标志时 v1 已在择根与取前缀里整池拒（G20），L ≠ 无的态上至多一条；取计数器最大的只为写全。
        flagged_record_counter = max(record.unsigned("jsn_counter") for record in flagged_records)
        return WitnessBranchOutcome(BRANCH_LAST_RECORD, str(flagged_record_counter), witness_counter > flagged_record_counter)
    record_at_counter = scan.records_by_counter.get(witness_counter)
    if record_at_counter is not None and record_is_readable(record_at_counter, scan):
        record_key = instance_major_key(record_at_counter.unsigned("jsn_instance"), record_at_counter.unsigned("checkpoint_txg"))
        return WitnessBranchOutcome(BRANCH_RECORD_AT_COUNTER, f"({record_key[0]},{record_key[1]})", record_key > selected_key)
    return WitnessBranchOutcome(BRANCH_BOTH_UNREADABLE, "-", True)  # 「两条都读不出按真」


@dataclass(frozen=True)
class VersionRotation:
    """录制流里一版那次发布的系统配置轮换写（登记读法 3）与 Q3-f 要的那一版末条记录的计数器。"""

    version_index: int
    rotation_write_ordinals: tuple[int, ...]  # 根槽 FUA 写之后、其后第一道屏障之前落在系统配置槽偏移上的写
    self_verified_rotation_tails: tuple[int, ...]  # 其中写进去的 4096 字节整槽自证过的那几份所带的 journal tail，按写序
    rotation_tail: int | None  # τ：上一列的最大值；一份自证过的都没有时 None（T 不判）
    last_record_counter: int | None  # 录制流里这一版带末条标志、(实例代号, checkpoint_txg) 等于这一版的记录的计数器；没有时 None


def rotation_write_ordinals_of_version(version_table, prepared_writes, base_pool: PoolSystemConfiguration,
                                       version_index: int) -> tuple[int, ...]:
    """v1 的 version_table.build_version_table 已按 D16 已定项 7 的持久顺序给每次写记了属于哪一版：系统配置槽写（偏移 0 或槽距、4096 字节）
    归到它之前最近的那次根槽写、且两者之间没有屏障——正是「那一版的根槽 FUA 写之后、其后第一道屏障之前、落在系统配置槽偏移上的那几次写」。"""
    spacing = base_pool.unsigned("fixed_structure_slot_spacing")
    return tuple(prepared.write_ordinal for prepared in prepared_writes
                 if version_table.owner_version_by_write_ordinal[prepared.write_ordinal] == version_index
                 and prepared.byte_offset in (0, spacing) and prepared.byte_count == layout.SYSTEM_CONFIGURATION_SLOT_BYTES)


def last_record_counter_of_version(version_table, prepared_writes, base_pool: PoolSystemConfiguration, version_index: int) -> int | None:
    """Q3-f（附带）：这一版的写里落在 journal 环上、magic 对、带末条标志、(实例代号, checkpoint_txg) 等于这一版投影的记录的计数器。"""
    projection = version_table.versions[version_index].projection
    ring_start = base_pool.unsigned("journal_ring_start_slot") * layout.PLACEMENT_SLOT_BYTES
    record_bytes = base_pool.unsigned("journal_maximum_record_bytes")
    ring_end = ring_start + base_pool.unsigned("journal_ring_bytes")
    counters = []
    for prepared in prepared_writes:
        if version_table.owner_version_by_write_ordinal[prepared.write_ordinal] != version_index:
            continue
        if not (ring_start <= prepared.byte_offset < ring_end and prepared.byte_count == record_bytes):
            continue
        if prepared.new_content[:layout.MAGIC_BYTES] != layout.JOURNAL_RECORD_MAGIC:
            continue
        record = JournalRecord((prepared.byte_offset - ring_start) // record_bytes, prepared.new_content)
        if record.is_last_of_publish and record.watermark == (projection.instance, projection.checkpoint_txg):
            counters.append(record.unsigned("jsn_counter"))
    return max(counters, default=None)


def rotation_of_version(version_table, prepared_writes, base_pool: PoolSystemConfiguration, version_index: int) -> VersionRotation:
    ordinals = rotation_write_ordinals_of_version(version_table, prepared_writes, base_pool, version_index)
    self_verified_tails = []
    for write_ordinal in ordinals:
        prepared = prepared_writes[write_ordinal]
        if not slot_is_self_verified(prepared.new_content):
            continue
        slot_number = 0 if prepared.byte_offset == 0 else 1
        slot = SystemConfigurationSlot(prepared.device_identity, slot_number, prepared.byte_offset, prepared.new_content)
        if slot_passes_every_check_of_self_verification(slot):
            self_verified_tails.append(slot.unsigned("journal_tail"))
    self_verified_tails = tuple(self_verified_tails)
    last_record_counter = last_record_counter_of_version(version_table, prepared_writes, base_pool, version_index)
    return VersionRotation(version_index, ordinals, self_verified_tails,
                           rotation_tail=max(self_verified_tails) if self_verified_tails else None,
                           last_record_counter=last_record_counter)


rotations_by_version_table: dict[int, tuple[object, list[VersionRotation]]] = {}


def rotations_of_every_version(context) -> list[VersionRotation]:
    """按版本下标排的每一版的轮换写（版本 0 在基镜像上，录制流里没有它的轮换写，那一格是空的）。同一条流只算一次；
    缓存连同版本表对象本身一起存，取的时候核是同一个对象（id 会被回收后的新对象复用）。"""
    key = id(context.version_table)
    cached = rotations_by_version_table.get(key)
    if cached is None or cached[0] is not context.version_table:
        rotations = [rotation_of_version(context.version_table, context.prepared_writes, context.base_pool, version_index)
                     for version_index in range(len(context.version_table.versions))]
        cached = (context.version_table, rotations)
        rotations_by_version_table[key] = cached
    return cached[1]


@dataclass(frozen=True)
class WitnessEvaluation:
    witness_counter: int | None  # c_见证；None = 没算（L(c) = 无）
    witness_branch: str
    witness_compared: str
    witness_true: bool | None
    ack_rotation_tail: int | None  # τ(c)
    tail_check: str

    @property
    def failed_predicates(self) -> str:
        failed = [name for name, has_failed in (("W", self.witness_true is True), ("T", self.tail_check == TAIL_CHECK_FALSE)) if has_failed]
        return ",".join(failed) or "-"


NOT_EVALUATED = WitnessEvaluation(None, BRANCH_NOT_EVALUATED, "-", None, None, TAIL_CHECK_NOT_EVALUATED)


def evaluate_witness_predicates(context, image: SparseCrashImage, recovery, allowed) -> WitnessEvaluation:
    """context：executor_core.StreamContext；recovery：executor_core.Recovery；allowed：version_table.AllowedVersions。"""
    if recovery.landed is None or recovery.pool is None or recovery.ring_scan is None:
        return NOT_EVALUATED
    witness_counter = witness_value_of(self_verified_system_configuration_slots_of_every_device(image, recovery.pool))
    branch_outcome = witness_predicate(witness_counter, recovery.landed, recovery.ring_scan)
    witness_true = branch_outcome.witness_true
    acknowledged = allowed.acknowledged_version_index
    rotation_tail = None
    tail_check = TAIL_CHECK_NOT_EVALUATED  # ack(c) 是版本 0（录制流里没有它的轮换写），或那次发布的轮换写一份自证过的都没有：T 不判
    if acknowledged != 0:
        rotation_tail = rotations_of_every_version(context)[acknowledged].rotation_tail
        if rotation_tail is not None:
            # T = 「c_见证 ≥ τ(c)」（登记读法 3）
            tail_check = TAIL_CHECK_TRUE if witness_counter >= rotation_tail else TAIL_CHECK_FALSE
    return WitnessEvaluation(witness_counter, branch_outcome.branch, branch_outcome.compared, witness_true, rotation_tail, tail_check)
