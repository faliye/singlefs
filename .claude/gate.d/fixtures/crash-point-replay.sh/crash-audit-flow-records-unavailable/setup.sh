#!/usr/bin/env bash
# 门禁 crash-point-replay 样本现场（备覆盖行时 prune-server 不可用：标记目录里的 collect-unreachable）：样本拷进临时目录之后在那里 git init
# （按条复用的判定库与覆盖行存放目录放 git common-dir，单元键要 git 列文件），假工具补上执行位（拷的时候丢了也不碍事）
set -euo pipefail
git init -q .
chmod +x .crash-point-replay-sample-tools/cargo .crash-point-replay-sample-tools/rustc
