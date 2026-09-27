"""E4 汇总：按主线臂，同一刻 P1、P3 各做一次的判定四格求和；P3 主线上多扣（槽）的分布，根环转过之前 / 之后。"""
import re, sys, collections, ast
agg = collections.defaultdict(lambda: [0, 0, 0, 0])
hb = collections.defaultdict(collections.Counter)
ha = collections.defaultdict(collections.Counter)
firsts = []
for line in open(sys.argv[1], encoding="utf-8"):
    if not line.startswith("E4 "):
        continue
    d = dict(re.findall(r"(\w+)=([^\s\[{]+)", line))
    a = agg[d["main"]]
    for i, f in enumerate(["both_ok", "both_refused", "P3_only_refused", "P1_only_refused"]):
        a[i] += int(d[f])
    for tag, h in (("overcharge_before_ring_wraps=", hb), ("overcharge_after_ring_wraps=", ha)):
        body = line.split(tag, 1)[1].split("}", 1)[0] + "}"
        for k, v in ast.literal_eval(body).items():
            h[d["main"]][k] += v
    if int(d["P3_only_refused"]) > 0:
        firsts.append(line.split("first_P3_only=[", 1)[1].split("]", 1)[0] + f"  <- slots={d['slots']} main={d['main']} script={d['script']}")
print("主线臂 | 两边都放行 | 两边都拒 | P3 拒而 P1 放行 | P1 拒而 P3 放行")
for m, a in agg.items():
    print(m, "|", " | ".join(str(x) for x in a))
for m in hb:
    print(f"{m} 多扣分布（根环转过之前）", dict(sorted(hb[m].items())))
    print(f"{m} 多扣分布（根环转过之后）", dict(sorted(ha[m].items())))
print("P3 拒而 P1 放行的首格（前 20 条）：")
for f in firsts[:20]:
    print("  ", f)
