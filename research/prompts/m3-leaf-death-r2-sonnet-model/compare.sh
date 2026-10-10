#!/usr/bin/env bash
# 把 <输出目录>/summary/ 里的每一份与本目录 expected/ 里的逐字节比；全 same 才退 0。
set -uo pipefail
MODEL_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
OUT=${1:?用法：compare.sh <输出目录>}
status=0
for expected in "$MODEL_DIR"/expected/*.txt; do
    name=$(basename "$expected")
    if cmp -s "$expected" "$OUT/summary/$name"; then
        echo "same  $name"
    else
        echo "DIFF  $name"
        status=1
    fi
done
exit $status
