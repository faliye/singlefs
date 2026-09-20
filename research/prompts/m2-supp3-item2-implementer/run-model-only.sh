#!/usr/bin/env bash
# 模型单独判不判得出 N2 与 B6：在副本里把第 1 件那两条执行器判定关掉（分配代、冷启动读回报错），再施加变异。
set -u
repository=/home/fy5090/code/singlefs
scratch=/tmp/claude-1000/m2-supp3-item2-implementer
copies=$scratch/copies
attacker=$repository/research/prompts/m2-supp3-item1-code-r1-opus-model/mutants.tsv
apply=$scratch/apply-mutation.py
field() { awk -F'\t' -v id="$1" -v column="$2" '$1 == id { print $column }' "$attacker"; }
history=crates/singlefs-harness/src/history.rs
disable_generation=("$history" '    (!records.is_empty()).then_some(HarnessJudgement::AllocationGenerationIsNotThePublishTxg {' '    (false && !records.is_empty()).then_some(HarnessJudgement::AllocationGenerationIsNotThePublishTxg {')
disable_cold_start_a=("$history" '            ColdStartReadBack::Failed => Some(' '            ColdStartReadBack::Failed if false => Some(')
disable_cold_start_b=("$history" '            ColdStartReadBack::NoFile | ColdStartReadBack::FileRead => None,' '            ColdStartReadBack::Failed | ColdStartReadBack::NoFile | ColdStartReadBack::FileRead => None,')
run_copy() {
  local name=$1; shift
  local directory=$copies/$name
  rm -rf "$directory"
  rsync -a --exclude target --exclude .git "$repository"/ "$directory"/
  while [ $# -ge 3 ]; do
    python3 "$apply" "$directory" "$1" "$2" "$3" || { echo "$name：施加失败" >> "$scratch/model-only-summary.txt"; return; }
    shift 3
  done
  echo "── $name 开始 $(date -u +%H:%M:%S) UTC" >> "$scratch/model-only-summary.txt"
  (cd "$directory" && nice -n 19 cargo test -p singlefs-harness --test second_transaction_supplement_three_random_history --no-run) > "$scratch/$name.build.log" 2>&1
  (cd "$directory" && nice -n 19 cargo test -p singlefs-harness --test second_transaction_supplement_three_random_history -- --test-threads 8) > "$scratch/$name.log" 2>&1
  echo "── $name 结束 $(date -u +%H:%M:%S) UTC：$(grep -E '^test result' "$scratch/$name.log")" >> "$scratch/model-only-summary.txt"
  grep -E '^test .* FAILED$|^test .* ok$' "$scratch/$name.log" >> "$scratch/model-only-summary.txt"
}
: > "$scratch/model-only-summary.txt"
run_copy judgements-disabled-baseline "${disable_generation[@]}" "${disable_cold_start_a[@]}" "${disable_cold_start_b[@]}"
run_copy N2-model-only "${disable_generation[@]}" "${disable_cold_start_a[@]}" "${disable_cold_start_b[@]}" "$(field N2 3)" "$(field N2 4)" "$(field N2 5)"
run_copy B6-model-only "${disable_generation[@]}" "${disable_cold_start_a[@]}" "${disable_cold_start_b[@]}" "$(field B6 3)" "$(field B6 4)" "$(field B6 5)"
