#!/usr/bin/env bash
# 用法：random-batch.sh <仓根> <树根（含 target/release 里编好的 attack_r2_random_histories）> <日志目录> <批号> <比重> <起始种子下标> <件数> <每件段数> [变量=值 …]
# 与 m4-segment-code-r2-opus-model/tools/rerun.sh 里的 run_batch 同一写法：每件经内存包装（2G）直接跑测试二进制，
# 退出码写进 <日志目录>/b<批号>-<件号>.rc，最后单独一个不带参数的 wait 等齐，再数 .rc 文件。
set -u
repo=$1; tree=$2; log=$3; batch=$4; weights=$5; start=$6; pieces=$7; per=$8; shift 8
cap="$repo/scripts/hosts/run-with-memory-cap.sh"
bin="$(ls -t "$tree"/target/release/deps/attack_r2_random_histories-* | grep -v '\.d$' | head -1)"
mkdir -p "$log"
rm -f "$log"/b"$batch"-*.rc
for piece in $(seq 0 $((pieces - 1))); do
  { piece_rc=0
    env "$@" ATTACK_WEIGHTS="$weights" ATTACK_SEED_START=$((start + piece * per)) ATTACK_SEEDS="$per" ATTACK_OPS=40 nice -n 19 \
      bash "$cap" 2G "$bin" --nocapture > "$log/b$batch-$piece.log" 2>&1 || piece_rc=$?
    echo "$piece_rc" > "$log/b$batch-$piece.rc"; } &
done
wait
count=$(ls "$log"/b"$batch"-*.rc | wc -l)
echo "BATCH $batch weights=$weights start=$start pieces=$pieces per=$per rc_files=$count"
[ "$count" -eq "$pieces" ] || { echo "BATCH $batch 作废：rc 文件数对不上"; exit 1; }
for piece in $(seq 0 $((pieces - 1))); do
  echo "piece $piece rc=$(cat "$log/b$batch-$piece.rc") $(grep -h ATTACK_SUMMARY "$log/b$batch-$piece.log")"
done
