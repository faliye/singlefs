#!/usr/bin/env bash
# 攻方腿 s2：把容器小树差分用例拷进改前、改后两棵树，各编各跑，逐行比两份输出。参数：草稿目录（两棵树 old/ new/ 已在里面）。
set -euo pipefail
work="$1"
cp "$work/model/tests/attack_container_tree_differential.rs" "$work/new/crates/singlefs-core/tests/"
cp "$work/model/tests/attack_container_tree_differential.rs" "$work/old/crates/singlefs-core/tests/"
cd "$work/new"
CARGO_TARGET_DIR="$work/target-new" cargo test --offline -p singlefs-core --test attack_container_tree_differential --no-run 2>&1 | grep -E '^(error|warning: unused)|Finished' -A5 | head -n 60
cd "$work/old"
CARGO_TARGET_DIR="$work/target-old" cargo test --offline -p singlefs-core --test attack_container_tree_differential --no-run 2>&1 | grep -E '^error|Finished' -A5 | head -n 60
new_binary=$(ls -t "$work"/target-new/debug/deps/attack_container_tree_differential-* | grep -v '\.d$' | head -n 1)
old_binary=$(ls -t "$work"/target-old/debug/deps/attack_container_tree_differential-* | grep -v '\.d$' | head -n 1)
ATTACK_OUT="$work/out-tree-old.txt" "$old_binary" > "$work/run-tree-old.txt" 2>&1 || echo "old binary rc=$?"
ATTACK_OUT="$work/out-tree-new.txt" "$new_binary" > "$work/run-tree-new.txt" 2>&1 || echo "new binary rc=$?"
wc -l "$work/out-tree-new.txt" "$work/out-tree-old.txt"
if cmp -s "$work/out-tree-new.txt" "$work/out-tree-old.txt"; then echo "differential: identical"; else echo "differential: differ"; diff "$work/out-tree-old.txt" "$work/out-tree-new.txt" | head -n 20 | cut -c1-400; fi
