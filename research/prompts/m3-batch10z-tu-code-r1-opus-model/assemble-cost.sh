#!/usr/bin/env bash
# 量法副本（core 里加了 ATK_SKIP_REHEARSAL / ATK_LAST_REHEARSAL）的拼法：assemble-cost.sh <量法副本根> <原型目录>
set -euo pipefail
repo=${1:?}; src=${2:?}
t=$repo/crates/singlefs-harness/tests
cat "$t/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs" "$src"/atk_tu_part*.rs "$src"/atk_cost_part.rs > "$t/atk_tu_cost_opus.rs.tmp"
mv "$t/atk_tu_cost_opus.rs.tmp" "$t/atk_tu_cost_opus.rs"
