#!/usr/bin/env bash
# checker-tier-crash-point-replay 「RocksDB 取值没判过不起 cargo」样本的现场：假 cargo、假 rustc 补上执行位（拷的时候丢了也不碍事）。
# 阶段在起 cargo 之前那一步（rocksdb-prebuilt.py entry）退 1，用不到 git 仓；弄坏开关 rocksdb-entry-unchecked 打开时往下走，输出里没有 entry 那一句。
set -euo pipefail
chmod 755 .crash-point-replay-sample-tools/cargo .crash-point-replay-sample-tools/rustc
