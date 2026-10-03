#!/usr/bin/env python3
"""第三轮：J5 谱系记录写法在两片叶的 inode 树模型上重量。写法：
  lin_tree      第二轮判决采的区间形（辩方 C 节）：R 看得到的记录里有一条 P.txg < T_rb ≤ R.txg，且 b 所在那棵树（R⁻ 不引用的树根）的根指针出生 txg ≤ 它的 T_old
  lin_cell      格级形（第二轮攻方「我提的读法」：b 的诞生代号 ≤ T_old）：有一条这样的记录且 birth(b) ≤ T_old 就免，不看树
  *_gap         同上，再加 gap（R.txg ≠ R⁻.txg + 1 整对不判）
  *_all         相邻差集全判（β）；不带 = 只判第一次出现
记录随现行那一版往后带。T_old 取模型里树完全相同、txg 比回退根小的非被抛弃根里 txg 最大的。"""
import sys
sys.dont_write_bytecode = True
import birth_floor_v3 as m
from birth_floor_v3 import candidates, refs, subtree

NAMES = ("lin_tree", "lin_tree_gap", "lin_cell", "lin_cell_gap", "lin_cellin", "lin_cellin_gap")
LV = tuple(n + s for n in NAMES for s in ("", "_all"))

def records_of(h):
    out = []
    for x in h.roots:
        if x[1] != "R" or x[0] in h.ab:
            continue
        same = [y for y in h.roots if y[0] < x[0] and y[0] not in h.ab and (y[2], y[3]) == (x[2], x[3])]
        out.append((x[0], max(y[0] for y in same)))
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
        gap = R[0] != P[0] + 1
        f_recs = [(trb, told) for (trb, told) in recs if P[0] < trb <= R[0]]
        ptr = {}
        for kind, idx in (("I", 2), ("X", 3)):
            u = R[idx]
            ptr[kind] = None if u is None else (u, po.get((R[0], kind), u[1]))
        def ex_tree(b):
            for kind, v in ptr.items():
                if v is None:
                    continue
                u, pb = v
                if u in rs[j - 1] or b not in subtree(u):
                    continue
                if any(pb <= told for (_, told) in f_recs):
                    return True
            return False
        def ex_cell(b):
            return any(birth(b) <= told for (_, told) in f_recs)
        def ex_cellin(b):
            # 格级 + 落在 R 的某棵用户可见树的子树里（固定点单元不在其内）
            if not any(v is not None and b in subtree(v[0]) for v in ptr.values()):
                return False
            return any(birth(b) <= told for (_, told) in f_recs)
        for b in rs[j] - rs[j - 1]:
            first = b not in seen
            if birth(b) > P[0]:
                continue
            for name, ex in (("lin_tree", ex_tree), ("lin_cell", ex_cell), ("lin_cellin", ex_cellin)):
                if ex(b):
                    continue
                for g in (False, True):
                    if g and gap:
                        continue
                    nm = name + ("_gap" if g else "")
                    out[nm + "_all"].append((b, j))
                    if first:
                        out[nm].append((b, j))
        seen |= rs[j]
    return out
