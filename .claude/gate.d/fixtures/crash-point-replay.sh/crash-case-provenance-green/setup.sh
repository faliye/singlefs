#!/usr/bin/env bash
# 门禁 crash-point-replay 样本现场（crash-case-provenance 格）：样本拷进临时目录之后在那里 git init
# （门禁取 git common-dir 放这一趟的日志目录），假工具补上执行位（拷的时候丢了也不碍事）
set -euo pipefail
git init -q .
chmod +x .crash-point-replay-sample-tools/cargo .crash-point-replay-sample-tools/rustc
