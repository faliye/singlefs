#!/usr/bin/env bash
# 用法：verus1.sh <lib.rs 所在 crate 目录> <输出文件> [线程数，默认 4] [rlimit，默认 10]
cd "$1" || exit 2
V=~/.local/share/singlefs/formal-tools/verus-0.2026.09.20/verus-x86-linux/verus
T="${3:-4}"; RL="${4:-10}"
RUN_WITH_MEMORY_CAP_PARALLEL_REASON="攻方腿 m3-formal-code-r1 在自己的拷贝上单文件验核" RUN_WITH_MEMORY_CAP_WAIT_SECONDS=600 \
RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="攻方腿改坏核或加探针，峰值随改动变" \
  nice -n 19 bash /home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh 1500M \
  bash /home/fy5090/code/singlefs/research/scripts/capped.sh 5 \
  "$V" --crate-type=lib --edition=2021 --crate-name singlefs_replay_prefix --no-cheating --rlimit "$RL" --num-threads "$T" src/lib.rs > "$2" 2>&1
rc=$?
pe=$(grep -E '^error' "$2" | grep -v 'Resource limit (rlimit) exceeded' | grep -vc '^error: aborting due to')
rl=$(grep -c 'Resource limit (rlimit) exceeded' "$2")
echo "exit=$rc proof_errors=$pe rlimit_lines=$rl $(grep -E 'verification results' "$2")"
