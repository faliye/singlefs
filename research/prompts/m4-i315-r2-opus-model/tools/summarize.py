"""把探针日志压成一行一个（注入点, 判法）、再按（判法, 归类器）数 N / L / K / -。用法：python3 -I summarize.py <日志>
每一格的记号：K 已知红、L 说谎的设备留下的、N 新发现、- 不红；方括号里是除 I-2.1、I-4.8、I-7.4 之外点名的不变量。"""
import re, sys
from collections import Counter
lines = open(sys.argv[1], encoding="utf-8").read().splitlines()
def short(cell):
    cell = cell.strip()
    if not cell:
        return "-"
    out = []
    for part in cell.split(" + "):
        kind = "N" if part.startswith("NEW") else ("L" if part.startswith("lying") else "K")
        inv = sorted(set(re.findall(r'"(I-[0-9.]+)"', part)))
        extra = [i for i in inv if i not in ("I-2.1", "I-4.8", "I-7.4")]
        out.append(kind + ("[" + ",".join(extra) + "]" if extra else ""))
    return "+".join(out)
tallies = {}
points = {}
for line in lines:
    m = re.match(r"POINT seed=(\d+) point=(\d+) (.*)", line)
    if m:
        points[(m[1], m[2])] = m[3]
        continue
    m = re.match(r"CLASS seed=(\d+) point=(\d+) arm=(\w+) (.*?) today=(.*?) \| jia_two=(.*)$", line)
    if not m:
        continue
    seed, point, arm, rest, today, jia = m.groups()
    p = points.get((seed, point), "")
    sw = re.search(r"swallowed=(\S+)(?: reused=(\w+))?", p)
    flagged = "flag" if re.search(r"flagged=\[\(", p) else "noflag"
    leaks = "leak" if re.search(r"leaks=\[\(", p) else "noleak"
    fault = re.search(r"fault=(\S+)", p)
    after = re.search(r"after_space=(\S+)", rest)
    replug = re.search(r"replug_space=(\S+)", rest)
    nl = re.search(r"noloc_after=(\S+)", rest)
    nr = re.search(r"noloc_replug=(\S+)", rest)
    print(seed[-3:], point, fault[1] if fault else "?", "reused=" + (sw[2] if sw and sw[2] else "-"), flagged, leaks, arm,
          "after:" + (after[1] if after else "?"), "replug:" + (replug[1] if replug else "-"),
          "noloc_after=" + (nl[1] if nl else "?"), "noloc_replug=" + (nr[1] if nr else "-"),
          f"today={short(today)} jia_two={short(jia)}")
    for classifier, cell in (("today", today), ("jia_two", jia)):
        for part in short(cell).split("+"):
            tallies.setdefault((arm, classifier), Counter())[part[0]] += 1
for (arm, classifier), counter in sorted(tallies.items()):
    print("TALLY", arm, classifier, dict(sorted(counter.items())))
