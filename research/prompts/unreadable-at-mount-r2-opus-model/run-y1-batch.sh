#!/usr/bin/env bash
# unreadable-at-mount-r2 云端攻方腿：Y1 轨迹原型分批并行跑。用法：run-y1-batch.sh <批号> <副本:部分>...
# 整条脚本由调用方经 run-with-memory-cap.sh 16G 与 capped.sh 16 起；每件写自己的日志与 .rc。
set -u
W=/tmp/claude-1000/unreadable-at-mount-r2/opus
batch=$1; shift
rm -f "$W"/scratch/b"$batch"-*.rc
piece=0
for item in "$@"; do
  piece=$((piece + 1))
  copy=${item%%:*}; part=${item#*:}
  { nice -n 19 "$W/target-$copy/release/r2-opus-trajectory-proto" "$part" > "$W/runs/$copy-traj-$part.log" 2>&1; echo "$?" > "$W/scratch/b$batch-$piece.rc"; } &
done
wait
count=$(ls "$W"/scratch/b"$batch"-*.rc | wc -l)
echo "batch=$batch pieces=$piece rc_files=$count"
if [ "$count" -ne "$piece" ]; then echo "batch=$batch 作废：rc 文件数与派出去的件数对不上"; exit 1; fi
piece=0
for item in "$@"; do piece=$((piece + 1)); echo "piece=$piece item=$item rc=$(cat "$W/scratch/b$batch-$piece.rc")"; done
