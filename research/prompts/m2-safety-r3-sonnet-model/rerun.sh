#!/usr/bin/env bash
# m2-safety-r3 云端辩方（Sonnet）的复跑脚本。
# 前置：本仓根目录有 crates/（今天的实现）。这份脚本不改动本仓，只在一个临时拷贝上打补丁、编译、跑测试。
# 用法：bash rerun.sh <本仓根目录> [<临时目录，默认 /tmp/m2-safety-r3-sonnet-rerun>]
set -euo pipefail
REPO_ROOT="${1:?用法：bash rerun.sh <本仓根目录> [<临时目录>]}"
WORK_DIR="${2:-/tmp/m2-safety-r3-sonnet-rerun}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "==> 从 $REPO_ROOT 拷贝 crates/ 到 $WORK_DIR"
rm -rf "${WORK_DIR:?}"
mkdir -p "$WORK_DIR"
cp -r "$REPO_ROOT/crates" "$WORK_DIR/crates"
cp "$REPO_ROOT/Cargo.toml" "$WORK_DIR/Cargo.toml"
cp "$REPO_ROOT/Cargo.lock" "$WORK_DIR/Cargo.lock"

echo "==> 打补丁（diff/ 下四个文件的改动：admission.rs 加 S4Candidate{Baseline,A1,A3,A4}；allocator.rs 挂上候选字段；"
echo "    transaction.rs 把发布路径的准入判法换成候选版并喂 A1 要的“这次自己的释放”；mount.rs 把挂载路径的准入判法换成候选版，"
echo "    加 A4 专用的挂载准入，并加本文件自己实现的 C283——mount_writable_with_admission_candidate_and_c283）"
for patch_file in "$HERE"/diff/*.patch; do
    echo "  -- $patch_file"
    (cd "$WORK_DIR" && patch -p1 < "$patch_file")
done

echo "==> 放进对拍测试（不经 history.rs 的模型对拍机器：候选与 C283 都是这一轮新加的分支，模型不认得）"
mkdir -p "$WORK_DIR/crates/singlefs-harness/tests/r3_defense_common"
cp "$HERE/tests/r3_defense_common/mod.rs" "$WORK_DIR/crates/singlefs-harness/tests/r3_defense_common/mod.rs"
cp "$HERE/tests/m2_safety_r3_defense_a1_a3_a4_c283.rs" "$WORK_DIR/crates/singlefs-harness/tests/m2_safety_r3_defense_a1_a3_a4_c283.rs"

echo "==> 编译 + 跑（按项目纪律套内存与线程上限；这两个数按跑的人的机器调）"
cd "$WORK_DIR"
MEMORY_CAP="${SINGLEFS_RERUN_MEMORY_CAP:-10G}"
THREAD_CAP="${SINGLEFS_RERUN_THREAD_CAP:-6}"
bash "$REPO_ROOT/research/scripts/run-with-memory-cap.sh" "$MEMORY_CAP" \
    bash "$REPO_ROOT/research/scripts/capped.sh" "$THREAD_CAP" \
    cargo test -p singlefs-harness --test m2_safety_r3_defense_a1_a3_a4_c283 -- --nocapture --test-threads 1
