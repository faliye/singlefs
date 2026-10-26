#!/usr/bin/env python3
"""核第二轮判决第六节说的「辩方 summary-rows.txt 与主 agent 复跑的逐行比：38 行不同、归类逐个相同」与「33 个键的分类在 S 之间不变」。
不读 rows.diff，直接把两份 summary-rows.txt 各行的「成立 / 判红 / 不适用」三个数折成「哪几类非零」再比。
用法：rows-diff-check.py <腿的 summary-rows.txt> <复跑的 summary-rows.txt>"""
import re
import sys
from collections import defaultdict


def parse(path):
    parsed = {}
    for line in open(path, encoding="utf-8"):
        match = re.match(r"S=(\d+)\s+(\S+)\s+(\S+)\s+(\S+)\s+张数=(\d+)\s+(.*)$", line.rstrip())
        if not match:
            continue
        s, cell, image, kind, count, rest = match.groups()
        columns = [c.strip() for c in rest.split("|")]
        classes = []
        for column in columns:
            holds, violated, not_applicable = map(int, column.split("/"))
            classes.append("".join(letter for letter, number in zip("HVN", (holds, violated, not_applicable)) if number > 0))
        parsed[(int(s), cell, image, kind)] = (int(count), classes, columns)
    return parsed


leg, rerun = parse(sys.argv[1]), parse(sys.argv[2])
print("行数：腿", len(leg), "复跑", len(rerun), "；只在一边有的行：", len(set(leg) ^ set(rerun)))
different = [k for k in sorted(set(leg) & set(rerun)) if leg[k][2] != rerun[k][2]]
class_different = [k for k in different if leg[k][1] != rerun[k][1]]
print("数字不同的行", len(different), "；其中归类（哪几类非零）不同的行", len(class_different))
for name, parsed in (("腿", leg), ("复跑", rerun)):
    by_key = defaultdict(dict)
    for (s, cell, image, kind), (_, classes, _) in parsed.items():
        by_key[(cell, image, kind)][s] = tuple(classes)
    moving = [k for k, v in by_key.items() if len(set(v.values())) > 1]
    print(name, "：不同的 (格, 镜像, 种类) 键", len(by_key), "个，归类在 S 之间变了的", len(moving), "个")
