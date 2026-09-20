#!/usr/bin/env bash
# 小盘段选比重：每组比重在基线与 m2h、m2j 下各跑一批种子（release，探针在副本里）。
D=/tmp/claude-1000/m2-supp3-item2-r2-fix-implementer
P=$D/copies/probe
export CARGO_TARGET_DIR=$P/target
cd $P || exit 2
FIRST=${FIRST:-0}; SEEDS=${SEEDS:-128}; STEPS=${STEPS:-150}; CHECKER=${CHECKER:-continue}
run() { # $1 标签 $2 比重
  local log=$D/logs/probe-unit-$1-$2-$CHECKER-$FIRST-$SEEDS-$STEPS.log
  PROBE_WEIGHTS=$2 PROBE_CHECKER=$CHECKER PROBE_WIDTH=small PROBE_FIRST=$FIRST PROBE_SEEDS=$SEEDS PROBE_STEPS=$STEPS PROBE_THREADS=24 \
    nice -n 19 cargo test --release -p singlefs-harness --test probe_unit_area_wall -- --ignored --nocapture > $log 2>&1
  echo "$1 $2 exit $?: $(grep 'PROBE SUMMARY' $log)"
  grep -E 'Err 成员 .*PlacementRefused|模型对拍|已知红第|checker 跑了' $log | sed 's/^/    /'
}
date -u
for weights in ${WEIGHTS:-unit wall more-mounts dense}; do
  run base $weights
  for m in ${MUTANTS:-m2h-alloc-nofree-as-somefull-userdata m2j-alloc-nofree-as-somefull-commit}; do
    M=$D/mutants/$m
    python3 $D/scripts/mutate.py apply $P "$(cat $M/file)" $M/old $M/new "$(cat $M/nth)" || exit 3
    run $m $weights
    python3 $D/scripts/mutate.py restore $P "$(cat $M/file)"
  done
done
date -u
