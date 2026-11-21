#!/usr/bin/env bash
# 滞后表「只缩不涨」要一个基准：建 git 仓（判法的判定库也按 git common-dir 取名），提交一版只有表头注释的滞后表，再把那一行加回来（工作区比基准多一行）。只起 git、head。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
cp .claude/gate.d/crate-cross-references-lag.tsv .grown-table
head -n 2 .grown-table > .claude/gate.d/crate-cross-references-lag.tsv
git add -A crates .claude .crate-cross-references-sample
git commit -qm '样本基准：滞后表没有数据行'
mv .grown-table .claude/gate.d/crate-cross-references-lag.tsv
