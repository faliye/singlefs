#!/usr/bin/env python3
# 用法：agg.py <结果文件…>  对所有 name=history 行把 w3_ 的数字字段求和，排成按候选的表
import sys, re, collections
tot = collections.Counter(); n = 0
for path in sys.argv[1:]:
    for line in open(path, encoding="utf-8"):
        if not line.startswith("E7RESULT name=history"): continue
        n += 1
        for tok in line.split():
            if tok.startswith("w3_") and "=" in tok:
                k, v = tok.split("=", 1)
                if re.fullmatch(r"\d+", v): tot[k] += int(v)
names = ["c1","c2a","c2b","c2c","c2d","c3","c4","c1b","c2i","c4g","c2ig"]
print(f"histories={n} q1_events={tot['w3_q1_events']} gone_events={tot['w3_gone_events']} hole_events(I-9.29 与 I-9.6 都不红)={tot['w3_hole_events']} residual_steps={tot['w3_residual_steps']} residual_hole_steps={tot['w3_residual_hole_steps']} steps_judged={tot['w3_steps_judged']}")
print("cand   q1_red  gone_red  hole_red  residual_red  residual_hole_red  red_steps")
for c in names:
    print(f"{c:6} {tot['w3_q1_red_'+c]:7} {tot['w3_gone_red_'+c]:9} {tot['w3_hole_red_'+c]:9} {tot['w3_residual_red_'+c]:13} {tot['w3_residual_hole_red_'+c]:18} {tot['w3_red_steps_'+c]:10}")
