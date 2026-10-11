#!/bin/bash
cd /tmp/claude-1000/m3-ring-r2-opus/repo
RUN_WITH_MEMORY_CAP_WAIT_SECONDS=3600 RUN_WITH_MEMORY_CAP_PARALLEL_REASON="m3-ring-r2 攻方腿原型编译，与别的子 agent 各跑各的" RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="改了源码要重编，增量编译峰值随改动不同" CARGO_TARGET_DIR=/tmp/claude-1000/m3-ring-r2-opus/target nice -n 19 bash /home/fy5090/code/singlefs/scripts/hosts/run-with-memory-cap.sh 3G bash /home/fy5090/code/singlefs/scripts/hosts/capped.sh 4 cargo build --offline --release -p ring-window-proto > /tmp/claude-1000/m3-ring-r2-opus/build.log 2>&1
echo "exit=$?" >> /tmp/claude-1000/m3-ring-r2-opus/build.log
grep -E '^error' -A10 /tmp/claude-1000/m3-ring-r2-opus/build.log | head -60
tail -2 /tmp/claude-1000/m3-ring-r2-opus/build.log
