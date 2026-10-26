"""judge.py <SW_ROOT> <cases.json> <输出前缀> : 真 bash 当裁判（oracle.py），再在 tree-orig（修补前）与 tree（修补后）两棵树上逐条喂钩子（fam4.py），
逐条打印：裁判（该拒 2 / 该放 0）、修补前 rc、修补后 rc、两棵树各判对没有（对 / 误拒 / 漏拒）；两棵树结果不同或修补后仍错的行尾带「<==」。
cases.json 里每条 {tag, cmd, hook:guard|det, agent, [cwd], [noexec 与 expect：不跑 bash，裁判用它自己带的 expect]}。
真 bash 自己报语法错 / 读不完的命令（bash -c 一个字都没跑）不进统计，单列条数。"""
import sys, os, json, subprocess
root, cases, prefix = sys.argv[1:4]
here = os.path.dirname(os.path.abspath(__file__))
env = dict(os.environ, SW_ROOT=root, SINGLEFS_WORK_ROOT="/var/empty/swr")
fam = prefix + "-fam.json"
subprocess.run(["python3", "-I", here + "/oracle.py", root + "/scratch", root + "/oracle/bin", cases, fam], check=True, env=env, stdout=subprocess.DEVNULL)
for tree in ("tree-orig", "tree"):
    subprocess.run(["python3", "-I", here + "/fam4.py", fam, tree, f"{prefix}-{tree}.json"], check=True, env=env)
o = json.load(open(prefix + "-tree-orig.json")); n = json.load(open(prefix + "-tree.json"))
def verdict(exp, rc):
    if rc == exp: return "对"
    return "误拒" if rc == 2 else ("漏拒" if rc == 0 else f"异常{rc}")
cnt, syntax = {}, 0
for a, b in zip(o, n):
    err = a.get("bash_err", "") or ""
    if not a.get("noexec") and ("syntax error" in err or "unexpected" in err):
        syntax += 1
        continue
    vo, vn = verdict(a["expect"], a["rc"]), verdict(a["expect"], b["rc"])
    cnt[(vo, vn)] = cnt.get((vo, vn), 0) + 1
    flag = "" if vo == vn == "对" else "  <=="
    print(f"{a['tag']:<14}|{a['hook']}/{a['agent']}|裁判={a['expect']}|前={a['rc']}({vo})|后={b['rc']}({vn}){flag}|{a['cmd']!r}" + (f"|bash:{err}" if err and vn != '对' else ""))
print("统计 (修补前判定,修补后判定):", {f"{k[0]}->{k[1]}": v for k, v in sorted(cnt.items())}, "语法错", syntax)
