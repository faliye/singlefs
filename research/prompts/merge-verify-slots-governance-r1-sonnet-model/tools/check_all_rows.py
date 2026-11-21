#!/usr/bin/env python3
"""只读：每张表整张（不只改过的行）数锚串命中行数，列出 0 命中与文件不在的行。用法：check_all_rows.py <仓根>"""
import os, re, sys
root = sys.argv[1]
def lines_of(p):
    with open(os.path.join(root, p), encoding="utf-8") as f:
        return f.read().splitlines()
def hits(file, anchor):
    full = os.path.join(root, file)
    if not os.path.isfile(full):
        return None
    return sum(1 for ln in lines_of(file) if anchor in ln)
for table, kind in [(".claude/heavy-thread-exempt","colon"),(".claude/gate.d/repository-copy-closure-exempt.tsv","colon"),(".claude/gate.d/hardcoded-quota-exempt.tsv","colon"),(".claude/term-rename-exempt","term")]:
    total = bad = 0; multi = 0
    for n, l in enumerate(lines_of(table), 1):
        if not l.strip() or l.startswith("#"):
            continue
        m = re.match(r"^(.*?)\s{2,}#\s*(.*)$", l)
        body = m.group(1) if m else l
        if kind == "term":
            mm = re.match(r"^(.*?):「(.*)」\s*$", body)
            if not mm:
                p = body.strip(); total += 1
                if not os.path.exists(os.path.join(root, p)):
                    bad += 1; print(f"{table}:{n}: 路径不在 {p}")
                continue
            file, anchor = mm.group(1), mm.group(2)
        else:
            file, _, anchor = body.partition(":")
        total += 1
        h = hits(file, anchor)
        if h is None or h == 0:
            bad += 1; print(f"{table}:{n}: hits={h} {file} 〔{anchor[:50]}〕")
        elif kind == "term" and h != 1:
            multi += 1; print(f"{table}:{n}: 行内锚命中 {h} 行（要恰好 1） {file}")
    print(f"== {table}: 共 {total} 行，0 命中或文件不在 {bad}，行内锚不恰好 1 的 {multi}")
