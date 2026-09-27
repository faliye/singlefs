"""E1 汇总：每条臂每个盘宽，8 条脚本求和：放行的用户发布 / 被拒的用户发布 / 准入推的空发布 / S4u / S4c / S4bu / S4bc；另报被拒的成员分布。"""
import re, sys, collections
path = sys.argv[1]
cells = collections.defaultdict(lambda: [0] * 7)
refusals = collections.defaultdict(collections.Counter)
sizes, arms = [], []
for line in open(path, encoding="utf-8"):
    if not line.startswith("E1 "):
        continue
    head = line.split(" first_")[0]
    d = dict(re.findall(r"(\w+)=(\S+)", head))
    slots, arm = int(d["slots"]), d["arm"]
    if slots not in sizes: sizes.append(slots)
    if arm not in arms: arms.append(arm)
    c = cells[(arm, slots)]
    for i, f in enumerate(["user_ok", "user_refused", "empties_in_admission", "S4u", "S4c", "S4bu", "S4bc"]):
        c[i] += int(d[f])
    outcomes = line.split("outcomes=[", 1)[1].rsplit("]", 1)[0]
    for part in outcomes.split(","):
        m = re.match(r"(.*?)(?:x(\d+))?$", part)
        name, n = m.group(1), int(m.group(2) or 1)
        if name != "ok":
            refusals[arm][name.split(";")[0].split("(")[0]] += n
print("格 = 放行 / 被拒 / 准入内空发布 / S4u / S4c / S4bu / S4bc（8 条脚本求和；u = 正常卸载再挂，c = 崩了再挂）")
print("臂 | " + " | ".join(str(s) for s in sizes) + " | S4u 合计 | S4c 合计 | S4bu 合计 | S4bc 合计")
for arm in arms:
    row = [cells[(arm, s)] for s in sizes]
    tot = [sum(r[i] for r in row) for i in range(7)]
    print(f"{arm} | " + " | ".join("/".join(str(x) for x in r) for r in row) + f" | {tot[3]} | {tot[4]} | {tot[5]} | {tot[6]}")
print()
print("被拒的用户发布按成员（全部盘宽与脚本求和）")
for arm in arms:
    print(arm, dict(refusals[arm].most_common()))
