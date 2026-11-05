"""x8910.py : 第二轮第 8 条（编译目录一道的函数体）、第 9 条（⑫ 花括号）、第 10 条（⑨ 从 cwd 起算）三组样本在 r1 / r2 / now 上的读数。"""
import json, collections
c = json.load(open("cases/cases-x8.json")); o = json.load(open("res/x8.oracle.json"))
for t in ("r1", "r2", "now"):
    r = json.load(open(f"res/x8.{t}.json")); cnt = collections.Counter(); ex = collections.defaultdict(list)
    for x, y, z in zip(c, o, r):
        if "syntax error" in y["err1"] or "unexpected" in y["err1"]: continue
        exp = 2 if any(l.startswith("cargo build") for l in y["log"]) else 0
        k = "对" if z["rc"] == exp else ("误拒" if z["rc"] == 2 else "漏拒"); cnt[k] += 1; ex[k].append(x["cmd"])
    print("第8条", t, dict(cnt), "放行的:", [x["cmd"] for x, z in zip(c, r) if z["rc"] == 0][:4])
for name, sel in (("第9条", "x9"), ("第10条", "x10")):
    cs = json.load(open(f"cases/cases-{sel}.json")); R = {t: json.load(open(f"res/{sel}.{t}.json")) for t in ("r1", "r2", "now")}
    bad = 0
    for i, x in enumerate(cs):
        rc = [R[t][i]["rc"] for t in ("r1", "r2", "now")]
        if rc[2] != x["expect"]:
            bad += 1; print(f"{name} 与我写的应当值不符: 应当{x['expect']} r1={rc[0]} r2={rc[1]} now={rc[2]} | cwd={x['cwd']} | {x['cmd']!r}")
    print(name, "共", len(cs), "条，与应当值不符", bad)
