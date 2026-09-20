#!/usr/bin/env bash
# 本该判红，一份样本放十三种坏法，各有一条 want 钉住：没有表；关系不认得；正文提到的决策没有行；指不到的分项；
# 页内历史比回看新；experiments-history 比回看新；支撑而依据没引；依据引了而实验页那一行不是支撑；
# 待回填清单新加一行；正文改了没回看；产物变了没回看；新判决没有回看决策；写了改了而决策文件没动。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/kb/decisions .claude/kb/experiments research/results research/prompts
cat > .claude/kb/decisions/01-甲.md <<'EOF'
## D1 甲 —— 已定

### 已定项

| # | 分项 | 定案 |
|---|---|---|
| 1 | 样本分项 | 一句话定案 |

#### 已定项 1：样本分项

**定案**：样本规则。

**依据**：E5（样本实验五） 证明了样本规则成立。

## 历史版本
EOF
cat > .claude/kb/decisions/03-丙.md <<'EOF'
## D3 丙 —— 已定（一条全定；2026-09-13 用户定案）

### 已定项

| # | 分项 | 定案 |
|---|---|---|
| 1 | 膨胀分项 | **已定（2026-09-13，用户定案）**：一句话 |

#### 已定项 1（2026-09-13，用户定案）：膨胀分项

**定案**：样本规则。

**射程**：只管样本。

**依据**：用户定案，没引实验。

**欠**：无

## 历史版本
EOF
cat > .claude/kb/decisions/02-乙.md <<'EOF'
## D2 乙 —— 已定

## 历史版本
EOF
page() {   # 实验号 简称 正文 表行（可以为空，为空就不写表） 页内历史日期
  local number="$1" name="$2" text="$3" rows="$4" dated="$5"
  {
    printf '## E%s %s —— 已跑\n\n%s\n\n' "$number" "$name" "$text"
    if [[ -n "$rows" ]]; then printf '### 影响的决策\n\n| 决策分项 | 关系 | 回看 |\n|---|---|---|\n%s\n\n' "$rows"; fi
    printf '## 历史版本\n\n'
    if [[ -n "$dated" ]]; then printf '### %s\n- 样本\n' "$dated"; fi
  } > ".claude/kb/experiments/0${number}-样本${number}.md"
}
page 1 样本实验一 '提到 D1（甲） 与 D2（乙）。' $'| D1（甲） 已定项 1 | 相关 | 2026-09-19 不受影响：样本理由 |\n| D1（甲） 已定项 9 | 不影响 | 2026-09-19 不受影响：样本理由 |' ''
page 2 样本实验二 '提到 D1（甲）。' '| D1（甲） 已定项 1 | 不影响 | 2026-09-19 不受影响：样本理由 |' ''
page 3 样本实验三 '提到 D1（甲）。' '' ''
page 4 样本实验四 '提到 D1（甲）。' '| D1（甲） 已定项 1 | 支撑 | 2026-09-01 不受影响：样本理由 |' '2026-09-10'
page 5 样本实验五 '提到 D1（甲）。' '| D1（甲） 已定项 1 | 不影响 | 2026-09-19 不受影响：样本理由 |' ''
page 7 样本实验七 '提到 D1（甲）。' '| D1（甲） | 不影响 | 2026-09-10 不受影响：样本理由 |' ''
page 8 样本实验八 '提到 D1（甲）。' '| D1（甲） | 不影响 | 2026-09-19 不受影响：样本理由 |' ''
cat > .claude/kb/experiments-history.md <<'EOF'
# 实验变更史

## 历史版本

### 2026-09-15：E7（样本实验七） 重跑
- 样本
EOF
printf 'E6  # 样本：基准里没有这一行也不该有\n' > .claude/decision-links-pending
git add -A && git commit -qm base
# 这次改动
printf 'E6  # 样本：基准里没有这一行也不该有\nE9  # 新加的一行，清单只减不增\n' > .claude/decision-links-pending
sed -i 's/^提到 D1（甲）。$/提到 D1（甲），重跑之后改了一句。/' .claude/kb/experiments/02-样本2.md
sed -i 's/| D1（甲） 已定项 1 | 相关 | 2026-09-19 不受影响：样本理由 |/| D1（甲） 已定项 1 | 相关 | 2026-09-19 改了 |/' .claude/kb/experiments/01-样本1.md
printf 'v1\n' > research/results/e8-sample.out
printf '# sample-r1 判决\n\n没有回看决策那一节。\n' > research/prompts/sample-r1-main-verification.md
