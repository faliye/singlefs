#!/usr/bin/env bash
# 排除表只缩不涨认拆分：建 git 仓，基准里 $(nproc) 那一行在入口 scripts/new-rig/report_cores.sh 里、表里那一行写入口的路径，提交成基准；
# 再放回工作区里拆好的样子（入口只调部分文件 report_cores-count.sh、表里那一行改成部分文件的路径）。只起 git、cp、mv、sed。
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mv scripts/new-rig/report_cores.sh .entry-now
mv .claude/heavy-thread-exempt .table-now
cp scripts/new-rig/report_cores-count.sh scripts/new-rig/report_cores.sh
mv scripts/new-rig/report_cores-count.sh .part-now
sed 's#report_cores-count\.sh#report_cores.sh#' .table-now > .claude/heavy-thread-exempt
git add -A .claude scripts
git commit -qm base
mv .entry-now scripts/new-rig/report_cores.sh
mv .part-now scripts/new-rig/report_cores-count.sh
mv .table-now .claude/heavy-thread-exempt
