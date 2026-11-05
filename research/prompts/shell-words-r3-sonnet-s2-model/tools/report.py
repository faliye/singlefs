#!/usr/bin/env python3
"""汇总 out/*.jsonl：每族样本数与判错数（供 rerun 与随报告带的 summary.txt 逐行比），并把判错的逐条写 mismatches.tsv。"""
import glob, json, os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from classify import classify
out_dir, summary = sys.argv[1], sys.argv[2]
rows = {}
for p in sorted(glob.glob(os.path.join(out_dir, "*.jsonl"))):
    for line in open(p):
        r = json.loads(line); rows[r["id"]] = r
fam = {}
bad = []
for r in rows.values():
    if "home-cargo" in r["id"]:
        continue
    key = r["id"].split("-")[0]
    f = fam.setdefault(key, [0, 0, 0])
    f[0] += 1
    if r["want_reject"] != (r["hook"] == "拒"):
        f[1 if r["want_reject"] else 2] += 1
        c = classify(r); bad.append((r, c))
with open(summary, "w") as fo:
    for k in sorted(fam):
        fo.write(f"{k}\t样本{fam[k][0]}\t漏放{fam[k][1]}\t误拒{fam[k][2]}\n")
    cat = {}
    for r, c in bad:
        cat[c[0]] = cat.get(c[0], 0) + 1
    for k in sorted(cat):
        fo.write(f"类{k}\t{cat[k]}\n")
    fo.write(f"总计\t样本{len(rows)}\t判错{len(bad)}\n")
with open(os.path.join(os.path.dirname(summary), "mismatches.tsv"), "w") as fo:
    for r, c in sorted(bad, key=lambda x: x[0]["id"]):
        fo.write("\t".join([r["id"], r["role"], "bg" if r["bg"] else "fg", r["cmd"].replace("\n", "\\n"), "拒" if r["want_reject"] else "放", r["hook"],
                            c[0], c[1], c[2].split("（")[0], ";".join(w[0] for w in r["want"])]) + "\n")
