#!/usr/bin/env bash
# 同样三条条目，各住在日期对得上的那一份；decisions-history.md 只放快查表。
set -e
mkdir -p .claude/kb/decisions-history
printf '# 决策变更史\n\n## 按日期\n\n### 2026-09\n\n说明。\n\n## 历史版本\n\n- 2026-09-12：拆档。\n' > .claude/kb/decisions-history.md
printf '# 决策变更史 · 2026-09\n\n## 历史版本\n\n### 2026-09-12（其一）：住对了\n\n### 2026-09-01：住对了\n' > .claude/kb/decisions-history/2026-09.md
printf '# 决策变更史 · 2026-08\n\n## 历史版本\n\n### 2026-08-31：住对了\n' > .claude/kb/decisions-history/2026-08.md
