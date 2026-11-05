#!/usr/bin/env python3
"""读 out/*.jsonl，写 markdown 片段：判错表（每条命令一行，几个角色合并）和"不在清单里"的真 bash 行为。用法：mkreport.py out_dir table.md"""
import glob, json, os, re, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from classify import classify
out_dir, table = sys.argv[1], sys.argv[2]
rows = {}
for p in sorted(glob.glob(os.path.join(out_dir, "*.jsonl"))):
    for line in open(p):
        r = json.loads(line); rows[r["id"]] = r
ROLE = {"sub": "子", "editor": "子(Edit)", "main": "主"}


def norm(cmd):
    cmd = re.sub(r"/home/fy5090/\.local/share/singlefs/work/shell-words-r3-sonnet-s2/u/[^/\s'\"]+", "{U}", cmd)
    cmd = re.sub(r"/home/fy5090/\.local/share/singlefs/work/shell-words-r3-sonnet-s2/wr/copy", "{C}", cmd)
    cmd = re.sub(r"/home/fy5090/\.local/share/singlefs/work/shell-words-r3-sonnet-s2/wr/[^/\s'\"]*", "{W}/…", cmd)
    cmd = re.sub(r"/home/fy5090/\.local/share/singlefs/work/shell-words-r3-sonnet-s2/model/tools/bin", "{B}", cmd)
    cmd = re.sub(r"/home/fy5090/\.local/share/singlefs/work/shell-words-r3-sonnet-s2", "{D}", cmd)
    return cmd.replace("{R}", "{R}")


groups = {}
for r in rows.values():
    if "home-cargo" in r["id"]:
        continue   # 假家目录下 ~ 展开不同，样本本身有毛病，已从语料删掉
    if r["want_reject"] == (r["hook"] == "拒"):
        continue
    c = classify(r)
    role = ROLE[r["role"]] + ("(bg)" if r["bg"] else "") + (("无前缀" if r["id"].endswith("np") else "带前缀") if r["role"] == "main" and r["id"].endswith(("np", "pf")) else "")
    key = (norm(r["cmd"]).replace("export SINGLEFS_HEAVY_TESTS=commit; ", "").replace("SINGLEFS_HEAVY_TESTS=commit ", ""), r["want_reject"], r["hook"], c[0], c[1])
    g = groups.setdefault(key, {"roles": [], "r": r, "c": c})
    g["roles"].append(role)
with open(table, "w") as fo:
    fo.write("| # | 命令原样 | 角色 | 该判 | 钩子判（重型闸/检出钩子退出码） | 归类 | 文件头写明看不见/接受 |\n|---|---|---|---|---|---|---|\n")
    for i, (key, g) in enumerate(sorted(groups.items(), key=lambda kv: (kv[1]["c"][0], kv[0][0])), 1):
        r, c = g["r"], g["c"]
        det = "/".join(str(v["rc"]) for v in r["hook_detail"].values())
        cmd = key[0].replace("|", "\\|").replace("\n", "↵")
        fo.write(f"| {i} | `{cmd[:230]}` | {'、'.join(sorted(set(g['roles'])))} | {'拒' if key[1] else '放'}（{'; '.join(w[0] for w in r['want']) or '-'}） | {key[2]}（{det}） | {c[0]} {c[1]} | {c[2].split('（')[0]} |\n")
print(len(groups), "行；逐样本判错", sum(len(g["roles"]) for g in groups.values()))


# —— "不在清单里"一节：每个子类挑一条代表，写真 bash 的行为与钩子退出码 ——
def behaviour(r):
    o = r["obs"]
    evs = []
    for name, args, heavy, mem in o["events"]:
        if name in ("cargo", "claude", "scripts/hosts/detached-run.sh", "scripts/agents/watch.sh") or name.startswith(("scripts/", ".claude/")):
            evs.append(f"{name} {' '.join(args[:5])}（前缀={heavy or '无'}，内存包装={'有' if mem else '无'}）")
    text = "；".join(evs[:3]) or "没有桩被起"
    extra = []
    if o["timed_out"]:
        extra.append("跑满超时被杀")
    if o["orphans"]:
        extra.append("残留进程 " + "、".join(o["orphans"][:2]))
    shown = [c["path"] for c in o["changed"] if "__pycache__" not in c["path"]]
    if shown:
        extra.append("仓内改动 " + "、".join(shown[:3]))
    return text + ("；" + "；".join(extra) if extra else "") + f"；bash 退出码 {o['rc']}"


zsec = os.path.join(os.path.dirname(table), "znotes.md")
by = {}
for g in groups.values():
    r, c = g["r"], g["c"]
    if c[0].startswith("Z"):
        by.setdefault(c[0], []).append((c, r, sorted(set(g["roles"]))))
with open(zsec, "w") as fo:
    for zid in sorted(by):
        items = by[zid]
        c = items[0][0]
        fo.write(f"### {zid} {c[1]}（{sum(len(i[2]) for i in items)} 个角色×写法，{len(items)} 条命令）\n\n文件头怎么说：{c[2]}\n\n")
        for c, r, roles in items[:4]:
            det = {k.split("-")[0]: v["rc"] for k, v in r["hook_detail"].items()}
            fo.write(f"- `{norm(r['cmd']).replace(chr(10), '↵')[:200]}`（{'、'.join(roles)}）：该判**{'拒' if r['want_reject'] else '放'}**；真 bash：{behaviour(r)}；钩子退出码 heavy={det['heavy']} detector={det['bash']}\n")
        if len(items) > 4:
            fo.write(f"- 同子类另有 {len(items) - 4} 条，见判错表\n")
        fo.write("\n")
