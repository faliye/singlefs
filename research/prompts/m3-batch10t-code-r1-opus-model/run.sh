#!/usr/bin/env bash
# 复跑攻方腿 m3-batch10t-code-r1 的模型。用法：run.sh <冻结副本根> <草稿目录>（草稿目录里建仓的拷贝与编译目录，跑完自己删）。
# 只跑 harness 档的两个原型测试目标（名字不带 layer0）与一份纯算术脚本；不上卡，账本在临时目录。
set -euo pipefail
frozen="$1"
work="$2"
model="$(cd "$(dirname "$0")" && pwd)"
repo=/home/fy5090/code/singlefs
rsync -a --exclude /target --exclude /.git --exclude /scripts/experiment-apparatus/target "$frozen/" "$work/tree/"
tests="$work/tree/crates/singlefs-harness/tests"
mkdir -p "$tests/opus_model"
{ echo '#![allow(dead_code, clippy::all)]'; head -n 1489 "$frozen/crates/singlefs-harness/src/gpu_memory_ledger.rs" | sed 's#//!#//#g'; } > "$work/base_ledger_copy.rs"
patch -s -o "$tests/opus_model/fixed_ledger.rs" "$work/base_ledger_copy.rs" "$model/fixed_ledger.patch"
cp "$model/opus_ledger_attack_model.rs" "$tests/"
bash "$model/build_scan_bypass_test.sh" "$frozen" "$tests/opus_scan_bypass_model.rs"
cd "$work/tree"
for target in opus_ledger_attack_model opus_scan_bypass_model; do
  CARGO_TARGET_DIR="$work/target" nice -n 19 bash "$repo/scripts/hosts/run-with-memory-cap.sh" 8G \
    bash "$repo/scripts/hosts/capped.sh" 12 cargo test --offline -p singlefs-harness --test "$target" -- --nocapture --test-threads 1 \
    > "$work/$target.log" 2>&1
  grep -o 'MODEL .*' "$work/$target.log" | sort
done
nice -n 19 python3 "$model/s5_product_path_budget_from_the_opening_free_memory.py"
rustc --edition 2021 -o "$work/raw_c_strings_check" "$model/compile_check_raw_c_strings.rs"
bash "$repo/scripts/hosts/run-with-memory-cap.sh" 1G "$work/raw_c_strings_check"
