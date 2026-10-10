#!/usr/bin/env bash
# 比一次复跑的 summary/ 与本目录 expected/：每份先排序再比（输出里没有耗时字段；耗时只进 sweep.err）。不依赖 SINGLEFS_REPO。
# 用法：compare.sh <复跑输出目录>；全 same 退 0。
set -euo pipefail
HERE=$(cd "$(dirname "$0")" && pwd)
OUT=${1:?用法：compare.sh <复跑输出目录>}
rc=0
for f in "$HERE"/expected/*.txt; do
    name=$(basename "$f")
    [[ "$name" == sorted-output-sha256.txt ]] && continue
    if cmp -s <(sort "$f") <(sort "$OUT/summary/$name"); then echo "same $name"; else echo "differ $name"; rc=1; fi
done
exit $rc
