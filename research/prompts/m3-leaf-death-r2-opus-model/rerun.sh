#!/usr/bin/env bash
# m3-leaf-death-r2 攻方腿（Opus）模型的复跑。模型本身不依赖 crates/；copybuild / copyrun 两阶段拷 SINGLEFS_REPO 的 crates/ 进
# <输出目录>/repo 改副本（加一个 harness 测试目标），给冻结树或仓根都行（要 Cargo.toml、Cargo.lock、.cargo、crates，不要 research/）。
# 每一阶段整条经内存包装与线程上限起（编译两阶段 6G、其余 4G），例：
#   M=research/prompts/m3-leaf-death-r2-opus-model; S=scripts
#   bash $S/run-with-memory-cap.sh 6G bash $S/capped.sh 6 bash $M/rerun.sh <空的输出目录> build
#   bash $S/run-with-memory-cap.sh 4G bash $S/capped.sh 6 bash $M/rerun.sh <输出目录> demos
# 阶段（按这个次序各跑一次就是全量）：build demos sweep-0 sweep-1 sweep-2 sweep-3 trace determinism copybuild copyrun summarize
# 产物：<输出目录>/*.out 与 summary/（compare.sh 比的那几份）。线程数读 R2_THREADS（默认 6），只影响挂钟。
set -euo pipefail
MODEL_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
OUT=${1:?用法：rerun.sh <输出目录> [阶段…]}
shift
mkdir -p "$OUT"
OUT=$(cd "$OUT" && pwd)
BIN="$OUT/target/release/m3-leaf-death-r2-opus-model"
THREADS=${R2_THREADS:-6}
model() { R2_THREADS=$THREADS nice -n 19 "$BIN" "$@"; }
stage() {
    case "$1" in
        build) CARGO_TARGET_DIR="$OUT/target" nice -n 19 cargo build --release --offline --manifest-path "$MODEL_DIR/Cargo.toml" ;;
        demos) model demo all > "$OUT/demos.out" ;;
        sweep-0) model sweep 0:96 > "$OUT/sweep-0.out" ;;
        sweep-1) model sweep 96:192 > "$OUT/sweep-1.out" ;;
        sweep-2) model sweep 192:288 > "$OUT/sweep-2.out" ;;
        sweep-3) model sweep 288:576 > "$OUT/sweep-3.out" ;;
        trace) model trace 395 > "$OUT/trace-395.out" ;;
        determinism) model determinism > "$OUT/determinism.out" ;;
        copybuild)
            local repo=${SINGLEFS_REPO:?copybuild 要 SINGLEFS_REPO}
            rm -rf "${OUT:?}/repo"
            mkdir -p "$OUT/repo"
            rsync -a --exclude target --exclude .git "$repo/Cargo.toml" "$repo/Cargo.lock" "$repo/.cargo" "$repo/crates" "$OUT/repo/"
            cp "$MODEL_DIR/copy/opus_r2_leaf_death_images.rs" "$OUT/repo/crates/singlefs-harness/tests/"
            (cd "$OUT/repo" && CARGO_TARGET_DIR="$OUT/repo-target" nice -n 19 cargo test --offline -p singlefs-harness --test opus_r2_leaf_death_images --no-run) ;;
        copyrun)
            (cd "$OUT/repo" && CARGO_TARGET_DIR="$OUT/repo-target" nice -n 19 cargo test --offline -p singlefs-harness --test opus_r2_leaf_death_images -- --nocapture --test-threads 1) > "$OUT/copy.log" 2>&1
            grep -o 'OPUSR2 .*' "$OUT/copy.log" > "$OUT/copy.out"
            grep -E '^test result' "$OUT/copy.log" ;;
        summarize)
            mkdir -p "$OUT/summary"
            cat "$OUT"/sweep-*.out | awk -f "$MODEL_DIR/tools/agg.awk" | sort > "$OUT/summary/agg.txt"
            cat "$OUT"/sweep-*.out | awk -f "$MODEL_DIR/tools/counts.awk" | sort > "$OUT/summary/counts.txt"
            awk -f "$MODEL_DIR/tools/demos.awk" "$OUT/demos.out" | sort > "$OUT/summary/demos.txt"
            sed -n '/^FIRST/,/^publish/p' "$OUT/trace-395.out" > "$OUT/summary/trace-395-first.txt"
            cp "$OUT/determinism.out" "$OUT/summary/determinism.txt"
            sort "$OUT/copy.out" > "$OUT/summary/copy.txt"
            (cd "$OUT" && for f in demos.out sweep-0.out sweep-1.out sweep-2.out sweep-3.out trace-395.out; do printf '%s  %s\n' "$(sort "$f" | sha256sum | cut -d' ' -f1)" "$f"; done) > "$OUT/summary/sorted-output-sha256.txt" ;;
        *) echo "不认的阶段：$1" >&2; exit 2 ;;
    esac
}
if [[ $# -eq 0 ]]; then
    set -- build demos sweep-0 sweep-1 sweep-2 sweep-3 trace determinism copybuild copyrun summarize
fi
for s in "$@"; do echo "== $s"; stage "$s"; done
