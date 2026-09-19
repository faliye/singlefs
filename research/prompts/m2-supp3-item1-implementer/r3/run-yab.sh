#!/usr/bin/env bash
set -u
base=/tmp/claude-1000/m2-supp3-item1/r3
for name in ya yb; do
  cd $base/$name/repo || exit 1
  start=$(date +%s)
  nice -n 19 cargo test -p singlefs-harness --test second_transaction_supplement_three_random_history -- --nocapture > $base/$name/debug-whole-binary.log 2>&1
  echo "$name debug whole binary exit $? seconds $(( $(date +%s) - start ))" >> $base/yab.done
done
echo all-done >> $base/yab.done
