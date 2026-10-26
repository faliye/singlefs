#!/usr/bin/env bash
# history-loss 搬迁没登记的样本：现场同 history-loss-moved-commit-ref-green（「依据」那一行换成了 tools/new-rig），
# 只是 .claude/gate.d/path-moves.tsv 里没有这一行搬迁（只有注释）：基准那一版不换算，「依据」那一行在改后找不到，判红。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p .claude/kb .claude/gate.d tools/old-rig records
printf '#!/bin/sh\nexit 0\n' > tools/old-rig/run.sh
printf '# 实验装置目录改名\n' > records/2026-10-23-装置目录改名.md
cat > .claude/kb/decisions-history.md <<'X'
# 决策变更史

## D1（样本决策）

### 2026-10-20

- 已定项 1 定案：节点 32 KiB
  - **改前**：`git show 31e47115:tools/old-rig/run.sh` 里是 16 KiB，产物 `e5-sample.out`。
  - **依据**：`tools/old-rig/run.sh` 的产物 `e5-sample.out` 量到 1.7 倍。

## 历史版本
X
git add -A && git commit -qm base
git mv tools/old-rig tools/new-rig
sed -i 's#^  - \*\*依据\*\*：`tools/old-rig/run.sh`#  - **依据**：`tools/new-rig/run.sh`#' .claude/kb/decisions-history.md
printf '# 搬迁登记：这一份样本里没登记 tools/old-rig 的搬迁\n' > .claude/gate.d/path-moves.tsv
