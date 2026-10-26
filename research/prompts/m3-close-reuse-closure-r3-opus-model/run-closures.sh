#!/usr/bin/env bash
# 几棵树（full = 原树副本，另几棵是 make-trees.py 造的）上，用每棵树自己的 lib_function_units.py 收清单里每个入口的闭包（closure --explain），
# 每件的输出落 <输出目录>/c-<树>-<序号>.txt；每件退出码写进自己的 .rc，一批派齐后 wait 等齐、数 .rc，对不上整批作废。
# 用法：run-closures.sh <原树> <变体目录> <入口清单> <输出目录> <并行件数> <树…>
set -u
base="$1" variants="$2" list="$3" out="$4" width="$5"; shift 5
mkdir -p "$out"
mapfile -t entries < "$list"
jobs=()
for tree in "$@"; do for ((i = 0; i < ${#entries[@]}; i++)); do jobs+=("$tree $i"); done; done
batch=0
for ((start = 0; start < ${#jobs[@]}; start += width)); do
  batch=$((batch + 1)); rm -f "$out"/b$batch-*.rc; sent=0
  for ((j = start; j < start + width && j < ${#jobs[@]}; j++)); do
    sent=$((sent + 1)); read -r tree i <<<"${jobs[$j]}"
    root="$variants/$tree"; [[ $tree == full ]] && root="$base"
    { (cd "$root" && nice -n 19 python3 -B scripts/gate/lib_function_units.py closure . "${entries[$i]}" --explain) > "$out/c-$tree-$i.txt" 2> "$out/c-$tree-$i.err"
      echo "$?" > "$out/b$batch-$sent.rc"; } &
  done
  wait
  got=$(ls "$out"/b$batch-*.rc 2>/dev/null | wc -l)
  [[ $got -eq $sent ]] || echo "name=closure_batch_incomplete batch=$batch sent=$sent got=$got（这一批作废）"
  for ((k = 1; k <= sent; k++)); do read -r tree i <<<"${jobs[$((start + k - 1))]}"; echo "name=closure_rc tree=$tree index=$i rc=$(cat "$out/b$batch-$k.rc")"; done
done
