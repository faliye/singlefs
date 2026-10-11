#!/usr/bin/env bash
# 一个变异（mutants.py 的名字，或 row:<crates/mutations.tsv 行号>）：run-mutant.sh build|run <变异> <工作目录>
# build：从冻结副本另拷一份到 <工作目录>/repo-mut、改、拼原型，编 harness 那份测试文件、atk_xy_opus 与 core 库单测（只编不跑，经内存包装 12G、线程 7）。
# run：跑那份 harness 测试文件整份（--test-threads 4）与 core 库单测整份；row: 的只跑那一行点名的用例。每件经 run-with-memory-cap.sh。
# 输出：<工作目录>/runs/mut-<名>-{build,file,core,row}.log；最后一行打 test result 行。
set -uo pipefail
ROOT=/home/fy5090/code/singlefs
MODEL=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
FROZEN=$HOME/.local/share/singlefs/work/m3-batch10z-xy-code-r1/tree
phase=${1:?}; mutant=${2:?}; WORK=${3:?}
CAP=$ROOT/scripts/hosts/run-with-memory-cap.sh
THREADS=$ROOT/scripts/hosts/capped.sh
OUT=$WORK/runs; mkdir -p "$OUT"
tag=${mutant//:/-}
export CARGO_TARGET_DIR=$WORK/target-mut
HARNESS=raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table
if [ "$phase" = build ]; then
  rsync -a --delete --exclude target --exclude .git "$FROZEN"/ "$WORK"/repo-mut/
  if [ "${mutant#row:}" != "$mutant" ]; then
    python3 "$MODEL"/apply-row.py "$WORK"/repo-mut "${mutant#row:}" > "$WORK"/row-args.txt || exit 3
  elif [ "$mutant" != baseline ]; then
    python3 "$MODEL"/mutants.py "$WORK"/repo-mut "$mutant" || exit 3
  fi
  bash "$MODEL"/assemble.sh "$WORK"/repo-mut "$MODEL"
  ( cd "$WORK"/repo-mut && bash "$CAP" 12G nice -n 19 bash "$THREADS" 7 cargo test --offline --release -p singlefs-harness --test "$HARNESS" --test atk_xy_opus --no-run \
    && bash "$CAP" 12G nice -n 19 bash "$THREADS" 7 cargo test --offline --release -p singlefs-core --lib --no-run ) > "$OUT/mut-$tag-build.log" 2>&1
  echo "build $mutant rc=$?"
  exit 0
fi
newest() { ls -t "$CARGO_TARGET_DIR"/release/deps/"$1"-* | grep -v '\.d$' | head -1; }
if [ "${mutant#row:}" != "$mutant" ]; then
  mapfile -t row < "$WORK"/row-args.txt
  ( cd "$WORK"/repo-mut && bash "$CAP" 12G nice -n 19 bash "$THREADS" 4 cargo test --offline --release ${row[0]} ) > "$OUT/mut-$tag-row.log" 2>&1
  echo "$mutant | ${row[1]} | $(grep -h '^test result' "$OUT/mut-$tag-row.log" | tail -1)"
  exit 0
fi
bash "$CAP" 12G nice -n 19 bash "$THREADS" 4 "$(newest $HARNESS)" --test-threads 4 > "$OUT/mut-$tag-file.log" 2>&1
bash "$CAP" 4G nice -n 19 bash "$THREADS" 2 "$(newest singlefs_core)" --test-threads 2 > "$OUT/mut-$tag-core.log" 2>&1
echo "$mutant | file: $(grep -h '^test result' "$OUT/mut-$tag-file.log" | tail -1) | core: $(grep -h '^test result' "$OUT/mut-$tag-core.log" | tail -1)"
