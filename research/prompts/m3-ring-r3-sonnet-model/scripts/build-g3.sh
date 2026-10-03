#!/bin/bash
# 用法：build-g3.sh <副本名>
d=$1
cd /tmp/claude-1000/m3-ring-r3-sonnet/$d
RUN_WITH_MEMORY_CAP_WAIT_SECONDS=3600 RUN_WITH_MEMORY_CAP_PARALLEL_REASON="m3-ring-r3 辩方腿 H5 移植 C331 网格到冻结副本，编译" RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="首次整编峰值未知" CARGO_TARGET_DIR=/tmp/claude-1000/m3-ring-r3-sonnet/target-$d nice -n 19 bash /home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh 6G bash /home/fy5090/code/singlefs/research/scripts/capped.sh 7 cargo test --offline --release -p singlefs-harness --test c331_candidates_grid --no-run > /tmp/claude-1000/m3-ring-r3-sonnet/build-$d.log 2>&1
echo "exit=$?" >> /tmp/claude-1000/m3-ring-r3-sonnet/build-$d.log
grep -E '^error' -A8 /tmp/claude-1000/m3-ring-r3-sonnet/build-$d.log | head -40
tail -3 /tmp/claude-1000/m3-ring-r3-sonnet/build-$d.log
