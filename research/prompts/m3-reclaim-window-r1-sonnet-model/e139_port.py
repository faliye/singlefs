#!/usr/bin/env python3
"""E139 装置（scripts/experiments/e139_tightened_floor.rs）活化一族世界的 Python 端口。
只端口 G2SW / G2SW_ringgate 在 activation_faults / activation_crash_no_loss / skip_then_raise 三族里走到的那部分。
目的：先逐行复现产物 research/results/e139-tightened-floor-2026-10-04.out 的 w_activation 行（fakes / unrecoverable / checks），
再在副本上一次只改一处。端口与 Rust 装置不共用代码，行号引自 Rust 装置。
"""
import sys, bisect, heapq
from collections import deque

R = 3
DISKS = 2
USER_BLOCKS = 8
META = 5
CAP = 4000
MIN_RETAIN = 4
M64 = (1 << 64) - 1
KIND_UNUSED, KIND_LIVE, KIND_FREED = 0, 1, 2


STATS = {'p4_calls': 0, 'p4_flag_disagree': 0}


class Cfg:
    """变体开关。默认值 = 装置原样（G2SW）。"""
    def __init__(self, **kw):
        self.gate = 'window'        # window | ring | alow | ext<N> | holes
        self.failure = 'advance'    # advance（装置，D23 目标形态） | resend（实现今天：原样重发同一个 txg）
        self.recover = 'chosen'     # chosen（装置：恢复到择中的根） | replay（实现：择中的根之上再施加记录，现行版 = 最新一版）
        self.p2 = False             # 抬 F 上限的有效根另滤 txg >= 今天的 F（实现 mount.rs 第 2247 行）
        self.p3 = False             # 生效 / 候选下界取条款的 SysPre：加系统配置里的 F、被抛弃根的 F
        self.p4 = False             # 非空按树根指针比、按内容去重（条款），不用真值集合
        self.p3c = False            # 只改候选下界（回退候选集 txg >= F_生效 取条款的 SysPre）
        self.p3s = False            # 只改恢复之后生效的 F（取条款的 SysPre）
        for k, v in kw.items():
            if not hasattr(self, k):
                raise KeyError(k)
            setattr(self, k, v)

    def tag(self):
        return ','.join(f'{k}={v}' for k, v in sorted(vars(self).items()))


class PF:
    __slots__ = ('value', 'carrier', 'durable')
    def __init__(self, value, carrier, durable):
        self.value, self.carrier, self.durable = value, carrier, durable


class Metrics:
    __slots__ = ('fake', 'unrec', 'checks', 'failures_injected', 'retention', 'stalls', 'max_pubs', 'false_enospc_arm', 'false_enospc_raw',
                 'true_enospc', 'write_after_delete_failed', 'pressure', 'forced', 'user_windows')
    def __init__(self):
        self.fake = 0; self.unrec = 0; self.checks = 0; self.failures_injected = 0; self.retention = 0
        self.stalls = 0; self.max_pubs = 0; self.false_enospc_arm = 0; self.false_enospc_raw = 0
        self.true_enospc = 0; self.write_after_delete_failed = 0; self.pressure = 0; self.forced = 0; self.user_windows = 0


class Sim:
    def __init__(self, cfg, S, seed, mount_cost=1, declared=5, keep_snapshots=True):
        self.cfg = cfg
        self.S = S
        self.keep_snapshots = keep_snapshots
        self.counting_window = 0
        self.broken_slot = None
        self.ring_depth = R * S
        # Rust 第 630–674 行 Simulation::new；窗口臂 G2SW 的公式开销 c_max = 5，挂载开销 c0 = 1，界 B = 4 + 2*2 = 8
        self.empty_cost = mount_cost
        bound = 4 + 2 * 2
        self.reserve = 2 * META + (bound - 1) * declared
        self.push_residue = META + (bound - 2) * declared
        self.max_empty_pubs = bound - 1
        self.rs = ((seed * 0x9E3779B97F4A7C15) & M64) | 1
        self.kind = bytearray(CAP)
        self.alloc = [0] * CAP
        self.freed = [0] * CAP
        self.by_gen = {}          # freed_at -> sorted list of block idx
        self.unused_count = CAP
        self.unused_ptr = 0
        self.objects = []
        self.metadata = []
        self.damaged = set()
        seed_root = (0, 1, 0, True)   # (txg, instance, floor, self_verified)
        self.ring = [seed_root] * self.ring_depth
        self.disk_lost = [False, False]
        self.instance = 1
        self.instance_rows = []
        self.gate_anchor = 0
        self.cand_anchor = 0
        self.newest_durable = 0
        self.max_published = 0
        self.next_txg = 1
        self.floor_carried = 0
        self.floor_active = 0
        self.sysconfig_floor = 0
        self.pending = None
        self.snapshots = deque()
        self.planned = set()
        self.root_write_attempts = 0
        self.fail_next = False
        self.cwcus = False
        self.nonempty = {0}
        self.empty = set()
        self.halted = False
        self.unpublished_txg = None
        self.state_key = {}
        self.m = Metrics()

    # ---- 克隆（前缀拷贝）----
    def clone(self):
        o = Sim.__new__(Sim)
        for k in Sim.__init__.__code__.co_names:  # 不可靠，下面显式拷
            pass
        o.cfg = self.cfg; o.S = self.S; o.ring_depth = self.ring_depth
        o.empty_cost = self.empty_cost; o.reserve = self.reserve; o.push_residue = self.push_residue
        o.max_empty_pubs = self.max_empty_pubs; o.rs = self.rs
        o.keep_snapshots = self.keep_snapshots; o.counting_window = self.counting_window; o.broken_slot = self.broken_slot
        o.kind = bytearray(self.kind); o.alloc = list(self.alloc); o.freed = list(self.freed)
        o.by_gen = {g: list(v) for g, v in self.by_gen.items()}
        o.unused_count = self.unused_count; o.unused_ptr = self.unused_ptr
        o.objects = list(self.objects); o.metadata = list(self.metadata)
        o.damaged = set(self.damaged)
        o.ring = list(self.ring); o.disk_lost = list(self.disk_lost)
        o.instance = self.instance; o.instance_rows = list(self.instance_rows)
        o.gate_anchor = self.gate_anchor; o.cand_anchor = self.cand_anchor
        o.newest_durable = self.newest_durable; o.max_published = self.max_published
        o.next_txg = self.next_txg
        o.floor_carried = self.floor_carried; o.floor_active = self.floor_active
        o.sysconfig_floor = self.sysconfig_floor
        o.pending = None if self.pending is None else PF(self.pending.value, self.pending.carrier, self.pending.durable)
        o.snapshots = deque(self.snapshots)
        o.planned = set(self.planned); o.root_write_attempts = self.root_write_attempts
        o.fail_next = self.fail_next; o.cwcus = self.cwcus
        o.nonempty = set(self.nonempty); o.empty = set(self.empty)
        o.halted = self.halted; o.unpublished_txg = self.unpublished_txg
        o.state_key = dict(self.state_key)
        o.m = Metrics()
        return o

    # ---- 随机 ----
    def next_random(self):
        x = self.rs
        x ^= x >> 12
        x ^= (x << 25) & M64
        x ^= x >> 27
        self.rs = x
        return (x * 0x2545F4914F6CDD1D) & M64

    # ---- 槽 ----
    def slot_index(self, txg):
        return (txg % R) * self.S + (txg // R) % self.S

    def disk_of_slot(self, slot):
        return 1 if slot // self.S == 1 else 0

    def survives(self, slot):
        return not self.disk_lost[self.disk_of_slot(slot)]

    def surviving_disks(self):
        return [d for d in range(DISKS) if not self.disk_lost[d]]

    def any_lost(self):
        return any(self.disk_lost)

    def is_instance_valid(self, root):
        limits = [lim for (inst, lim) in self.instance_rows if inst == root[1]]
        if not limits:
            return True
        return root[0] <= min(limits)

    def visible_roots(self):
        out = []
        for slot, r in enumerate(self.ring):
            if r is not None and self.survives(slot) and r[3]:
                out.append(r)
        return out

    def valid_roots_with_disk(self):
        out = []
        for slot, r in enumerate(self.ring):
            if r is not None and self.survives(slot) and r[3] and self.is_instance_valid(r):
                out.append((self.disk_of_slot(slot), r))
        return out

    def valid_roots(self):
        return [r for _, r in self.valid_roots_with_disk()]

    def oldest_persisted_valid_txg(self):
        v = self.valid_roots()
        return min(r[0] for r in v) if v else 0

    def newest_valid_root(self):
        v = self.valid_roots()
        return max(v, key=lambda r: (r[1], r[0])) if v else None

    def newest_valid_txg_on_disk(self, disk):
        t = [r[0] for d, r in self.valid_roots_with_disk() if d == disk]
        return max(t) if t else None

    def newest_valid_on_every_surviving_disk(self):
        s = self.surviving_disks()
        if not s:
            return 0
        return min((self.newest_valid_txg_on_disk(d) or 0) for d in s)

    # ---- 窗口 ----
    def in_window(self, txg, anchor, length):
        return txg + length > anchor

    def in_cand_window(self, txg):
        return self.in_window(txg, self.cand_anchor, self.ring_depth)

    def gate_window_length_and_anchor(self):
        g = self.cfg.gate
        if g == 'alow':
            return self.newest_valid_on_every_surviving_disk(), self.ring_depth
        if g == 'alowr':
            newest = {}
            for slot, r in enumerate(self.ring):
                if r is not None and self.survives(slot) and r[3] and self.is_instance_valid(r):
                    reg = slot // self.S
                    newest[reg] = max(newest.get(reg, 0), r[0])
            return (min(newest.values()) if len(newest) == R else 0), self.ring_depth
        if g.startswith('ext'):
            return self.gate_anchor, self.ring_depth + int(g[3:])
        if g == 'holes':
            n = self.gate_anchor
            present = {r[0] for r in self.ring if r is not None}
            holes = sum(1 for t in range(max(0, n - self.ring_depth + 1), n + 1) if t not in present)
            return n, self.ring_depth + 2 + 3 * holes
        return self.gate_anchor, self.ring_depth

    def oldest_valid_for_gate(self):
        if self.cfg.gate == 'ring':
            return self.oldest_persisted_valid_txg()
        anchor, length = self.gate_window_length_and_anchor()
        t = [r[0] for r in self.valid_roots() if self.in_window(r[0], anchor, length)]
        return min(t) if t else 0

    # ---- 非空 ----
    def content_key(self, root):
        return self.state_key.get((root[1], root[0]))

    def nonempty_flags(self, roots):
        """roots 是候选的有效根（已按窗口、F 滤过）。返回 txg -> 是否非空。
        真值：装置的 nonempty 集合。条款（p4）：与前一条有效根的树根指针比。"""
        truth = {r[0]: (r[0] in self.nonempty) for r in roots}
        if not self.cfg.p4:
            return truth
        srt = sorted(roots, key=lambda r: (r[0], r[1]))
        out = {}
        prev_key = None
        for i, r in enumerate(srt):
            k = self.content_key(r)
            out[r[0]] = (i == 0) or (k != prev_key)
            prev_key = k
        STATS['p4_calls'] += 1
        if out != truth:
            STATS['p4_flag_disagree'] += 1
        return out

    def distinct_nonempty_newest_first(self, roots):
        flags = self.nonempty_flags(roots)
        srt = sorted(roots, key=lambda r: (r[0], r[1]), reverse=True)
        out = []
        seen = set()
        for r in srt:
            if not flags[r[0]]:
                continue
            if self.cfg.p4:
                k = self.content_key(r)
                if k in seen:
                    continue
                seen.add(k)
                out.append(r[0])
            else:
                if r[0] not in out:
                    out.append(r[0])
        return out

    # ---- W3 抬 F 的上限（读法 P）----
    def ceiling_roots(self):
        roots = self.valid_roots_with_disk()
        if not roots:
            return []
        anchor = max(r[0] for _, r in roots)
        out = [(d, r) for d, r in roots if self.in_window(r[0], anchor, self.ring_depth)]
        if self.cfg.p2:
            out = [(d, r) for d, r in out if r[0] >= self.floor_carried]
        return out

    def fourth_newest_nonempty_in_window(self):
        rs = self.ceiling_roots()
        roots = [r for _, r in rs]
        distinct = self.distinct_nonempty_newest_first(roots)
        if len(distinct) >= MIN_RETAIN:
            return distinct[MIN_RETAIN - 1]
        return min((r[0] for r in roots), default=0)

    def newest_on_every_surviving_disk_in_window(self):
        rs = self.ceiling_roots()
        vals = []
        for d in self.surviving_disks():
            t = [r[0] for dd, r in rs if dd == d]
            if t:
                vals.append(max(t))
        return min(vals, default=0)

    def upper_limit(self):
        return min(self.fourth_newest_nonempty_in_window(), self.newest_on_every_surviving_disk_in_window())

    def disposal_target(self, goal):
        rs = self.ceiling_roots()
        roots = [r for _, r in rs]
        flags = self.nonempty_flags(roots)
        count = len({r[0] for r in roots if flags[r[0]]})
        if count >= MIN_RETAIN:
            return min(goal, self.fourth_newest_nonempty_in_window())
        return goal

    def lag_bound(self):
        # 读法 a：第 4 新的非空有效根取窗口，与整环最旧有效根取大（Rust 第 862–868 行）。p2 不改这一处。
        p2 = self.cfg.p2
        self.cfg.p2 = False
        try:
            f = self.fourth_newest_nonempty_in_window()
        finally:
            self.cfg.p2 = p2
        return max(f, self.oldest_persisted_valid_txg())

    # ---- 块 ----
    def reuse_bound(self):
        return max(self.floor_active, self.oldest_valid_for_gate())

    def reusable_count(self):
        b = self.reuse_bound()
        return self.unused_count + sum(len(v) for g, v in self.by_gen.items() if g <= b)

    def lag_blocks(self):
        b = self.lag_bound()
        return sum(len(v) for g, v in self.by_gen.items() if g > b and g not in self.empty)

    def df_raw(self):
        return self.unused_count + sum(len(v) for v in self.by_gen.values())

    def df_arm(self):
        raw = self.df_raw() + len(self.metadata)
        sub = self.reserve + self.push_residue + self.lag_blocks()
        return raw - sub if raw > sub else 0

    def user_allocatable(self):
        plain = self.reusable_count() + len(self.metadata)
        plain = plain - self.reserve if plain > self.reserve else 0
        return min(plain, self.df_arm())

    def free_blocks(self, blocks, txg):
        for i in blocks:
            if self.kind[i] == KIND_LIVE:
                self.kind[i] = KIND_FREED
                self.freed[i] = txg
                bisect.insort(self.by_gen.setdefault(txg, []), i)

    def oldest_visible_txg(self):
        return min((r[0] for r in self.ring if r is not None), default=0)

    def allocate(self, count, txg):
        bound = self.reuse_bound()
        eligible = [self.by_gen[g] for g in self.by_gen if g <= bound and self.by_gen[g]]
        chosen = list(_take(heapq.merge(*eligible), count)) if eligible else []
        if len(chosen) < count:
            missing = count - len(chosen)
            extra = []
            p = self.unused_ptr
            kind = self.kind
            while len(extra) < missing and p < CAP:
                if kind[p] == KIND_UNUSED:
                    extra.append(p)
                p += 1
            chosen += extra
        if len(chosen) < count:
            return None
        oldest = self.oldest_visible_txg()
        for i in chosen:
            if self.kind[i] == KIND_FREED:
                a, f = self.alloc[i], self.freed[i]
                for t in range(max(a, oldest), f):
                    self.damaged.add(t)
                lst = self.by_gen[f]
                lst.remove(i)
                if not lst:
                    del self.by_gen[f]
            else:
                self.unused_count -= 1
            self.kind[i] = KIND_LIVE
            self.alloc[i] = txg
        # 推进 unused_ptr 到第一个未用块
        p = self.unused_ptr
        while p < CAP and self.kind[p] != KIND_UNUSED:
            p += 1
        self.unused_ptr = p
        return chosen

    def relabel(self, frm, to):
        for i in range(CAP):
            k = self.kind[i]
            if k == KIND_LIVE:
                if self.alloc[i] == frm:
                    self.alloc[i] = to
            elif k == KIND_FREED:
                if self.alloc[i] == frm:
                    self.alloc[i] = to
                if self.freed[i] == frm:
                    self.freed[i] = to
        self.by_gen = {}
        for i in range(CAP):
            if self.kind[i] == KIND_FREED:
                self.by_gen.setdefault(self.freed[i], []).append(i)
        for v in self.by_gen.values():
            v.sort()

    # ---- 快照 ----
    def save_snapshot(self):
        if not self.keep_snapshots:
            return
        self.snapshots.append(((self.instance, self.newest_durable),
                               (bytes(self.kind), tuple(self.alloc), tuple(self.freed), tuple(self.objects), tuple(self.metadata))))
        while len(self.snapshots) > 64:
            self.snapshots.popleft()

    def restore_snapshot(self, instance, txg):
        for key, st in reversed(self.snapshots):
            if key == (instance, txg):
                kind, alloc, freed, objects, metadata = st
                self.kind = bytearray(kind); self.alloc = list(alloc); self.freed = list(freed)
                self.objects = list(objects); self.metadata = list(metadata)
                self.rebuild_buckets()
                return True
        return False

    def rebuild_buckets(self):
        self.by_gen = {}
        self.unused_count = 0
        for i in range(CAP):
            k = self.kind[i]
            if k == KIND_FREED:
                self.by_gen.setdefault(self.freed[i], []).append(i)
            elif k == KIND_UNUSED:
                self.unused_count += 1
        self.unused_ptr = 0
        while self.unused_ptr < CAP and self.kind[self.unused_ptr] != KIND_UNUSED:
            self.unused_ptr += 1

    def objects_key(self):
        return hash(tuple(sorted(self.objects)))

    # ---- 候选 / 下界 ----
    def clause_floor(self):
        """条款的 F_生效（SysPre）：各幸存盘最新持久有效根所带 F 的最大值、被抛弃时间线上根所带 F 的最大值、系统配置里的 F 取大。"""
        vals = [self.sysconfig_floor]
        for d in self.surviving_disks():
            best = None
            for dd, r in self.valid_roots_with_disk():
                if dd == d and (best is None or r[0] > best[0]):
                    best = r
            if best is not None:
                vals.append(best[2])
        for slot, r in enumerate(self.ring):
            if r is not None and self.survives(slot) and r[3] and not self.is_instance_valid(r):
                vals.append(r[2])
        return max(vals)

    def candidacy_floor(self):
        if self.cfg.p3 or self.cfg.p3c:
            return self.clause_floor()
        return max((r[2] for r in self.visible_roots()), default=0)

    def count_fake(self):
        floor = self.candidacy_floor()
        n = 0
        for r in self.valid_roots():
            if r[0] >= floor and self.in_cand_window(r[0]) and r[0] in self.damaged:
                n += 1
        return n

    def prune_damage(self):
        o = self.oldest_visible_txg()
        self.damaged = {t for t in self.damaged if t >= o}

    def record_check(self):
        self.m.fake += self.count_fake()
        self.m.checks += 1

    # ---- 发布 ----
    def floor_written_on_every_surviving_disk(self, value):
        roots = self.valid_roots_with_disk()
        return all(any(d == disk and r[2] >= value for d, r in roots) for disk in self.surviving_disks())

    def publish(self, txg):
        slot = self.slot_index(txg)
        attempt = self.root_write_attempts
        self.root_write_attempts += 1
        planned = attempt in self.planned
        if planned:
            self.planned.discard(attempt)
        if self.fail_next or planned or self.broken_slot == slot:
            self.fail_next = False
            self.m.failures_injected += 1
            if self.cfg.failure == 'advance':
                self.relabel(txg, txg + 1)
                self.next_txg = txg + 1
                self.publish(txg + 1)
            else:
                self.publish(txg)
            return
        self.ring[slot] = (txg, self.instance, self.floor_carried, True)
        if self.cwcus:
            self.nonempty.add(txg)
            self.cwcus = False
        else:
            self.empty.add(txg)
        self.newest_durable = txg
        self.gate_anchor = txg
        self.cand_anchor = txg
        self.max_published = max(self.max_published, txg)
        self.next_txg = txg + 1
        self.unpublished_txg = None
        self.state_key[(self.instance, txg)] = self.objects_key()
        self.save_snapshot()
        if self.pending is not None:
            pf = self.pending
            if pf.carrier is None:
                pf.carrier = txg
            if txg > pf.carrier:
                pf.durable += 1
            if self.floor_written_on_every_surviving_disk(pf.value):
                self.floor_active = max(self.floor_active, pf.value)
                self.pending = None
        self.prune_damage()
        self.record_check()

    def empty_publication(self):
        txg = self.next_txg
        old = self.metadata
        self.metadata = []
        self.free_blocks(old, txg)
        blocks = self.allocate(self.empty_cost, txg)
        if blocks is None:
            self.m.stalls += 1
            self.halted = True
            return False
        self.metadata = blocks
        self.m.forced += 1
        self.publish(txg)
        return not self.halted

    def raise_floor_to(self, target):
        if target > self.floor_carried:
            self.floor_carried = target
            self.sysconfig_floor = max(self.sysconfig_floor, target)
            self.pending = PF(target, None, 0)

    def commit_open_window(self, txg):
        blocks = self.allocate(META, txg)
        if blocks is None:
            self.m.stalls += 1
            self.halted = True
            return False
        self.metadata = blocks
        self.m.forced += 1
        self.publish(txg)
        return not self.halted

    def is_goal_released(self, goal):
        return self.reuse_bound() >= goal

    def push_after_commit(self, goal, stop_when_writable):
        target = self.disposal_target(goal)
        pubs = 0
        for _ in range(self.max_empty_pubs):
            if self.halted:
                break
            done = (stop_when_writable and self.user_allocatable() >= USER_BLOCKS) or self.is_goal_released(target)
            if done:
                break
            if self.floor_carried < target and self.upper_limit() >= target:
                self.raise_floor_to(target)
            if not self.empty_publication():
                break
            pubs += 1
        return pubs

    def run_user_window(self, delete_first, crash_before_root):
        if self.halted:
            return 'Halted'
        self.m.user_windows += 1
        open_txg = self.next_txg
        deleted = False
        if delete_first and self.objects:
            victim = self.next_random() % len(self.objects)
            obj = self.objects[victim]
            self.objects[victim] = self.objects[-1]
            self.objects.pop()
            self.free_blocks(obj, open_txg)
            deleted = True
        old = self.metadata
        self.metadata = []
        self.free_blocks(old, open_txg)
        self.cwcus = deleted
        create_txg = open_txg
        if self.user_allocatable() < USER_BLOCKS:
            self.m.pressure += 1
            if not self.commit_open_window(open_txg):
                return 'Halted'
            committed = self.newest_durable
            pubs = 1 + self.push_after_commit(committed, True)
            self.m.max_pubs = max(self.m.max_pubs, pubs)
            if self.halted:
                return 'Halted'
            if self.user_allocatable() < USER_BLOCKS:
                if self.df_raw() >= USER_BLOCKS:
                    self.m.false_enospc_raw += 1
                if self.df_arm() >= USER_BLOCKS:
                    self.m.false_enospc_arm += 1
                else:
                    self.m.true_enospc += 1
                return 'NoSpace'
            create_txg = self.next_txg
            cm = self.metadata
            self.metadata = []
            self.free_blocks(cm, create_txg)
        blocks = self.allocate(USER_BLOCKS, create_txg)
        assert blocks is not None, 'user_allocatable 说够，allocate 却拿不到'
        self.objects.append(tuple(blocks))
        self.cwcus = True
        md = self.allocate(META, create_txg)
        if md is None:
            self.m.stalls += 1
            self.halted = True
            return 'Halted'
        self.metadata = md
        if crash_before_root:
            self.unpublished_txg = create_txg
            return 'AllocatedWithoutPublish'
        self.publish(create_txg)
        return 'Created'

    # ---- 恢复 ----
    def floors_after_recovery(self):
        candidacy = self.candidacy_floor()
        if self.cfg.p3:
            return candidacy, candidacy
        if self.cfg.p3s:
            return candidacy, self.clause_floor()
        roots = self.valid_roots_with_disk()
        per = []
        for d in self.surviving_disks():
            f = [r[2] for dd, r in roots if dd == d]
            per.append(max(f) if f else 0)
        settled = min(per, default=0)
        return candidacy, min(settled, candidacy)

    def adopt_floors(self, candidacy, settled):
        self.floor_carried = max(candidacy, settled) if self.cfg.p3s else candidacy
        self.floor_active = settled
        if candidacy > settled:
            roots = self.valid_roots()
            carriers = [r[0] for r in roots if r[2] >= candidacy]
            carrier = min(carriers) if carriers else None
            durable = 0 if carrier is None else len({r[0] for r in roots if r[0] > carrier})
            self.pending = PF(candidacy, carrier, durable)
        else:
            self.pending = None

    def crash_and_recover(self):
        chosen = self.newest_valid_root()
        if chosen is None:
            self.m.unrec += 1
            self.halted = True
            return
        replay = self.cfg.recover == 'replay'
        if not replay and chosen[0] in self.damaged:
            self.m.unrec += 1
        candidacy, settled = self.floors_after_recovery()
        if replay:
            eff = self.unpublished_txg if self.unpublished_txg is not None else self.max_published
            self.instance_rows.append((self.instance, eff))
            self.instance += 1
            self.newest_durable = eff
            self.gate_anchor = chosen[0]      # 实现 mount.rs 第 1828 行：挂载时门槛的锚 = 读得出的最新有效根
            self.cand_anchor = eff            # 实现 mount.rs 第 6280 行：回退候选集的锚 = 现行那一版的 txg
            self.next_txg = eff + 1
            self.unpublished_txg = None
            self.adopt_floors(candidacy, settled)
            self.save_snapshot()
            self.record_check()
            return
        if not self.restore_snapshot(chosen[1], chosen[0]):
            self.m.unrec += 1
            self.halted = True
            return
        self.instance_rows.append((self.instance, chosen[0]))
        self.instance += 1
        self.newest_durable = chosen[0]
        self.gate_anchor = chosen[0]
        self.cand_anchor = chosen[0]
        self.next_txg = self.max_published + 1
        self.unpublished_txg = None
        self.adopt_floors(candidacy, settled)
        self.save_snapshot()
        self.record_check()

    def prefill(self, fill):
        target = int(CAP * fill)
        nb = 0
        while nb + 8 <= target:
            obj = []
            for _ in range(8):
                self.kind[nb] = KIND_LIVE
                self.alloc[nb] = 0
                obj.append(nb)
                nb += 1
            self.objects.append(tuple(obj))
        for _ in range(META):
            self.kind[nb] = KIND_LIVE
            self.alloc[nb] = 0
            self.metadata.append(nb)
            nb += 1
        self.unused_count = CAP - nb
        self.unused_ptr = nb
        self.state_key[(1, 0)] = self.objects_key()
        self.save_snapshot()


def _take(it, n):
    for i, x in enumerate(it):
        if i >= n:
            return
        yield x


COST_PAIRS = [(1, 5), (5, 10)]


def fill_to_enospc(cfg, S, mount_cost, declared, seed):
    sim = Sim(cfg, S, seed, mount_cost, declared, keep_snapshots=False)
    sim.prefill(0.70)
    for _ in range(2 * sim.ring_depth + 10):
        sim.run_user_window(True, False)
    stalls = sim.m.stalls
    sim.m = Metrics()
    sim.m.stalls = stalls
    for _ in range(CAP):
        if sim.run_user_window(False, False) != 'Created':
            break
    sim.counting_window = 0
    return sim


def counted_window(sim):
    sim.counting_window += 1
    out = sim.run_user_window(True, False)
    if out == 'NoSpace':
        sim.m.write_after_delete_failed += 1
    return out


def drain_prefix(cfg, S, mount_cost, declared, seed):
    sim = fill_to_enospc(cfg, S, mount_cost, declared, seed)
    for _ in range(5):
        sim.run_user_window(True, False)
    if sim.halted:
        return None
    sim.m = Metrics()
    sim.counting_window = 0
    return sim


def drain_patterns(limit):
    pats = [[a] for a in range(limit + 1)]
    for a in range(limit + 1):
        for b in range(a + 1, limit + 1):
            pats.append([a, b])
    triples = 0
    done = False
    for a in range(limit + 1):
        for b in range(a + 1, limit + 1):
            for c in range(b + 1, limit + 1):
                if triples >= 20:
                    done = True
                    break
                pats.append([a, b, c])
                triples += 1
            if done:
                break
        if done:
            break
    return pats


def run_drain_case(prefix, pattern):
    sim = prefix.clone()
    base = sim.root_write_attempts
    for off in pattern:
        sim.planned.add(base + off)
    for _ in range(20):
        if sim.halted:
            break
        counted_window(sim)
    return sim.m


def run_broken_slot(cfg, S, mount_cost, declared, seed):
    sim = Sim(cfg, S, seed, mount_cost, declared, keep_snapshots=False)
    sim.prefill(0.70)
    for _ in range(2 * sim.ring_depth + 10):
        sim.run_user_window(True, False)
    sim.m = Metrics()
    sim.counting_window = 0
    sim.broken_slot = 1
    for _ in range(1000):
        if sim.halted:
            break
        counted_window(sim)
    return sim.m


def add_metrics(a, b):
    a.fake += b.fake; a.unrec += b.unrec; a.checks += b.checks; a.stalls += b.stalls
    a.max_pubs = max(a.max_pubs, b.max_pubs); a.false_enospc_arm += b.false_enospc_arm
    a.false_enospc_raw += b.false_enospc_raw; a.true_enospc += b.true_enospc
    a.write_after_delete_failed += b.write_after_delete_failed; a.pressure += b.pressure
    a.failures_injected += b.failures_injected


def schedule_event_window(seed, ring_depth):
    state = ((seed * 0xD1B54A32D192ED03) & M64) | 1
    state ^= state >> 29
    return 2 * ring_depth + 20 + state % 150


def failure_patterns():
    pats = [[]]
    for a in range(5):
        pats.append([a])
    for a in range(5):
        for b in range(a + 1, 5):
            pats.append([a, b])
    return pats


def activation_prefix(cfg, S, seed, skip_repeats):
    sim = Sim(cfg, S, seed)
    sim.prefill(0.70)
    ev = schedule_event_window(seed, sim.ring_depth)
    for _ in range(ev):
        sim.run_user_window(True, False)
    remaining = skip_repeats
    while remaining > 0 and not sim.halted:
        if sim.next_txg % R == 1:
            sim.fail_next = True
            remaining -= 1
        sim.run_user_window(True, False)
    if sim.halted:
        return None
    sim.raise_floor_to(sim.upper_limit())
    return sim


def run_activation_case(prefix, pattern, crash_window, lost_disk, variant):
    sim = prefix.clone()
    base = sim.root_write_attempts
    for off in pattern:
        sim.planned.add(base + off)
    for _ in range(1, crash_window):
        sim.run_user_window(True, False)
    sim.run_user_window(True, True)
    if variant == 'crash_no_loss':
        if not sim.halted:
            sim.crash_and_recover()
        for _ in range(2):
            sim.run_user_window(True, False)
        sim.run_user_window(True, True)
    if not sim.halted:
        sim.disk_lost[lost_disk] = True
        sim.crash_and_recover()
    return sim.m


VARIANTS = ['plain', 'crash_no_loss', 'skip1', 'skip2', 'skip3']
VARIANT_NAME = {'plain': 'activation_faults', 'crash_no_loss': 'activation_crash_no_loss',
                'skip1': 'skip_then_raise_r1', 'skip2': 'skip_then_raise_r2', 'skip3': 'skip_then_raise_r3'}


def run_task(args):
    cfg, variant, S, seed = args
    skip = {'skip1': 1, 'skip2': 2, 'skip3': 3}.get(variant, 0)
    prefix = activation_prefix(cfg, S, seed, skip)
    out = {}
    if prefix is None:
        return (variant, S, seed, out)
    pats = failure_patterns()
    for pat in pats:
        for cw in range(1, 9):
            for lost in range(2):
                m = run_activation_case(prefix, pat, cw, lost, variant)
                key = (lost, len(pat))
                a = out.setdefault(key, [0, 0, 0])
                a[0] += m.fake; a[1] += m.unrec; a[2] += m.checks
    return (variant, S, seed, out)


def aggregate(results):
    rows = {}
    for variant, S, seed, out in results:
        for (lost, nf), (f, u, c) in out.items():
            a = rows.setdefault((variant, S, lost, nf), [0, 0, 0])
            a[0] += f; a[1] += u; a[2] += c
    return rows


def main():
    import argparse
    from multiprocessing import Pool
    ap = argparse.ArgumentParser()
    ap.add_argument('--cfg', default='')
    ap.add_argument('--variants', default=','.join(VARIANTS))
    ap.add_argument('--slots', default='4,16')
    ap.add_argument('--seeds', type=int, default=8)
    ap.add_argument('--jobs', type=int, default=24)
    ap.add_argument('--stats', action='store_true')
    a = ap.parse_args()
    kw = {}
    for kv in filter(None, a.cfg.split(',')):
        k, v = kv.split('=')
        kw[k] = (v == 'True') if v in ('True', 'False') else v
    cfg = Cfg(**kw)
    tasks = [(cfg, v, int(S), seed) for v in a.variants.split(',') for S in a.slots.split(',') for seed in range(a.seeds)]
    if a.stats:
        res = [run_task(t) for t in tasks]
    else:
        with Pool(a.jobs) as pool:
            res = pool.map(run_task, tasks, chunksize=1)
    rows = aggregate(res)
    print('# cfg', cfg.tag())
    if a.stats:
        print('# STATS', STATS)
    for (variant, S, lost, nf), (f, u, c) in sorted(rows.items(), key=lambda kv: (VARIANTS.index(kv[0][0]), kv[0][1], kv[0][2], kv[0][3])):
        print(f'variant={VARIANT_NAME[variant]} s={S} lost_disk={lost} failures={nf} fakes={f} unrecoverable={u} checks={c}')
    tf = sum(1 for v in rows.values() if v[0] > 0)
    tu = sum(1 for v in rows.values() if v[1] > 0)
    print(f'# rows={len(rows)} rows_with_fakes={tf} rows_with_unrecoverable={tu}')


if __name__ == '__main__':
    main()
