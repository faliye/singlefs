#!/usr/bin/env bash
# /dev/shm 在 03:27 UTC 前后被三路并发扫描写满（镜像文件每个约 0.7 GB 实占），那之后起跑的几组要重跑：一次只跑一组。
set -u
D=/tmp/claude-1000/m2-newq-opus
B3=$D/target3/debug/deps/m2_newq_attack3-d70f2ca914405fb7
B4=$D/target4/debug/deps/m2_newq_attack4-47d791a332be7289
B2=$D/target2/debug/deps/m2_newq_attack-c059cac0a3913059
run() { TMPDIR=/dev/shm/m2-newq-opus NEWQ_OUT="$5/$4-$2-$3.tsv" NEWQ_FAULT="$2" NEWQ_ARM="$3" NEWQ_SUFFIX="$(cat "$6")" NEWQ_WORKERS=8 nice -n 19 "$1" --nocapture 2>&1 | grep 'test result' | sed "s/^/$4 $2 $3: /"; df -h /dev/shm | tail -1; }
mkdir -p $D/out3r $D/out4r $D/outr
run $B3 F10 N3-iso-recompute tailM $D/out3r $D/patch3/suffixes-P4-M-FO15.txt
run $B3 F10 N3-iso-recompute tailFO $D/out3r $D/patch3/suffixes-P4-FO15.txt
bash $D/patch4/run4.sh $B4 $D/out4r
for f in F5 F3; do for a in 'N1A-every-isoAll-mem' 'N1A-every-isoBad-mem'; do
  [ "$f-$a" = "F5-N1A-every-isoAll-mem" ] && continue
  run $B2 "$f" "$a" long $D/outr $D/patch/suffixes-len7-O.txt
done; done
date -u
