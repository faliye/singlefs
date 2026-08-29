#!/usr/bin/env bash
# 建一个最小的 git 仓：先提交一份决策正文与一份空变更史，再按样本意图改动。
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
printf '# 决策变更史\n\n## 历史版本\n' > .claude/kb/decisions-history.md
printf '# 设计决策记录\n\n## 历史版本\n' > .claude/kb/decisions.md
git add -A && git commit -qm base
# 改决策正文 —— 超过「小改动」的 4 行阈值
sed -i 's/^第一行。/第一行（改过）。\n新增一行。\n再新增一行。/' .claude/kb/decisions/01-样本决策.md
sed -i 's/^第三行。/第三行（也改过）。\n又一行。/' .claude/kb/decisions/01-样本决策.md
printf '\n### 2026-08-29\n- 曾经 X / 现在 Y / 依据 Z\n' >> .claude/kb/decisions-history.md
