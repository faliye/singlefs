#!/usr/bin/env bash
# 与 history-loss-red 同一个基准；工作区把两条变更整理成一条结论，数（32 KiB、16 KiB、1.7）与编号 E5 都在：判绿。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p .claude/kb
cat > .claude/kb/decisions-history.md <<'X'
# 决策变更史

## D1（样本决策）

### 2026-09-10

- 已定项 1 定案：节点 32 KiB
  - **依据**：E5（乙） 量到 1.7 倍。
- 索引改成 16 KiB 起步

## 历史版本
X
git add -A && git commit -qm base
cat > .claude/kb/decisions-history.md <<'X'
# 决策变更史

## D1（样本决策）

### 2026-09-10

- 已定项 1 定案：节点 32 KiB，索引 16 KiB 起步；依据 E5（乙） 量到 1.7 倍。

## 历史版本
X
