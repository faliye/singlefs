#!/usr/bin/env python3
"""alloc-basis-r1 Opus 攻方腿的计数模型。

只读导入 crates/ 的事实（工作区 2026-09-17 00:5x UTC 快照，hash-object 见报告开头），不与实现共用代码，不编译 crates/。
导入的事实（每条带出处）：
- 单元区起点槽 50176、聚簇段 64 槽（allocator.rs 第 10 行 import 的常量；unit_area_of_a_4_gib_image 单测）。
- mkfs 两个单元：实例表 m1 @50176 跨 2、第 0 版树表 m2 @50178 跨 1，分配代 0（allocator.rs 第 438–447 行单测、layout/01-first-txn.md 第 56–57 行）。
- 第 0 代根种进三个区域各自槽 0（make_filesystem.rs 第 4 行）；暖机根照抄 mkfs 根的树表与实例表指针（transaction.rs 第 319–320 行）。
- 根槽落区域 txg mod 3、槽 (txg div 3) mod 8，区域 0/1/2 → 盘 0/1/0（transaction.rs 第 74 行；D22 已定项 16 第 4 句）。
- 角色、跨度与落点规则（transaction.rs 第 402–487 行）；重写次序 rewritten_roles（transaction.rs 第 789–811 行）。
- 用户数据取最低的偶数空槽对、不在开放段；提交内生块从最低全空 64 槽段 bump、两槽的按偶数对齐（allocator.rs 第 218–251、392–420 行）。
- 释放 = 记录改成已释放 + 释放代、槽仍占着（allocator.rs 第 164–173、351–370 行）；重建时已释放的进 defer（第 290–309 行）。
- 第一个事务 previous = None、一个落点都不释放（transaction.rs 第 899–904 行）⇒ m2 从没被释放。
- 写行与之后的空发布：rollback_floor 取上一版的（mount.rs 第 280、313 行）；重开后开放段不续（allocator.rs 第 287–288 行）。
- checker：记账只取 (txg, 实例) 最大的根下的行，I-3.1 比「按 (设备, 槽, 跨度) 去重后的跨度和」（walk.rs 第 746–751、810–820 行）。
没有实现、由条款或里程碑预想给的（模型里的取法写在各函数的注释里）：可再分配谓词与 F（D16 已定项 1）、
回退与影子账（D23 已定项 14、窄读法 2026-09-16）、步 4 / 步 5 的脚本（里程碑 02-second-txn.md 步 4、步 5 的预想）。
"""
import argparse
import copy
import sys
from dataclasses import dataclass, field

UNIT_AREA_START_SLOT = 50176
UNIT_AREA_SLOT_COUNT = 211968
CLUSTER_SEGMENT_SLOTS = 64
REGION_COUNT = 3
SLOTS_PER_REGION = 8
REGION_DEVICE = (0, 1, 0)
DEVICES = (0, 1)

ROLE_SPAN = {
    "data": 2, "instance_table": 2, "extent_root": 1, "inode_leaf": 2, "inode_root": 1,
    "allocation_tree": 1, "accounting_tree": 1, "mapping_tree": 1, "tree_table": 1,
}
ROLE_ORDER = ["data", "instance_table", "extent_root", "inode_leaf", "inode_root",
              "allocation_tree", "accounting_tree", "mapping_tree", "tree_table"]
FILE_ROLES = ["data", "extent_root", "inode_leaf", "inode_root"]
FIXED_POINT_ROLES = ["allocation_tree", "accounting_tree", "mapping_tree", "tree_table"]


@dataclass(frozen=True)
class Unit:
    identifier: int
    role: str
    slot: int
    span: int
    allocation_txg: int


@dataclass
class RootRecord:
    txg: int
    instance: int
    rollback_floor: int
    position: tuple
    references: dict
    instance_rows: tuple
    ledger: dict = field(default_factory=dict)
    allocator_records: dict = field(default_factory=dict)


def root_position_of(txg):
    return (txg % REGION_COUNT, (txg // REGION_COUNT) % SLOTS_PER_REGION)


def root_is_valid(root, instance_rows):
    """D23 已定项 14：(i, T) 有效 ⟺ 实例表无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti。"""
    for row_instance, row_txg, _row_transaction, _is_rollback in instance_rows:
        if row_instance == root.instance:
            return root.txg <= row_txg
    return True


def oldest_valid_root_txg(ring, instance_rows):
    """D16 已定项 1：盘上全部根槽里自证合法、按实例表判仍然有效的根的最小 checkpoint_txg。"""
    txgs = [root.txg for root in ring.values() if root is not None and root_is_valid(root, instance_rows)]
    return min(txgs) if txgs else 0


def effective_rollback_floor(ring, instance_rows, root_set_reading):
    """D16 已定项 1「恢复后生效值 = 各幸存盘所带 F 最大值的最小值」。
    root_set_reading = 'all'：盘上全部自证过的根都算「所带」；'valid'：只算按实例表仍然有效的根（条款没写是哪一个）。"""
    per_device_maximum = []
    for device in DEVICES:
        floors = [root.rollback_floor for position, root in ring.items()
                  if root is not None and REGION_DEVICE[position[0]] == device
                  and (root_set_reading == "all" or root_is_valid(root, instance_rows))]
        per_device_maximum.append(max(floors) if floors else 0)
    return min(per_device_maximum)


class Allocator:
    """一块盘的分配器（两盘同构、同槽，D2 已定项 10）。记录态：allocated / released / reclaimed（记录还在、槽空闲）。"""

    def __init__(self):
        self.record_state = {}
        self.occupied_slots = set()
        self.free_slot_count = UNIT_AREA_SLOT_COUNT
        self.open_segment_start = None
        self.bump_cursor = None

    def occupied_count(self):
        return len(self.occupied_slots)

    def deferred_count(self, units):
        return sum(units[identifier].span for identifier, (state, _generation) in self.record_state.items()
                   if state == "released")

    def deferred_count_with_generation_at_most(self, units, threshold):
        return sum(units[identifier].span for identifier, (state, generation) in self.record_state.items()
                   if state == "released" and generation <= threshold)

    def mark_allocated(self, unit, units):
        for slot in range(unit.slot, unit.slot + unit.span):
            assert slot not in self.occupied_slots, f"槽 {slot} 已占着"
            self.occupied_slots.add(slot)
        self.free_slot_count -= unit.span
        # 已回收的旧记录在它的落点被重新分配时被覆盖（D3 已定项 7「条目留到该落点被重新分配时覆盖」）。
        for identifier, (state, _generation) in list(self.record_state.items()):
            other = units[identifier]
            overlaps = other.slot < unit.slot + unit.span and unit.slot < other.slot + other.span
            if state == "reclaimed" and overlaps:
                del self.record_state[identifier]
        self.record_state[unit.identifier] = ("allocated", unit.allocation_txg)

    def release(self, identifier, txg):
        state, _generation = self.record_state[identifier]
        assert state == "allocated", f"单元 {identifier} 释放了两次"
        self.record_state[identifier] = ("released", txg)

    def reclaim(self, units, threshold):
        """可再分配 ⟺ 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)：槽放回空闲，记录留着。"""
        for identifier, (state, generation) in list(self.record_state.items()):
            if state == "released" and generation <= threshold:
                unit = units[identifier]
                for slot in range(unit.slot, unit.slot + unit.span):
                    self.occupied_slots.discard(slot)
                self.free_slot_count += unit.span
                self.record_state[identifier] = ("reclaimed", generation)

    def rebuild_from_records(self, records, units):
        """重开 / 回退：从所选根的分配记录重建（allocator.rs 第 290–309 行）：已释放的（含已回收、记录还在的）一律进 defer、槽占着；开放段不续。"""
        self.record_state = {}
        self.occupied_slots = set()
        self.free_slot_count = UNIT_AREA_SLOT_COUNT
        self.open_segment_start = None
        self.bump_cursor = None
        for identifier, (state, generation) in sorted(records.items()):
            unit = units[identifier]
            for slot in range(unit.slot, unit.slot + unit.span):
                self.occupied_slots.add(slot)
            self.free_slot_count -= unit.span
            self.record_state[identifier] = ("allocated", generation) if state == "allocated" else ("released", generation)

    def lowest_user_data_slot(self, isolated_slots):
        """allocator.rs 第 220–236 行：偶数槽、两槽都空、不在开放段；加上影子账隔离的槽不许发（D23 已定项 14）。"""
        candidate = UNIT_AREA_START_SLOT
        while candidate + 1 < UNIT_AREA_START_SLOT + UNIT_AREA_SLOT_COUNT:
            inside_open_segment = (self.open_segment_start is not None
                                   and self.open_segment_start <= candidate < self.open_segment_start + CLUSTER_SEGMENT_SLOTS)
            both_free = all(slot not in self.occupied_slots and slot not in isolated_slots for slot in (candidate, candidate + 1))
            if not inside_open_segment and both_free:
                return candidate
            candidate += 2
        raise RuntimeError("单元区满")

    def lowest_empty_segment(self, isolated_slots):
        segment_start = UNIT_AREA_START_SLOT
        while True:
            slots = range(segment_start, segment_start + CLUSTER_SEGMENT_SLOTS)
            if all(slot not in self.occupied_slots and slot not in isolated_slots for slot in slots):
                return segment_start
            segment_start += CLUSTER_SEGMENT_SLOTS

    def commit_generated_slot(self, span, isolated_slots):
        """allocator.rs 第 393–420 行：开放段 bump，两槽的对齐到偶数，段满开下一个全空段。"""
        if self.open_segment_start is None:
            self.open_segment_start = self.lowest_empty_segment(isolated_slots)
            self.bump_cursor = self.open_segment_start
        start = self.bump_cursor
        if span == 2 and start % 2 == 1:
            start += 1
        if start + span > self.open_segment_start + CLUSTER_SEGMENT_SLOTS:
            self.open_segment_start = None
            return self.commit_generated_slot(span, isolated_slots)
        self.bump_cursor = start + span
        return start


@dataclass
class Settings:
    reclaim_enabled: bool = True
    isolation_reading: str = "narrow"          # narrow（按槽，条款字面）/ narrow_unit / narrow_candidate / wide / off
    allocator_floor_roots: str = "all"         # F_生效 按哪些根算：all / valid
    rollback_root_floor: str = "selected_root"  # 回退那次发布的根记录写哪个 F：selected_root（照 R_old）/ effective（回退时的 F_生效）
    gen0_tree_table_release: str = "never"     # m2 什么时候释放：never（今天的实现）/ first_transaction / first_overwrite
    reuse_window_zero: bool = False            # 里程碑步 5 必红那条的开关（C22）宽形态：已释放的一律当场可再分配
    force_reuse_slot_at_label: tuple = None    # 窄形态：(发布标签, 槽号)，只把那一个已释放落点在那次发布里当场放回
    skip_floor_raise: bool = False             # 里程碑脚本里 15、16 两次空发布不抬 F（必红那条要 F 仍是 0）


class Pool:
    def __init__(self, settings):
        self.settings = settings
        self.units = {}
        self.next_unit_identifier = 1
        self.slot_content = {}
        self.allocator = Allocator()
        self.allocator_ring = {}   # 分配器眼里的根环（写失败的槽按旧内容算）
        self.disk_ring = {}        # checker 从盘上读到的根环
        self.current_version = {}
        self.current_rows = ()
        self.current_instance = 0
        self.current_floor = 0
        self.next_txg = 1
        self.highest_instance = 0
        self.gen0_tree_table_identifier = None
        self.published_file_versions = 0
        self.log = []
        instance_table = self.new_unit("instance_table", UNIT_AREA_START_SLOT, 0)
        tree_table = self.new_unit("tree_table", UNIT_AREA_START_SLOT + 2, 0)
        self.gen0_tree_table_identifier = tree_table.identifier
        self.current_version = {"instance_table": instance_table.identifier, "tree_table": tree_table.identifier}
        for region in range(REGION_COUNT):
            seed = RootRecord(0, 0, 0, (region, 0), dict(self.current_version), ())
            seed.is_nonempty = False
            seed.ledger = {"pre": (3, 0), "post": (3, 0)}
            seed.still_allocated = 3
            seed.allocator_records = copy.deepcopy(self.allocator.record_state)
            self.allocator_ring[(region, 0)] = seed
            self.disk_ring[(region, 0)] = seed

    def new_unit(self, role, slot, txg):
        unit = Unit(self.next_unit_identifier, role, slot, ROLE_SPAN[role], txg)
        self.next_unit_identifier += 1
        self.units[unit.identifier] = unit
        self.allocator.mark_allocated(unit, self.units)
        for offset in range(unit.span):
            self.slot_content[slot + offset] = unit.identifier
        return unit

    def isolated_slots(self):
        """影子账（D23 已定项 14）。narrow：只被被抛弃根（按当前实例表判无效）引用、不被任何有效根引用的槽；
        wide：环里有被抛弃根时，凡被环里任一可读根引用的槽；off：不隔离（必红那两格的开关）。"""
        if self.settings.isolation_reading == "off":
            return set()
        roots = [root for root in self.allocator_ring.values() if root is not None]
        abandoned = [root for root in roots if not root_is_valid(root, self.current_rows)]
        if not abandoned:
            return set()

        def slots_of(root_list):
            return {slot for root in root_list for identifier in root.references.values()
                    for slot in range(self.units[identifier].slot, self.units[identifier].slot + self.units[identifier].span)}
        if self.settings.isolation_reading == "wide":
            return slots_of(roots)
        valid = [root for root in roots if root_is_valid(root, self.current_rows)]
        if self.settings.isolation_reading == "narrow_unit":
            # 按单元比而不按槽比：有效根引用的是同一个单元才豁免（条款写的是「槽」，这一臂是本腿加的对照）。
            valid_units = {identifier for root in valid for identifier in root.references.values()}
            return {slot for root in abandoned for identifier in root.references.values() if identifier not in valid_units
                    for slot in range(self.units[identifier].slot, self.units[identifier].slot + self.units[identifier].span)}
        if self.settings.isolation_reading == "narrow_candidate":
            # 豁免只给候选集里的根（有效 ∧ txg ≥ F_生效）引用的槽：本腿提的收严，被攻过零轮。
            floor = effective_rollback_floor(self.allocator_ring, self.current_rows, self.settings.allocator_floor_roots)
            return slots_of(abandoned) - slots_of([root for root in valid if root.txg >= floor])
        return slots_of(abandoned) - slots_of(valid)

    def reclaim_threshold(self, ring, instance_rows):
        if self.settings.reuse_window_zero:
            return 10 ** 9
        floor = effective_rollback_floor(ring, instance_rows, self.settings.allocator_floor_roots)
        return max(floor, oldest_valid_root_txg(ring, instance_rows))

    def publish(self, label, rewritten_roles, new_floor=None, root_write=("ok",)):
        """一次发布：可再分配谓词按分配那一刻盘上的环判（C282：在飞的发布不算）→ 释放 → 分配 → 装根。
        记账行两种算法都记：pre = 分配器当时的占着；post = 按「这个根持久之后的环」再判一次可再分配（本腿提的回收时点，被攻过零轮）。
        root_write：("ok",) 正常；("fail_old",) 根槽写失败、槽里仍是旧内容；("fail_torn",) 根槽写失败、槽读不出（分配器按旧内容算）。"""
        txg = self.next_txg
        self.next_txg += 1
        allocator_before = copy.deepcopy(self.allocator)
        version_before = dict(self.current_version)
        if self.settings.reclaim_enabled:
            self.allocator.reclaim(self.units, self.reclaim_threshold(self.allocator_ring, self.current_rows))
        if self.settings.force_reuse_slot_at_label is not None and self.settings.force_reuse_slot_at_label[0] == label:
            # 必红那条（C22）的窄形态：只把这一个已释放落点当场放回，别的照谓词。
            forced_slot = self.settings.force_reuse_slot_at_label[1]
            for identifier, (state, generation) in list(self.allocator.record_state.items()):
                if state == "released" and self.units[identifier].slot == forced_slot:
                    self.allocator.record_state[identifier] = ("released", -1)
            self.allocator.reclaim(self.units, -1)
        isolated = self.isolated_slots()
        # 第一个事务 previous = None，一个落点都不释放（transaction.rs 第 899–904 行）：m2 不走通用的「换下就释放」。
        releases = [] if label == "A" else [self.current_version[role] for role in rewritten_roles if role in self.current_version]
        release_gen0 = (self.settings.gen0_tree_table_release == "first_transaction" and label == "A") or (
            self.settings.gen0_tree_table_release == "first_overwrite" and label == "B")
        if release_gen0 and self.allocator.record_state.get(self.gen0_tree_table_identifier, ("",))[0] == "allocated":
            releases.append(self.gen0_tree_table_identifier)
        for identifier in releases:
            self.allocator.release(identifier, txg)
        new_version = dict(self.current_version)
        for role in ROLE_ORDER:
            if role not in rewritten_roles:
                continue
            if role == "data":
                slot = self.allocator.lowest_user_data_slot(isolated)
            else:
                slot = self.allocator.commit_generated_slot(ROLE_SPAN[role], isolated)
            new_version[role] = self.new_unit(role, slot, txg).identifier
        floor = self.current_floor if new_floor is None else new_floor
        root = RootRecord(txg, self.current_instance, floor, root_position_of(txg), new_version, self.current_rows)
        ring_after = dict(self.allocator_ring)
        ring_after[root.position] = root
        occupied = self.allocator.occupied_count()
        deferred = self.allocator.deferred_count(self.units)
        post_reclaimable = self.allocator.deferred_count_with_generation_at_most(
            self.units, self.reclaim_threshold(ring_after, self.current_rows)) if self.settings.reclaim_enabled else 0
        root.ledger = {"pre": (occupied, deferred), "post": (occupied - post_reclaimable, deferred - post_reclaimable)}
        root.still_allocated = occupied - deferred
        root.allocator_records = copy.deepcopy(self.allocator.record_state)
        if root_write[0] != "ok":
            if root_write[0] == "fail_torn":
                self.disk_ring[root.position] = None
            self.allocator = allocator_before
            self.current_version = version_before
            self.log.append((label, txg, "根槽写失败：" + root_write[0]))
            return None
        self.allocator_ring[root.position] = root
        self.disk_ring[root.position] = root
        self.current_version = new_version
        self.current_floor = floor
        if "data" in rewritten_roles:
            self.published_file_versions += 1
        root.is_nonempty = "data" in rewritten_roles or label.startswith("D回退")
        return root

    def publish_first_file(self):
        return self.publish("A", FILE_ROLES + FIXED_POINT_ROLES)

    def publish_overwrite(self, label, root_write=("ok",)):
        return self.publish(label, FILE_ROLES + FIXED_POINT_ROLES, root_write=root_write)

    def publish_empty(self, label, new_floor=None, root_write=("ok",)):
        return self.publish(label, FIXED_POINT_ROLES, new_floor=new_floor, root_write=root_write)

    def first_mount_warm_up(self):
        """第一次可写挂载：实例 1，两次空发布照抄 mkfs 根（transaction.rs 第 319–370 行），不分配、不释放。"""
        self.current_instance = 1
        self.highest_instance = 1
        for _ in range(2):
            txg = self.next_txg
            self.next_txg += 1
            root = RootRecord(txg, 1, 0, root_position_of(txg), dict(self.current_version), ())
            root.is_nonempty = False
            root.ledger = {"pre": (self.allocator.occupied_count(), 0), "post": (self.allocator.occupied_count(), 0)}
            root.still_allocated = self.allocator.occupied_count()
            root.allocator_records = copy.deepcopy(self.allocator.record_state)
            self.allocator_ring[root.position] = root
            self.disk_ring[root.position] = root

    def newest_root(self, ring):
        return max((root for root in ring.values() if root is not None), key=lambda root: (root.txg, root.instance))

    def cover_both_devices(self, first_root_txg, label):
        covered = {REGION_DEVICE[root_position_of(first_root_txg)[0]]}
        count = 0
        while covered != set(DEVICES):
            root = self.publish_empty(f"{label}暖机{count + 1}")
            covered.add(REGION_DEVICE[root.position[0]])
            count += 1

    def remount_and_write_row(self):
        """步 3：重开取号、从所选根重建分配器、写行 (上一实例, 所选根 txg, 0)、暖机到两块盘（mount.rs 第 169–320 行）。"""
        chosen = self.newest_root(self.disk_ring)
        self.allocator.rebuild_from_records(chosen.allocator_records, self.units)
        self.current_version = dict(chosen.references)
        self.current_floor = chosen.rollback_floor
        self.highest_instance += 1
        self.current_rows = chosen.instance_rows + ((chosen.instance, chosen.txg, 0, False),)
        self.current_instance = self.highest_instance
        self.next_txg = max(root.txg for root in self.disk_ring.values() if root is not None) + 1
        root = self.publish("写行", ["instance_table"] + FIXED_POINT_ROLES)
        self.cover_both_devices(root.txg, "写行后")

    def administrator_rollback(self, selected_txg):
        """步 4：回退到 txg 为 selected_txg 的根（D23 已定项 14 显式例外）；回退行 + 中间实例行写在 R_old 那一版实例表上；
        defer、游标、记账从 R_old 那棵账重新载入；第一个新根 txg = 根环最大 + 1。回退根写哪个 F 条款没写，按 settings 取。"""
        selected = next(root for root in self.disk_ring.values() if root is not None and root.txg == selected_txg)
        assert root_is_valid(selected, self.current_rows), "回退目标要按实例表有效"
        floor_before = effective_rollback_floor(self.allocator_ring, self.current_rows, self.settings.allocator_floor_roots)
        assert selected.txg >= floor_before, "回退目标要在候选集里（txg ≥ F_生效）"
        self.allocator.rebuild_from_records(selected.allocator_records, self.units)
        self.current_version = dict(selected.references)
        self.highest_instance += 1
        intermediate = tuple((instance, 0, 0, False) for instance in range(selected.instance + 1, self.highest_instance))
        self.current_rows = selected.instance_rows + ((selected.instance, selected.txg, 0, True),) + intermediate
        self.current_instance = self.highest_instance
        self.next_txg = max(root.txg for root in self.disk_ring.values() if root is not None) + 1
        new_floor = selected.rollback_floor if self.settings.rollback_root_floor == "selected_root" else floor_before
        self.current_floor = new_floor
        root = self.publish("D回退", ["instance_table"] + FIXED_POINT_ROLES, new_floor=new_floor)
        self.cover_both_devices(root.txg, "回退后")


def judge_disk_state(pool, checker_floor_roots="all"):
    """按盘上读到的根环判一个状态（崩溃状态与在飞发布之前的持久态相同，所以只判每次根持久之后）。
    返回每条臂的 I-3.1 差（记账 − 并集，单位槽；0 = 绿）、各走读集合里「根引用的槽内容已被换掉」的根数、D23 主句的违例。"""
    roots = [root for root in pool.disk_ring.values() if root is not None]
    newest = pool.newest_root(pool.disk_ring)
    rows = newest.instance_rows
    valid = [root for root in roots if root_is_valid(root, rows)]
    floor = effective_rollback_floor(pool.disk_ring, rows, checker_floor_roots)
    candidates = [root for root in valid if root.txg >= floor]
    abandoned = [root for root in roots if not root_is_valid(root, rows)]

    def checker_union(root_list):
        # walk.rs 第 794–815 行：引用按 (设备, 槽, 跨度) 去重，求跨度和。
        keys = {(pool.units[identifier].slot, pool.units[identifier].span)
                for root in root_list for identifier in root.references.values()}
        return sum(span for _slot, span in keys)

    def roots_with_replaced_content(root_list):
        return sorted({root.txg for root in root_list for identifier in root.references.values()
                       if any(pool.slot_content.get(slot) != identifier
                              for slot in range(pool.units[identifier].slot,
                                                pool.units[identifier].slot + pool.units[identifier].span))})

    newest_reachable = checker_union([newest])
    union_all, union_valid, union_candidates = checker_union(roots), checker_union(valid), checker_union(candidates)
    return {
        "txg": newest.txg, "instance": newest.instance, "F_eff": floor,
        "甲候post": newest.ledger["post"][0] - union_candidates,
        "甲候pre": newest.ledger["pre"][0] - union_candidates,
        "乙仍主": newest.still_allocated - newest_reachable,
        "乙仍辅post": newest.ledger["post"][0] - union_candidates,
        "丙现(i)pre": newest.ledger["pre"][0] - union_all,
        "(ii)post": newest.ledger["post"][0] - union_valid,
        "I-5.2": 0,
        "走读(i)换内容": roots_with_replaced_content(roots),
        "走读(iii)换内容": roots_with_replaced_content(candidates),
        "D23主句违例": roots_with_replaced_content(abandoned),
        "候选集txg": sorted(root.txg for root in candidates),
    }


def format_judgement(label, judgement):
    keys = ["甲候post", "甲候pre", "乙仍主", "乙仍辅post", "丙现(i)pre", "(ii)post"]
    columns = " ".join(f"{key}={judgement[key]:+d}" for key in keys)
    return (f"{label:<10} txg={judgement['txg']:>3} inst={judgement['instance']} F_eff={judgement['F_eff']:>2} {columns} "
            f"走读(i)换内容={judgement['走读(i)换内容']} 走读(iii)换内容={judgement['走读(iii)换内容']} "
            f"D23主句违例={judgement['D23主句违例']}")


def scenario_overwrite_loop(settings, last_txg, failure_at=None, failure_kind="fail_old", checker_floor_roots="all"):
    """S1 / S3：mkfs → 暖机 1、2 → A(3) → 覆盖写一直到 last_txg；failure_at 那次发布根槽写失败、推进一格重发（D23 已定项 14）。"""
    pool = Pool(settings)
    pool.first_mount_warm_up()
    lines = []
    root = pool.publish_first_file()
    lines.append(format_judgement("A", judge_disk_state(pool, checker_floor_roots)))
    label_index = 0
    while pool.next_txg <= last_txg:
        label = "B" if label_index == 0 else f"覆写{pool.next_txg}"
        label_index += 1
        if failure_at is not None and pool.next_txg == failure_at:
            pool.publish_overwrite(label + "失败", root_write=(failure_kind,))
            lines.append(f"{label}失败   txg={failure_at} 根槽写失败（{failure_kind}），推进一格重发")
            continue
        root = pool.publish_overwrite(label)
        lines.append(format_judgement(label, judge_disk_state(pool, checker_floor_roots)))
    return pool, lines


def scenario_milestone(settings, extra_overwrites=0, checker_floor_roots="all"):
    """S2：里程碑固定脚本的预想——A(3)、B(4)、重开写行(5)与暖机(6,7)、C(8)、回退到 A 得 D(9) 与暖机(10)、
    覆写 11（释放第一个数据单元）、再覆写 12..14、抬 F=11 的空发布(15)与再推一次(16)、E(17)；之后再覆写 extra_overwrites 次。"""
    pool = Pool(settings)
    pool.first_mount_warm_up()
    lines = []
    events = {}
    pool.publish_first_file()
    lines.append(format_judgement("A", judge_disk_state(pool, checker_floor_roots)))
    pool.publish_overwrite("B")
    lines.append(format_judgement("B", judge_disk_state(pool, checker_floor_roots)))
    pool.remount_and_write_row()
    lines.append(format_judgement("写行暖机后", judge_disk_state(pool, checker_floor_roots)))
    pool.publish_overwrite("C")
    events["C数据槽"] = pool.units[pool.current_version["data"]].slot
    lines.append(format_judgement("C", judge_disk_state(pool, checker_floor_roots)))
    pool.administrator_rollback(3)
    lines.append(format_judgement("D回退暖机后", judge_disk_state(pool, checker_floor_roots)))
    events["回退后隔离槽数"] = len(pool.isolated_slots())
    for label in ["覆写11", "覆写12", "覆写13", "覆写14"]:
        pool.publish_overwrite(label)
        lines.append(format_judgement(label, judge_disk_state(pool, checker_floor_roots)))
    pool.publish_empty("抬F15", new_floor=None if settings.skip_floor_raise else 11)
    lines.append(format_judgement("抬F15", judge_disk_state(pool, checker_floor_roots)))
    pool.publish_empty("再推16")
    lines.append(format_judgement("再推16", judge_disk_state(pool, checker_floor_roots)))
    pool.publish_overwrite("E")
    events["E数据槽"] = pool.units[pool.current_version["data"]].slot
    events["E时隔离槽数"] = len(pool.isolated_slots())
    lines.append(format_judgement("E", judge_disk_state(pool, checker_floor_roots)))
    for _ in range(extra_overwrites):
        label = f"覆写{pool.next_txg}"
        pool.publish_overwrite(label)
        lines.append(format_judgement(label, judge_disk_state(pool, checker_floor_roots)))
    return pool, lines, events


def scenario_floor_raised_then_rollback(settings, checker_floor_roots, failure_at=None, last_txg=60):
    """S4：抬 F 发生在回退之前。A(3)、覆写 4..14、抬 F=10 的空发布(15)与再推一次(16，F 生效)、覆写 17、18（回收 g ≤ 10 的槽并复用）、
    回退到 txg 12（有效且 12 ≥ F_生效 10，在候选集里）、回退根与暖机、之后一直覆写到 last_txg；failure_at 那次根槽写失败（槽里留旧根）。
    条款没写的两个参数由 settings 与 checker_floor_roots 取：回退根写哪个 F；「各幸存盘所带 F」数哪些根。"""
    pool = Pool(settings)
    pool.first_mount_warm_up()
    lines = []
    pool.publish_first_file()
    while pool.next_txg <= 14:
        pool.publish_overwrite("B" if pool.next_txg == 4 else f"覆写{pool.next_txg}")
    pool.publish_empty("抬F15", new_floor=10)
    pool.publish_empty("再推16")
    lines.append(format_judgement("再推16", judge_disk_state(pool, checker_floor_roots)))
    for label in ["覆写17", "覆写18"]:
        pool.publish_overwrite(label)
        lines.append(format_judgement(label, judge_disk_state(pool, checker_floor_roots)))
    pool.administrator_rollback(12)
    lines.append(format_judgement("回退暖机后", judge_disk_state(pool, checker_floor_roots)))
    while pool.next_txg <= last_txg:
        label = f"覆写{pool.next_txg}"
        if failure_at is not None and pool.next_txg == failure_at:
            pool.publish_overwrite(label + "失败", root_write=("fail_old",))
            lines.append(f"{label}失败 根槽写失败（槽里留旧根），推进一格重发")
            continue
        pool.publish_overwrite(label)
        lines.append(format_judgement(label, judge_disk_state(pool, checker_floor_roots)))
    return pool, lines


def first_nonzero(lines, column):
    for line in lines:
        marker = f"{column}="
        if marker in line:
            value = int(line.split(marker)[1].split()[0])
            if value != 0:
                return line.split()[0], value
    return None


def summarize(title, lines, columns):
    output = [f"--- {title}"]
    for column in columns:
        hit = first_nonzero(lines, column)
        output.append(f"    {column}: " + ("全程 0" if hit is None else f"第一次非 0 在 {hit[0]}，差 {hit[1]:+d} 槽"))
    return output


ARM_COLUMNS = ["甲候post", "甲候pre", "乙仍主", "乙仍辅post", "丙现(i)pre", "(ii)post"]


def run_all():
    out = []
    out.append("# alloc-basis-r1 Opus 攻方腿模型产物（每行一个根持久之后的盘上状态；列 = 记账 − checker 并集，单位 16 KiB 槽，0 = I-3.1 绿）")
    out.append("# 列：甲候post / 甲候pre = 甲候在「本腿提的回收时点（按持久之后的环记账）」/「分配那一刻的环记账」下；乙仍主 = 仍分配 − 最新根可达；")
    out.append("# 乙仍辅post = 仍分配 + defer − 候选集并集；丙现(i)pre = 占着 − 全部自证根并集；(ii)post = 占着 − 按实例表有效根并集。")
    for release in ["never", "first_overwrite", "first_transaction"]:
        for reclaim in [False, True]:
            _pool, lines = scenario_overwrite_loop(Settings(reclaim_enabled=reclaim, gen0_tree_table_release=release), 60)
            title = f"S1 单实例覆写到 txg 60；m2 释放={release}；回收={'有' if reclaim else '无（今天的实现）'}"
            out += summarize(title, lines, ARM_COLUMNS)
            if release == "never" and reclaim:
                out += ["    " + line for line in lines if " txg= 2" in line[:20] or " txg= 3" in line[:20]][:8]
    for kind in ["fail_old", "fail_torn"]:
        _pool, lines = scenario_overwrite_loop(Settings(gen0_tree_table_release="first_transaction"), 60, failure_at=30, failure_kind=kind)
        out += summarize(f"S3 同 S1（回收有、m2 在 A 里释放）+ txg 30 根槽写失败 {kind}", lines, ARM_COLUMNS)
        out += ["    " + line for line in lines if any(f"txg= {txg}" in line for txg in (29, 31, 32, 53, 54, 55)) or "失败" in line]
    for isolation in ["narrow", "narrow_unit", "narrow_candidate", "wide", "off"]:
        for release in ["never", "first_transaction"]:
            pool, lines, events = scenario_milestone(Settings(isolation_reading=isolation, gen0_tree_table_release=release), extra_overwrites=0)
            out.append(f"--- S2 里程碑脚本到 E；影子账={isolation}；m2 释放={release}；事件 {events}")
            out += ["    " + line for line in lines]
    for title, settings in [
        ("复用窗口置 0 宽形态", Settings(reuse_window_zero=True, gen0_tree_table_release="first_transaction")),
        ("复用窗口置 0 窄形态：不抬 F、E 当场复用 50180", Settings(gen0_tree_table_release="first_transaction",
                                                         force_reuse_slot_at_label=("E", 50180), skip_floor_raise=True)),
    ]:
        pool, lines, events = scenario_milestone(settings)
        out.append(f"--- S2 {title}；m2 释放=first_transaction；事件 {events}")
        out += ["    " + line for line in lines]
    _pool, lines = scenario_overwrite_loop(Settings(gen0_tree_table_release="first_transaction", force_reuse_slot_at_label=("覆写10", 50190)), 12)
    out.append("--- S1′ 单实例、无回退、F = 0：覆写10 当场复用版本 8 的数据槽 50190（仍被根 8 引用），复用窗口置 0 的窄形态")
    out += ["    " + line for line in lines if line.split()[0] in ("覆写9", "覆写10", "覆写11")]
    for rollback_floor in ["selected_root", "effective"]:
        for allocator_floor in ["all", "valid"]:
            for checker_floor in ["all", "valid"]:
                for failure_at in [None, 33]:
                    settings = Settings(gen0_tree_table_release="first_transaction", rollback_root_floor=rollback_floor,
                                        allocator_floor_roots=allocator_floor)
                    _pool, lines = scenario_floor_raised_then_rollback(settings, checker_floor, failure_at=failure_at)
                    title = (f"S4 抬 F 在回退之前；回退根的 F={rollback_floor}；分配器 F_生效数={allocator_floor}；"
                             f"checker F_生效数={checker_floor}；根槽写失败={failure_at}")
                    out += summarize(title, lines, ["甲候post", "乙仍主"])
                    replaced = [line.split()[0] for line in lines if "走读(iii)换内容=[]" not in line and "走读(iii)换内容=" in line]
                    out.append(f"    走读(iii)换内容 非空的状态：{replaced[:6]}{' …共 ' + str(len(replaced)) if len(replaced) > 6 else ''}")
                    violated = [line.split()[0] for line in lines if "D23主句违例=[]" not in line and "D23主句违例=" in line]
                    out.append(f"    D23主句违例 非空的状态：{violated[:6]}{' …共 ' + str(len(violated)) if len(violated) > 6 else ''}")
    for isolation in ["narrow", "narrow_unit", "narrow_candidate", "wide"]:
        _pool, lines = scenario_floor_raised_then_rollback(Settings(isolation_reading=isolation, gen0_tree_table_release="first_transaction"), "all")
        violated = [line.split()[0] for line in lines if "D23主句违例=[]" not in line and "D23主句违例=" in line]
        out.append(f"--- S4 影子账读法对照（回退根的 F=selected_root、F_生效数 all、无写失败）：影子账={isolation}；D23主句违例状态 {len(violated)} 个 {violated[:3]}")
    return out


def line_for(lines, label):
    return next(line for line in lines if line.split()[0] == label)


def value_in(line, column):
    return int(line.split(f"{column}=")[1].split()[0])


def selftest():
    """钉住报告引的每个数，并证明判定会红：把记账改大 1、把候选根引用的槽换成别的内容，判定必须由 0 / 空转非 0 / 非空。"""
    failures = []
    checked = []

    def expect(condition, message):
        checked.append(message)
        if not condition:
            failures.append(message)

    _pool, lines = scenario_overwrite_loop(Settings(), 60)
    expect(first_nonzero(lines, "甲候post") == ("覆写26", 1), "S1 m2 不释放：甲候post 第一次非 0 应在覆写26、+1")
    expect(value_in(line_for(lines, "覆写27"), "甲候pre") == 11, "S1 m2 不释放：覆写27 甲候pre 应 +11")
    expect(value_in(line_for(lines, "A"), "乙仍主") == 1, "S1 m2 不释放：A 之后乙仍主应 +1")
    _pool, lines = scenario_overwrite_loop(Settings(reclaim_enabled=False), 60)
    expect(first_nonzero(lines, "丙现(i)pre") == ("覆写26", 1), "S1 无回收（今天）：丙现第一次非 0 应在覆写26、+1")
    expect(value_in(line_for(lines, "覆写27"), "丙现(i)pre") == 11, "S1 无回收：覆写27 应 +11")
    _pool, lines = scenario_overwrite_loop(Settings(gen0_tree_table_release="first_transaction"), 60)
    expect(first_nonzero(lines, "甲候post") is None, "S1 m2 在 A 里释放：甲候post 应全程 0")
    expect(first_nonzero(lines, "甲候pre") == ("覆写26", 1), "S1 m2 在 A 里释放：甲候pre 第一次非 0 应在覆写26、+1")
    expect(value_in(line_for(lines, "覆写27"), "甲候pre") == 10, "S1 m2 在 A 里释放：覆写27 甲候pre 应 +10")
    _pool, lines = scenario_overwrite_loop(Settings(gen0_tree_table_release="first_overwrite"), 60)
    expect(first_nonzero(lines, "甲候post") == ("覆写26", 1), "S1 m2 在 B 里释放：甲候post 覆写26 应 +1")
    expect(value_in(line_for(lines, "覆写27"), "甲候post") == 0, "S1 m2 在 B 里释放：覆写27 甲候post 应回 0")
    _pool, lines = scenario_overwrite_loop(Settings(gen0_tree_table_release="first_transaction"), 60, failure_at=30)
    expect([value_in(line_for(lines, label), "甲候post") for label in ("覆写29", "覆写31", "覆写53", "覆写54")] == [0, 10, 230, 0],
           "S3 fail_old：甲候post 在 29/31/53/54 应为 0/10/230/0")
    _pool, lines = scenario_overwrite_loop(Settings(gen0_tree_table_release="first_transaction"), 60, failure_at=30, failure_kind="fail_torn")
    expect([value_in(line_for(lines, label), "甲候post") for label in ("覆写31", "覆写53", "覆写54")] == [20, 240, 0],
           "S3 fail_torn：甲候post 在 31/53/54 应为 20/240/0")
    _pool, lines, events = scenario_milestone(Settings())
    expect(events["E数据槽"] == 50176 and events["回退后隔离槽数"] == 34, "S2 窄读法：E 落 50176、回退后隔离 34 槽")
    expect(value_in(line_for(lines, "D回退暖机后"), "丙现(i)pre") == -34, "S2：回退暖机后丙现应 −34")
    expect(value_in(line_for(lines, "再推16"), "甲候pre") == 21 and value_in(line_for(lines, "再推16"), "甲候post") == 1,
           "S2 m2 不释放：再推16 甲候pre +21、post +1")
    expect("D23主句违例=[4]" in line_for(lines, "E"), "S2 窄读法：E 之后被抛弃根 4 引用的槽被换了内容")
    _pool, lines, events = scenario_milestone(Settings(isolation_reading="wide"))
    expect(events["E数据槽"] == 50194, "S2 宽读法：E 落 50194")
    _pool, lines, events = scenario_milestone(Settings(isolation_reading="off", gen0_tree_table_release="first_transaction"))
    bad = line_for(lines, "D回退暖机后")
    expect(value_in(bad, "甲候post") == 0 and value_in(bad, "乙仍主") == 0 and "D23主句违例=[5, 6, 7, 8]" in bad,
           "S2 影子账关：回退暖机后甲候 / 乙仍判 0 而被抛弃根 5–8 的槽被换了内容")
    _pool, lines, events = scenario_milestone(Settings(reuse_window_zero=True, gen0_tree_table_release="first_transaction"))
    expect(value_in(line_for(lines, "B"), "甲候post") == -11 and "走读(iii)换内容=[0, 1, 2]" in line_for(lines, "B"),
           "S2 复用窗口置 0（宽形态）：B 之后甲候post −11、候选根 0–2 的槽被换内容")
    settings = Settings(gen0_tree_table_release="first_transaction", force_reuse_slot_at_label=("E", 50180), skip_floor_raise=True)
    _pool, lines, events = scenario_milestone(settings)
    narrow_bad = line_for(lines, "E")
    expect(events["E数据槽"] == 50180 and "F_eff= 0" in narrow_bad and "走读(iii)换内容=[3, 9, 10]" in narrow_bad
           and value_in(narrow_bad, "甲候post") == 0 and value_in(narrow_bad, "乙仍主") == 0,
           "S2 复用窗口置 0（窄形态）：E 在 F = 0 时落 50180，候选根 3/9/10 的槽被换内容而甲候 / 乙仍的 I-3.1 判 0")
    settings = Settings(gen0_tree_table_release="first_transaction")
    _pool, lines = scenario_floor_raised_then_rollback(settings, "all", failure_at=33)
    expect("走读(iii)换内容=[9]" in line_for(lines, "覆写40") and "F_eff= 0" in line_for(lines, "覆写40"),
           "S4 回退根照抄 R_old 的 F + 写失败：覆写40 F_生效掉回 0、根 9 成了假候选")
    settings = Settings(gen0_tree_table_release="first_transaction", rollback_root_floor="effective")
    _pool, lines = scenario_floor_raised_then_rollback(settings, "all", failure_at=33)
    expect(all("走读(iii)换内容=[]" in line for line in lines if "走读" in line), "S4 回退根写 F_生效：没有假候选")
    _pool, lines = scenario_floor_raised_then_rollback(Settings(gen0_tree_table_release="first_transaction"), "valid")
    expect(value_in(line_for(lines, "回退暖机后"), "甲候post") == -71, "S4 checker 只数有效根的 F：回退暖机后 −71")
    for isolation, release, slot, violation in [("narrow_unit", "first_transaction", 50176, "D23主句违例=[4]"),
                                                 ("narrow_candidate", "first_transaction", 50178, "D23主句违例=[]"),
                                                 ("narrow_candidate", "never", 50180, "D23主句违例=[]")]:
        _pool, lines, events = scenario_milestone(Settings(isolation_reading=isolation, gen0_tree_table_release=release))
        expect(events["E数据槽"] == slot and violation in line_for(lines, "E"), f"S2 影子账={isolation}、m2={release}：E 落 {slot}、{violation}")
    for isolation, expected_count in [("narrow", 21), ("narrow_unit", 0), ("narrow_candidate", 0), ("wide", 0)]:
        _pool, lines = scenario_floor_raised_then_rollback(Settings(isolation_reading=isolation, gen0_tree_table_release="first_transaction"), "all")
        count = sum(1 for line in lines if "D23主句违例=[]" not in line and "D23主句违例=" in line)
        expect(count == expected_count, f"S4 影子账={isolation}：D23 主句违例状态应 {expected_count} 个，实为 {count}")
    _pool, lines = scenario_floor_raised_then_rollback(Settings(gen0_tree_table_release="first_transaction", allocator_floor_roots="valid"), "valid")
    false_candidates = [line.split()[0] for line in lines if "走读(iii)换内容=[]" not in line and "走读(iii)换内容=" in line]
    expect(len(false_candidates) == 7 and false_candidates[0] == "回退暖机后" and all(value_in(line, "甲候post") == 0 for line in lines if "甲候post=" in line),
           "S4 F_生效两边都只数有效根、回退根照抄 R_old 的 F、无写失败：回退暖机后起 7 个状态有假候选，而甲候 I-3.1 全程 0")
    _pool, lines = scenario_overwrite_loop(Settings(gen0_tree_table_release="first_transaction", force_reuse_slot_at_label=("覆写10", 50190)), 12)
    single_bad = line_for(lines, "覆写10")
    expect("走读(iii)换内容=[8]" in single_bad and "走读(i)换内容=[8]" in single_bad
           and all(value_in(single_bad, column) == 0 for column in ["甲候post", "乙仍主", "丙现(i)pre"]),
           "S1′ 单实例复用仍被根 8 引用的 50190：三条臂的 I-3.1 都判 0，只有走读判得出")
    # 判定自己会红：记账改大 1；候选根引用的槽换成别的单元。
    pool, _lines = scenario_overwrite_loop(Settings(gen0_tree_table_release="first_transaction"), 10)
    newest = pool.newest_root(pool.disk_ring)
    expect(judge_disk_state(pool)["甲候post"] == 0, "判别力自证的基线应为 0")
    newest.ledger["post"] = (newest.ledger["post"][0] + 1, newest.ledger["post"][1])
    expect(judge_disk_state(pool)["甲候post"] == 1, "记账改大 1 槽，甲候post 必须变 +1")
    victim = pool.units[newest.references["data"]]
    pool.slot_content[victim.slot] = -1
    expect(judge_disk_state(pool)["走读(iii)换内容"] == [newest.txg], "候选根的数据槽换内容，走读(iii) 必须报出那个根")
    return failures, len(checked)


def legal_floor_upper_bound(pool):
    """D16 已定项 1：抬 F 的上限 = min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)；非空有效根不足 4 个时取最旧的有效根。"""
    rows = pool.newest_root(pool.disk_ring).instance_rows
    valid = [(position, root) for position, root in pool.disk_ring.items() if root is not None and root_is_valid(root, rows)]
    newest_per_device = [max((root.txg for position, root in valid if REGION_DEVICE[position[0]] == device), default=0) for device in DEVICES]
    nonempty = sorted((root.txg for _position, root in valid if root.is_nonempty), reverse=True)
    fourth = nonempty[3] if len(nonempty) >= 4 else min(root.txg for _position, root in valid)
    return min(min(newest_per_device), fourth)


def scan_user_actions_after_rollback(isolation_reading, gen0_release, before_raise_lengths, after_raise_lengths):
    """放开用户动作扫（three-way-inference.md 2026-09-16 那条）：前缀固定为 A、B、重开写行暖机、C、回退到 A 与暖机；
    之后 k 个用户动作（每个是覆写或空发布）→ 抬 F 到当时的合法上限（比现行 F 大才抬）并推到两块盘都带上 → 再 m 个用户动作。
    每个状态判甲候post、候选根的槽有没有被换内容（假候选）、被抛弃根的槽有没有被换内容（D23 主句）。"""
    import itertools
    totals = {"序列数": 0, "状态数": 0, "甲候post非0状态": 0, "假候选状态": 0, "D23违例状态": 0, "含D23违例的序列": 0,
              "抬了F的序列": 0, "第一条D23违例序列": None, "第一条甲候post非0序列": None}
    for before_length in before_raise_lengths:
        for before_actions in itertools.product(["覆写", "空"], repeat=before_length):
            for after_length in after_raise_lengths:
                for after_actions in itertools.product(["覆写", "空"], repeat=after_length):
                    pool = Pool(Settings(isolation_reading=isolation_reading, gen0_tree_table_release=gen0_release))
                    pool.first_mount_warm_up()
                    pool.publish_first_file()
                    pool.publish_overwrite("B")
                    pool.remount_and_write_row()
                    pool.publish_overwrite("C")
                    pool.administrator_rollback(3)
                    sequence = list(before_actions)
                    judgements = []

                    def act(action):
                        if action == "覆写":
                            pool.publish_overwrite(f"覆写{pool.next_txg}")
                        else:
                            pool.publish_empty(f"空{pool.next_txg}")
                        judgements.append(judge_disk_state(pool))
                    for action in before_actions:
                        act(action)
                    target = legal_floor_upper_bound(pool)
                    if target > pool.current_floor:
                        totals["抬了F的序列"] += 1
                        sequence.append(f"抬F={target}")
                        root = pool.publish_empty(f"抬F{pool.next_txg}", new_floor=target)
                        judgements.append(judge_disk_state(pool))
                        covered = {REGION_DEVICE[root.position[0]]}
                        while covered != set(DEVICES):
                            root = pool.publish_empty(f"再推{pool.next_txg}")
                            covered.add(REGION_DEVICE[root.position[0]])
                            judgements.append(judge_disk_state(pool))
                    for action in after_actions:
                        sequence.append(action)
                        act(action)
                    totals["序列数"] += 1
                    totals["状态数"] += len(judgements)
                    nonzero = sum(1 for judgement in judgements if judgement["甲候post"] != 0)
                    violated = sum(1 for judgement in judgements if judgement["D23主句违例"])
                    totals["甲候post非0状态"] += nonzero
                    totals["假候选状态"] += sum(1 for judgement in judgements if judgement["走读(iii)换内容"])
                    totals["D23违例状态"] += violated
                    totals["含D23违例的序列"] += int(violated > 0)
                    if violated and totals["第一条D23违例序列"] is None:
                        totals["第一条D23违例序列"] = " ".join(sequence)
                    if nonzero and totals["第一条甲候post非0序列"] is None:
                        totals["第一条甲候post非0序列"] = " ".join(sequence)
    return totals


def history_table(gen0_release, failure_at, shown_txgs):
    """Y3 要的历史：每次发布的 txg、它覆写哪个根、释放了哪一代的哪些槽、两种记账时点的占着、盘上三种并集。"""
    pool = Pool(Settings(gen0_tree_table_release=gen0_release))
    pool.first_mount_warm_up()
    rows = [f"# m2 释放={gen0_release}；根槽写失败={failure_at}",
            "# txg | 覆写的根 | 本次释放（单元角色@槽, 分配代→释放代） | 分配时可再分配界 | 持久后可再分配界 | 占着pre | 占着post | 候选集并集 | 全部自证根并集"]
    pool.publish_first_file()
    while pool.next_txg <= max(shown_txgs):
        txg = pool.next_txg
        overwritten = pool.disk_ring.get(root_position_of(txg))
        before_version = dict(pool.current_version)
        threshold_before = pool.reclaim_threshold(pool.allocator_ring, pool.current_rows)
        failing = failure_at is not None and txg == failure_at
        root = pool.publish_overwrite(f"覆写{txg}", root_write=("fail_old",) if failing else ("ok",))
        if txg not in shown_txgs:
            continue
        released = ", ".join(f"{pool.units[identifier].role}@{pool.units[identifier].slot}({pool.units[identifier].allocation_txg}→{txg})"
                             for role, identifier in sorted(before_version.items()) if role in FILE_ROLES + FIXED_POINT_ROLES)
        if root is None:
            rows.append(f"{txg} | 写失败，槽里仍是 {overwritten.txg if overwritten else None} | （分配器退回，推进一格重发） | {threshold_before} | — | — | — | — | —")
            continue
        judgement = judge_disk_state(pool)
        roots = [candidate for candidate in pool.disk_ring.values() if candidate is not None]
        union_all = sum(span for _slot, span in {(pool.units[i].slot, pool.units[i].span) for r in roots for i in r.references.values()})
        threshold_after = pool.reclaim_threshold(pool.allocator_ring, pool.current_rows)
        rows.append(f"{txg} | {overwritten.txg if overwritten else None} | {released} | {threshold_before} | {threshold_after} | "
                    f"{root.ledger['pre'][0]} | {root.ledger['post'][0]} | {root.ledger['post'][0] - judgement['甲候post']} | {union_all}")
    return rows


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--selftest", action="store_true")
    parser.add_argument("--history", action="store_true")
    parser.add_argument("--scan", action="store_true")
    arguments = parser.parse_args()
    if arguments.scan:
        for isolation in ["narrow", "narrow_unit", "narrow_candidate"]:
            for release in ["first_transaction", "never"]:
                totals = scan_user_actions_after_rollback(isolation, release, range(0, 7), range(0, 5))
                print(f"影子账={isolation} m2释放={release} {totals}")
        print("SCAN-COMPLETE")
        return 0
    if arguments.history:
        output = history_table("never", None, [3, 4, 5, 24, 25, 26, 27, 28])
        output += history_table("first_transaction", None, [24, 25, 26, 27, 28])
        output += history_table("first_transaction", 30, [29, 30, 31, 32, 53, 54, 55])
        print("\n".join(output))
        return 0
    if arguments.selftest:
        failures, checked_count = selftest()
        for failure in failures:
            print("✗", failure)
        if failures:
            print("→ 模型改动让报告引的数对不上：先查是哪一处取法变了，再决定改报告还是改模型")
            return 1
        print(f"✓ selftest：{checked_count} 条断言全过（末尾三条是判别力自证：基线 0、记账改大 1 必红、候选根的槽换内容必报）")
        return 0
    output = run_all()
    output.append("RUN-COMPLETE")
    print("\n".join(output))
    return 0


if __name__ == "__main__":
    sys.exit(main())
