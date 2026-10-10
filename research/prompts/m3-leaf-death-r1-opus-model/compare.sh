#!/usr/bin/env bash
# 比一次复跑的 summary/ 与模型目录 expected/ 里入库的那一份：逐个文件排序之后比（不依赖 SINGLEFS_REPO；k3.txt 里 harness 的
# 「finished in …」耗时已在 rerun.sh 里滤掉，这里再去一遍以防万一）。用法：bash compare.sh <rerun 的输出目录>。退出码 0 全同、1 有不同。
set -uo pipefail
MODEL_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
OUT=${1:?用法：compare.sh <rerun 的输出目录>}
status=0
for expected in "$MODEL_DIR"/expected/*.txt; do
    name=$(basename "$expected")
    if diff <(sed 's/finished in [0-9.]*s//' "$expected" | sort) <(sed 's/finished in [0-9.]*s//' "$OUT/summary/$name" | sort) > /dev/null; then
        echo "same $name"
    else
        echo "DIFFERENT $name"
        status=1
    fi
done
exit $status
