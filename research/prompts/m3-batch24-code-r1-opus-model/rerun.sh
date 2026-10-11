#!/usr/bin/env bash
# m3-batch24-code-r1 攻方模型复跑。
#   SINGLEFS_REPO=<仓或冻结副本，含 crates/ 与 Cargo.*> bash rerun.sh <空工作目录> <段|all>
#   bash rerun.sh compare <工作目录>        # 各段日志里的 RESULT 行（去掉秒数）排序，与 expected-summary.txt 比；不依赖 SINGLEFS_REPO
# 段：setup build n1-inode n1-T1 n1-T2 n1-T3 n1-T3ring-a n1-T3ring-b n1-T3ring-c n1-L1 n1-L2 n1-L3 n1-stale n3-turn n3-b1 n3-b2
#     fix-setup fix-build fix-stale fix-matrix fix-kbi crash-build crash-count summary
# 编译与跑都经 run-with-memory-cap.sh（编译 6G、跑 4G），线程经 capped.sh（OPUS_THREADS，默认 10）。每段前台四分钟以内跑完。
# 包装脚本取 $SINGLEFS_REPO/scripts/，没有就取 $SINGLEFS_TOOLS/scripts/（SINGLEFS_REPO 是只含 crates 的冻结副本时给它）。
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ "${1:-}" == compare ]]; then
    work="$(cd "$2" && pwd)"
    bash "$0" "$work" summary >/dev/null
    if diff <(LC_ALL=C sort "$here/expected-summary.txt") <(LC_ALL=C sort "$work/summary.txt") >"$work/compare.diff"; then
        echo "SUMMARY_MATCHES $(wc -l <"$work/summary.txt") 行"
    else
        echo "SUMMARY_DIFFERS（差异在 $work/compare.diff，$(wc -l <"$work/compare.diff") 行）"
        exit 1
    fi
    exit 0
fi
work="${1:?要给工作目录}"
stage="${2:?要给段名或 all}"
mkdir -p "$work/logs"
work="$(cd "$work" && pwd)"
if [[ "$stage" == summary ]]; then
    # RESULT 行原样（测试名与它同一行时只取 RESULT 起的那一段），去掉随机器变的秒数；排序在 compare 里做。
    logs=()
    for name in n1-inode n1-T1 n1-T2 n1-T3 n1-T3ring-a n1-T3ring-b n1-T3ring-c n1-L1 n1-L2 n1-L3 n1-stale n3-turn \
        b1-1 b1-2 b1-4 b2-1 b2-2 b2-3 fix-stale fix-T1 fix-T2 fix-T3 fix-L2 fix-L3 fix-inode fix-kbi crash-count; do
        logs+=("$work/logs/$name.log")
    done
    grep -ohE 'RESULT .*|test result: .*' "${logs[@]}" | sed -E 's/ (enumerate_|judge_)?seconds=[0-9.]+//g; s/; finished in [0-9.]+s//' >"$work/summary.txt"
    echo "summary: $(wc -l <"$work/summary.txt") 行 -> $work/summary.txt"
    exit 0
fi
repo="${SINGLEFS_REPO:?要设 SINGLEFS_REPO}"
tools="$repo"
[[ -f "$tools/scripts/hosts/run-with-memory-cap.sh" ]] || tools="${SINGLEFS_TOOLS:?SINGLEFS_REPO 下没有 scripts，要设 SINGLEFS_TOOLS}"
cap="$tools/scripts/hosts/run-with-memory-cap.sh"
capped="$tools/scripts/hosts/capped.sh"
threads="${OPUS_THREADS:-10}"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="${RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON:-派发提示写死编译 6G、跑 4G}"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-三方攻方腿模型复跑，在自己的工作目录里各跑各的}"

build_test() { # 源码树、编译目录、日志名：编 opus_b24_attack 与 checker_known_bad_images，把两份测试二进制的路径记下
    local tree="$1" target="$2" log="$3"
    (cd "$tree" && CARGO_TARGET_DIR="$target" nice -n 19 bash "$cap" 6G bash "$capped" "$threads" \
        cargo test --release --offline -p singlefs-harness --test opus_b24_attack --test checker_known_bad_images --no-run) \
        >"$work/logs/$log.log" 2>&1
    grep -oE '\(/[^)]*/opus_b24_attack-[0-9a-f]+\)' "$work/logs/$log.log" | tr -d '()' | tail -1 >"$target/opus-bin.txt"
    grep -oE '\(/[^)]*/checker_known_bad_images-[0-9a-f]+\)' "$work/logs/$log.log" | tr -d '()' | tail -1 >"$target/kbi-bin.txt"
}

run_test() { # 编译目录、日志名、过滤串，其余是环境变量赋值；直接跑编好的测试二进制
    local target="$1" log="$2" filter="$3"; shift 3
    env "$@" nice -n 19 bash "$cap" 4G bash "$capped" "$threads" "$(cat "$target/opus-bin.txt")" "$filter" --nocapture --test-threads 1 \
        >"$work/logs/$log.log" 2>&1
}

run_batch() { # 批号，之后每件「日志名|过滤串|变量…」：并行起，每件退出码写 logs/<日志名>.rc，不带参数的 wait 等齐，数过再按次序读
    local batch="$1"; shift
    local names=()
    for item in "$@"; do names+=("${item%%|*}"); done
    for name in "${names[@]}"; do rm -f "$work/logs/$name.rc"; done
    for item in "$@"; do
        local name="${item%%|*}" rest="${item#*|}"
        local filter="${rest%%|*}" vars="${rest#*|}"
        # shellcheck disable=SC2086
        { set +e; run_test "$work/target" "$name" "$filter" $vars; echo "$?" >"$work/logs/$name.rc"; } &
    done
    wait
    local count=0
    for name in "${names[@]}"; do [[ -f "$work/logs/$name.rc" ]] && count=$((count + 1)); done
    if [[ "$count" -ne "${#names[@]}" ]]; then
        echo "批 $batch：退出码文件 $count 个，派出去 ${#names[@]} 件，整批作废" >&2
        return 1
    fi
    for name in "${names[@]}"; do echo "批 $batch $name 退出码 $(cat "$work/logs/$name.rc")"; done
}

run_stage() {
    case "$1" in
    setup)
        mkdir -p "$work/tree"
        rsync -a --exclude target --exclude .git "$repo/crates" "$repo/Cargo.toml" "$repo/Cargo.lock" "$work/tree/"
        cp "$here/opus_b24_attack.rs" "$work/tree/crates/singlefs-harness/tests/opus_b24_attack.rs"
        mkdir -p "$work/crash/src"
        sed "s#@REPO@#$work/tree#g" "$here/crash/Cargo.toml.in" >"$work/crash/Cargo.toml"
        cp "$here/crash/src/main.rs" "$work/crash/src/main.rs"
        cp "$work/tree/Cargo.lock" "$work/crash/Cargo.lock"
        ln -sfn "$work/tree/crates/singlefs-harness/tests" "$work/crash/harness_tests"
        ;;
    build) build_test "$work/tree" "$work/target" build ;;
    n1-inode) run_test "$work/target" n1-inode n1_inode OPUS_UNUSED=1 ;;
    n1-T1 | n1-T2 | n1-T3 | n1-L1 | n1-L2 | n1-L3) run_test "$work/target" "$1" n1_type_segment "OPUS_POOL=${1#n1-}" ;;
    n1-T3ring-a) run_test "$work/target" "$1" n1_type_segment OPUS_POOL=T3ring OPUS_SKIP=0 OPUS_TAKE=17 ;;
    n1-T3ring-b) run_test "$work/target" "$1" n1_type_segment OPUS_POOL=T3ring OPUS_SKIP=17 OPUS_TAKE=17 ;;
    n1-T3ring-c) run_test "$work/target" "$1" n1_type_segment OPUS_POOL=T3ring OPUS_SKIP=34 OPUS_TAKE=16 ;;
    n1-stale) run_test "$work/target" n1-stale n1_entry_type_mismatch_with_one_stale OPUS_UNUSED=1 ;;
    n3-turn) run_test "$work/target" n3-turn n3_after_the_queued OPUS_UNUSED=1 ;;
    n3-b1) run_batch 1 \
        "b1-1|n3_reasons|OPUS_KNOB=0 OPUS_SEED_FROM=10000 OPUS_HISTORIES=40 OPUS_STEPS=10" \
        "b1-2|n3_reasons|OPUS_KNOB=1 OPUS_SEED_FROM=11000 OPUS_HISTORIES=40 OPUS_STEPS=10" \
        "b1-4|n3_reasons|OPUS_KNOB=0 OPUS_PREFIX=reissue OPUS_SEED_FROM=10100 OPUS_HISTORIES=40 OPUS_STEPS=10" ;;
    n3-b2) run_batch 2 \
        "b2-1|n3_reasons|OPUS_KNOB=2 OPUS_PREFIX=reissue OPUS_SEED_FROM=13000 OPUS_HISTORIES=40 OPUS_STEPS=10" \
        "b2-2|n3_reasons|OPUS_KNOB=0 OPUS_SEED_FROM=12000 OPUS_HISTORIES=14 OPUS_STEPS=25" \
        "b2-3|n3_reasons|OPUS_KNOB=1 OPUS_SEED_FROM=12100 OPUS_HISTORIES=14 OPUS_STEPS=25" ;;
    fix-setup)
        mkdir -p "$work/tree-fix"
        rsync -a "$work/tree/" "$work/tree-fix/"
        patch -d "$work/tree-fix" -p1 <"$here/fix-F1.diff" >"$work/logs/fix-setup.log" 2>&1 ;;
    fix-build) build_test "$work/tree-fix" "$work/target-fix" fix-build ;;
    fix-stale) run_test "$work/target-fix" fix-stale n1_entry_type_mismatch_with_one_stale OPUS_UNUSED=1 ;;
    fix-matrix)
        for pool in T1 T2 T3 L2 L3; do run_test "$work/target-fix" "fix-$pool" n1_type_segment "OPUS_POOL=$pool"; done
        run_test "$work/target-fix" fix-inode n1_inode OPUS_UNUSED=1 ;;
    fix-kbi)
        nice -n 19 bash "$cap" 4G bash "$capped" "$threads" "$(cat "$work/target-fix/kbi-bin.txt")" --test-threads "$threads" \
            >"$work/logs/fix-kbi.log" 2>&1 ;;
    crash-build)
        (cd "$work/crash" && CARGO_TARGET_DIR="$work/target" nice -n 19 bash "$cap" 6G bash "$capped" "$threads" cargo build --release --offline) \
            >"$work/logs/crash-build.log" 2>&1 ;;
    crash-count)
        OPUS_COUNT_ONLY=1 nice -n 19 bash "$cap" 4G bash "$capped" "$threads" "$work/target/release/opus-b24-crash" \
            >"$work/logs/crash-count.log" 2>&1 ;;
    *) echo "不认得的段：$1" >&2; return 2 ;;
    esac
}

if [[ "$stage" == all ]]; then
    for s in setup build n1-inode n1-T1 n1-T2 n1-T3 n1-T3ring-a n1-T3ring-b n1-T3ring-c n1-L1 n1-L2 n1-L3 n1-stale n3-turn n3-b1 n3-b2 \
        fix-setup fix-build fix-stale fix-matrix fix-kbi crash-build crash-count; do
        echo "== $s"
        run_stage "$s"
    done
    bash "$0" "$work" summary
else
    run_stage "$stage"
fi
