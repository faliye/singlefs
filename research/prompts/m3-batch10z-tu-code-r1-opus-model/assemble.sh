#!/usr/bin/env bash
# 把冻结副本里的 harness 测试文件与攻方原型拼成一个新测试目标：assemble.sh <仓副本根> <原型目录>
set -euo pipefail
repo=${1:?}; src=${2:?}
t=$repo/crates/singlefs-harness/tests
cat "$t/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs" "$src"/atk_tu_part*.rs > "$t/atk_tu_opus.rs.tmp"
mv "$t/atk_tu_opus.rs.tmp" "$t/atk_tu_opus.rs"
