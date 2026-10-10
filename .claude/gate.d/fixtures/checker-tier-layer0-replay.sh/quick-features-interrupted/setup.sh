#!/usr/bin/env bash
# checker-tier-layer0-replay 带特性那一格的样本现场：样本拷进临时目录之后在那里 git init（快档核崩溃枚举用例的标记要 git 列文件、取 common-dir），
# 假工具补上执行位（拷的时候丢了也不碍事）
set -euo pipefail
git init -q .
chmod +x .layer0-sample-tools/cargo .layer0-sample-tools/rustc
