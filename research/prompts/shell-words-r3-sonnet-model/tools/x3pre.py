"""x3pre.py : 第一轮 575 条，第一轮判决之前的树（pre）-> 第二轮被判那一版（r2）/ 此刻（now）的退出码矩阵；探针目录取第一轮的形状（没有 scripts/hosts）。再列「pre 拒、now 放」的每一条与真 bash 的行为。"""
import json, collections
c = json.load(open("cases/cases-r1-575-bare.json")); o = json.load(open("res/r1-575.oracle.json"))
R = {t: json.load(open(f"res/r1-575-bare.{t}.json")) for t in ("pre", "r1", "r2", "now")}
for t in ("r2", "now"):
    m = collections.Counter(f"{R['pre'][i]['rc']}->{R[t][i]['rc']}" for i in range(len(c)))
    print("pre ->", t, dict(sorted(m.items())), "共", sum(m.values()))
k = 0
for i, x in enumerate(c):
    if R["pre"][i]["rc"] == 2 and R["now"][i]["rc"] == 0:
        k += 1; y = o[i]
        heavy = (y.get("heavy") if x["hook"] == "guard" else y.get("detached"))
        print(f"{k}. {x['hook']}/{x['agent']} r1树={R['r1'][i]['rc']} 真bash跑出重活={'是' if heavy else '否'} bash_rc={y.get('rc')} 命令={x['cmd']!r}")
