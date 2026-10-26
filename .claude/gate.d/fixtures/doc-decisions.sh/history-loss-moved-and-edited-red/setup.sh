#!/usr/bin/env bash
# history-loss 搬迁登记了、但除了路径还改了别处的样本：现场同 history-loss-moved-commit-ref-green（搬迁登记了，「依据」那一行换成了新路径），
# 另把「依据」那一行的「1.7 倍」删了：换算之后基准那一块的 1.7 在改后找不到，判红。
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
sed -i 's#^  - \*\*依据\*\*：`tools/old-rig/run.sh` 的产物 `e5-sample.out` 量到 1.7 倍。#  - **依据**：`tools/new-rig/run.sh` 的产物 `e5-sample.out`。#' .claude/kb/decisions-history.md
printf 'tools/old-rig\ttools/new-rig\t2026-10-23\trecords/2026-10-23-装置目录改名.md\n' > .claude/gate.d/path-moves.tsv
