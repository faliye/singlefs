#!/usr/bin/env bash
# 五个挂点依次跑 E156 副本（0 不补、1 P1、2 P2 根前、3 P2 根后、4 P3）
set -u
bin=${RING_HOLE_BIN_DIR:-/tmp/claude-1000/m3-ring-r1-opus/target/release}/e156_fill_probe
out=${RING_HOLE_OUT:-/tmp/claude-1000/m3-ring-r1-opus/e156}
for mode in 0 1 2 3 4; do
  start=$(date +%s)
  HOLE_FILL_POINT=$mode E156_FILL_PROBE_IGNORE_STOPS=1 nice -n 19 "$bin" > "$out/mode$mode.out" 2> "$out/mode$mode.err"
  echo "mode=$mode rc=$? seconds=$(( $(date +%s) - start ))" >> "$out/progress.txt"
done
