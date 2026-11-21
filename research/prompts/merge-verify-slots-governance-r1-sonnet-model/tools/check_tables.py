#!/usr/bin/env python3
"""只读：对每张排除表，列出相对 HEAD 新增的行，并数锚串在目标文件里命中几行。用法：check_tables.py <仓根>"""
import os, re, subprocess, sys
root = sys.argv[1]
def head(path):
    p = subprocess.run(["git", "-C", root, "show", f"HEAD:{path}"], capture_output=True, text=True)
    return p.stdout.splitlines() if p.returncode == 0 else []
def now(path):
    with open(os.path.join(root, path), encoding="utf-8") as f:
        return f.read().splitlines()
def hits(file, anchor):
    full = os.path.join(root, file)
    if not os.path.isfile(full):
        return None
    with open(full, encoding="utf-8", errors="replace") as f:
        return sum(1 for ln in f.read().splitlines() if anchor in ln)
def split_row(ln, quoted=False):
    m = re.match(r"^(.*?)\s{2,}#\s*(.*)$", ln)
    body, reason = (m.group(1), m.group(2)) if m else (ln, "")
    return body, reason
tables = [
 (".claude/heavy-thread-exempt", "colon"),
 (".claude/gate.d/repository-copy-closure-exempt.tsv", "colon"),
 (".claude/gate.d/hardcoded-quota-exempt.tsv", "colon"),
 (".claude/memory-bounded-parallelism.tsv", "tab"),
 (".claude/preflight-exclude", "path"),
 (".claude/term-rename-exempt", "term"),
]
for table, kind in tables:
    old = set(head(table)); new = now(table)
    added = [(i + 1, l) for i, l in enumerate(new) if l not in old and l.strip() and not l.startswith("#")]
    removed = [l for l in head(table) if l not in set(new) and l.strip() and not l.startswith("#")]
    print(f"=== {table}: 新增行 {len(added)}，删去行 {len(removed)}")
    for n, l in added:
        if kind == "colon":
            body, reason = split_row(l)
            file, _, anchor = body.partition(":")
            h = hits(file, anchor)
            print(f"{n}\thits={h}\t{file}\t{anchor[:60]}")
        elif kind == "tab":
            parts = l.split("\t")
            e = parts[0]
            ex = os.path.isfile(os.path.join(root, e)) if not e.startswith("cargo:") else None
            print(f"{n}\texists={ex}\t{e}")
        elif kind == "path":
            body, reason = split_row(l)
            print(f"{n}\texists={os.path.exists(os.path.join(root, body.strip()))}\t{body}\t# {reason[:70]}")
        else:
            body, reason = split_row(l)
            m = re.match(r"^(.*?):「(.*)」\s*$", body)
            if m:
                file, anchor = m.group(1), m.group(2)
                print(f"{n}\thits={hits(file, anchor)}\t{file}\t「{anchor[:50]}」")
            else:
                full = os.path.join(root, body.strip())
                print(f"{n}\texists={os.path.exists(full)}\tdir={os.path.isdir(full)}\t{body}")
