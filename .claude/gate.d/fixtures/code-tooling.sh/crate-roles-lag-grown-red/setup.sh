#!/usr/bin/env bash
# 滞后表「只缩不涨」要一个基准：建 git 仓，提交一版不带 e8_moving_out 那一行的滞后表，再把那一行加回来（工作区比基准多一行）。只起 git、grep。
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
cp .claude/gate.d/crate-roles-lag.tsv .grown-table
grep -v 'e8_moving_out' .grown-table > .claude/gate.d/crate-roles-lag.tsv
git add -A .claude crates tools Cargo.toml
git commit -qm base
mv .grown-table .claude/gate.d/crate-roles-lag.tsv
