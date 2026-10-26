#!/usr/bin/env bash
# 从 mutation-tables-green 整份拷来，crates/mutations.tsv 只多第 8 行：替换文写 halved_before（哪里都没有定义，照拆分提案第六十一节那样被删掉的标识符），
# static-check 报一行 UNDEFINED，mutation-tables 这一格判红。git 现场照绿样本那一份建（滞后表只缩不涨要有基线可比）。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
git add -A
git commit -qm '样本基准：两处存量违规各登记一行豁免'
