#!/usr/bin/env bash
# Z12 第二遍：第一遍里建池阶段就撞分配记录墙的 21 个组合，改成 A 之后只覆盖写到 txg 4；另把 3 个组合跑 12 轮。
set -u
BIN=$(ls -t /tmp/claude-1000/m2-final-code-r2-opus/target/debug/deps/r2_opus_z12-* | grep -v '\.d$' | head -1)
OUT=/tmp/claude-1000/m2-final-code-r2-opus/logs/z12b
mkdir -p "$OUT"
run_one() {
  caps=$1; last=$2; rounds=$3
  name=$(echo "$caps" | tr ',' '_')
  SINGLEFS_R2_OPUS_CAPS=$caps R2_OPUS_LAST_TXG=$last R2_OPUS_ROUNDS=$rounds R2_OPUS_INODES_EVERY=2 RUST_TEST_THREADS=2 nice -n 19 "$BIN" --nocapture --test-threads 2 z12_with_file > "$OUT/caps_${name}_t${last}_r${rounds}.log" 2>&1
  echo "caps=$caps last=$last rounds=$rounds rc=$?"
}
export -f run_one; export BIN OUT
{
for c in 1,2,1,2 1,2,1,3 1,2,2,2 1,2,2,3 1,2,3,2 1,2,3,3 1,2,4,2 1,2,4,3 1,3,1,2 1,3,1,3 1,3,2,2 1,3,2,3 1,3,3,2 1,3,3,3 1,3,4,2 2,2,1,2 2,2,1,3 2,3,1,2 2,3,1,3 3,2,1,2 3,2,1,3; do echo "$c 4 7"; done
for c in 5,3,4,3 3,3,4,3 5,2,3,2; do echo "$c 10 14"; done
} | xargs -P 4 -L 1 bash -c 'run_one "$0" "$1" "$2"'
echo done
