#!/usr/bin/env python3
"""读 out/*.jsonl，列期望与钩子判定对不上的：误拒（期望放、钩子拒）与漏放（期望拒、钩子放）。"""
import glob, json, sys
rows = []
seen = {}
for p in sorted(glob.glob(sys.argv[1])):
    for l in open(p):
        r = json.loads(l); seen[r["id"]] = r
rows = list(seen.values())
bad = [r for r in rows if r["want_reject"] != (r["hook"] == "拒")]
print(f"总数 {len(rows)}，对不上 {len(bad)}（误拒 {sum(1 for r in bad if not r['want_reject'])}，漏放 {sum(1 for r in bad if r['want_reject'])}）")
for r in bad:
    kind = "漏放" if r["want_reject"] else "误拒"
    first = ""
    for k, v in r["hook_detail"].items():
        if v["rc"] == 2:
            first = k.split("-")[0] + ":" + v["first"][:70]
    print(kind, r["id"], "|", r["cmd"].replace("\n", "\\n")[:150], "|", [w[0] for w in r["want"]], "|", first)
