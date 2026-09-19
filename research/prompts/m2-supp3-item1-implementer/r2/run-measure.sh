#!/usr/bin/env bash
set -u
base=/tmp/claude-1000/m2-supp3-item1/r2
large() {
  local name=$1 seeds=$2 operations=$3 label=$4
  cd $base/$name/repo || return 1
  local start=$(date +%s)
  SINGLEFS_RANDOM_HISTORY_SEEDS=$seeds SINGLEFS_RANDOM_HISTORY_OPERATIONS=$operations SINGLEFS_RANDOM_HISTORY_THREADS=16 \
  SINGLEFS_RANDOM_HISTORY_WEIGHTS=reuse SINGLEFS_RANDOM_HISTORY_SHRINK=none \
    nice -n 19 cargo test --release -p singlefs-harness --test second_transaction_supplement_three_random_history -- --ignored random_histories_large_tier --nocapture \
    > $base/$name/$label.log 2>&1
  echo "$name $label exit $? seconds $(( $(date +%s) - start ))" >> $base/measure.done
}
fast() {
  local name=$1
  cd $base/$name/repo || return 1
  local start=$(date +%s)
  nice -n 19 cargo test --release -p singlefs-harness --test second_transaction_supplement_three_random_history -- random_histories_fast_tier --nocapture > $base/$name/fast-release.log 2>&1
  echo "$name fast-release exit $? seconds $(( $(date +%s) - start ))" >> $base/measure.done
}
large base 6000 30 reuse-6000x30
large m121 6000 30 reuse-6000x30
fast base
fast n2
fast b6
fast b1
fast a1
echo all-done >> $base/measure.done
