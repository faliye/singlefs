#!/usr/bin/env python3
"""H6 在今天的脚本上重核：取今天 research/scripts/e165-pin-red-examples.py 里 state_before_outcome 与 input_fingerprint 的原文逐字执行，
对第二轮 168 行（rows.tsv：表里 151 个 ⑦ 加 17 条怪行）各算一遍，与第二轮装置侧的输出 rust_out.tsv 逐行比。装置源文件的 sha256 与第二轮相同，所以装置侧的输出不变。"""
import re, hashlib, sys
path = "/home/fy5090/code/singlefs/research/scripts/e165-pin-red-examples.py"
text = open(path, encoding="utf-8").read()
start = text.index("def state_before_outcome")
end = text.index('.encode("utf-8")).hexdigest()', text.index("def input_fingerprint")) + len('.encode("utf-8")).hexdigest()')
ns = {"re": re, "hashlib": hashlib}
exec(text[start:end], ns)
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
