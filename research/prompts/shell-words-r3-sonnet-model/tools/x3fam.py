"""x3fam.py : 修补报告「三、正推腿甲」各族迁移表，自己的判法重算（r1 = 第一轮被判那一版 -> r2 = 第二轮被判那一版 / now = 此刻），逐格对「腿登记的第二轮」与「报告写的此刻」。"""
import json, re, ast, collections, sys
import os
REG = os.environ.get("REPO_ROOT", "/home/fy5090/code/singlefs") + "/research/prompts/shell-words-r2-sonnet-model/out/totals.txt"
reg = {}
for l in open(REG):
    m = re.match(r"(\S+) 合计 (\{.*\}) 语法错 (\d+) 共 (\d+)", l)
    if m: reg[m.group(1)] = (ast.literal_eval(m.group(2)), int(m.group(3)))
# 报告「此刻」列里写出的格（只写变了的；没写的格报告说与登记相同）
rep = {
 "brace": {"对->对": 177, "对->误拒": 6}, "redirect": {"对->对": 194, "对->误拒": 24, "漏拒->对": 39, "漏拒->漏拒": 68, "误拒->对": 64, "误拒->误拒": 31, "对->漏拒": 0},
 "env": {"对->对": 161, "对->误拒": 3}, "shell": {"对->对": 570, "对->误拒": 8, "误拒->对": 53, "误拒->误拒": 38}, "fncall": {"对->对": 468, "对->漏拒": 0},
 "hwrite": {"误拒->对": 8, "误拒->误拒": 0}, "ansic": {"漏拒->对": 23, "漏拒->漏拒": 0}, "r1-575": {"对->对": 440, "漏拒->对": 42, "漏拒->漏拒": 20, "误拒->对": 8, "误拒->误拒": 59},
 "fuzz2": {"误拒->对": 143, "误拒->误拒": 112}, "head1": {"漏拒->对": 778, "对->漏拒": 0, "漏拒->漏拒": 0},
}
def trans(fam, a, b):
    cases = json.load(open(f"cases/cases-{fam}.json")); orc = json.load(open(f"res/{fam}.oracle.json"))
    ra = json.load(open(f"res/{fam}.{a}.json")); rb = json.load(open(f"res/{fam}.{b}.json"))
    cnt = collections.Counter(); syn = 0
    for c, o, x, y in zip(cases, orc, ra, rb):
        if o.get("noexec"): exp = c["expect"]
        else:
            if "syntax error" in o["err1"] or "unexpected" in o["err1"]: syn += 1; continue
            exp = 2 if (o["heavy"] if c["hook"] == "guard" else o["detached"]) else 0
        v = lambda rc: "对" if rc == exp else ("误拒" if rc == 2 else "漏拒")
        cnt[f"{v(x['rc'])}->{v(y['rc'])}"] += 1
    return cnt, syn
fams = ["key", "cpforms", "brace", "heredoc", "redirect", "ln", "env", "shell", "builtin", "function", "fncall", "empties", "hwrite", "ansic", "r1-575", "fuzz2", "head1"]
print("族 | 腿登记的第二轮 | 我重算 r1->r2 | 报告写的此刻 | 我重算 r1->now")
for f in fams:
    c2, s2 = trans(f, "r1", "r2"); cn, sn = trans(f, "r1", "now"); rg, rs = reg[f]
    cells = sorted(set(rg) | set(c2) | set(cn) | set(rep.get(f, {})))
    d2 = [k for k in cells if rg.get(k, 0) != c2.get(k, 0)]; dn = [k for k in cells if k in rep.get(f, {}) and rep[f][k] != cn.get(k, 0)]
    # 报告没写的格视为与登记相同
    dn2 = [k for k in cells if k not in rep.get(f, {}) and rg.get(k, 0) != cn.get(k, 0)]
    print(f"== {f}: 登记语法错 {rs} / 我 r2 语法错 {s2} / 我 now 语法错 {sn}")
    print(f"   登记 vs 我(r2) 不同的格: " + (", ".join(f"{k}:{rg.get(k,0)}|{c2.get(k,0)}" for k in d2) or "无"))
    print(f"   报告此刻 vs 我(now) 不同的格: " + (", ".join(f"{k}:{rep[f][k]}|{cn.get(k,0)}" for k in dn) or "无") + ("；报告没写的格里与登记不同: " + ", ".join(f"{k}:登记{rg.get(k,0)}|我now{cn.get(k,0)}" for k in dn2) if dn2 else ""))
