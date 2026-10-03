#!/usr/bin/env python3
"""python3 run_r3c_bad.py <段数> <最大步数> <种子>：在随机合法历史（FAULT 字母表，含丢根槽 L、抛弃 X）里，只在「R.txg ≠ R⁻.txg + 1」的对上盖小 R 新写的单元，
数 gap=all 与 gap=fxonly 各抓到几个。盖法：固定点单元 fx（树表、记账、分配记录树、中央映射根）诞生代号改成 R⁻.txg；用户可见树里的新叶（Ila / Ilb / Xl）同样改。"""
import sys, os
sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import birth_floor_v3 as m
import r3c_variants as r

n, length, seed = int(sys.argv[1]), int(sys.argv[2]), int(sys.argv[3])
alpha = m.ALPHABETS["FAULT"] + [("W",)] * 3 + [("N",)] * 2
import random
def sample(alpha, n, length, seed):
    rnd = random.Random(seed)
    out = []
    for _ in range(n):
        h = m.start()
        for _ in range(rnd.randint(4, length)):
            for _try in range(60):
                nh = m.step(h, rnd.choice(alpha), False, True)
                if nh is not None:
                    h = nh
                    break
        out.append(h)
    return out
hs = sample(alpha, n, length, seed)
tot = {}
holes = 0
for h in hs:
    cand = sorted(m.candidates(h), key=lambda x: x[0])
    rs = [m.refs(x) for x in cand]
    for j in range(1, len(cand)):
        R, P = cand[j], cand[j - 1]
        if R[0] == P[0] + 1:
            continue
        holes += 1
        seen = set().union(*rs[:j])
        fresh = [b for b in (rs[j] - seen) if b[1] == R[0]]
        groups = {"固定点 fx": [b for b in fresh if b[0] == "fx"],
                  "用户树新叶 Ila/Ilb/Xl": [b for b in fresh if b[0] in ("Ila", "Ilb", "Xl")]}
        for name, units in groups.items():
            for b in units:
                bo = {b: P[0]}
                a = r.analyse3(h, bo=bo)
                row = tot.setdefault(name, {"n": 0, "none": 0, "all": 0, "fxonly": 0})
                row["n"] += 1
                for g in ("none", "all", "fxonly"):
                    row[g] += int(any(jj == j for (_, jj) in a["scp_all|" + g]))
print(f"[r3 缺口对上的盖小] {len(hs)} 段历史（FAULT 字母表加权，种子 {seed}），含缺口的相邻对 {holes} 个；每个新单元各盖一张（改成 R⁻.txg）")
print("  类 | 张数 | 不处置抓到 | gap=all 抓到 | gap=fxonly 抓到")
for name, row in tot.items():
    print(f"  {name} | {row['n']} | {row['none']} | {row['all']} | {row['fxonly']}")
