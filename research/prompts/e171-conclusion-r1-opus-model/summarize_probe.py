"""e171-conclusion-r1 攻方腿：把崩溃之后可写挂载探针的输出汇总成报告里引的几行。
用法：python3 summarize_probe.py <ackdump/o3-verdicts.tsv> <ack 探针输出> <挑态探针输出>
"""
import re
import sys
from collections import Counter


def mount_kind(field: str) -> str:
    if field.startswith("mount=ok"):
        return "ok"
    m = re.search(r"comparison:_([A-Za-z]+)", field)
    if m:
        return "refused:" + m.group(1)
    return field.split("(")[0][:60]


o3 = {}
for line in open(sys.argv[1]):
    c = line.rstrip("\n").split("\t")
    o3[c[0]] = (c[2].split("=")[1], c[7].split("=")[1])
ack = Counter()
for line in open(sys.argv[2]):
    c = line.rstrip("\n").split("\t")
    strict, persisted = o3[c[1]]
    key = (strict, c[6], c[7], "persisted_txg_inst=" + persisted, mount_kind(c[8]), c[5])
    ack[key] += 1
for key, n in sorted(ack.items()):
    print("name=ack_probe count=%d o3_strict=%s %s %s %s mount=%s %s" % ((n,) + key))
rows = {}
for line in open(sys.argv[3]):
    c = line.rstrip("\n").split("\t")
    rows[(c[0], c[1])] = (c[5], c[6], c[7], mount_kind(c[8]))
per_label = Counter((label, v[3], v[2] == "o2_red=-", v[0]) for (label, _), v in rows.items())
for (label, mount, o2_green, digest), n in sorted(per_label.items()):
    print(f"name=mount_after_crash label={label} mount={mount} o2_green={o2_green} {digest} count={n}")
for stream in ("s1", "s2"):
    pairs = Counter()
    for (label, state), v in rows.items():
        if label != f"r269{stream}":
            continue
        base = rows.get((f"b269{stream}", state))
        pairs[(base[3] if base else "baseline_missing", v[3])] += 1
    for (b, r), n in sorted(pairs.items()):
        print(f"name=row269_vs_baseline stream={stream} baseline={b} row269={r} count={n}")
