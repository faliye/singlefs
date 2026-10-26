#!/usr/bin/env bash
# checker-tier-crash-point-replay 快档 crash-case-replay 格的样本现场：样本拷进临时目录之后在那里 git init（快档按条复用的判定库放 git common-dir，单元键要 git 列文件），
# 假工具补上执行位（拷的时候丢了也不碍事）
set -euo pipefail
git init -q .
chmod +x .crash-point-replay-sample-tools/cargo .crash-point-replay-sample-tools/rustc
