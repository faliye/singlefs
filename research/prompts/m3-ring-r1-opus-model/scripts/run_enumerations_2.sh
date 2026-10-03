#!/usr/bin/env bash
# H2 第二批：对照（不补）的树表 0 条暖机洞；带文件的前洞 S = 4 三个挂点（后洞那段历史 4325419 态，超过 10⁶，不跑）
set -u
bin=${RING_HOLE_BIN_DIR:-/tmp/claude-1000/m3-ring-r1-opus/target/release}
out=${RING_HOLE_OUT:-/tmp/claude-1000/m3-ring-r1-opus-2/h2}
one() {
  local name=$1; shift
  local envs=()
  while [ "$1" != "--" ]; do envs+=("$1"); shift; done; shift
  local start; start=$(date +%s)
  env "${envs[@]}" nice -n 19 "$bin/ring_hole_crash_states" "$@" > "$out/$name.out" 2> "$out/$name.err"
  echo "$name rc=$? seconds=$(( $(date +%s) - start ))" >> "$out/progress.md"
}
one "wf-warm-s4-m0" PREFIX_WITHOUT_FILE=1 CRASH_THE_WARM_UP=1 SHOW_VIOLATION_DETAIL=1 -- 4 back 0 2 1
for m in 1 3 5; do
  one "file-s4-front-m$m" SHOW_VIOLATION_DETAIL=1 -- 4 front $m 2 1
done
