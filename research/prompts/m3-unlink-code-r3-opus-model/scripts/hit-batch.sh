#!/usr/bin/env bash
# 用法：hit-batch.sh <批号> <二进制> <日志前缀>   每份 hitpiece.NN.joined 一件并行跑（FAMILY=list），退出码进 b<批号>-<件号>.rc。
set -u
batch=$1; binary=$2; prefix=$3
dir=${ATTACK_DIR:-/tmp/claude-1000/m3-unlink-code-r3-opus}
rm -f "$dir"/b"$batch"-*.rc
piece=0
for file in "$dir"/hitpiece.${4:-}*.joined; do
  piece=$((piece + 1))
  name=$(basename "$file" .joined)
  { FAMILY=list SHAPES="$(cat "$file")" "$dir/run.sh" "$binary" "$dir/out/$prefix-$name.log"; echo "$?" > "$dir/b$batch-$piece.rc"; } &
done
wait
count=$(ls "$dir"/b"$batch"-*.rc 2>/dev/null | wc -l)
echo "batch=$batch pieces=$piece rc_files=$count"
for i in $(seq 1 "$piece"); do echo "piece=$i rc=$(cat "$dir/b$batch-$i.rc")"; done
