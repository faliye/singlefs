#!/usr/bin/env bash
# 建一个最小的 git 仓：先提交决策正文、索引表与还没有条目的变更史，再按样本意图改动。
# 三格都要绿：决策正文改了 7 行，D1 节里新增一个日期块、块下一条 `#### `；节顶上的现状行与索引表一致。
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

**现状**：已定 1 项 / 未定 0 项。块取 16 KiB

## 历史版本
X
git add -A && git commit -qm base
# 改决策正文 —— 超过「小改动」的 4 行阈值
sed -i 's/^第一行。/第一行（改过）。\n新增一行。\n再新增一行。/' .claude/kb/decisions/01-样本决策.md
sed -i 's/^第三行。/第三行（也改过）。\n又一行。/' .claude/kb/decisions/01-样本决策.md
# 条目写进 D1 自己的节：现状行下面新开一个日期块
cat > .claude/kb/decisions-history.md <<'X'
# 决策变更史

## D1（样本决策）

**现状**：已定 1 项 / 未定 0 项。块取 16 KiB

### 2026-08-29

#### 曾经 X，现在 Y

- **改前**：X。
- **改后**：Y。
- **依据**：Z。

## 历史版本
X
