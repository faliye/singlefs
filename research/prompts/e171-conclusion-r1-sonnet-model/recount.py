#!/usr/bin/env python3
"""只读：从比对器产物的 name=row 与 name=row_matrix 行重数 Q1 全放行名单与 Q3 只靠一条名单（批 1，O3 取严格甲），
并算「把没触达却红的两行算进去」之后的数。用法：recount.py <产物> """
import re, sys

def kv(line):
    out = {}
    for tok in line.split(" ")[1:]:
        if "=" in tok:
            k, v = tok.split("=", 1)
            out.setdefault(k, v)
    return out

rows = {}
matrix = {}
for line in open(sys.argv[1], encoding="utf-8"):
    line = line.rstrip("\n")
    if line.startswith("E7RESULT name=row row="):
        d = kv(line.replace("E7RESULT ", "E7RESULT ", 1))
        # row_name 里没有空格（下划线替换）
        rows[int(d["row"])] = d["category"]
    elif line.startswith("E7RESULT name=row_matrix row="):
        d = kv(line)
        matrix[(int(d["row"]), int(d["batch"]))] = d

def red(d):
    return {c for c, k in (("O1", "O1"), ("O2", "O2"), ("O3", "O3甲")) if d.get(k) == "red"}

def tally(include_untouched_red):
    counted = [r for r, c in rows.items() if c == "counted" or (include_untouched_red and c == "untouched" and (r, 1) in matrix and red(matrix[(r, 1)]))]
    quiet, only = [], {"O1": [], "O2": [], "O3": []}
    for r in sorted(counted):
        c = red(matrix[(r, 1)])
        if not c:
            quiet.append(r)
        elif len(c) == 1:
            only[next(iter(c))].append(r)
    return len(counted), quiet, only

for flag in (False, True):
    n, quiet, only = tally(flag)
    print(f"include_untouched_but_red={flag} D={n} quiet_rows_alpha={len(quiet)} {','.join(map(str, quiet))}")
    print("   only=" + " ".join(f"{k}:{len(v)}[{','.join(map(str, v))}]" for k, v in only.items()))
cats = {}
for r, c in rows.items():
    cats[c] = cats.get(c, 0) + 1
print("categories", dict(sorted(cats.items())))
