#!/usr/bin/env bash
# 用法：run-probe.sh <仓副本目录> <日志名> <weights> <first> <seeds> <steps> <skip|observe>
set -u
repo="$1"; name="$2"
cd "$repo" || exit 2
logdir=/tmp/claude-1000/m2-supp3-item2-code-r2-opus/logs
PROBE_WEIGHTS="$3" PROBE_FIRST="$4" PROBE_SEEDS="$5" PROBE_STEPS="$6" PROBE_CHECKER="$7" PROBE_THREADS=24 \
CARGO_TARGET_DIR=/tmp/claude-1000/m2-supp3-item2-code-r2-opus/target \
nice -n 19 cargo test --release -p singlefs-harness --test opus_r2_probe -- --ignored --nocapture > "$logdir/$name.log" 2>&1
echo "EXIT $?" >> "$logdir/$name.log"
grep -h 'PROBE SUMMARY\|^EXIT' "$logdir/$name.log"
