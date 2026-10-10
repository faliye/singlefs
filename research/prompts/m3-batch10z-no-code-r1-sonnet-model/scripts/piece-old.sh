#!/usr/bin/env bash
# piece.sh <批号> <件名> <环境变量…>  经内存包装起一件 s2 探针
set -u
W=$HOME/.local/share/singlefs/work/m3-batch10z-no-code-r1-sonnet
R=/home/fy5090/code/singlefs/research/scripts
batch=$1; name=$2; shift 2
mkdir -p "$W/runs/$batch"
B=$(ls -t $W/target-old/release/deps/s2_probe-* | grep -v '\.d$' | head -1)
cd "$W/tree-old"
env "$@" nice -n 19 bash $R/run-with-memory-cap.sh 3G bash $R/capped.sh 1 "$B" --nocapture s2_sweep > "$W/runs/$batch/$name.log" 2>&1
echo "$?" > "$W/runs/$batch/$name.rc"
