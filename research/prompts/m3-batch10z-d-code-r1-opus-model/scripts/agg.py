#!/usr/bin/env python3
"""把 runs/<前缀>*.log 里的 OPUS10ZD 行按 (arm, k, rkind, fault) 汇总：段数、挂载被拒、选到哪一版、命中（终态 I-7.4 / I-4.8 红）。"""
import collections, glob, re, sys
prefix = sys.argv[1]
rows = collections.defaultdict(lambda: collections.Counter())
hits = collections.defaultdict(list)
for path in sorted(glob.glob(f"{prefix}*.log")):
    for line in open(path, encoding="utf-8"):
        at = line.find("OPUS10ZD ")
        if at < 0:
            continue
        line = line[at:]
        line = re.sub(r"\((\d+), (\d+)\)", r"(\1,\2)", line)
        f = dict(re.findall(r"(\w+)=(\S+)", line))
        key = (f["arm"], f["k"], f["rkind"], f["fault"])
        c = rows[key]
        c["histories"] += 1
        if "mount=Err" in line:
            c["mount_refused"] += 1
        else:
            c["selected=" + f.get("selected", "?")] += 1
            c["borrowed=" + f.get("borrowed_at_mount", "?")] += 1
        if " Err:" in line or "=Err" in line.split("selected=")[-1]:
            c["some_step_refused"] += 1
        if f.get("first_red_step", "-") != "-":
            c["red_at_some_step"] += 1
        if f["hit"] == "true":
            c["hit"] += 1
            hits[key].append(f["actions"])
for key in sorted(rows):
    c = rows[key]
    extra = " ".join(f"{k}:{v}" for k, v in sorted(c.items()) if k not in ("histories", "hit", "mount_refused", "some_step_refused"))
    print(f"arm={key[0]} k={key[1]} rkind={key[2]} fault={key[3]} histories={c['histories']} mount_refused={c['mount_refused']} step_refused={c['some_step_refused']} hit={c['hit']} {extra}")
if len(sys.argv) > 2:
    for key in sorted(hits):
        print("HITS", *key, " ".join(sorted(hits[key])))
