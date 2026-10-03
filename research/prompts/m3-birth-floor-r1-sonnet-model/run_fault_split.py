#!/usr/bin/env python3
"""故障档拆开看：只有 X（抛弃最新根）/ 只有 L（丢根槽）各自让哪些写法红。python3 run_fault_split.py > run_fault_split.out"""
import os, sys
sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import birth_floor_v2 as m
no_l = [x for x in m.ALPHABETS["FAULT"] if x[0] != "L"]
no_x = [x for x in m.ALPHABETS["FAULT"] if x[0] != "X"]
s, c, e = m.dfs(no_l, 7, False, True)
m.show("[穷举] FAULT 去掉 L（只有 X 抛弃最新根）深度 7", s, "plain 红单元按 R_first 归类 " + str(c))
s, c, e = m.dfs(no_x, 6, False, True)
m.show("[穷举] FAULT 去掉 X（只有 L 丢根槽）深度 6", s, "plain 红单元按 R_first 归类 " + str(c))
for k in ("opusI", "both"):
    if k in e:
        print(f"   首例 {k}: {e[k]}")
