#!/usr/bin/env bash
# 接手腿的扫描：F9 / F10 × N3 各臂 × （用户前四步全枚举 + 固定尾巴），F8 / F10 × N2 各臂 × 前四步，F9 × N1 各臂 × 前四步。
set -u
BIN=$1; OUT=$2; HERE=$(dirname "$0")
run() { TMPDIR=/dev/shm/m2-newq-opus NEWQ_OUT="$OUT/$4-$1-$2.tsv" NEWQ_FAULT="$1" NEWQ_ARM="$2" NEWQ_SUFFIX="$(cat "$HERE/$3")" NEWQ_WORKERS=8 nice -n 19 "$BIN" --nocapture 2>&1 | grep 'test result' | sed "s/^/$4 $1 $2: /"; }
for f in F10 F9; do for a in 'off(today)' 'N1A-any-isoAll-mem' 'N3-iso-recompute' 'N3-iso-persisted' 'N3-keep-allocated'; do
  run "$f" "$a" suffixes-P4-M-FO15.txt tailM
  run "$f" "$a" suffixes-P4-FO15.txt tailFO
done; done
for f in F8 F10; do for a in 'off(today)' 'N1A-any-isoAll-mem' 'N2-rebuild-checks-mapping-refuses' 'N2-same-verdict-rebuild-tolerates-data'; do
  run "$f" "$a" suffixes-P4.txt p4
done; done
for a in 'N1A-any-isoAll-mem' 'N1B-any' 'N1C-any' 'N2-same-verdict-rebuild-tolerates-data'; do run F9 "$a" suffixes-P4.txt p4; done
date -u
