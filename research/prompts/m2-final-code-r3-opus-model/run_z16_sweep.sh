#!/usr/bin/env bash
# Z16 扫描：给定盘宽与参数，把单元区每个槽分 8 段并行跑 z16_sweep_read_fault_on_one_slot（每段各自顺序跑，stderr 各进各的日志）。
# 用法：run_z16_sweep.sh <日志目录> <单元区槽数> <盘号>；其余参数走环境变量（R3_OPUS_WIDTH、R3_OPUS_OVERWRITES、R3_OPUS_ROLLBACK_INDEX）。
set -euo pipefail
LOGS=${1:?}; SLOTS=${2:?}; DEVICE=${3:?}
BIN=${R3_OPUS_Z16_BIN:?}
mkdir -p "$LOGS"
FROM=50176; TO=$((50176 + SLOTS)); STEP=$(( (SLOTS + 7) / 8 ))
for shard in 0 1 2 3 4 5 6 7; do
  a=$((FROM + shard * STEP)); b=$((a + STEP)); [ "$b" -gt "$TO" ] && b=$TO
  [ "$a" -ge "$TO" ] && continue
  R3_OPUS_DEVICE=$DEVICE R3_OPUS_SLOT_FROM=$a R3_OPUS_SLOT_TO=$b SINGLEFS_R3_OPUS_REPORT=1 \
    nice -n 19 "$BIN" --exact z16_sweep_read_fault_on_one_slot --nocapture > "$LOGS/shard$shard.log" 2>&1 &
done
wait
cat "$LOGS"/shard*.log | grep -c 'R3OPUS-RUN'
