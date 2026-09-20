#!/usr/bin/env bash
# 模型自己的变异：每条一份副本，跑 harness 的 lib 测试二进制与随机历史测试二进制。
set -u
repository=/home/fy5090/code/singlefs
scratch=/tmp/claude-1000/m2-supp3-item2-implementer
copies=$scratch/copies
apply=$scratch/apply-mutation.py
: > "$scratch/model-mutants-summary.txt"
run_copy() {
  local name=$1; shift
  local directory=$copies/$name
  rm -rf "$directory"
  rsync -a --exclude target --exclude .git "$repository"/ "$directory"/
  while [ $# -ge 3 ]; do
    python3 "$apply" "$directory" "$1" "$2" "$3" || { echo "$name：施加失败" >> "$scratch/model-mutants-summary.txt"; return; }
    shift 3
  done
  echo "── $name $(date -u +%H:%M:%S) UTC" >> "$scratch/model-mutants-summary.txt"
  (cd "$directory" && nice -n 19 cargo test -p singlefs-harness --lib) > "$scratch/$name.lib.log" 2>&1
  echo "lib：$(grep -E '^test result' "$scratch/$name.lib.log")" >> "$scratch/model-mutants-summary.txt"
  grep -E '^test .* FAILED$' "$scratch/$name.lib.log" >> "$scratch/model-mutants-summary.txt"
  grep -E '^error' "$scratch/$name.lib.log" | head -3 >> "$scratch/model-mutants-summary.txt"
}
run_copy model-baseline
while IFS=$'\t' read -r name file original replacement; do
  case "$name" in '#'*|'') continue ;; esac
  run_copy "$name" "$file" "$original" "$replacement"
done < "$scratch/model-mutants.tsv"
