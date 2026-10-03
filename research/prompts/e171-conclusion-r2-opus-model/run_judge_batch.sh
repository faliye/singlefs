#!/usr/bin/env bash
# e171-conclusion-r2 攻方腿：一批判官探针切成 N 片并行跑（照 r1 攻方 run_crash_probe_batch.sh 的收法）：每片退出码写进自己的 .rc，
# 跑之前删掉这一批的 .rc，最后一个不带参数的 wait 等齐，数 .rc 与片数对得上才按片序读。
# 用法：run_judge_batch.sh <探针二进制> <批号> <片数> <输出目录> <批次(如 1,2；空串=全部)> <标签=目录>...
# 别的环境变量（OPUS_JUDGE_COMMITTED、OPUS_JUDGE_REFFS、OPUS_JUDGE_SKIP_MOUNT）由调用方设好传进来。
set -u
probe=$1; batch=$2; shards=$3; out=$4; batches=$5; shift 5
spec=$(IFS=,; echo "$*")
rm -f "$out"/j"$batch"-*.rc
for ((i = 0; i < shards; i++)); do
  { OPUS_JUDGE_DUMPS="$spec" OPUS_JUDGE_BATCHES="$batches" OPUS_JUDGE_SHARD="$i/$shards" OPUS_JUDGE_OUT="$out/j$batch-$i.tsv" \
      nice -n 19 "$probe" --test-threads 1 > "$out/j$batch-$i.log" 2>&1; echo "$?" > "$out/j$batch-$i.rc"; } &
done
wait
count=$(ls "$out"/j"$batch"-*.rc 2>/dev/null | wc -l)
if [ "$count" -ne "$shards" ]; then echo "batch=$batch rc_files=$count expected=$shards VOID"; exit 1; fi
for ((i = 0; i < shards; i++)); do echo "batch=$batch shard=$i rc=$(cat "$out/j$batch-$i.rc")"; done
