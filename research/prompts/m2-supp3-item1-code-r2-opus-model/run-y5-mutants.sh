#!/usr/bin/env bash
# 攻方腿（Opus）m2-supp3-item1-code-r2 · Y5：在仓副本 SLOT 上逐条施加 y5-mutants.tsv 的变异，跑 crates/mutations.tsv 第 131–138 行点名的测试
# （release；并片次序那条重复 REPEAT 次，数红了几次），跑完还原。用法：run-y5-mutants.sh SLOT LOGDIR REPEAT ID...
set -u
slot=$1; logs=$2; repeat=$3; shift 3
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
  (cd "$slot" && nice -n 19 cargo test --offline --release -p singlefs-harness --lib -- crash::tests > "$logs/$id-lib.log" 2>&1); echo "$id lib exit $? $(grep -E '^test result' "$logs/$id-lib.log")"
  red=0
  for n in $(seq 1 "$repeat"); do
    (cd "$slot" && nice -n 19 cargo test --offline --release -p singlefs-harness --test second_transaction_step_zero_layer0 -- one_state_slices_on_eight_threads every_crash_state_outside_the_two_unit_segments > "$logs/$id-it-$n.log" 2>&1) || red=$((red+1))
  done
  echo "$id integration red $red / $repeat ; last: $(grep -E '^test result' "$logs/$id-it-$repeat.log")"
  cp "$logs/$id.pristine" "$slot/$file"
done
