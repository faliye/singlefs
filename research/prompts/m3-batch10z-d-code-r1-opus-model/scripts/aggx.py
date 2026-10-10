#!/usr/bin/env python3
"""runs/<前缀>*.log 里的 OPUS10ZDX 行按 (k, i3, fault) 汇总：段数、挂载被拒、某一步之后红、终态红、某一步被拒。"""
import collections, glob, re, sys
rows = collections.defaultdict(collections.Counter)
for path in sorted(glob.glob(f"{sys.argv[1]}*.log")):
    for line in open(path, encoding="utf-8"):
        at = line.find("OPUS10ZDX ")
        if at < 0:
            continue
        line = line[at:]
        f = dict(re.findall(r"(\w+)=(\S+)", line))
        c = rows[(f["k"], f["i3"], f["fault"])]
        c["histories"] += 1
        if "mount=Err" in line:
            c["mount_refused"] += 1
            continue
        c["recorded_at_mount=" + f["recorded_at_mount"]] += 1
        if f["first_red"] != "-":
            c["red_at_some_step"] += 1
        if f["final"] != "-":
            c["red_final"] += 1
        if "=Err" in line:
            c["some_step_refused"] += 1
for key in sorted(rows):
    c = rows[key]
    print(f"k={key[0]} i3={key[1]} fault={key[2]} " + " ".join(f"{k}={v}" for k, v in sorted(c.items())))
