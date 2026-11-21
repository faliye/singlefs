#!/usr/bin/env python3
"""只读：三张「路径:锚串」表里，相对 HEAD 新增的行按（锚串, 理由）对 HEAD 的行：对得上的是换路径（搬迁或拆分），对不上的是新登记。用法：growth.py <仓根>"""
import os, re, subprocess, sys
root = sys.argv[1]
def rows(text):
    out = []
    for n, l in enumerate(text.splitlines(), 1):
        if not l.strip() or l.startswith("#"): continue
        m = re.match(r"^(.*?)\s{2,}#\s*(.*)$", l)
        body, reason = (m.group(1), m.group(2)) if m else (l, "")
        f, _, a = body.partition(":")
        out.append((n, f, a, reason))
    return out
for table in [".claude/heavy-thread-exempt", ".claude/gate.d/repository-copy-closure-exempt.tsv", ".claude/gate.d/hardcoded-quota-exempt.tsv"]:
    old = rows(subprocess.run(["git","-C",root,"show",f"HEAD:{table}"],capture_output=True,text=True).stdout)
    new = rows(open(os.path.join(root, table), encoding="utf-8").read())
    old_keys = {(f, a) for _, f, a, _ in old}
    old_anchor_reason = {(a, r) for _, f, a, r in old}
    old_anchor = {a for _, f, a, r in old}
    moved = [x for x in new if (x[1], x[2]) not in old_keys and (x[2], x[3]) in old_anchor_reason]
    reworded = [x for x in new if (x[1], x[2]) not in old_keys and (x[2], x[3]) not in old_anchor_reason and x[2] in old_anchor]
    fresh = [x for x in new if (x[1], x[2]) not in old_keys and x[2] not in old_anchor]
    print(f"== {table}: 新表 {len(new)} 行；换路径且锚串理由都与 HEAD 某行相同 {len(moved)}；锚串同、理由改字 {len(reworded)}；锚串在 HEAD 里没有的新登记 {len(fresh)}")
    for n, f, a, r in reworded: print(f"   理由改字 {n}: {f} 〔{a[:40]}〕")
    for n, f, a, r in fresh: print(f"   新登记 {n}: {f} 〔{a[:40]}〕")
