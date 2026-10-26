#!/usr/bin/env bash
# checker-tier-crash-point-replay 产品 GPU 包那一格的样本现场：样本拷进临时目录之后在那里 git init（快档按条复用的判定库放 git common-dir，单元键要 git 列文件），
# 假工具补上执行位（拷的时候丢了也不碍事）。
set -euo pipefail
chmod +x .crash-point-replay-sample-tools/cargo .crash-point-replay-sample-tools/rustc
git init -q .
printf '%s\n' .unit-verdict-sample/ >> .git/info/exclude
git add -A
git -c user.name=sample -c user.email=sample@example.invalid commit -q -m "样本：这一版"
