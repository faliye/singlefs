#!/usr/bin/env bash
# arun.sh <pqrs|tu> <log名> <测试名> <环境变量…>：经内存包装（3G）跑攻方原型的一个测试，线程 3
W=$HOME/.local/share/singlefs/work/m3-batch10z-vw-code-r1-sonnet
R=/home/fy5090/code/singlefs/scripts
which=$1; name=$2; test=$3; shift 3
B=$(ls -t $W/target-${TGT:-a}/release/deps/atk_${which}_vw-* | grep -v '\.d$' | head -1)
cd "$W/tree-${TGT:-a}"
env "$@" nice -n 19 bash $R/run-with-memory-cap.sh 3G bash $R/capped.sh 3 "$B" --ignored --nocapture --exact "$test" > "$W/runs/$name.log" 2>&1
echo $? > "$W/runs/$name.rc"
