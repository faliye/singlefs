#!/usr/bin/env bash
# S4 二轮云端正推腿（Sonnet）的复跑脚本。
# 前置：本仓根目录有 crates/（今天的实现）。这份脚本不改动本仓，只在一个临时拷贝上打补丁、编译、跑测试。
# 用法：bash rerun.sh <本仓根目录> [<临时目录，默认 /tmp/s4-r2-sonnet-rerun>]
set -euo pipefail
REPO_ROOT="${1:?用法：bash rerun.sh <本仓根目录> [<临时目录>]}"
WORK_DIR="${2:-/tmp/s4-r2-sonnet-rerun}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "==> 从 $REPO_ROOT 拷贝 crates/ 到 $WORK_DIR"
rm -rf "${WORK_DIR:?}"
mkdir -p "$WORK_DIR"
cp -r "$REPO_ROOT/crates" "$WORK_DIR/crates"
cp "$REPO_ROOT/Cargo.toml" "$WORK_DIR/Cargo.toml"
cp "$REPO_ROOT/Cargo.lock" "$WORK_DIR/Cargo.lock"

echo "==> 打补丁（diff/ 下六个文件的改动，patch -p0 风格）"
for patch_file in "$HERE"/diff/*.patch; do
    echo "  -- $patch_file"
    (cd "$WORK_DIR" && patch -p1 < "$patch_file")
done

echo "==> 放进对拍测试"
cp "$HERE/tests/s4_r2_candidates.rs" "$WORK_DIR/crates/singlefs-harness/tests/s4_r2_candidates.rs"

echo "==> 编译 + 跑（按项目纪律套内存与线程上限；这两个数按跑的人的机器调）"
cd "$WORK_DIR"
MEMORY_CAP="${SINGLEFS_RERUN_MEMORY_CAP:-10G}"
THREAD_CAP="${SINGLEFS_RERUN_THREAD_CAP:-8}"
bash "$REPO_ROOT/research/scripts/run-with-memory-cap.sh" "$MEMORY_CAP" \
    bash "$REPO_ROOT/research/scripts/capped.sh" "$THREAD_CAP" \
    cargo test -p singlefs-core --lib admission::
bash "$REPO_ROOT/research/scripts/run-with-memory-cap.sh" "$MEMORY_CAP" \
    bash "$REPO_ROOT/research/scripts/capped.sh" "$THREAD_CAP" \
    cargo test -p singlefs-harness --test s4_r2_candidates -- --nocapture --test-threads 1
# 上面这条里 width=384、candidate=a3 的 k 循环（0..=80）跑满要超过 40 分钟（checker 成本随历史变长）；
# 本腿的报告只测到 k=66 就因时长预算主动停止。width=384、candidate=A4 的数据靠下面这份独立小测试补上（26 秒内跑完）：
cp "$HERE/tests/s4_r2_width384_a4_supplement.rs" "$WORK_DIR/crates/singlefs-harness/tests/s4_r2_width384_a4_supplement.rs"
bash "$REPO_ROOT/research/scripts/run-with-memory-cap.sh" "$MEMORY_CAP" \
    bash "$REPO_ROOT/research/scripts/capped.sh" "$THREAD_CAP" \
    cargo test -p singlefs-harness --test s4_r2_width384_a4_supplement -- --nocapture
