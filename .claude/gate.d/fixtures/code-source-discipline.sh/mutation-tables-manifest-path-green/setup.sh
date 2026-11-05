#!/usr/bin/env bash
# 从 mutation-tables-green 整份拷来，加了 Cargo.toml 工作区与三条判绿的行（crates/mutations.tsv 第 7 至 9 行）：
# 第 7、8 行用 --manifest-path（空格写法与等号写法）点被 exclude 掉的独立工程 crates/standalone，点名的测试在它的 src/ 与 tests/ 里；
# 第 9 行写 -p renamed-package，那个包是工作区成员、目录叫 crates/renamed-dir（包名与目录名不同，包目录要从仓根 Cargo.toml 现读，不能按 crates/<包名> 硬拼）。
# git 现场照绿样本那一份建（滞后表只缩不涨要有基线可比）。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
git add -A
git commit -qm '样本基准：两处存量违规各登记一行豁免'
