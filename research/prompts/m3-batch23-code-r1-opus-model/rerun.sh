#!/usr/bin/env bash
# m3-batch23-code-r1 攻方模型复跑（只跑 harness 档：singlefs-harness 的两个测试目标；GPU 不跑，checker 档一个不跑）。
# 用法：
#   SINGLEFS_REPO=<含 crates/ 与 Cargo.toml、Cargo.lock 的仓或冻结副本> [SINGLEFS_TOOL=<含 run-with-memory-cap.sh、capped.sh 的目录>] \
#     bash rerun.sh <工作目录> <段…|all>
#   bash rerun.sh compare <工作目录>      # 只读工作目录与本目录的 expected-summary.txt，不读 SINGLEFS_REPO
# setup 要工作目录不存在或为空；它把 SINGLEFS_REPO 的 crates/ 与 Cargo.* 拷成 tree（加只记数的钩子 apply_capture_hook.py）
# 与 tree-fixa（钩子 + 改法 A apply_fix_a.py），模型 opus_b23_attack.rs 拷进两份的 crates/singlefs-harness/tests/。
# 段：setup build n5 n5-fixa batch-test fixa-legal n4 n6 n7 cliff st-29 st-24 st-18 st-death
#     b1-1a b1-1b b1-2a b1-2b b1-3a b1-3b b1-4a b1-4b b2-1a b2-1b b2-2a b2-2b b2-3a b2-3b b2-4a b2-4b
# all = 依次跑全部；批段与自证段四段一组并行（每段退出码写进 logs/<段>.rc，单个不带参数的 wait 等齐）。
# 包装：编译经 run-with-memory-cap.sh 6G、跑经 4G；线程经 capped.sh "${OPUS_THREADS:-7}"；全部 nice -n 19。
# 包装脚本目录取 SINGLEFS_TOOL，没给就取 $SINGLEFS_REPO/research/scripts（冻结副本里没有 research/，要给 SINGLEFS_TOOL）。
set -uo pipefail
HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
die() { echo "rerun.sh: $*" >&2; exit 2; }

BATCH_SEGMENTS="b1-1a b1-1b b1-2a b1-2b b1-3a b1-3b b1-4a b1-4b b2-1a b2-1b b2-2a b2-2b b2-3a b2-3b b2-4a b2-4b"
SELFTEST_SEGMENTS="st-29 st-24 st-18 st-death"
SIMPLE_SEGMENTS="n5 n5-fixa batch-test fixa-legal n4 n6 n7 cliff"

# 摘要：各段日志里的 RESULT 行（从 RESULT 起）、`test … ok` 行、`test result:` 行（去掉耗时），加各段退出码；每行前面带段名。
summarize() {
    local W=$1 f seg
    for f in "$W"/logs/*.log; do
        [ -e "$f" ] || continue
        seg=$(basename "$f" .log)
        grep -aE 'RESULT |^test .* \.\.\. ok$|^test result:' "$f" \
            | awk -v seg="$seg" '{ i = index($0, "RESULT "); line = i ? substr($0, i) : $0; sub(/; finished in [0-9.]+s$/, "", line); print seg "\t" line }'
    done
    for f in "$W"/logs/*.rc; do
        [ -e "$f" ] || continue
        printf '%s\tRC %s\n' "$(basename "$f" .rc)" "$(cat "$f")"
    done
}

if [ "${1:-}" = compare ]; then
    W=${2:?compare 要工作目录}
    [ -d "$W/logs" ] || die "$W/logs 不存在"
    summarize "$W" | LC_ALL=C sort > "$W/summary.txt"
    if diff <(LC_ALL=C sort "$HERE/expected-summary.txt") "$W/summary.txt" > "$W/summary.diff"; then
        echo "compare: 一致（$(wc -l < "$W/summary.txt") 行）"
        exit 0
    fi
    echo "compare: 不一致，差异 $(grep -c '^[<>]' "$W/summary.diff") 行，见 $W/summary.diff"
    exit 1
fi

W=${1:?要工作目录}
shift
[ $# -ge 1 ] || die "要段名或 all"
W=$(mkdir -p "$W" && cd "$W" && pwd)
THREADS=${OPUS_THREADS:-7}
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-m3-batch23-code-r1 攻方模型复跑，在自己的仓副本里}"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="${RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON:-派发提示写死编译 6G、跑 4G}"

tool_dir() {
    local dir=${SINGLEFS_TOOL:-${SINGLEFS_REPO:-}/research/scripts}
    [ -f "$dir/run-with-memory-cap.sh" ] && [ -f "$dir/capped.sh" ] || die "包装脚本不在 $dir（给 SINGLEFS_TOOL）"
    echo "$dir"
}

setup() {
    [ -n "${SINGLEFS_REPO:-}" ] || die "SINGLEFS_REPO 没给"
    [ -z "$(ls -A "$W")" ] || die "setup 要空的工作目录：$W 不空"
    mkdir -p "$W/logs"
    local t
    for t in tree tree-fixa; do
        mkdir -p "$W/$t"
        rsync -a --exclude target --exclude .git "$SINGLEFS_REPO/crates" "$SINGLEFS_REPO/Cargo.toml" "$SINGLEFS_REPO/Cargo.lock" "$W/$t/" || die "拷仓失败"
        python3 "$HERE/apply_capture_hook.py" "$W/$t/crates/singlefs-checker/src/walk.rs" || die "钩子没加上"
        cp "$HERE/opus_b23_attack.rs" "$W/$t/crates/singlefs-harness/tests/opus_b23_attack.rs"
    done
    python3 "$HERE/apply_fix_a.py" "$W/tree-fixa/crates/singlefs-checker/src/walk.rs" || die "改法 A 没加上"
    {
        printf 'RESULT setup 被判 walk.rs sha256=%s\n' "$(sha256sum < "$SINGLEFS_REPO/crates/singlefs-checker/src/walk.rs" | cut -d' ' -f1)"
        printf 'RESULT setup 被判 checker_known_bad_images.rs sha256=%s\n' "$(sha256sum < "$SINGLEFS_REPO/crates/singlefs-harness/tests/checker_known_bad_images.rs" | cut -d' ' -f1)"
    } > "$W/logs/setup.log"
    echo 0 > "$W/logs/setup.rc"
    echo "段 setup rc=0"
}

build() {
    local tool t tg rc
    tool=$(tool_dir)
    for t in tree tree-fixa; do
        tg=target; [ "$t" = tree-fixa ] && tg=target-fixa
        (cd "$W/$t" && CARGO_TARGET_DIR="$W/$tg" nice -n 19 bash "$tool/run-with-memory-cap.sh" 6G bash "$tool/capped.sh" "$THREADS" \
            cargo build --release --offline -p singlefs-harness --test opus_b23_attack --test checker_known_bad_images) > "$W/logs/build-$t.build" 2>&1
        rc=$?
        echo "$rc" > "$W/logs/build-$t.rc"
        echo "段 build（$t）rc=$rc"
    done
}

# run_test 段名 树 测试目标 过滤串 是否--nocapture(1/0) [环境变量赋值…]
run_test() {
    local seg=$1 t=$2 target=$3 filter=$4 nocapture=$5 tool tg rc started
    shift 5
    tool=$(tool_dir)
    tg=target; [ "$t" = tree-fixa ] && tg=target-fixa
    local capture=(); [ "$nocapture" = 1 ] && capture=(--nocapture)
    started=$(date +%s)
    (cd "$W/$t" && env CARGO_TARGET_DIR="$W/$tg" "$@" nice -n 19 bash "$tool/run-with-memory-cap.sh" 4G bash "$tool/capped.sh" "$THREADS" \
        cargo test --release --offline -p singlefs-harness --test "$target" -- "$filter" "${capture[@]}" --test-threads 1) > "$W/logs/$seg.log" 2>&1
    rc=$?
    echo "$rc" > "$W/logs/$seg.rc"
    echo "段 $seg rc=$rc 挂钟 $(( $(date +%s) - started ))s"
}

campaign() { # 段名 树 knob prefix seed_from histories steps mutations [fault]
    run_test "$1" "$2" opus_b23_attack opus_b23_gpu_model 1 OPUS_KNOB="$3" OPUS_PREFIX="$4" OPUS_SEED_FROM="$5" OPUS_HISTORIES="$6" \
        OPUS_STEPS="$7" OPUS_MUTATIONS="$8" OPUS_GPU_MODEL_FAULT="${9:-}"
}

segment() {
    case "$1" in
        setup) setup ;;
        build) build ;;
        n5) run_test n5 tree opus_b23_attack n5_the_one_birth 1 ;;
        n5-fixa) run_test n5-fixa tree-fixa opus_b23_attack n5_the_one_birth 1 ;;
        batch-test)
            run_test batch-test-tree tree checker_known_bad_images an_abandoned_root_naming_another_object_birth 0
            run_test batch-test-tree-fixa tree-fixa checker_known_bad_images an_abandoned_root_naming_another_object_birth 0 ;;
        fixa-legal) campaign fixa-legal tree-fixa 0 reissue 300 8 10 0 ;;
        n4) run_test n4 tree opus_b23_attack n4_second_sample 1 ;;
        n6) run_test n6 tree opus_b23_attack n6_product_tombstone 1 ;;
        n7) run_test n7 tree opus_b23_attack n7_second_sample 1 ;;
        cliff) run_test cliff tree opus_b23_attack n6_gpu_tables 1 OPUS_CLIFF_TOTAL=2100 ;;
        st-29) campaign st-29 tree 1 none 0 6 8 6 no-lineage-filter-29 ;;
        st-24) campaign st-24 tree 1 none 0 6 8 6 queue-bound-off-24 ;;
        st-18) campaign st-18 tree 1 rich 0 6 8 6 no-whole-object-18 ;;
        st-death) campaign st-death tree 1 rich 0 6 8 6 death-bound-plus-one ;;
        # 第一批：每段 10 段历史 × 10 步 × 每步改坏 6 次，拆成前后两半；第二批：6 段 × 20 步 × 2 次，拆两半。
        b1-1a) campaign b1-1a tree 0 rich 0 5 10 6 ;;
        b1-1b) campaign b1-1b tree 0 rich 5 5 10 6 ;;
        b1-2a) campaign b1-2a tree 1 rich 100 5 10 6 ;;
        b1-2b) campaign b1-2b tree 1 rich 105 5 10 6 ;;
        b1-3a) campaign b1-3a tree 2 rich 200 5 10 6 ;;
        b1-3b) campaign b1-3b tree 2 rich 205 5 10 6 ;;
        b1-4a) campaign b1-4a tree 0 reissue 300 5 10 6 ;;
        b1-4b) campaign b1-4b tree 0 reissue 305 5 10 6 ;;
        b2-1a) campaign b2-1a tree 0 rich 1000 3 20 2 ;;
        b2-1b) campaign b2-1b tree 0 rich 1003 3 20 2 ;;
        b2-2a) campaign b2-2a tree 1 rich 1100 3 20 2 ;;
        b2-2b) campaign b2-2b tree 1 rich 1103 3 20 2 ;;
        b2-3a) campaign b2-3a tree 0 reissue 1200 3 20 2 ;;
        b2-3b) campaign b2-3b tree 0 reissue 1203 3 20 2 ;;
        b2-4a) campaign b2-4a tree 3 none 1300 3 20 2 ;;
        b2-4b) campaign b2-4b tree 3 none 1303 3 20 2 ;;
        *) die "不认得的段：$1" ;;
    esac
}

# 一组段并行：每段退出码由 run_test 写进 logs/<段>.rc；先删这一组的 .rc，起完单个 wait 等齐，再数 .rc 个数。
parallel_group() {
    local seg count=0 present=0
    for seg in "$@"; do rm -f "$W/logs/$seg.rc"; done
    for seg in "$@"; do
        segment "$seg" &
        count=$((count + 1))
    done
    wait
    for seg in "$@"; do [ -e "$W/logs/$seg.rc" ] && present=$((present + 1)); done
    [ "$present" = "$count" ] || die "并行组 $* 只收到 $present / $count 个退出码，整组作废"
}

if [ "$1" = all ]; then
    segment setup
    segment build
    for seg in $SIMPLE_SEGMENTS; do segment "$seg"; done
    parallel_group $SELFTEST_SEGMENTS
    set -- $BATCH_SEGMENTS
    while [ $# -gt 0 ]; do
        group=("$@"); group=("${group[@]:0:4}")
        parallel_group "${group[@]}"
        shift $(( $# < 4 ? $# : 4 ))
    done
    exit 0
fi
if [ "$1" = parallel ]; then
    shift
    parallel_group "$@"
    exit 0
fi
for seg in "$@"; do segment "$seg"; done
