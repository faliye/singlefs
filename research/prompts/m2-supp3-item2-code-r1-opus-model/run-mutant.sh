#!/usr/bin/env bash
# Usage: run-mutant.sh <id>   (id from model/mutants.tsv, or "base" for the unmutated instrumented copy)
# Makes copies/<id> from copies/base (instrumented by attack-patch.py), applies the mutation (old text must hit exactly once),
# builds the random-history test binary in release with its own target, runs three suites on the three campaign segments:
#   std          as shipped
#   noharness    ATTACK_NO_HARNESS=1 (item-1 harness judgements dropped)
#   modelonly    ATTACK_NO_HARNESS=1 ATTACK_NO_CHECKER=1 (only model disagreement and panics stop a history)
set -u
D=/tmp/claude-1000/m2-supp3-item2-code-r1-opus
id=$1
C=$D/copies/$id
L=$D/logs/$id
mkdir -p "$L"
if [ "$id" != base ]; then
  rsync -a --delete --exclude target "$D/copies/base/" "$C/"
  python3 "$D/model/apply-mutation.py" "$C" "$id" > "$L/apply.log" 2>&1 || { echo "$id apply failed"; cat "$L/apply.log"; exit 3; }
fi
( cd "$C" && CARGO_TARGET_DIR="$C/target" nice -n 19 cargo test --release -p singlefs-harness --test second_transaction_supplement_three_random_history --no-run > "$L/build.log" 2>&1 ) || { echo "$id build failed"; tail -20 "$L/build.log"; exit 4; }
BIN=$(ls -t "$C"/target/release/deps/second_transaction_supplement_three_random_history-* | grep -v '\.d$' | head -1)
echo "$BIN" > "$L/bin.txt"
TESTS="random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation reuse_heavy_random_histories_cover_released_records_and_end_only_in_known_red_forms rollback_heavy_random_histories_reach_the_floor_root_and_end_only_in_known_red_forms"
for suite in std noharness modelonly; do
  envs=()
  case $suite in noharness) envs=(ATTACK_NO_HARNESS=1);; modelonly) envs=(ATTACK_NO_HARNESS=1 ATTACK_NO_CHECKER=1);; esac
  for t in $TESTS; do
    env "${envs[@]}" nice -n 19 "$BIN" --exact "$t" --test-threads 1 > "$L/$suite-$t.log" 2>&1
    echo "$id $suite ${t%%_end_only*} exit=$? $(grep -m1 -E '^历史 ' "$L/$suite-$t.log")"
  done
done
