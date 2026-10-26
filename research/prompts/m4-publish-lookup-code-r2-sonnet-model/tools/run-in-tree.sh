#!/usr/bin/env bash
# 在一棵 crates 副本上跑一条 cargo test，包线程上限与内存上限，原样输出写进日志。
# 用法：run-in-tree.sh <仓根> <副本> <编译目录> <日志> <范围一句话> <cargo test 的参数…>
set -uo pipefail
repo=$1; tree=$2; target=$3; log=$4; scope=$5; shift 5
cd "$tree" || exit 2
CARGO_TARGET_DIR=$target SINGLEFS_RUN_SCOPE="$scope" nice -n 19 bash "$repo/scripts/hosts/capped.sh" 31 \
  bash "$repo/scripts/hosts/run-with-memory-cap.sh" 16G cargo test --offline "$@" > "$log" 2>&1
echo "exit=$?" >> "$log"
tail -3 "$log"
