#!/usr/bin/env bash
# 比一次复跑的 summary.txt 与期望文件：只比判定行，排序之后逐行比；不依赖 SINGLEFS_REPO。
# 用法：compare.sh <复跑的 summary.txt> [<期望文件>]（期望文件缺省取本目录 expected/summary-<档>.txt，档由 PROFILE 给，缺省 quick）。
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
actual="${1:?要复跑的 summary.txt}"
expected="${2:-$here/expected/summary-${PROFILE:-quick}.txt}"
strip() { grep -E '^(CM|L0TALLY|CMCLASS|CMMOUNT|G|GTALLY|CC|CCCLASS|NARROW|SILENT|STALE) ' "$1" | sed -E 's/ elapsed_seconds=[0-9.]+//' | LC_ALL=C sort; }
if diff <(strip "$expected") <(strip "$actual") > /dev/null; then
  echo "全同（$(strip "$expected" | wc -l) 行）"
else
  echo "不同："; diff <(strip "$expected") <(strip "$actual") | head -40; exit 1
fi
