#!/usr/bin/env bash
# 整条经 run-with-memory-cap.sh 跑：... bash per_txg_checker_baseline_and_d.sh <草稿副本根>
set -uo pipefail
tree="$1"; tools="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cp "$tools/per_txg_checker_probe.rs" "$tree/crates/singlefs-harness/tests/zz_per_txg_checker.rs"
cd "$tree" || exit 2
echo "== 基线"
cargo test --offline -p singlefs-harness --test zz_per_txg_checker -- --nocapture 2>&1 | grep -E 'PERTXG|^error|test result|^test ' | cut -c1-400
python3 "$tools/mutate_core_reclaim.py" "$tree" d
echo "== 变异 d"
cargo test --offline -p singlefs-harness --test zz_per_txg_checker -- --nocapture 2>&1 | grep -E 'PERTXG|^error|test result|^test ' | cut -c1-400
python3 "$tools/mutate_core_reclaim.py" "$tree" restore
rm -f "$tree/crates/singlefs-harness/tests/zz_per_txg_checker.rs"
