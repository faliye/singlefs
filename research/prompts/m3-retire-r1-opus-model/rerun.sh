#!/usr/bin/env bash
# m3-retire-r1 攻方腿（opus）模型的复跑。路径一律从环境变量与参数取：
#   RETIRE_R1_MODEL_DIR  模型目录（本目录）；SINGLEFS_REPO  拷 crates/ 改副本用的树（冻结树或仓根：要 Cargo.toml、Cargo.lock、.cargo、crates）。
# 每一阶段由调用方整条经内存包装与线程上限起（copybuild 6G，其余 4G），例：
#   S=<仓根>/scripts; export RETIRE_R1_MODEL_DIR=<本目录> SINGLEFS_REPO=<冻结树>
#   bash $S/run-with-memory-cap.sh 6G bash $S/capped.sh 10 bash $RETIRE_R1_MODEL_DIR/rerun.sh <空的输出目录> copybuild
#   bash $S/run-with-memory-cap.sh 4G bash $S/capped.sh 10 bash $RETIRE_R1_MODEL_DIR/rerun.sh <输出目录> copyrun
# 阶段（按这个次序各跑一次就是全量）：copybuild copyrun demo sweep-0 sweep-1 sweep-2 sweep-3 sweep-4
#   random-0 random-1 random-2 random-3 random-norb
# 只有 copybuild 编译（cargo，只编 singlefs-core 的一个集成测试目标，自己的 target 目录）；sweep-0..3 各约 2.5 分钟，别的都在 30 秒内。
# 比对：bash compare.sh <输出目录>（不依赖 SINGLEFS_REPO）。
set -euo pipefail
MODEL_DIR=${RETIRE_R1_MODEL_DIR:?要 RETIRE_R1_MODEL_DIR（模型目录）}
OUT=${1:?用法：rerun.sh <输出目录> [阶段…]}
shift
mkdir -p "$OUT"
OUT=$(cd "$OUT" && pwd)
model() { nice -n 19 python3 -I "$MODEL_DIR/model/retire_model.py" "$@"; }
stage() {
    case "$1" in
        copybuild)
            local repo=${SINGLEFS_REPO:?copybuild 要 SINGLEFS_REPO}
            rm -rf "${OUT:?}/repo"
            mkdir -p "$OUT/repo"
            rsync -a --exclude target --exclude .git "$repo/Cargo.toml" "$repo/Cargo.lock" "$repo/.cargo" "$repo/crates" "$OUT/repo/"
            mkdir -p "$OUT/repo/crates/singlefs-core/tests"
            cp "$MODEL_DIR/copy/opus_retire_r1_counts.rs" "$OUT/repo/crates/singlefs-core/tests/"
            (cd "$OUT/repo" && CARGO_TARGET_DIR="$OUT/target" nice -n 19 cargo test --offline -p singlefs-core \
                --test opus_retire_r1_counts --no-run) > "$OUT/copybuild.log" 2>&1 ;;
        copyrun)
            (cd "$OUT/repo" && CARGO_TARGET_DIR="$OUT/target" nice -n 19 cargo test --offline -p singlefs-core \
                --test opus_retire_r1_counts -- --nocapture --test-threads 1) > "$OUT/copy.log" 2>&1
            grep -E '^R ' "$OUT/copy.log" > "$OUT/copy.out"
            grep -E '^test result' "$OUT/copy.log" ;;
        demo) model demo 8 > "$OUT/demo.out" ;;
        sweep-0|sweep-1|sweep-2|sweep-3) model sweep "${1#sweep-}" 7 > "$OUT/$1-7.out" ;;
        sweep-4) model sweep 4 6 > "$OUT/sweep-4-6.out" ;;
        random-0|random-1|random-2|random-3)
            for seed in 11 12; do model random "${1#random-}" "$seed" 20000 40 > "$OUT/$1-$seed.out"; done ;;
        random-norb)
            for c in 0 3; do model random-norb "$c" 11 20000 40 > "$OUT/random-norb-$c-11.out"; done ;;
        *) echo "不认识的阶段：$1" >&2; exit 2 ;;
    esac
}
for s in "$@"; do stage "$s"; done
