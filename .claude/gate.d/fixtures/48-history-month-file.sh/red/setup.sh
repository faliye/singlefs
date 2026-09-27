#!/usr/bin/env bash
# 四处住错：一条写回了放快查表的 decisions-history.md，一条 09 月的条目住进了 08 月那一份，
# 一份按旧名字放在 kb 根下的月份文件，还有一条写在了决策正文的文末；
# 另有三份按月的文件放错地方、名字却不是旧名字那一种：decisions-history-<年-月>.md、kb 根下的 <年-月>.md、decisions-history/ 底下又套一层的。
set -e
mkdir -p .claude/kb/decisions-history .claude/kb/decisions
printf '# 决策变更史\n\n## 历史版本\n\n### 2026-09-12（其一）：写回了快查表\n' > .claude/kb/decisions-history.md
printf '# 决策变更史 · 2026-08\n\n## 历史版本\n\n### 2026-09-01：住错了月份\n\n### 2026-08-31：住对了\n' > .claude/kb/decisions-history/2026-08.md
printf '# 决策变更史 · 2026-07\n\n## 历史版本\n' > .claude/kb/2026-07-decisions-history.md
printf '# 决策变更史 · 2026-06\n\n## 历史版本\n' > .claude/kb/decisions-history-2026-06.md
printf '# 决策变更史 · 2026-05\n\n## 历史版本\n' > .claude/kb/2026-05.md
mkdir -p .claude/kb/decisions-history/2026
printf '# 决策变更史 · 2026-04\n\n## 历史版本\n' > .claude/kb/decisions-history/2026/2026-04.md
printf '## D99 样本 —— 已定\n\n正文。\n\n## 历史版本\n\n### 2026-09-13\n- 新立。\n' > .claude/kb/decisions/99-样本.md
