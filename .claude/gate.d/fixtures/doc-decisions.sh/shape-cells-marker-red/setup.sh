#!/usr/bin/env bash
# 现场与 red 同样三格都红：决策首行状态翻了、已提交没推出去、变更史没新增条目（entry-added）；
# 新建的一份 kb 页用了日期与序号粘在一起的标题（shape）；D1 节顶上的现状行与索引表对不上（status-sync）。
# 样本根另放 .gate-cells 只点名 shape（随样本目录拷进来）：只该跑 shape 那一格，汇总只列一格。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p .claude/kb/decisions
printf '%s\n' '## D1 样本决策 —— 已定' '第一行。' '第二行。' '## 历史版本' > .claude/kb/decisions/01-样本决策.md
printf '%s\n' '# 设计决策记录' '' '| 决策 | 状态 | 结论（简报） | 正文 |' '|---|---|---|---|' \
  '| D1（样本决策） | 已定 1 项 / 未定 0 项 | 块取 16 KiB | [01-样本决策.md](decisions/01-样本决策.md) |' > .claude/kb/decisions.md
printf '%s\n' '# 决策变更史' '' '## D1（样本决策）' '' '**现状**：已定 0 项 / 未定 1 项。块取 16 KiB' '' '## 历史版本' > .claude/kb/decisions-history.md
git add -A && git commit -qm base
git init -q --bare "$PWD/.git/fixture-upstream.git"
git remote add origin "$PWD/.git/fixture-upstream.git"
git push -q -u origin HEAD
sed -i 's/^## D1 样本决策 —— 已定$/## D1 样本决策 —— 半定（一项未定）/' .claude/kb/decisions/01-样本决策.md
git commit -qam '翻了状态，没写变更史，也还没推出去'
printf '%s\n' '# 不变量' '' '## 历史版本' '' '### 2026-09-28（其一）：新写的一条还是旧形态' > .claude/kb/invariants.md
