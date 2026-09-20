#!/usr/bin/env bash
# 本该判绿：D1 已瘦身、依据引 E1，E1 的表里支撑那一行对得上；D2 与 E2 在待回填清单里；
# 这次改动改了 E1 正文与产物、同时回看了表，新写的三方判决带「## 回看决策」且写「改了」的 D1 也在这次改动里。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/kb/decisions .claude/kb/experiments research/results research/prompts
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
cat > .claude/kb/decisions/02-乙.md <<'EOF'
## D2 乙 —— 已定

还没瘦身的决策，分项用编号列表列、没有依据段。

### 已定项

1. **样本分项**：只在编号列表里。

## 历史版本
EOF
cat > .claude/kb/decisions/03-丙.md <<'EOF'
## D3 丙 —— 已定

射程：纯政策的样本决策。

### 已定项

| # | 分项 | 定案 |
|---|---|---|
| 1 | 政策分项 | 不进样本主线 |

#### 已定项 1：政策分项

**定案**：不进样本主线。

**射程**：只管样本。

**依据**：无实验：纯政策定案，没有可量的量；用户定案原话在变更史。

**欠**：无

## 历史版本
EOF
cat > .claude/kb/experiments/01-样本一.md <<'EOF'
## E1 样本实验一 —— 已跑

正文提到 D1（甲） 与 D2（乙），RAID5（不是决策） 不算。

### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D1（甲） 已定项 1 | 支撑 | 2026-09-18 不受影响：结论与定案同向 |
| D2（乙） 已定项 1 | 不影响 | 2026-09-18 不受影响：只作背景提到 |

## 历史版本

### 2026-09-18
- 样本
EOF
cat > .claude/kb/experiments/02-样本二.md <<'EOF'
## E2 样本实验二 —— 已跑

提到 D1（甲），还在待回填清单里。

## 历史版本
EOF
cat > .claude/kb/experiments-history.md <<'EOF'
# 实验变更史

## 历史版本

### 2026-09-17：E1（样本实验一） 样本条目
- 样本
EOF
cat > .claude/kb/experiments/03-样本三.md <<'EOF'
## E3 样本实验三 —— 已跑，结论作废

提到 D1（甲）。

### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D1（甲） | 不影响 | 2026-09-19 不受影响：结论作废，只留着记那条路不通 |

## 历史版本
EOF
printf 'D2  # 样本：还没瘦身\nE2  # 样本：还没回填\n' > .claude/decision-links-pending
printf 'v1\n' > research/results/e1-sample.out
git add -A && git commit -qm base
# 这次改动：E1 正文改了、产物换了，同时回看两行；新判决；D1 改了一句
sed -i 's/^正文提到 D1/正文（重跑之后）提到 D1/' .claude/kb/experiments/01-样本一.md
sed -i 's/2026-09-18 不受影响：结论与定案同向/2026-09-19 不受影响：重跑后结论与定案同向/; s/2026-09-18 不受影响：只作背景提到/2026-09-19 不受影响：只作背景提到/' .claude/kb/experiments/01-样本一.md
printf 'v2\n' > research/results/e1-sample.out
sed -i 's/^\*\*定案\*\*：样本规则。/**定案**：样本规则（收严一处）。/' .claude/kb/decisions/01-甲.md
cat > research/prompts/sample-r1-main-verification.md <<'EOF'
# sample-r1 判决

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D1（甲） 已定项 1 | 支撑 | 2026-09-19 改了 |
EOF
