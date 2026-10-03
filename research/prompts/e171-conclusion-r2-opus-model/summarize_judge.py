"""e171-conclusion-r2 攻方腿：把判官探针的逐态输出与 O3 的逐态判定（o3-verdicts.tsv）并起来，按列计数。
用法：python3 summarize_judge.py <标签> <o3-verdicts.tsv> <判官探针输出.tsv>
每一种「O3 严格 / 实现恢复 / 读回 / O1-崩溃判官 / RefFS / O2 / 第一次可写挂载 / 第一次之后盘上变没变 / 第二次可写挂载」组合一行，带态数。
"""
import re
import sys
from collections import Counter

label, o3_path, probe_path = sys.argv[1:4]
o3 = {}
for line in open(o3_path):
    c = line.rstrip("\n").split("\t")
    o3[c[0]] = c[2].split("=")[1]
rows = Counter()
for line in open(probe_path):
    c = line.rstrip("\n").split("\t")
    f = dict(x.split("=", 1) for x in c[2:] if "=" in x)
    reffs = re.sub(r"\(.*", "", f["reffs"])
    o1 = f["o1_crash_judge"].split(":")[0]
    rows[(o3[c[1]], f["recover"], f["read_back"][:9], o1, reffs, f["o2_red"], f.get("mount1", "-"),
          f.get("unchanged_after_mount1", "-"), f.get("mount2", "-"), f["digest_ok"])] += 1
for k, n in sorted(rows.items(), key=lambda kv: (kv[0][0] != "not_in", kv[0])):
    print(f"name=judge_summary label={label} count={n} o3_strict={k[0]} recover={k[1]} read_back={k[2]} o1_crash_judge={k[3]} "
          f"reffs={k[4]} o2_red={k[5]} mount1={k[6]} unchanged_after_mount1={k[7]} mount2={k[8]} digest_ok={k[9]}")
