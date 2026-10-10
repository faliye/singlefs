#!/usr/bin/env python3
"""B1 两份 sweep 的汇总（m3-formal-code-b23-r1 opus）。用法：summarize_b1.py <out 目录>
第一段：b1-neighbourhood.txt 按 (覆盖写, 单元, 批) 分组，GREEN-KILLS-BOTH（分得开）与 GREEN-KILLS-SOME-ONLY（照绿而建 0 个不红）的槽区间、136 槽的类。
第二段：b1-slots.txt 与 b1-neighbourhood.txt 合起来，去掉造盘失败的，按类、建 0 个变异红不红、建 0 个做没做成三列计数。"""
import collections, re, sys
out = sys.argv[1]
groups = collections.defaultdict(list)
for line in open(f"{out}/b1-neighbourhood.txt", encoding="utf-8"):
    m = re.match(r"b1 slots=(\d+) ow=(\d+) units=(\d+) batches=(\d+) (?:class=(\S+))?", line)
    slots, ow, units, batches, cls = m.groups()
    groups[(int(ow), int(units), int(batches))].append((int(slots), cls or "build-failed"))
for (ow, units, batches), rows in sorted(groups.items()):
    rows.sort()
    band = [s for s, c in rows if c == "GREEN-KILLS-BOTH"]
    silent = [s for s, c in rows if c == "GREEN-KILLS-SOME-ONLY"]
    span = lambda xs: f"{min(xs)}..{max(xs)}" if xs else "-"
    print(f"ow={ow} units={units} batches={batches} band={span(band)} silent={span(silent)} at136={dict(rows).get(136)}")
counts = collections.Counter()
for name in ("b1-slots.txt", "b1-neighbourhood.txt"):
    for line in open(f"{out}/{name}", encoding="utf-8"):
        if "build-failed" in line:
            continue
        f = line.split()
        counts[(f[5], "zero-killed" if "red" in f[8] else "zero-survives", "create0-published" if f[9] == "create0=Published" else "create0-refused")] += 1
for key in sorted(counts):
    print(f"{counts[key]:7d} {' '.join(key)}")
