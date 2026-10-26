"""把 attack_probe3 的日志压成一行一个（注入点, 判法），再按（判法, 归类器）数 K / L / N / -。用法：python3 -I summarize3.py <日志>
记号同第二轮攻方：K 已知红、L 说谎的设备留下的、N 新发现、- 不红；方括号里是除 I-2.1、I-4.8、I-7.4 之外点名的不变量。
归类器四列：today（按名比）、jia2n（甲二按名比）、jia2s（甲二按（盘, 槽）比、对照历史末尾插回去那张盘）、jia2k（甲二按（盘, 槽）比、
对照第一张红的每步镜像那一刻插回去的盘）。另数每个判法下 HITS 行三种盲区的点数，与 DONE 行（跑完的段数）。"""
import re, sys
from collections import Counter
lines = open(sys.argv[1], encoding="utf-8").read().splitlines()
CLASSIFIERS = ("today", "jia2n", "jia2s", "jia2k")
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
tallies, points, hits = {}, {}, Counter()
done = 0
for line in lines:
    if line.startswith("DONE "):
        done += 1
        continue
    m = re.match(r"POINT seed=(\d+) point=(\d+) (.*)", line)
    if m:
        points[(m[1], m[2])] = m[3]
        continue
    m = re.match(r"HITS seed=(\d+) point=(\d+) after=(\S+) genEq_otherDiff=(\d+) genNe_otherSame=(\d+) genEq_otherSame=(\d+)", line)
    if m:
        for name, value in (("genEq_otherDiff", m[4]), ("genNe_otherSame", m[5]), ("genEq_otherSame", m[6])):
            if int(value):
                hits[name] += 1
        continue
    m = re.match(r"LOCS seed=(\d+) point=(\d+) loc_self_pass=(\d+) equal_birth=(\d+)", line)
    if m:
        if int(m[3]):
            hits["loc_self_pass_points"] += 1
        if int(m[4]):
            hits["loc_equal_birth_points"] += 1
        continue
    m = re.match(r"CLASS seed=(\d+) point=(\d+) arm=(\w+) (.*?) today=(.*?) \| jia2n=(.*?) \| jia2s=(.*?) \| jia2k=(.*)$", line)
    if not m:
        continue
    seed, point, arm, rest, *cells = m.groups()
    p = points.get((seed, point), "")
    reused = "reused=true" if "reused=true" in p else ("reused=false" if "reused=false" in p else "reused=-")
    swallowed = len(re.findall(r"dev\d:slot", p))
    flagged = "flag" if re.search(r"flagged=\[\(", p) else "noflag"
    leaks = "leak" if re.search(r"leaks=\[\(", p) else "noleak"
    after = re.search(r"after_space=(\S+)", rest)
    replug = re.search(r"replug_space=(\S+)", rest)
    print(seed[-4:], point, f"sw={swallowed}", reused, flagged, leaks, arm,
          "after:" + (after[1] if after else "?"), "replug:" + (replug[1] if replug else "-"),
          " ".join(f"{c}={short(v)}" for c, v in zip(CLASSIFIERS, cells)))
    for classifier, cell in zip(CLASSIFIERS, cells):
        for part in short(cell).split("+"):
            tallies.setdefault((arm, classifier), Counter())[part[0]] += 1
for (arm, classifier), counter in sorted(tallies.items()):
    print("TALLY", arm, classifier, dict(sorted(counter.items())))
print("BLIND", dict(sorted(hits.items())))
print("DONE", done)
