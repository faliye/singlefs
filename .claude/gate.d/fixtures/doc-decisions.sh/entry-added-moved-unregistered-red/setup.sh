#!/usr/bin/env bash
# entry-added 搬迁没登记的样本：现场同 entry-added-moved-only-not-run（决策正文 6 行点名的 tools/old-rig 搬到 tools/new-rig，只换了这 6 行的路径），
# 只是 .claude/gate.d/path-moves.tsv 里没有这一行搬迁（只有注释）：换了路径照算决策正文改了 12 行，变更史没新增条目，判红。
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
printf '# 搬迁登记：这一份样本里没登记 tools/old-rig 的搬迁\n' > .claude/gate.d/path-moves.tsv
