#!/usr/bin/env bash
# 门禁 crash-point-replay「键与跑的环境同一份」样本的现场：样本拷进临时目录之后，假工具补上执行位，
# git init，这一版连 HEAD 一起提交（判定库放 git common-dir，单元键要 git 列文件）。没有上一版：不带特性那一趟现判、记库，带特性那一趟的键要照清掉调用方变量之后的环境算。
set -euo pipefail
chmod +x .crash-point-replay-sample-tools/cargo .crash-point-replay-sample-tools/rustc
git init -q .
git add -A
git -c user.name=sample -c user.email=sample@example.invalid commit -q -m "样本：这一版"
