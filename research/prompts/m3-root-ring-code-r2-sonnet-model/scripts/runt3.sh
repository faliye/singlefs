#!/usr/bin/env bash
# 用法：runt3.sh <整条命令…>  同 runt2，等内存最多 200 秒，上限 6G
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON='复核变异行在冻结副本上编译要余量' RUN_WITH_MEMORY_CAP_PARALLEL_REASON='同一会话派出的正推腿与攻方腿各跑各的' RUN_WITH_MEMORY_CAP_WAIT_SECONDS=200
cd /tmp/claude-1000/m3-root-ring-code-r2-sonnet/work/tree
exec nice -n 19 bash /home/fy5090/code/singlefs/research/scripts/capped.sh 15 bash /home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh 6G "$@"
