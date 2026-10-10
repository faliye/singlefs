#!/usr/bin/env bash
# m3-leaf-death-r3 攻方腿第二次抽样（opus-s2）模型的复跑。路径一律从环境变量与参数取，不靠这份脚本自己在仓里的位置：
#   R3S2_MODEL_DIR  模型目录（本目录）；SINGLEFS_REPO  拷 crates/ 改副本用的树（冻结树或仓根：要 Cargo.toml、Cargo.lock、.cargo、crates）。
# 每一阶段由调用方整条经内存包装与线程上限起（编译类 6G、其余 4G），例：
#   S=<仓根>/research/scripts; export R3S2_MODEL_DIR=<本目录> SINGLEFS_REPO=<冻结树>
#   bash $S/run-with-memory-cap.sh 6G bash $S/capped.sh 6 bash $R3S2_MODEL_DIR/rerun.sh <空的输出目录> build
# 阶段（按这个次序各跑一次就是全量）：build sweep-0 sweep-1 sweep-2 sweep-3 sweep-4 demos trace determinism clone
#   copybuild copyrun-k2 copyrun-k3 copyrun-writer corelib summarize
# 编译类：build、copybuild（copyrun-* 与 corelib 在 copybuild 之后只剩链接与跑）。线程数读 R2_THREADS（默认 6），只影响挂钟。
set -euo pipefail
MODEL_DIR=${R3S2_MODEL_DIR:?要 R3S2_MODEL_DIR（模型目录）}
OUT=${1:?用法：rerun.sh <输出目录> [阶段…]}
shift
mkdir -p "$OUT"
OUT=$(cd "$OUT" && pwd)
BIN="$OUT/target/release/m3-leaf-death-r3-opus-s2-model"
THREADS=${R2_THREADS:-6}
model() { R2_THREADS=$THREADS nice -n 19 "$BIN" "$@"; }
copytest() {
    local filter=$1 log=$2
    (cd "$OUT/repo" && CARGO_TARGET_DIR="$OUT/repo-target" nice -n 19 cargo test --release --offline -p singlefs-harness \
        --test opus_r3s2_leaf_death_images "$filter" -- --nocapture --test-threads 1) > "$OUT/$log" 2>&1
    grep -E '^test result' "$OUT/$log"
}
stage() {
    case "$1" in
        build) CARGO_TARGET_DIR="$OUT/target" nice -n 19 cargo build --release --offline --manifest-path "$MODEL_DIR/Cargo.toml" ;;
        sweep-0) model sweep 0:96 > "$OUT/sweep-0.out" 2> "$OUT/sweep-0.err" ;;
        sweep-1) model sweep 96:192 > "$OUT/sweep-1.out" 2> "$OUT/sweep-1.err" ;;
        sweep-2) model sweep 192:240 > "$OUT/sweep-2.out" 2> "$OUT/sweep-2.err" ;;
        sweep-3) model sweep 240:288 > "$OUT/sweep-3.out" 2> "$OUT/sweep-3.err" ;;
        sweep-4) model sweep 288:368 > "$OUT/sweep-4.out" 2> "$OUT/sweep-4.err" ;;
        demos) model demo all > "$OUT/demos.out" ;;
        trace) model trace 64 > "$OUT/trace-64.out" ;;
        determinism) model determinism > "$OUT/determinism.out" ;;
        clone)
            nice -n 19 python3 -I "$MODEL_DIR/clone/clone_heads.py" 4 > "$OUT/clone.out"
            nice -n 19 python3 -I "$MODEL_DIR/clone/clone_heads.py" trace zero_path_rebirth_retirement K.unlink_min > "$OUT/clone-trace.out" ;;
        copybuild)
            local repo=${SINGLEFS_REPO:?copybuild 要 SINGLEFS_REPO}
            rm -rf "${OUT:?}/repo"
            mkdir -p "$OUT/repo"
            rsync -a --exclude target --exclude .git "$repo/Cargo.toml" "$repo/Cargo.lock" "$repo/.cargo" "$repo/crates" "$OUT/repo/"
            python3 -I "$MODEL_DIR/copy/patch_copy.py" "$OUT/repo"
            cp "$MODEL_DIR/copy/opus_r3s2_leaf_death_images.rs" "$OUT/repo/crates/singlefs-harness/tests/"
            (cd "$OUT/repo" && CARGO_TARGET_DIR="$OUT/repo-target" nice -n 19 cargo test --release --offline -p singlefs-harness --test opus_r3s2_leaf_death_images --no-run)
            (cd "$OUT/repo" && CARGO_TARGET_DIR="$OUT/repo-target" nice -n 19 cargo test --offline -p singlefs-core --lib --no-run) ;;
        copyrun-k2) copytest opus_r3s2_range_tombstones copy-k2.log ;;
        copyrun-k3) copytest opus_r3s2_container_number_family copy-k3.log ;;
        copyrun-writer) copytest opus_r3s2_wrong_right_half_number_written_by_the_writer copy-writer.log ;;
        corelib)
            : > "$OUT/corelib.out"
            for cfg in "0 0" "1 0" "40 0" "1 1"; do
                set -- $cfg
                rc=0
                (cd "$OUT/repo" && OPUS_R3S2_RIGHT_HALF_NUMBER_OFFSET=$1 OPUS_R3S2_N1_SEPARATOR=$2 CARGO_TARGET_DIR="$OUT/repo-target" \
                    nice -n 19 cargo test --offline -p singlefs-core --lib -- inode_tree) > "$OUT/corelib-$1-$2.log" 2>&1 || rc=$?
                { echo "offset=$1 n1_separator=$2 rc=$rc $(grep -E '^test result' "$OUT/corelib-$1-$2.log" | sed 's/; finished in .*//')"
                  { grep -E '^test .* FAILED$' "$OUT/corelib-$1-$2.log" || true; } | sort | sed "s/^/offset=$1 n1_separator=$2 /"; } >> "$OUT/corelib.out"
            done
            cat "$OUT/corelib.out" ;;
        summarize)
            mkdir -p "$OUT/summary"
            cat "$OUT"/sweep-*.out | awk -f "$MODEL_DIR/tools/agg.awk" | sort > "$OUT/summary/agg.txt"
            awk -f "$MODEL_DIR/tools/group.awk" "$OUT/summary/agg.txt" | sort > "$OUT/summary/group.txt"
            cat "$OUT"/sweep-*.out | awk -f "$MODEL_DIR/tools/counts.awk" | sort > "$OUT/summary/counts.txt"
            awk -f "$MODEL_DIR/tools/demos.awk" "$OUT/demos.out" | sort > "$OUT/summary/demos.txt"
            grep 'name=demo_counts' "$OUT/demos.out" | grep 'suffix=none' | sort > "$OUT/summary/demo-prefix-counts.txt"
            sed -n '/^FIRST/,/^publish/p' "$OUT/trace-64.out" > "$OUT/summary/trace-64-first.txt"
            cp "$OUT/determinism.out" "$OUT/summary/determinism.txt"
            sort "$OUT/clone.out" > "$OUT/summary/clone.txt"
            cp "$OUT/clone-trace.out" "$OUT/summary/clone-trace.txt"
            cat "$OUT"/copy-k2.log "$OUT"/copy-k3.log "$OUT"/copy-writer.log | grep -o 'OPUSR3S2 .*' | sort > "$OUT/summary/copy.txt"
            cp "$OUT/corelib.out" "$OUT/summary/corelib.txt"
            (cd "$OUT" && for f in sweep-0.out sweep-1.out sweep-2.out sweep-3.out sweep-4.out demos.out trace-64.out; do
                printf '%s  %s\n' "$(sort "$f" | sha256sum | cut -d' ' -f1)" "$f"; done) > "$OUT/summary/sorted-output-sha256.txt" ;;
        *) echo "不认的阶段：$1" >&2; exit 2 ;;
    esac
}
if [[ $# -eq 0 ]]; then
    set -- build sweep-0 sweep-1 sweep-2 sweep-3 sweep-4 demos trace determinism clone copybuild copyrun-k2 copyrun-k3 copyrun-writer corelib summarize
fi
for s in "$@"; do echo "== $s"; stage "$s"; done
