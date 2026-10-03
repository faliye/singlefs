#!/usr/bin/env bash
# 用法：n2-batch.sh <批号> <二进制> <SLOTS 取值…>   每个单元区槽数一件，OVERWRITES/UNITS/BATCHES 照环境变量；MODE=n2。
set -u
batch=$1; binary=$2; shift 2
dir=${ATTACK_DIR:-/tmp/claude-1000/m3-unlink-code-r3-opus}
rm -f "$dir"/b"$batch"-*.rc
piece=0
for value in "$@"; do
  piece=$((piece + 1))
  { MODE=n2 SLOTS="$value" "$dir/run.sh" "$binary" "$dir/out/n2-$(basename "$(dirname "$(dirname "$binary")")")-$value-k${OVERWRITES//,/_}.log"; echo "$?" > "$dir/b$batch-$piece.rc"; } &
done
wait
count=$(ls "$dir"/b"$batch"-*.rc 2>/dev/null | wc -l)
echo "batch=$batch pieces=$piece rc_files=$count"
for i in $(seq 1 "$piece"); do echo "piece=$i rc=$(cat "$dir/b$batch-$i.rc")"; done
