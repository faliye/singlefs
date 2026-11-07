#!/usr/bin/env bash
# 从 mutation-tables-green 整份拷来，crates/mutations.tsv 只多第 8 行：-p demo --features gpu --release --lib 带 gpu 特性、第五段写了 --release，
# static-check 不报 RELEASE，成功句报 release 那半句。git 现场照绿样本那一份建（滞后表只缩不涨要有基线可比）。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
git add -A
git commit -qm '样本基准：两处存量违规各登记一行豁免'
