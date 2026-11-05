#!/usr/bin/env bash
# 从 mutation-tables-reachability-green 整份拷来，crates/mutations.tsv 只多第 3 行：改 never_called_by_the_test，点名的测试 reaches_the_counted_function 的闭包里没有它，
# static-check 报一行 UNREACHABLE（closure），mutation-tables 这一格判红。git 现场照绿样本那一份建。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
git add -A
git commit -qm '样本基准：两处存量违规各登记一行豁免'
