#!/usr/bin/env bash
# m3-retire-r1 攻方腿第二次抽样（opus-s2）模型的复跑。路径一律从环境变量与参数取，不靠这份脚本自己在仓里的位置：
#   S2_MODEL_DIR   模型目录（本目录）；SINGLEFS_REPO  拷 crates/ 改副本用的树（冻结树：要 Cargo.toml、Cargo.lock、.cargo、crates）。
# 每一阶段由调用方整条经内存包装与线程上限起（编译类 6G、其余 4G），例（仓根）：
#   S=scripts; export S2_MODEL_DIR=$PWD/research/prompts/m3-retire-r1-opus-s2-model SINGLEFS_REPO=<冻结树>
#   bash $S/run-with-memory-cap.sh 6G bash $S/capped.sh 16 bash $S2_MODEL_DIR/rerun.sh <空的输出目录> build
# 阶段（按这个次序各跑一次就是全量）：build sweep demos determinism copybuild copyrun summarize
# 编译类：build、copybuild。线程数读 S2_THREADS（默认 6），只影响挂钟。
set -euo pipefail
MODEL_DIR=${S2_MODEL_DIR:?要 S2_MODEL_DIR（模型目录）}
OUT=${1:?用法：rerun.sh <输出目录> [阶段…]}
shift
mkdir -p "$OUT"
OUT=$(cd "$OUT" && pwd)
BIN="$OUT/target/release/m3-retire-r1-opus-s2-model"
model() { nice -n 19 "$BIN" "$@"; }
stage() {
    case "$1" in
        build) CARGO_TARGET_DIR="$OUT/target" nice -n 19 cargo build --release --offline --manifest-path "$MODEL_DIR/Cargo.toml" ;;
        sweep) model sweep 0:304 > "$OUT/sweep.out" 2> "$OUT/sweep.err" ;;
        demos)
            model demo double > "$OUT/double.out"
            model demo trace > "$OUT/trace.out"
            model demo cascade > "$OUT/cascade.out" ;;
        determinism) model determinism > "$OUT/determinism.out" ;;
        copybuild)
            local repo=${SINGLEFS_REPO:?copybuild 要 SINGLEFS_REPO}
            mkdir -p "$OUT/repo"
            rsync -a --exclude target --exclude .git "$repo/" "$OUT/repo/"
            cp "$MODEL_DIR/copy/opus_s2_retire_history.rs" "$OUT/repo/crates/singlefs-harness/tests/"
            (cd "$OUT/repo" && CARGO_TARGET_DIR="$OUT/repo-target" nice -n 19 cargo test --release --offline -p singlefs-harness \
                --test opus_s2_retire_history --no-run) > "$OUT/copybuild.log" 2>&1 ;;
        copyrun)
            (cd "$OUT/repo" && CARGO_TARGET_DIR="$OUT/repo-target" nice -n 19 cargo test --release --offline -p singlefs-harness \
                --test opus_s2_retire_history -- --nocapture --test-threads 1) > "$OUT/copyrun.log" 2>&1
            grep -E '^test result' "$OUT/copyrun.log"
            grep -o 'S2C .*' "$OUT/copyrun.log" > "$OUT/copy.txt" ;;
        summarize)
            mkdir -p "$OUT/summary"
            python3 -I "$MODEL_DIR/tools/summarize.py" "$OUT/sweep.out" > "$OUT/summary/sweep-summary.txt"
            sort "$OUT/sweep.out" > "$OUT/summary/sweep-sorted.txt"
            for f in double trace cascade determinism; do cp "$OUT/$f.out" "$OUT/summary/$f.txt"; done
            cp "$OUT/copy.txt" "$OUT/summary/copy.txt"
            (cd "$OUT/summary" && sha256sum sweep-summary.txt sweep-sorted.txt double.txt trace.txt cascade.txt determinism.txt copy.txt > sorted-output-sha256.txt) ;;
        *) echo "不认识的阶段：$1" >&2; exit 2 ;;
    esac
}
for s in "$@"; do stage "$s"; done
