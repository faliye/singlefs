#!/usr/bin/env bash
# 接手核现场：主工作区跑整道 check.sh，再把第四段、小盘段、快档在 debug 与 release 下各单跑一次量墙钟。
D=/tmp/claude-1000/m2-supp3-item2-r2-fix-implementer
cd /home/fy5090/code/singlefs || exit 2
date -u > $D/logs/check-handover.log
/usr/bin/time -v nice -n 19 bash .claude/scripts/check.sh >> $D/logs/check-handover.log 2>&1
echo "exit $?" >> $D/logs/check-handover.log
date -u >> $D/logs/check-handover.log
for profile in debug release; do
  flag=""
  [[ $profile == release ]] && flag="--release"
  nice -n 19 cargo test $flag -p singlefs-harness --test second_transaction_supplement_three_random_history --no-run > /dev/null 2>&1
  for filter in allocation_record_wall_sampling unit_area_wall_sampling random_histories_fast_tier; do
    log=$D/logs/handover-time-$profile-$filter.log
    /usr/bin/time -f 'wall %e s, user %U s, sys %S s' nice -n 19 cargo test $flag -p singlefs-harness --test second_transaction_supplement_three_random_history -- "$filter" --nocapture > "$log" 2>&1
    echo "$profile $filter exit $?: $(grep -E '^用时' "$log") $(tail -1 "$log")"
  done
done
