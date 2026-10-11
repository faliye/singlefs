#!/usr/bin/env bash
# usage: run.sh <treedir> <log> <test filter...>
T=$1; shift; cd $T
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="派发提示给定内存上限八G，差分草稿测试按提示上限跑"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="同会话派出的几个腿各跑各的，这一条是差分草稿测试"
log=$1; shift
nice -n 19 bash /home/fy5090/code/singlefs/scripts/run-with-memory-cap.sh 8G bash /home/fy5090/code/singlefs/scripts/capped.sh 6 cargo test --release --offline -p singlefs-core --lib "$@" > $log 2>&1
echo "exit $?" >> $log
