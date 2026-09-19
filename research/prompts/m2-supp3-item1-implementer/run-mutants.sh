#!/usr/bin/env bash
set -u
for number in 41 121; do
  cd /tmp/claude-1000/m2-supp3-item1/mutant-$number/repo || exit 1
  SINGLEFS_RANDOM_HISTORY_SEEDS=200 SINGLEFS_RANDOM_HISTORY_OPERATIONS=30 SINGLEFS_RANDOM_HISTORY_THREADS=12 \
    nice -n 19 cargo test --release -p singlefs-harness --test second_transaction_supplement_three_random_history -- --ignored --nocapture \
    > /tmp/claude-1000/m2-supp3-item1/mutant-$number/run-200x30.log 2>&1
  echo "mutant $number exit $?" >> /tmp/claude-1000/m2-supp3-item1/mutant-runs.done
done
