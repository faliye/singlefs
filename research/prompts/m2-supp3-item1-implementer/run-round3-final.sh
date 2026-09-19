#!/usr/bin/env bash
set -u
base=/tmp/claude-1000/m2-supp3-item1
cd /home/fy5090/code/singlefs || exit 1
sha256sum crates/singlefs-harness/src/history.rs crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs crates/singlefs-harness/src/crash.rs crates/singlefs-harness/src/lib.rs crates/mutations.tsv > $base/round3-final.sha256
ps -o pid,etime,args -u "$(id -u)" | grep -E 'cargo|gate.sh|qemu|vm-bench|e152|fio' | grep -v grep > $base/round3-final.ps
date -u +%H:%M:%SZ > $base/check-sh-round3.start
nice -n 19 bash .claude/scripts/check.sh > $base/check-sh-round3.log 2>&1
echo "exit $?" >> $base/check-sh-round3.log
date -u +%H:%M:%SZ >> $base/check-sh-round3.start
date -u +%H:%M:%SZ > $base/gate59-round3.start
GATE59_LOG_PREFIX=gate59-round3 python3 $base/gate59-rows.py /home/fy5090/code/singlefs 143 144 145 > $base/gate59-round3.log 2>&1
echo "exit $?" >> $base/gate59-round3.log
date -u +%H:%M:%SZ >> $base/gate59-round3.start
echo done > $base/round3-final.done
