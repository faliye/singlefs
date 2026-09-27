"""E2 汇总：按臂、格的种类（end / first-S4u / first-S4c / first-refused）数吸收态格数与放开扫之后仍逃不出的格数。"""
import re, sys, collections
agg = collections.defaultdict(lambda: [0, 0, 0])
for line in open(sys.argv[1], encoding="utf-8"):
    if not line.startswith("E2 "):
        continue
    d = dict(re.findall(r"(\w+)=(\S+)", line))
    cell = re.sub(r"@\d+", "", d["cell"])
    esc = int(d["escapes"].split("/")[0])
    a = agg[(d["arm"], cell)]
    a[0] += 1
    a[1] += d["absorbing"] == "true"
    a[2] += esc == 0
print("臂 格 | 格数 | 吸收态（重挂两支、写、删、回退全不成） | 放开扫长度 ≤ 3 的用户序列之后仍写不进")
for k in sorted(agg):
    print(" ".join(k), "|", " | ".join(str(x) for x in agg[k]))
