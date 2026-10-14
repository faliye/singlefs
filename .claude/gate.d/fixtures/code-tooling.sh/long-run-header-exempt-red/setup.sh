#!/usr/bin/env bash
# 豁免表「只缩不涨」那一条要一个基准：建 git 仓，提交一版不带 grown.sh 那一行的表，再把那一行加回来（工作区比基准多一行）。
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
cp .claude/long-run-live-hosts-exempt .grown-table
grep -v 'grown.sh' .grown-table > .claude/long-run-live-hosts-exempt
git add -A .claude scripts
git commit -qm base
mv .grown-table .claude/long-run-live-hosts-exempt
