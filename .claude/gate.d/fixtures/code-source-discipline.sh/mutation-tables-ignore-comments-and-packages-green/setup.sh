#!/usr/bin/env bash
# 从 mutation-tables-green 整份拷来，crates/mutations.tsv 多第 8、9 行：第 8 行点名的 commented_ignore_case 只在注释里写着 #[ignore]、
# 本身没标，行里不带 --include-ignored；第 9 行写 -p demo -p retry --lib，点名的 retry_rounds_pass_through 只在 retry 包里。两行都判绿。
# git 现场照绿样本那一份建（滞后表只缩不涨要有基线可比）。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
git add -A
git commit -qm '样本基准：两处存量违规各登记一行豁免'
