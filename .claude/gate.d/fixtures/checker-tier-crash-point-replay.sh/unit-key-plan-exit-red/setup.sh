#!/usr/bin/env bash
# checker-tier-crash-point-replay 按条复用失灵样本的现场：假工具补上执行位，git init 并提交这一版（判定库放 git common-dir）；
# .unit-verdict-sample/plan-failure 是样本里放好的（admission.py 文件头「按条复用失灵」），这里不造上一版。
set -euo pipefail
chmod +x .crash-point-replay-sample-tools/cargo .crash-point-replay-sample-tools/rustc
git init -q .
printf '%s\n' .unit-verdict-sample/ >> .git/info/exclude
git add -A
git -c user.name=sample -c user.email=sample@example.invalid commit -q -m "样本：这一版"
