#!/usr/bin/env bash
set -u
base=/tmp/claude-1000/m2-supp3-item1/r3
cd $base/y2/repo || exit 1
for mode in off a b; do
  start=$(date +%s)
  OPUS_Y2=$mode nice -n 19 cargo test --release -p singlefs-harness --test second_transaction_supplement_three_random_history -- random_histories_fast_tier reuse_heavy --nocapture > $base/y2/two-tests-$mode.log 2>&1
  echo "y2 two-tests mode $mode exit $? seconds $(( $(date +%s) - start ))" >> $base/y2.done
done
start=$(date +%s)
OPUS_Y2=a SINGLEFS_RANDOM_HISTORY_SEEDS=200 SINGLEFS_RANDOM_HISTORY_OPERATIONS=30 SINGLEFS_RANDOM_HISTORY_THREADS=32 SINGLEFS_RANDOM_HISTORY_WEIGHTS=reuse SINGLEFS_RANDOM_HISTORY_SHRINK=every \
  nice -n 19 cargo test --release -p singlefs-harness --test second_transaction_supplement_three_random_history -- --ignored random_histories_large_tier --nocapture > $base/y2/large-a-reuse-200x30-shrunk.log 2>&1
echo "y2 large a reuse 200x30 shrink exit $? seconds $(( $(date +%s) - start ))" >> $base/y2.done
echo all-done >> $base/y2.done
