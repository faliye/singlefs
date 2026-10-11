#!/usr/bin/env bash
# 用法：build.sh <臂名>   $ATTACK_DIR/<臂名> 是冻结副本的拷贝（补丁已打、原型已放进 crates/attack-b16-probe），经内存包装编 release。
set -eu
REPO=${SINGLEFS_REPO:-/home/fy5090/code/singlefs}
arm=$1
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS=${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-180}
export CARGO_TARGET_DIR=$ATTACK_DIR/target-$arm
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="整编时依赖重编，峰值比增量编大得多，auto 撞过上限"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="三方攻方腿与别的会话各跑各的编译"
cd "$ATTACK_DIR/$arm" && nice -n 19 bash "$REPO/scripts/hosts/run-with-memory-cap.sh" 4G bash "$REPO/scripts/hosts/capped.sh" "${ATTACK_THREADS:-7}" cargo build --release --offline -p attack-b16-probe
