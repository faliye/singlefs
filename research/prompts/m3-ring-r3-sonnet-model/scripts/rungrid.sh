#!/bin/bash
# 用法：rungrid.sh <副本名> <日志名> [环境变量赋值…]
d=$1; log=$2; shift; shift
cd /tmp/claude-1000/m3-ring-r3-sonnet/$d
bin=$(ls -t /tmp/claude-1000/m3-ring-r3-sonnet/target-$d/release/deps/c331_candidates_grid-* | grep -v '\.d$' | head -1)
env "$@" TMPDIR=/tmp/claude-1000/m3-ring-r3-sonnet/tmp RUN_WITH_MEMORY_CAP_WAIT_SECONDS=3600 RUN_WITH_MEMORY_CAP_PARALLEL_REASON="m3-ring-r3 辩方腿 H5 C331 网格复跑" RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="网格峰值未知先放宽" nice -n 19 bash /home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh 8G bash /home/fy5090/code/singlefs/research/scripts/capped.sh 7 $bin --nocapture > /tmp/claude-1000/m3-ring-r3-sonnet/runs/$log.log 2>&1
echo "rc=$?" >> /tmp/claude-1000/m3-ring-r3-sonnet/runs/$log.log
