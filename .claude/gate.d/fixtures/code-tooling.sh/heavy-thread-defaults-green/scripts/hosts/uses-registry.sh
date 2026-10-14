#!/usr/bin/env bash
# 样本：经登记位取；注释里的 nproc 与 capped.sh 16 不点名
source "$(dirname "${BASH_SOURCE[0]}")/heavy-threads.sh"
threads="$(heavy_thread_default)"
echo "不按 nproc 取，也不写 capped.sh 16：$threads"
SAMPLE_COMMAND="bash scripts/hosts/capped.sh 4 cargo build"
bash scripts/hosts/capped.sh default cargo build
