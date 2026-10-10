#!/usr/bin/env bash
# 用法：build.sh <臂名>   经内存包装编 $ATTACK_DIR/<臂名> 的原型（release）。
set -eu
REPO=${SINGLEFS_REPO:-/home/fy5090/code/singlefs}
arm=$1
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS=${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-180}
export CARGO_TARGET_DIR=$ATTACK_DIR/target-$arm
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="整编时依赖重编，峰值比增量编大得多，auto 撞过上限（第 16 批攻方同一条）"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="三方攻方腿与别的会话各跑各的编译"
cd "$ATTACK_DIR/$arm" && nice -n 19 bash "$REPO/research/scripts/run-with-memory-cap.sh" 4G bash "$REPO/research/scripts/capped.sh" "${ATTACK_THREADS:-15}" cargo build --release --offline -p attack-b17-probe
