"""x3kf.py : 第二轮腿登记的 32 条关键反例（keyfeeds.sh 里 one 行的命令），r1 / r2 / now 三棵树上钩子退出码；r2 与 now 不同的逐条列出。"""
import json
c = json.load(open("cases/cases-kf.json")); R = {t: json.load(open(f"res/kf.{t}.json")) for t in ("r1", "r2", "now")}
n = 0
for i, x in enumerate(c):
    if R["r2"][i]["rc"] != R["now"][i]["rc"]:
        n += 1; print(f"r2={R['r2'][i]['rc']} now={R['now'][i]['rc']} {x['hook']}/{x['agent']} {x['cmd']!r}")
print("r2 与 now 退出码不同:", n, "/", len(c), "；r1 与 r2 不同:", sum(1 for i in range(len(c)) if R["r1"][i]["rc"] != R["r2"][i]["rc"]))
