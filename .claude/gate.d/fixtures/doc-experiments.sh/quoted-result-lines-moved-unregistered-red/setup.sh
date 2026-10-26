#!/usr/bin/env bash
# quoted-result-lines 搬迁没登记的样本：现场同 quoted-result-lines-moved-history-green（历史节里那一行换成了 tools/new-rig，正文新抄一行对的），
# 只是 .claude/gate.d/path-moves.tsv 里没有这一行搬迁（只有注释）：历史节第 12 行算新增、照判，在登记着的产物里找不到，判红并只点名这一行。
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
printf '# 搬迁登记：这一份样本里没登记 tools/old-rig 的搬迁\n' > .claude/gate.d/path-moves.tsv
