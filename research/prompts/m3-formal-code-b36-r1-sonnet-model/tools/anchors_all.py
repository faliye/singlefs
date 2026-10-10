#!/usr/bin/env python3
"""冻结副本 crates/mutations.tsv 整表：每行的锚点（第 3 列，\\n 还原成换行）在它的源文件（第 2 列）里命中几次；不是恰好 1 次的列出来。
只读、不起 cargo。用法 anchors_all.py <冻结副本根>"""
import sys
root = sys.argv[1]
rows = [l.split("\t") for l in open(f"{root}/crates/mutations.tsv", encoding="utf-8").read().split("\n") if l]
print("行数", len(rows), "列数不是 6 的行", sum(1 for r in rows if len(r) != 6))
cache, bad = {}, []
for i, r in enumerate(rows, 1):
    if len(r) != 6:
        continue
    name, src, find, repl, args, test = r
    if src not in cache:
        try: cache[src] = open(f"{root}/{src}", encoding="utf-8").read()
        except OSError: cache[src] = None
    body = cache[src]
    if body is None: bad.append((i, "源文件不存在", src)); continue
    n = body.count(find.replace("\\n", "\n"))
    if n != 1: bad.append((i, n, name[:50]))
print("锚点不是恰好命中一次的行:", bad)
