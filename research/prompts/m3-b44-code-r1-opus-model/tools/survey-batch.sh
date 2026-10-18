#!/usr/bin/env bash
# 换几何的取样：survey-batch.sh <二进制> <前缀表> <输出目录> <并行件数> <读故障组表>。前缀表每行「R 的种类 前缀」；每件一个 survey
# （M1 写行根之前的写全落那一个盘面，入库 `urJ` 用例的盘面换几何），退出码写进 <输出目录>/s<批号>-<件号>.rc，每批开跑前删这一批的 .rc，
# 不带参数 wait 等齐、数 .rc、按起的次序读。
set -euo pipefail
binary="$1" table="$2" out="$3" per_batch="$4" faults="$5"
mapfile -t lines < "$table"
total=${#lines[@]}
for ((batch = 0; batch * per_batch < total; batch++)); do
  rm -f "$out"/s"$batch"-*.rc
  index=0
  for ((i = batch * per_batch; i < total && i < (batch + 1) * per_batch; i++)); do
    read -r kind prefix <<<"${lines[$i]}"
    { nice -n 19 "$binary" "$kind" "$prefix" survey "$faults" > "$out/$kind-$prefix.txt" 2> "$out/$kind-$prefix.err"; echo "$?" > "$out/s$batch-$index.rc"; } &
    index=$((index + 1))
  done
  wait
  rc_files=$(ls "$out"/s"$batch"-*.rc 2>/dev/null | wc -l)
  if [[ "$rc_files" -ne "$index" ]]; then echo "batch=$batch void"; exit 3; fi
  bad=0
  for ((j = 0; j < index; j++)); do [[ "$(cat "$out/s$batch-$j.rc")" == 0 ]] || bad=$((bad + 1)); done
  echo "survey_batch=$batch launched=$index rc_files=$rc_files nonzero=$bad"
done
