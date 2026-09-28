#!/usr/bin/env bash
# 用法：run-arm.sh <control|mutated> <日志名> <原型参数…>
set -u
arm=$1; log=$2; shift 2
cd /tmp/claude-1000/k3-9-barrier-r1-opus
if [ "$arm" = mutated ]; then export K39_DROP_UNIT_RECORD_BARRIER=1; else unset K39_DROP_UNIT_RECORD_BARRIER; fi
export SINGLEFS_LAYER0_THREADS=10
start=$(date +%s)
nice -n 19 repo/target/release/k39-barrier-attack "$@" > "runs/$log-$arm.log" 2>&1
rc=$?
echo "WALL $(( $(date +%s) - start ))s rc=$rc" >> "runs/$log-$arm.log"
exit $rc
