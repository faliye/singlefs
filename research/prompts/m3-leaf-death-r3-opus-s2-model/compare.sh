#!/usr/bin/env bash
# 比 <输出目录>/summary/ 与 expected/ 的每一份，逐份打 same / differ，有一份不同就退 1。模型目录从 R3S2_MODEL_DIR 取。
set -euo pipefail
MODEL_DIR=${R3S2_MODEL_DIR:?要 R3S2_MODEL_DIR（模型目录）}
OUT=${1:?用法：compare.sh <输出目录>}
bad=0
for f in "$MODEL_DIR"/expected/*; do
    name=$(basename "$f")
    if cmp -s "$f" "$OUT/summary/$name"; then echo "same $name"; else echo "differ $name"; bad=1; fi
done
exit "$bad"
