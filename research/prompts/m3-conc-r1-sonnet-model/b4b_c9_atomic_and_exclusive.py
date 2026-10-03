#!/usr/bin/env python3
"""C9 补一刀（接 b4_c9_delete_interleave.py，不改它）。b4 里「续做独占」那一行 0 违例是靠定义排除的（只放行 W 的计划排在 C2 之后的 1 条交错），
没量。这里把独占写成能被量的规则，并加上 Q4 取整体独占（A）时的读法，数同一批 28 条交错（D 六步：PI CI P1 C1 P2 C2；W 两步：PW CW）：
  F   并行、提交盲写                                                   （同 b4）
  V   并行、提交时校验读集（X 的 inode 版本）                           （同 b4）
  S1  续做独占·计划闸：队列非空时 W 不许开始计划；已经计划过的 W 不管     （b4 的「独占」可以量的最弱读法）
  S2  续做独占·计划闸 + 提交闸：W 提交时队列非空或 X 的版本变了就作废这次事务（不重新计划）
  A   Q4 取整体独占：每一次发布的「计划 + 提交」在同一个临界区里，不可分（W 一步、D 三步：I、B1、B2）
违例的定义同 b4。
"""
from itertools import combinations, permutations

import b4_c9_delete_interleave as b4


def run(order, discipline):
    head = b4.Head()
    plans = {}
    aborted = False
    blocked = False
    for who, step in order:
        if step.startswith("P"):
            kind = {"PI": "I", "P1": "B1", "P2": "B2", "PW": "W"}[step]
            if discipline in ("S1", "S2") and who == "W" and head.queue:
                blocked = True           # 队列非空：W 不许开始计划，这条交错不在规则允许之内
                break
            plans[step] = (b4.plan(kind, head), head.inode["ver"] if head.inode else None)
        else:
            p, base_ver = plans["P" + step[1:]]
            if p is None:
                continue
            if discipline == "S2" and who == "W" and (head.queue or head.inode is None or head.inode["ver"] != base_ver):
                aborted = True
                continue
            b4.apply(p, head)
    return blocked, aborted, b4.verdict(head)


def atomic_run(sequence):
    """A：每一步计划 + 提交连着做。sequence 是 D 三步（I、B1、B2）与 W 的一个排列，D 内部保持次序。"""
    head = b4.Head()
    for kind in sequence:
        p = b4.plan(kind, head)
        if p is not None:
            b4.apply(p, head)
    return b4.verdict(head)


def main():
    d_steps = ["PI", "CI", "P1", "C1", "P2", "C2"]
    schedules = []
    for slots in combinations(range(8), 2):
        order, di = [], 0
        for pos in range(8):
            if pos == slots[0]:
                order.append(("W", "PW"))
            elif pos == slots[1]:
                order.append(("W", "CW"))
            else:
                order.append(("D", d_steps[di]))
                di += 1
        schedules.append(order)
    print(f"计划与提交可分的交错：{len(schedules)} 条")
    for discipline, name in (("F", "并行·盲写"), ("S1", "独占·计划闸"), ("S2", "独占·计划闸+提交闸")):
        allowed = bad = aborts = 0
        kinds = {}
        for order in schedules:
            blocked, aborted, problems = run(order, discipline)
            if blocked:
                continue
            allowed += 1
            aborts += aborted
            if problems:
                bad += 1
                kinds[tuple(problems)] = kinds.get(tuple(problems), 0) + 1
        print(f"  {name:12s} 规则放行 {allowed:2d} 条，违例 {bad:2d} 条 {kinds}；其中提交闸作废 W 的 {aborts} 条")
    v_bad = sum(1 for o in schedules if b4.simulate(o, "V")[0])
    print(f"  并行·校验读集 违例 {v_bad} 条（同 b4）")
    seqs = {s for s in permutations(["I", "B1", "B2", "W"]) if s.index("I") < s.index("B1") < s.index("B2")}
    bad_a = [s for s in sorted(seqs) if atomic_run(s)]
    print(f"  A 整体独占（四步各自不可分）：W 插进 D 三步的位置 {len(seqs)} 种，违例 {len(bad_a)} 种 {bad_a}")


if __name__ == "__main__":
    main()
