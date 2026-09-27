#!/usr/bin/env bash
# 54 号判别力样本的现场：样本拷进临时目录之后在那里 git init（崩溃枚举用例的指纹要 git 列文件，全绿标记与日志放 git common-dir），
# 假工具补上执行位（拷的时候丢了也不碍事）
set -euo pipefail
git init -q .
chmod +x .layer0-sample-tools/cargo .layer0-sample-tools/rustc
