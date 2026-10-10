#!/usr/bin/env bash
# 变异 d（环转之前把门槛 0 提前拿 3 当）在 harness 档已有用例上红不红：整条经 run-with-memory-cap.sh 跑：
#   bash run-with-memory-cap.sh 8G bash capped.sh 5 bash mutant_d_existing_suite.sh <草稿副本根>
set -uo pipefail
tree="$1"; tools="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
python3 "$tools/mutate_core_reclaim.py" "$tree" d || exit 2
cd "$tree" || exit 2
echo "MUTANT d 对已有用例 root_ring_turn_in_one_mount"
cargo test --offline -p singlefs-harness --test root_ring_turn_in_one_mount 2>&1 | grep -E '^test |test result|^error'
python3 "$tools/mutate_core_reclaim.py" "$tree" restore
