#!/usr/bin/env bash
# 每行 checker 档变异（crates/mutations.tsv 2362–2367）：在副本里换上变异后的 crash_injection.rs，cargo check --tests（只编不跑），
# 看变异后的源码编不编得过（编不过的「红」是假红）。结束后还原副本。
set -u
W=/tmp/claude-1000/m3-formal-code-b20-r1-sonnet/work
export CARGO_TARGET_DIR=/tmp/claude-1000/m3-formal-code-b20-r1-sonnet/target
R=/home/fy5090/code/singlefs
M=$R/research/prompts/m3-formal-code-b20-r1-sonnet-model
OUT=/tmp/claude-1000/m3-formal-code-b20-r1-sonnet/check-mutants; mkdir -p $OUT
F=crates/singlefs-checker-tier/src/crash_injection.rs
cp $W/$F $OUT/orig.rs
python3 $M/anchor_check.py /tmp/claude-1000/m3-formal-code-b20-r1/tree 2362 2367 --emit $OUT/emit > $OUT/anchors.txt
cd $W
for row in 2362 2363 2364 2365 2366 2367; do
  cp $OUT/emit/row$row.rs $W/$F
  timeout 200 bash $R/research/scripts/capped.sh 10 nice -n 19 cargo check --offline -p singlefs-checker-tier --tests > $OUT/row$row.log 2>&1
  echo "row$row cargo check rc=$? warnings=$(grep -c '^warning' $OUT/row$row.log) errors=$(grep -c '^error' $OUT/row$row.log)" >> $OUT/summary.txt
done
cp $OUT/orig.rs $W/$F; cmp $OUT/orig.rs $W/$F && echo "副本已还原" >> $OUT/summary.txt
