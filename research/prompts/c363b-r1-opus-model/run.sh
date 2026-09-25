#!/usr/bin/env bash
# c363b-r1 攻方腿（V1）复跑：冻结副本拷进草稿目录，加一个测试文件 v1_reach.rs，只编译、只跑这一个测试二进制。
#   bash run.sh [冻结副本] [草稿目录] [输出目录]
# 默认：冻结副本 /tmp/claude-1000/c363b-r1/tree；草稿 /tmp/claude-1000/c363b-r1-opus/rerun；输出 = 本目录的 results/。
# 冻结副本只读；拷贝先按 research/prompts/c363b-r1-snapshot/crates-src-sha256.txt 核一遍。线程上限 5。
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/../../.." && pwd)"
FROZEN="${1:-/tmp/claude-1000/c363b-r1/tree}"
DRAFT="${2:-/tmp/claude-1000/c363b-r1-opus/rerun}"
OUT="${3:-$HERE/results}"
mkdir -p "$DRAFT" "$OUT"
rsync -a --delete --exclude target --exclude .git "$FROZEN/" "$DRAFT/tree/"
(cd "$DRAFT/tree" && sha256sum -c --quiet "$REPO/research/prompts/c363b-r1-snapshot/crates-src-sha256.txt")
cp "$HERE/v1_reach.rs" "$DRAFT/tree/crates/singlefs-harness/tests/v1_reach.rs"
cd "$DRAFT/tree"
bash "$REPO/research/scripts/capped.sh" 5 nice -n 19 cargo build --release -p singlefs-harness --test v1_reach
V1_WIDTH_STEP=2 V1_CHECKER_BUDGET=2 \
  bash "$REPO/research/scripts/capped.sh" 5 nice -n 19 cargo test --release -p singlefs-harness --test v1_reach -- --nocapture \
  > "$OUT/v1_reach.out" 2>&1
V1_CLASSES=D V1_WIDTH_STEP=8 V1_TRACE='D-raise,136,k=20,s=1000' \
  bash "$REPO/research/scripts/capped.sh" 5 nice -n 19 cargo test --release -p singlefs-harness --test v1_reach -- --nocapture 2>&1 \
  | grep '^S' > "$OUT/trace-D-raise-136-k20-s1000.txt"
V1_CLASSES=A V1_WIDTH_STEP=2 V1_TRACE='A-mkfs-process-overwrite,66,K=72' \
  bash "$REPO/research/scripts/capped.sh" 5 nice -n 19 cargo test --release -p singlefs-harness --test v1_reach -- --nocapture 2>&1 \
  | grep '^S' > "$OUT/trace-A-66.txt"
V1_CLASSES=C V1_WIDTH_STEP=8 V1_TRACE='C-live-data-then-inodes,64,n=24' \
  bash "$REPO/research/scripts/capped.sh" 5 nice -n 19 cargo test --release -p singlefs-harness --test v1_reach -- --nocapture 2>&1 \
  | grep '^S' > "$OUT/trace-C-64-n24.txt"
V1_CLASSES=C V1_WIDTH_STEP=8 V1_TRACE='C-live-data-then-inodes,120,n=48' \
  bash "$REPO/research/scripts/capped.sh" 5 nice -n 19 cargo test --release -p singlefs-harness --test v1_reach -- --nocapture 2>&1 \
  | grep '^S' > "$OUT/trace-C-120-n48.txt"
# 副本改法 Z-restore（只在这一份副本上）：抬 F 第一次空发布就被拒时分配器退回进来时的样子；只跑 D、H、I 三类对照。
rsync -a --delete --exclude target --exclude .git "$FROZEN/" "$DRAFT/tree-z/"
(cd "$DRAFT/tree-z" && patch -p1 < "$HERE/copy-probe/z-restore.patch")
cp "$HERE/v1_reach.rs" "$DRAFT/tree-z/crates/singlefs-harness/tests/v1_reach.rs"
(cd "$DRAFT/tree-z" && bash "$REPO/research/scripts/capped.sh" 5 nice -n 19 cargo build --release -p singlefs-harness --test v1_reach)
(cd "$DRAFT/tree-z" && V1_CLASSES=DHI V1_WIDTH_STEP=2 V1_CHECKER_BUDGET=2 \
  bash "$REPO/research/scripts/capped.sh" 5 nice -n 19 cargo test --release -p singlefs-harness --test v1_reach -- --nocapture) \
  > "$OUT/v1_reach.z-restore.out" 2>&1
bash "$HERE/summarize.sh" "$OUT/v1_reach.z-restore.out" > "$OUT/summary.z-restore.txt"
bash "$HERE/summarize.sh" "$OUT/v1_reach.out" > "$OUT/summary.txt"
grep -E 'test result' "$OUT/v1_reach.out"
