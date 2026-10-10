#!/usr/bin/env bash
# 整条脚本经 run-with-memory-cap.sh 跑：bash run-with-memory-cap.sh 8G bash capped.sh 5 bash mutate_core_reclaim.sh <草稿副本根> <变异号>
# 在草稿副本里改 core 记根时的回收门槛，跑两条 harness 档用例，打结果行，最后把 allocator.rs 还原。
set -uo pipefail
tree="$1"; which="$2"; tools="$(dirname "${BASH_SOURCE[0]}")"
python3 "$tools/mutate_core_reclaim.py" "$tree" "$which" || exit 2
cd "$tree" || exit 2
echo "MUTANT $which"
cargo test --offline -p singlefs-harness --test root_ring_turn_in_the_make_filesystem_session 2>&1 | grep -E '^test |test result|^error' | sed 's/^/  C551新用例: /'
cargo test --offline -p singlefs-harness --test overwrite_in_one_instance -- allocated_statistic_equals 2>&1 | grep -E '^test |test result|^error|panicked|left|right' | sed 's/^/  812用例: /'
python3 "$tools/mutate_core_reclaim.py" "$tree" restore
