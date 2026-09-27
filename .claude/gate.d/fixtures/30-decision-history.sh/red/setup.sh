#!/usr/bin/env bash
# 建一个最小的 git 仓：先提交一份决策正文与一份空变更史，推到本地的上游，再按样本意图改动。
# 改动只把首行的状态从「已定」翻成「半定（一项未定）」：增删合计 2 行，不超过小改动的 4 行，
# 但碰了状态行，不许按小改动放行；而且这一笔已经提交、还没推出去——基准取 HEAD 就看不见它。
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
git init -q --bare "$PWD/.git/fixture-upstream.git"
git remote add origin "$PWD/.git/fixture-upstream.git"
git push -q -u origin HEAD
sed -i 's/^## D1 样本决策 —— 已定$/## D1 样本决策 —— 半定（一项未定）/' .claude/kb/decisions/01-样本决策.md
git commit -qam '翻了状态，没写变更史，也还没推出去'
