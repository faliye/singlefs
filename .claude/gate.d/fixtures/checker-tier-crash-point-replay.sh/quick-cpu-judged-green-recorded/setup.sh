#!/usr/bin/env bash
# checker-tier-crash-point-replay「没开 GPU 时判段判在 CPU 的绿照记」样本的现场：样本拷进临时目录之后 git init，这一版连 HEAD 一起提交
# （判定库放 git common-dir，单元键要 git 列文件）；假工具补上执行位（拷的时候丢了也不碍事）。判定库是空的：不带特性那一趟现判、记库，带特性那一趟照键复用
set -euo pipefail
chmod +x .crash-point-replay-sample-tools/cargo .crash-point-replay-sample-tools/rustc
git init -q .
git add -A
git -c user.name=sample -c user.email=sample@example.invalid commit -q -m "样本：这一版"
