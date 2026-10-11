#!/usr/bin/env bash
# 用法：run-bin.sh <内存上限> <日志> <测试二进制的参数…>；直接跑副本里编好的 release 原型二进制（经内存包装）
cap="$1"; log="$2"; shift 2
cd /tmp/claude-1000/m3-batch10-code-r2-opus/tree || exit 99
bin=$(ls -t target/release/deps/opus_r2_rr2_attack-* | grep -v '\.d$' | head -n 1)
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS="${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-150}"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="派出去的攻方腿 m3-batch10-code-r2-opus 在自己的副本上并行跑原型"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="原型各段参数不同，峰值表里那一次是另一段"
nice -n 19 bash /home/fy5090/code/singlefs/scripts/run-with-memory-cap.sh "$cap" bash /home/fy5090/code/singlefs/scripts/capped.sh 1 "$bin" "$@" > "$log" 2>&1
echo "$?"
