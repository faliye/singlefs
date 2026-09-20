#!/usr/bin/env bash
# 变异证明：每个变异一份仓副本（rsync，不带 target 与 .git）、各用各的 target；逐个串行跑随机历史那个测试二进制。
set -u
repository=/home/fy5090/code/singlefs
scratch=/tmp/claude-1000/m2-supp3-item2-implementer
copies=$scratch/copies
attacker=$repository/research/prompts/m2-supp3-item1-code-r1-opus-model/mutants.tsv
apply=$scratch/apply-mutation.py
field() { awk -F'\t' -v id="$1" -v column="$2" '$1 == id { print $column }' "$attacker"; }
disable_item_one_generation='crates/singlefs-harness/src/history.rs	    (!records.is_empty()).then_some(HarnessJudgement::AllocationGenerationIsNotThePublishTxg {	    (false \&\& !records.is_empty()).then_some(HarnessJudgement::AllocationGenerationIsNotThePublishTxg {'
run_copy() {
  local name=$1; shift
  local directory=$copies/$name
  rm -rf "$directory"
  rsync -a --exclude target --exclude .git "$repository"/ "$directory"/
  while [ $# -ge 3 ]; do
    python3 "$apply" "$directory" "$1" "$2" "$3" || { echo "$name：施加失败" >> "$scratch/mutants-summary.txt"; return; }
    shift 3
  done
  echo "── $name 开始 $(date -u +%H:%M:%S) UTC" >> "$scratch/mutants-summary.txt"
  (cd "$directory" && nice -n 19 cargo test -p singlefs-harness --test second_transaction_supplement_three_random_history --no-run) > "$scratch/$name.build.log" 2>&1
  (cd "$directory" && nice -n 19 cargo test -p singlefs-harness --test second_transaction_supplement_three_random_history -- --test-threads 8) > "$scratch/$name.log" 2>&1
  echo "── $name 结束 $(date -u +%H:%M:%S) UTC：$(grep -E '^test result' "$scratch/$name.log")" >> "$scratch/mutants-summary.txt"
  grep -E '^test .* FAILED$|^test .* ok$' "$scratch/$name.log" >> "$scratch/mutants-summary.txt"
}
: > "$scratch/mutants-summary.txt"
run_copy baseline
for id in B1 B2 B3 B5 B6 N2; do
  run_copy "$id" "$(field $id 3)" "$(field $id 4)" "$(field $id 5)"
done
