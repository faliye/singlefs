#!/usr/bin/env python3
"""汇总 P1 子集矩阵：每个改法下，checker 红 I-5.4（= 认出了 B 那一版并在它的树上判到重叠）与 core 恢复施加 B（前缀 3 条或拒在两条记录罩住同一个槽）的交叉计数。
用法 summarize_p1.py <t-p1b.log> [<t-p1-fl-off.log>]"""
import collections, re, sys
def load(path):
    rows = {}
    for l in open(path, encoding="utf-8"):
        m = re.match(r'PROBE-P1 mask=(\d+) variant=(\S+) red=(\[.*?\]) i54=(\w) i310=(\w) core=(.*)$', l.strip())
        if m: rows[(m.group(1), m.group(2))] = m.groups()[2:]
    return rows
def core_applies(core):
    return core.startswith("ok") and "prefix=3" in core or "AllocationRecordOutsideThePoolGeometry" in core
for path in sys.argv[1:]:
    rows = load(path)
    print("==", path.split("/")[-1], "（%d 行 = 64 个子集 × 4 种改法）" % len(rows))
    for variant in ["clean", "overlap", "overlap+stale_first", "overlap+stale_last"]:
        c = collections.Counter()
        for (mask, v), (red, i54, i310, core) in rows.items():
            if v != variant: continue
            c[("checker 红 I-5.4" if "I-5.4" in red else "checker 不红", "core 施加 B" if core_applies(core) else "core 不施加 B")] += 1
        print(f"  {variant:22s}", dict(sorted(c.items())))
