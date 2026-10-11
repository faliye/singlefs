#!/usr/bin/env bash
# 原型：在冻结副本上给树表多注册一棵 day-1 零根的 logged_ops 树（种类码 9、树 ID 19、水位 20），
# 看要碰几处代码、harness 档点名的用例红几条。不碰 singlefs-checker-tier。
#
# 用法（整条脚本经内存包装跑，线程上限 10，排不上设 RUN_WITH_MEMORY_CAP_WAIT_SECONDS=3600）：
#   RUN_WITH_MEMORY_CAP_WAIT_SECONDS=3600 bash scripts/hosts/run-with-memory-cap.sh 8G \
#     bash research/prompts/m3-intent-r1-sonnet-model/rerun.sh /tmp/claude-1000/m3-delete-r2/tree <草稿目录>
#
# 输出：<草稿目录>/rerun.log；末几行是每个点名用例的 rc。基线（未打补丁的冻结副本）的对照：第三个参数给 baseline 时改跑不打补丁的副本。
set -u
HERE=$(cd "$(dirname "$0")" && pwd)
REPO=$(cd "$HERE/../../.." && pwd)
SRC=${1:?冻结副本根（含 Cargo.toml 与 crates/）}
WORK=${2:?草稿目录}
MODE=${3:-patched}
mkdir -p "$WORK"
rm -rf "$WORK/tree"
cp -r "$SRC" "$WORK/tree"
if [ "$MODE" = "patched" ]; then
  (cd "$WORK/tree" && patch -p1 < "$HERE/proto-day1-logged-ops-tree.patch") || exit 1
fi
export CARGO_TARGET_DIR="$WORK/target-$MODE"
LOG="$WORK/rerun-$MODE.log"
: > "$LOG"
cd "$WORK/tree" || exit 1
nice -n 19 bash "$REPO/scripts/hosts/capped.sh" 10 cargo check --offline -p singlefs-format -p singlefs-core -p singlefs-checker -p singlefs-harness --all-targets >> "$LOG" 2>&1
echo "CHECK rc=$?" >> "$LOG"
for pkg in singlefs-format singlefs-core singlefs-checker; do
  nice -n 19 bash "$REPO/scripts/hosts/capped.sh" 10 cargo test --offline --release -p "$pkg" --lib --no-fail-fast >> "$LOG" 2>&1
  echo "LIB $pkg rc=$?" >> "$LOG"
done
tests=(tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker overwrite_in_one_instance new_pool_file_creation_publish unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over rollback_by_a_forward_publish writable_mount_of_a_formatted_pool corrupt_on_disk_content_is_refused_instead_of_panicking)
for t in "${tests[@]}"; do
  nice -n 19 bash "$REPO/scripts/hosts/capped.sh" 10 cargo test --offline --release -p singlefs-harness --test "$t" --no-fail-fast >> "$LOG" 2>&1
  echo "TEST $t rc=$?" >> "$LOG"
done
echo "ALL DONE" >> "$LOG"
grep -E '^(CHECK|LIB|TEST|ALL DONE)' "$LOG"
