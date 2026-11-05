#!/usr/bin/env bash
# 攻方腿 s2：把差分用例拷进改前、改后两棵树，各编各跑，逐行比两份输出。参数：草稿目录（两棵树 old/ new/ 已在里面）。
set -euo pipefail
work="$1"
cp "$work/model/tests/attack_codec_differential.rs" "$work/new/crates/singlefs-core/tests/"
cp "$work/model/tests/attack_codec_differential.rs" "$work/old/crates/singlefs-core/tests/"
export ROCKSDB_LIB_DIR=/home/fy5090/.local/share/singlefs/rocksdb-prebuilt/librocksdb-sys-0.19.0+11.8.1/67873f27d6329162/lib ROCKSDB_STATIC=1
export SNAPPY_LIB_DIR=/home/fy5090/.local/share/singlefs/rocksdb-prebuilt/librocksdb-sys-0.19.0+11.8.1/67873f27d6329162/lib SNAPPY_STATIC=1
cd "$work/new"
CARGO_TARGET_DIR="$work/target-new" cargo test --offline -p singlefs-core --test attack_codec_differential --no-run 2>&1 | tail -n 2
cd "$work/old"
CARGO_TARGET_DIR="$work/target-old" cargo test --offline -p singlefs-core --test attack_codec_differential --no-run 2>&1 | tail -n 2
new_binary=$(ls -t "$work"/target-new/debug/deps/attack_codec_differential-* | grep -v '\.d$' | head -n 1)
old_binary=$(ls -t "$work"/target-old/debug/deps/attack_codec_differential-* | grep -v '\.d$' | head -n 1)
ATTACK_OUT="$work/out-new.txt" "$new_binary" > "$work/run-new.txt" 2>&1
ATTACK_OUT="$work/out-old.txt" "$old_binary" > "$work/run-old.txt" 2>&1
wc -l "$work/out-new.txt" "$work/out-old.txt"
if cmp -s "$work/out-new.txt" "$work/out-old.txt"; then echo "differential: identical"; else echo "differential: differ"; diff "$work/out-old.txt" "$work/out-new.txt" | head -n 20 | cut -c1-400; fi
