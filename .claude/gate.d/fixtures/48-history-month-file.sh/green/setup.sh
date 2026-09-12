#!/usr/bin/env bash
# 同样三条条目，各住在日期对得上的那一份；decisions-history.md 只放说明。
set -e
mkdir -p .claude/kb
printf '# 决策变更史\n\n## 条目住在哪\n\n说明。\n\n## 历史版本\n\n- 2026-09-12：拆档。\n' > .claude/kb/decisions-history.md
printf '# 决策变更史 · 2026-09\n\n## 历史版本\n\n### 2026-09-12（其一）：住对了\n\n### 2026-09-01：住对了\n' > .claude/kb/2026-09-decisions-history.md
printf '# 决策变更史 · 2026-08\n\n## 历史版本\n\n### 2026-08-31：住对了\n' > .claude/kb/2026-08-decisions-history.md
