#!/usr/bin/env bash
# shape 搬迁没登记的样本：现场同 shape-moved-only-green（存量节外日期块里那一行换成了 tools/new-rig），
# 只是 .claude/gate.d/path-moves.tsv 里没有这一行搬迁（只有注释）：那一行算新增，那一块按新写的判，红在「没住进自己的节」。
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
printf '# 搬迁登记：这一份样本里没登记 tools/old-rig 的搬迁\n' > .claude/gate.d/path-moves.tsv
