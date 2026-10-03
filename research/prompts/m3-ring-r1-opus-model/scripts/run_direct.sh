#!/usr/bin/env bash
# H1 / H3 直接量：A 挂点矩阵；B 必红世界；C 连崩写行；D I-3.9 单值
set -u
bin=${RING_HOLE_BIN_DIR:-/tmp/claude-1000/m3-ring-r1-opus/target/release}/ring_hole_direct
out=${RING_HOLE_OUT:-/tmp/claude-1000/m3-ring-r1-opus-2/h1}
run() { # 名字 环境… -- 参数…
  local name=$1; shift
  local envs=()
  while [ "$1" != "--" ]; do envs+=("$1"); shift; done; shift
  env "${envs[@]}" nice -n 19 "$bin" "$@" > "$out/$name.out" 2> "$out/$name.err"
  echo "$name rc=$? $(grep -h 'name=direct_mount\|name=direct_peak\|name=direct_rollback' "$out/$name.out" | sed 's/PROTO //' | tr '\n' ' ')"
}
for s in 4 8; do for pos in front back; do for row in 0 1; do for m in 0 1 2 3 4 5; do
  run "A-s$s-$pos-row$row-m$m" CRASH_THE_ROW_PUBLISH=$row -- $s $pos $m $((3*s+4))
done; done; done; done
for pos in front back; do for bytes in zero chosen; do for m in 1 3 4; do
  run "B-s8-$pos-$bytes-m$m" HOLE_FILL_BYTES=$bytes SHOW_VIOLATION_DETAIL=1 -- 8 $pos $m 28
done; done; done
for s in 4 8; do for n in 0 1 2 3 4 6; do for m in 0 1; do
  run "C-s$s-back-row1-repeats$n-m$m" CRASH_THE_ROW_PUBLISH=1 ROW_CRASH_REPEATS=$n -- $s back $m $((3*s+4))
done; done; done
for n in 1 2; do for m in 1 3 4 5; do
  run "C-s8-back-row0-repeats$n-m$m" CRASH_THE_ROW_PUBLISH=0 ROW_CRASH_REPEATS=$n SHOW_VIOLATION_DETAIL=1 -- 8 back $m 28
done; done
for pos in front back; do for row in 0 1; do for m in 0 1; do
  run "D-s8-$pos-row$row-m$m-single" CRASH_THE_ROW_PUBLISH=$row I39_SINGLE_VALUE=1 SHOW_VIOLATION_DETAIL=1 ROLL_BACK_BY=5 -- 8 $pos $m 6
  run "D-s8-$pos-row$row-m$m-interval" CRASH_THE_ROW_PUBLISH=$row SHOW_VIOLATION_DETAIL=1 ROLL_BACK_BY=5 -- 8 $pos $m 6
done; done; done
