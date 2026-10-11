#!/usr/bin/env bash
# 用法：qf.sh <副本目录> <日志>   经内存包装在副本上编跑 Q-F 那一个测试目标与攻方判别形那一个目标（harness 档）。
set -u
REPO=${SINGLEFS_REPO:-/home/fy5090/code/singlefs}
cd "$1" || exit 2
export CARGO_TARGET_DIR=$1/target
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS=180
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="副本上编跑两个 harness 测试目标，峰值未登记"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="攻方腿副本与别的会话各跑各的"
nice -n 19 bash "$REPO/scripts/hosts/run-with-memory-cap.sh" 6G bash "$REPO/scripts/hosts/capped.sh" "${ATTACK_THREADS:-15}" \
  cargo test --offline --no-fail-fast -p singlefs-harness --test unlink_mount_continuation_and_the_emptied_version --test qf_attack_probe > "$2" 2>&1
echo "exit=$?" >> "$2"
