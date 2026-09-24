#!/usr/bin/env bash
# 长后缀扫描：首步 O（让核验走到故障那个单元），其后六步 O/M/F 全枚举，729 条。
set -u
BIN=$1; OUT=$2
SUF=$(cat "$(dirname "$0")/suffixes-len7-O.txt")
run() { TMPDIR=/dev/shm/m2-newq-opus NEWQ_OUT="$OUT/long-$1-$2.tsv" NEWQ_FAULT="$1" NEWQ_ARM="$2" NEWQ_SUFFIX="$SUF" NEWQ_WORKERS=4 nice -n 19 "$BIN" --nocapture 2>&1 | grep 'test result'; }
for f in F6 F4; do for a in 'off(today)' 'N1A-any-isoAll-mem' 'N3-iso-recompute' 'N3-iso-persisted' 'N3-keep-allocated'; do run "$f" "$a"; done; done
for f in F5 F3; do for a in 'N1A-every-isoAll-mem' 'N1A-every-isoBad-mem'; do run "$f" "$a"; done; done
date -u
