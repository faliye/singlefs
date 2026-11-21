#!/usr/bin/env bash
# 基准里一份记录带一行旧任务号、一份记录的批号在围栏里；这一轮在旧号那一份末尾加干净的行（旧号那一行不动），往围栏里那一份的围栏里再加一行批号，
# 另放一份未跟踪的新记录，表头是「批次」的那一列写批号：判绿，基准里那一行旧号不报（成功句报放过 1 行）。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p records
cat > records/2026-09-24-样本调度.md <<'X'
# 样本调度（2026-09-24）

- 旧行：调度那一条跑完（任务 21036），基准里就有。
X
printf '%s\n' '# 围栏样本' '' '```' '那一路第 7 批交回' '```' > records/fenced.md
git add -A && git commit -qm base
cat >> records/2026-09-24-样本调度.md <<'X'
- 一句干净的新话。
X
printf '%s\n' '# 围栏样本' '' '```' '那一路第 7 批交回' '那一路第 8 批交回' '```' > records/fenced.md
printf '%s\n' '# 新记录' '' '| 批次 | 依赖 |' '|---|---|' '| 批 6 | 无 |' > records/2026-09-25-新记录.md
