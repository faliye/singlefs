#!/usr/bin/env python3
"""J5：谱系记录落地之后的 (b)。模型里把每条回退根 R 记成一条谱系记录 (T_rb = R.txg, T_old = 它回退到的那条根的 txg)，
记录随现行那一版往后带（D18 已定项 24：谱系树随现行那一版、只增不删），所以 txg 小于等于 R 的非被抛弃根沿父链都看得到它。
T_old 取模型里「树完全相同、txg 比 R 小的非被抛弃根里 txg 最大的」（真实的回退目标是其中之一；继承来的树根指针出生 txg ≤ 它们任何一个）。
写法：
  lin_v        第一轮判决第二节第 5 行的字面：「R 里有一条 T_rb = R.txg 的谱系记录、树根指针的出生 txg 不大于它的 T_old」才免
  lin_fix      辩方改的：R 能看到的谱系记录里有一条 P.txg < T_rb ≤ R.txg（回退发生在 R⁻ 之后、R 之前或就是 R）且树根指针出生 txg ≤ 它的 T_old，才免
  *_all        相邻差集全判（β）；不带 = 只判第一次出现
树根指针「R⁻ 不引用」与 (a) birth(b) > R⁻.txg 同 r2_variants。"""
import sys
sys.dont_write_bytecode = True
import birth_floor_v2 as m
from birth_floor_v2 import candidates, refs, subtree

LV = ("lin_v", "lin_v_all", "lin_fix", "lin_fix_all")

def records_of(h):
    out = []
    for x in h.roots:
        if x[1] != "R" or x[0] in h.ab:
            continue
        same = [y for y in h.roots if y[0] < x[0] and y[0] not in h.ab and (y[2], y[3]) == (x[2], x[3])]
        t_old = max(y[0] for y in same)
        out.append((x[0], t_old))
    return out

def analyse_lin(h, bo=None, po=None):
    bo = bo or {}
    po = po or {}
    birth = lambda b: bo.get(b, b[1])
    cand = sorted(candidates(h), key=lambda r: r[0])
    rs = [refs(r) for r in cand]
    recs = records_of(h)
    out = {k: [] for k in LV}
    seen = set(rs[0]) if rs else set()
    for j in range(1, len(cand)):
        R, P = cand[j], cand[j - 1]
        seeable = [(trb, told) for (trb, told) in recs if trb <= R[0]]
        v_recs = [(trb, told) for (trb, told) in seeable if trb == R[0]]
        f_recs = [(trb, told) for (trb, told) in seeable if P[0] < trb <= R[0]]
        ptr = {}
        for kind, idx in (("I", 2), ("X", 3)):
            u = R[idx]
            ptr[kind] = None if u is None else (u, po.get((R[0], kind), u[1]))
        def exempt(b, rec_list):
            for kind, v in ptr.items():
                if v is None:
                    continue
                u, pb = v
                if u in rs[j - 1] or b not in subtree(u):
                    continue
                if any(pb <= told for (_, told) in rec_list):
                    return True
            return False
        for b in rs[j] - rs[j - 1]:
            first = b not in seen
            if birth(b) > P[0]:
                continue
            for name, lst in (("lin_v", v_recs), ("lin_fix", f_recs)):
                if not exempt(b, lst):
                    out[name + "_all"].append((b, j))
                    if first:
                        out[name].append((b, j))
        seen |= rs[j]
    return out
