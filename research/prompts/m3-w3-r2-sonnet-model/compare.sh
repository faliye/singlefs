#!/usr/bin/env bash
# 用法：bash compare.sh <期望目录> <新输出目录>        比
#       bash compare.sh --make <期望目录> <新输出目录>  把新输出整理成期望目录（只在生成 expected 时用）
# 模型输出（model-*.out）：去掉 W3TRACE 行、排序之后比 sha256；装置输出：按组用 tools/summarize.py 出摘要再比 sha256。
# 全部相同退 0，有不同或缺文件退 1。不读 SINGLEFS_REPO。
set -uo pipefail
MAKE=0; if [ "${1:-}" = "--make" ]; then MAKE=1; shift; fi
EXPECTED=${1:?期望目录}; NEW=${2:?新输出目录}
HERE=$(cd "$(dirname "$0")" && pwd)
normalize_model() { grep -v '^W3TRACE ' "$1" | LC_ALL=C sort; }
groups=("G-long-V1:dev-G-long-V1-[0-9]*-[0-9]*.out" "F2-V1:dev-F2-V1-[0-9]*-[0-9]*.out" "G-long-P:dev-G-long-P-[0-9]*-[0-9]*.out" "F2-P:dev-F2-P-[0-9]*-[0-9]*.out" "G-long-V1-idx24:dev-G-long-V1-idx24.out")
summary_of() { # $1 目录 $2 glob
  local files; files=$(cd "$1" && ls $2 2>/dev/null | sed "s|^|$1/|")
  [ -n "$files" ] || return 1
  python3 "$HERE/tools/summarize.py" $files
}
status=0; compared=0
if [ "$MAKE" = 1 ]; then
  mkdir -p "$EXPECTED"
  for f in "$NEW"/model-*.out; do normalize_model "$f" > "$EXPECTED/$(basename "$f")"; done
  for g in "${groups[@]}"; do name=${g%%:*}; glob=${g#*:}; summary_of "$NEW" "$glob" > "$EXPECTED/device-$name.summary"; done
  echo "已写 $EXPECTED"; exit 0
fi
for expected in "$EXPECTED"/model-*.out; do
  name=$(basename "$expected"); new="$NEW/$name"
  if [ ! -f "$new" ]; then echo "缺 $name"; status=1; continue; fi
  a=$(LC_ALL=C sort "$expected" | sha256sum | cut -c1-16); b=$(normalize_model "$new" | sha256sum | cut -c1-16)
  compared=$((compared + 1))
  if [ "$a" = "$b" ]; then echo "same $name $a"; else echo "DIFFERENT $name expected=$a new=$b"; status=1; fi
done
for g in "${groups[@]}"; do
  name=${g%%:*}; glob=${g#*:}; expected="$EXPECTED/device-$name.summary"
  if [ ! -f "$expected" ]; then echo "缺期望 device-$name.summary"; status=1; continue; fi
  new_text=$(summary_of "$NEW" "$glob") || { echo "缺 $glob"; status=1; continue; }
  a=$(sha256sum < "$expected" | cut -c1-16); b=$(printf '%s\n' "$new_text" | sha256sum | cut -c1-16)
  compared=$((compared + 1))
  if [ "$a" = "$b" ]; then echo "same device-$name.summary $a"; else echo "DIFFERENT device-$name.summary expected=$a new=$b"; status=1; fi
done
echo "compared=$compared status=$status"
exit $status
