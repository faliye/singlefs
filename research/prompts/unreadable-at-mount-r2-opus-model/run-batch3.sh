#!/usr/bin/env bash
# 批 3：甲-读错-每盘（jiaeiodev）的 C331 网格与两部分轨迹，三件并行，每件写 .rc；整条脚本由调用方经内存包装起。
set -u
W=/tmp/claude-1000/unreadable-at-mount-r2/opus
rm -f "$W"/scratch/b3-*.rc
{ cd "$W/jiaeiodev" && TMPDIR="$W/tmp" CARGO_TARGET_DIR="$W/target-jiaeiodev" nice -n 19 cargo test --offline --release -p singlefs-harness --test c331_candidates_grid -- --nocapture > "$W/runs/jiaeiodev-c331_candidates_grid.log" 2>&1; echo "$?" > "$W/scratch/b3-1.rc"; } &
{ nice -n 19 "$W/target-jiaeiodev/release/r2-opus-trajectory-proto" torn > "$W/runs/jiaeiodev-traj-torn.log" 2>&1; echo "$?" > "$W/scratch/b3-2.rc"; } &
{ nice -n 19 "$W/target-jiaeiodev/release/r2-opus-trajectory-proto" single_slot > "$W/runs/jiaeiodev-traj-single_slot.log" 2>&1; echo "$?" > "$W/scratch/b3-3.rc"; } &
wait
count=$(ls "$W"/scratch/b3-*.rc | wc -l)
echo "batch=3 pieces=3 rc_files=$count"
[ "$count" -eq 3 ] || { echo "batch=3 作废"; exit 1; }
for piece in 1 2 3; do echo "piece=$piece rc=$(cat "$W/scratch/b3-$piece.rc")"; done
