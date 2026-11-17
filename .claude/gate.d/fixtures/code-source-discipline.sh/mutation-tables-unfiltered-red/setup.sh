#!/usr/bin/env bash
# 从 mutation-tables-release-green 整份拷来，样本根多放一份 .mutation-filter-words-judged（static-check 认这个标记才判「第五段 `--` 之后没有测试名筛选词」）：
# crates/mutations.tsv 的第 2、5、6、7、8 行第五段没有筛选词（第 2 行 -p demo、第 8 行 -p demo --features gpu --release --lib 这一类），static-check 各报一行 MALFORMED，mutation-tables 这一格判红。
# git 现场照绿样本那一份建（滞后表只缩不涨要有基线可比）。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
git add -A
git commit -qm '样本基准：两处存量违规各登记一行豁免'
