#!/usr/bin/env python3
"""H6：同一批行（表里 151 个 ⑦ 加 17 条构造的怪行）上，research/scripts/e165-pin-red-examples.py 的 input_fingerprint（取原文第 99–110 行逐字执行）
与装置 input_fingerprint_of（zz_h6_fingerprint.rs 里取原文 1724–1737 行逐字编的）各算一遍，逐行比。用法：compare.py"""
import re, hashlib, sys
src = open("/home/fy5090/code/singlefs/research/prompts/m3-verus-land-code-r2-sonnet-model/e165-pin-red-examples.py.read", encoding="utf-8").read().split("\n")
ns = {"re": re, "hashlib": hashlib}
exec("\n".join(src[99:110]), ns)   # 第 100–110 行：state_before_outcome 与 input_fingerprint
py = []
for line in open("rows.tsv", encoding="utf-8").read().split("\n"):
    if not line: continue
    d, n, row = line.split("\t", 2)
    row = row.replace("\\t", "\t").replace("\\r", "\r")
    py.append(ns["input_fingerprint"](d, n, row) or "None")
rs = [l.split("\t")[1] for l in open("rust_out.tsv", encoding="utf-8").read().split("\n") if l]
diff = [(i, p, r) for i, (p, r) in enumerate(zip(py, rs)) if p != r]
print(f"行数 python={len(py)} rust={len(rs)}；不同 {len(diff)} 行；None 的行数 python={py.count('None')} rust={rs.count('None')}")
for i, p, r in diff: print("不同：行", i, p, r)
