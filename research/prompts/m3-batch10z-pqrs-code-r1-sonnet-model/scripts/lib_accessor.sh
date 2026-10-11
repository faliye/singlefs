#!/usr/bin/env bash
# 访问器 slots_counted_both_as_allocated_and_as_isolated 恒交 0 的变异，只跑 singlefs-core 库里钉它的那一条单元测试（release）
set -u
W=$HOME/.local/share/singlefs/work/m3-batch10z-pqrs-code-r1-sonnet
R=/home/fy5090/code/singlefs/scripts
F=$W/tree-m/crates/singlefs-core/src/allocator.rs
cp "$F" "$F.orig"
python3 - "$F" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
old = '.filter(|(is_allocated, is_isolated)| **is_allocated && **is_isolated)'
assert s.count(old) == 1
open(p, 'w').write(s.replace(old, old[:-1] + ' && false)'))
PY
cd "$W/tree-m"
CARGO_TARGET_DIR=$W/target-m nice -n 19 bash $R/run-with-memory-cap.sh 6G bash $R/capped.sh 8 cargo test --release --offline -p singlefs-core --lib -- isolating_a_whole_span_over_allocated_slots 2>&1 | grep -E '^test |test result'
cp "$F.orig" "$F"; rm "$F.orig"
