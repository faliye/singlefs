#!/usr/bin/env bash
# piece.sh <log名> <环境变量…>  经内存包装起一件 p10z 探针
set -u
W=$HOME/.local/share/singlefs/work/m3-batch10z-pqrs-code-r1-sonnet
R=/home/fy5090/code/singlefs/scripts
name=$1; shift
mkdir -p "$W/runs-mut"
B=$(ls -t $W/target-ctl/release/deps/p10z_probe-* | grep -v '\.d$' | head -1)
cd "$W/tree-a"
env "$@" nice -n 19 bash $R/run-with-memory-cap.sh 3G bash $R/capped.sh 1 "$B" --nocapture s2_sweep > "$W/runs-mut/$name.log" 2>&1
echo "$?" > "$W/runs-mut/$name.rc"
