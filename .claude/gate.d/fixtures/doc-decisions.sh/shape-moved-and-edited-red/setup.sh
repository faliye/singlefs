#!/usr/bin/env bash
# shape 搬迁登记了、但除了路径还改了别处的样本：现场同 shape-moved-only-green（搬迁登记了），
# 存量节外日期块里那一行除了换路径还补了半句：那一行不只差搬迁，算新增，那一块按新写的判，红在「没住进自己的节」。
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
sed -i 's#^- 装置在 tools/old-rig/run.sh$#- 装置在 tools/new-rig/run.sh，参数改成两轮#' .claude/kb/decisions-history.md
printf 'tools/old-rig\ttools/new-rig\t2026-10-23\trecords/2026-10-23-装置目录改名.md\n' > .claude/gate.d/path-moves.tsv
