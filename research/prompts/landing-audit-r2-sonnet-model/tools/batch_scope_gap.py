#!/usr/bin/env python3
"""被判文件里哪些对 HEAD 有改动、命中 .claude/gate.d/knowledge-sync-triggers.tsv 的触发清单、却没登记在 .claude/batch-scope。
用法：batch_scope_gap.py <仓根> <被判文件清单（sha256sum 格式，如 judged-sha256.txt）>
只读。判法是我读 doc-process-records.sh 的 batch-scope 格文件头得来的，不是跑那一格。"""
import re, sys, os, subprocess
root, listing = sys.argv[1], sys.argv[2]
triggers = [l.split("\t")[0].strip() for l in open(os.path.join(root, ".claude/gate.d/knowledge-sync-triggers.tsv"), encoding="utf-8") if l.strip() and not l.startswith("#")]
scope = {l.split("\t")[0].split(" ")[0].strip() for l in open(os.path.join(root, ".claude/batch-scope"), encoding="utf-8") if l.strip() and not l.startswith("#")}
judged = [l.split(None, 1)[1].strip() for l in open(listing, encoding="utf-8") if l.strip()]

def git(*args):
    return subprocess.run(["git", "-C", root, *args], capture_output=True, text=True, check=True).stdout.split("\n")

changed = set(git("diff", "--name-only", "HEAD")) | set(git("ls-files", "--others", "--exclude-standard"))
judged = [p for p in judged if p in changed]   # 只算对 HEAD 真有改动的
hit = [p for p in judged if any(re.search(t, p) for t in triggers)]
missing = [p for p in hit if p not in scope]
print(f"被判文件里对 HEAD 有改动的 {len(judged)} 份；命中触发清单 {len(hit)} 份；其中已在 batch-scope {len(hit) - len(missing)} 份、不在 {len(missing)} 份")
for p in missing:
    print("  不在:", p)
