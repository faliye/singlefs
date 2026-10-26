#!/usr/bin/env bash
# quoted-result-lines 只差搬迁的样本：已跟踪的实验页历史节里早先抄下一行产物行，点名 tools/old-rig/run.sh（那份产物早已归档，登记着的产物里没有，早先就在，不判）。
# 这一次把 tools/old-rig 搬到 tools/new-rig（git mv）并登记在 .claude/gate.d/path-moves.tsv，页里的路径一起换；正文另新抄一行对的（产物里逐字找得到）。
# 历史节里只因登记的搬迁才变了的那一行不算新增，不判：判了 1 行（正文新抄的），没判的 2 行（页里第 5、12 行）。
# 弄坏开关 LAG_BASELINE_BREAK=moved-only-kept 下第 12 行算新增、照判，在登记着的产物里找不到，判红。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p tools/old-rig records
printf '#!/bin/sh\nexit 0\n' > tools/old-rig/run.sh
printf '# 实验装置目录改名\n' > records/2026-10-23-装置目录改名.md
git add -A
git commit -qm '基准：历史节里早先抄下一行点名旧装置路径的产物行'
git mv tools/old-rig tools/new-rig
sed -i 's/^E7RESULT name=verdict arm=a count=3$/E7RESULT name=verdict arm=a count=3\nE7RESULT name=done emitted=2/; s#tools/old-rig#tools/new-rig#g' .claude/kb/experiments/99-样本.md
mkdir -p .claude/gate.d
printf 'tools/old-rig\ttools/new-rig\t2026-10-23\trecords/2026-10-23-装置目录改名.md\n' > .claude/gate.d/path-moves.tsv
