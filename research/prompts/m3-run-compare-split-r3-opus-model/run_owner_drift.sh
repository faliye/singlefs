#!/usr/bin/env bash
# 攻方腿模型（m3-run-compare-split-r3）：从副本抽 WeightedShares（结构与 impl）原文、编 owner-drift（只用 std），跑三组真权重两两之间与一组差 1 的。
# 用法：run_owner_drift.sh <副本根> <草稿目录>（整条经 run-with-memory-cap.sh 起）；输出打到标准输出
set -euo pipefail
copy_root="$1"; work="$2"; model="$(cd "$(dirname "$0")" && pwd)"
source_file="$copy_root/crates/singlefs-checker-tier/src/crash_amplification.rs"
mkdir -p "$work/owner-drift/src"
cp "$model/owner-drift/Cargo.toml" "$work/owner-drift/Cargo.toml"
cp "$model/owner-drift/src/main.rs.template" "$work/owner-drift/src/main.rs"
awk 'NR>=962 && NR<=998' "$source_file" > "$work/owner-drift/src/weighted_shares.rs"
grep -q '^pub struct WeightedShares {' "$work/owner-drift/src/weighted_shares.rs"
grep -q 'pub fn owner_of_block(&self, block_sequence: u64) -> usize {' "$work/owner-drift/src/weighted_shares.rs"
(cd "$work/owner-drift" && CARGO_TARGET_DIR="$work/target-owner-drift" cargo build --release --offline --quiet)
tool="$work/target-owner-drift/release/owner-drift"
"$tool" 136700,45275 140427,62390
"$tool" 140427,62390 642805,291558
"$tool" 136700,45275 642805,291558
"$tool" 136700,45275 136701,45275
"$tool" 136700,45275 273400,90550
