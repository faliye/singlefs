#!/usr/bin/env bash
# 同样三条条目，各住在日期对得上的那一份；decisions-history.md 只放快查表。
# 决策正文文末只放指路；正文里带日期的小节标题（论证的轮次）不算条目。
set -e
mkdir -p .claude/kb/decisions-history .claude/kb/decisions
printf '# 决策变更史\n\n## 按日期\n\n### 2026-09\n\n说明。\n\n## 历史版本\n\n- 2026-09-12：拆档。\n' > .claude/kb/decisions-history.md
printf '# 决策变更史 · 2026-09\n\n## 历史版本\n\n### 2026-09-12（其一）：住对了\n\n### 2026-09-01：住对了\n' > .claude/kb/decisions-history/2026-09.md
printf '# 决策变更史 · 2026-08\n\n## 历史版本\n\n### 2026-08-31：住对了\n' > .claude/kb/decisions-history/2026-08.md
printf '# 设计决策记录\n\n## 历史版本\n\ndecisions.md 与全部决策的变更史，原文按月住在 decisions-history/<年-月>.md。\n' > .claude/kb/decisions.md
printf '## D99 样本 —— 已定\n\n### 2026-09-08 三轮对抗论证\n\n正文里带日期的小节标题。\n\n## 历史版本\n\nD99（样本）的历史条目集中在 decisions-history.md。\n' > .claude/kb/decisions/99-样本.md
