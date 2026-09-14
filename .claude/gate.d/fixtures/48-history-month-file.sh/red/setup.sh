#!/usr/bin/env bash
# 四处住错：一条写回了放快查表的 decisions-history.md，一条 09 月的条目住进了 08 月那一份，
# 一份按旧名字放在 kb 根下的月份文件，还有一条写在了决策正文的文末。
set -e
mkdir -p .claude/kb/decisions-history .claude/kb/decisions
printf '# 决策变更史\n\n## 历史版本\n\n### 2026-09-12（其一）：写回了快查表\n' > .claude/kb/decisions-history.md
printf '# 决策变更史 · 2026-08\n\n## 历史版本\n\n### 2026-09-01：住错了月份\n\n### 2026-08-31：住对了\n' > .claude/kb/decisions-history/2026-08.md
printf '# 决策变更史 · 2026-07\n\n## 历史版本\n' > .claude/kb/2026-07-decisions-history.md
printf '## D99 样本 —— 已定\n\n正文。\n\n## 历史版本\n\n### 2026-09-13\n- 新立。\n' > .claude/kb/decisions/99-样本.md
