#!/usr/bin/env python3
"""V2（H-S1-fold）抽象模型：CPU 走读的折叠表（walk.rs 的 unit_of_each_placement）怎么改，I-3.5 按「每版走完当场折」（snapshot，改前）
与「全部走完之后按最终表折」（final，第 36 批）各判出哪些落点，与条款字面（按落点判）比。只比「判不判」这一层，不比诞生代号的值。
规则照冻结副本 walk.rs：记引用时每条位置条目 or_insert 成这条指针第 0 条的落点；读回的是第 1 条、那个落点此刻还折在这条指针第 0 条上时改记成它自己。
穷举：落点 0..P-1、版本 V 个（txg 0..V-1，相邻成对）、每版 1..M 条指针（两条位置条目不同槽）、每条指针第 0 份好 / 坏、走读次序取全部排列。
用法：v2_fold_model.py [P V M]；输出 V2MODEL 行（数），不带耗时。"""
import itertools
import sys

P, V, M = (int(x) for x in sys.argv[1:4]) if len(sys.argv) > 3 else (4, 3, 1)
pointers = [(a, b) for a in range(P) for b in range(P) if a != b]
pointer_sets = [c for n in range(1, M + 1) for c in itertools.combinations(pointers, n)]

cases = 0
literal_judged = 0
miss = {"snap": 0, "final": 0}
v2_induced = 0
v2_removed = 0
v2_example = None
for sets in itertools.product(pointer_sets, repeat=V):
    copy_choices = [list(itertools.product([0, 1], repeat=len(s))) for s in sets]
    for choices in itertools.product(*copy_choices):
        for order in itertools.permutations(range(V)):
            cases += 1
            table = {}
            snap = {}
            for v in order:
                for (l0, l1), c in zip(sets[v], choices[v]):
                    for loc in (l0, l1):
                        table.setdefault(loc, l0)
                    if c == 1 and table.get(l1) == l0:
                        table[l1] = l1
                placements = {loc for ptr in sets[v] for loc in ptr}
                snap[v] = {p: table[p] for p in placements}
            final = {v: {p: table[p] for p in snap[v]} for v in range(V)}
            for older, newer in zip(range(V), range(1, V)):
                s_old, s_new = set(snap[older]), set(snap[newer])
                for b in s_new - s_old:
                    literal_judged += 1
                    missed = {}
                    for name, fold in (("snap", snap), ("final", final)):
                        units_old = set(fold[older].values())
                        missed[name] = fold[newer][b] in units_old
                        miss[name] += missed[name]
                    if missed["snap"] and not missed["final"]:
                        v2_removed += 1
                    if missed["final"] and not missed["snap"]:
                        v2_induced += 1
                        if v2_example is None:
                            v2_example = (sets, choices, order, older, newer, b)
print(f"V2MODEL P={P} V={V} M={M} cases={cases} literal_judged_placements={literal_judged} missed_snapshot={miss['snap']} missed_final={miss['final']} missed_final_not_snapshot={v2_induced} missed_snapshot_not_final={v2_removed}")
print(f"V2MODEL_EXAMPLE {v2_example}")
