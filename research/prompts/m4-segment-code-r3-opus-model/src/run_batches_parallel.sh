#!/usr/bin/env bash
# 一批并行跑 H4 原型：参数 批号 宽 比重 起始下标 件数 每件段数；每件的退出码写 b<批号>-<件号>.rc，最后不带参数的 wait 等齐再数。
set -u
batch="$1"; width="$2"; weights="$3"; first="$4"; pieces="$5"; per="$6"
dir=/home/fy5090/.local/share/singlefs/work/m4-segment-code-r3-attack/logs
bin=/home/fy5090/.local/share/singlefs/work/m4-segment-code-r3-attack/repo/target/release/deps/attack_r3_compaction_batches-60b31fd73cc9d4e3
rm -f "$dir"/b"$batch"-*.rc
for piece in $(seq 0 $((pieces - 1))); do
  start=$((first + piece * per))
  { ATTACK_WIDTH="$width" ATTACK_WEIGHTS="$weights" ATTACK_SEED_START="$start" ATTACK_SEEDS="$per" \
      nice -n 19 bash /home/fy5090/.local/share/singlefs/work/m4-predev/scripts/hosts/run-with-memory-cap.sh 2G "$bin" --nocapture > "$dir/b$batch-$piece.log" 2>&1; echo "$?" > "$dir/b$batch-$piece.rc"; } &
done
wait
count=$(ls "$dir"/b"$batch"-*.rc | wc -l)
echo "batch=$batch rc_files=$count pieces=$pieces"
for piece in $(seq 0 $((pieces - 1))); do echo "piece=$piece rc=$(cat "$dir/b$batch-$piece.rc")"; done
