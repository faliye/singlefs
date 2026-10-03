#!/usr/bin/env python3
"""正推腿的小模型：根环补洞挂点在「补洞那一段」的崩溃状态（H2）。

抽象（与 crates 不共用代码，只按 crates 今天的规则写死，出处在报告里）：
- 根槽位置 pos(txg) = (txg % 3) * S + (txg // 3) % S（root_ring.rs target_for_publish）；环长 3S；mkfs 把第 0 代根种进三个区域的槽 0。
- 每次发布 k 分出一个单元 A_k、换下上一个 A_{k-1}（释放代 k）；根 r 引用 A_r。「一版一个单元」是最小抽象。
- 可再分配门槛 = 分配器眼里环里最旧有效根的 txg（F 取 0，mount.rs reclaim_floor）；释放代 <= 门槛的单元槽才能被新单元盖掉；
  本次发布自己换下的不在本次候选里（本事务内释放的块不得重用）。分配器记根发生在这次发布落点取完之后
  （allocator.rs record_root_written_by_this_process），所以一次发布记的根只影响下一次发布的落点。
- 落点取法两种都跑：low = 可回收槽里取最低槽号，high = 取单元号最大的（最贴近仍被候选引用的那批）；判红只要有一种取法红就算红。
- 崩溃状态照 D13 已定项 4：前面各段全持久，当前段任意子集，之后全没持久；FUA 根槽写一次两态。
- 判红：回退候选集（环里自证过的根）里有根引用的单元槽已被别的单元盖掉（I-7.4 / I-4.8 同一格）。
输出每个变体的状态数、红的状态数、下次挂载还带着洞的状态数。
"""
import itertools
import sys
from collections import Counter

S = 4
L = 3 * S


def pos(txg):
    return (txg % 3) * S + (txg // 3) % S


class Disk:
    def __init__(self):
        self.ring = {}          # pos -> txg 或 None（垃圾）
        self.slot = {}          # 单元槽 -> 单元号（出生 txg）
        self.unit_slot = {}     # 单元号 -> 槽
        self.records = set()    # 持久的记录 txg

    def copy(self):
        d = Disk()
        d.ring = dict(self.ring)
        d.slot = dict(self.slot)
        d.unit_slot = dict(self.unit_slot)
        d.records = set(self.records)
        return d


class Alloc:
    def __init__(self, belief, release_gen, disk, policy):
        self.belief = dict(belief)
        self.release_gen = dict(release_gen)
        self.disk = disk
        self.policy = policy
        self.next_fresh = max(disk.slot) + 1 if disk.slot else 1

    def floor(self):
        return min(self.belief.values())

    def place(self):
        floor = self.floor()
        reclaimable = [
            (u, s) for s, u in self.disk.slot.items()
            if u in self.release_gen and self.release_gen[u] <= floor
        ]
        if reclaimable:
            if self.policy == "low":
                return min(s for _, s in reclaimable)
            return max(reclaimable)[1]
        s = self.next_fresh
        self.next_fresh += 1
        return s

    def record_root(self, txg):
        self.belief[pos(txg)] = txg


def run_history(h, policy):
    """h 之前的发布正常完成；第 h 次只落单元与记录，根槽没落（记录已持久、根槽没持久的崩溃）。"""
    d = Disk()
    for r in range(3):
        d.ring[r * S] = 0
    release = {}
    alloc = Alloc(d.ring, release, d, policy)
    live = None
    for k in range(1, h + 1):
        s = alloc.place()
        if live is not None:
            alloc.release_gen[live] = k
        d.slot[s] = k
        d.unit_slot[k] = s
        d.records.add(k)
        live = k
        if k < h:
            d.ring[pos(k)] = k
            alloc.record_root(k)
    return d, alloc.release_gen


def mount_plan(variant, h, disk, release, policy):
    """返回段列表；每段是写的列表，写 = (种类, k, 槽)。段与段之间隔着屏障或 FUA。"""
    plan_disk = disk.copy()
    alloc = Alloc(disk.ring, release, plan_disk, policy)
    live = h
    segments = []
    if variant in ("P1", "P0"):
        segments.append([("patch", h, None)])
        alloc.record_root(h)
    elif variant == "P1-wipe":
        segments.append([("wipe", h, None)])
        alloc.record_root(h)
    elif variant == "P2-eager":
        alloc.record_root(h)
    for i, k in enumerate((h + 1, h + 2, h + 3)):
        s = alloc.place()
        alloc.release_gen[live] = k
        live = k
        plan_disk.slot[s] = k
        plan_disk.unit_slot[k] = s
        root_segment = [("root", k, None)]
        if variant in ("P2-eager", "P2-late") and i == 0:
            root_segment.append(("patch", h, None))
        segments.append([("unit", k, s)])
        segments.append([("record", k, None)])
        segments.append(root_segment)
        alloc.record_root(k)
        if variant == "P2-late" and i == 0:
            alloc.record_root(h)
    if variant == "P3":
        segments.append([("patch", h, None)])
    return segments


def apply_write(d, w):
    kind, k, s = w
    if kind == "unit":
        d.slot[s] = k
        d.unit_slot[k] = s
    elif kind == "record":
        d.records.add(k)
    elif kind in ("root", "patch"):
        d.ring[pos(k)] = k
    elif kind == "wipe":
        d.ring[pos(k)] = None


def crash_states(base, segments):
    yield base.copy()
    d = base.copy()
    for seg in segments:
        for r in range(len(seg) + 1):
            for subset in itertools.combinations(range(len(seg)), r):
                e = d.copy()
                for j in subset:
                    apply_write(e, seg[j])
                yield e
        for w in seg:
            apply_write(d, w)


def judge(d):
    roots = {t for t in d.ring.values() if t is not None}
    chosen = max(roots)
    effective = chosen
    k = chosen + 1
    while k in d.records and d.slot.get(d.unit_slot.get(k, -1)) == k:
        effective = k
        k += 1
    red = any(t >= 1 and d.slot.get(d.unit_slot.get(t, -1)) != t for t in roots)
    lo = min(roots)
    interior_hole = any(d.ring.get(pos(t)) != t for t in range(lo, chosen + 1))
    return chosen, effective, lo, red, interior_hole


def run(h, variant):
    total = Counter()
    for policy in ("low", "high"):
        disk, release = run_history(h, policy)
        segments = mount_plan(variant, h, disk, release, policy)
        for image in crash_states(disk, segments):
            chosen, effective, lo, red, interior_hole = judge(image)
            total[policy + ":states"] += 1
            total[policy + ":red"] += red
            total[policy + ":interior_hole"] += interior_hole
            total[policy + ":unrooted_tip"] += effective > chosen
    return total, len(segments)


def main():
    print(f"S={S} 环长={L}")
    for name, h in (("back", 3 * S + 5), ("front", 5)):
        held = f"txg {h - L}" if h > L else "从没写过"
        print(f"== 洞位置 {name}: h={h} slot(h)=pos {pos(h)} 洞槽原有根={held}")
        for variant in ("none", "P1", "P0", "P2-late", "P2-eager", "P3", "P1-wipe"):
            total, nseg = run(h, variant)
            cells = []
            for policy in ("low", "high"):
                cells.append(
                    f"{policy}: 状态={total[policy + ':states']:3d} 红={total[policy + ':red']:3d}"
                    f" 内部洞={total[policy + ':interior_hole']:3d} 记录在根不在={total[policy + ':unrooted_tip']:3d}"
                )
            print(f"  {variant:9s} 段数={nseg:2d} | " + " | ".join(cells))


if __name__ == "__main__":
    main()
