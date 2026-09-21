#!/usr/bin/env bash
# 红样本：这次改动新增了一行不变量，而它没点名自己判的字段住在哪条已定分项。
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
text = text.replace(old, old + '| I-99.2 | 样本新条目 | 样本正文，这一行一个已定分项都没点名 | 未实现 |\n')
open(path, 'w', encoding='utf-8').write(text)
PY
