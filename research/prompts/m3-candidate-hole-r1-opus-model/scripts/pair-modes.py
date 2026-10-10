#!/usr/bin/env python3
"""用法：pair-modes.py <草稿 out 目录> <前缀>。坏写者 m4a 的每一步与同参数正确写者的同一步配对，按「洞表里有没有存在过却走不到」
「m4a 盘 0 已分配是否少于正确写者（short>0 = 坏镜像，记账少）」分格，数每组开关（rows/today、rows/na、rows/e1、rows/e1w、rows/rc、rows/rcw）
判 I-3.1 红 / 不适用 / 成立的盘面数。"""
import collections
import glob
import re
import sys

out, prefix = sys.argv[1], sys.argv[2]
MODES = ("", "na", "e1", "e1w", "rc", "rcw")


def parse(path):
    d = collections.defaultdict(dict)
    for line in open(path, encoding="utf-8"):
        m = re.match(r'  NOTE (.*) skip=rows i31="e1w" I31 device=0 allocated=Some\((\d+)\)', line)
        if m:
            d[m.group(1)]["alloc"] = int(m.group(2))
        m = re.match(r'  NOTE (.*) skip=rows i31="e1w" HOLE_TABLE .*non_present=(\[.*\])', line)
        if m:
            d[m.group(1)]["unr"] = "ExistedUnreachable" in m.group(2)
        m = re.match(r'JUDGE (.*) skip=rows i31="([^"]*)" reds=\[[^\]]*\] I-3.1=(\w+)', line)
        if m:
            d[m.group(1)][m.group(2)] = m.group(3)
    return d


counts = collections.Counter()
for bad in sorted(glob.glob(f"{out}/{prefix}*-m4a.out")):
    b, o = parse(bad), parse(bad.replace("-m4a.out", "-ok.out"))
    for tag, e in b.items():
        if "alloc" not in e:
            continue
        short = o.get(tag, {}).get("alloc", 0) - e["alloc"]
        cell = ("unreachable" if e.get("unr") else "no_unreachable", "short>0" if short > 0 else "short<=0")
        for mode in MODES:
            counts[cell + (mode or "today", e.get(mode, "?"))] += 1
for key, n in sorted(counts.items()):
    print("PAIR", *key, n)
