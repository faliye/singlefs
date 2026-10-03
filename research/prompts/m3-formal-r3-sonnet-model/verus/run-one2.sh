#!/usr/bin/env bash
# run-one.sh <目录>：在目录里 verus 直调，经包装一次
set -u
dir="$1"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="三方复核辩方腿与同会话别的任务各跑各的"
cd "$dir" && nice -n 19 bash /home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh auto bash -c "$HOME/.local/share/singlefs/formal-tools/verus-0.2026.09.20/verus-x86-linux/verus --crate-type=lib --edition=2021 --crate-name singlefs_replay_prefix --no-cheating src/lib.rs" > verus.out 2>&1
echo "exit=$?" >> verus.out
tail -25 verus.out
