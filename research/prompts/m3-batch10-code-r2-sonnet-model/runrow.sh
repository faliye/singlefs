#!/usr/bin/env bash
# usage: runrow.sh <row>  ; applies mutation row in ./work, runs its test args, restores
set -u
n=$1
W=/tmp/claude-1000/m3-batch10-code-r2-sonnet
cd $W/work
export CARGO_TARGET_DIR=$W/target RUN_WITH_MEMORY_CAP_PARALLEL_REASON="多个子 agent 各跑各的，正推腿在自己的目录跑 harness 档用例" RUN_WITH_MEMORY_CAP_WAIT_SECONDS=200 RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="变异后重编比测试占内存"
args=$(awk -F'\t' -v n=$n 'NR==n{print $5}' /tmp/claude-1000/m3-batch10-code-r2/tree/crates/mutations.tsv)
python3 $W/mut2.py apply $n | head -1
timeout 200 nice -n 19 bash /home/fy5090/code/singlefs/scripts/run-with-memory-cap.sh 2G bash /home/fy5090/code/singlefs/scripts/capped.sh 7 cargo test --offline $args > $W/row-$n.log 2>&1
echo "row $n exit $?"
python3 $W/mut2.py restore $n
grep -E 'test result|^test .*(FAILED|ok)$' $W/row-$n.log | cut -c1-220
