#!/usr/bin/env bash
# 门禁 crash-point-replay 逐项点名写错项的样本现场：样本拷进临时目录之后在那里 git init，假工具补上执行位
set -euo pipefail
git init -q .
chmod +x .crash-point-replay-sample-tools/cargo .crash-point-replay-sample-tools/rustc
# .gate-items 点名 feature-gated:no_such_file.rs（tests/ 里没有它）：一趟 cargo 都不起，退 2 并列出可点的 feature-gated 项
