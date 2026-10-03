#!/bin/bash
bin=/tmp/claude-1000/m3-ring-r3-sonnet/target/release/ring_hole_direct
out=/tmp/claude-1000/m3-ring-r3-sonnet/h1
run() { local name=$1; shift; local envs=(); while [ "$1" != "--" ]; do envs+=("$1"); shift; done; shift
  env "${envs[@]}" RUN_WITH_MEMORY_CAP_WAIT_SECONDS=3600 RUN_WITH_MEMORY_CAP_PARALLEL_REASON="m3-ring-r3 辩方 H5" RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="同一条命令换环境变量选不同的臂峰值不同" nice -n 19 bash /home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh 1G bash /home/fy5090/code/singlefs/research/scripts/capped.sh 7 "$bin" "$@" > "$out/$name.out" 2> "$out/$name.err"
  echo "$name rc=$?" ; }
for s in 4 8; do for pos in front back; do for row in 1; do for m in 0 1 2 3 4 5; do
  run "A-s$s-$pos-row$row-m$m" CRASH_THE_ROW_PUBLISH=$row -- $s $pos $m $((3*s+4))
done; done; done; done
