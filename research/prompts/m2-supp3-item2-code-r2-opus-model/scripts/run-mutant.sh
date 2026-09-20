#!/usr/bin/env bash
# 用法：run-mutant.sh <变异名>   变异定义在 mutants/<名>/{file,old,new[,nth],extra}
set -u
D=/tmp/claude-1000/m2-supp3-item2-code-r2-opus
M=$D/mutants/$1
R=$D/mut
L=$D/logs/mutant-$1.log
export CARGO_TARGET_DIR=$D/target-mut
file="$(cat "$M/file")"
cd "$R" || exit 2
{
  date -u
  echo "mutant $1 on $file"
  if [[ -f $M/nth ]]; then python3 $D/probe/mutate.py --nth "$(cat $M/nth)" "$file" "$M/old" "$M/new"; else python3 $D/probe/mutate.py "$file" "$M/old" "$M/new"; fi || { echo "APPLY FAILED"; exit 3; }
  diff -u "$file.orig" "$file"
  echo "=== gate-74 binary (four segments) ==="
  nice -n 19 cargo test --release -p singlefs-harness --test second_transaction_supplement_three_random_history -- --nocapture 2>&1 | grep -E '^──|模型对拍|新发现|^test |test result|error(\[|:)|panicked|Err 成员 .*(Rollback|Placement|AllocationRecords)' 
  echo "GATE74-BINARY-EXIT ${PIPESTATUS[0]}"
  if [[ -f $M/extra ]]; then
    while IFS= read -r line; do
      [[ -z "$line" ]] && continue
      echo "=== extra: $line ==="
      n=$((${n:-0}+1)); eval "nice -n 19 $line" > "$L.extra$n" 2>&1 </dev/null; rc=$?; grep -E "PROBE SUMMARY|test result|^test .*FAILED" "$L.extra$n" | head -60; grep -c "first_non_known_red=Operation" "$L.extra$n"; grep -c "ending=NEW" "$L.extra$n"
      echo "EXTRA-EXIT $rc"
    done < "$M/extra"
  fi
  python3 $D/probe/mutate.py --restore "$file"
  date -u
} > "$L" 2>&1
grep -E 'GATE74-BINARY-EXIT|EXTRA-EXIT|APPLY FAILED|PROBE SUMMARY|test result' "$L"
