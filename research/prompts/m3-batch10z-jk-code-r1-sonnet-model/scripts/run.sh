#!/usr/bin/env bash
# run.sh <label> <arm> [ENV=val ...]  -> runs/<label>.log
set -u
W=$HOME/.local/share/singlefs/work/m3-batch10z-jk-code-r1-sonnet
R=/home/fy5090/code/singlefs/research/scripts
label=$1; arm=$2; shift 2
mkdir -p $W/runs
cd $W/tree-p
env PROBE_LABEL=$label SINGLEFS_PROBE_ARM=$arm "$@" nice -n 19 bash $R/run-with-memory-cap.sh 4G bash $R/capped.sh 1 $W/target-p/release/deps/probe_jk-fe83cfe73387f21b --nocapture probe_sweep > $W/runs/$label-$arm.log 2>&1
echo $? > $W/runs/$label-$arm.rc
