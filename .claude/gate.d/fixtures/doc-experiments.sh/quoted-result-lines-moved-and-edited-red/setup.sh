#!/usr/bin/env bash
# quoted-result-lines 搬迁登记了、但除了路径还改了别处的样本：现场同 quoted-result-lines-moved-history-green（搬迁登记了），
# 历史节里那一行除了换路径还多写了一段 rounds=2：不只差搬迁，第 12 行算新增、照判，在登记着的产物里找不到，判红并只点名这一行。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p tools/old-rig records
printf '#!/bin/sh\nexit 0\n' > tools/old-rig/run.sh
printf '# 实验装置目录改名\n' > records/2026-10-23-装置目录改名.md
git add -A
git commit -qm '基准：历史节里早先抄下一行点名旧装置路径的产物行'
git mv tools/old-rig tools/new-rig
sed -i 's/^E7RESULT name=verdict arm=a count=3$/E7RESULT name=verdict arm=a count=3\nE7RESULT name=done emitted=2/; s#tools/old-rig#tools/new-rig#g; s#^E7RESULT name=rig source=tools/new-rig/run.sh$#E7RESULT name=rig source=tools/new-rig/run.sh rounds=2#' .claude/kb/experiments/99-样本.md
mkdir -p .claude/gate.d
printf 'tools/old-rig\ttools/new-rig\t2026-10-23\trecords/2026-10-23-装置目录改名.md\n' > .claude/gate.d/path-moves.tsv
