#!/usr/bin/env bash
# 用法：run_base.sh <测试二进制> [过滤…]  在草稿副本（未改）上跑一个 harness 档测试二进制，打结果行。
set -u
S=/tmp/claude-1000/m3-unlink-r2-forward
bin=$1; shift
cd "$S/tree"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="正推腿 m3-unlink-r2-forward 与别的派出去的 agent 各跑各的，只跑 harness 档单个测试二进制"
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS=150
CARGO_TARGET_DIR=$S/target nice -n 19 bash /home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh "${CAP:-2G}" \
  bash /home/fy5090/code/singlefs/research/scripts/capped.sh 6 \
  cargo test --offline -p singlefs-harness --test "$bin" -- "$@" > "$S/base-$bin.log" 2>&1
rc=$?
echo "base bin=$bin rc=$rc"
grep -E '^test |^test result|panicked' "$S/base-$bin.log" | head -20
