"""E10 汇总：只重挂（不写）。每条臂、分支、先长几个单元：第一次挂载被拒是第几次（None = 400 次内没被拒），按盘宽列；被拒之后吸收态的格数。"""
import re, sys, collections
rows = collections.defaultdict(dict)
sizes = []
absorbing = collections.Counter()
refused = collections.Counter()
for line in open(sys.argv[1], encoding="utf-8"):
    if not line.startswith("E10 "):
        continue
    d = dict(re.findall(r"(\w+)=([^\s\[]+)", line))
    s = int(d["slots"])
    if s not in sizes:
        sizes.append(s)
    key = (d["arm"], d["branch"], d["grow"].split(":")[0])
    rows[key][s] = d["first_refused_remount"].replace("Some(", "").replace(")", "")
    if d["first_refused_remount"] != "None":
        refused[d["arm"]] += 1
        absorbing[d["arm"]] += "absorbing=true" in line
print("臂 分支 先长单元 | " + " | ".join(str(s) for s in sizes))
for k in sorted(rows):
    print(" ".join(k), "|", " | ".join(rows[k].get(s, "-") for s in sizes))
print("被拒的格 / 其中吸收态：", {a: f"{refused[a]}/{absorbing[a]}" for a in refused})
