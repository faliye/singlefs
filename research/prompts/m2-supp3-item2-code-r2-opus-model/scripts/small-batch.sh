#!/usr/bin/env bash
# 小盘副本（单元区 1280 槽、两块等大）：基线与 m2h / m2j 下跑门禁 74 号那个测试二进制与探针。
D=/tmp/claude-1000/m2-supp3-item2-code-r2-opus
S=$D/small
export CARGO_TARGET_DIR=$D/target-small
cd $S || exit 2
run() { # $1 名
  local L=$D/logs/small-$1
  nice -n 19 cargo test --release -p singlefs-harness --test second_transaction_supplement_three_random_history -- --nocapture > $L-gate74bin.log 2>&1 </dev/null; echo "$1 gate74bin exit $?"
  grep -E '^──|模型对拍|新发现 |Err 成员 .*Placement|^test .*(ok|FAILED)$' $L-gate74bin.log | cut -c1-260
  for w in wall wall_plain wall_rb; do
    PROBE_THREADS=24 PROBE_WEIGHTS=$w PROBE_FIRST=0 PROBE_SEEDS=256 PROBE_STEPS=150 PROBE_CHECKER=skip nice -n 19 cargo test --release -p singlefs-harness --test opus_r2_probe -- --ignored --nocapture > $L-probe-$w.log 2>&1 </dev/null; echo "$1 probe $w exit $?"
    grep 'PROBE SUMMARY' $L-probe-$w.log; grep -c "ending=NEW" $L-probe-$w.log; grep "ending=NEW" $L-probe-$w.log | sed -n "s/.*implementation_answer: \"\([^\"]*\)\".*/\1/p" | sort | uniq -c | head; grep "^seed" $L-probe-$w.log | sed -n "s/.*placement_refusals_seen_by_observer=\([0-9]*\).*/\1/p" | awk "{s+=\$1} END {print \"placement_refusals_seen=\" s}"
  done
}
date -u
run base
for m in m2h-alloc-nofree-as-somefull-userdata m2j-alloc-nofree-as-somefull-commit; do
  M=$D/mutants/$m; f=$(cat $M/file)
  python3 $D/probe/mutate.py --nth "$(cat $M/nth)" "$f" "$M/old" "$M/new"
  run $m
  python3 $D/probe/mutate.py --restore "$f"
done
date -u
