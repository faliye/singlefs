#!/usr/bin/env bash
# 用法：build.sh <臂名>   $ATTACK_DIR/<臂名> 是冻结副本的拷贝（补丁已打、原型拷进 crates/attack-rw-probe、工作区成员已加），经内存包装编 release。
set -eu
REPO=${SINGLEFS_REPO:-/home/fy5090/code/singlefs}
ATTACK_DIR=${ATTACK_DIR:-/tmp/claude-1000/reclaim-window-attack}
arm=$1
export CARGO_TARGET_DIR=$ATTACK_DIR/target-$arm
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS=${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-180}
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="整编时依赖重编，峰值比增量编大"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="三方攻方腿与别的会话各跑各的编译"
cd "$ATTACK_DIR/$arm" && nice -n 19 bash "$REPO/scripts/run-with-memory-cap.sh" 6G bash "$REPO/scripts/capped.sh" "${ATTACK_THREADS:-15}" cargo build --release --offline -p attack-rw-probe
