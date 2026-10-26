#!/usr/bin/env bash
# W2：按调用边切掉 HostLookup -> host-resources.py 之后，登记的 35 个判法入口（w2-entries.txt，取自第一轮正推腿的 tools/entries.txt）里哪几个的闭包变了。
# 每个入口在原树与 cut 树上各收一次闭包，报两边单元数、拒算、少掉的单元数（按种类、限定名、文件比）。八件一批并行，每件退出码写进自己的文件。
# 用法：w2-entries.sh <原树> <cut 树> <入口清单> <输出目录>
set -u
base="$1" cut="$2" list="$3" out="$4"
mkdir -p "$out"
mapfile -t entries < "$list"
units_of() { awk -F'\t' '$1!="via" && $1!="refused" && $1!="key" && NF>=4 {print $1"\t"$2"\t"substr($3,1,index($3,":")-1)}' "$1" | sort -u; }
batch=0
for ((start = 0; start < ${#entries[@]}; start += 8)); do
  batch=$((batch + 1))
  rm -f "$out"/b$batch-*.rc
  sent=0
  for ((i = start; i < start + 8 && i < ${#entries[@]}; i++)); do
    sent=$((sent + 1))
    entry="${entries[$i]}"
    { for arm in full cut; do
        root="$base"; [[ $arm == cut ]] && root="$cut"
        (cd "$base" && python3 -B scripts/gate/lib_function_units.py closure "$root" "$entry") > "$out/c-$arm-$i.txt" 2>&1
      done
      echo 0 > "$out/b$batch-$sent.rc"; } &
  done
  wait
  got=$(ls "$out"/b$batch-*.rc 2>/dev/null | wc -l)
  [[ $got -eq $sent ]] || { echo "name=entries_batch_incomplete batch=$batch sent=$sent got=$got（这一批作废）"; continue; }
  for ((i = start; i < start + 8 && i < ${#entries[@]}; i++)); do
    units_of "$out/c-full-$i.txt" > "$out/u-full-$i.txt"; units_of "$out/c-cut-$i.txt" > "$out/u-cut-$i.txt"
    refused_full="$(awk -F'\t' '$1=="refused"{print $2}' "$out/c-full-$i.txt" | sort | uniq -c | awk '{printf "%s:%s;", $2, $1}')"
    refused_cut="$(awk -F'\t' '$1=="refused"{print $2}' "$out/c-cut-$i.txt" | sort | uniq -c | awk '{printf "%s:%s;", $2, $1}')"
    echo "name=entry index=$i entry=${entries[$i]} units_full=$(wc -l < "$out/u-full-$i.txt") units_cut=$(wc -l < "$out/u-cut-$i.txt") dropped=$(comm -23 "$out/u-full-$i.txt" "$out/u-cut-$i.txt" | wc -l) added=$(comm -13 "$out/u-full-$i.txt" "$out/u-cut-$i.txt" | wc -l) refused_full=[${refused_full}] refused_cut=[${refused_cut}]"
  done
done
