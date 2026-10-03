#!/usr/bin/env python3
"""坏镜像层：整条链（树根指针 + 树里每个新写单元的头）一起盖小 k，分三种：两棵树的链都盖（M2both）、只 extent 链（M2x）、只 inode 链（M2i）。
对每条非最古候选根 R_j（非回退根）数各写法抓到几次。盖到不比 R_{j-1}.txg 小的（天花板外）任何下界写法都抓不到，单列。
python3 run_r2_bad.py <段数> <最大步数> <种子> > out"""
import sys, os
sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import birth_floor_v2 as m
import r2_variants as r

SHOW = ("plain", "sc", "scp", "scp_all", "v1p", "v1p_all")

def study(histories, ks=(1, 2, 3)):
    tot = {}
    for h in histories:
        cand = sorted(m.candidates(h), key=lambda x: x[0])
        rs = [m.refs(x) for x in cand]
        for j in range(1, len(cand)):
            R, P = cand[j], cand[j - 1]
            seen = set().union(*rs[:j]) if j else set()
            fresh = [b for b in (rs[j] - seen) if b[1] == R[0]]
            if not fresh or R[1] == "R":
                continue
            def chain(idx):
                u = R[idx]
                return {b for b in (m.subtree(u) if u is not None else ()) if b in fresh}
            ci, cx = chain(2), chain(3)
            for mode, units, kinds in (("M2both", ci | cx, ("I", "X")), ("M2x(只extent链)", cx, ("X",)), ("M2i(只inode链)", ci, ("I",))):
                if not units:
                    continue
                if mode != "M2both" and not (cx and R[3] in fresh if kinds == ("X",) else ci and R[2] in fresh):
                    continue
                for k in ks:
                    if R[0] - k < 0:
                        continue
                    bo = {b: b[1] - k for b in units}
                    po = {(R[0], kd): R[0] - k for kd in kinds}
                    a = r.analyse2(h, bo=bo, po=po)
                    flags = {x: int(any(jj == j for (_, jj) in a[x])) for x in SHOW}
                    ok = R[0] - k <= P[0]
                    row = tot.setdefault((mode, k), {"n": 0, "天花板外": 0, **{x: 0 for x in SHOW}})
                    row["n"] += 1
                    row["天花板外"] += (not ok)
                    for x in SHOW:
                        row[x] += flags[x]
    return tot

if __name__ == "__main__":
    n, length, seed = int(sys.argv[1]), int(sys.argv[2]), int(sys.argv[3])
    alpha = m.ALPHABETS["API"] + [("W",)] * 4 + [a for a in m.ALPHABETS["API"] if a[0] in "RF"]
    hs = m.sample_histories(alpha, n, length, seed)
    print(f"坏镜像层·整条链盖小：{len(hs)} 段随机合法历史（API 字母表，无故障），种子 {seed}")
    print(f"  {'模式':16s} {'k':>2s} {'镜像数':>7s} {'天花板外':>8s} " + " ".join(f"{x:>8s}" for x in SHOW))
    for (mode, k), row in sorted(study(hs).items()):
        print(f"  {mode:16s} {k:2d} {row['n']:7d} {row['天花板外']:8d} " + " ".join(f"{row[x]:8d}" for x in SHOW))
