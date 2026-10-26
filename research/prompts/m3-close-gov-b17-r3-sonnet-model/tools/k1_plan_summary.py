#!/usr/bin/env python3
"""K1 路径 3：各份 plan 输出的末行，和每一行相对 seedbase-base-base 的复用 / 现判 / 拒算变化。用法：k1_plan_summary.py <目录> <标签…>（头一个是参照）"""
import collections
import os
import sys

directory, labels = sys.argv[1], sys.argv[2:]
def load(label):
    rows = {}
    for line in open(os.path.join(directory, label + ".txt"), encoding="utf-8"):
        parts = line.rstrip("\n").split("\t")
        if len(parts) >= 3 and parts[0].isdigit():
            rows[int(parts[0])] = parts[2]
    return rows
reference = load(labels[0])
for label in labels:
    rows = load(label)
    changed = collections.Counter(f"{reference[line]}->{rows[line]}" for line in reference if reference[line] != rows[line])
    print(f"{label}: {dict(sorted(collections.Counter(rows.values()).items()))} 相对参照变了 {dict(sorted(changed.items()))}")
