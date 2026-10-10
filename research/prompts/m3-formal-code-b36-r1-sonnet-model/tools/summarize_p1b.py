#!/usr/bin/env python3
"""汇总 P1b 网格（t-p1bgrid.log）：checker 认 B 为下一次挂载先施加的那一版 与 core 只读恢复至少施加 1 条（B）的交叉表，
再按 I-4.1 理由（起点含介质损坏 / 一次发布都不施加 / 走读）与 core vfail 分组，列出不一致行。用法 summarize_p1b.py <log>"""
import collections, re, sys
rows = []
for l in open(sys.argv[1], encoding="utf-8"):
    m = re.match(r'PROBE-P1B copiesB=(\d+) copiesC=(\d+) namedB=(\d) namedC=(\d) checker_takes_B=(\w+) i54=(.*?) i41=(.*?) rec=prefix=(\d+),vfail=(\d+),eff=(\([0-9,]+\)|None)(?:,FAILED.*)?$', l.strip())
    if m: rows.append(m.groups())
print("行数", len(rows))
c = collections.Counter(); odd = []
for cb, cc, nb, nc, taken, i54, i41, prefix, vfail, eff in rows:
    core_b = int(prefix) >= 1
    c[(("checker 认 B" if taken == "true" else "checker 不认 B"), ("core 施加 ≥1 条" if core_b else "core 不施加"))] += 1
    if (taken == "true") != core_b: odd.append((cb, cc, nb, nc, taken, prefix, vfail, eff))
print(dict(sorted(c.items())))
print("不一致行:", odd)
g = collections.Counter()
for cb, cc, nb, nc, taken, i54, i41, prefix, vfail, eff in rows:
    g[(i41[:8], f"vfail={vfail}", f"prefix={prefix}", "认B" if taken == "true" else "不认B")] += 1
for k, v in sorted(g.items()): print(" ", k, v)
