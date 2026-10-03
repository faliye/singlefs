#!/usr/bin/env bash
# 一批：给定一组 标签=流目录，切成 N 片并行跑崩溃之后可写挂载的探针，每片退出码写进自己的 .rc，最后不带参数的 wait 等齐、数 .rc。
# 用法：run_crash_probe_batch.sh <探针二进制> <批号> <片数> <输出目录> <批次(如 1,2)> <标签=目录>...
set -u
probe=$1; batch=$2; shards=$3; out=$4; batches=$5; shift 5
spec=$(IFS=,; echo "$*")
rm -f "$out"/b"$batch"-*.rc
for ((i = 0; i < shards; i++)); do
  { OPUS_PROBE_DUMPS="$spec" OPUS_PROBE_BATCHES="$batches" OPUS_PROBE_SHARD="$i/$shards" OPUS_PROBE_OUT="$out/b$batch-$i.tsv" \
      nice -n 19 "$probe" --test-threads 1 > "$out/b$batch-$i.log" 2>&1; echo "$?" > "$out/b$batch-$i.rc"; } &
done
wait
count=$(ls "$out"/b"$batch"-*.rc 2>/dev/null | wc -l)
if [ "$count" -ne "$shards" ]; then echo "batch=$batch rc_files=$count expected=$shards VOID"; exit 1; fi
for ((i = 0; i < shards; i++)); do echo "batch=$batch shard=$i rc=$(cat "$out/b$batch-$i.rc")"; done
