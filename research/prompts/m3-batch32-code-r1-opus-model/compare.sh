#!/usr/bin/env bash
# 用法：bash compare.sh <rerun.sh 产出的 summary.txt>。与同目录 expected-summary.txt 排序后逐行比（两边都不带耗时字段），不依赖 SINGLEFS_REPO。
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
if diff <(LC_ALL=C sort "$1") <(LC_ALL=C sort "$here/expected-summary.txt") > /dev/null; then
  echo "SAME $(wc -l < "$1") 行"
else
  diff <(LC_ALL=C sort "$1") <(LC_ALL=C sort "$here/expected-summary.txt") | head -n 40
  echo "DIFF"; exit 1
fi
