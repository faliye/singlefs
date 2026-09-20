#!/usr/bin/env bash
# 改之前的树（开工时拷的副本）：debug 下跑随机历史那个测试二进制，量四段各自的用时（报告里比「跑 checker 之后第四段的耗时」用）。
D=/tmp/claude-1000/m2-supp3-item2-r2-fix-implementer
cd $D/copies/before || exit 2
export CARGO_TARGET_DIR=$D/copies/before/target
date -u
nice -n 19 cargo test -p singlefs-harness --test second_transaction_supplement_three_random_history --no-run 2>&1 | tail -3
/usr/bin/time -v nice -n 19 cargo test -p singlefs-harness --test second_transaction_supplement_three_random_history -- --nocapture > $D/logs/before-random-history-debug.log 2>&1
echo "exit $?"
grep -E '^── |用时|^test result' $D/logs/before-random-history-debug.log
date -u
