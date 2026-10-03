#!/usr/bin/env python3
"""列出清单里不在任何函数里的运算符变异（cargo-mutants 的名字不带「in <函数>」、也不是「replace <函数> with <值>」），即常量 / static 初始化式里的变异，并配上这一版结果表里的结局。"""
import json, re, sys
from collections import Counter
repo, version = sys.argv[1], sys.argv[2]
d = json.load(open(version + "/list.json"))
res = {}
for line in open(version + "/results.tsv"):
    if line.startswith("#"): continue
    f = line.rstrip("\n").split("\t"); res[int(f[0])] = (f[3], f[5])
src = {}
def text_of(i):
    p = repo + "/" + i["file"]
    if p not in src: src[p] = open(p).read().split("\n")
    return src[p][i["start"][0] - 1].strip()
out = []
for i in d:
    desc = i["name"].split(": ", 1)[1]
    if " in " in desc: continue
    m = re.fullmatch(r"replace (\S+) with (\S+)", desc)
    if not m: continue
    if re.search(r"[A-Za-z]", m.group(1)): continue   # 换掉整个函数体的那一类
    out.append(i)
print("total", len(d), "operator-mutations-outside-any-fn", len(out))
print("outcomes", dict(Counter(res.get(i["sequence"], ("none", ""))[0] for i in out)))
print("by-package", dict(Counter(i["file"].split("/")[1] for i in out)))
bad = [i for i in out if not re.search(r"\b(const|static)\b", text_of(i)) and not text_of(i).endswith((",", ";", "+", "-", "*", "("))]
print("lines-not-looking-like-initializer", len(bad))
for i in out:
    o = res.get(i["sequence"], ("none", ""))
    print(i["sequence"], o[0], o[1], "|", i["name"], "|", text_of(i)[:100])
