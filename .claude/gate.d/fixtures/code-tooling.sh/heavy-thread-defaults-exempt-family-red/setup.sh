#!/usr/bin/env bash
# 与 heavy-thread-defaults-exempt-family-green 同一个拆分，只是挪进的文件叫 other-cores.sh（族外）：建 git 仓，基准里 $(nproc) 那一行在入口
# scripts/new-rig/report_cores.sh 里、表里那一行写入口的路径，提交成基准；再放回工作区里拆好的样子。只起 git、cp、mv、sed。
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mv scripts/new-rig/report_cores.sh .entry-now
mv .claude/heavy-thread-exempt .table-now
cp scripts/new-rig/other-cores.sh scripts/new-rig/report_cores.sh
mv scripts/new-rig/other-cores.sh .part-now
sed 's#other-cores\.sh#report_cores.sh#' .table-now > .claude/heavy-thread-exempt
git add -A .claude scripts
git commit -qm base
mv .entry-now scripts/new-rig/report_cores.sh
mv .part-now scripts/new-rig/other-cores.sh
mv .table-now .claude/heavy-thread-exempt
