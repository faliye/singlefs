#!/usr/bin/env bash
# arun2.sh <树后缀 a|k14> <目标名> <log名> <测试名> <环境变量…>：经内存包装（2G，线程 1）跑一个攻方原型 / 我的测试；rc 写 runs/<log名>.rc
W=$HOME/.local/share/singlefs/work/m3-batch10z-xy-code-r1-sonnet
R=/home/fy5090/code/singlefs/scripts
tree=$1; target=$2; name=$3; test=$4; shift 4
B=$(ls -t $W/target-$tree/release/deps/$target-* | grep -v '\.d$' | head -1)
cd "$W/tree-$tree"
env "$@" nice -n 19 bash $R/run-with-memory-cap.sh 2G bash $R/capped.sh 1 "$B" --ignored --nocapture --exact "$test" > "$W/runs/$name.log" 2>&1
echo $? > "$W/runs/$name.rc"
