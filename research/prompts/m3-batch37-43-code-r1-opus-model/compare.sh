#!/usr/bin/env bash
# 用法：compare.sh <复跑的 summary.txt> [<期望汇总>]（期望默认本目录 expected-summary.txt）。两边先去掉耗时字段（finished in …），排序之后逐行比；不依赖 SINGLEFS_REPO。
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
actual="${1:?要复跑出来的 summary.txt}"
expected="${2:-$here/expected-summary.txt}"
strip() { sed -E 's/;? finished in [0-9.]+s//' "$1" | LC_ALL=C sort; }
if diff <(strip "$expected") <(strip "$actual"); then
  echo "COMPARE same lines=$(wc -l < "$expected")"
else
  echo "COMPARE differ" >&2
  exit 1
fi
