#!/usr/bin/env bash
# 用法：bash compare.sh <期望目录> <新输出目录>        比
#       bash compare.sh --make <期望目录> <新输出目录>  把新输出整理成期望目录（只在生成 expected 时用）
# 模型输出（model-*.out）：去掉 W3TRACE 行、排序之后比 sha256；cost-model.out、probe-*.out、cost-check.out、probe-compare*.out 逐字节比。
# 全部相同退 0，有不同或缺文件退 1。不读 SINGLEFS_REPO。
set -uo pipefail
MAKE=0; if [ "${1:-}" = "--make" ]; then MAKE=1; shift; fi
EXPECTED=${1:?期望目录}; NEW=${2:?新输出目录}
normalize_model() { grep -v '^W3TRACE ' "$1" | LC_ALL=C sort; }
VERBATIM=(report-tables.out cost-model.out probe-small-real.out probe-bulk.out probe-shape.out cost-check.out probe-compare.out probe-compare-red.out)
if [ "$MAKE" = 1 ]; then
  mkdir -p "$EXPECTED"
  for f in "$NEW"/model-*.out; do normalize_model "$f" > "$EXPECTED/$(basename "$f")"; done
  for v in "${VERBATIM[@]}"; do [ -f "$NEW/$v" ] && cp "$NEW/$v" "$EXPECTED/$v"; done
  echo "已写 $EXPECTED"; exit 0
fi
status=0; compared=0
for expected in "$EXPECTED"/model-*.out; do
  name=$(basename "$expected"); new="$NEW/$name"
  if [ ! -f "$new" ]; then echo "缺 $name"; status=1; continue; fi
  a=$(LC_ALL=C sort "$expected" | sha256sum | cut -c1-16); b=$(normalize_model "$new" | sha256sum | cut -c1-16)
  compared=$((compared + 1))
  if [ "$a" = "$b" ]; then echo "same $name $a"; else echo "DIFFERENT $name expected=$a new=$b"; status=1; fi
done
for v in "${VERBATIM[@]}"; do
  [ -f "$EXPECTED/$v" ] || continue
  if [ ! -f "$NEW/$v" ]; then echo "缺 $v"; status=1; continue; fi
  compared=$((compared + 1))
  if cmp -s "$EXPECTED/$v" "$NEW/$v"; then echo "same $v"; else echo "DIFFERENT $v"; status=1; fi
done
echo "compared=$compared status=$status"
exit $status
