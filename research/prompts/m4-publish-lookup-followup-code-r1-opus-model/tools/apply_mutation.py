#!/usr/bin/env python3
"""apply_mutation.py <变异表> <名> <副本根>：把表里那一行的原文在副本里那份文件中换成变异文，原文要恰好命中一次。"""
import sys
from pathlib import Path

def unescape(text):
    out, i = [], 0
    while i < len(text):
        if text[i] == "\\" and i + 1 < len(text):
            out.append({"n": "\n", "t": "\t", "\\": "\\"}[text[i + 1]]); i += 2
        else:
            out.append(text[i]); i += 1
    return "".join(out)

table, name, root = sys.argv[1], sys.argv[2], Path(sys.argv[3])
for line in Path(table).read_text(encoding="utf-8").splitlines():
    fields = line.split("\t")
    if fields[0] != name:
        continue
    path = root / fields[2]
    old, new = unescape(fields[3]), unescape(fields[4])
    text = path.read_text(encoding="utf-8")
    hits = text.count(old)
    if hits != 1:
        sys.exit(f"{name}: 原文在 {path} 里命中 {hits} 次，不是 1 次")
    path.write_text(text.replace(old, new), encoding="utf-8")
    print(f"applied {name} {fields[2]}")
    sys.exit(0)
sys.exit(f"{name}: 表里没有这一行")
