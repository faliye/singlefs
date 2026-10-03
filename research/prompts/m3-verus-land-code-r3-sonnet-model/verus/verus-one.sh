#!/usr/bin/env bash
# 用法：verus-one.sh <核 crate 目录（含 src/lib.rs）> <日志>
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="Verus 在核上的峰值几百 M，取 2G 求稳，在派发给的 12G 之内"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="同会话几个腿各跑各的"
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS=900
V=$HOME/.local/share/singlefs/formal-tools/verus-0.2026.09.20/verus-x86-linux/verus
cd "$1" && nice -n 19 bash /home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh 2G "$V" --crate-type=lib --edition=2021 --crate-name singlefs_replay_prefix --no-cheating --rlimit 10 src/lib.rs > "$2" 2>&1
echo "exit $?" >> "$2"
