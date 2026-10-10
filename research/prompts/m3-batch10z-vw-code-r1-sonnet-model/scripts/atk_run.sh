#!/usr/bin/env bash
# atk_run.sh <log名> <测试名> <环境变量…>
W=$HOME/.local/share/singlefs/work/m3-batch10z-vw-code-r1-sonnet
name=$1; test=$2; shift 2
cd "$W/tree-b"
env "$@" nice -n 19 bash /home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh 3G bash /home/fy5090/code/singlefs/research/scripts/capped.sh 3 "$(cat $W/runs/atkbin)" --ignored --nocapture --exact "$test" > "$W/runs/$name.log" 2>&1
echo $? > "$W/runs/$name.rc"
