"""trans.py <族> <前树> <后树> [--list 条件] : 族的裁判结果 res/<族>.oracle.json 与两棵树的钩子 rc 比，打出「前->后」计数（对 / 误拒 / 漏拒），
语法错（bash 自己报 syntax error / unexpected、一个字都没跑）不进统计，单列。--list 列出所有判定变了或后仍错的条（tag|hook/agent|裁判|前|后|cmd）。"""
import sys, json
fam, a, b = sys.argv[1:4]
listing = "--list" in sys.argv
cases = json.load(open(f"cases/cases-{fam}.json")); orc = json.load(open(f"res/{fam}.oracle.json"))
ra = json.load(open(f"res/{fam}.{a}.json")); rb = json.load(open(f"res/{fam}.{b}.json"))
def verdict(exp, rc):
    if rc == exp: return "对"
    return "误拒" if rc == 2 else ("漏拒" if rc == 0 else f"异常{rc}")
cnt = {}; syn = 0; lines = []
for c, o, x, y in zip(cases, orc, ra, rb):
    if o.get("noexec"): exp = c["expect"]
    else:
        if "syntax error" in o["err1"] or "unexpected" in o["err1"]: syn += 1; continue
        exp = 2 if (o["heavy"] if c["hook"] == "guard" else o["detached"]) else 0
    va, vb = verdict(exp, x["rc"]), verdict(exp, y["rc"])
    cnt[f"{va}->{vb}"] = cnt.get(f"{va}->{vb}", 0) + 1
    if va != vb or vb != "对": lines.append(f"{c.get('tag','')}|{c['hook']}/{c['agent']}|裁判={exp}|{a}={x['rc']}({va})|{b}={y['rc']}({vb})|{c['cmd']!r}" + (f"|bash:{o['err1']}" if o.get('err1') else ""))
print(fam, a, b, dict(sorted(cnt.items())), "语法错", syn, "共", sum(cnt.values()) + syn)
if listing:
    for l in lines: print(l)
