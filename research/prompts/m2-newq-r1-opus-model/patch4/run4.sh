#!/usr/bin/env bash
# 接手腿第四版扫描：N1 候选丁两臂 × F1/F2/F3/F9 × 前四步；F11 × 四臂 × 前四步；F9 × 候选丁留在已分配 × 两种尾巴。
set -u
BIN=$1; OUT=$2; HERE=$(dirname "$0")
run() { TMPDIR=/dev/shm/m2-newq-opus NEWQ_OUT="$OUT/$4-$1-$2.tsv" NEWQ_FAULT="$1" NEWQ_ARM="$2" NEWQ_SUFFIX="$(cat "$HERE/$3")" NEWQ_WORKERS=8 nice -n 19 "$BIN" --nocapture 2>&1 | grep 'test result' | sed "s/^/$4 $1 $2: /"; }
for f in F1- F2- F3- F9-; do for a in 'N1D-reread-then-mismatch-iso-mem' 'N1D-reread-then-mismatch-keep-allocated'; do run "$f" "$a" suffixes-P4.txt p4; done; done
for a in 'off(today)' 'N1A-any-isoAll-mem' 'N3-keep-allocated' 'N1D-reread-then-mismatch-keep-allocated'; do run F11 "$a" suffixes-P4.txt p4; done
run F9- 'N1D-reread-then-mismatch-keep-allocated' suffixes-P4-M-FO15.txt tailM
run F9- 'N1D-reread-then-mismatch-keep-allocated' suffixes-P4-FO15.txt tailFO
date -u
