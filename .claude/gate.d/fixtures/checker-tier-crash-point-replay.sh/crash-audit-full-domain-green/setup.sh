#!/usr/bin/env bash
# checker-tier-crash-point-replay 判别力样本的现场（--full 里出口 4 的全量域）：样本拷进临时目录之后在那里 git init（--full 的日志与记录文件放 git common-dir），
# 假工具补上执行位（拷的时候丢了也不碍事）
set -euo pipefail
git init -q .
chmod +x .crash-point-replay-sample-tools/cargo .crash-point-replay-sample-tools/rustc
