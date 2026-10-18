#!/usr/bin/env bash
# 换几何取样：表里每行一段历史（<R 的种类> <k|前缀>），按批并行跑 survey，每件退出码写进自己的 .rc，整批等齐后核件数、按次序读。
# 用法：survey-batch.sh <二进制> <历史表> <输出目录> <每批件数>
set -u
binary="$1"; table="$2"; out="$3"; per_batch="$4"
mkdir -p "$out"
mapfile -t histories < "$table"
total=${#histories[@]}
batch=0
for ((first = 0; first < total; first += per_batch)); do
  rm -f "$out"/b"$batch"-*.rc
  launched=0
  for ((index = first; index < first + per_batch && index < total; index++)); do
    read -r kind prefix <<< "${histories[$index]}"
    { nice -n 19 "$binary" "$kind" "$prefix" survey > "$out/b$batch-$launched.out" 2>&1; echo "$?" > "$out/b$batch-$launched.rc"; } &
    launched=$((launched + 1))
  done
  wait
  found=$(ls "$out"/b"$batch"-*.rc 2>/dev/null | wc -l)
  if [[ "$found" -ne "$launched" ]]; then echo "name=survey_batch batch=$batch launched=$launched rc_files=$found 整批作废"; exit 3; fi
  for ((piece = 0; piece < launched; piece++)); do
    echo "name=survey_piece batch=$batch piece=$piece rc=$(cat "$out/b$batch-$piece.rc")"
  done
  batch=$((batch + 1))
done
