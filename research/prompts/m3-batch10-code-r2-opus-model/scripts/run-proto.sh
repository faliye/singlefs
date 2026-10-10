#!/usr/bin/env bash
# 用法：run-proto.sh <内存上限> <线程数> <日志> <子命令>；直接跑副本里编好的层 0 原型二进制（经内存包装），环境变量由调用方设
cap="$1"; threads="$2"; log="$3"; shift 3
cd /tmp/claude-1000/m3-batch10-code-r2-opus/tree || exit 99
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS="${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-150}"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="派出去的攻方腿 m3-batch10-code-r2-opus 在自己的副本上并行跑层 0 原型"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="原型各段参数不同，峰值表里那一次是另一段"
nice -n 19 bash /home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh "$cap" bash /home/fy5090/code/singlefs/research/scripts/capped.sh "$threads" target/release/opus-r2-crash-states "$@" > "$log" 2>&1
echo "$?"
