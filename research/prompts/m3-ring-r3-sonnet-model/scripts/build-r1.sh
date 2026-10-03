#!/bin/bash
cd /tmp/claude-1000/m3-ring-r3-sonnet/chk/r1
RUN_WITH_MEMORY_CAP_WAIT_SECONDS=3600 RUN_WITH_MEMORY_CAP_PARALLEL_REASON="m3-ring-r3 辩方腿 H5 重做 r1 模型，编译副本" RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="首次整编峰值未知" CARGO_TARGET_DIR=/tmp/claude-1000/m3-ring-r3-sonnet/target nice -n 19 bash /home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh 4G bash /home/fy5090/code/singlefs/research/scripts/capped.sh 7 cargo build --offline --release -p ring-hole-proto > /tmp/claude-1000/m3-ring-r3-sonnet/build-r1.log 2>&1
echo "exit=$?" >> /tmp/claude-1000/m3-ring-r3-sonnet/build-r1.log
tail -3 /tmp/claude-1000/m3-ring-r3-sonnet/build-r1.log
