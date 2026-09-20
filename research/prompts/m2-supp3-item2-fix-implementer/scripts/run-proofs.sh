#!/usr/bin/env bash
# run-proofs.sh: base copy + one copy per mutation (each its own target); each runs the whole test binaries listed, records failing tests.
set -uo pipefail
D=/tmp/claude-1000/m2-supp3-item2-fix-implementer
ROOT=/home/fy5090/code/singlefs
ROWS=$D/scripts/new-rows.tsv
PROGRESS=$D/logs/proof-progress.txt
LIB="-p singlefs-harness --lib"
HISTORY="-p singlefs-harness --test second_transaction_supplement_three_random_history"
UNEQUAL="-p singlefs-harness --test second_transaction_supplement_two_unequal_devices"
STEP4="-p singlefs-harness --test second_transaction_step_four_rollback"
row_name() { sed -n "${1}p" "$ROWS" | cut -f1; }
run_case() {
  local label="$1" row="$2"; shift 2
  local copy=$D/copies/$label
  rm -rf "$copy"
  rsync -a --exclude target --exclude .git "$ROOT"/ "$copy"/
  if [[ -n "$row" ]]; then
    python3 $D/scripts/apply-tsv-row.py "$copy" "$ROWS" "$(row_name "$row")" >> $D/logs/proof-$label.log 2>&1 || { echo "$(date -u +%H:%M:%S) $label: 施加失败" >> "$PROGRESS"; return; }
  fi
  local binary_index=0
  for args in "$@"; do
    binary_index=$((binary_index + 1))
    local log=$D/logs/proof-$label-$binary_index.log
    ( cd "$copy" && echo "cargo test $args" > "$log" && nice -n 19 cargo test $args >> "$log" 2>&1; echo "exit $?" >> "$log" )
    local failed
    failed=$(grep -E '^test .* \.\.\. FAILED$' "$log" | sed 's/^test //; s/ \.\.\. FAILED$//' | tr '\n' ' ')
    echo "$(date -u +%H:%M:%S) $label [$args] $(tail -1 "$log")；红：${failed:-（无）}" >> "$PROGRESS"
  done
}
mkdir -p $D/copies
: > "$PROGRESS"
run_case base "" "$LIB" "$HISTORY" "$UNEQUAL" "$STEP4"
run_case w1 1 "$HISTORY"
run_case model-lower-end 3 "$LIB"
run_case model-base-root 4 "$LIB"
run_case checker-half 5 "$HISTORY"
run_case r1 6 "$HISTORY" "$STEP4"
run_case glue-exclusion 7 "$HISTORY" "$LIB"
run_case core-placement 8 "$UNEQUAL"
run_case glue-placement 9 "$LIB"
echo "$(date -u +%H:%M:%S) 全部跑完" >> "$PROGRESS"
