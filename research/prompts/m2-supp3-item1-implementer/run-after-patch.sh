#!/usr/bin/env bash
set -u
base=/tmp/claude-1000/m2-supp3-item1
cd /home/fy5090/code/singlefs || exit 1
sha256sum crates/singlefs-harness/src/history.rs crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs crates/singlefs-harness/src/crash.rs crates/singlefs-harness/src/lib.rs crates/mutations.tsv > $base/after-patch.sha256
date -u +%H:%M:%SZ > $base/check-sh-after-patch.start
nice -n 19 bash .claude/scripts/check.sh > $base/check-sh-after-patch.log 2>&1
echo "exit $?" >> $base/check-sh-after-patch.log
date -u +%H:%M:%SZ >> $base/check-sh-after-patch.start
date -u +%H:%M:%SZ > $base/gate59-rows.start
python3 $base/gate59-rows.py /home/fy5090/code/singlefs 129 130 > $base/gate59-rows.log 2>&1
echo "exit $?" >> $base/gate59-rows.log
date -u +%H:%M:%SZ >> $base/gate59-rows.start
echo done > $base/after-patch.done
