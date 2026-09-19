#!/usr/bin/env bash
# run-mutations.sh <变异名或 baseline>…：在 mutant/ 副本里逐条改坏、跑四个测试二进制（debug）、记红了哪些、从 copy/ 拷回原件并 touch。
set -uo pipefail
base=/tmp/claude-1000/m2-layer0-parallel
table="$base/draft/mutations-local.tsv"
cd "$base/mutant" || exit 2
for name in "$@"; do
  log="$base/logs/mutation-$name.log"
  echo "MUTATION $name START $(date -u +%Y-%m-%dT%H:%M:%SZ) $(uptime)" >"$log"
  file=""
  if [[ "$name" != baseline ]]; then
    row="$(awk -F'\t' -v wanted="$name" '$1 == wanted' "$table")"
    [[ -n "$row" ]] || { echo "没有这条变异：$name" >>"$log"; continue; }
    file="$(cut -f2 <<<"$row")"
    python3 - "$file" "$(cut -f3 <<<"$row")" "$(cut -f4 <<<"$row")" >>"$log" 2>&1 <<'PY'
import sys
path, old, new = sys.argv[1], sys.argv[2].replace("\\n", "\n"), sys.argv[3].replace("\\n", "\n")
text = open(path, encoding="utf-8").read()
assert text.count(old) == 1, ("原文命中次数", text.count(old))
open(path, "w", encoding="utf-8").write(text.replace(old, new, 1))
print("APPLIED", path)
PY
  fi
  nice -n 19 cargo test --offline -p singlefs-harness --lib --test first_transaction_step_seven_layer0 \
    --test second_transaction_step_zero_layer0 --test second_transaction_step_three_formatted_pool_layer0 \
    --no-fail-fast >>"$log" 2>&1
  echo "MUTATION $name EXIT $? END $(date -u +%Y-%m-%dT%H:%M:%SZ)" >>"$log"
  if [[ -n "$file" ]]; then
    cp "$base/copy/$file" "$base/mutant/$file" && touch "$base/mutant/$file"
    cmp "$base/copy/$file" "$base/mutant/$file" && echo "RESTORED $file" >>"$log"
  fi
done
