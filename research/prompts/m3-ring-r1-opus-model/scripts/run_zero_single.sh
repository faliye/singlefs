#!/usr/bin/env bash
# H3：slot(h) 读不出（全零）时把 I-3.9 收成单值
set -u
bin=${RING_HOLE_BIN_DIR:-/tmp/claude-1000/m3-ring-r1-opus/target/release}
out=${RING_HOLE_OUT:-/tmp/claude-1000/m3-ring-r1-opus-2/h1}
for m in 1; do for single in 0 1; do
  HOLE_FILL_BYTES=zero I39_SINGLE_VALUE=$single SHOW_VIOLATION_DETAIL=1 nice -n 19 "$bin/ring_hole_direct" 8 back $m 6 > "$out/E-s8-back-zero-m$m-single$single.out" 2> "$out/E-s8-back-zero-m$m-single$single.err"
  echo "E-s8-back-zero-m$m-single$single rc=$? $(grep -h name=direct_peak "$out/E-s8-back-zero-m$m-single$single.out")"
done; done
