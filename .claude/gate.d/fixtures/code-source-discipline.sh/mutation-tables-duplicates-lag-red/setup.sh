#!/usr/bin/env bash
# 滞后表红样本的现场：基准提交里的滞后表没有 e1_sample.tsv 那一行，提交之后换回此刻的样子，那一行就是相对基准那一版多出的一组。
set -euo pipefail
lag_table=.claude/gate.d/mutation-duplicates-lag.tsv
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
cp "$lag_table" .git/lag-table-now
grep -v $'^scripts/mutation/tables/e1_sample.tsv\t' .git/lag-table-now > "$lag_table"
git add -A
git commit -qm '样本基准：滞后表还没登记 e1_sample.tsv 那一组'
cp .git/lag-table-now "$lag_table"
