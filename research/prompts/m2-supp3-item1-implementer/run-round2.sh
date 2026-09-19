#!/usr/bin/env bash
set -u
base=/tmp/claude-1000/m2-supp3-item1
for number in 41 121; do
  cd $base/mutant-$number/repo || exit 1
  start=$(date +%s)
  nice -n 19 cargo test -p singlefs-harness --test second_transaction_supplement_three_random_history -- random_histories_fast_tier --nocapture \
    > $base/mutant-$number/fast-tier-debug.log 2>&1
  status=$?
  echo "mutant $number fast tier debug exit $status seconds $(( $(date +%s) - start ))" >> $base/round2.done
done
cd $base/explore/repo || exit 1
start=$(date +%s)
SINGLEFS_RANDOM_HISTORY_SEEDS=1000 SINGLEFS_RANDOM_HISTORY_OPERATIONS=40 SINGLEFS_RANDOM_HISTORY_THREADS=16 \
  nice -n 19 cargo test --release -p singlefs-harness --test second_transaction_supplement_three_random_history -- --ignored --nocapture \
  > $base/explore/run-1000x40.log 2>&1
echo "explore 1000x40 exit $? seconds $(( $(date +%s) - start ))" >> $base/round2.done
