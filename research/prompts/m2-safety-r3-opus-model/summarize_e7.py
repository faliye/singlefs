"""E7 汇总：行数逼近 369 的整数倍。每条臂每个 rows0，各盘宽的 S4c 求和（S4u、S4bu、S4bc 也报），与推行数时就被拒的格。"""
import re, sys, collections
agg = collections.defaultdict(lambda: [0, 0, 0, 0])
bad = []
for line in open(sys.argv[1], encoding="utf-8"):
    if not line.startswith("E7 "):
        continue
    if "rows0=" in line and "admitted=" in line:
        d = dict(re.findall(r"(\w+)=([^\s\[]+)", line.split(" first_")[0]))
        a = agg[(d["arm"], int(d["rows0"]))]
        for i, f in enumerate(["S4u", "S4c", "S4bu", "S4bc"]):
            a[i] += int(d[f])
    else:
        bad.append(line.strip()[:160])
print("臂 rows0 | S4u | S4c | S4bu | S4bc（各盘宽求和）")
for k in sorted(agg):
    print(k[0], k[1], "|", " | ".join(str(x) for x in agg[k]))
print("推行数时就被拒 / 没推到的格：")
for b in bad:
    print("  ", b)
