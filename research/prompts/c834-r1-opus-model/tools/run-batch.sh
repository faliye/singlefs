#!/usr/bin/env bash
# 跑一批件：run-batch.sh <二进制> <件表> <批号> <每批件数> <输出目录>。每件把退出码写进 <输出目录>/b<批号>-<件号>.rc，最后不带参数 wait 等齐。
set -euo pipefail
binary="$1" pieces="$2" batch="$3" per_batch="$4" out="$5"
rm -f "$out"/b"$batch"-*.rc
first=$((batch * per_batch))
mapfile -t lines < <(sed -n "$((first + 1)),$((first + per_batch))p" "$pieces")
index=0
for line in "${lines[@]}"; do
  read -r candidate prefix start end <<<"$line"
  { C834_CANDIDATE="$candidate" nice -n 19 "$binary" "$prefix" run "$start" "$end" 1 > "$out/b$batch-$index.out" 2> "$out/b$batch-$index.err"; echo "$?" > "$out/b$batch-$index.rc"; } &
  index=$((index + 1))
done
wait
echo "batch=$batch launched=$index rc_files=$(ls "$out"/b"$batch"-*.rc 2>/dev/null | wc -l)"
for ((i = 0; i < index; i++)); do echo "piece=$i rc=$(cat "$out/b$batch-$i.rc")"; done
