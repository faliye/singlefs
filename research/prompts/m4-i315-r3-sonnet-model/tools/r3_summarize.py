"""把 r3_probe 的日志压成每个臂 × 三种归类器的计数（TALLY 行）与臂间有差别的点（DIFF 行）。用法：python3 -I r3_summarize.py <日志...>
记号：L 说谎的设备留下的、N 新发现、K 已知红、- 不红；一个点上有几条失败按字母排序连起来（例 LN）。"""
import re, sys
from collections import Counter, defaultdict

def letter(cell):
    cell = cell.strip()
    if cell == "-":
        return "-"
    return "".join(sorted({("N" if p.startswith("N") else p[0]) for p in cell.split("+")}))

arms_order = []
compact = "--compact" in sys.argv
sys.argv = [a for a in sys.argv if a != "--compact"]
tally = defaultdict(lambda: defaultdict(Counter))
partly = defaultdict(lambda: defaultdict(Counter))
points = {}
per_point = defaultdict(dict)
for path in sys.argv[1:]:
    for line in open(path, encoding="utf-8"):
        line = line.rstrip("\n")
        m = re.match(r"POINT seed=(\d+) point=(\d+) (.*)", line)
        if m:
            points[(m[1], m[2])] = m[3]
            continue
        m = re.match(r"CLASS seed=(\d+) point=(\d+) arm=(\S+) name=(.*?) \| j2n=(.*?) \| j2s=(.*?) \| j2t=(.*?) \| after_space=(\S+) ", line)
        if not m:
            continue
        seed, point, arm, name, j2n, j2s, j2t, after = m.groups()
        if arm not in arms_order:
            arms_order.append(arm)
        cells = {"name": letter(name), "j2n": letter(j2n), "j2s": letter(j2s), "j2t": letter(j2t)}
        for k, v in cells.items():
            tally[arm][k][v] += 1
        for k, raw in (("name", name), ("j2n", j2n), ("j2s", j2s), ("j2t", j2t)):
            for part in raw.strip().split("+"):
                partly[arm][k][("N" if part.startswith("N") else part[0]) if part else "-"] += 1
        per_point[(seed, point)][arm] = (cells, after)
print("points", len(per_point))
if compact:
    for arm in arms_order:
        print(f"{arm:9s}", " ".join(f"{k}:L{partly[arm][k]['L']}/N{partly[arm][k]['N']}/-{partly[arm][k]['-']}" for k in ("name", "j2n", "j2s", "j2t")))
    sys.exit(0)
for arm in arms_order:
    print("TALLY", f"{arm:9s}", " ".join(f"{k}={dict(sorted(partly[arm][k].items()))}" for k in ("name", "j2n", "j2s", "j2t")))
    print("POINTS", f"{arm:8s}", " ".join(f"{k}={dict(sorted(tally[arm][k].items()))}" for k in ("name", "j2n", "j2s", "j2t")))
shown = 0
for key, arms in sorted(per_point.items()):
    sig = {tuple(sorted(c.items())) for c, _ in arms.values()}
    if len(sig) > 1 or any(("N" in v) for c, _ in arms.values() for v in c.values()):
        if shown < int(__import__("os").environ.get("R3_SHOW", "60")):
            p = points.get(key, "")
            ex = re.search(r"swallowed=(\S+) reused=(\w+)", p)
            print("DIFF", key[0][-4:], key[1], (ex[1] + " reused=" + ex[2]) if ex else "", " ".join(f"{a}:{c['name']}/{c['j2n']}/{c['j2s']}/{c['j2t']}" for a, (c, _) in arms.items()))
        shown += 1
