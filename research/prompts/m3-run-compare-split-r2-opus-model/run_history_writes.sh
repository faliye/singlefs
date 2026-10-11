#!/usr/bin/env bash
# 攻方腿模型（m3-run-compare-split-r2）：编 history-writes（只依赖 harness 与 core），跑 <首个种子偏移> <种子数> <步数…>。整条经 run-with-memory-cap.sh 起。
# 用法：run_history_writes.sh <副本根> <草稿目录> <首个种子偏移> <种子数> <步数…>
set -euo pipefail
copy_root="$1"; work="$2"; shift 2; model="$(cd "$(dirname "$0")" && pwd)"
mkdir -p "$work/history-writes/src"
sed "s|COPY_ROOT|$copy_root|" "$model/history-writes/Cargo.toml.template" > "$work/history-writes/Cargo.toml"
cp "$model/history-writes/src/main.rs" "$work/history-writes/src/main.rs"
[[ -f "$work/history-writes/Cargo.lock" ]] || cp "$copy_root/Cargo.lock" "$work/history-writes/Cargo.lock"
(cd "$work/history-writes" && CARGO_TARGET_DIR="$work/target-history-writes" cargo build --release --offline --quiet)
"$work/target-history-writes/release/history-writes" "$@"
