#!/usr/bin/env bash
# 用法: run-probe.sh <输出文件> 其余环境变量 PROBE_GRID/PROBE_DISKS/PROBE_O/PROBE_U/PROBE_B/PROBE_K
set -euo pipefail
cd /tmp/claude-1000/m3-unmount-forward/tree
BIN=$(ls -t target/debug/deps/probe_unmount-* | grep -v '\.d$' | head -1)
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="正推腿是派出去的子 agent，与主 agent 的整包慢档各跑各的"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="探针格越大用得越多，峰值表里的数只是一次短跑的数"
timeout 235 bash /home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh 2G bash /home/fy5090/code/singlefs/research/scripts/capped.sh 10 nice -n 19 "$BIN" --nocapture probe_grid > "$1" 2>&1
