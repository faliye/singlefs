#!/usr/bin/env bash
set -u
cd /tmp/claude-1000/m3-formal-r3-sonnet
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="三方复核辩方腿与同会话别的任务各跑各的"
for attempt in $(seq 1 400); do
  nice -n 19 bash /home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh 6G bash /home/fy5090/code/singlefs/research/scripts/capped.sh 4 bash master.sh > master.out 2> master.err
  code=$?
  echo "attempt=$attempt exit=$code" > master.exit
  if [ "$code" != 252 ]; then break; fi
  sleep 20
done
