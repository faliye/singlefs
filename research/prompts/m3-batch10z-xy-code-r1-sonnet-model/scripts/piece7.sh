#!/usr/bin/env bash
# piece7.sh <log名> <环境变量…>  经内存包装（1500M，线程 1）起一件 p10z 探针
set -u
W=$HOME/.local/share/singlefs/work/m3-batch10z-xy-code-r1-sonnet
R=/home/fy5090/code/singlefs/research/scripts
name=$1; shift
B=$(ls -t $W/target-a/release/deps/p10z_probe-* | grep -v '\.d$' | head -1)
cd "$W/tree-a"
env "$@" nice -n 19 bash $R/run-with-memory-cap.sh 1500M bash $R/capped.sh 1 "$B" --nocapture s2_sweep > "$W/runs/$name.log" 2>&1
echo "$?" > "$W/runs/$name.rc"
