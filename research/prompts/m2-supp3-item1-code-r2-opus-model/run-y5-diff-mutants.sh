#!/usr/bin/env bash
# 攻方腿（Opus）m2-supp3-item1-code-r2 · Y5：拿差分探针自证会红——在追加了 y5-differential.rs 的副本 SLOT 上逐条施加 y5-mutants.tsv 的变异，
# 跑差分 TRIALS 趟，记退出码与第一处不等，跑完还原。用法：run-y5-diff-mutants.sh SLOT LOGDIR TRIALS ID...
set -u
slot=$1; logs=$2; trials=$3; shift 3
mkdir -p "$logs"
here=$(cd "$(dirname "$0")" && pwd)
for id in "$@"; do
  file=$(awk -F'\t' -v id="$id" '$1==id{print $3}' "$here/y5-mutants.tsv")
  cp "$slot/$file" "$logs/$id.pristine"
  python3 - "$here/y5-mutants.tsv" "$id" "$slot" <<'PY'
import sys
table, ident, slot = sys.argv[1:]
for line in open(table, encoding="utf-8"):
    f = line.rstrip("\n").split("\t")
    if f[0] == ident:
        path = f"{slot}/{f[2]}"; old = f[3].replace("\\n", "\n"); new = f[4].replace("\\n", "\n")
        text = open(path, encoding="utf-8").read(); assert text.count(old) == 1
        open(path, "w", encoding="utf-8").write(text.replace(old, new))
PY
  (cd "$slot" && OPUS_TRIALS=$trials nice -n 19 cargo test --offline --release -p singlefs-harness --test second_transaction_step_zero_layer0 -- --ignored --exact opus_r2_parallel_matches_the_single_threaded_walk --nocapture > "$logs/$id-diff.log" 2>&1); code=$?
  echo "$id diff exit $code first_unequal: $(grep -m1 -E 'equal_tally=false|equal_observed=false' "$logs/$id-diff.log" | cut -c1-260) | $(grep -m1 -A2 'panicked at' "$logs/$id-diff.log" | tail -1 | cut -c1-200)"
  cp "$logs/$id.pristine" "$slot/$file"
done
