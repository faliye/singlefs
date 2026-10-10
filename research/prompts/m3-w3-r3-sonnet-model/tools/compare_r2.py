#!/usr/bin/env python3
# 用法：compare_r2.py <r2 辩方腿的 expected 文件> <这一轮同一组的输出>
# 老候选（r2 辩方腿的 25 个）的每个字段、W3MODEL 那一行的每个字段，两边逐数相等才退 0；新追加的候选与字段不比。
import sys
def parse(path):
    head = None; rows = {}
    for line in open(path, encoding="utf-8"):
        if line.startswith("W3MODEL"):
            head = dict(kv.split("=", 1) for kv in line.split()[1:])
        elif line.startswith("W3COUNT"):
            d = dict(kv.split("=", 1) for kv in line.split()[1:]); rows[d["candidate"]] = d
    return head, rows
oh, orows = parse(sys.argv[1]); nh, nrows = parse(sys.argv[2])
bad = 0; cells = 0
for k, v in oh.items():
    cells += 1
    if nh.get(k) != v:
        print(f"头不同 {k}: 老={v} 新={nh.get(k)}"); bad += 1
for cand, d in orows.items():
    if cand not in nrows:
        print(f"缺候选 {cand}"); bad += 1; continue
    for k, v in d.items():
        cells += 1
        if nrows[cand].get(k) != v:
            print(f"不同 {cand} {k}: 老={v} 新={nrows[cand].get(k)}"); bad += 1
print(f"compare-r2 比了 {cells} 格（{len(orows)} 个老候选），不同 {bad} 格")
sys.exit(1 if bad else 0)
