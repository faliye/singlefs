#!/usr/bin/env bash
# 照 mutation-tables-green 拷来，crates/ 换成 singlefs-verification 一个小包（放着 crates-mutation-rows.py 的选路表、截断表、豁免表点名的函数），
# crates/mutations.tsv 只一行（改 counted，点名的测试调得到它）：static-check 判可达性、一行都不报，成功句写判了 1 行。git 现场照绿样本那一份建（滞后表只缩不涨要有基线可比）。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
git add -A
git commit -qm '样本基准：两处存量违规各登记一行豁免'
