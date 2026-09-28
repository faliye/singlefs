#!/usr/bin/env bash
# 批 4：(b)-从映射现算（bmap）的 C393 网格与两部分轨迹，三件并行，每件写 .rc；整条脚本由调用方经内存包装起。
set -u
W=/tmp/claude-1000/unreadable-at-mount-r2/opus
rm -f "$W"/scratch/b4-*.rc
{ cd "$W/bmap" && TMPDIR="$W/tmp" CARGO_TARGET_DIR="$W/target-bmap" SINGLEFS_R2_B_COMPARE=1 nice -n 19 cargo test --offline --release -p singlefs-harness --test c393_candidates_grid -- --nocapture > "$W/runs/bmap-c393_candidates_grid.log" 2>&1; echo "$?" > "$W/scratch/b4-1.rc"; } &
{ nice -n 19 "$W/target-bmap/release/r2-opus-trajectory-proto" c393_single > "$W/runs/bmap-traj-c393_single.log" 2>&1; echo "$?" > "$W/scratch/b4-2.rc"; } &
{ nice -n 19 "$W/target-bmap/release/r2-opus-trajectory-proto" c393_double > "$W/runs/bmap-traj-c393_double.log" 2>&1; echo "$?" > "$W/scratch/b4-3.rc"; } &
wait
count=$(ls "$W"/scratch/b4-*.rc | wc -l)
echo "batch=4 pieces=3 rc_files=$count"
[ "$count" -eq 3 ] || { echo "batch=4 作废"; exit 1; }
for piece in 1 2 3; do echo "piece=$piece rc=$(cat "$W/scratch/b4-$piece.rc")"; done
