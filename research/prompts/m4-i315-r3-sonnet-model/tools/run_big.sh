#!/usr/bin/env bash
# 辩方腿（m4-i315-r3）：放大档 60 步 × 300 段（种子偏移 1000 起，六片各 50 段）逐片续跑；每片写 progress.md 一行。
# 用法：bash run_big.sh <仓根> <副本根> <日志目录> <进度文件>
set -euo pipefail
repo="$1"; tree="$2"; logs="$3"; progress="$4"
for offset in 1000 1050 1100 1150 1200 1250; do
  start=$SECONDS
  RUN_SLICE_CAP=10G bash "$(dirname "${BASH_SOURCE[0]}")/run_slice.sh" "$repo" "$tree" "$logs" "big-o$offset" 60 50 "$offset" none 31 broad
  echo "$(TZ=Asia/Tokyo date +%F) 放大档片 big-o$offset 完成，耗时 $((SECONDS - start)) 秒，下一片同样量级" >> "$progress"
done
