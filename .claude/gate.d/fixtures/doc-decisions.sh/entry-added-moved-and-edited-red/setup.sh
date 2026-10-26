#!/usr/bin/env bash
# entry-added 搬迁登记了、但除了路径还改了别处的样本：现场同 entry-added-moved-only-not-run（搬迁登记在 .claude/gate.d/path-moves.tsv），
# 另把决策正文第一行的说法改了：这份决策不只差搬迁，照算改了（6 行换路径加 1 行改写，共 14 行），变更史没新增条目，判红。
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
块取 16 KiB。
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
sed -i 's#tools/old-rig/#tools/new-rig/#; s#^块取 16 KiB。#块取 32 KiB。#' .claude/kb/decisions/01-样本决策.md
printf 'tools/old-rig\ttools/new-rig\t2026-10-23\trecords/2026-10-23-装置目录改名.md\n' > .claude/gate.d/path-moves.tsv
