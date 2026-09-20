#!/usr/bin/env bash
# 主工作区、debug：随机历史那两段各自单跑（同一二进制里别的测试滤掉），量墙钟；再在 release 下各跑一次（门禁 74 号跑 release）。
D=/tmp/claude-1000/m2-supp3-item2-r2-fix-implementer
cd /home/fy5090/code/singlefs || exit 2
for profile in debug release; do
  flag=""; [[ $profile == release ]] && flag="--release"
  nice -n 19 cargo test $flag -p singlefs-harness --test second_transaction_supplement_three_random_history --no-run > /dev/null 2>&1
  for filter in allocation_record_wall_sampling unit_area_wall_sampling random_histories_fast_tier; do
    log=$D/logs/time-$profile-$filter.log
    /usr/bin/time -f 'wall %e s, user %U s, sys %S s' nice -n 19 cargo test $flag -p singlefs-harness --test second_transaction_supplement_three_random_history -- $filter --nocapture > $log 2>&1
    echo "$profile $filter exit $?: $(grep -E '^用时' $log) $(tail -1 $log)"
  done
done
