"""E8 汇总：同一段历史里回退过、又有根被抛弃。按臂 × 次序求和 S4u / S4c / S4bu / S4bc、checker 违例；隔离槽「会话里 → 同一刻崩了再挂那次挂载读数」的变化计数。"""
import re, sys, collections
agg = collections.defaultdict(lambda: [0] * 6)
div = collections.defaultdict(collections.Counter)
nosession = 0
for line in open(sys.argv[1], encoding="utf-8"):
    if not line.startswith("E8 "):
        continue
    if "抛弃之后没有会话" in line:
        nosession += 1
        print("没有会话：", line.strip()[:200])
        continue
    d = dict(re.findall(r"(\w+)=([^\s\[]+)", line.split(" first=")[0]))
    a = agg[(d["arm"], d["order"])]
    a[0] += 1
    for i, f in enumerate(["S4u", "S4c", "S4bu", "S4bc"]):
        a[i + 1] += int(d[f])
    a[5] += int(line.rsplit("viol=", 1)[1])
    body = line.split("divergences={", 1)[1].split("}", 1)[0]
    for s, m, n in re.findall(r'"iso\(session (\d+) -> mount \[(\d+), \d+\]\)": (\d+)', body):
        kind = "相同" if s == m else ("挂载多" if int(m) > int(s) else "挂载少")
        div[d["arm"]][kind] += int(n)
print("臂 次序 | 格数 | S4u | S4c | S4bu | S4bc | 违例")
for k in sorted(agg):
    print(" ".join(k), "|", " | ".join(str(x) for x in agg[k]))
print("隔离槽：会话里（发布路径读数）与同一刻崩了再挂那一次挂载路径读数相比（放行的每一步各计一次）")
for arm, c in sorted(div.items()):
    print(arm, dict(c))
