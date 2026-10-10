#!/usr/bin/env bash
# 对每一行（GPU 内核或表的变异）：套用到仓副本、编 sizes 小程序（只编 lib，gpu 特性）、跑它（解析 + naga 校验 + 展开表达式数）、还原
set -u
REPO=${SINGLEFS_REPO}; PRISTINE=${PRISTINE_TREE}; TABLE=${PRISTINE}/crates/mutations.tsv
export CARGO_TARGET_DIR=${SCRATCH}/target-sizes
for row in "$@"; do
  OUT=${SCRATCH}/logs/wgsl-row-${row}.log
  python3 -I ${SCRATCH}/bin/apply_row.py "$TABLE" "$row" "$REPO" "$PRISTINE" restore >/dev/null
  python3 -I ${SCRATCH}/bin/apply_row.py "$TABLE" "$row" "$REPO" "$PRISTINE" apply > "$OUT.head" 2>&1 || { echo "row $row"; cat "$OUT.head"; continue; }
  file=$(grep '^applied' "$OUT.head" | sed 's/.*-> //')
  (cd ${SCRATCH}/sizes && cargo build --release --offline > "$OUT.build" 2>&1; echo "build exit $?" >> "$OUT.build")
  bexit=$(tail -1 "$OUT.build")
  if [ "$bexit" = "build exit 0" ]; then run=$(${SCRATCH}/target-sizes/release/kernel-sizes 2>&1 | tr '\n' ';' | cut -c1-330); else run="BUILD-FAILED $(grep -E '^error' "$OUT.build" | head -2 | tr '\n' ' ' | cut -c1-200)"; fi
  python3 -I ${SCRATCH}/bin/apply_row.py "$TABLE" "$row" "$REPO" "$PRISTINE" restore >/dev/null
  echo "row $row $file | $bexit | $run"
done
