#!/usr/bin/env bash
# 把 rerun.sh 的输出目录与 expected/ 逐份比：两边各自排序后比（产物里没有耗时字段；copy.out 只收 `R ` 行）。不依赖 SINGLEFS_REPO。
set -euo pipefail
HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
OUT=${1:?用法：compare.sh <输出目录>}
status=0
for expected in "$HERE"/expected/*.out; do
    name=$(basename "$expected")
    if [[ ! -f "$OUT/$name" ]]; then
        echo "missing $name"
        status=1
        continue
    fi
    if diff <(LC_ALL=C sort "$expected") <(LC_ALL=C sort "$OUT/$name") > /dev/null; then
        echo "same $name"
    else
        echo "differ $name"
        status=1
    fi
done
exit $status
