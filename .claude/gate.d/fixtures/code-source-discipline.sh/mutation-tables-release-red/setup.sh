#!/usr/bin/env bash
# 从 mutation-tables-green 整份拷来，crates/mutations.tsv 只多第 8 行：-p demo --features gpu --lib 带 gpu 特性、第五段没写 --release，
# static-check 报一行 RELEASE，mutation-tables 这一格判红。git 现场照绿样本那一份建（滞后表只缩不涨要有基线可比）。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
git add -A
git commit -qm '样本基准：两处存量违规各登记一行豁免'
