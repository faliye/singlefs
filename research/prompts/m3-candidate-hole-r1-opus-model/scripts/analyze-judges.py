#!/usr/bin/env python3
"""用法：analyze-judges.py <输出文件…>。读原型打印的 JUDGE / NOTE 行，按（写者、洞表里有没有「存在过却走不到」、开关组）
数每组开关下 I-3.1 红 / 不适用 / 成立与「池级 checker 全绿」的盘面数。写者按文件名里的 -m4a 认，洞表取 skip=rows i31="e1w" 那一行的 NOTE。"""
import collections
import re
import sys

counts = collections.Counter()
steps = collections.Counter()
for path in sys.argv[1:]:
    writer = "m4a" if "-m4a" in path else "ok"
    tables = {}
    judges = []
    for line in open(path, encoding="utf-8"):
        m = re.match(r'  NOTE (.*) skip=rows i31="e1w" HOLE_TABLE .*non_present=(\[.*\])', line)
        if m:
            tables[m.group(1)] = "ExistedUnreachable" in m.group(2)
            continue
        m = re.match(r'JUDGE (.*) skip=(\S+) i31="([^"]*)" reds=(\[[^\]]*\]) I-3.1=(\w+)', line)
        if m:
            judges.append(m.groups())
    for tag, skip, i31, reds, i31v in judges:
        unreachable = tables.get(tag)
        if unreachable is None:
            continue
        key = (writer, "unreachable" if unreachable else "no_unreachable", f"{skip}/{i31 or 'today'}")
        counts[key + ("i31=" + {"RED": "red", "NA": "na", "Holds": "holds"}.get(i31v, i31v),)] += 1
        counts[key + ("checker_all_green" if reds == "[]" else "checker_some_red",)] += 1
        if skip == "rows" and i31 == "":
            steps[(writer, "unreachable" if unreachable else "no_unreachable")] += 1
for key, n in sorted(steps.items()):
    print("STEPS", *key, n)
for key, n in sorted(counts.items()):
    print("COUNT", *key, n)
