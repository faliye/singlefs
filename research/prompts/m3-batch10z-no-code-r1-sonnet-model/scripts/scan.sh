#!/usr/bin/env bash
# scan.sh <gpu|crash> <文件> : 经内存包装跑扫源码模型
W=$HOME/.local/share/singlefs/work/m3-batch10z-no-code-r1-sonnet
R=/home/fy5090/code/singlefs/scripts
nice -n 19 bash $R/run-with-memory-cap.sh 1G bash $R/capped.sh 1 $W/target-scan2/release/scan-model "$1" "$2" 2>&1 | tail -1
