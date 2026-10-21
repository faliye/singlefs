#!/usr/bin/env bash
# 用法：run_random_batch.sh <批号> <比重> <起始种子下标> <件数> <每件段数> <每段步数>
# 每件经内存包装直接跑随机历史原型的测试二进制，退出码写进 logs/b<批号>-<件号>.rc；最后单独一个 wait 等齐，数 .rc。
set -u
W=/home/fy5090/.local/share/singlefs/work/m4-segment-code-r2-attack
R=/home/fy5090/.local/share/singlefs/work/m4-predev
batch=$1; weights=$2; start=$3; pieces=$4; per=$5; ops=$6
B=$(ls -t "$W"/repo/target/release/deps/attack_r2_random_histories-* | grep -v '\.d$' | head -1)
rm -f "$W"/logs/b"$batch"-*.rc
for piece in $(seq 0 $((pieces - 1))); do
  s=$((start + piece * per))
  { ATTACK_WEIGHTS="$weights" ATTACK_SEED_START=$s ATTACK_SEEDS=$per ATTACK_OPS=$ops nice -n 19 \
      bash "$R"/scripts/hosts/run-with-memory-cap.sh 2G "$B" --nocapture > "$W"/logs/b"$batch"-"$piece".log 2>&1
    echo "$?" > "$W"/logs/b"$batch"-"$piece".rc; } &
done
wait
count=$(ls "$W"/logs/b"$batch"-*.rc | wc -l)
echo "BATCH $batch weights=$weights start=$start pieces=$pieces rc_files=$count"
if [ "$count" -ne "$pieces" ]; then echo "BATCH $batch 作废：rc 文件数对不上"; exit 1; fi
for piece in $(seq 0 $((pieces - 1))); do
  echo "piece $piece rc=$(cat "$W"/logs/b"$batch"-"$piece".rc) $(grep -h ATTACK_SUMMARY "$W"/logs/b"$batch"-"$piece".log)"
done
