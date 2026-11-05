#!/usr/bin/env bash
# 从 mutation-tables-manifest-path-green 整份拷来，crates/mutations.tsv 多第 11 至 18 行，各判红一样（第 7 至 10 行照绿样本那样不判红）：
# 第 11 行 -p standalone-project：包在被 exclude 掉的 crates/standalone，不是工作区成员，-p 认不出；
# 第 12 行 --manifest-path ../outside-project/Cargo.toml：解到仓根之外；
# 第 13 行 --manifest-path crates/standalone/src/lib.rs：文件在，但文件名不是 Cargo.toml；
# 第 14 行 --manifest-path crates/standalone-gone/Cargo.toml：路径在仓内、文件名对，文件不在；
# 第 15 行 -p demo 与 --manifest-path 并写；
# 第 16 行 --manifest-path 写了两次（一次空格写法、一次等号写法）；
# 第 17 行 --manifest-path 在参数串末尾、后面没有路径；
# 第 18 行 --manifest-path 指到独立工程 crates/standalone，点名的 absent_in_standalone_case 在它的 --lib 里找不到（证明包目录取的是 Cargo.toml 所在目录，测试在那里找）。
# git 现场照绿样本那一份建（滞后表只缩不涨要有基线可比）。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
git add -A
git commit -qm '样本基准：两处存量违规各登记一行豁免'
