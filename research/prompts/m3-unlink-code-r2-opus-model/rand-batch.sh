#!/usr/bin/env bash
# 用法：rand-batch.sh <批名> <盘宽> <比重> <种子基> <种子数> <步数>
set -u
cd /tmp/claude-1000/m3-unlink-r2-attack/tree
RUN_WITH_MEMORY_CAP_WAIT_SECONDS=150 RUN_WITH_MEMORY_CAP_PARALLEL_REASON='派出的攻方腿在自己的副本上跑原型，与别的腿各跑各的' \
ATTACK_SEED_BASE="$4" ATTACK_SEEDS="$5" ATTACK_OPERATIONS="$6" ATTACK_EXECUTION="$2" ATTACK_WEIGHTS="$3" \
CARGO_TARGET_DIR=/tmp/claude-1000/m3-unlink-r2-attack/target nice -n 19 \
bash /home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh 480M \
bash /home/fy5090/code/singlefs/research/scripts/capped.sh 6 \
cargo test --release --offline -p singlefs-harness --test attack_r2_random_probe -- --nocapture \
> "/tmp/claude-1000/m3-unlink-r2-attack/rand-$1.log" 2>&1
echo "batch=$1 rc=$?"
L="/tmp/claude-1000/m3-unlink-r2-attack/rand-$1.log"
grep -c '^R ' "$L"
grep '^R ' "$L" | grep -o 'ending=[A-Za-z0-9]*' | sort | uniq -c
grep '^ATTACK' "$L" | sed -E 's/[0-9]+/N/g' | cut -c1-110 | sort | uniq -c
