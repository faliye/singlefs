#!/usr/bin/env bash
# 本该判绿：基准那一版的 E1 页没有语言切换行，这次改动只在顶上第三行新插了切换行，外加紧跟它的一个空行——
# 切换行是三语导航，新插进来与改了一样不算「正文改了」，影响的决策表一行没回看也不红（触发回看 0 处，只改语言切换行的改动 1 份）。
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=inserted-switch-counted 下去掉切换行时留着紧跟的空行，两版多出一个空行，这一份判红在「没回看」；
# EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=language-switch-counted 下切换行算正文，同样判红在「没回看」。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/kb/decisions .claude/kb/experiments research/results
cat > .claude/kb/decisions/01-甲.md <<'EOF'
## D1 甲 —— 已定

### 已定项

| # | 分项 | 定案 |
|---|---|---|
| 1 | 样本分项 | 一句话定案 |

#### 已定项 1：样本分项

**定案**：样本规则。

**射程**：只管样本。

**依据**：E1（样本实验一） 证明了样本规则成立。

**欠**：无

## 历史版本
EOF
cat > .claude/kb/experiments/01-样本一.md <<'EOF'
## E1 样本实验一 —— 已跑

正文提到 D1（甲）。产物 `research/results/e1-sample.out`，复跑经 `scripts/experiment-apparatus/replay.sh`。

### 2026-09-18：第一段

- 第一段跑了 5 轮。

### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D1（甲） 已定项 1 | 支撑 | 2026-09-18 不受影响：结论与定案同向 |

## 历史版本

### 2026-09-18
- 样本
EOF
cat > .claude/kb/experiments-history.md <<'EOF'
# 实验变更史

## 历史版本
EOF
printf 'v1\n' > research/results/e1-sample.out
git add -A && git commit -qm base
# 这次改动：只在第三行新插语言切换行，外加紧跟它的一个空行，正文与表一行不动
cat > .claude/kb/experiments/01-样本一.md <<'EOF'
## E1 样本实验一 —— 已跑

**中文** · [English](01-sample-one.en.md) · [日本語](01-サンプル一.ja.md)

正文提到 D1（甲）。产物 `research/results/e1-sample.out`，复跑经 `scripts/experiment-apparatus/replay.sh`。

### 2026-09-18：第一段

- 第一段跑了 5 轮。

### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D1（甲） 已定项 1 | 支撑 | 2026-09-18 不受影响：结论与定案同向 |

## 历史版本

### 2026-09-18
- 样本
EOF
