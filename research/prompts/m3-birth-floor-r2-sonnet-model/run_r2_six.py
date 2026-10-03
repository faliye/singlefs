#!/usr/bin/env python3
"""第一轮判决 V1 出局那「6 张」：FAULT 字母表去掉 X、深度 6 的穷举里，V1 系（opusI 与 v1p）红的镜像逐张列出，并看各写法在这几张上判什么。
python3 run_r2_six.py"""
import sys, os
sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import birth_floor_v2 as m
import r2_variants as r

alpha = [x for x in m.ALPHABETS["FAULT"] if x[0] != "X"]
hits_opus, hits_v1p, total = [], [], 0
def rec(h, d):
    global total
    if d > 0:
        total += 1
        if m.analyse(h)["opusI"]:
            hits_opus.append(h)
        a = r.analyse2(h)
        if a["v1p"] or a["v1p_all"]:
            hits_v1p.append((h, {k: len(a[k]) for k in ("plain", "sc", "scp_all", "v1p", "v1p_skip")}))
    if d == 6:
        return
    for op in alpha:
        n = m.step(h, op, False, True)
        if n is not None:
            rec(n, d + 1)
rec(m.start(), 0)
print(f"FAULT 去掉 X、深度 6 穷举：{total} 张镜像；opusI（第一轮攻方 V1 写法）红 {len(hits_opus)} 张；v1p（V1 条件 ∧ 压得住的继承子树，攻方的 V3s）红 {len(hits_v1p)} 张")
for h in hits_opus:
    a = r.analyse2(h)
    cand = sorted(m.candidates(h), key=lambda x: x[0])
    print("  opusI 红：", h.ops, " 候选集 txg", [x[0] for x in cand], " 丢掉的根槽 txg", sorted(h.lost),
          " 各写法红的单元数", {k: len(a[k]) for k in ("plain", "sc", "scp", "v1p", "v1p_skip")})
