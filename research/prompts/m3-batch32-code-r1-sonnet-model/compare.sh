#!/usr/bin/env bash
# 把 <工作目录>/out 里的结果整理成可比对的汇总：去掉耗时字段、排序，不依赖 SINGLEFS_REPO。用法：compare.sh <工作目录> → 打印汇总；两次复跑的汇总用 diff 比。
set -euo pipefail
OUT="${1:?用法：compare.sh <工作目录>}/out"
{
  sed -E 's/; finished in [0-9.]+s//; s/; [0-9]+ filtered out//' "$OUT/baseline.txt"
  cat "$OUT/probe.txt"
  cat "$OUT/i49.txt"
  cat "$OUT/mutations.txt"
  cat "$OUT/mutations-broad.txt"
  cat "$OUT/extra.txt"
  cat "$OUT/model.txt"
  cat "$OUT/anchors.txt"
} | LC_ALL=C sort
