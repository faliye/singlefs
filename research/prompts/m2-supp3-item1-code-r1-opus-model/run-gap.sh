#!/usr/bin/env bash
# 攻方腿（Opus）m2-supp3-item1-code-r1：在装了 opus_gap_probe.rs 的仓副本上逐条施加变异、跑机理探针，存 GAP 行。
# 用法：bash run-gap.sh <副本> <target> <输出目录> <种子数> <步数> <编号…>（编号取自 mutants.tsv；BASE 不施加）
set -uo pipefail
slot=$1; target=$2; out=$3; seeds=$4; ops=$5; shift 5
here=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$out"
for id in "$@"; do
  python3 - "$here/mutants.tsv" "$slot" "$id" apply <<'PY'
import sys, os
table, slot, ident, mode = sys.argv[1:5]
if ident == "BASE":
    sys.exit(0)
for line in open(table, encoding="utf-8"):
    line = line.rstrip("\n")
    if not line or line.startswith("#"):
        continue
    i, name, path, old, new = line.split("\t")
    if i != ident:
        continue
    old, new = old.replace("\\n", "\n"), new.replace("\\n", "\n")
    full = os.path.join(slot, path)
    text = open(full, encoding="utf-8").read()
    assert text.count(old) == 1, (ident, text.count(old))
    open(full + ".opus-pristine", "w", encoding="utf-8").write(text)
    open(full, "w", encoding="utf-8").write(text.replace(old, new))
PY
  (cd "$slot" && OPUS_SEEDS=$seeds OPUS_OPS=$ops OPUS_THREADS=16 CARGO_TARGET_DIR=$target nice -n 19 cargo test --offline --release -p singlefs-harness --test opus_gap_probe -- --ignored --nocapture 2>&1 | grep -E '^GAP|error' > "$out/gap-$id-${seeds}x$ops.log")
  for backup in $(find "$slot/crates" -name '*.opus-pristine'); do mv "$backup" "${backup%.opus-pristine}"; touch "${backup%.opus-pristine}"; done
  echo "$id $(tail -1 "$out/gap-$id-${seeds}x$ops.log")"
done
