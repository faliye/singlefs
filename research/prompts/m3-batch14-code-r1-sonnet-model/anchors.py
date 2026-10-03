#!/usr/bin/env python3
"""变异表锚点核：每条名字里含给定子串的行，原文在被改文件里命中几次（要恰好 1 次）。
用法：anchors.py <冻结副本根> <子串>…"""
import sys
root = sys.argv[1].rstrip("/") + "/"
needles = sys.argv[2:]
for number, line in enumerate(open(root + "crates/mutations.tsv", encoding="utf-8").read().split("\n"), 1):
    fields = line.split("\t")
    if len(fields) < 6 or not any(needle in line for needle in needles):
        continue
    name, source, original, replacement, command, test = fields[:6]
    original = original.replace("\\n", "\n")
    replacement = replacement.replace("\\n", "\n")
    hits = open(root + source, encoding="utf-8").read().count(original)
    print(f"row {number}: anchor_hits={hits} same={original == replacement} src={source}\n   name={name[:90]}\n   test={test[:130]}")
