#!/usr/bin/env bash
# 模型 M9（gate-shrink-r1 云端攻方）：B7 里 47 号 research-script-selftests 那一格慢在哪、并行之后还全不全。
# runner 表从 47 号文件里「for runner in」到「; do」那一段现抽（不手抄）；① 逐条顺序跑、计时、记退出码；
# ② 按①的耗时从长到短，每批 8 条并行（每条写自己的 .rc，不带参数的 wait 等齐，数 .rc 份数），记整批挂钟。
# 在仓根上跑，与那一格同一个工作目录、同一条命令；不改仓。
# 用法：bash m9_research_selftest_timing.sh <仓根> <输出目录>
set -uo pipefail
repository_root="$(cd "${1:?仓根}" && pwd)"
output_directory="${2:?输出目录}"
mkdir -p "$output_directory"
stage="$repository_root/.claude/gate.d/47-code-tooling-selftests-and-registries.sh"
awk '/^for runner in /{inside=1} inside{print} inside && /; do$/{exit}' "$stage" | grep -oE '"(bash|python3) [^"]+"' | tr -d '"' > "$output_directory/runners.txt"
cd "$repository_root" || exit 2
: > "$output_directory/sequential.tsv"
while IFS= read -r runner; do
  start=$(date +%s%N); $runner > "$output_directory/last.log" 2>&1; code=$?; end=$(date +%s%N)
  printf '%s\t%d\t%.1f\n' "$runner" "$code" "$(echo "($end - $start)/1000000000" | bc -l)" >> "$output_directory/sequential.tsv"
done < "$output_directory/runners.txt"
sort -t$'\t' -k3 -rn "$output_directory/sequential.tsv" | cut -f1 > "$output_directory/by-time.txt"
mapfile -t runners < "$output_directory/by-time.txt"
: > "$output_directory/parallel.tsv"
start_all=$(date +%s); batch=0
for ((i = 0; i < ${#runners[@]}; i += 8)); do
  batch=$((batch + 1)); rm -f "$output_directory"/b${batch}-*.rc; sent=0
  for ((j = i; j < i + 8 && j < ${#runners[@]}; j++)); do
    piece=$((j - i + 1)); sent=$((sent + 1))
    { ${runners[$j]} > "$output_directory/b${batch}-${piece}.log" 2>&1; echo "$?" > "$output_directory/b${batch}-${piece}.rc"; } &
  done
  wait
  if [[ "$(ls "$output_directory"/b${batch}-*.rc 2>/dev/null | wc -l)" != "$sent" ]]; then echo "batch $batch 作废：派 $sent 条，.rc 份数对不上" >> "$output_directory/parallel.tsv"; continue; fi
  for ((piece = 1; piece <= sent; piece++)); do
    printf '%d\t%s\t%s\n' "$batch" "${runners[$((i + piece - 1))]}" "$(cat "$output_directory/b${batch}-${piece}.rc")" >> "$output_directory/parallel.tsv"
  done
done
echo "sequential_total_seconds=$(awk -F'\t' '{s += $3} END {printf "%.1f", s}' "$output_directory/sequential.tsv") nonzero=$(awk -F'\t' '$2 != 0' "$output_directory/sequential.tsv" | wc -l) runners=$(wc -l < "$output_directory/runners.txt")"
echo "parallel_wall_seconds=$(( $(date +%s) - start_all )) nonzero=$(awk -F'\t' '$3 != 0' "$output_directory/parallel.tsv" | wc -l)"
