#!/bin/bash
# 用法：closures.sh <unit-graph 二进制> <仓副本> <输出目录> <入口路径>…
UG=$1; TREE=$2; OUT=$3; shift 3
mkdir -p "$OUT"
for e in "$@"; do
  n=${e##*::}
  "$UG" closure "$TREE" "$e" --toolchain-line 'host: x86_64-unknown-linux-gnu' --threads 31 > "$OUT/closure-$n.txt" 2> "$OUT/closure-$n.err"
  echo "$e rc=$?"; tail -1 "$OUT/closure-$n.err"
  python3 -I "$(dirname "$0")/closure_stats.py" "$OUT/closure-$n.txt"
done
