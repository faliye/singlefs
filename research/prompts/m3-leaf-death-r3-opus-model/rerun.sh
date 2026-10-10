#!/usr/bin/env bash
# m3-leaf-death-r3 攻方腿（Opus）模型的复跑。模型（src/）不依赖 crates/；copybuild / copyrun-* 几段拷 SINGLEFS_REPO 的 crates/ 进
# <输出目录>/repo，用 copy/patch_checker.py 给副本的 CPU checker 加三个环境变量开关，再加一个 harness 测试目标（copy/opus_r3_leaf_death_images.rs）。
# SINGLEFS_REPO 给冻结树（~/.local/share/singlefs/work/m3-leaf-death-r1/tree）或仓根都行（要 Cargo.toml、Cargo.lock、.cargo、crates）。
# 每一段整条经内存包装与线程上限起（编译两段 6G、其余 4G），例：
#   M=research/prompts/m3-leaf-death-r3-opus-model; S=research/scripts
#   bash $S/run-with-memory-cap.sh 6G bash $S/capped.sh 6 bash $M/rerun.sh <空的输出目录> build
#   bash $S/run-with-memory-cap.sh 4G bash $S/capped.sh 6 bash $M/rerun.sh <输出目录> sweep-0
# 段（按这个次序各跑一次就是全量）：build sweep-0 sweep-1 sweep-2 sweep-3 sweep-4 clone determinism copybuild
#   copyrun-today-k2 copyrun-r1-k2 copyrun-r2-k2 copyrun-today-k3 copyrun-drop-k3 existing-r1 existing-r2 summarize
# 产物：<输出目录>/*.out 与 summary/（compare.sh 比的那几份）。线程数读 R3_THREADS（默认 6），只影响挂钟。
set -euo pipefail
MODEL_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
OUT=${1:?用法：rerun.sh <输出目录> [段…]}
shift
mkdir -p "$OUT"
OUT=$(cd "$OUT" && pwd)
BIN="$OUT/target/release/m3-leaf-death-r3-opus-model"
THREADS=${R3_THREADS:-6}
model() { R3_THREADS=$THREADS nice -n 19 "$BIN" "$@"; }
copyrun() {
    local mode=$1 filter=$2 env=()
    case "$mode" in today) ;; r1) env=(OPUS_R3_RANGE=r1) ;; r2) env=(OPUS_R3_RANGE=r2) ;; drop) env=(OPUS_R3_DROP_I94_THIRD=1) ;; esac
    (cd "$OUT/repo" && env "${env[@]}" CARGO_TARGET_DIR="$OUT/repo-target" nice -n 19 cargo test --offline -p singlefs-harness --test opus_r3_leaf_death_images "$filter" -- --nocapture --test-threads 1) > "$OUT/copy-$mode-$filter.log" 2>&1
    grep -E '^test result' "$OUT/copy-$mode-$filter.log"
}
stage() {
    case "$1" in
        build) CARGO_TARGET_DIR="$OUT/target" nice -n 19 cargo build --release --offline --manifest-path "$MODEL_DIR/Cargo.toml" ;;
        sweep-0) model sweep 0:300 > "$OUT/sweep-0.out" ;;
        sweep-1) model sweep 300:600 > "$OUT/sweep-1.out" ;;
        sweep-2) model sweep 600:900 > "$OUT/sweep-2.out" ;;
        sweep-3) model sweep 900:1152 > "$OUT/sweep-3.out" ;;
        sweep-4) model sweep 1152:2304 > "$OUT/sweep-4.out" ;;
        clone) model clone > "$OUT/clone.out" ;;
        determinism) model determinism 1152:1440 > "$OUT/determinism.out" ;;
        copybuild)
            local repo=${SINGLEFS_REPO:?copybuild 要 SINGLEFS_REPO}
            rm -rf "${OUT:?}/repo"
            mkdir -p "$OUT/repo"
            rsync -a --exclude target --exclude .git "$repo/Cargo.toml" "$repo/Cargo.lock" "$repo/.cargo" "$repo/crates" "$OUT/repo/"
            python3 "$MODEL_DIR/copy/patch_checker.py" "$OUT/repo/crates/singlefs-checker/src/walk.rs"
            cp "$MODEL_DIR/copy/opus_r3_leaf_death_images.rs" "$OUT/repo/crates/singlefs-harness/tests/"
            (cd "$OUT/repo" && CARGO_TARGET_DIR="$OUT/repo-target" nice -n 19 cargo test --offline -p singlefs-harness --test opus_r3_leaf_death_images --no-run) ;;
        copyrun-today-k2) copyrun today opus_r3_range ;;
        copyrun-r1-k2) copyrun r1 opus_r3_range ;;
        copyrun-r2-k2) copyrun r2 opus_r3_range ;;
        copyrun-today-k3) copyrun today opus_r3_right_half ;;
        copyrun-drop-k3) copyrun drop opus_r3_right_half ;;
        existing-r1|existing-r2)
            # 仓里已有的删除态判法用例里名字带 tombstone 的那几条（今天的 I-9.18 / I-9.19 / I-9.20 坏镜像在这里），在区间墓碑最小改法下跑；
            # 用例本身断言今天的判定，改法下断言不成立的那几条就是改法放过或改了判定的坏镜像。cargo test 失败（退 101）是要的结果，不算这一段失败。
            local mode=${1#existing-}
            (cd "$OUT/repo" && OPUS_R3_RANGE=$mode CARGO_TARGET_DIR="$OUT/repo-target" nice -n 19 cargo test --offline -p singlefs-harness \
                --test pool_checker_judges_the_deletion_state_of_every_walked_version tombstone -- --test-threads 2) > "$OUT/existing-$mode.log" 2>&1 || true
            grep -E '^test result' "$OUT/existing-$mode.log" ;;
        summarize)
            mkdir -p "$OUT/summary"
            cat "$OUT"/sweep-*.out | awk -f "$MODEL_DIR/tools/agg.awk" | sort > "$OUT/summary/agg.txt"
            cat "$OUT"/sweep-*.out | awk -f "$MODEL_DIR/tools/coverage.awk" | sort > "$OUT/summary/coverage.txt"
            cp "$OUT/clone.out" "$OUT/summary/clone.txt"
            cp "$OUT/determinism.out" "$OUT/summary/determinism.txt"
            cat "$OUT"/copy-*-opus_r3_range.log | grep -o 'OPUSR3 k2 .*' | sort > "$OUT/summary/copy-k2.txt"
            cat "$OUT"/copy-*-opus_r3_right_half.log | grep -o 'OPUSR3 k3 .*' | sort > "$OUT/summary/copy-k3.txt"
            for m in r1 r2; do grep -E '^test .* \.\.\. (ok|FAILED)$|^test result' "$OUT/existing-$m.log" | sed -E 's/finished in [0-9.]+s//' | sort > "$OUT/summary/existing-$m.txt"; done
            (cd "$OUT" && for f in sweep-0.out sweep-1.out sweep-2.out sweep-3.out sweep-4.out; do printf '%s  %s\n' "$(sort "$f" | sha256sum | cut -d' ' -f1)" "$f"; done) > "$OUT/summary/sorted-output-sha256.txt" ;;
        *) echo "不认的段：$1" >&2; exit 2 ;;
    esac
}
if [[ $# -eq 0 ]]; then
    set -- build sweep-0 sweep-1 sweep-2 sweep-3 sweep-4 clone determinism copybuild copyrun-today-k2 copyrun-r1-k2 copyrun-r2-k2 copyrun-today-k3 copyrun-drop-k3 existing-r1 existing-r2 summarize
fi
for s in "$@"; do echo "== $s"; stage "$s"; done
