#!/usr/bin/env bash
# 门禁 crash-point-replay 带特性那一格逐项点名的样本现场：样本拷进临时目录之后在那里 git init（按条复用的判定库放 git common-dir，单元键要 git 列文件），
# 假工具补上执行位（拷的时候丢了也不碍事）
set -euo pipefail
git init -q .
chmod +x .crash-point-replay-sample-tools/cargo .crash-point-replay-sample-tools/rustc
# 点名 feature-gated:store_only.rs（.gate-items）：只编、只跑 tests/store_only.rs，不带 --lib，特性照整格那一组；
# 它的 ITEM_DONE 与 features-green 整格那一趟里同一项那一行逐字相同（点名单跑一项与它在整格里跑的判定相同）
