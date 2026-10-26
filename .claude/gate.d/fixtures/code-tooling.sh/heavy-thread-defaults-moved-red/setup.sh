#!/usr/bin/env bash
# 与 heavy-thread-defaults-moved-green 同一个搬迁，只是没有搬迁登记表：建 git 仓，把 scripts/new-rig 放回搬家之前的 scripts/old-rig、
# 表里那一行写旧路径，提交成基准；再 git mv 到 scripts/new-rig、放回工作区的表（新路径）。只起 git、sed。
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
cp .claude/heavy-thread-exempt .table-new
mv scripts/new-rig scripts/old-rig
sed 's#scripts/new-rig#scripts/old-rig#g' .table-new > .claude/heavy-thread-exempt
git add -A .claude scripts
git commit -qm base
git mv scripts/old-rig scripts/new-rig
mv .table-new .claude/heavy-thread-exempt
