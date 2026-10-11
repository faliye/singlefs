#!/usr/bin/env bash
# usage: run.sh <treedir> <log> -- <test args...>；差分装置经内存包装 6G（派发上限 12G 之内，整机排不上 12G）、线程 10
T=$1; shift; cd $T
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="派发提示给定内存上限十二G，差分草稿测试按提示上限之内的六G跑"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="同会话派出的几个腿各跑各的，这一条是差分草稿测试"
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS=1500
log=$1; shift
nice -n 19 bash /home/fy5090/code/singlefs/scripts/hosts/run-with-memory-cap.sh 6G bash /home/fy5090/code/singlefs/scripts/hosts/capped.sh 10 cargo test --release --offline -p singlefs-core --lib "$@" > $log 2>&1
echo "exit $?" >> $log
