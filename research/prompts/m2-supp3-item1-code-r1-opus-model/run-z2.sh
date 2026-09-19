#!/usr/bin/env bash
# 攻方腿（Opus）m2-supp3-item1-code-r1 · Z2：在装了 opus_z2_history.rs 的仓副本上，对 BASE 与若干变异各跑一次那段手写历史，存 OPUS-Z2 行。
# 用法：bash run-z2.sh <副本> <target> <输出文件> <编号…>
set -uo pipefail
slot=$1; target=$2; out=$3; shift 3
here=$(cd "$(dirname "$0")" && pwd)
for id in "$@"; do
  python3 - "$here/mutants.tsv" "$slot" "$id" <<'PY'
import sys, os
table, slot, ident = sys.argv[1:4]
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
  echo "== $id" >> "$out"
  (cd "$slot" && CARGO_TARGET_DIR=$target nice -n 19 cargo test --offline --release -p singlefs-harness --test opus_z2_history -- --ignored --nocapture 2>&1 | grep -E '^OPUS-Z2|^error' >> "$out")
  for backup in $(find "$slot/crates" -name '*.opus-pristine'); do mv "$backup" "${backup%.opus-pristine}"; touch "${backup%.opus-pristine}"; done
done
