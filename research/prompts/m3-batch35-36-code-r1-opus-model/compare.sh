#!/usr/bin/env bash
# 用法：compare.sh <复跑的 summary.txt> [<期望汇总>]（期望默认本目录 expected-summary.txt）。两边排序之后逐行比，不依赖 SINGLEFS_REPO；汇总里本来就不带耗时。
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
actual="${1:?要复跑出来的 summary.txt}"
expected="${2:-$here/expected-summary.txt}"
if diff <(LC_ALL=C sort "$expected") <(LC_ALL=C sort "$actual"); then
  echo "COMPARE same lines=$(wc -l < "$expected")"
else
  echo "COMPARE differ" >&2
  exit 1
fi
