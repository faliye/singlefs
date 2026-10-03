#!/usr/bin/env python3
"""V2①：拿模型的语法切函数，对库目录这一版清单里每条变异找包住它的最内层 / 最外层 fn，与变异名里的函数名比。用法：parser_check.py <副本> <库目录版本目录>"""
import json, os, re, sys, collections
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from fnparse import items
REPO, D = sys.argv[1:3]
L = json.load(open(os.path.join(D, "list.json")))
cache = {}
stats = collections.Counter(); examples = collections.defaultdict(list)
for m in L:
    if m["file"] not in cache:
        text = open(os.path.join(REPO, m["file"]), encoding="utf-8").read()
        cache[m["file"]] = (text, items(text), [0] + [i + 1 for i, c in enumerate(text) if c == "\n"])
    text, fns, starts = cache[m["file"]]
    offset = starts[m["start"][0] - 1] + m["start"][1] - 1
    around = [f for f in fns if f["start"] <= offset < f["end"]]
    desc = m["name"].split(": ", 1)[1]
    found = re.search(r" in (.+)$", desc) or re.match(r"replace (.+?) -> ", desc) or re.match(r"replace (\S+) with \(\)$", desc)
    named = found.group(1).split("::")[-1].split("<")[0] if found else None
    if not around:
        k = "no-enclosing-fn"
    else:
        inner = min(around, key=lambda f: f["end"] - f["start"]); outer = max(around, key=lambda f: f["end"] - f["start"])
        if len(around) > 1:
            stats["nested(inner!=outer)"] += 1; examples["nested(inner!=outer)"].append(m["name"])
        k = "innermost-name-matches" if named == inner["name"] else "innermost-name-differs"
    stats[k] += 1
    if len(examples[k]) < 4:
        examples[k].append(m["name"])
print("mutations", len(L), dict(stats))
for k, v in examples.items():
    for e in v[:4]:
        print(" ", k, "|", e)
