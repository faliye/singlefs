#!/usr/bin/env bash
# 拿一次复跑的 summary.txt 比本目录 baseline/summary.txt（两边都已排好序、去掉耗时）。不读 SINGLEFS_REPO。
# 用法：compare.sh <复跑的工作目录>；一致退 0 并打 SAME，不一致退 1 并打 diff。
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
if diff -u "$here/baseline/summary.txt" "${1:?复跑的工作目录}/summary.txt"; then
  echo "SAME $(wc -l < "$here/baseline/summary.txt") 行"
else
  exit 1
fi
