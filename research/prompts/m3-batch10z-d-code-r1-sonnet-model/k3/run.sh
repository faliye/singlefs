#!/usr/bin/env bash
# 用法: run.sh <变异名>  施加 -> 跑 core lib 与 harness 档目标 -> 还原；结果摘要写 out/<名>.txt
set -u
W=$HOME/.local/share/singlefs/work/m3-batch10z-d-code-r1-sonnet
R=/home/fy5090/code/singlefs
export CARGO_TARGET_DIR=$W/target
name=$1
mkdir -p $W/k3/out
out=$W/k3/out/$name.txt
: > $out
python3 $W/k3/mut.py $name >> $out 2>&1 || { echo "apply failed" >> $out; exit 1; }
trap "python3 $W/k3/mut.py $name restore >> $out 2>&1" EXIT
cd $W/repo
run() { # label, args...
  label=$1; shift
  timeout ${STAGE_TIMEOUT:-105} nice -n 19 bash $R/research/scripts/run-with-memory-cap.sh 12G bash $R/research/scripts/capped.sh 6 cargo test --offline "$@" > $W/k3/out/$name.$label.log 2>&1
  rc=$?
  echo "## $label rc=$rc" >> $out
  grep -E "^error(\[|:)|test result|^test .* FAILED|panicked at|could not compile" $W/k3/out/$name.$label.log | cut -c1-260 >> $out
}
run core -p singlefs-core --lib
run raise -p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table --test an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping --no-fail-fast -- --test-threads 14
if [ "${WIDE:-0}" = 1 ]; then
run wide -p singlefs-harness --test raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount --test admission_raises_the_floor_before_refusing --test root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread --test writable_mount_keeps_a_released_slot_named_by_an_unapplied_record_isolated_until_its_root_ring_slot_is_overwritten --test raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window --no-fail-fast -- --test-threads 14
fi
