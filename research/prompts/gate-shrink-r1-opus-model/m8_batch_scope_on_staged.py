#!/usr/bin/env python3
"""模型 M8（gate-shrink-r1 云端攻方）：B9 里 11 号 batch-scope 那一格，按提交时 gate-staged.sh 的暂存树会判出什么。
不起临时 worktree，只按那一格的判据现算：触发文件 = 暂存区相对 HEAD 改动的路径里命中 .claude/gate.d/knowledge-sync-triggers.tsv
任一条正则的；登记 = .claude/batch-scope 里每行第一个词。报：暂存路径数、其中触发文件数、没登记的触发文件数、登记了却不在这一批的条数。
（暂存树里没有未跟踪文件，工作区的改动也不在，所以只看 git diff --cached。）只读。
用法：python3 m8_batch_scope_on_staged.py <仓根>
"""
import os, re, subprocess, sys

root = sys.argv[1]
patterns = []
for line in open(os.path.join(root, ".claude/gate.d/knowledge-sync-triggers.tsv"), encoding="utf-8"):
    if line.strip() and not line.startswith("#"):
        patterns.append(re.compile(line.split("\t")[0]))
staged = [path for path in subprocess.run(["git", "-C", root, "-c", "core.quotepath=false", "diff", "--cached", "--name-only", "HEAD"],
                                          capture_output=True, text=True, check=True).stdout.split("\n") if path]
triggers = [path for path in staged if any(pattern.search(path) for pattern in patterns)]
registered = set()
for line in open(os.path.join(root, ".claude/batch-scope"), encoding="utf-8"):
    stripped = line.strip()
    if stripped and not stripped.startswith("#"):
        registered.add(re.split(r"[\t ]", stripped)[0])
print(f"staged={len(staged)} staged_triggers={len(triggers)} registered={len(registered)} "
      f"unregistered_triggers={len([path for path in triggers if path not in registered])} "
      f"registered_not_in_batch={len([path for path in registered if path not in set(triggers)])}")
