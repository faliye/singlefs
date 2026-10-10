#!/usr/bin/env python3
"""J1-J3：第 26 批三份补丁的变异行（append 与 replacements）逐行在冻结副本里数锚点：旧串（\\n 还原成换行）要恰好命中 1 次。
用法：check_anchors.py <改动全文 md> <冻结树根>
"""
import re, sys
md, tree = sys.argv[1], sys.argv[2]
lines = open(md, encoding="utf-8").read().split("\n")
section = None; patch = None; fence = False; rows = []
for i, line in enumerate(lines, 1):
    if line.startswith("## m3-formal-26"): patch = line[3:].strip()
    elif line.startswith("### mutations-append.tsv"): section = "append"
    elif line.startswith("### mutations-replacements.tsv"): section = "replace"
    elif line.startswith("### "): section = None
    elif line.startswith("````"):
        fence = not fence
    elif fence and section and line.strip():
        rows.append((patch, section, i, line))
bad = 0; cache = {}
for patch, section, lineno, row in rows:
    cols = row.split("\t")
    if len(cols) != 6:
        print("COLS", patch, section, lineno, len(cols)); bad += 1; continue
    name, path, old, new, cmd, test = cols
    old = old.replace("\\n", "\n")
    text = cache.setdefault(path, open(f"{tree}/{path}", encoding="utf-8").read())
    n = text.count(old)
    status = "OK" if n == 1 else "BAD"
    if n != 1: bad += 1
    print(status, patch, section, f"md:{lineno}", f"hits={n}", name[:50])
print("rows", len(rows), "bad", bad)
