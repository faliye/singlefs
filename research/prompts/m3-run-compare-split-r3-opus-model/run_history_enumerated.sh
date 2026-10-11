#!/usr/bin/env bash
# 攻方腿模型（m3-run-compare-split-r3）：编 history-enumerated（依赖副本的 harness、core 与 checker 档的库，不编它的测试），跑一批种子。
# 用法：run_history_enumerated.sh <副本根> <草稿目录> <首个种子偏移> <种子数> <步数>（整条经 run-with-memory-cap.sh 起）；输出打到标准输出
set -euo pipefail
copy_root="$1"; work="$2"; first="$3"; count="$4"; steps="$5"; model="$(cd "$(dirname "$0")" && pwd)"
mkdir -p "$work/history-enumerated/src"
sed "s|COPY_ROOT|$copy_root|" "$model/history-enumerated/Cargo.toml.template" > "$work/history-enumerated/Cargo.toml"
cp "$model/history-enumerated/src/main.rs" "$work/history-enumerated/src/main.rs"
(cd "$work/history-enumerated" && CARGO_TARGET_DIR="$work/target-history-enumerated" cargo build --release --offline --quiet)
"$work/target-history-enumerated/release/history-enumerated" "$first" "$count" "$steps"
