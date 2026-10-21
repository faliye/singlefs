#!/usr/bin/env python3
"""辩方腿（m3-close-sysconf-r2）的取号崩溃点扫描小模型：纯 Python、不碰 crates。

它回答的是「第一轮攻方只量了一个起点、一个切点（写流前缀第 26 步）」之外，取号写的各种改法在整条写序的
每个崩溃状态、每种「每盘最多坏一槽」的故障下，下一次可写挂载的结局。模型是按 transaction.rs 的写序读出来的抽象，
不是入库装置，数不进 kb（见报告「没做什么」）。

写序（读自 transaction.rs：取号写 write_the_witnessed_acquisition、perform_commit_step、write_system_configuration_slot）：
  取号写（逐盘各写一次，覆写较旧的那一槽） → 屏障 → 单元写到每块盘 → 屏障 → journal 记录写到每块盘 → 屏障
  → 根槽 FUA → 发布末尾的轮换（逐盘各写一次，覆写较旧的那一槽） → 屏障
崩溃状态 = 某一段（两次屏障之间）正在飞：该段每个写可能「没落」「落了」，系统配置槽的写还可能「撕裂」；之前各段全落，之后各段全没落。
故障 = 崩溃之后、下一次可写挂载之前，每盘最多一槽出问题：翻坏（读得出、自证不过）或持续读错（设备报错）。
结局：
  R2拒      有槽读错，且没有一块盘两槽都自证过（recovery.rs 的 system_configuration_witness_is_undecidable）
  盘全废拒  某块盘一份自证过的系统配置都没有（transaction.rs 取号读见证值时拒）
  撞号      取号取到的号等于崩溃那一次挂载的号，而那一次留下的单元/记录/根里有带这个号的
  干净      挂上且没撞号
取号式子的臂：today（系统配置 + 根环）、ring（再并进 journal 环里读得出的记录）、units（再并进读得出的单元写序）。
取号写的臂：one_round（今天）、two_rounds（形态甲：两轮，每轮逐盘写、每轮一道屏障，两轮完成才动单元）。
出路的臂（只对 R2拒 的状态）：alpha（只重写不自证的槽，号取各盘自证槽/根环/记录环最大值）、alpha_scan（再取单元写序）。
"""
import itertools
import sys
from collections import Counter, defaultdict

PREVIOUS_INSTANCE = 1
CRASHED_INSTANCE = 2
START_GENS = (4, 5)  # 每盘两槽的世代号：槽 0 = 4，槽 1 = 5

# 槽状态：("ok", gen, inst) / ("torn",) / ("rderr",)  ——「flip」与「torn」对系统配置判法同类（读得出、自证不过）
def start_disk():
    return [("ok", START_GENS[0], PREVIOUS_INSTANCE), ("ok", START_GENS[1], PREVIOUS_INSTANCE)]


def write_target(slots, instance):
    """write_system_configuration_slot：世代号 = 自证槽最大世代号 + 1，槽 = 世代号 mod 2。"""
    oks = [s for s in slots if s[0] == "ok"]
    gen = (max(s[1] for s in oks) if oks else 0) + 1
    return gen % 2, ("ok", gen, instance)


def segments(arm_write):
    """每段 = (名字, [写])，写 = (种类, 盘号)；种类 sys 是系统配置槽写（带实例），unit/rec/root 是别的。"""
    segs = []
    if arm_write == "two_rounds":
        segs.append(("取号写第一轮", [("sys", 0), ("sys", 1)]))
        segs.append(("取号写第二轮", [("sys", 0), ("sys", 1)]))
    else:
        segs.append(("取号写", [("sys", 0), ("sys", 1)]))
    segs.append(("单元写", [("unit", 0), ("unit", 1)]))
    segs.append(("journal记录写", [("rec", 0), ("rec", 1)]))
    segs.append(("根槽FUA", [("root", 0)]))
    segs.append(("发布末尾轮换", [("sys", 0), ("sys", 1)]))
    return segs


def crash_states(arm_write):
    """产出 (段名, 盘面)；盘面 = dict(disks=[槽表, 槽表], units=集合, recs=集合, root=bool)。去重。"""
    segs = segments(arm_write)
    seen = set()
    for si, (name, writes) in enumerate(segs):
        outcomes = []
        for kind, d in writes:
            outcomes.append(("absent", "done", "torn") if kind == "sys" else ("absent", "done"))
        for choice in itertools.product(*outcomes):
            state = {"disks": [start_disk(), start_disk()], "units": set(), "recs": set(), "root": False}
            for pj in range(si):  # 之前各段全落
                apply_segment(state, segs[pj][1], ["done"] * len(segs[pj][1]), arm_write, segs, pj)
            apply_segment(state, writes, list(choice), arm_write, segs, si)
            key = (tuple(tuple(d) for d in state["disks"]), tuple(sorted(state["units"])), tuple(sorted(state["recs"])), state["root"])
            if key in seen:
                continue
            seen.add(key)
            yield name, state


def apply_segment(state, writes, choice, arm_write, segs, index):
    for (kind, d), how in zip(writes, choice):
        if how == "absent":
            continue
        if kind == "sys":
            slot, new = write_target(state["disks"][d], CRASHED_INSTANCE)
            state["disks"][d][slot] = new if how == "done" else ("torn",)
        elif kind == "unit":
            state["units"].add(d)
        elif kind == "rec":
            state["recs"].add(d)
        elif kind == "root":
            state["root"] = True


FAULTS = [None] + [(s, k) for s in (0, 1) for k in ("flip", "rderr")]


def with_faults(state, f0, f1):
    disks = [list(d) for d in state["disks"]]
    for d, f in ((0, f0), (1, f1)):
        if f:
            disks[d][f[0]] = ("torn",) if f[1] == "flip" else ("rderr",)
    return disks


def undecidable(disks):
    any_read_error = any(s[0] == "rderr" for d in disks for s in d)
    some_disk_both_ok = any(all(s[0] == "ok" for s in d) for d in disks)
    return any_read_error and not some_disk_both_ok


def has_dead_disk(disks):
    return any(not any(s[0] == "ok" for s in d) for d in disks)


def highest_visible(disks, state, formula, include_units=False):
    m = max([s[2] for d in disks for s in d if s[0] == "ok"] + [PREVIOUS_INSTANCE])
    if state["root"]:
        m = max(m, CRASHED_INSTANCE)
    if formula in ("ring", "units") and state["recs"]:
        m = max(m, CRASHED_INSTANCE)
    if (formula == "units" or include_units) and state["units"]:
        m = max(m, CRASHED_INSTANCE)
    return m


def harmful_artifacts(state):
    return bool(state["units"] or state["recs"] or state["root"])


def outcome(disks, state, formula):
    if undecidable(disks):
        return "R2拒"
    if has_dead_disk(disks):
        return "盘全废拒"
    nxt = highest_visible(disks, state, formula) + 1
    if nxt == CRASHED_INSTANCE and harmful_artifacts(state):
        return "撞号"
    return "干净"


def repaired(disks, state, scan_units):
    """alpha：每盘不自证的槽（有自证槽的盘）写成 世代号 = 自证最大 + 1、号 = 各盘自证槽/根环/记录环（/单元写序）最大值。
    返回 (新盘面, 持续读错槽写不进时的盘面)。"""
    m = highest_visible(disks, state, "ring", include_units=scan_units)
    fixed, stuck = [], []
    for d in disks:
        oks = [s for s in d if s[0] == "ok"]
        if len(oks) == 2:
            fixed.append(list(d)); stuck.append(list(d)); continue
        if not oks:
            fixed.append(list(d)); stuck.append(list(d)); continue
        slot, new = write_target(d, m)
        nd = list(d); nd[slot] = new
        fixed.append(nd)
        sd = list(d)
        if d[slot][0] != "rderr":
            sd[slot] = new
        stuck.append(sd)
    return fixed, stuck


def run(arm_write, formula):
    by_segment = defaultdict(Counter)
    totals = Counter()
    fault_free = Counter()
    alpha_after = {"alpha": Counter(), "alpha_scan": Counter()}
    alpha_after_stuck = {"alpha": Counter(), "alpha_scan": Counter()}
    for name, state in crash_states(arm_write):
        for f0, f1 in itertools.product(FAULTS, FAULTS):
            disks = with_faults(state, f0, f1)
            res = outcome(disks, state, formula)
            if f0 is None and f1 is None:
                fault_free[res] += 1
            by_segment[name][res] += 1
            totals[res] += 1
            if res == "R2拒":
                for arm, scan in (("alpha", False), ("alpha_scan", True)):
                    fixed, stuck = repaired(disks, state, scan)
                    alpha_after[arm][outcome(fixed, state, formula)] += 1
                    alpha_after_stuck[arm][outcome(stuck, state, formula)] += 1
    return by_segment, totals, fault_free, alpha_after, alpha_after_stuck


def r2_refusal_classes(arm_write):
    """R2拒 的组合按成因分：持续读错的槽数、崩溃自己撕裂的槽数（故障之前盘上就不自证的槽）。无崩溃 = 崩溃状态里没有撕裂的槽。"""
    classes = Counter()
    crash_free_start = Counter()
    for name, state in crash_states(arm_write):
        torn_by_crash = sum(1 for d in state["disks"] for s in d if s[0] == "torn")
        for f0, f1 in itertools.product(FAULTS, FAULTS):
            disks = with_faults(state, f0, f1)
            if outcome(disks, state, "today") != "R2拒":
                continue
            n_read_errors = sum(1 for d in disks for s in d if s[0] == "rderr")
            n_flips = sum(1 for f in (f0, f1) if f and f[1] == "flip")
            key = (f"持续读错槽={n_read_errors}", f"翻坏故障槽={n_flips}", "崩溃撕裂槽>0" if torn_by_crash else "崩溃撕裂槽=0")
            classes[key] += 1
            if name.startswith("取号写") and state["disks"] == [start_disk(), start_disk()] and not state["units"]:
                crash_free_start[key] += 1
    return classes, crash_free_start


def main():
    print("# 取号崩溃点扫描：臂 = (取号写, 取号式子)；故障 = 崩溃后每盘最多坏一槽（翻坏或持续读错），共 25 种 × 每个崩溃状态")
    for arm_write in ("one_round", "two_rounds"):
        n_states = sum(1 for _ in crash_states(arm_write))
        print(f"\n## arm_write={arm_write} 崩溃状态数（去重）={n_states}")
        for formula in ("today", "ring", "units"):
            by_segment, totals, fault_free, after, after_stuck = run(arm_write, formula)
            print(f"\n### arm_write={arm_write} formula={formula}")
            print(f"总计 {dict(sorted(totals.items()))}；无故障时 {dict(sorted(fault_free.items()))}")
            for seg, c in by_segment.items():
                print(f"  段={seg} {dict(sorted(c.items()))}")
            for arm in ("alpha", "alpha_scan"):
                print(f"  出路={arm}：R2拒的 {totals['R2拒']} 个状态里，重写之后再可写挂载 {dict(sorted(after[arm].items()))}；"
                      f"持续读错的槽写不进时 {dict(sorted(after_stuck[arm].items()))}")


def print_classes():
    for arm_write in ("one_round", "two_rounds"):
        classes, start = r2_refusal_classes(arm_write)
        print(f"\n## arm_write={arm_write} R2拒 的成因分类（组合数）")
        for key, n in sorted(classes.items()):
            print(f"  {key} {n}")
        print(f"  其中盘面就是取号之前的原样（没有任何崩溃）的组合：{dict(sorted(start.items()))}")


if __name__ == "__main__":
    main()
    print_classes()
