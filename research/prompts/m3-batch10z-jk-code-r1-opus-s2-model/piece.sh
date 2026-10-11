#!/usr/bin/env bash
# 一件：piece.sh <批号> <件名> <环境变量…>；日志 runs/<批号>/<件名>.log，退出码 runs/<批号>/<件名>.rc
set -u
S=$HOME/.local/share/singlefs/work/m3-batch10z-jk-code-r1-opus-s2
batch=$1; name=$2; shift 2
mkdir -p "$S/runs/$batch"
B=$(ls -t $S/target/release/deps/opus_s2_probe-* | grep -v '\.d$' | head -1)
cd "$S"
env "$@" nice -n 19 bash /home/fy5090/code/singlefs/scripts/run-with-memory-cap.sh 4G bash /home/fy5090/code/singlefs/scripts/capped.sh 1 "$B" --nocapture s2_sweep > "$S/runs/$batch/$name.log" 2>&1
echo "$?" > "$S/runs/$batch/$name.rc"
