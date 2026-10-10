#!/usr/bin/env bash
# 比两份输出：用法 bash compare.sh <期望目录> <新输出目录>。期望目录里每一份文件在新输出目录里找同名的，
# 去掉 E7TIMING 行与 elapsed_ns 字段、排序之后比 sha256；不读 SINGLEFS_REPO。全部相同退 0，有不同或缺文件退 1。
set -uo pipefail
EXPECTED=${1:?期望目录}; NEW=${2:?新输出目录}
normalize() { grep -v '^E7TIMING ' "$1" | sed -E 's/ elapsed_ns=[0-9]+//g' | LC_ALL=C sort; }
status=0; compared=0
for expected in "$EXPECTED"/*.out; do
  name=$(basename "$expected"); new="$NEW/$name"
  if [ ! -f "$new" ]; then echo "缺 $name"; status=1; continue; fi
  a=$(normalize "$expected" | sha256sum | cut -c1-16); b=$(normalize "$new" | sha256sum | cut -c1-16)
  compared=$((compared + 1))
  if [ "$a" = "$b" ]; then echo "same $name $a"; else echo "DIFFERENT $name expected=$a new=$b"; status=1; fi
done
echo "compared=$compared status=$status"
exit $status
