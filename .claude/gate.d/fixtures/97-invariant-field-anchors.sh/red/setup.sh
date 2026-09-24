#!/usr/bin/env bash
# 红样本：这次改动新增了一行不变量，而它没点名自己判的字段住在哪条已定分项。
# invariants.md 是这次新建、还没 git add 的（未跟踪）：整份都算新写，I-99.1 点了名、I-99.2 没点名，只红 I-99.2 那一行。
# 不把未跟踪算进改动范围的取法会报「这次改动没有新写或改写不变量行」而判绿。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
printf '样本仓\n' > README.md
git add -A
git commit -qm '基准：还没有不变量清单'
mkdir -p .claude/kb
cat > .claude/kb/invariants.md <<'NEW'
# 不变量清单（判别力样本）

| # | 简称 | 内容 | 状态 |
|---|---|---|---|
| I-99.1 | 样本点名了分项的条目 | 样本正文，字段落点写在 D99（样本决策） 已定项 1 | 未实现 |
| I-99.2 | 样本新条目 | 样本正文，这一行一个已定分项都没点名 | 未实现 |

## 历史版本

样本文件，没有历史。
NEW
