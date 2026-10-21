#!/usr/bin/env python3
"""m4-trunc-r2 攻方腿的纯算术小模型：截断、变长与扫描重建的认领判法，逐臂比扫描重建与遍历两侧的数据单元集合。

抄自工作树 ~/.local/share/singlefs/work/m4-predev 的这几处（行号在报告「模型抄的是哪几行」一节逐条列）：
- 单元数与声明长度：crates/singlefs-core/src/truncate.rs 的 data_units_a_size_needs、declared_length_within_a_size；
- 截断意图那次重写尾单元：truncate.rs 的 tail_data_unit_rewrite_of_a_truncation；变长那次：tail_data_unit_rewrite_of_an_extension；
- 批：truncate.rs 的 next_truncation_step；
- 认领：crates/singlefs-checker/src/rebuild.rs 的 newest_birth_txg_below_each_data_unit 与 rebuild_index_tables 末段
  （对象出生代、⌈size ÷ 净荷⌉.max(1) 之内、写序三条），谱系段 txgs_on_the_lineage_of_the_newest_version，当前版本按写序取最大；
- 遍历：crates/singlefs-checker/src/walk.rs 的 index_tables_walked_from_the_newest_root（现行 extent 树里、size 之内的单元）。
臂（甲、乙、丙及其变体）是这一轮正文 W1 的三条臂按报告里逐字写出的判法实现的，不是仓里的代码。
"""
import copy
import random
import sys

P = 32634  # 净荷容量：crates/singlefs-core/src/unit.rs 的 data_unit_payload_capacity，32768 − 105 − 29
BATCH_LEAF = 144  # UnlinkBatchSize::OneExtentLowerLeaf 一批放一片下段叶的单元数
ROLLBACK_CANDIDATES = 4  # 回退候选：最近几个已发布的版本（推的：根环里留四个状态）


def units_needed(size):
    """⌈size ÷ 净荷⌉.max(1)：truncate.rs data_units_a_size_needs；rebuild.rs 认领时同一个式子。"""
    return max(1, -(-size // P))


def ceil_units(size):
    """⌈size ÷ 净荷⌉（不取 max(1)）：「size 之外」的第一个单元号，甲的字段与乙的区间起点用它。"""
    return -(-size // P)


def declared(unit, size):
    """truncate.rs declared_length_within_a_size：min(size − 起点, 净荷)，起点在 size 之外是 0。"""
    return min(max(size - unit * P, 0), P)


def tail_rewrite_of_a_truncation(existing, size_before, target, always):
    """truncate.rs tail_data_unit_rewrite_of_a_truncation；always=True 是丙（k 存在就重写）。"""
    unit = units_needed(target) - 1
    if unit not in existing:
        return None
    if always or declared(unit, size_before) != declared(unit, target):
        return unit
    return None


def tail_rewrite_of_an_extension(existing, size_before, target):
    """truncate.rs tail_data_unit_rewrite_of_an_extension：最后一个存在的单元声明长度变大才补零重写。"""
    unit = max(existing)
    if declared(unit, size_before) < declared(unit, target):
        return unit
    return None


def next_truncation_step(existing_count, target, batch):
    """truncate.rs next_truncation_step：返回 (是不是最后一批, 留下几个)。"""
    kept_at_the_end = min(existing_count, units_needed(target))
    beyond = existing_count - kept_at_the_end
    if beyond <= batch:
        return True, kept_at_the_end
    return False, existing_count - batch


ARMS = {
    # 今天的实现：K6 三条（对象出生代、size 之内、写序）。
    "today": dict(),
    # 丙：截断意图那次留下的最后一个单元只要存在就重写；判法不动。
    "bing": dict(tail_always=True),
    # 甲一：字面读法「最近一次截断的 txg 与 size」，写序保留；整份写照抄字段。
    "jia_latest": dict(field="latest"),
    # 甲一不要写序：答 W2「写序那一条还要不要」。
    "jia_latest_no_write_order": dict(field="latest", write_order=False),
    # 甲二：水位读法，截断取 (这次 txg, min(这次 N, 上一个 N))，整份写清零字段；写序保留。「size 之外」取 ⌈N ÷ 净荷⌉.max(1) 起。
    "jia_watermark": dict(field="min_clear"),
    # 甲二而「size 之外」取 ⌈N ÷ 净荷⌉ 起（不取 max(1)）：截到 0 之后单元 0 落在「之外」。
    "jia_watermark_boundary_without_max_one": dict(field="min_clear", field_boundary="ceil"),
    # 甲三：水位取 min 而整份写不清零。
    "jia_watermark_no_clear": dict(field="min_noclear"),
    # 甲四：整份写也写字段 (这次 txg, 新长度)，截断取 min；写序去掉。
    "jia_full_no_write_order": dict(field="full", write_order=False),
    # 甲二的变异：变长那次重建记录时没带上字段（W4 必红世界「改坏那一句」的一种）。
    "jia_watermark_extension_drops_field": dict(field="min_clear", extension_drops_field=True),
    # 乙：截断意图那次写一条区间墓碑 [⌈N ÷ 净荷⌉ × 净荷, ∞)、死亡 txg = 意图 txg；写序保留。
    "yi": dict(tomb=True),
    # 乙不要写序。
    "yi_no_write_order": dict(tomb=True, write_order=False),
    # 乙全：截断、punch、整份写变短都写区间墓碑；写序去掉（W5 以后的入口用）。
    "yi_full_no_write_order": dict(tomb=True, tomb_on_whole_write=True, write_order=False),
    # 攒批（D16 已定项 10：一个 txg 装多次改动）下的几种改法：比较从出生 txg 换成写序 (实例, 事务号)，或「早于」放宽成「不晚于」。
    "today_by_write_order": dict(granularity="order"),
    "bing_by_write_order": dict(tail_always=True, granularity="order"),
    "jia_watermark_by_write_order": dict(field="min_clear", granularity="order"),
    "jia_watermark_not_later": dict(field="min_clear", compare="le"),
    "yi_by_write_order": dict(tomb=True, granularity="order"),
    "yi_not_later": dict(tomb=True, compare="le"),
    "yi_full_by_write_order": dict(tomb=True, tomb_on_whole_write=True, write_order=False, granularity="order"),
    # 乙而墓碑照 D18 已定项 10 回收规则初值回收：死亡已被全部还引用它的根送走（没有活快照、已出回退候选）就不再算。
    "yi_with_tombstones_recycled": dict(tomb=True, tomb_recycle=True),
}


ARM_DEFAULTS = dict(tail_always=False, field=None, write_order=True, tomb=False,
                    tomb_on_whole_write=False, extension_drops_field=False, field_boundary="needed",
                    granularity="txg", compare="lt", tomb_recycle=False)


def arm_option(arm, key):
    return ARMS[arm].get(key, ARM_DEFAULTS[key])


class FileState:
    """现行那一版里第一个文件的样子：size、extent 树里存在的单元、截断条目、甲的字段、活快照钉住的版本。"""

    def __init__(self):
        self.size = 0
        self.live = {}  # 单元号 -> 版本 (单元号, 出生 txg, 写序)
        self.truncation_target = None  # logged_ops 截断条目的目标 size（K1），没有截断在做时是 None
        self.field = (0, 0)  # 甲：inode 记录偏移 108 起的 (截断 txg, 截断 size)，全零是没写过
        self.snapshots = []  # 每个活快照钉住的版本集合


def lineage_segments(lineage):
    """rebuild.rs txgs_on_the_lineage_of_the_newest_version：按 T_rb 从大到小接父链，起点 2⁶⁴ − 1。"""
    segments = []
    walked_to = 2 ** 64 - 1
    for rollback_txg, target_txg in sorted(lineage, key=lambda r: -r[0]):
        assert target_txg < rollback_txg
        if rollback_txg > walked_to:
            continue
        segments.append((rollback_txg, walked_to))
        walked_to = target_txg
    segments.append((0, walked_to))
    return segments


class World:
    def __init__(self, arm, batch, destroy_samples=0, rng=None):
        self.arm, self.batch = arm, batch
        self.destroy_samples, self.rng = destroy_samples, rng
        self.txg = 3  # 第一次写的出生 txg 是 4（题面那一形）
        self.write_order = 0
        self.versions = set()  # 盘上写过的每一个单元版本（释放了也还在，除非被抽中「槽被复用」）
        self.tombstones = []  # 乙：(起点单元号, 止点单元号或 None 即 ∞, 死亡 txg)；墓碑树随现行那一版、只增
        self.lineage = []  # 谱系记录 (T_rb, T_old)，随现行那一版、只增
        self.file = FileState()
        self.published = []  # (txg, FileState 深拷贝, 标签)
        self.findings = []  # (标签, txg, 认回来的, 判死的, 抽掉的版本数)
        self.hold, self.hold_txg_taken = False, False

    def new_txg(self):
        """一次发布一个 txg；攒批（D16 已定项 10）时同一批里的几次改动共用一个 txg，只在批首加一。"""
        if self.hold and self.hold_txg_taken:
            return self.txg
        self.hold_txg_taken = self.hold
        self.txg += 1
        return self.txg

    def mark(self, txg):
        """臂记下的「这次放掉之前」的戳：按 txg（正文的读法）或按写序（这一次改动之前最后一个写序 + 1）。"""
        return txg if arm_option(self.arm, "granularity") == "txg" else self.write_order + 1

    def stamp(self, version):
        return version[1] if arm_option(self.arm, "granularity") == "txg" else version[2]

    def older_than(self, version, mark):
        if arm_option(self.arm, "compare") == "le":
            return self.stamp(version) <= mark
        return self.stamp(version) < mark

    def write_unit(self, unit, txg):
        self.write_order += 1
        version = (unit, txg, self.write_order)
        self.versions.add(version)
        self.file.live[unit] = version

    def publish(self, label):
        if self.hold:
            return
        self.published.append((self.txg, copy.deepcopy(self.file), label))
        self.check(label, frozenset())
        free = self.free_versions()
        for _ in range(self.destroy_samples if free else 0):
            destroyed = frozenset(v for v in free if self.rng.random() < 0.5)
            self.check(label, destroyed)

    def free_versions(self):
        """放掉了、槽可以被复用的版本：不在现行版本里、不被活快照钉住、不在最近几个回退候选里（defer 窗口）。"""
        held = set(self.file.live.values())
        for pinned in self.file.snapshots:
            held |= pinned
        for _, state, _ in self.published[-1 - ROLLBACK_CANDIDATES:]:
            held |= set(state.live.values())
            for pinned in state.snapshots:
                held |= pinned
        return sorted(self.versions - held)

    # ── 两侧的表 ──
    def rebuild_claims(self, destroyed):
        segments = lineage_segments(self.lineage)

        def on_lineage(txg):
            return any(low <= txg <= high for low, high in segments)

        current = {}
        for version in self.versions - destroyed:
            unit, birth, order = version
            if not on_lineage(birth):
                continue
            if unit not in current or order > current[unit][2]:
                current[unit] = version
        newest_below = {}
        newest_so_far = None
        for unit in sorted(current):
            if newest_so_far is not None:
                newest_below[unit] = newest_so_far
            stamp = self.stamp(current[unit])
            newest_so_far = stamp if newest_so_far is None else max(newest_so_far, stamp)
        need = units_needed(self.file.size)
        field_txg, field_size = self.file.field
        claimed = {}
        for unit in sorted(current):
            unit_, birth, _ = current[unit]
            if unit >= need:
                continue
            if arm_option(self.arm, "write_order") and newest_below.get(unit, -1) > self.stamp(current[unit]):
                continue
            field_first_unit = (ceil_units(field_size) if arm_option(self.arm, "field_boundary") == "ceil"
                                else units_needed(field_size))
            if (arm_option(self.arm, "field") and field_txg > 0 and unit >= field_first_unit
                    and self.older_than(current[unit], field_txg)):
                continue
            if arm_option(self.arm, "tomb") and any(
                    on_lineage(death_txg) and not self.tombstone_recycled(death_txg)
                    and start <= unit and (end is None or unit < end)
                    and self.older_than(current[unit], death_mark)
                    for start, end, death_txg, death_mark in self.tombstones):
                continue
            claimed[unit] = current[unit]
        return claimed

    def tombstone_recycled(self, death_txg):
        if not arm_option(self.arm, "tomb_recycle") or self.file.snapshots:
            return False
        if len(self.published) <= ROLLBACK_CANDIDATES:
            return False
        return death_txg <= self.published[-1 - ROLLBACK_CANDIDATES][0]

    def walk_table(self):
        need = units_needed(self.file.size)
        return {unit: v for unit, v in self.file.live.items() if unit < need}

    def check(self, label, destroyed):
        claimed, walked = self.rebuild_claims(destroyed), self.walk_table()
        resurrected = sorted((u, claimed[u]) for u in claimed if walked.get(u) != claimed[u])
        killed = sorted((u, walked[u]) for u in walked if claimed.get(u) != walked[u])
        if resurrected or killed:
            self.findings.append((label, self.txg, resurrected, killed, len(destroyed)))

    # ── 用户动作（UserChange 的几种与管理员回退）──
    def whole_write(self, length, label):
        """SequentialWrite / Overwrite：从偏移 0 整份写，单元 0..n−1 出生 txg 都是这一次，多出来的同一次放掉（transaction.rs publish_sequential_write）。"""
        txg = self.new_txg()
        mark = self.mark(txg)
        count = units_needed(length)
        before = set(self.file.live)
        for unit in sorted(before):
            if unit >= count:
                del self.file.live[unit]
        for unit in range(count):
            self.write_unit(unit, txg)
        self.file.size = length
        mode = arm_option(self.arm, "field")
        if mode == "min_clear":
            self.file.field = (0, 0)
        elif mode == "full":
            self.file.field = (mark, length)
        if arm_option(self.arm, "tomb_on_whole_write") and any(u >= count for u in before):
            self.tombstones.append((ceil_units(length), None, txg, mark))
        self.publish(label)

    def size_change(self, target):
        """UserChange::Truncate → transaction.rs publish_size_change：N = S 只改记录，N > S 变长，N < S 意图 + 各批 + 最后一批。"""
        size_before = self.file.size
        txg = self.new_txg()
        if target == size_before:
            self.publish("size_unchanged")
            return
        if target > size_before:
            unit = tail_rewrite_of_an_extension(set(self.file.live), size_before, target)
            if unit is not None:
                self.write_unit(unit, txg)
            self.file.size = target
            if arm_option(self.arm, "extension_drops_field"):
                self.file.field = (0, 0)
            self.publish(f"extension_to_{describe(target)}")
            return
        mark = self.mark(txg)
        unit = tail_rewrite_of_a_truncation(set(self.file.live), size_before, target,
                                            arm_option(self.arm, "tail_always"))
        if unit is not None:
            self.write_unit(unit, txg)
        self.file.size = target
        self.file.truncation_target = target
        mode = arm_option(self.arm, "field")
        old_txg, old_size = self.file.field
        if mode == "latest":
            self.file.field = (mark, target)
        elif mode in ("min_clear", "min_noclear", "full"):
            self.file.field = (mark, min(target, old_size) if old_txg > 0 else target)
        if arm_option(self.arm, "tomb"):
            self.tombstones.append((ceil_units(target), None, txg, mark))
        self.publish(f"truncation_intent_to_{describe(target)}")
        self.continue_truncation()

    def continue_truncation(self):
        """会话的走空：截断条目在就一批批从尾部放（next_truncation_step），最后一批删条目。"""
        while self.file.truncation_target is not None:
            self.new_txg()
            target = self.file.truncation_target
            beyond = sorted(u for u in self.file.live if u >= units_needed(target))
            final = len(beyond) <= self.batch
            released = beyond if final else beyond[-self.batch:]
            if sorted(self.file.live) == list(range(len(self.file.live))):
                # 今天的入口单元从 0 起连号：与 truncate.rs next_truncation_step 逐步相同（核一遍抄得对不对）
                code_final, code_kept = next_truncation_step(len(self.file.live), target, self.batch)
                assert (code_final, code_kept) == (final, len(self.file.live) - len(released))
            for u in released:
                del self.file.live[u]
            if final:
                self.file.truncation_target = None
            self.publish("truncation_final_batch" if final else "truncation_batch")

    def create_snapshot(self):
        self.new_txg()
        self.file.snapshots.append(frozenset(self.file.live.values()))
        self.publish("create_snapshot")

    def destroy_snapshot(self):
        if not self.file.snapshots:
            return False
        self.new_txg()
        self.file.snapshots.pop(0)
        self.publish("destroy_snapshot")
        return True

    def rollback(self, back):
        """管理员回退：取 back 次发布之前那一版（inode 树、extent 树、截断条目随 R_old），谱系追加 (T_rb, T_old)；带快照的版本上拒。"""
        if len(self.published) <= back:
            return False
        old_txg, old_state, _ = self.published[-1 - back]
        if self.file.snapshots or old_state.snapshots:
            return False
        rollback_txg = self.new_txg()
        self.lineage.append((rollback_txg, old_txg))
        self.file = copy.deepcopy(old_state)
        self.publish(f"rollback_to_txg_{old_txg}")
        self.continue_truncation()
        return True

    # ── 以后的入口（W5）：写到偏移 o 而不重写别的单元、打洞而不改 size ──
    def pwrite_unit(self, unit):
        txg = self.new_txg()
        new_size = max(self.file.size, (unit + 1) * P)
        if self.file.live:
            last = max(self.file.live)
            if last != unit and declared(last, self.file.size) < declared(last, new_size):
                self.write_unit(last, txg)
        self.write_unit(unit, txg)
        self.file.size = new_size
        self.publish(f"pwrite_unit_{unit}")

    def punch(self, start, end):
        txg = self.new_txg()
        mark = self.mark(txg)
        for u in [u for u in self.file.live if start <= u < end and u != 0]:
            del self.file.live[u]
        if arm_option(self.arm, "tomb"):
            self.tombstones.append((start, end, txg, mark))
        self.publish(f"punch_{start}_{end}")


def describe(size):
    whole, rest = divmod(size, P)
    return f"{whole}P" + (f"+{rest}" if rest else "")


def apply(world, op):
    kind = op[0]
    if kind == "seq":
        world.whole_write(op[1], f"sequential_write_{describe(op[1])}")
    elif kind == "ow":
        assert op[1] <= P, "Overwrite 只写一个数据单元以内的内容"
        world.whole_write(op[1], f"overwrite_{describe(op[1])}")
    elif kind == "size":
        world.size_change(op[1])
    elif kind == "snap":
        world.create_snapshot()
    elif kind == "unsnap":
        world.destroy_snapshot()
    elif kind == "rb":
        world.rollback(op[1])
    elif kind == "pwrite":
        world.pwrite_unit(op[1])
    elif kind == "punch":
        world.punch(op[1], op[2])
    elif kind == "group":
        world.hold, world.hold_txg_taken = True, False
        for inner in op[1]:
            apply(world, inner)
        world.hold = False
        world.publish("checkpoint_of_" + "+".join(describe_op(inner) for inner in op[1]))
    else:
        raise ValueError(op)


def run(arm, ops, batch, destroy_samples=0, rng=None):
    world = World(arm, batch, destroy_samples, rng)
    for op in ops:
        apply(world, op)
    return world


def describe_op(op):
    if op[0] in ("seq", "ow", "size"):
        return f"{op[0]}({describe(op[1])})"
    if op[0] == "group":
        return "checkpoint[" + "+".join(describe_op(inner) for inner in op[1]) + "]"
    return op[0] + ("(" + ",".join(str(x) for x in op[1:]) + ")" if len(op) > 1 else "")


def describe_finding(finding):
    label, txg, resurrected, killed, destroyed = finding
    parts = [f"state={label}@txg{txg}"]
    if resurrected:
        parts.append("resurrected=" + ";".join(f"unit{u}(birth{v[1]})" for u, v in resurrected[:3])
                     + (f";…{len(resurrected)}" if len(resurrected) > 3 else ""))
    if killed:
        parts.append("killed=" + ";".join(f"unit{u}(birth{v[1]})" for u, v in killed[:3])
                     + (f";…{len(killed)}" if len(killed) > 3 else ""))
    if destroyed:
        parts.append(f"destroyed_free_versions={destroyed}")
    return " ".join(parts)


def verdict_of(world):
    if not world.findings:
        return "holds"
    kinds = set()
    for _, _, resurrected, killed, _ in world.findings:
        if resurrected:
            kinds.add("resurrects")
        if killed:
            kinds.add("kills_live")
    return "+".join(sorted(kinds))


def p(units, extra=0):
    return units * P + extra


# W2 列的每种历史，加这条腿自己找的（名字写它是什么）。
FIXED_HISTORIES = {
    "fixed_history_of_the_body_300_units_cut_at_one_payload_then_3P_plus_1": [("seq", p(300)), ("size", p(1)), ("size", p(3, 1))],
    "cut_on_a_unit_boundary_then_extend": [("seq", p(3)), ("size", p(1)), ("size", p(3, 1))],
    "cut_inside_a_unit_then_extend": [("seq", p(3)), ("size", p(1, 7)), ("size", p(3, 1))],
    "cut_to_zero_then_extend": [("seq", p(3)), ("size", 0), ("size", p(3))],
    "cut_to_N1_extend_cut_to_N2_above_N1_in_the_hole_extend": [("seq", p(5)), ("size", p(1)), ("size", p(5)), ("size", p(3)), ("size", p(5))],
    "cut_then_whole_write_longer_then_cut_shorter_then_extend": [("seq", p(3)), ("size", p(1)), ("seq", p(5)), ("size", p(2)), ("size", p(5))],
    "cut_then_rollback_to_before_the_cut_then_extend": [("seq", p(3)), ("size", p(1)), ("size", p(3, 1)), ("rb", 3), ("size", p(4))],
    "rollback_into_the_middle_of_a_cut_then_extend": [("seq", p(5)), ("size", p(1)), ("rb", 1), ("size", p(5))],
    "cut_under_a_snapshot_then_extend_then_destroy_the_snapshot": [("seq", p(3)), ("snap",), ("size", p(1)), ("size", p(3, 1)), ("unsnap",), ("size", p(4))],
    "cut_extend_cut_lower_extend_cut_in_hole_extend": [("seq", p(6)), ("size", p(4)), ("size", p(6)), ("size", p(2)), ("size", p(6)), ("size", p(5)), ("size", p(6))],
    "overwrite_shorter_then_extend": [("seq", p(3)), ("ow", P // 2), ("size", p(3))],
    "whole_write_shorter_then_extend": [("seq", p(4)), ("seq", p(1)), ("size", p(4))],
    "cut_in_hole_twice_then_extend": [("seq", p(4)), ("size", p(1)), ("size", p(4)), ("size", p(3)), ("size", p(4)), ("size", p(2)), ("size", p(4))],
    "cut_then_publishes_past_the_rollback_window_then_extend": [("seq", p(3)), ("size", p(1)), ("snap",), ("unsnap",), ("snap",), ("unsnap",), ("size", p(3, 1))],
    "cut_then_whole_write_longer_then_cut_in_between": [("seq", p(2)), ("size", p(1)), ("seq", p(4)), ("size", p(3)), ("size", p(4))],
}


def report_fixed(out):
    for name, ops in FIXED_HISTORIES.items():
        for batch in (BATCH_LEAF, 1):
            for arm in ARMS:
                world = run(arm, ops, batch)
                first = describe_finding(world.findings[0]) if world.findings else "-"
                out.write(f"name=fixed history={name} batch={batch} arm={arm} states={len(world.published)} "
                          f"verdict={verdict_of(world)} first={first}\n")


# 枚举的字母表：今天的入口（SequentialWrite、Overwrite、Truncate 的三条、建删快照、管理员回退）。
ALPHABET = ([("seq", L) for L in (0, P // 2, p(1), p(2), p(4, 1))] + [("ow", P // 2)]
            + [("size", N) for N in (0, P // 2, p(1), p(1, 1), p(2), p(3), p(4), p(6))]
            + [("snap",), ("unsnap",), ("rb", 1), ("rb", 2), ("rb", 3)])
FIRST_WRITE = ("seq", p(4))
# 攒批：一个 checkpoint 里接连几次改动（一个 txg），今天的会话每次改动单独发布，造不出这一形。
CHECKPOINT_GROUPS = [
    ("group", (("seq", p(3)), ("size", p(1)))),
    ("group", (("size", p(1)), ("seq", p(3)))),
    ("group", (("seq", p(4, 1)), ("size", p(2)), ("size", p(4)))),
    ("group", (("size", p(1)), ("size", p(3, 1)))),
    ("group", (("seq", p(2)), ("size", 0))),
    ("group", (("size", p(2)), ("ow", P // 2))),
]
BATCHED_ALPHABET = ALPHABET + CHECKPOINT_GROUPS


def enumerate_histories(length, alphabet=None):
    alphabet = alphabet or ALPHABET

    def rec(prefix):
        if len(prefix) == length:
            yield prefix
            return
        for op in alphabet:
            yield from rec(prefix + [op])
    yield from rec([FIRST_WRITE])


def report_exhaustive(out, length, arms, alphabet=None, label="exhaustive"):
    for arm in arms:
        for batch in (BATCH_LEAF, 1):
            histories = states = 0
            counts = {}
            first = {}
            for ops in enumerate_histories(length, alphabet):
                world = run(arm, ops, batch)
                histories += 1
                states += len(world.published)
                verdict = verdict_of(world)
                counts[verdict] = counts.get(verdict, 0) + 1
                if verdict != "holds" and verdict not in first:
                    shortest = ops
                    first[verdict] = (shortest, describe_finding(world.findings[0]))
            out.write(f"name={label} length={length} batch={batch} arm={arm} histories={histories} states={states} "
                      + " ".join(f"{k}={v}" for k, v in sorted(counts.items())) + "\n")
            for verdict, (ops, finding) in sorted(first.items()):
                out.write(f"  first_{verdict}: ops=" + ",".join(describe_op(o) for o in ops) + f" {finding}\n")


def report_after_prefix(out, suffix_length, arms):
    """每条写死的历史只固定前缀（去掉最后一步），之后由用户决定的几步在今天的字母表上放开扫（攻方腿定义第 3 条）。"""
    suffixes = list(enumerate_histories(suffix_length + 1))
    for name, ops in FIXED_HISTORIES.items():
        prefix = ops[:-1]
        for arm in arms:
            counts = {}
            first = {}
            for suffix in suffixes:
                tail = suffix[1:]
                world = run(arm, prefix + tail, BATCH_LEAF)
                verdict = verdict_of(world)
                counts[verdict] = counts.get(verdict, 0) + 1
                if verdict != "holds" and verdict not in first:
                    first[verdict] = (tail, describe_finding(world.findings[0]))
            out.write(f"name=after_prefix history={name} suffix_length={suffix_length} arm={arm} histories={len(suffixes)} "
                      + " ".join(f"{k}={v}" for k, v in sorted(counts.items())) + "\n")
            for verdict, (tail, finding) in sorted(first.items()):
                out.write(f"  first_{verdict}: suffix=" + ",".join(describe_op(o) for o in tail) + f" {finding}\n")


def random_history(rng, length, alphabet):
    return [FIRST_WRITE] + [rng.choice(alphabet) for _ in range(length - 1)]


def report_random(out, count, length, seed, arms, alphabet=None, label="random", destroy_samples=2):
    alphabet = alphabet or ALPHABET
    for arm in arms:
        rng = random.Random(seed)
        histories = states = 0
        counts = {}
        first = {}
        for _ in range(count):
            ops = random_history(rng, length, alphabet)
            batch = rng.choice((BATCH_LEAF, 1, 2))
            world = run(arm, ops, batch, destroy_samples, rng)
            histories += 1
            states += len(world.published)
            verdict = verdict_of(world)
            counts[verdict] = counts.get(verdict, 0) + 1
            if verdict != "holds" and verdict not in first:
                first[verdict] = (ops, batch, describe_finding(world.findings[0]))
        out.write(f"name={label} seed={seed} length={length} arm={arm} histories={histories} states={states} "
                  + " ".join(f"{k}={v}" for k, v in sorted(counts.items())) + "\n")
        for verdict, (ops, batch, finding) in sorted(first.items()):
            out.write(f"  first_{verdict}: batch={batch} ops=" + ",".join(describe_op(o) for o in ops) + f" {finding}\n")


def report_pairs(out, label, histories, arm_pairs):
    """同一批历史上两条臂各自成立与否：A 成立而 B 不成立的条数，即「A 比 B 多罩了几条」。"""
    verdicts = {}
    for arm in {a for pair in arm_pairs for a in pair}:
        verdicts[arm] = [verdict_of(run(arm, ops, batch)) == "holds" for ops, batch in histories]
    for a, b in arm_pairs:
        only_a = [i for i in range(len(histories)) if verdicts[a][i] and not verdicts[b][i]]
        only_b = [i for i in range(len(histories)) if verdicts[b][i] and not verdicts[a][i]]
        out.write(f"name={label}_pair arm_a={a} arm_b={b} histories={len(histories)} "
                  f"a_holds_b_not={len(only_a)} b_holds_a_not={len(only_b)}\n")
        for tag, chosen in (("a_holds_b_not", only_a), ("b_holds_a_not", only_b)):
            if chosen:
                ops, batch = histories[chosen[0]]
                out.write(f"  first_{tag}: batch={batch} ops=" + ",".join(describe_op(o) for o in ops) + "\n")


PAIRS = [("jia_watermark", "bing"), ("jia_full_no_write_order", "bing"), ("yi", "bing"),
         ("yi_full_no_write_order", "jia_full_no_write_order"), ("jia_watermark_by_write_order", "bing_by_write_order")]


# W5：以后有 pwrite（写到偏移 o 而不重写别的单元）与 punch（打洞、不改 size）时。
FUTURE_HISTORIES = {
    "pwrite_into_the_middle_of_a_file": [("seq", p(4)), ("pwrite", 1)],
    "punch_then_extend_nothing_else": [("seq", p(4)), ("punch", 1, 3), ("size", p(4))],
    "punch_a_middle_range_then_cut_in_hole_then_extend": [("seq", p(6)), ("punch", 2, 4), ("size", p(5)), ("size", p(6))],
    "cut_then_pwrite_far_then_cut_then_pwrite_inside": [("seq", p(4)), ("size", p(1)), ("pwrite", 6), ("size", p(5)), ("pwrite", 3), ("size", p(4))],
}
FUTURE_ALPHABET = ALPHABET + [("pwrite", u) for u in (1, 3, 6)] + [("punch", 1, 3), ("punch", 2, 5)]


def report_future(out):
    for name, ops in FUTURE_HISTORIES.items():
        for arm in ARMS:
            world = run(arm, ops, BATCH_LEAF)
            first = describe_finding(world.findings[0]) if world.findings else "-"
            out.write(f"name=future history={name} arm={arm} verdict={verdict_of(world)} first={first}\n")


def main(argv):
    mode = argv[1] if len(argv) > 1 else "fixed"
    out = sys.stdout
    if mode == "list_arms":
        out.write("\n".join(ARMS) + "\n")
    elif mode == "fixed":
        report_fixed(out)
    elif mode == "exhaustive":
        arms = argv[3].split(",") if len(argv) > 3 else list(ARMS)
        report_exhaustive(out, int(argv[2]), arms)
    elif mode == "random":
        count, length, seed = int(argv[2]), int(argv[3]), int(argv[4])
        arms = argv[5].split(",") if len(argv) > 5 else list(ARMS)
        report_random(out, count, length, seed, arms)
    elif mode == "after_prefix":
        arms = argv[3].split(",") if len(argv) > 3 else list(ARMS)
        report_after_prefix(out, int(argv[2]), arms)
    elif mode == "checkpoint":
        arms = argv[3].split(",") if len(argv) > 3 else list(ARMS)
        report_exhaustive(out, int(argv[2]), arms, BATCHED_ALPHABET, "checkpoint_exhaustive")
    elif mode == "future":
        report_future(out)
        count, length, seed = int(argv[2]), int(argv[3]), int(argv[4])
        report_random(out, count, length, seed, list(ARMS), FUTURE_ALPHABET, "future_random", 1)
        rng = random.Random(seed)
        histories = [(random_history(rng, length, FUTURE_ALPHABET), BATCH_LEAF) for _ in range(count)]
        report_pairs(out, "future", histories, PAIRS)
    elif mode == "checkpoint_pairs":
        count, length, seed = int(argv[2]), int(argv[3]), int(argv[4])
        rng = random.Random(seed)
        histories = [(random_history(rng, length, BATCHED_ALPHABET), BATCH_LEAF) for _ in range(count)]
        report_pairs(out, "checkpoint", histories, PAIRS)
    else:
        raise SystemExit(f"认不出模式 {mode}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
