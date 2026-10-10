#!/usr/bin/env python3
"""从冻结副本的 crates/mutations.tsv 里选出第 34 批 34-1..34-4 点名的行（名字取自改动全文），逐行核：整行只出现一次、锚点在源文件里恰好命中一次。
用法 select_rows.py <副本根> <改动全文.md> -> 打印 序号、批号、名字前 40 字、源文件、锚点命中次数、档（harness / checker-tier）。写 rows.json 给 mutate_rows.py。"""
import json, re, sys
root, diff = sys.argv[1], sys.argv[2]
tsv = open(f"{root}/crates/mutations.tsv", encoding="utf-8").read().split("\n")
text = open(diff, encoding="utf-8").read().split("\n")
sections = []  # (批, 表名, 行)
batch = None; table = None; inblock = False
for line in text:
    m = re.match(r"^## (m3-formal-34-\d|#9)", line)
    if m: batch = m.group(1)
    m = re.match(r"^### (mutations-\w+\.tsv)", line)
    if m: table = m.group(1); continue
    if line.startswith("````") and table:
        inblock = not inblock
        if not inblock: table = None
        continue
    if inblock and table and "\t" in line:
        sections.append((batch, table, line))
out = []
for batch, table, line in sections:
    name = line.split("\t")[0]
    hits = [i for i, r in enumerate(tsv) if r.split("\t")[0] == name]
    same = [i for i in hits if tsv[i] == line]
    cols = line.split("\t")
    src = f"{root}/{cols[1]}"
    body = open(src, encoding="utf-8").read()
    find = cols[2].replace("\\n", "\n")
    out.append(dict(batch=batch, table=table, name=name, tsv_lines=[i + 1 for i in hits], identical_to_diff=len(same) == len(hits) and len(same) > 0,
                    source=cols[1], find_hits=body.count(find), args=cols[4], tier="checker-tier" if "-p singlefs-checker-tier" in cols[4] else "harness", test=cols[5]))
for k, r in enumerate(out):
    print(k, r["batch"], r["table"][10:14], r["name"][:34], r["tsv_lines"], "same" if r["identical_to_diff"] else "DIFF", r["source"].split("/")[1], "hits=%d" % r["find_hits"], r["tier"])
json.dump(out, open("rows.json", "w"), ensure_ascii=False, indent=0)
