#!/usr/bin/env bash
# m2-rollback-forward-r3 云端攻方腿原型的复跑（冻结副本拷贝上的数，不是入库装置上的数）。
#   bash rerun.sh <冻结副本的 tree 目录> <草稿目录> [fast|full]
# 冻结副本：/tmp/claude-1000/m2-rollback-forward-r3/tree（清单 research/prompts/m2-rollback-forward-r3-snapshot/crates-sha256.txt）。
# 先打第二轮攻方原型 prototype-r2.patch（与 m2-rollback-forward-r2-opus-model/prototype.patch 逐字节相同），再打这一轮的 prototype3.patch，
# 放进 rbf3_attack.rs（前半借第二轮 rbf2_attack.rs 的装置）。
# fast：这一轮的 10 条用例（约 5 分钟，编译另计）；full：另跑 K1 式子那一臂的用户动作扫描（第二轮 g1_sweep_user_steps，2400 段历史）。
set -euo pipefail
TREE=$1; WORK=$2; MODE=${3:-fast}
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$HERE/../../.." && pwd)"
S="$REPO_ROOT/research/scripts"
mkdir -p "$WORK/tmp"
rsync -a --delete --exclude target --exclude .git "$TREE/" "$WORK/repo/"
cp "$REPO_ROOT/Cargo.toml" "$REPO_ROOT/Cargo.lock" "$WORK/repo/"
(cd "$WORK/repo" && patch -p0 -s < "$HERE/prototype-r2.patch" && patch -p0 -s < "$HERE/prototype3.patch")
cp "$HERE/rbf3_attack.rs" "$WORK/repo/crates/singlefs-harness/tests/rbf3_attack.rs"
cd "$WORK/repo"
export CARGO_TARGET_DIR="$WORK/repo/target" TMPDIR="$WORK/tmp"
run() { nice -n 19 bash "$S/run-with-memory-cap.sh" 16G bash "$S/capped.sh" 12 cargo test --release -p singlefs-harness --test rbf3_attack -- --nocapture --test-threads 1 "$@"; }
keep() { grep -E 'RBF[23] |test result|panicked' | sed -E 's/^.*(RBF[23] )/\1/'; }
: > "$WORK/rerun-fast.out"
for t in k1_watermark_literal_when_the_current_root_is_unreadable k2_candidates_on_the_abandoned_timeline \
         k3_dedup_patterns_and_long_cycles k4_x2_cells_under_five_arms k4_k5_chain_crash_then_mount_crash \
         k4_x4_carriers_syscfg_and_checker k4_user_steps_opened k5_b1_crash_points_under_syscfg \
         k6_i79_variants k7_old_image_under_the_two_fixes; do
  run --exact "$t" 2>&1 | keep >> "$WORK/rerun-fast.out"
done
if [ "$MODE" = full ]; then
  RBF2_SWEEP_ARM=formula RBF2_SWEEP_LEN=3 run --exact g1_sweep_user_steps 2>&1 | keep > "$WORK/rerun-sweep-formula.out"
fi
echo "outputs in $WORK"
