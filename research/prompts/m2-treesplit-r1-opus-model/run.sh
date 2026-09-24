#!/usr/bin/env bash
# m2-treesplit-r1 云端攻方原型的驱动（副本上跑）。用法：bash run.sh <副本根> <输出目录>
# 机器负载高时全格要几个小时：T1 全格只跑 PA/PB/PC/PE 四个前缀；PD（树高 3）只跑两个代表格。
set -uo pipefail
repo=${1:?副本根}
out=${2:?输出目录}
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-/tmp/claude-1000/m2-treesplit-opus/target}
export CARGO_BUILD_JOBS=4
export TREESPLIT_THREADS=${TREESPLIT_THREADS:-8}
cd "$repo" || exit 2
run() {
  local tag=$1; local test=$2; shift 2
  { date -u +%FT%TZ; env "$@" nice -n 19 cargo test --offline --release -p singlefs-harness --test "$test" -- --nocapture --test-threads 1 2>&1; echo "exit=$?"; date -u +%FT%TZ; } > "$out/$tag.out"
}
run selftest treesplit_opus_prototype TREESPLIT_SCAN=none
run costs treesplit_opus_costs TREESPLIT_SCAN=none
run t6 treesplit_opus_prototype TREESPLIT_SCAN=t6 TREESPLIT_PREFIXES=PA,PB,PC,PE
run t1 treesplit_opus_prototype TREESPLIT_SCAN=t1 TREESPLIT_PREFIXES=PA,PB,PC,PE
run targeted treesplit_opus_prototype TREESPLIT_SCAN=none
