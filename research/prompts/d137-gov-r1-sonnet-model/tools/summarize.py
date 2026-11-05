#!/usr/bin/env python3
"""把各步输出的 tsv 缩成稳定的摘要行（不含路径、进程号、时刻），复跑时与 out/expected-summary.txt 逐行比。参数：输出目录。"""
import os, sys, collections
d = sys.argv[1]
def rows(name):
    p = os.path.join(d, name)
    return [l.rstrip("\n").split("\t") for l in open(p, encoding="utf-8")] if os.path.exists(p) else []
out = []
r = rows("heavy.tsv")
c = collections.Counter(x[5] for x in r)
out.append("heavy 行=%d 符合=%d 不符=%d 漏拒(主agent带前缀类)=%d" % (len(r), c["符合"], c["不符"], c["漏拒"]))
out.append("heavy 不符: " + ",".join(sorted(x[0] for x in r if x[5] == "不符")))
out.append("heavy 漏拒: " + ",".join(sorted(x[0] for x in r if x[5] == "漏拒")))
r = rows("fuzz.tsv")
c = collections.Counter(x[3] for x in r)
out.append("fuzz 行=%d 符合=%d 漏拒=%d 放过且没起=%d 拒了但没起=%d" % (len(r), c["符合"], c["漏拒"], c["放过且没起"], c["拒了但没起"]))
out.append("fuzz 漏拒: " + ",".join(sorted(x[0] for x in r if x[3] == "漏拒")))
r = rows("opts.tsv")
c = collections.Counter(x[2] for x in r)
out.append("opts 行=%d 符合=%d 漏拒=%d 误拒=%d" % (len(r), c["符合"], c["漏拒"], c["误拒"]))
out.append("opts 漏拒: " + " | ".join(sorted(x[4] for x in r if x[2] == "漏拒")))
r = rows("hits.tsv")
for x in r:
    out.append("hits " + x[0] + " " + x[1].replace("检出", "检出") + " " + x[2] + " " + x[3])
for name, key in (("battery.tsv", "battery"), ("g2.tsv", "g2")):
    r = [x for x in rows(name) if len(x) >= 5 and x[0] != "合计"]
    g = collections.OrderedDict()
    for x in r:
        a = g.setdefault(x[0], [0, 0]); a[0] += 1; a[1] += (x[1] == x[2])
    out.append(key + " " + " ".join("%s=%d/%d" % (k, v[1], v[0]) for k, v in sorted(g.items())))
    bad = [x for x in r if x[1] != x[2]]
    out.append(key + " 不符: " + " | ".join(sorted(x[-1] if key == "battery" else x[-1] for x in bad))[:600])
r = rows("garbage.tsv")
out.append("garbage 行=%d rc全0=%s 每行检出+1=%s" % (len(r), all("rc=0" in x[2] for x in r), all(x[3] == "检出+1" for x in r)))
rf = os.path.join(d, "refs.txt")
if os.path.exists(rf): out.append(open(rf, encoding="utf-8").read().strip().splitlines()[-1])
r = [x for x in rows("overblock.tsv") if x[0] == "overblock"]
out.append("overblock 行=%d 符合=%d 误拒=%s" % (len(r), sum(1 for x in r if x[1] == x[2]), ",".join(x[3] for x in r if x[1] == "0" and x[2] == "2")))
r = [x for x in r if x[1] == "2" and x[2] == "2"]
out.append("overblock 该拒且拒=%s" % ",".join(x[3] for x in r))
print("\n".join(out))
