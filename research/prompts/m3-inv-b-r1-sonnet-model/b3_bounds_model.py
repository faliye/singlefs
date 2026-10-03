#!/usr/bin/env python3
"""B3 抽象模型：I-3.5 ② 挪到回退候选集上的两半（上界、下界）在带管理员回退的历史上成不成立。
只建模：每次发布 = 一条根（txg 逐次 +1），根引用一组单元，单元诞生代号 = 写出它那次发布的 txg。
操作：W 覆盖写（换新单元）、E 空发布（照抄上一条根的引用）、R(k) 回退到候选集里第 k 条根（新根引用回退目标的单元，诞生代号不变——D23 已定项 14「复活」）、
F(x) 抬 F 到 x（x 不高于抬 F 的上限：第 4 新的非空根的 txg；非空根不足 4 条时上限 = 最旧根，抬不动）。
候选集 = txg ≥ F 的根。不模型：实例表、被抛弃时间线、快照。
"""
import itertools, sys
sys.dont_write_bytecode = True

def run(ops):
    roots = [(3, frozenset({("u0", 3)}))]          # 第一个文件版本：txg 3，单元 u0 诞生代号 3
    floor = 0
    txg = 3
    nonempty = [True]
    for op in ops:
        kind = op[0]
        if kind == "F":
            cand = [i for i, r in enumerate(roots) if r[0] >= floor]
            ne = [roots[i][0] for i in cand if nonempty[i]]
            if len(ne) < 4:
                return None
            ceiling = sorted(ne, reverse=True)[3]
            target = op[1]
            if not (floor < target <= ceiling):
                return None
            floor = target
            continue
        txg += 1
        prev_refs = roots[-1][1]
        if kind == "W":
            refs = frozenset({(f"u{txg}", txg)})
        elif kind == "E":
            refs = prev_refs
        elif kind == "R":
            cand = [r for r in roots if r[0] >= floor]
            if op[1] >= len(cand):
                return None
            refs = cand[op[1]][1]
        roots.append((txg, refs))
        nonempty.append(refs != prev_refs)
    return roots, floor

def violations(roots, floor):
    cand = [r for r in roots if r[0] >= floor]
    upper, lower = [], []
    units = {u for _, refs in cand for u in refs}
    for unit in sorted(units):
        referencing = [r[0] for r in cand if unit in r[1]]
        first = min(referencing)
        if unit[1] > first:
            upper.append((unit, first))
        before = [r[0] for r in cand if r[0] < first]
        if before and not unit[1] > max(before):
            lower.append((unit, first, max(before)))
    return upper, lower

def all_ops(depth):
    alphabet = [("W",), ("E",)] + [("R", k) for k in range(4)] + [("F", x) for x in range(4, 12)]
    for n in range(1, depth + 1):
        yield from itertools.product(alphabet, repeat=n)

total = with_rollback_low = with_rollback_up = without_rollback_low = without_rollback_up = 0
first_example = None
for ops in all_ops(6):
    result = run(ops)
    if result is None:
        continue
    total += 1
    upper, lower = violations(*result)
    used_rollback = any(op[0] == "R" for op in ops)
    if used_rollback:
        with_rollback_low += bool(lower); with_rollback_up += bool(upper)
    else:
        without_rollback_low += bool(lower); without_rollback_up += bool(upper)
    if lower and first_example is None:
        first_example = (ops, result, lower)
print("合法历史条数（操作串长 ≤ 6，抬 F 不越上限）:", total)
print("含回退：下界违例 %d 条，上界违例 %d 条" % (with_rollback_low, with_rollback_up))
print("不含回退：下界违例 %d 条，上界违例 %d 条" % (without_rollback_low, without_rollback_up))
if first_example:
    ops, (roots, floor), lower = first_example
    print("最短的一条下界违例历史：", ops)
    print("  根（txg → 引用）:", [(t, sorted(r)) for t, r in roots], " F =", floor)
    print("  违例（单元, R_first, R_prev）:", lower)
