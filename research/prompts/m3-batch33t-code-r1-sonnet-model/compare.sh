#!/usr/bin/env bash
# 用法：bash compare.sh <输出目录甲> <输出目录乙>   比 probes.txt / mutations.txt / control.txt / static.txt：排序、去掉耗时字段（secs=、PROBE6 那行的用时）之后逐行相同才报「全同」。不依赖 SINGLEFS_REPO。
set -euo pipefail
a="$1"; b="$2"; bad=0
norm() { grep -v '^PROBE6' "$1" | sed -E 's/ secs=[0-9]+//' | LC_ALL=C sort; }
for f in probes.txt mutations.txt control.txt static.txt; do
  if diff <(norm "$a/$f") <(norm "$b/$f") > /dev/null; then echo "  ✓ $f 全同（$(norm "$a/$f" | wc -l) 行）"; else echo "  ✗ $f 不同"; diff <(norm "$a/$f") <(norm "$b/$f") | head -20; bad=1; fi
done
exit $bad
