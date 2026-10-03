#!/usr/bin/env python3
"""python3 run_r3e_bad.py <段数> <最大步数> <种子>：两片叶模型上，随机合法历史（API 字母表，无故障）里每条非回退候选根 R 的新写单元盖小 k（k=1,2,3）：
两条链都盖（M2both）、只盖 extent 链（M2x）、只盖 inode 链（M2i）；数 TF（scp_all）与 V1 系（v1p_all）各抓到几张。"""
import sys, os
sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import birth_floor_v3 as m
import r3c_variants as r
n, length, seed = int(sys.argv[1]), int(sys.argv[2]), int(sys.argv[3])
alpha = m.ALPHABETS["API"] + [("W",)] * 4 + [a for a in m.ALPHABETS["API"] if a[0] in "RF"]
hs = m.sample_histories(alpha, n, length, seed)
tot = {}
for h in hs:
    cand = sorted(m.candidates(h), key=lambda x: x[0])
    rs = [m.refs(x) for x in cand]
    for j in range(1, len(cand)):
        R, P = cand[j], cand[j - 1]
        seen = set().union(*rs[:j])
        fresh = [b for b in (rs[j] - seen) if b[1] == R[0]]
        if not fresh or R[1] == "R":
            continue
        ci = {b for b in (m.subtree(R[2]) if R[2] is not None else ()) if b in fresh}
        cx = {b for b in (m.subtree(R[3]) if R[3] is not None else ()) if b in fresh}
        for mode, units, kinds in (("M2both", ci | cx, ("I", "X")), ("M2x", cx, ("X",)), ("M2i", ci, ("I",))):
            if not units:
                continue
            for k in (1, 2, 3):
                if R[0] - k < 0:
                    continue
                bo = {b: b[1] - k for b in units}
                po = {(R[0], kd): R[0] - k for kd in kinds}
                a = r.analyse3(h, bo=bo, po=po)
                row = tot.setdefault((mode, k), {"n": 0, "scp_all|none": 0, "v1p_all|none": 0, "scp_all|all": 0, "v1p_all|all": 0})
                row["n"] += 1
                for x in ("scp_all|none", "v1p_all|none", "scp_all|all", "v1p_all|all"):
                    row[x] += int(any(jj == j for (_, jj) in a[x]))
print(f"[r3 两片叶模型·整条链盖小] {len(hs)} 段随机合法历史（API 字母表，无故障），种子 {seed}")
print("  模式 k | 镜像数 | TF(scp_all) | V1系(v1p_all) | TF+gap | V1系+gap")
for (mode, k), row in sorted(tot.items()):
    print(f"  {mode} k={k} | {row['n']} | {row['scp_all|none']} | {row['v1p_all|none']} | {row['scp_all|all']} | {row['v1p_all|all']}")
