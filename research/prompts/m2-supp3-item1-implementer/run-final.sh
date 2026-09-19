#!/usr/bin/env bash
set -u
base=/tmp/claude-1000/m2-supp3-item1
cd $base/proof || exit 1
echo "== 第四轮（最终代码，全部重跑）" >> proofs.log
sha256sum repo/crates/singlefs-harness/src/history.rs repo/crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs repo/crates/singlefs-harness/src/crash.rs >> proofs.log
python3 run-proofs.py
cd $base/proposal/repo || exit 1
start=$(date +%s)
nice -n 19 cargo test -p singlefs-harness --test second_transaction_supplement_three_random_history -- --nocapture > $base/proposal/final-whole-binary.log 2>&1
echo "proposal final whole binary exit $? seconds $(( $(date +%s) - start ))" >> $base/final.done
cd /home/fy5090/code/singlefs || exit 1
date -u +%H:%M:%SZ > $base/check-sh-final.start
nice -n 19 bash .claude/scripts/check.sh > $base/check-sh-final.log 2>&1
echo "exit $?" >> $base/check-sh-final.log
date -u +%H:%M:%SZ >> $base/check-sh-final.start
echo "check.sh final done" >> $base/final.done
