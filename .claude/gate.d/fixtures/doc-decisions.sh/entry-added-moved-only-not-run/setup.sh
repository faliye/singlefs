#!/usr/bin/env bash
# entry-added 只差搬迁的样本：决策正文有 6 行点名 tools/old-rig 下的装置，这一次把 tools/old-rig 搬到 tools/new-rig（git mv），
# 决策正文里只把这 6 行的路径换成新前缀，搬迁登记在 .claude/gate.d/path-moves.tsv；变更史没新增条目。
# 与基准只差登记的搬迁，不算决策正文改了：entry-added 无对象可判，整道退 77。
# 弄坏开关 LAG_BASELINE_BREAK=moved-only-kept 下照旧算改了 12 行、没新增条目，判红。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p .claude/kb/decisions .claude/gate.d tools/old-rig records
printf '#!/bin/sh\nexit 0\n' > tools/old-rig/run.sh
printf '# 实验装置目录改名\n' > records/2026-10-23-装置目录改名.md
cat > .claude/kb/decisions/01-样本决策.md <<'X'
## D1 样本决策 —— 已定
第一行，装置在 tools/old-rig/run.sh。
第二行，复跑用 tools/old-rig/run.sh。
第三行，产物由 tools/old-rig/run.sh 写。
第四行，参数见 tools/old-rig/run.sh。
第五行，判法在 tools/old-rig/run.sh。
第六行，入口是 tools/old-rig/run.sh。
## 历史版本
本决策的历史条目集中在 decisions-history.md。
X
cat > .claude/kb/decisions-history.md <<'X'
# 决策变更史

## D1（样本决策）

## 历史版本
X
git add -A && git commit -qm base
git mv tools/old-rig tools/new-rig
sed -i 's#tools/old-rig/#tools/new-rig/#' .claude/kb/decisions/01-样本决策.md
printf 'tools/old-rig\ttools/new-rig\t2026-10-23\trecords/2026-10-23-装置目录改名.md\n' > .claude/gate.d/path-moves.tsv
