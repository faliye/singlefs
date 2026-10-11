#!/usr/bin/env bash
# m3-leaf-death-r3 正推腿（Sonnet）模型的复跑。模型本身不依赖 crates/（src/model.rs、src/clone.rs）；copybuild / copyrun 两阶段拷 SINGLEFS_REPO 的 crates/ 进
# <输出目录>/repo 改副本（加一个 harness 测试目标、给 I-9.4 第三句加一个开关），给冻结树 ~/.local/share/singlefs/work/m3-leaf-death-r1/tree 或仓根都行
# （要 Cargo.toml、Cargo.lock、.cargo、crates，不要 research/）。仓路径、输出目录一律从环境变量 / 参数取，不靠本脚本在仓里的位置。
# 每一阶段整条经内存包装起（build 与 copybuild 6G、其余 4G），例（工作目录随意，S 指仓里的 scripts）：
#   M=<本目录>; S=<仓>/scripts; export SINGLEFS_REPO=<冻结树或仓根>
#   bash $S/run-with-memory-cap.sh 6G bash $M/rerun.sh <空的输出目录> build
#   bash $S/run-with-memory-cap.sh 4G bash $M/rerun.sh <输出目录> sweep-0
#   …（sweep-0 … sweep-15 各一条，clone、determinism 各一条）
#   bash $S/run-with-memory-cap.sh 6G bash $M/rerun.sh <输出目录> copybuild
#   bash $S/run-with-memory-cap.sh 4G bash $M/rerun.sh <输出目录> copyrun summarize
#   bash $M/compare.sh <输出目录>
# 阶段：build sweep-0 … sweep-15 clone determinism copybuild copyrun summarize。线程数读 R3_THREADS（默认 6），只影响挂钟、不影响输出。
# 产物：<输出目录>/*.out（模型行）与 summary/（compare.sh 比的那几份）。
set -euo pipefail
MODEL_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
OUT=${1:?用法：rerun.sh <输出目录> [阶段…]}
shift
mkdir -p "$OUT"
OUT=$(cd "$OUT" && pwd)
BIN="$OUT/target/release/m3-leaf-death-r3-sonnet-model"
THREADS=${R3_THREADS:-6}
model() { R3_THREADS=$THREADS nice -n 19 "$BIN" "$@"; }
# 阶段 sweep-k 的配置下标区间
sweep_range() {
    case "$1" in
        0) echo 0:189 ;; 1) echo 189:378 ;; 2) echo 378:1638 ;; 3) echo 1638:1827 ;; 4) echo 1827:2016 ;;
        5) echo 2016:2079 ;; 6) echo 2079:2142 ;; 7) echo 2142:2205 ;; 8) echo 2205:2268 ;;
        9) echo 2268:2412 ;; 10) echo 2412:2556 ;; 11) echo 2556:2700 ;; 12) echo 2700:2844 ;;
        13) echo 2844:3036 ;; 14) echo 3036:3192 ;; 15) echo 3192:3300 ;;
        *) echo "不认的批：$1" >&2; return 2 ;;
    esac
}
stage() {
    case "$1" in
        build) CARGO_TARGET_DIR="$OUT/target" nice -n 19 cargo build --release --offline --manifest-path "$MODEL_DIR/Cargo.toml" ;;
        sweep-[0-9]|sweep-1[0-5])
            local k=${1#sweep-}
            model sweep "$(sweep_range "$k")" > "$OUT/sweep-$k.out"
            tail -n 1 "$OUT/sweep-$k.out" ;;
        clone) model clone > "$OUT/clone.out" ;;
        determinism) model determinism > "$OUT/determinism.out" ;;
        copybuild)
            local repo=${SINGLEFS_REPO:?copybuild 要 SINGLEFS_REPO}
            rm -rf "${OUT:?}/repo"
            mkdir -p "$OUT/repo"
            rsync -a --exclude target --exclude .git "$repo/Cargo.toml" "$repo/Cargo.lock" "$repo/.cargo" "$repo/crates" "$OUT/repo/"
            cp "$MODEL_DIR/copy/sonnet_r3_leaf_death_images.rs" "$OUT/repo/crates/singlefs-harness/tests/"
            python3 "$MODEL_DIR/tools/patch_checker.py" "$OUT/repo/crates/singlefs-checker/src/walk.rs"
            (cd "$OUT/repo" && CARGO_TARGET_DIR="$OUT/repo-target" nice -n 19 cargo test --offline -p singlefs-harness --test sonnet_r3_leaf_death_images --no-run) ;;
        copyrun)
            (cd "$OUT/repo" && CARGO_TARGET_DIR="$OUT/repo-target" nice -n 19 cargo test --offline -p singlefs-harness --test sonnet_r3_leaf_death_images -- --nocapture --test-threads 1) > "$OUT/copy.log" 2>&1
            grep -o 'SONR3 .*' "$OUT/copy.log" > "$OUT/copy.out"
            grep -E '^test result' "$OUT/copy.log" ;;
        summarize)
            mkdir -p "$OUT/summary"
            python3 "$MODEL_DIR/tools/tables.py" "$OUT/summary" "$OUT"/sweep-*.out
            grep -h 'name=clone' "$OUT/clone.out" | sort > "$OUT/summary/clone.txt"
            cp "$OUT/determinism.out" "$OUT/summary/determinism.txt"
            sort "$OUT/copy.out" > "$OUT/summary/copy.txt"
            (cd "$OUT" && for f in sweep-0.out sweep-1.out sweep-2.out sweep-3.out sweep-4.out sweep-5.out sweep-6.out sweep-7.out sweep-8.out sweep-9.out sweep-10.out sweep-11.out sweep-12.out sweep-13.out sweep-14.out sweep-15.out clone.out; do
                printf '%s  %s\n' "$(grep -v 'name=done' "$f" | sort | sha256sum | cut -d' ' -f1)" "$f"; done) > "$OUT/summary/sorted-output-sha256.txt"
            ls -la "$OUT/summary" ;;
        *) echo "不认的阶段：$1" >&2; exit 2 ;;
    esac
}
if [[ $# -eq 0 ]]; then
    set -- build sweep-0 sweep-1 sweep-2 sweep-3 sweep-4 sweep-5 sweep-6 sweep-7 sweep-8 sweep-9 sweep-10 sweep-11 sweep-12 sweep-13 sweep-14 sweep-15 clone determinism copybuild copyrun summarize
fi
for s in "$@"; do echo "== $s"; stage "$s"; done
