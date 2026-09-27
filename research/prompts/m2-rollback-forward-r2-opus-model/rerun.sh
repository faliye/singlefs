#!/usr/bin/env bash
# m2-rollback-forward-r2 云端攻方腿原型的复跑（冻结副本拷贝上的数，不是入库装置上的数）。
#   bash rerun.sh <冻结副本的 tree 目录> <草稿目录> [fast|sweep|full]
# 冻结副本：/tmp/claude-1000/m2-rollback-forward-r2/tree（清单 research/prompts/m2-rollback-forward-r2-snapshot/crates-sha256.txt）。
# fast：除 G1 扫描与整段子集崩溃枚举之外的全部用例（约 4 分钟，编译另计）；
# sweep：另跑 G1 用户动作扫描（2400 段历史，约 10 分钟）；
# full：另跑回退到 A 那一次发布的整段子集崩溃枚举（1048583 个状态，每 500 个状态做一次探针，约 30 分钟）。
set -euo pipefail
TREE=$1; WORK=$2; MODE=${3:-fast}
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$HERE/../../.." && pwd)"
S="$REPO_ROOT/research/scripts"
mkdir -p "$WORK/tmp"
rsync -a --delete --exclude target --exclude .git "$TREE/" "$WORK/repo/"
cp "$REPO_ROOT/Cargo.toml" "$REPO_ROOT/Cargo.lock" "$WORK/repo/"
(cd "$WORK/repo" && patch -p0 < "$HERE/prototype.patch")
cp "$HERE/rbf2_attack.rs" "$WORK/repo/crates/singlefs-harness/tests/rbf2_attack.rs"
cd "$WORK/repo"
export CARGO_TARGET_DIR="$WORK/repo/target" TMPDIR="$WORK/tmp"
run() { nice -n 19 bash "$S/run-with-memory-cap.sh" 16G bash "$S/capped.sh" 14 cargo test --release -p singlefs-harness --test rbf2_attack -- --nocapture --test-threads 1 "$@"; }
keep() { grep -E 'RBF2 |test result|panicked' | sed -E 's/^.*(RBF2 )/\1/'; }
: > "$WORK/rerun-fast.out"
for t in g1_smoke g1_literal_whole_tree_diff_after_a_ring_turn g2_stale_previous_around_the_rollback \
         g2_stale_write_after_a_rollback_that_releases_nothing g2_inode_numbers_after_rollback \
         g3_h1_cells_under_both_rules g3_chain_crashed_before_device_one g3_chain_crash_then_row_publish_crash \
         g3_device_one_gone g3_complete_chain_control g4_b1_literal_ceiling g4_remount_after_b1_then_newest_roots_damaged \
         g4_unmount_chain_and_admission_raise_are_the_same_bytes g5_old_image_under_the_deleting_code g1_g4_crash_points_prefix \
         g1_distinct_states_after_ping_pong_rollbacks_and_a_raise; do
  run --exact "$t" 2>&1 | keep >> "$WORK/rerun-fast.out"
done
if [ "$MODE" = sweep ] || [ "$MODE" = full ]; then
  RBF2_SWEEP_LEN=3 run --exact g1_sweep_user_steps 2>&1 | keep > "$WORK/rerun-sweep.out"
fi
if [ "$MODE" = full ]; then
  RBF2_ONLY_A=1 RBF2_PROBE_EVERY=500 RBF2_AUDIT=1 run --exact g1_crash_rollback_publish 2>&1 | keep > "$WORK/rerun-full-crash.out"
fi
echo "outputs in $WORK"
