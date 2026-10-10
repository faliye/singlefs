#!/usr/bin/env python3
"""K3 第四点：20-1 原补丁的每个 hunk 与背景材料里「合入前备份对合入后主仓」那份 diff 的对应 hunk 逐行比（含上下文行），
列出两边不同的行：不同的行就是主 agent 重定基时碰过的地方。用法：hunk_context_diff.py <补丁> <diff.md>"""
import difflib, sys

def hunks(lines):
    out, cur, file = [], None, None
    for l in lines:
        if l.startswith("diff --git"):
            file, cur = l.split(" b/")[-1].strip(), None
            continue
        if l.startswith("@@"):
            cur = {"file": file, "hdr": l.strip(), "body": []}
            out.append(cur)
            continue
        if cur is not None and l[:1] in ("+", "-", " ") and l != "" and not l.startswith(("+++", "---")):
            cur["body"].append(l)
    return out

patch = open(sys.argv[1], encoding="utf-8").read().split("\n")
diffmd = open(sys.argv[2], encoding="utf-8").read().split("\n")
hp = hunks(patch)
hd = [h for h in hunks(diffmd) if h["file"] and not h["file"].endswith("mutations.tsv")]
# 补丁的第 14 块在 diff.md 里被拆成两块（解构入参一块、摆法 match 一块）；其余一一对应，diff.md 比补丁多出来的那一块是拆出来的
pairs = [(i, i) for i in range(14)] + [(14, None)] + [(15, 16), (16, 17), (17, 18), (18, 19), (19, 20), (20, 21), (21, 22), (22, 23)]
print(f"补丁 {len(hp)} 块；diff.md 里非 tsv 的 {len(hd)} 块")
for a, b in pairs:
    pb = hp[a]["body"]
    mb = hd[14]["body"] + hd[15]["body"] if b is None else hd[b]["body"]
    d = [l for l in difflib.unified_diff(pb, mb, lineterm="", n=0) if not l.startswith(("---", "+++", "@@"))]
    print(f"## 补丁第 {a} 块（{hp[a]['file'].split('/')[-1]}）与合入后对应块不同的行：{len(d)}")
    for l in d:
        print("    " + l[:170])
