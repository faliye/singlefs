"""e171-conclusion-r2 攻方腿：判官探针输出里「第一次可写挂载的结局 × 崩溃注入第二截那一道拒绝判定 × 盘上变没变 × 第二次结局」按标签计数。
用法：python3 summarize_refusals.py <核的名字> <判官探针输出>...
"""
import sys
from collections import Counter

prefix = sys.argv[1]
counts = Counter()
for path in sys.argv[2:]:
    for line in open(path):
        c = line.rstrip("\n").split("\t")
        f = dict(x.split("=", 1) for x in c[2:] if "=" in x)
        counts[(c[0], f["mount1"], f.get("refusal_judged_by_crash_injection_second_cut", "-"), f["unchanged_after_mount1"], f["mount2"])] += 1
for (label, m1, verdict, unchanged, m2), n in sorted(counts.items()):
    print(f"name=refusal_judge core={prefix} label={label} mount1={m1} second_cut_verdict={verdict} unchanged_after_mount1={unchanged} mount2={m2} count={n}")
