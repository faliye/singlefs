#!/usr/bin/env bash
# 腿自己的 N2 实验：把草稿副本里 cargo test --no-run 编出的 harness 档测试二进制逐个直接跑（工作目录是包目录），
# 记每个二进制的 test result 行与 RECLAIM_AT_RECORD 行（草稿副本的 core 在记根回收出东西时 eprintln）。
# 用法（整条脚本经 run-with-memory-cap.sh 跑）：run_binaries_chunk.sh <模式 post|pre> <起> <止> <草稿副本根> <列表文件> <输出目录>
#   post = 第 31 批合入后的行为；pre = 环境变量 SIM_PRE31=1：草稿副本的 core 在测试 / 装置 / on_device_modes 调 install_root_ring_occupancy 时不装、
#   记根没装表时静默返回（还原第 31 批之前的行为；core 自己 allocator_after_make_filesystem 与挂载那几处照装）。
set -uo pipefail
mode="$1"; first="$2"; last="$3"; tree="$4"; list="$5"; out="$6"
mkdir -p "$out/$mode"
index=0
while IFS= read -r binary; do
  index=$((index + 1))
  [[ $index -lt $first || $index -gt $last ]] && continue
  name="$(basename "$binary" | sed 's/-[0-9a-f]\{16\}$//')"
  log="$out/$mode/$index-$name.log"
  if [[ "$mode" == pre ]]; then export SIM_PRE31=1; else unset SIM_PRE31; fi
  ( cd "$tree/crates/singlefs-harness" && timeout 150 "$binary" > "$log" 2>&1; echo "exit=$?" >> "$log" )
done < "$list"
