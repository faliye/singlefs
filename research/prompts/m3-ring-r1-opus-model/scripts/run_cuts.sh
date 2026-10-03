#!/usr/bin/env bash
# H2 补充：带文件那段历史的前缀截点（不是层 0 枚举域），六个挂点 × 前洞后洞 × S ∈ {4, 8}
set -u
bin=${RING_HOLE_BIN_DIR:-/tmp/claude-1000/m3-ring-r1-opus/target/release}/ring_hole_prefix_cuts
out=${RING_HOLE_OUT:-/tmp/claude-1000/m3-ring-r1-opus-2/h2}
for s in 4 8; do for pos in front back; do for m in 0 1 2 3 4 5; do
  SHOW_VIOLATION_DETAIL=1 nice -n 19 "$bin" $s $pos $m > "$out/cuts-s$s-$pos-m$m.out" 2> "$out/cuts-s$s-$pos-m$m.err"
  echo "cuts-s$s-$pos-m$m rc=$? $(grep -h 'name=cuts_summary' "$out/cuts-s$s-$pos-m$m.out")"
done; done; done
