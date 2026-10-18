#!/usr/bin/env bash
# 一批：一段历史从 <起点> 起切成 <件数> 件、每件 <每件状态数> 个崩溃状态，并行跑 run 模式；每件退出码写进自己的 .rc，
# 开跑前删掉这一批的 .rc，最后一个不带参数的 wait 等齐，数 .rc 与件数对得上再按次序读。
# 用法：run-batch.sh <二进制> <R 的种类> <k|前缀> <起点> <状态总数> <件数> <每件状态数> <读故障组串> <输出目录> <批号>
set -u
binary="$1"; kind="$2"; prefix="$3"; start="$4"; total="$5"; pieces="$6"; per_piece="$7"; faults="$8"; out="$9"; batch="${10}"
mkdir -p "$out"
rm -f "$out"/b"$batch"-*.rc
launched=0
for ((piece = 0; piece < pieces; piece++)); do
  first=$((start + piece * per_piece))
  ((first >= total)) && break
  last=$((first + per_piece)); ((last > total)) && last=$total
  { nice -n 19 "$binary" "$kind" "$prefix" run "$first" "$last" "$faults" > "$out/b$batch-$piece.out" 2>&1; echo "$?" > "$out/b$batch-$piece.rc"; } &
  launched=$((launched + 1))
done
wait
found=$(ls "$out"/b"$batch"-*.rc 2>/dev/null | wc -l)
if [[ "$found" -ne "$launched" ]]; then echo "name=run_batch batch=$batch launched=$launched rc_files=$found 整批作废"; exit 3; fi
for ((piece = 0; piece < launched; piece++)); do
  echo "name=run_piece batch=$batch piece=$piece rc=$(cat "$out/b$batch-$piece.rc")"
done
