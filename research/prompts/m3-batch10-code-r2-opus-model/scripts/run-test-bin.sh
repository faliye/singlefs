#!/usr/bin/env bash
# 用法：run-test-bin.sh <内存上限> <日志> <测试二进制名前缀> <libtest 参数…>；直接跑副本里编好的 release 测试二进制（经内存包装），环境变量由调用方设
cap="$1"; log="$2"; name="$3"; shift 3
cd /tmp/claude-1000/m3-batch10-code-r2-opus/tree || exit 99
bin=$(ls -t target/release/deps/"$name"-* | grep -v '\.d$' | head -n 1)
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS="${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-150}"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="派出去的攻方腿 m3-batch10-code-r2-opus 在自己的副本上并行跑入库用例（副本 mount.rs 带开关）"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="副本上的开关改了臂，峰值表里那一次是另一臂"
nice -n 19 bash /home/fy5090/code/singlefs/scripts/run-with-memory-cap.sh "$cap" bash /home/fy5090/code/singlefs/scripts/capped.sh 2 "$bin" "$@" > "$log" 2>&1
echo "$?"
