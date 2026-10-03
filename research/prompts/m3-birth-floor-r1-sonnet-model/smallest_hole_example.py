#!/usr/bin/env python3
"""F3：最小的合法历史上 ① 就红：第一个文件之后覆盖写一次、回退到第一版，没有抬 F、目标还在候选集里。python3 smallest_hole_example.py > smallest_hole_example.out"""
import os, sys
sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import birth_floor_v2 as m
h = m.step(m.step(m.start(), ("W",)), ("R", ("o", 0)))
a = m.analyse(h)
print("操作:", h.ops)
print("候选根 (txg, 谁写的):", [(r[0], r[1]) for r in a["cand"]])
print("① 连续无洞，原写法（holes_plain）红的单元数:", len(a["holes_plain"]), "例:", a["holes_plain"][:2])
print("② 下界 plain 红:", a["plain"], " 下界 sc_lt 红:", a["sc_lt"])
print("① 放宽（holes_lt）红:", a["holes_lt"], " 出现规则 app_lt 红:", a["app_lt"])
cand = a["cand"]
for j in range(1, len(cand)):
    pass
for b in sorted({u for (u, _) in a["holes_plain"]}, key=repr):
    pos = [i for i, r in enumerate(cand) if b in m.refs(r)]
    print("  单元", b[0], "诞生代号", b[1], "被候选根（txg）引用:", [cand[i][0] for i in pos])
