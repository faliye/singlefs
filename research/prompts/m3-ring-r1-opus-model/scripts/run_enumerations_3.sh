#!/usr/bin/env bash
# H2 第三批：树表 0 条暖机洞上 P2 根前的两种记法（分配器在写行那次之后 / 之前记补洞）
set -u
bin=${RING_HOLE_BIN_DIR:-/tmp/claude-1000/m3-ring-r1-opus/target/release}
out=${RING_HOLE_OUT:-/tmp/claude-1000/m3-ring-r1-opus-2/h2}
for m in 2 5; do
  start=$(date +%s)
  PREFIX_WITHOUT_FILE=1 CRASH_THE_WARM_UP=1 SHOW_VIOLATION_DETAIL=1 nice -n 19 "$bin/ring_hole_crash_states" 4 back $m 2 1 > "$out/wf-warm-s4-m$m.out" 2> "$out/wf-warm-s4-m$m.err"
  echo "wf-warm-s4-m$m rc=$? seconds=$(( $(date +%s) - start ))" >> "$out/progress.md"
done
