"""E3 汇总：删了再写。按臂、分支、填充步汇总：做成的格数 / 总格数，做成之前用户发布数的分布，准入里推的空发布数的分布，没做成的格的结局。"""
import re, sys, collections
agg = collections.defaultdict(lambda: {"n": 0, "ok": 0, "users": collections.Counter(), "empties": collections.Counter(), "fail": collections.Counter(), "dead": 0, "skip": 0})
for line in open(sys.argv[1], encoding="utf-8"):
    if not line.startswith("E3 "):
        continue
    d = dict(re.findall(r"(\w+)=([^\s\[]+)", line))
    key = (d["arm"], d["branch"], d["filler"])
    a = agg[key]
    if "N=0" in line:
        a["skip"] += 1
        continue
    a["n"] += 1
    if d["success_after"].startswith("Some"):
        a["ok"] += 1
        a["users"][int(d["user_publishes_before_success"])] += 1
        a["empties"][int(d["empties_in_admission_after_delete"])] += 1
    else:
        steps = line.split("steps=[", 1)[1].split("]", 1)[0]
        last = steps.split(" ")[-1].split("/")[-1]
        a["fail"][re.sub(r"\(.*", "", last)] += 1
    if d.get("dead") == "true":
        a["dead"] += 1
print("臂 分支 填充 | 做成/格（N=0 跳过的格） | 做成之前的用户发布数分布 | 删之后准入里推的空发布数分布 | 没做成的最后结局")
for key in sorted(agg):
    a = agg[key]
    print(f"{' '.join(key)} | {a['ok']}/{a['n']} (跳过 {a['skip']}) | {dict(sorted(a['users'].items()))} | {dict(sorted(a['empties'].items()))} | {dict(a['fail'])}")
