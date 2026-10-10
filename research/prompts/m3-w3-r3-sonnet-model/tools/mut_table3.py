#!/usr/bin/env python3
# 用法：mut_table3.py <outdir> <候选,候选…>  把 mutation 组 0..5（老三种 A / B / C）与 mutation2 组 0..9（新五种 H / Z / R / D / Mh）排成一张表：
# P 臂：历史被改过的镜像里不红的张数 / 被改过的张数；V1 臂：V1 臂「现有检查都不红」的事件里候选不红的个数（分母在表头）。
import sys
d = sys.argv[1]; want = sys.argv[2].split(",")
def load(path):
    head = None; rows = {}
    for line in open(path, encoding="utf-8"):
        if line.startswith("W3MODEL"): head = dict(kv.split("=", 1) for kv in line.split()[1:])
        elif line.startswith("W3COUNT"):
            r = dict(kv.split("=", 1) for kv in line.split()[1:]); rows[r["candidate"]] = r
    return head, rows
cols = []  # (名字, 臂, 文件)
for i, n in enumerate(["A", "B", "C"]): cols.append((f"P+{n}", "P", f"{d}/model-mutation-{i}.out"))
for i, n in enumerate(["A", "B", "C"]): cols.append((f"V1+{n}", "V1", f"{d}/model-mutation-{3+i}.out"))
for i, n in enumerate(["H", "Z", "R", "D", "Mh"]): cols.append((f"P+{n}", "P", f"{d}/model-mutation2-{i}.out"))
for i, n in enumerate(["H", "Z", "R", "D", "Mh"]): cols.append((f"V1+{n}", "V1", f"{d}/model-mutation2-{5+i}.out"))
data = [(n, arm, *load(p)) for n, arm, p in cols]
print("表头（V1 臂列的分母 = 现有检查都不红的事件数）:", " ".join(f"{n}:{h['mutation']}/{h['events_where_existing_checks_silent']}" for n, arm, h, _ in data if arm == "V1"))
print("候选".ljust(24) + "".join(n.rjust(20) if arm == "P" else n.rjust(10) for n, arm, _, _ in data))
for c in want:
    s = c.ljust(24)
    for n, arm, h, r in data:
        d_ = r[c]
        s += (f"{d_['mutated_history_undetected']}/{d_['mutated_images']}".rjust(20) if arm == "P" else d_["event_missed_where_existing_checks_silent"].rjust(10))
    print(s)
