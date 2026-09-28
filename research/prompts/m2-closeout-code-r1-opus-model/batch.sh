#!/usr/bin/env bash
# usage: batch.sh <batch-no> <first-seed> <seeds> <steps>  — runs Z1 walk on widths 4g and 384 in parallel
D=/tmp/claude-1000/m2-closeout-code-r1-opus
B=$1; F=$2; N=$3; S=$4
T=opus_r1_z1_rollback_walk_with_multi_unit_files_inodes_and_unmount
W="bash research/scripts/run-with-memory-cap.sh 6G"
C="bash research/scripts/capped.sh 2"
export CARGO_TARGET_DIR=$D/target
cd $D/tree
rm -f $D/b$B-*.rc
{ OPUS_Z1_WIDTH=4g OPUS_Z1_FIRST_SEED=$F OPUS_Z1_SEEDS=$N OPUS_Z1_STEPS=$S nice -n 19 $C $W cargo test -p singlefs-harness --release --test $T -- --nocapture > $D/b$B-1.log 2>&1; echo "$?" > $D/b$B-1.rc; } &
{ OPUS_Z1_WIDTH=384 OPUS_Z1_FIRST_SEED=$F OPUS_Z1_SEEDS=$N OPUS_Z1_STEPS=$S nice -n 19 $C $W cargo test -p singlefs-harness --release --test $T -- --nocapture > $D/b$B-2.log 2>&1; echo "$?" > $D/b$B-2.rc; } &
wait
echo "rc_files=$(ls $D/b$B-*.rc | wc -l)"
cat $D/b$B-1.rc $D/b$B-2.rc
