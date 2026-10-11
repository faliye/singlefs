#!/usr/bin/env bash
# 用法：run.sh <内存上限> <日志> <cargo test 的参数…>；环境变量由调用方在前面设
cap="$1"; log="$2"; shift 2
cd /tmp/claude-1000/m3-batch10-code-r2-opus/tree || exit 99
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS="${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-150}"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="派出去的攻方腿 m3-batch10-code-r2-opus 在自己的副本上跑原型"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="原型源码在改，峰值表里那一次是改之前或半途 panic 的"
nice -n 19 bash /home/fy5090/code/singlefs/scripts/hosts/run-with-memory-cap.sh "$cap" bash /home/fy5090/code/singlefs/scripts/hosts/capped.sh 31 cargo test --offline -p singlefs-harness --test opus_r2_rr2_attack "$@" > "$log" 2>&1
echo "rc=$?"
