#!/usr/bin/env bash
# 攻方腿 s2：把部分写切分差分用例拷进改前、改后两棵树，各编各跑，逐行比两份输出。参数：草稿目录（两棵树 old/ new/ 已在里面）。
set -euo pipefail
work="$1"
cp "$work/model/tests/attack_partial_write_split_differential.rs" "$work/new/crates/singlefs-core/tests/"
cp "$work/model/tests/attack_partial_write_split_differential.rs" "$work/old/crates/singlefs-core/tests/"
cd "$work/new"
CARGO_TARGET_DIR="$work/target-new" cargo test --offline -p singlefs-core --test attack_partial_write_split_differential --no-run 2>&1 | grep -E '^error|Finished' -A5 | head -n 40
cd "$work/old"
CARGO_TARGET_DIR="$work/target-old" cargo test --offline -p singlefs-core --test attack_partial_write_split_differential --no-run 2>&1 | grep -E '^error|Finished' -A5 | head -n 40
new_binary=$(ls -t "$work"/target-new/debug/deps/attack_partial_write_split_differential-* | grep -v '\.d$' | head -n 1)
old_binary=$(ls -t "$work"/target-old/debug/deps/attack_partial_write_split_differential-* | grep -v '\.d$' | head -n 1)
ATTACK_OUT="$work/out-split-old.txt" "$old_binary" > "$work/run-split-old.txt" 2>&1 || echo "old binary rc=$?"
ATTACK_OUT="$work/out-split-new.txt" "$new_binary" > "$work/run-split-new.txt" 2>&1 || echo "new binary rc=$?"
wc -l "$work/out-split-new.txt" "$work/out-split-old.txt"
if cmp -s "$work/out-split-new.txt" "$work/out-split-old.txt"; then echo "differential: identical"; else echo "differential: differ"; diff "$work/out-split-old.txt" "$work/out-split-new.txt" > "$work/split-diff.txt" || true; head -n 6 "$work/split-diff.txt" | cut -c1-600; fi
