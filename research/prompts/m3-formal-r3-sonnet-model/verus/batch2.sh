#!/usr/bin/env bash
set -u
cd /tmp/claude-1000/m3-formal-r3-sonnet/verus
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="三方复核辩方腿与同会话别的任务各跑各的"
nice -n 19 bash /home/fy5090/code/singlefs/scripts/run-with-memory-cap.sh 1G bash run-verus-batch.sh $(ls -d variants2/*/ | sed 's:/$::') > batch2.out 2>&1
echo "exit=$?" >> batch2.out
cp summary.txt summary-variants2.txt
