#!/usr/bin/env bash
# shape 只差搬迁的样本：决策变更史里一块存量日期块住在节外（上一级标题是「## 杂项」，基准里就有，只计数），
# 块里一行点名 tools/old-rig/run.sh；这一次把 tools/old-rig 搬到 tools/new-rig（git mv）并登记在 .claude/gate.d/path-moves.tsv，
# 那一行只换了路径。只因登记的搬迁才变了的行不算新增，那一块仍是存量：shape 绿，存量「条目住在节外 1 处」。
# 弄坏开关 LAG_BASELINE_BREAK=moved-only-kept 下那一行算新增，那一块按新写的判，红在「没住进自己的节」。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p .claude/kb .claude/gate.d tools/old-rig records
printf '#!/bin/sh\nexit 0\n' > tools/old-rig/run.sh
printf '# 实验装置目录改名\n' > records/2026-10-23-装置目录改名.md
cat > .claude/kb/decisions-history.md <<'X'
# 决策变更史

## 杂项

### 2026-10-20

- 装置在 tools/old-rig/run.sh

## D1（样本决策）

### 2026-10-21

- 正常的一条

## 历史版本
X
git add -A && git commit -qm base
git mv tools/old-rig tools/new-rig
sed -i 's#^- 装置在 tools/old-rig/run.sh$#- 装置在 tools/new-rig/run.sh#' .claude/kb/decisions-history.md
printf 'tools/old-rig\ttools/new-rig\t2026-10-23\trecords/2026-10-23-装置目录改名.md\n' > .claude/gate.d/path-moves.tsv
