#!/usr/bin/env bash
# history-loss 搬迁与提交引用的样本：tools/old-rig 搬到 tools/new-rig（git mv），搬迁登记在 .claude/gate.d/path-moves.tsv。
# 变更史 D1 节一块里两行产物行：「改前」那一行写 `git show 31e47115:tools/old-rig/run.sh`（指那个提交里的文件，有意留旧路径），
# 「依据」那一行写 `tools/old-rig/run.sh`；这一次只把「依据」那一行换成新路径。
# 基准那一版照登记换算时「<提交哈希>:」后面的路径不换，换算之后与现在逐字相同：判绿。
# 弄坏开关 LAG_BASELINE_BREAK=commit-ref-rewritten 下基准那一版连提交引用也换成新路径，「改前」那一行在改后找不到，判红。
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
printf 'tools/old-rig\ttools/new-rig\t2026-10-23\trecords/2026-10-23-装置目录改名.md\n' > .claude/gate.d/path-moves.tsv
