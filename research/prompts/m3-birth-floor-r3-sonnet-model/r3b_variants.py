#!/usr/bin/env python3
"""第三轮辩方腿：在两片叶的 inode 树模型（birth_floor_v3.py：覆盖写重写低号叶、建 inode 重写高号叶）上，
数 TF（scp，第二轮判决措辞：按树继承下界 + 压得住 + 严格小于）与 V1 系（v1p）在几种「缺口」处置下红的镜像数。
缺口处置：none 不处置；all = R.txg ≠ R⁻.txg + 1 就整对不判（第二轮判决定的 gap）；lost = 只有 (R⁻,R) 之间有根槽读不出 / 被盖的 txg 才整对不判；
ab = 只有之间有被抛弃的根才整对不判。写法后缀 _all = 相邻差集全判（β），不带 = 只判第一次出现（α 的下界半）。
再数：相邻对总数、gap=all 跳过的对数、被跳过的对里新单元数（盲区的量）。"""
import sys
sys.dont_write_bytecode = True
import birth_floor_v3 as m
from birth_floor_v3 import candidates, refs, subtree

VARS = ("scp", "scp_all", "v1p", "v1p_all", "plain", "plain_all")
GAPS = ("none", "all", "lost", "ab")

def analyse3(h, bo=None, po=None):
    bo = bo or {}
    po = po or {}
    birth = lambda b: bo.get(b, b[1])
    cand = sorted(candidates(h), key=lambda r: r[0])
    rs = [refs(r) for r in cand]
    out = {f"{v}|{g}": [] for v in VARS for g in GAPS}
    pairs = 0
    skipped_pairs = 0
    cells = 0
    cells_in_skipped = 0
    skipped_lost = 0
    cells_in_skipped_lost = 0
    seen = set(rs[0]) if rs else set()
    for j in range(1, len(cand)):
        R, P = cand[j], cand[j - 1]
        pairs += 1
        gap_all = R[0] != P[0] + 1
        gap_lost = any(P[0] < t < R[0] for t in h.lost)
        gap_ab = any(P[0] < t < R[0] for t in h.ab)
        gapflag = {"none": False, "all": gap_all, "lost": gap_lost, "ab": gap_ab}
        if gap_all:
            skipped_pairs += 1
        if gap_lost:
            skipped_lost += 1
        ptr, inh, pressed = {}, {}, {}
        for kind, idx in (("I", 2), ("X", 3)):
            u = R[idx]
            if u is None:
                inh[kind] = False; ptr[kind] = None; pressed[kind] = False
                continue
            pb = po.get((R[0], kind), u[1])
            ptr[kind] = u
            inh[kind] = (u not in rs[j - 1]) and pb < P[0]
            pressed[kind] = all(birth(x) <= pb for x in subtree(u))
        def exempt(b, need_pressed, need_inode):
            if need_inode and not inh["I"]:
                return False
            return any(inh[k] and (pressed[k] or not need_pressed) and b in subtree(ptr[k]) for k in ("I", "X"))
        for b in rs[j] - rs[j - 1]:
            first = b not in seen
            cells += 1
            if gap_all:
                cells_in_skipped += 1
            if gap_lost:
                cells_in_skipped_lost += 1
            if birth(b) > P[0]:
                continue
            red = {"plain": first, "plain_all": True,
                   "scp": first and not exempt(b, True, False), "scp_all": not exempt(b, True, False),
                   "v1p": first and not exempt(b, True, True), "v1p_all": not exempt(b, True, True)}
            for v, isred in red.items():
                if not isred:
                    continue
                for g in GAPS:
                    if not gapflag[g]:
                        out[f"{v}|{g}"].append((b, j))
        seen |= rs[j]
    out["_pairs"] = pairs
    out["_skipped_pairs"] = skipped_pairs
    out["_cells"] = cells
    out["_cells_in_skipped"] = cells_in_skipped
    out["_skipped_lost"] = skipped_lost
    out["_cells_in_skipped_lost"] = cells_in_skipped_lost
    return out
