#!/usr/bin/env bash
# 绿样本要 git 仓（invariant-anchors 那一格）：样本里的文件先整份提交当基准，那时清单里只有一条退役的行；
# 再给清单加一行点名了已定分项的不变量、改完又 git add。「基准到工作区」与「HEAD 到暂存区」两份 diff 里都有这一行，
# 只许算一行（不去重会报 2 行）。加完之后在用 1 条，与清单那句「现共 1 条在用」、决策 361 那句「1 条在用」对得上。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
git add -A
git commit -qm '基准：清单里只有一条退役的行'
python3 - <<'PY'
path = '.claude/kb/invariants.md'
text = open(path, encoding='utf-8').read()
old = '| I-92.9 | 绿样退役条 | **此编号不再使用**（样例） |\n'
if old not in text:
    raise SystemExit('样本的清单里找不到退役的那一行，加不上新行')
text = text.replace(old, '| I-92.1 | 绿样条 | 样例陈述，字段落点写在 D340（绿样决策） 已定项 1 |\n' + old)
open(path, 'w', encoding='utf-8').write(text)
PY
git add .claude/kb/invariants.md
