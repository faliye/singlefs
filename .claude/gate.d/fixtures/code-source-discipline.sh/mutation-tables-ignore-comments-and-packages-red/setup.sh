#!/usr/bin/env bash
# 从 mutation-tables-ignore-comments-and-packages-green 整份拷来，crates/mutations.tsv 多第 10、11 行，各判红一样：
# 第 10 行点名的 interleaved_slow_case 真标了 #[ignore]（注释行夹在 #[test] 与 #[ignore] 之间），行里不带 --include-ignored；
# 第 11 行写 -p demo -p retry --lib，点名的 absent_from_both_packages_case 两个包里都没有。第 8、9 行照绿样本那样不判红。
# git 现场照绿样本那一份建（滞后表只缩不涨要有基线可比）。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
git add -A
git commit -qm '样本基准：两处存量违规各登记一行豁免'
