#!/usr/bin/env bash
# 建一个最小的 git 仓：先提交决策正文、索引表与变更史，推到本地的上游，再按样本意图改动。
# 改动只把首行的状态从「已定」翻成「半定（一项未定）」：增删合计 2 行，不超过小改动的 4 行，
# 但碰了状态行，不许按小改动放行；而且这一笔已经提交、还没推出去——基准取 HEAD 就看不见它。
# 变更史里基准就有的那条条目不算这一轮新增，entry-added 照样判红。
# 另一格也红：新建的一份 kb 页用了日期与序号粘在一起的标题（shape）。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p .claude/kb/decisions
cat > .claude/kb/decisions/01-样本决策.md <<'X'
## D1 样本决策 —— 已定
第一行。
第二行。
第三行。
第四行。
第五行。
## 历史版本
本决策的历史条目集中在 decisions-history.md。
X
cat > .claude/kb/decisions.md <<'X'
# 设计决策记录

| 决策 | 状态 | 结论（简报） | 正文 |
|---|---|---|---|
| D1（样本决策） | 已定 1 项 / 未定 0 项 | 块取 16 KiB | [01-样本决策.md](decisions/01-样本决策.md) |

## 历史版本
X
cat > .claude/kb/decisions-history.md <<'X'
# 决策变更史

## D1（样本决策）

### 2026-09-12

- 基准里就有的一条

## 历史版本
X
git add -A && git commit -qm base
git init -q --bare "$PWD/.git/fixture-upstream.git"
git remote add origin "$PWD/.git/fixture-upstream.git"
git push -q -u origin HEAD
sed -i 's/^## D1 样本决策 —— 已定$/## D1 样本决策 —— 半定（一项未定）/' .claude/kb/decisions/01-样本决策.md
git commit -qam '翻了状态，没写变更史，也还没推出去'
# 新建、没进 git 的一份 kb 页：整份算新增
cat > .claude/kb/invariants.md <<'X'
# 不变量

## 历史版本

### 2026-09-28（其一）：新写的一条还是旧形态
X
