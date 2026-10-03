"""步 6、7、8：取重放前缀、施加前验点名单元、得落到的版本。

前缀口径五条（.claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换）：
jsn 严格连续（断号即止）、(实例代号, checkpoint_txg) 大于根的水位、提交标记齐全的事务才施加（已定项 7）、
施加前逐项验证点名单元（两份都读得出、校验和都对才施加）、施加的单位是一次发布（按末条标志认边界）；注 1–4。
"""

from __future__ import annotations

from dataclasses import dataclass

import on_disk_layout as layout
import spec_gaps
from crash_image import SparseCrashImage
from crc32c_castagnoli import crc32c
from journal_ring import JournalRecord, NamedItem, RingScan
from root_ring import SelfVerifiedRoot, VersionProjection
from system_configuration import PoolRefusal, PoolSystemConfiguration

PATH_FIRST_GENERATION_ROOT = "first_generation_root"
PATH_ANCHOR_READABLE = "anchor_readable"
PATH_ANCHOR_UNREADABLE_HEAD_BY_ORDINAL_ONE = "anchor_unreadable_head_by_ordinal_one"
PATH_ANCHOR_UNREADABLE_NO_HEAD = "anchor_unreadable_no_head"


@dataclass(frozen=True)
class RebuiltRoot:
    """格式补丁（E170 第 2 次重跑登记 12.5 格二）：.claude/kb/decisions/23-journal的角色与格式.md@已定项 15：崩在记录持久之后、根槽持久之前，
    恢复由记录重建那次发布的根——「施加一条记录 = 把所选根的这七个字段换成记录里的（实例表指针、卸载记号、分配记录树根从新根段来，
    不再照抄所选根）」。投影照旧只取六样（D13 已定项 6 射程：实例代号、checkpoint_txg、树表单元指针、中央映射树根指针、树 ID 水位、F），
    多出的三样进重建的根、不进投影（读法 11）。卸载记号存它的值：根记录取 flags 的位 0（D22 已定项 7），记录取新根段那 4 字节。"""

    instance: int
    checkpoint_txg: int
    tree_table_pointer: bytes
    central_mapping_root_pointer: bytes
    tree_identifier_watermark: int
    rollback_floor: int
    instance_table_pointer: bytes
    allocation_records_root_pointer: bytes
    unmount_marker: int

    def projection(self) -> VersionProjection:
        return VersionProjection(self.instance, self.checkpoint_txg, self.tree_table_pointer, self.central_mapping_root_pointer,
                                 self.tree_identifier_watermark, self.rollback_floor)


@dataclass(frozen=True)
class ReplayOutcome:
    applied_records: list[JournalRecord]
    landed: VersionProjection
    chain_path: str
    stop_reason: str
    rebuilt_root: RebuiltRoot


def rebuilt_root_after_applying(root: SelfVerifiedRoot, last_applied: JournalRecord | None) -> RebuiltRoot:
    if last_applied is None:
        return RebuiltRoot(root.unsigned("instance"), root.unsigned("checkpoint_txg"), root.field_bytes("tree_table_pointer"),
                           root.field_bytes("central_mapping_root_pointer"), root.unsigned("tree_identifier_watermark"),
                           root.unsigned("rollback_floor"), root.field_bytes("instance_table_pointer"),
                           root.field_bytes("allocation_records_root_pointer"),
                           root.unsigned("flags") & layout.ROOT_FLAGS_UNMOUNT_MARKER_BIT)
    return RebuiltRoot(last_applied.unsigned("jsn_instance"), last_applied.unsigned("checkpoint_txg"),
                       tree_table_pointer=last_applied.field_bytes("new_root_tree_table_pointer"),
                       central_mapping_root_pointer=last_applied.field_bytes("new_root_central_mapping_root_pointer"),
                       tree_identifier_watermark=last_applied.unsigned("new_root_tree_identifier_watermark"),
                       rollback_floor=last_applied.unsigned("new_root_rollback_floor"),
                       instance_table_pointer=last_applied.field_bytes("new_root_instance_table_pointer"),
                       allocation_records_root_pointer=last_applied.field_bytes("new_root_allocation_records_root_pointer"),
                       unmount_marker=last_applied.unsigned("new_root_unmount_marker"))


def outcome_without_records(root: SelfVerifiedRoot, chain_path: str, stop_reason: str) -> ReplayOutcome:
    return ReplayOutcome([], root.projection(), chain_path, stop_reason, rebuilt_root_after_applying(root, None))


class UnitChecksumCache:
    """同一份单元字节的 CRC 只算一次；只是提速，不改判定。"""

    def __init__(self, capacity: int = 8192):
        self.capacity = capacity
        self.checksum_by_content: dict[bytes, int] = {}

    def checksum(self, content: bytes) -> int:
        cached = self.checksum_by_content.get(content)
        if cached is None:
            if len(self.checksum_by_content) >= self.capacity:
                self.checksum_by_content.clear()
            cached = crc32c(content)
            self.checksum_by_content[content] = cached
        return cached


def named_unit_verifies_on_both_copies(item: NamedItem, image: SparseCrashImage, cache: UnitChecksumCache,
                                       gaps: spec_gaps.GapsHitInOneState) -> bool:
    """D23 已定项 14 第四条与 D16 已定项 7：一个点名单元的两份（两条位置条目）都读得出、校验和都对才施加。
    位置条目 = 设备 + 16 KiB 槽号 + 密文校验和（D19 已定项 4）；校验和罩整单元（layout 三节「CRC32C 整单元」）；
    单元多长按类型标签取（D18 已定项 11 登记表）。标签不认得或设备不在池里，kb 没写怎么读（G11），暂代读法当读不出。"""
    unit_bytes = layout.UNIT_BYTES_BY_TYPE_TAG.get(item.unit_type_tag)
    if unit_bytes is None:
        gaps.record_hit(spec_gaps.GAP_NAMED_UNIT_WITH_UNKNOWN_TYPE_TAG_OR_FOREIGN_DEVICE)
        return False
    for entry in item.location_entries():
        content = image.read_bytes(entry.device_identity, entry.slot_number * layout.PLACEMENT_SLOT_BYTES, unit_bytes)
        if content is None:
            gaps.record_hit(spec_gaps.GAP_NAMED_UNIT_WITH_UNKNOWN_TYPE_TAG_OR_FOREIGN_DEVICE)
            return False
        if cache.checksum(content) != entry.ciphertext_checksum:
            return False
    return True


def flagged_record_is_followed_within_its_publish(record: JournalRecord, scan: RingScan) -> bool:
    """D23 已定项 4 第四格：同一 (实例代号, checkpoint_txg) 里带末条标志的那条之后还有记录（按计数器读、缺号不停）⇒ 那条当损坏。"""
    if not record.is_last_of_publish:
        return False
    counter = record.unsigned("jsn_counter")
    return any(other.watermark == record.watermark and other.unsigned("jsn_counter") > counter
               for other in scan.records_by_counter.values())


def count_flagged_records_of_publish(watermark: tuple[int, int], scan: RingScan) -> int:
    return sum(1 for record in scan.records_by_counter.values() if record.watermark == watermark and record.is_last_of_publish)


def cut_uncommitted_transaction_tail(candidate: list[JournalRecord]) -> list[JournalRecord]:
    """D23 已定项 7：丢掉提交标记还没出现的那个事务的全部记录；事务号 0（空发布）不进这一判（已定项 19 ①）。"""
    for start_index, record in enumerate(candidate):
        transaction_number = record.unsigned("transaction_number")
        if transaction_number == 0:
            continue
        same_transaction = [other for other in candidate[start_index:] if other.unsigned("transaction_number") == transaction_number]
        if not any(other.unsigned("commit_marker") == 1 for other in same_transaction):
            return candidate[:start_index]
    return candidate


def cut_to_publish_boundary(candidate: list[JournalRecord]) -> list[JournalRecord]:
    """第五条（D16 已定项 4）：施加的单位是一次发布；前缀里一次发布的末条没到，那次发布整体不施加，前缀到它之前为止。"""
    kept_until = 0
    for index, record in enumerate(candidate):
        is_last_of_group = index + 1 == len(candidate) or candidate[index + 1].watermark != record.watermark
        if is_last_of_group:
            if not record.is_last_of_publish:
                break
            kept_until = index + 1
    return candidate[:kept_until]


def replay_prefix(root: SelfVerifiedRoot, scan: RingScan, image: SparseCrashImage, pool: PoolSystemConfiguration,
                  cache: UnitChecksumCache, gaps: spec_gaps.GapsHitInOneState) -> ReplayOutcome | PoolRefusal:
    root_instance = root.unsigned("instance")
    root_txg = root.unsigned("checkpoint_txg")
    root_watermark = (root_instance, root_txg)
    if root_txg == 0:
        # D23 已定项 14 注 1：所选根是 mkfs 的第 0 代根时它一条记录都不覆盖 ⇒ 一条都不施加。
        return outcome_without_records(root, PATH_FIRST_GENERATION_ROOT, "first_generation_root_covers_no_record")
    records = scan.records_by_counter
    if count_flagged_records_of_publish(root_watermark, scan) > 1:
        # D23 已定项 14「这一版的失败处置」：同一 (实例代号, checkpoint_txg) 两条带末条标志 ⇒ 盘坏了，在任何写之前停下。
        # 与已定项 4 第四格（前一条当损坏）重叠，两种读法（G20）；暂代读法取停下。
        gaps.record_hit(spec_gaps.GAP_TWO_LAST_RECORD_FLAGS_IN_ONE_PUBLISH_STOP_OR_BREAK)
        return PoolRefusal("root_publish_carries_more_than_one_last_record_flag")
    # 注 1（读法乙）：「所选根覆盖的最后一条」按末条标志认；读得出的几条里没有带标志的（或它按已定项 4 第四格当损坏），算读不出。
    anchors = [record for record in records.values() if record.watermark == root_watermark and record.is_last_of_publish
               and not flagged_record_is_followed_within_its_publish(record, scan)]
    previous: JournalRecord | None
    if anchors:
        chain_path = PATH_ANCHOR_READABLE
        previous = anchors[0]
        expected_counter = previous.unsigned("jsn_counter") + 1
    else:
        # 注 1：那条读不出时，链首接水位之上第一条可读记录，前提是它的本次发布内序号为 1、checkpoint_txg = 所选根 + 1。
        above = [record for record in records.values() if record.watermark > root_watermark
                 and not flagged_record_is_followed_within_its_publish(record, scan)]
        if not above:
            return outcome_without_records(root, PATH_ANCHOR_UNREADABLE_NO_HEAD, "no_readable_record_above_the_watermark")
        head = min(above, key=lambda record: record.unsigned("jsn_counter"))
        chain_path = PATH_ANCHOR_UNREADABLE_HEAD_BY_ORDINAL_ONE
        if head.unsigned("jsn_instance") != root_instance:
            return outcome_without_records(root, chain_path, "head_belongs_to_another_instance")
        if head.unsigned("ordinal_within_publish") != 1 or head.unsigned("checkpoint_txg") != root_txg + 1:
            return outcome_without_records(root, chain_path, "head_is_not_ordinal_one_of_the_next_txg")
        previous = None
        expected_counter = head.unsigned("jsn_counter")
    in_flight_limit = pool.unsigned("journal_in_flight_record_limit")
    candidate: list[JournalRecord] = []
    publishes_started = 0
    stop_reason = "walk_not_stopped"
    while True:
        record = records.get(expected_counter)
        if record is None:
            stop_reason = "jsn_not_readable_or_gap"  # D23 已定项 22 断号即止；已定项 18 校验和不过断链即止
            break
        if record.unsigned("jsn_instance") != root_instance:
            stop_reason = "instance_boundary"  # 注 1：下一条的实例代号与所选根不同即停
            break
        if record.watermark <= root_watermark:
            stop_reason = "not_above_root_watermark"  # 第二条
            break
        if flagged_record_is_followed_within_its_publish(record, scan):
            stop_reason = "last_record_flag_followed_within_publish"  # 已定项 4 第四格
            break
        if count_flagged_records_of_publish(record.watermark, scan) > 1:
            gaps.record_hit(spec_gaps.GAP_TWO_LAST_RECORD_FLAGS_IN_ONE_PUBLISH_STOP_OR_BREAK)
            return PoolRefusal("publish_in_prefix_carries_more_than_one_last_record_flag")
        ordinal = record.unsigned("ordinal_within_publish")
        starts_a_publish = previous is None or previous.watermark != record.watermark
        if starts_a_publish:
            publishes_started += 1
            if ordinal != 1:
                if publishes_started > 1:
                    gaps.record_hit(spec_gaps.GAP_FIRST_ORDINAL_OF_LATER_PUBLISHES_IN_THE_PREFIX)
                stop_reason = "first_ordinal_of_publish_not_one"  # 已定项 4：锚点读得出时，下一次发布的首条序号不是 1
                break
        elif ordinal != previous.unsigned("ordinal_within_publish") + 1:
            stop_reason = "ordinal_skips_within_publish"  # 已定项 4：一次发布之内跳号
            break
        if previous is not None and record.unsigned("back_chain") != crc32c(previous.header_with_checksum_zeroed()):
            stop_reason = "back_chain_mismatch"  # 已定项 8、10、19 ②；I-8.6「不等的记录不进重放前缀」
            break
        if not all(named_unit_verifies_on_both_copies(item, image, cache, gaps) for item in record.named_items()):
            stop_reason = "named_unit_not_verified_on_both_copies"  # 第四条
            break
        if len(candidate) >= in_flight_limit:
            gaps.record_hit(spec_gaps.GAP_PREFIX_LONGER_THAN_THE_IN_FLIGHT_LIMIT)
            stop_reason = "in_flight_limit_reached"  # I-8.3 补句：前缀最多 N 条
            break
        candidate.append(record)
        previous = record
        expected_counter += 1
    applied = cut_to_publish_boundary(cut_uncommitted_transaction_tail(candidate))
    if chain_path == PATH_ANCHOR_UNREADABLE_HEAD_BY_ORDINAL_ONE and applied:
        gaps.record_hit(spec_gaps.GAP_BACK_CHAIN_OF_A_CHAIN_HEAD_WITHOUT_READABLE_ANCHOR)
    last_applied = applied[-1] if applied else None
    rebuilt_root = rebuilt_root_after_applying(root, last_applied)
    return ReplayOutcome(applied, rebuilt_root.projection(), chain_path, stop_reason, rebuilt_root)
