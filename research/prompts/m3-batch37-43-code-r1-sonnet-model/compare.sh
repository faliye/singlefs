#!/usr/bin/env bash
# 用法：bash compare.sh <输出目录甲> <输出目录乙>   比 static / f2diff / baseline / mutations / adhoc 五个 .txt（两边都有的才比）：排序、去掉耗时与 ROWS 行（分段追加会多出几行 ROWS）之后逐行相同才报「全同」。不依赖 SINGLEFS_REPO。
set -euo pipefail
a="$1"; b="$2"; bad=0
norm() { sed -E 's/ finished in [0-9.]+s//; s/ secs=[0-9]+//' "$1" | grep -v '^ROWS ' | LC_ALL=C sort; }
for f in static.txt f2diff.txt baseline.txt mutations.txt adhoc.txt; do
  [ -e "$a/$f" ] && [ -e "$b/$f" ] || continue
  if diff <(norm "$a/$f") <(norm "$b/$f") > /dev/null; then echo "  ✓ $f 全同（$(norm "$a/$f" | wc -l) 行）"; else echo "  ✗ $f 不同"; diff <(norm "$a/$f") <(norm "$b/$f") | head -20; bad=1; fi
done
exit $bad
