#!/usr/bin/env python3
"""用法：anchors-check.py <改动原样.md> <冻结副本根>：数改动原样里每条变异行（六段、第二段以 crates/ 开头）的「原文」在冻结副本对应文件里命中几次。"""
import sys
diff_path, tree = sys.argv[1], sys.argv[2]
rows = []
for line in open(diff_path, encoding="utf-8").read().split("\n"):
    fields = line.split("\t")
    if len(fields) >= 6 and fields[1].startswith("crates/"):
        rows.append(fields)
bad = 0
for number, fields in enumerate(rows, 1):
    source = open(tree.rstrip("/") + "/" + fields[1], encoding="utf-8").read()
    hits = source.count(fields[2].replace("\\n", "\n"))
    if hits != 1:
        bad += 1
    print(f"row {number:2d} hits={hits} file={fields[1]}")
print(f"ROWS={len(rows)} NOT_EXACTLY_ONE={bad}")
