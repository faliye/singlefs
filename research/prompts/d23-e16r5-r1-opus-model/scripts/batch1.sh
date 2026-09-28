#!/usr/bin/env bash
# 批 1：reproduce 六个几何点 + overwrite + difference + separated 三组，共 11 件，各写退出码文件
set -u
cd /tmp/claude-1000/d23-e16r5-r1-opus
BINARY=/tmp/claude-1000/d23-e16r5-r1-opus/target/release/d23-e16r5-r1-opus-model
rm -f b1-*.rc
jobs_list=(
  "reproduce four_gibibyte_file"
  "reproduce sixteen_tebibyte_file"
  "reproduce full_root_level_four"
  "reproduce two_fifty_six_mebibyte_file"
  "reproduce small_fanout_one_tebibyte_file"
  "reproduce large_fanout_one_tebibyte_file"
  "overwrite"
  "difference"
  "separated root_children_knob"
  "separated height_knob"
  "separated fanout_knob"
)
piece=0
for job in "${jobs_list[@]}"; do
  piece=$((piece + 1))
  output_name="b1-${piece}-$(echo "$job" | tr ' ' '-')"
  { nice -n 19 "$BINARY" $job > "$output_name.out" 2> "$output_name.err"; echo "$?" > "b1-${piece}.rc"; } &
done
wait
count=$(ls b1-*.rc | wc -l)
echo "派出 ${#jobs_list[@]} 件，收回 $count 个退出码文件"
for index in $(seq 1 ${#jobs_list[@]}); do echo "件 $index (${jobs_list[$((index-1))]}): rc=$(cat b1-$index.rc)"; done
