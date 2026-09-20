#!/usr/bin/env bash
# run-long.sh: 攻方的长历史设置（checker 关掉、96 段 × 200 步、broad / reuse 两组比重），在仓副本上把大档那条用例临时改成不跑 checker；
# base 与 W1 各一份副本、各用各的 target，release。
set -uo pipefail
D=/tmp/claude-1000/m2-supp3-item2-fix-implementer
ROOT=/home/fy5090/code/singlefs
PROGRESS=$D/logs/long-progress.txt
: > "$PROGRESS"
for label in long-base long-w1; do
  copy=$D/copies/$label
  rm -rf "$copy"
  rsync -a --exclude target --exclude .git "$ROOT"/ "$copy"/
  python3 - "$copy" <<'PY'
import sys, pathlib
p = pathlib.Path(sys.argv[1]) / 'crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs'
s = p.read_text(encoding='utf-8')
old = '        &weights,\n        PerStepChecker::Run,\n        worker_threads,\n        shrinking,'
assert s.count(old) == 1, s.count(old)
p.write_text(s.replace(old, old.replace('PerStepChecker::Run', 'PerStepChecker::Skipped')), encoding='utf-8')
print('large tier switched to Skipped')
PY
  if [[ "$label" == long-w1 ]]; then
    python3 $D/scripts/apply-tsv-row.py "$copy" $D/scripts/new-rows.tsv "$(sed -n 1p $D/scripts/new-rows.tsv | cut -f1)"
  fi
  for weights in broad reuse; do
    log=$D/logs/$label-$weights.log
    ( cd "$copy" && SINGLEFS_RANDOM_HISTORY_SEEDS=96 SINGLEFS_RANDOM_HISTORY_OPERATIONS=200 SINGLEFS_RANDOM_HISTORY_WEIGHTS=$weights SINGLEFS_RANDOM_HISTORY_SHRINK=none \
      nice -n 19 cargo test --release -p singlefs-harness --test second_transaction_supplement_three_random_history -- --ignored --exact random_histories_large_tier_seeds_and_length_from_the_environment --nocapture > "$log" 2>&1; echo "exit $?" >> "$log" )
    echo "$(date -u +%H:%M:%S) $label $weights $(tail -1 "$log")；$(grep -m1 '^历史 ' "$log")" >> "$PROGRESS"
  done
done
echo "$(date -u +%H:%M:%S) 全部跑完" >> "$PROGRESS"
