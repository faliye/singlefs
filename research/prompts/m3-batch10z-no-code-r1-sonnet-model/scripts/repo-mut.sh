#!/usr/bin/env bash
# repo-mut.sh <变异名…> : 套变异、编入库用例文件、跑（含 ignored），写 runs/RM/<名>.log，打印失败的用例名
set -u
W=$HOME/.local/share/singlefs/work/m3-batch10z-no-code-r1-sonnet
R=/home/fy5090/code/singlefs/research/scripts
mkdir -p $W/runs/RM
for name in "$@"; do
  python3 $W/scripts/mutate.py $name > /dev/null || { echo "$name APPLYFAIL"; continue; }
  cd $W/tree-mut
  export CARGO_TARGET_DIR=$W/target-mut
  nice -n 19 bash $R/run-with-memory-cap.sh 12G bash $R/capped.sh 8 cargo test --release --offline -p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table --no-run > $W/runs/RM/$name.build 2>&1 || { echo "$name BUILDFAIL"; continue; }
  B=$(ls -t $W/target-mut/release/deps/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table-* | grep -v '\.d$' | head -1)
  nice -n 19 bash $R/run-with-memory-cap.sh 6G bash $R/capped.sh 4 $B --include-ignored > $W/runs/RM/$name.log 2>&1
  rc=$?
  echo "$name rc=$rc passed=$(grep -c ' \.\.\. ok$' $W/runs/RM/$name.log) failed=$(grep -c ' \.\.\. FAILED$' $W/runs/RM/$name.log)"
  grep ' \.\.\. FAILED$' $W/runs/RM/$name.log | sed 's/^test /    FAILED: /; s/ \.\.\. FAILED//' | cut -c1-200
done
