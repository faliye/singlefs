#!/bin/bash
# 在脚本所在的草稿目录里编原型（tree/ 是打过 core-and-checker.diff、放进 rr-attack-proto 的冻结副本）。经内存包装 6G、capped.sh 15。
here=$(cd "$(dirname "$0")" && pwd)
cd "$here/tree"
RUN_WITH_MEMORY_CAP_WAIT_SECONDS=3600 RUN_WITH_MEMORY_CAP_PARALLEL_REASON="m3-root-ring-code-r1 攻方腿原型编译，与别的会话各跑各的" RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="改了源码要重编，增量编译峰值随改动不同" CARGO_TARGET_DIR="$here/target" nice -n 19 bash /home/fy5090/code/singlefs/scripts/run-with-memory-cap.sh 6G bash /home/fy5090/code/singlefs/scripts/capped.sh 15 cargo build --offline --release -p rr-attack-proto > "$here/build.log" 2>&1
echo "exit=$?" >> "$here/build.log"
grep -E '^error' -A12 "$here/build.log" | head -80
tail -2 "$here/build.log"
