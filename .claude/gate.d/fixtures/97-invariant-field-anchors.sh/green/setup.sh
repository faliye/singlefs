#!/usr/bin/env bash
# 绿样本：同一次改动新增的那一行点名了它判的字段住在哪条已定分项。
# 这一行改完又 git add 了：「基准到工作区」与「HEAD 到暂存区」两份 diff 里都有它，只许算一行（不去重会报 2 行）。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
mkdir -p .claude/kb
cat > .claude/kb/invariants.md <<'BASE'
# 不变量清单（判别力样本）

| # | 简称 | 内容 | 状态 |
|---|---|---|---|
| I-99.1 | 样本已有条目 | 样本正文，字段落点写在 D99（样本决策） 已定项 1 | 未实现 |

## 历史版本

样本文件，没有历史。
BASE
git add -A
git commit -qm '基准：只有一条已有的不变量'
python3 - <<'PY'
path = '.claude/kb/invariants.md'
text = open(path, encoding='utf-8').read()
old = '| I-99.1 | 样本已有条目 | 样本正文，字段落点写在 D99（样本决策） 已定项 1 | 未实现 |\n'
text = text.replace(old, old + '| I-99.2 | 样本新条目 | 样本正文，字段落点写在 D99（样本决策） 已定项 2 | 未实现 |\n')
open(path, 'w', encoding='utf-8').write(text)
PY
git add .claude/kb/invariants.md
