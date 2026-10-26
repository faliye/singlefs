#!/usr/bin/env python3
"""compare_leg_scan.py <diff-vs-current.txt> <第二轮正推腿的 rerun-1.out>：narrow 块（只收按路径载入的模块、与它们 import 引来的模块的模块级 import）逐入口的单元数，对第二轮正推腿 SCAN load-edge 那一段（同一种补丁、同一份 35 入口清单）。"""
import re, sys
mine = {}
block = open(sys.argv[1], encoding="utf-8").read().split("DIFF narrow:")[1].split("DIFF ")[0]
for line in block.splitlines()[1:]:
    m = re.match(r"\s+(\S+) rc .*原始行 (\d+)→(\d+)", line)
    if m: mine[m.group(1)] = (int(m.group(2)), int(m.group(3)))
his, lines = {}, open(sys.argv[2], encoding="utf-8").read().splitlines()
begin = next(i for i, l in enumerate(lines) if l.startswith("SCAN load-edge"))
for line in lines[begin + 1:]:
    m = re.match(r"\s+(\S+) rc .*单元 (\d+)→(\d+)", line)
    if not m: break
    his[m.group(1)] = (int(m.group(2)), int(m.group(3)))
print(f"name=compare_leg_scan 现在这棵树上变的入口={len(mine)} 腿报的={len(his)}")
print("  仅现在有：", sorted(set(mine) - set(his)))
print("  仅腿有：", sorted(set(his) - set(mine)))
deltas = [mine[e][1] - mine[e][0] for e in mine if e in his]
print(f"  两边都有的 {len(deltas)} 个：增量范围 {min(deltas)}~{max(deltas)}；单元数不同的：" + "；".join(f"{e.split('::')[1]} 现在{mine[e]} 腿{his[e]}" for e in sorted(set(mine) & set(his)) if mine[e] != his[e]))
