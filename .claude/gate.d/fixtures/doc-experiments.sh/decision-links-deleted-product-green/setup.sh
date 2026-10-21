#!/usr/bin/env bash
# 本该判绿：这次改动只删了一份上一轮已提交的产物（research/results/e1-sample.out，归档删上一轮产物那一种），
# E1 的实验页与它影响的决策表一行没动、D1 也没动。删掉的产物不算「产物变了」，不要求回看决策表，成功行报「这次改动触发回看 0 处」。
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=deleted-products-counted 下删除照旧算产物变了，这一份判红在「没回看」。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/kb/decisions .claude/kb/experiments research/results
cat > .claude/kb/decisions/01-甲.md <<'EOF'
## D1 甲 —— 已定

射程：样本决策。

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

正文提到 D1（甲）。

产物 `research/results/e1-sample.out`。

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

### 2026-09-17：E1（样本实验一） 样本条目
- 样本
EOF
printf 'v1\n' > research/results/e1-sample.out
git add -A && git commit -qm base
# 这次改动：只删掉那份已提交的产物（工作区里删，不暂存；暂存了照样是删除）
rm research/results/e1-sample.out
