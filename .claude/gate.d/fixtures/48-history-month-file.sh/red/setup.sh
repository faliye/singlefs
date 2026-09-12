#!/usr/bin/env bash
# 两处住错：一条写回了只放说明的 decisions-history.md，一条 09 月的条目住进了 08 月那一份。
set -e
mkdir -p .claude/kb
printf '# 决策变更史\n\n## 历史版本\n\n### 2026-09-12（其一）：写回了说明文件\n' > .claude/kb/decisions-history.md
printf '# 决策变更史 · 2026-08\n\n## 历史版本\n\n### 2026-09-01：住错了月份\n\n### 2026-08-31：住对了\n' > .claude/kb/2026-08-decisions-history.md
