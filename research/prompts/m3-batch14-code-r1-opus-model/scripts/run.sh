#!/usr/bin/env bash
# 用法：run.sh <臂名> <日志>   经内存包装、经 capped.sh 跑那一臂的原型（环境变量照传；线程上限 ATTACK_THREADS，默认 7，层 0 枚举线程同取它）。
set -u
REPO=${SINGLEFS_REPO:-/home/fy5090/code/singlefs}
THREADS=${ATTACK_THREADS:-7}
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="原型换参数、开层 0 枚举后内存会涨"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="同一批几件原型并行跑，件数不超过线程上限"
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS=${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-180}
export SINGLEFS_LAYER0_THREADS=${SINGLEFS_LAYER0_THREADS:-$THREADS}
nice -n 19 bash "$REPO/scripts/run-with-memory-cap.sh" 2G bash "$REPO/scripts/capped.sh" "$THREADS" "$ATTACK_DIR/target-$1/release/attack-b14-probe" > "$2" 2>&1
