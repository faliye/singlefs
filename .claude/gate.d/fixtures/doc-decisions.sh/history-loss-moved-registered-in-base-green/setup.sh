#!/usr/bin/env bash
# history-loss 登记先进基准、搬迁后做的样本：搬迁登记 tools/old-rig → tools/new-rig 写进 .claude/gate.d/path-moves.tsv、随基准那一次一起提交，
# tools/old-rig 那时还在、tools/new-rig 还没有；这一次才 git mv，变更史 D1 节一块里「依据」那一行产物行照 path-moves.md「怎么做」第 3 步换成新路径。
# lib-lag-baseline.py 的 effective_among 不认基准登记表里已有的行，show 不换；history-loss 另交 --moves-base，照「基准的树里旧前缀下有文件、新前缀下没有」再换一遍：判绿。
# 弄坏开关 DOC_DECISIONS_BREAK=history-loss-registered-moves-ignored 下不交 --moves-base，「依据」那一行在改后找不到，判红。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p .claude/kb .claude/gate.d tools/old-rig records
printf '#!/bin/sh\nexit 0\n' > tools/old-rig/run.sh
printf '# 实验装置目录改名\n' > records/2026-10-23-装置目录改名.md
printf 'tools/old-rig\ttools/new-rig\t2026-10-23\trecords/2026-10-23-装置目录改名.md\n' > .claude/gate.d/path-moves.tsv
cat > .claude/kb/decisions-history.md <<'Y'
# 决策变更史

## D1（样本决策）

### 2026-10-20

- 已定项 1 定案：节点 32 KiB
  - **依据**：`tools/old-rig/run.sh` 的产物 `e5-sample.out` 量到 1.7 倍。

## 历史版本
Y
git add -A && git commit -qm base
git mv tools/old-rig tools/new-rig
sed -i 's#^  - \*\*依据\*\*：`tools/old-rig/run.sh`#  - **依据**：`tools/new-rig/run.sh`#' .claude/kb/decisions-history.md
grep -qF '`tools/new-rig/run.sh`' .claude/kb/decisions-history.md
