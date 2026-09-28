#!/usr/bin/env bash
# 复跑：在快照 refs/sop/m2-closeout-code-r1-snapshot 的副本上加这几份用例跑（不动主工作区）。用法：bash rerun.sh <草稿目录>
set -euo pipefail
S=${1:?草稿目录}
R=<仓根>
M=$R/research/prompts/m2-closeout-code-r1-opus-model
mkdir -p "$S/tree"
git -C "$R" archive refs/sop/m2-closeout-code-r1-snapshot crates Cargo.toml Cargo.lock | tar -x -C "$S/tree"
cp "$M"/opus_r1_*.rs "$S/tree/crates/singlefs-harness/tests/"
export CARGO_TARGET_DIR="$S/target"
cd "$S/tree"
CAP="bash $R/research/scripts/capped.sh 5 bash $R/research/scripts/run-with-memory-cap.sh 12G"
for t in opus_r1_z3_entries_with_a_substituted_device opus_r1_z1_rollback_and_quarantined_copies opus_r1_z5_torn_split_points_and_fua_release; do
  nice -n 19 $CAP cargo test -p singlefs-harness --test "$t" -- --nocapture
done
# 随机走（release，约 9 分钟一格）：宽度 4g / 384，种子 201..400，每段 60 步
for w in 4g 384; do
  OPUS_Z1_WIDTH=$w OPUS_Z1_FIRST_SEED=201 OPUS_Z1_SEEDS=200 OPUS_Z1_STEPS=60 nice -n 19 $CAP cargo test -p singlefs-harness --release --test opus_r1_z1_rollback_walk_with_multi_unit_files_inodes_and_unmount -- --nocapture
done
