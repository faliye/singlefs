#!/usr/bin/env bash
# piece.sh <批号> <件名> <探针: s2|jk> <环境变量…>
set -u
W=$HOME/.local/share/singlefs/work/m3-batch10z-lm-code-r1-sonnet
R=/home/fy5090/code/singlefs/scripts
batch=$1; name=$2; probe=$3; shift 3
mkdir -p "$W/runs/$batch"
if [ "$probe" = s2 ]; then B=$W/target-core/release/deps/s2_probe-4473cb0b5afd0bf9; T=s2_sweep; else B=$W/target-core/release/deps/probe_jk-fe83cfe73387f21b; T=probe_sweep; fi
cd "$W/tree-core"
env "$@" nice -n 19 bash $R/run-with-memory-cap.sh 3G bash $R/capped.sh 1 "$B" --nocapture $T > "$W/runs/$batch/$name.log" 2>&1
echo "$?" > "$W/runs/$batch/$name.rc"
