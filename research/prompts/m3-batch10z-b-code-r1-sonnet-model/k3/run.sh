#!/usr/bin/env bash
# 用法：run.sh <变异名|baseline> ；在 repo-k3 上施加变异，跑一串 harness 档目标，写 k3/out/<名>.txt，最后还原。
set -u
W=$HOME/.local/share/singlefs/work/m3-batch10z-b-code-r1-sonnet
name=$1
mkdir -p $W/k3/out
out=$W/k3/out/$name.txt
: > $out
if [ "$name" != baseline ]; then python3 $W/k3/mut.py "$name" >> $out 2>&1 || { echo "施加失败" >> $out; exit 3; }; fi
cd $W/repo-k3
export CARGO_TARGET_DIR=$W/target-k3
run() {
  label=$1; shift
  nice -n 19 bash /home/fy5090/code/singlefs/scripts/run-with-memory-cap.sh 12G bash /home/fy5090/code/singlefs/scripts/capped.sh 8 cargo test --offline "$@" > $W/k3/out/$name.$label.log 2>&1
  rc=$?
  echo "## $label rc=$rc :: $(grep -E '^test result' $W/k3/out/$name.$label.log | tr '\n' ' ')" >> $out
  grep -E '^test .* FAILED$' $W/k3/out/$name.$label.log | sed 's/^/   FAILED: /' >> $out
  grep -E '^error(\[|:)' $W/k3/out/$name.$label.log | head -3 | sed 's/^/   BUILD: /' >> $out
}
stages=${STAGES:-lib newfile lr3 credential slotbad reuse onepass}
for st in $stages; do
  case $st in
    lib) run lib -p singlefs-core --lib ;;
    newfile) run newfile -p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table ;;
    lr3) run lr3 -p singlefs-harness --test an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping -- a_floor_raise_whose_shadow_ledger an_abandoned_root_with_a_two_page ;;
    lr3full) run lr3full -p singlefs-harness --test an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping ;;
    credential) run credential -p singlefs-harness --test abandoned_root_whose_account_is_unreadable_passes_when_the_current_account_records_later_units_over_its_slots ;;
    slotbad) run slotbad -p singlefs-harness --test unreadable_abandoned_root_slot ;;
    reuse) run reuse -p singlefs-harness --test reuse_after_raising_the_floor ;;
    reuseslow) run reuseslow -p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored raising_the_floor_counts_abandoned_roots_whose_ledger_is_unreadable ;;
    onepass) run onepass -p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root ;;
    corrupt) run corrupt -p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking -- an_abandoned_root ;;
  esac
done
if [ "$name" != baseline ]; then python3 $W/k3/mut.py "$name" restore >> $out 2>&1; fi
echo done >> $out
