#!/usr/bin/env bash
# usage: run_m.sh <log> <test filter...>  （在 tree_m 上跑，目标目录独立）
cd /tmp/claude-1000/m3-verus-land-code-r1-sonnet/tree_m
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="派发提示给定内存上限八G，差分草稿测试按提示上限跑"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="同会话派出的几个腿各跑各的，这一条是差分草稿测试"
log=/tmp/claude-1000/m3-verus-land-code-r1-sonnet/$1; shift
nice -n 19 bash /home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh 8G bash /home/fy5090/code/singlefs/research/scripts/capped.sh 10 cargo test --release --offline -p singlefs-core --lib "$@" > $log 2>&1
echo "exit $?" >> $log
