#!/bin/bash
# 复跑：bash rerun.sh <输出目录>（经 run-with-memory-cap.sh 与 capped.sh 起，见报告开头）
# 深度 8：14 条臂 × 4 个几何，每批 4 件并行；深度 9：几何 3×4、环 3，挑 9 条臂一件一件跑。
# 跑完把输出与本目录的 model-output.txt 比，末行报「model-output 相同 / 不同」。
set -u
here=$(cd "$(dirname "$0")" && pwd)
out=${1:?输出目录}
mkdir -p "$out"
rm -f "$out/model-output.txt" "$out/progress.txt"
M="$here/gc_intent_model.py"
arms=$(python3 -B "$M" list)
geos=("3 4 3 2 3" "3 4 4 2 3" "4 4 3 2 3" "3 6 3 2 3")
jobs=()
for g in "${geos[@]}"; do for a in $arms; do jobs+=("$a|$g"); done; done
i=0
batch=0
while [ "$i" -lt "${#jobs[@]}" ]; do
  batch=$((batch + 1))
  rm -f "$out"/b"$batch"-*.rc "$out"/b"$batch"-*.out
  n=0
  for k in 0 1 2 3; do
    j=$((i + k))
    [ "$j" -lt "${#jobs[@]}" ] || break
    IFS='|' read -r a g <<< "${jobs[$j]}"
    # shellcheck disable=SC2086
    { python3 -B "$M" "$a" $g 8 > "$out/b$batch-$k.out"; echo "$?" > "$out/b$batch-$k.rc"; } &
    n=$((n + 1))
  done
  wait
  c=$(find "$out" -maxdepth 1 -name "b$batch-*.rc" | wc -l)
  if [ "$c" -ne "$n" ]; then echo "批 $batch 作废：rc 文件 $c 件、派出 $n 件"; exit 3; fi
  for k in $(seq 0 $((n - 1))); do
    rc=$(cat "$out/b$batch-$k.rc")
    if [ "$rc" != 0 ]; then echo "批 $batch 件 $k 退出码 $rc"; exit 4; fi
    cat "$out/b$batch-$k.out" >> "$out/model-output.txt"
  done
  echo "深度 8 批 $batch 完成 $n 件" >> "$out/progress.txt"
  i=$((i + n))
done
for a in nogc AB-old-recheck AB-old-skip AB-cur-recheck AB-cur-skip C-old-recheck C-cur-recheck-naive adm-plain adm-rterm_issue adm-delexempt; do
  python3 -B "$M" "$a" 3 4 3 2 3 9 >> "$out/model-output.txt" || { echo "深度 9 $a 失败"; exit 5; }
  echo "深度 9 $a 完成" >> "$out/progress.txt"
done
grep -c '^E ' "$out/model-output.txt"
if [ -f "$here/model-output.txt" ]; then
  if cmp -s "$out/model-output.txt" "$here/model-output.txt"; then echo "model-output 相同"; else echo "model-output 不同"; fi
fi
