"""把探针日志的 CLASS / POINT 行压成一行一个吞写点。用法：python3 -I summarize.py <日志>"""
import re, sys
from collections import Counter
text = open(sys.argv[1], encoding="utf-8").read().splitlines()
def short(cell):
    cell = cell.strip()
    if not cell:
        return "-"
    out = []
    for part in cell.split(" + "):
        kind = "N" if part.startswith("NEW") else ("L" if part.startswith("lying") else "K")
        inv = re.findall(r'"(I-[0-9.]+)"', part)
        extra = [i for i in inv if i not in ("I-2.1", "I-4.8", "I-7.4")]
        out.append(kind + ("[" + ",".join(extra) + "]" if extra else ""))
    return "+".join(out)
tallies = {k: Counter() for k in ("today", "jia_device", "jia_unit", "jia_two")}
points = {}
for line in text:
    m = re.match(r"POINT seed=(\d+) point=(\d+) (.*)", line)
    if m:
        points[(m[1], m[2])] = m[3]
    m = re.match(r"CLASS seed=(\d+) point=(\d+) today=(.*?) \| jia_device=(.*?) \| jia_unit=(.*?) \| jia_two=(.*)$", line)
    if m:
        cells = dict(zip(("today", "jia_device", "jia_unit", "jia_two"), m.groups()[2:]))
        p = points.get((m[1], m[2]), "")
        reused = re.search(r"reused=(\w+)", p)
        flagged = "flag" if re.search(r"flagged=\[\(", p) else "noflag"
        after = re.search(r"after_space=(\S+)", p)
        replug = re.search(r"replug_space=(\S+)", p)
        print(m[1][-3:], m[2], "reused=" + (reused[1] if reused else "?"), flagged,
              "after:" + (after[1] if after else "?"), "replug:" + (replug[1] if replug else "?"),
              " ".join(f"{k}={short(v)}" for k, v in cells.items()))
        for k, v in cells.items():
            for part in short(v).split("+"):
                tallies[k][part[0]] += 1
for k, c in tallies.items():
    print("TALLY", k, dict(c))
