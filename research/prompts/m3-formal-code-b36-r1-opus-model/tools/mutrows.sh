#!/usr/bin/env bash
# 副本上逐行跑第 36 批四行 CPU 变异（crates/mutations.tsv 第 963、2927、3020、3021 行）：基线一次、每行改坏一次、还原核 sha。
set -uo pipefail
W="$1"; ROWS=("${@:2}")
cd "$W/mut"
ORIG_SHA=$(sha256sum crates/singlefs-checker/src/walk.rs | cut -d' ' -f1)
cp crates/singlefs-checker/src/walk.rs "$W/walk.rs.orig"
run() {
  local tag="$1"; shift
  CARGO_TARGET_DIR="$W/target-mut" RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="副本上跑变异行" nice -n 19 bash /home/fy5090/code/singlefs/scripts/run-with-memory-cap.sh 8G bash /home/fy5090/code/singlefs/scripts/capped.sh 16 cargo test --offline -p singlefs-harness --test checker_known_bad_images -- "$@" > "$W/out/mut-$tag.log" 2>&1
  local rc=$?
  echo "B36M tag=$tag rc=$rc compiled_checker=$(grep -c 'Compiling singlefs-checker ' "$W/out/mut-$tag.log") result=$(grep -m1 '^test result' "$W/out/mut-$tag.log") failed=[$(grep -E '^test .* FAILED$' "$W/out/mut-$tag.log" | tr '\n' ' ')] panic=[$(grep -m1 -A2 'panicked at' "$W/out/mut-$tag.log" | tr '\n' ' ' | cut -c1-300)]"
}
NAMES=()
for r in "${ROWS[@]}"; do NAMES+=("$(awk -F'\t' -v r="$r" 'NR==r{print $6}' crates/mutations.tsv)"); done
run baseline "${NAMES[@]}"
for i in "${!ROWS[@]}"; do
  r="${ROWS[$i]}"
  python3 - "$r" <<'PY'
import sys
r = int(sys.argv[1])
f = open('crates/mutations.tsv').read().split('\n')[r - 1].split('\t')
path = f[1]
src = open(path).read()
anchor, replacement = f[2].replace('\\n', '\n'), f[3].replace('\\n', '\n')
assert src.count(anchor) == 1, (r, src.count(anchor))
open(path + '.tmp', 'w').write(src.replace(anchor, replacement, 1))
import os
os.replace(path + '.tmp', path)
PY
  run "row$r" "${NAMES[$i]}"
  cp "$W/walk.rs.orig" crates/singlefs-checker/src/walk.rs.tmp && mv crates/singlefs-checker/src/walk.rs.tmp crates/singlefs-checker/src/walk.rs
  NOW=$(sha256sum crates/singlefs-checker/src/walk.rs | cut -d' ' -f1)
  echo "B36M row=$r restored=$([ "$NOW" = "$ORIG_SHA" ] && echo ok || echo MISMATCH)"
done
run after-restore "${NAMES[@]}"
