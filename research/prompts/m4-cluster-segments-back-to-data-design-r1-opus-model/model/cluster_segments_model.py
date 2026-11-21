#!/usr/bin/env python3
"""聚簇段还给数据的计数模型（设计轮 m4-cluster-segments-back-to-data-design-r1 攻方腿）。

只建分配器的段层行为，一块盘代表池（第一版两盘同槽：数据单元与提交内生块各盘落点相同）：
- 用户数据：每个数据单元两槽，取起点最小、偶数起点、两槽都空、所在段对数据开着的那一对（D3 已定项 8 第 1 条）。
- 提交内生块：每块一槽，从开放段游标起取最低空槽 bump；开放段装不下时按臂的规则换段（默认最低的全空段，
  allocator.rs 的 lowest_empty_segment），没有全空段就回落到全区最低空槽，记一次回落。
- 寿命：每次用户发布重写几棵「顺序树」的当前叶（每 period 次发布换一片新叶，旧叶最后那一版留下、长寿）、
  internal_blocks 个总被重写的块（树根、树表这类）、这次碰到的每一片分配记录叶（一片罩 12 段的槽；
  这次分配或释放落在它里面就重写它，固定点到不再碰到新叶为止）。
- 回收：释放之后隔 reclaim_lag 次发布才回到空闲（「释放代 ≤ 环里最旧有效根」那一截的近似）。
- 准入：前台写入发布写之前全空段数（按记录口径：段里没有活槽）< N₀ = 1 判合取 7 拒；数据落不下判段外槽对拒；
  提交内生块连回落都落不下判提交内生块拒。第一次拒就停这一趟。
臂见 ARMS。只读参数、只写标准输出，不读写盘。
"""
import argparse
import hashlib
import heapq
import sys
import time
from collections import deque

SLOTS_PER_SEGMENT = 64
PAIRS_PER_SEGMENT = SLOTS_PER_SEGMENT // 2
FREE = 0
LIVE = 1
DEFERRED = 2
FREE_BYTE = b"\x00"
EMPTY_SEGMENTS_FLOOR = 1
ALLOCATION_RECORD_LEAF_SEGMENTS = 12
REUSE_SMALLEST_FREE_SLOTS = 16
COMPACTION_LARGEST_LIVE_SLOTS = 16
TRAJECTORY_EVERY_USER_PUBLISHES = 4096

ARMS = {
    "today": "今日：这次挂载开过的段一律对数据关着，集合只进不出",
    "return_when_empty": "甲：已关的段没有活槽也没有等回收的槽就摘出集合",
    "return_at_most_one_live": "乙门槛一：已关的段活槽不超过 1 就摘出集合，混放",
    "return_at_most_two_live": "乙门槛二：已关的段活槽不超过 2 就摘出集合，混放",
    "close_only_the_open_segment": "戊：只关当前开放段，换段即还（乙门槛取无穷）",
    "compaction_empties_sparse_then_return": "丙：每次用户发布之后整理搬空一个稀疏的已关段，再按甲还（放宽本次挂载资格与签发前判，钉住照旧）",
    "commit_generated_reuses_sparse": "丁：换段时先复用这次挂载开过、空槽最多的段，集合只进不出",
    "reuse_sparse_and_return_when_empty": "丁加甲：换段时先复用这次挂载开过、空槽最多的段，已关的段全空就摘出集合（攻方提的组合，被攻过零轮）",
}

HISTORIES = {
    "pool_construction_like": {
        "what": "任务 20071 那一类：一次挂载里每次发布写一个数据单元，几棵顺序树的叶写完就不再碰",
        "segments": 2288,
        "data_units_per_publish": 1,
        "trees": [("extent_leaf", 17, True), ("inode_leaf", 29, True), ("mapping_leaf", 97, False)],
        "internal_blocks": 2,
        "max_user_publishes": 80000,
        "burst": None,
    },
    "long_lived_heavy": {
        "what": "长寿的提交内生块多：每次发布约一个旧叶留下，每段钉住约十个活槽",
        "segments": 2288,
        "data_units_per_publish": 1,
        "trees": [("extent_leaf", 2, True), ("inode_leaf", 3, True), ("directory_leaf", 5, True)],
        "internal_blocks": 2,
        "max_user_publishes": 80000,
        "burst": None,
    },
    "mapping_nodes_pin_the_segments": {
        "what": "长寿块是中央映射树的节点（整理当它钉住、不搬，带它的段没资格当 R）",
        "segments": 2288,
        "data_units_per_publish": 1,
        "trees": [("mapping_leaf", 7, False), ("extent_leaf", 1000, True)],
        "internal_blocks": 2,
        "max_user_publishes": 80000,
        "burst": None,
    },
    "metadata_burst_then_delete": {
        "what": "先一阵只建元数据（每次发布留三个长寿块），一次删掉它们，再每次发布写 16 个数据单元",
        "segments": 2288,
        "data_units_per_publish": 16,
        "trees": [("extent_leaf", 17, True), ("inode_leaf", 29, True)],
        "internal_blocks": 2,
        "max_user_publishes": 9000,
        "burst": {"publishes": 3000, "permanent_blocks_per_publish": 3},
    },
}

RECLAIM_LAGS = [4, 32]


class Refused(Exception):
    """这一趟第一次被拒：reason 是拒因。"""

    def __init__(self, reason):
        super().__init__(reason)
        self.reason = reason


class ModelInconsistency(AssertionError):
    """模型自己的计数对不上（模型的错，不是被测的臂的结论）。"""


def segment_of(slot):
    return slot // SLOTS_PER_SEGMENT


def record_leaf_of(slot):
    return segment_of(slot) // ALLOCATION_RECORD_LEAF_SEGMENTS


class SegmentLevelPool:
    """一块盘的单元区，按槽记 FREE / LIVE / DEFERRED，按段增量维护计数与「这次挂载开过的聚簇段」集合。"""

    def __init__(self, segments, arm, reclaim_lag, history):
        self.segments = segments
        self.arm = arm
        self.reclaim_lag = reclaim_lag
        self.history = history
        slots = segments * SLOTS_PER_SEGMENT
        self.slot_state = bytearray(slots)
        self.live_per_segment = [0] * segments
        self.blocked_per_segment = [0] * segments
        self.free_pairs_per_segment = [PAIRS_PER_SEGMENT] * segments
        self.data_slots_per_segment = [0] * segments
        self.unmovable_live_per_segment = [0] * segments
        self.in_cluster_set = bytearray(segments)
        self.opened_in_this_mount = []
        self.opened_flag = bytearray(segments)
        self.open_segment = None
        self.cursor = 0
        self.compaction_region = None
        self.free_pairs_open_to_data = segments * PAIRS_PER_SEGMENT
        self.empty_segments_by_records = segments
        self.holes_in_data_segments = 0
        self.data_heap = list(range(segments))
        self.empty_heap = list(range(segments))
        self.compaction_candidates = [[] for _ in range(COMPACTION_LARGEST_LIVE_SLOTS + 1)]
        self.reclaim_queue = deque()
        self.owner = {}
        self.leaf_slot = [None] * len(history["trees"])
        self.internal_slots = [None] * history["internal_blocks"]
        self.record_leaf_slot = {}
        self.permanent_slot = {}
        self.permanent_movable = {}
        self.permanent_tag = {}
        self.next_permanent_serial = 0
        self.publish_index = 0
        self.user_publishes = 0
        self.data_units = 0
        self.fallbacks = 0
        self.commit_generated_allocations = 0
        self.compaction_intents = 0
        self.issue_check_would_refuse = 0
        self.yield_predicate_false = 0
        self.empty_segments_at_last_issue = None
        self.touched_record_leaves = set()
        self.placement_digest = hashlib.sha256()

    # ── 槽的状态与按段计数 ──

    def set_slot_state(self, slot, new_state):
        old_state = self.slot_state[slot]
        if old_state == new_state:
            raise ModelInconsistency(f"槽 {slot} 已是状态 {new_state}")
        segment = segment_of(slot)
        pair = slot - slot % 2
        pair_was_free = self.slot_state[pair] == FREE and self.slot_state[pair + 1] == FREE
        self.slot_state[slot] = new_state
        pair_is_free = self.slot_state[pair] == FREE and self.slot_state[pair + 1] == FREE
        if pair_was_free != pair_is_free:
            change = 1 if pair_is_free else -1
            self.free_pairs_per_segment[segment] += change
            if not self.in_cluster_set[segment]:
                self.free_pairs_open_to_data += change
            if pair_is_free and self.free_pairs_per_segment[segment] == 1:
                heapq.heappush(self.data_heap, segment)
        if old_state == LIVE:
            self.live_per_segment[segment] -= 1
            if self.live_per_segment[segment] == 0:
                self.empty_segments_by_records += 1
        if new_state == LIVE:
            if self.live_per_segment[segment] == 0:
                self.empty_segments_by_records -= 1
            self.live_per_segment[segment] += 1
        was_blocked = old_state != FREE
        is_blocked = new_state != FREE
        if was_blocked != is_blocked:
            change = 1 if is_blocked else -1
            self.blocked_per_segment[segment] += change
            if self.data_slots_per_segment[segment] > 0:
                self.holes_in_data_segments -= change
            if self.blocked_per_segment[segment] == 0:
                heapq.heappush(self.empty_heap, segment)
        self.consider_leaving_the_cluster_set(segment)
        self.note_compaction_candidate(segment)

    # ── 「这次挂载开过的聚簇段」集合：开段时进，按臂的规则出 ──

    def leaves_the_cluster_set_now(self, segment):
        if segment == self.open_segment or not self.in_cluster_set[segment]:
            return False
        if self.arm in ("today", "commit_generated_reuses_sparse"):
            return False
        if self.arm in (
            "return_when_empty",
            "compaction_empties_sparse_then_return",
            "reuse_sparse_and_return_when_empty",
        ):
            return self.blocked_per_segment[segment] == 0
        if self.arm == "return_at_most_one_live":
            return self.live_per_segment[segment] <= 1
        if self.arm == "return_at_most_two_live":
            return self.live_per_segment[segment] <= 2
        if self.arm == "close_only_the_open_segment":
            return True
        raise ValueError(self.arm)

    def consider_leaving_the_cluster_set(self, segment):
        if segment is None or not self.leaves_the_cluster_set_now(segment):
            return
        self.in_cluster_set[segment] = 0
        self.free_pairs_open_to_data += self.free_pairs_per_segment[segment]
        heapq.heappush(self.data_heap, segment)

    def enter_the_cluster_set(self, segment):
        if self.in_cluster_set[segment]:
            return
        self.in_cluster_set[segment] = 1
        self.free_pairs_open_to_data -= self.free_pairs_per_segment[segment]
        if not self.opened_flag[segment]:
            self.opened_flag[segment] = 1
            self.opened_in_this_mount.append(segment)

    def open_new_segment(self, segment):
        previous = self.open_segment
        self.open_segment = segment
        self.cursor = segment * SLOTS_PER_SEGMENT
        self.enter_the_cluster_set(segment)
        self.consider_leaving_the_cluster_set(previous)
        self.note_compaction_candidate(previous)

    def note_compaction_candidate(self, segment):
        if self.arm != "compaction_empties_sparse_then_return" or segment is None:
            return
        live = self.live_per_segment[segment]
        if self.is_compaction_candidate(segment) and live <= COMPACTION_LARGEST_LIVE_SLOTS:
            heapq.heappush(self.compaction_candidates[live], segment)

    def is_compaction_candidate(self, segment):
        return (
            self.in_cluster_set[segment]
            and segment != self.open_segment
            and segment != self.compaction_region
            and self.live_per_segment[segment] > 0
            and self.unmovable_live_per_segment[segment] == 0
        )

    # ── 用户数据落点（D3 已定项 8 第 1 条） ──

    def pairs_open_to_data_outside_the_region(self):
        pairs = self.free_pairs_open_to_data
        region = self.compaction_region
        if region is not None and not self.in_cluster_set[region]:
            pairs -= self.free_pairs_per_segment[region]
        return pairs

    def place_a_data_unit(self):
        while self.data_heap:
            segment = self.data_heap[0]
            if (
                self.in_cluster_set[segment]
                or self.free_pairs_per_segment[segment] == 0
                or segment == self.compaction_region
            ):
                heapq.heappop(self.data_heap)
                continue
            if segment == self.open_segment:
                raise ModelInconsistency(f"开放段 {segment} 不在集合里")
            start = segment * SLOTS_PER_SEGMENT
            for pair in range(start, start + SLOTS_PER_SEGMENT, 2):
                if self.slot_state[pair] == FREE and self.slot_state[pair + 1] == FREE:
                    self.set_slot_state(pair, LIVE)
                    self.set_slot_state(pair + 1, LIVE)
                    if self.data_slots_per_segment[segment] == 0:
                        self.holes_in_data_segments += SLOTS_PER_SEGMENT - self.blocked_per_segment[segment]
                    self.data_slots_per_segment[segment] += 2
                    self.touched_record_leaves.add(record_leaf_of(pair))
                    self.placement_digest.update(pair.to_bytes(8, "little"))
                    self.data_units += 1
                    return pair
            raise ModelInconsistency(f"段 {segment} 记着有空对却找不到")
        return None

    # ── 提交内生块落点（D3 已定项 8 第 2 条、已定项 10 ①⑤） ──

    def lowest_free_slot_in(self, start, end):
        found = self.slot_state.find(FREE_BYTE, start, end)
        return None if found < 0 else found

    def lowest_empty_segment(self):
        while self.empty_heap:
            segment = self.empty_heap[0]
            if (
                self.blocked_per_segment[segment] == 0
                and segment != self.compaction_region
                and segment != self.open_segment
            ):
                return segment
            heapq.heappop(self.empty_heap)
        return None

    def most_free_segment_opened_in_this_mount(self):
        best = None
        best_free = REUSE_SMALLEST_FREE_SLOTS - 1
        for segment in self.opened_in_this_mount:
            if (
                not self.in_cluster_set[segment]
                or segment == self.open_segment
                or segment == self.compaction_region
            ):
                continue
            free = SLOTS_PER_SEGMENT - self.blocked_per_segment[segment]
            if free > best_free or (free == best_free and best is not None and segment < best):
                best = segment
                best_free = free
        return best

    def lowest_free_slot_outside_the_region(self):
        region = self.compaction_region
        found = self.lowest_free_slot_in(0, len(self.slot_state))
        if found is not None and region is not None and segment_of(found) == region:
            found = self.lowest_free_slot_in((region + 1) * SLOTS_PER_SEGMENT, len(self.slot_state))
        return found

    def commit_generated_slot(self):
        if self.open_segment is not None:
            end = (self.open_segment + 1) * SLOTS_PER_SEGMENT
            slot = self.lowest_free_slot_in(self.cursor, end)
            if slot is not None:
                self.cursor = slot + 1
                return slot
        segment = None
        if self.arm in ("commit_generated_reuses_sparse", "reuse_sparse_and_return_when_empty"):
            segment = self.most_free_segment_opened_in_this_mount()
        if segment is None:
            segment = self.lowest_empty_segment()
        if segment is not None:
            self.open_new_segment(segment)
            slot = self.lowest_free_slot_in(self.cursor, (segment + 1) * SLOTS_PER_SEGMENT)
            self.cursor = slot + 1
            return slot
        slot = self.lowest_free_slot_outside_the_region()
        if slot is None:
            return None
        previous = self.open_segment
        self.open_segment = None
        self.consider_leaving_the_cluster_set(previous)
        self.note_compaction_candidate(previous)
        self.fallbacks += 1
        return slot

    def owner_is_movable(self, owner):
        kind = owner[0]
        if kind == "leaf":
            return self.history["trees"][owner[1]][2]
        if kind == "permanent":
            return self.permanent_movable[owner[1]]
        if kind == "record":
            return True
        if kind == "internal":
            return False
        raise ValueError(owner)

    def allocate_commit_generated(self, owner):
        slot = self.commit_generated_slot()
        if slot is None:
            raise Refused("commit_generated_without_a_slot")
        segment = segment_of(slot)
        self.commit_generated_allocations += 1
        if not self.owner_is_movable(owner):
            self.unmovable_live_per_segment[segment] += 1
        self.owner[slot] = owner
        self.set_slot_state(slot, LIVE)
        self.touched_record_leaves.add(record_leaf_of(slot))
        return slot

    def release(self, slot):
        owner = self.owner.pop(slot)
        segment = segment_of(slot)
        if not self.owner_is_movable(owner):
            self.unmovable_live_per_segment[segment] -= 1
        self.set_slot_state(slot, DEFERRED)
        self.reclaim_queue.append((self.publish_index + self.reclaim_lag, slot))
        self.touched_record_leaves.add(record_leaf_of(slot))

    def reclaim_due(self):
        while self.reclaim_queue and self.reclaim_queue[0][0] <= self.publish_index:
            _, slot = self.reclaim_queue.popleft()
            self.set_slot_state(slot, FREE)

    def make_permanent(self, slot, movable, tag):
        serial = self.next_permanent_serial
        self.next_permanent_serial += 1
        self.permanent_slot[serial] = slot
        self.permanent_movable[serial] = movable
        self.permanent_tag[serial] = tag
        self.owner[slot] = ("permanent", serial)
        return serial

    def point_owner_at(self, owner, slot):
        kind = owner[0]
        if kind == "leaf":
            self.leaf_slot[owner[1]] = slot
        elif kind == "permanent":
            self.permanent_slot[owner[1]] = slot
        elif kind == "record":
            self.record_leaf_slot[owner[1]] = slot
        elif kind == "internal":
            self.internal_slots[owner[1]] = slot
        else:
            raise ValueError(owner)

    def rewrite_the_touched_record_leaves(self):
        rewritten = set()
        while True:
            pending = sorted(self.touched_record_leaves - rewritten)
            if not pending:
                return
            for leaf in pending:
                rewritten.add(leaf)
                old = self.record_leaf_slot.get(leaf)
                if old is not None:
                    self.release(old)
                self.record_leaf_slot[leaf] = self.allocate_commit_generated(("record", leaf))

    def rewrite_the_internal_blocks(self):
        for index, old in enumerate(self.internal_slots):
            if old is not None:
                self.release(old)
            self.internal_slots[index] = self.allocate_commit_generated(("internal", index))

    def allocate_permanent(self, movable, tag):
        serial = self.next_permanent_serial
        self.next_permanent_serial += 1
        self.permanent_movable[serial] = movable
        self.permanent_tag[serial] = tag
        self.permanent_slot[serial] = self.allocate_commit_generated(("permanent", serial))

    def release_permanent_blocks_tagged(self, tag):
        for serial in [serial for serial, kept in self.permanent_tag.items() if kept == tag]:
            self.release(self.permanent_slot.pop(serial))
            del self.permanent_movable[serial]
            del self.permanent_tag[serial]

    # ── 一次用户发布 ──

    def user_publish(self, data_units, is_foreground_write, permanent_blocks=0, delete_tag=None):
        self.publish_index += 1
        self.reclaim_due()
        if is_foreground_write and self.empty_segments_by_records < EMPTY_SEGMENTS_FLOOR:
            raise Refused("empty_cluster_segments_below_the_floor")
        if self.pairs_open_to_data_outside_the_region() < data_units:
            raise Refused("slot_pairs_outside_the_cluster_segments_for_data_units")
        self.touched_record_leaves = set()
        for _ in range(data_units):
            if self.place_a_data_unit() is None:
                raise ModelInconsistency("准入数过的对落不下")
        self.user_publishes += 1
        for index, (_, period, movable) in enumerate(self.history["trees"]):
            old = self.leaf_slot[index]
            if old is not None:
                if self.user_publishes % period == 0:
                    self.make_permanent(old, movable, "tree")
                else:
                    self.release(old)
            self.leaf_slot[index] = self.allocate_commit_generated(("leaf", index))
        for _ in range(permanent_blocks):
            self.allocate_permanent(True, "burst")
        if delete_tag is not None:
            self.release_permanent_blocks_tagged(delete_tag)
        self.rewrite_the_internal_blocks()
        self.rewrite_the_touched_record_leaves()

    # ── 丙：整理搬空一个稀疏的已关段（一条意图 = 签发一次发布 + 搬迁一批并删意图一次发布） ──

    def choose_sparse_region(self):
        for live in range(1, COMPACTION_LARGEST_LIVE_SLOTS + 1):
            heap = self.compaction_candidates[live]
            while heap:
                segment = heap[0]
                if self.is_compaction_candidate(segment) and self.live_per_segment[segment] == live:
                    return segment
                heapq.heappop(heap)
        return None

    def compaction_publish(self, slots_to_move):
        self.publish_index += 1
        self.reclaim_due()
        self.touched_record_leaves = set()
        for slot in slots_to_move:
            owner = self.owner[slot]
            new_slot = self.allocate_commit_generated(owner)
            self.owner[slot] = owner
            self.release(slot)
            self.point_owner_at(owner, new_slot)
        self.rewrite_the_internal_blocks()
        self.rewrite_the_touched_record_leaves()

    def compaction_intent(self):
        region = self.choose_sparse_region()
        if region is None:
            return
        units_to_move = self.live_per_segment[region]
        holes = self.holes_in_data_segments
        if self.data_slots_per_segment[region] > 0:
            holes -= SLOTS_PER_SEGMENT - self.blocked_per_segment[region]
        if holes < units_to_move:
            self.issue_check_would_refuse += 1
        if (
            self.empty_segments_at_last_issue is not None
            and self.empty_segments_by_records <= self.empty_segments_at_last_issue
        ):
            self.yield_predicate_false += 1
        self.empty_segments_at_last_issue = self.empty_segments_by_records
        self.compaction_region = region
        try:
            self.compaction_publish([])
            start = region * SLOTS_PER_SEGMENT
            live_slots = [
                slot for slot in range(start, start + SLOTS_PER_SEGMENT) if self.slot_state[slot] == LIVE
            ]
            self.compaction_publish(live_slots)
        finally:
            self.compaction_region = None
            heapq.heappush(self.data_heap, region)
            heapq.heappush(self.empty_heap, region)
            self.note_compaction_candidate(region)
        self.compaction_intents += 1

    # ── 被拒那一刻的静态读数（今日臂）：各候选此刻各放出多少 ──

    def static_release_at_this_moment(self):
        histogram = {}
        returned = {"return_when_empty": 0, "at_most_one_live": 0, "at_most_two_live": 0, "every_closed": 0}
        returned_once_reclaimed = dict(returned)
        for segment in self.opened_in_this_mount:
            if not self.in_cluster_set[segment] or segment == self.open_segment:
                continue
            live = self.live_per_segment[segment]
            histogram[live] = histogram.get(live, 0) + 1
            start = segment * SLOTS_PER_SEGMENT
            pairs_without_live = sum(
                1
                for pair in range(start, start + SLOTS_PER_SEGMENT, 2)
                if self.slot_state[pair] != LIVE and self.slot_state[pair + 1] != LIVE
            )
            free_pairs = self.free_pairs_per_segment[segment]
            for name, qualifies in (
                ("return_when_empty", self.blocked_per_segment[segment] == 0),
                ("at_most_one_live", live <= 1),
                ("at_most_two_live", live <= 2),
                ("every_closed", True),
            ):
                if qualifies:
                    returned[name] += free_pairs
            for name, qualifies in (
                ("return_when_empty", live == 0),
                ("at_most_one_live", live <= 1),
                ("at_most_two_live", live <= 2),
                ("every_closed", True),
            ):
                if qualifies:
                    returned_once_reclaimed[name] += pairs_without_live
        return histogram, returned, returned_once_reclaimed

    # ── 模型自检：增量计数与逐槽重数对得上；集合规则守住 ──

    def check_the_model(self):
        for segment in range(self.segments):
            start = segment * SLOTS_PER_SEGMENT
            states = self.slot_state[start : start + SLOTS_PER_SEGMENT]
            live = states.count(LIVE)
            blocked = SLOTS_PER_SEGMENT - states.count(FREE)
            pairs = sum(1 for offset in range(0, SLOTS_PER_SEGMENT, 2) if states[offset] == FREE and states[offset + 1] == FREE)
            if (live, blocked, pairs) != (
                self.live_per_segment[segment],
                self.blocked_per_segment[segment],
                self.free_pairs_per_segment[segment],
            ):
                raise ModelInconsistency(f"段 {segment} 的计数与逐槽重数不同")
            if self.in_cluster_set[segment] and self.data_slots_per_segment[segment] > 0 and self.arm in (
                "today",
                "commit_generated_reuses_sparse",
            ):
                raise ModelInconsistency(f"今日规则下集合里的段 {segment} 装着数据")
        open_to_data = sum(
            self.free_pairs_per_segment[segment] for segment in range(self.segments) if not self.in_cluster_set[segment]
        )
        if open_to_data != self.free_pairs_open_to_data:
            raise ModelInconsistency("对数据开着的空对计数与重数不同")
        empty_by_records = sum(1 for live in self.live_per_segment if live == 0)
        if empty_by_records != self.empty_segments_by_records:
            raise ModelInconsistency("记录口径全空段数与重数不同")
        if self.arm in ("today", "commit_generated_reuses_sparse"):
            if sum(self.in_cluster_set) != len(self.opened_in_this_mount):
                raise ModelInconsistency("今日规则下集合出过段")
        if self.open_segment is not None and not self.in_cluster_set[self.open_segment]:
            raise ModelInconsistency("开放段不在集合里")


def histogram_text(histogram):
    return ",".join(f"{live}:{count}" for live, count in sorted(histogram.items())) or "none"


def run_one(history_name, arm, reclaim_lag, user_publish_limit=None, emit=print):
    history = HISTORIES[history_name]
    pool = SegmentLevelPool(history["segments"], arm, reclaim_lag, history)
    limit = history["max_user_publishes"] if user_publish_limit is None else user_publish_limit
    burst = history["burst"]
    prefix = f"history={history_name} reclaim_lag={reclaim_lag} arm={arm}"
    refusal = "none"
    refusal_publish = 0
    started = time.monotonic()
    try:
        if burst is not None:
            for _ in range(burst["publishes"]):
                pool.user_publish(0, True, permanent_blocks=burst["permanent_blocks_per_publish"])
            emit(
                f"name=cluster_after_burst {prefix} user_publishes={pool.user_publishes} cluster_set={sum(pool.in_cluster_set)} "
                f"opened_in_this_mount={len(pool.opened_in_this_mount)}"
            )
            pool.user_publish(0, False, delete_tag="burst")
            emit(
                f"name=cluster_after_delete {prefix} user_publishes={pool.user_publishes} cluster_set={sum(pool.in_cluster_set)} "
                f"pairs_open_to_data={pool.pairs_open_to_data_outside_the_region()} empty_segments={pool.empty_segments_by_records}"
            )
        while pool.user_publishes < limit:
            pool.user_publish(history["data_units_per_publish"], True)
            if arm == "compaction_empties_sparse_then_return":
                pool.compaction_intent()
            if pool.user_publishes % TRAJECTORY_EVERY_USER_PUBLISHES == 0:
                emit(
                    f"name=cluster_trajectory {prefix} user_publishes={pool.user_publishes} data_units={pool.data_units} "
                    f"cluster_set={sum(pool.in_cluster_set)} opened_in_this_mount={len(pool.opened_in_this_mount)} "
                    f"pairs_open_to_data={pool.pairs_open_to_data_outside_the_region()} "
                    f"empty_segments={pool.empty_segments_by_records} fallbacks={pool.fallbacks}"
                )
    except Refused as refused:
        refusal = refused.reason
        refusal_publish = pool.user_publishes + 1
    pool.check_the_model()
    seconds = time.monotonic() - started
    emit(
        f"name=cluster_result {prefix} refusal={refusal} refused_at_user_publish={refusal_publish} "
        f"user_publishes={pool.user_publishes} data_units={pool.data_units} cluster_set={sum(pool.in_cluster_set)} "
        f"opened_in_this_mount={len(pool.opened_in_this_mount)} pairs_open_to_data={pool.pairs_open_to_data_outside_the_region()} "
        f"empty_segments={pool.empty_segments_by_records} fallbacks={pool.fallbacks} "
        f"commit_generated_slots_per_publish={pool.commit_generated_allocations / max(1, pool.publish_index):.3f} "
        f"compaction_intents={pool.compaction_intents} issue_check_would_refuse={pool.issue_check_would_refuse} "
        f"yield_predicate_false={pool.yield_predicate_false} placement_digest={pool.placement_digest.hexdigest()[:16]}"
    )
    if arm == "today":
        histogram, returned, returned_once_reclaimed = pool.static_release_at_this_moment()
        emit(
            f"name=cluster_static_release_at_the_end {prefix} closed_live_histogram={histogram_text(histogram)} "
            + " ".join(f"free_pairs_{name}={pairs}" for name, pairs in returned.items())
            + " "
            + " ".join(f"pairs_without_live_{name}={pairs}" for name, pairs in returned_once_reclaimed.items())
        )
    emit(f"timing {prefix} seconds={seconds:.2f} user_publishes={pool.user_publishes}")
    return pool


def planned_runs(histories, arms, reclaim_lags):
    return [(history, arm, lag) for history in histories for lag in reclaim_lags for arm in arms]


def estimate_minutes(runs):
    sample_publishes = 2000
    started = time.monotonic()
    run_one("pool_construction_like", "today", 32, user_publish_limit=sample_publishes, emit=lambda line: None)
    seconds_per_publish = (time.monotonic() - started) / sample_publishes
    weighted_publishes = 0
    for history, arm, _ in runs:
        spec = HISTORIES[history]
        publishes = spec["max_user_publishes"] + (spec["burst"]["publishes"] if spec["burst"] else 0)
        weight = 3 if arm == "compaction_empties_sparse_then_return" else 1
        weighted_publishes += publishes * weight * max(1, spec["data_units_per_publish"] // 4)
    return seconds_per_publish, weighted_publishes * seconds_per_publish / 60


def main():
    parser = argparse.ArgumentParser(description="聚簇段还给数据的计数模型")
    parser.add_argument("--history", action="append", choices=sorted(HISTORIES))
    parser.add_argument("--arm", action="append", choices=list(ARMS))
    parser.add_argument("--reclaim-lag", action="append", type=int)
    parser.add_argument("--estimate", action="store_true", help="只跑一小段，打出按它估的全量挂钟（分钟）")
    arguments = parser.parse_args()
    histories = arguments.history or list(HISTORIES)
    arms = arguments.arm or list(ARMS)
    reclaim_lags = arguments.reclaim_lag or RECLAIM_LAGS
    runs = planned_runs(histories, arms, reclaim_lags)
    seconds_per_publish, minutes = estimate_minutes(runs)
    print(
        f"name=estimate runs={len(runs)} seconds_per_user_publish={seconds_per_publish:.6f} "
        f"estimated_minutes_upper_bound={minutes:.1f}"
    )
    if arguments.estimate:
        return 0
    for arm, what in ARMS.items():
        print(f"arm {arm}：{what}")
    for history, spec in HISTORIES.items():
        print(f"history {history}：{spec['what']}")
    for history, arm, lag in runs:
        run_one(history, arm, lag)
    if "pool_construction_like" in histories and "return_when_empty" in arms and 32 in reclaim_lags:
        digests = []
        for _ in range(2):
            lines = []
            run_one("pool_construction_like", "return_when_empty", 32, emit=lines.append)
            digests.append([line for line in lines if line.startswith("name=cluster_result")])
        same = "yes" if digests[0] == digests[1] else "no"
        print(f"name=cluster_determinism history=pool_construction_like arm=return_when_empty reclaim_lag=32 two_runs_identical={same}")
        if same != "yes":
            return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
