"""x3chg.py : 每族里「第二轮被判那一版（r2）与此刻（now）的退出码不同」的条，按（r2 判定 -> 此刻判定）计数。判定对着自己的真 bash 裁判。"""
import json, collections
fams = ["key", "cpforms", "brace", "heredoc", "redirect", "ln", "env", "shell", "builtin", "function", "fncall", "empties", "hwrite", "ansic", "r1-575", "fuzz2", "head1"]
tot = collections.Counter()
for fam in fams:
    cases = json.load(open(f"cases/cases-{fam}.json")); orc = json.load(open(f"res/{fam}.oracle.json"))
    a = json.load(open(f"res/{fam}.r2.json")); b = json.load(open(f"res/{fam}.now.json"))
    cnt = collections.Counter()
    for c, o, x, y in zip(cases, orc, a, b):
        if x["rc"] == y["rc"]: continue
        if o.get("noexec"): exp = c["expect"]
        else:
            if "syntax error" in o["err1"] or "unexpected" in o["err1"]: cnt["语法错"] += 1; continue
            exp = 2 if (o["heavy"] if c["hook"] == "guard" else o["detached"]) else 0
        v = lambda rc: "对" if rc == exp else ("误拒" if rc == 2 else "漏拒")
        cnt[f"{v(x['rc'])}->{v(y['rc'])}"] += 1
    print(fam, dict(sorted(cnt.items())) or "无变化")
