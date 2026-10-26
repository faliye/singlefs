#!/usr/bin/env python3
"""把 mutants.tsv 里点名的一条（或用 + 连起来的几条）打到一棵树上：原文须恰好命中一次，否则退 3。
用法：apply_mutant.py <mutants.tsv> <树根> <名字前缀>[+<名字前缀>…]"""
import sys


def unescape(text):
    out, i = [], 0
    while i < len(text):
        if text[i] == "\\" and i + 1 < len(text):
            out.append({"n": "\n", "t": "\t", "\\": "\\"}[text[i + 1]])
            i += 2
        else:
            out.append(text[i])
            i += 1
    return "".join(out)


table, root, wanted = sys.argv[1], sys.argv[2], sys.argv[3].split("+")
rows = [line.rstrip("\n").split("\t") for line in open(table, encoding="utf-8") if line.strip() and not line.startswith("#")]
for prefix in wanted:
    matching = [row for row in rows if row[0].split(" ")[0] == prefix]
    if len(matching) != 1:
        print(f"变异 {prefix} 在表里命中 {len(matching)} 行")
        sys.exit(3)
    name, path, old, new = matching[0]
    old, new = unescape(old), unescape(new)
    full = f"{root}/{path}"
    text = open(full, encoding="utf-8").read()
    if text.count(old) != 1:
        print(f"变异 {prefix}：原文在 {path} 里命中 {text.count(old)} 次")
        sys.exit(3)
    open(full, "w", encoding="utf-8").write(text.replace(old, new))
    print(f"applied {prefix} {path}")
