#!/usr/bin/env python3
# 用法：mut_table.py <outdir>  把 mutation-0..5 排成一张表：P+A/B/C 的 mut_undetected/mut_images，V1+A/B/C 的 event_missed_where_existing_checks_silent（总 existing 静默事件数在表头）
import sys
d = sys.argv[1]
names = ["P+A","P+B","P+C","V1+A","V1+B","V1+C"]
data = []
for i in range(6):
    rows = {}; head = None
    for line in open(f"{d}/model-mutation-{i}.out", encoding="utf-8"):
        if line.startswith("W3MODEL"): head = dict(kv.split("=",1) for kv in line.split()[1:])
        elif line.startswith("W3COUNT"):
            r = dict(kv.split("=",1) for kv in line.split()[1:]); rows[r["candidate"]] = r
    data.append((head, rows))
print("mutations:", " ".join(f"{n}:{h['mutation']}/{h['arm'][:2]} images={h['images']} events_existing_silent={h['events_where_existing_checks_silent']}" for n,(h,_) in zip(names,data)))
cands = list(data[0][1].keys())
print("candidate".ljust(9) + "".join(n.rjust(22) for n in names[:3]) + "".join(n.rjust(16) for n in names[3:]))
for c in cands:
    s = c.ljust(9)
    for k in range(3):
        r = data[k][1][c]; s += f"{r['mutated_history_undetected']}/{r['mutated_images']}".rjust(22)
    for k in range(3,6):
        r = data[k][1][c]; s += r['event_missed_where_existing_checks_silent'].rjust(16)
    print(s)
