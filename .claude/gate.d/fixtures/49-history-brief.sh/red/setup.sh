#!/usr/bin/env bash
# 同一个现场改坏两处：少了 2026-09-01 那一行，改后那一格写成原文里没有的 32。
set -e
mkdir -p .claude/kb/decisions-history .claude/kb/decisions
cat > .claude/kb/decisions-history/2026-09.md <<'X'
# 决策变更史 · 2026-09

## 历史版本

### 2026-09-02（其一）：D1（样本决策） 已定项 1 定成 16 KiB

- **改前**：D1（样本决策） 第 1 项未定。
- **改后**：定成 16 KiB。

### 2026-09-01

- 新立 D1（样本决策）。
X
cat > .claude/kb/decisions/01-样本决策.md <<'X'
## D1 样本决策 —— 已定
X
cat > .claude/kb/decisions-history.md <<'X'
# 决策变更史

<!-- gen:history-brief:start -->

## 按决策

每条决策在快查表里出现过几次、最近一次是哪天改了什么；按「改了什么」那一格里写着的「D<n>（简称）」数。

| 决策 | 条目数 | 最近一次 | 最近那一次改了什么 |
|---|---|---|---|
| D1（样本决策） | 2 | 2026-09-02（其一） | D1（样本决策） 已定项 1 定成 16 KiB |

## 按日期

### [2026-09](decisions-history/2026-09.md)

| 日期 | 改了什么 | 改前 | 改后 |
|---|---|---|---|
| 2026-09-02（其一） | D1（样本决策） 已定项 1 定成 16 KiB | 第 1 项未定 | 定成 32 KiB |

<!-- gen:history-brief:end -->

## 历史版本
X
