#!/usr/bin/env python3
"""把探针打出的 ROW / SCAN 行按格汇总。用法：tally.py <ROW/SCAN 合并文件> [--cell 格名] [--damaged-only]

ROW 行里：N 是最新根的 txg，S 每区槽数，img 是镜像种类（healthy 之外的 zero、torn_half、torn_enum 是根槽写坏，before_root 是记录落了根槽没写，
oldest_slot_zero 是事后读坏）。候选列 G 成立、R 判红、N 不适用。
"""
import re
import sys
from collections import Counter, defaultdict

CANDIDATES = ["today", "yi_lit", "yi_gate", "ding_yi", "ding_yi_same", "jia", "ding_jia"]


def parse(line):
    head, _, rest = line.partition(" ")
    fields = {}
    for match in re.finditer(r"(\w+)=(\[[^\]]*\]|\S+)", rest):
        fields[match.group(1)] = match.group(2)
    fields["_kind"] = head
    return fields


def load(path):
    rows, scans = [], []
    for line in open(path, encoding="utf-8"):
        line = line.rstrip("\n")
        if line.startswith("ROW "):
            rows.append(parse(line))
        elif line.startswith("SCAN "):
            scans.append(parse(line))
    return rows, scans


def vector(row):
    return "".join(row.get(name, "-") for name in CANDIDATES)


def main():
    path = sys.argv[1]
    only_cell = None
    if "--cell" in sys.argv:
        only_cell = sys.argv[sys.argv.index("--cell") + 1]
    rows, scans = load(path)
    tally = defaultdict(Counter)
    for row in rows:
        if only_cell and row["cell"] != only_cell:
            continue
        big = "big" if "N" in row and int(row["N"]) + 1 >= 3 * int(row["S"]) else "small"
        key = (row["cell"], row["img"], big, "unreadable" if row.get("next_unreadable") == "1" else "readable",
               "rec" if row.get("rec_any") == "1" else "norec")
        tally[key][vector(row)] += 1
    print("# 向量次序：" + " ".join(CANDIDATES))
    for key in sorted(tally):
        print(key, dict(tally[key]))


if __name__ == "__main__":
    main()
