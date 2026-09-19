#!/usr/bin/env bash
set -u
base=/tmp/claude-1000/m2-supp3-item1
for copy in proposal proposal-mutant-41 proposal-mutant-121; do
  cd $base/$copy/repo || exit 1
  start=$(date +%s)
  nice -n 19 cargo test -p singlefs-harness --test second_transaction_supplement_three_random_history -- random_histories_fast_tier --nocapture \
    > $base/$copy/fast-tier-debug.log 2>&1
  status=$?
  echo "$copy fast tier debug exit $status seconds $(( $(date +%s) - start ))" >> $base/round3.done
done
cd $base/proposal/repo || exit 1
start=$(date +%s)
SINGLEFS_RANDOM_HISTORY_SEEDS=3000 SINGLEFS_RANDOM_HISTORY_OPERATIONS=40 SINGLEFS_RANDOM_HISTORY_THREADS=16 \
  nice -n 19 cargo test --release -p singlefs-harness --test second_transaction_supplement_three_random_history -- --ignored random_histories_large_tier --nocapture \
  > $base/proposal/large-tier-3000x40.log 2>&1
echo "proposal large tier 3000x40 exit $? seconds $(( $(date +%s) - start ))" >> $base/round3.done
