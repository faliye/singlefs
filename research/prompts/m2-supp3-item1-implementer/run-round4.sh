#!/usr/bin/env bash
set -u
base=/tmp/claude-1000/m2-supp3-item1
run_debug() {
  local copy=$1 filter=$2 label=$3
  cd $base/$copy/repo || return 1
  local start=$(date +%s)
  nice -n 19 cargo test -p singlefs-harness --test second_transaction_supplement_three_random_history -- $filter --nocapture > $base/$copy/round4-$label.log 2>&1
  echo "$copy $label exit $? seconds $(( $(date +%s) - start ))" >> $base/round4.done
}
run_debug explore random_histories_fast_tier fast-tier
run_debug proposal "" whole-binary
run_debug proposal-mutant-41 random_histories_fast_tier fast-tier
run_debug proposal-mutant-121 random_histories_fast_tier fast-tier
run_debug mutant-41 random_histories_fast_tier fast-tier
run_debug mutant-121 random_histories_fast_tier fast-tier
cd $base/proposal/repo || exit 1
start=$(date +%s)
SINGLEFS_RANDOM_HISTORY_SEEDS=3000 SINGLEFS_RANDOM_HISTORY_OPERATIONS=40 SINGLEFS_RANDOM_HISTORY_THREADS=16 \
  nice -n 19 cargo test --release -p singlefs-harness --test second_transaction_supplement_three_random_history -- --ignored random_histories_large_tier --nocapture \
  > $base/proposal/round4-large-tier-3000x40.log 2>&1
echo "proposal large tier 3000x40 exit $? seconds $(( $(date +%s) - start ))" >> $base/round4.done
