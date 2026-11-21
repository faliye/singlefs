#!/usr/bin/env bash
# 与 repository-copy-closure-exempt-family-green 同一个拆分，只是挪进的文件叫 lib/other-cells.sh（族外）：建 git 仓，基准里拷 crates 那一行在入口
# .claude/gate.d/stage-copy.sh 里、表里那一行写入口的路径，提交成基准；再放回工作区里拆好的样子。只起 git、cp、mv、sed。
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mv .claude/gate.d/stage-copy.sh .entry-now
mv .claude/gate.d/repository-copy-closure-exempt.tsv .table-now
cp .claude/gate.d/lib/other-cells.sh .claude/gate.d/stage-copy.sh
mv .claude/gate.d/lib/other-cells.sh .part-now
sed 's#\.claude/gate\.d/lib/other-cells\.sh#.claude/gate.d/stage-copy.sh#' .table-now > .claude/gate.d/repository-copy-closure-exempt.tsv
git add -A .claude
git commit -qm base
mv .entry-now .claude/gate.d/stage-copy.sh
mv .part-now .claude/gate.d/lib/other-cells.sh
mv .table-now .claude/gate.d/repository-copy-closure-exempt.tsv
