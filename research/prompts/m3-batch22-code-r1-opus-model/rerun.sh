#!/usr/bin/env bash
# m3-batch22-code-r1 攻方模型复跑。
#   SINGLEFS_REPO=<仓或冻结副本，含 crates/ 与 Cargo.*> bash rerun.sh <空工作目录> <段|all>
#   bash rerun.sh compare <工作目录>        # 把各段日志里的 RESULT 行（去掉秒数）排序，与 expected-summary.txt 比
# 段：setup build n4n7 lineage-n6 scenarios sweep-k0[-b] sweep-k0-reissue[-b] sweep-k1[-b] sweep-k0-long[-b] sweep-k1-long[-b] sweep-k2-reissue[-b]
#     crash-build crash-count crash-boundaries summary
# 编译与跑都经 run-with-memory-cap.sh（编译 6G、跑 4G），线程经 capped.sh（OPUS_THREADS，默认 10）。
# 包装脚本取 $SINGLEFS_REPO/scripts/，没有就取 $SINGLEFS_TOOLS/scripts/（SINGLEFS_REPO 是只含 crates 的冻结副本时给它）。
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ "${1:-}" == compare ]]; then
    work="$(cd "$2" && pwd)"
    bash "$0" "$work" summary >/dev/null
    if diff <(sort "$here/expected-summary.txt") <(sort "$work/summary.txt") >"$work/compare.diff"; then
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
repo="${SINGLEFS_REPO:?要设 SINGLEFS_REPO}"
tools="$repo"
[[ -x "$tools/scripts/run-with-memory-cap.sh" || -f "$tools/scripts/run-with-memory-cap.sh" ]] || tools="${SINGLEFS_TOOLS:?SINGLEFS_REPO 下没有 scripts，要设 SINGLEFS_TOOLS}"
cap="$tools/scripts/run-with-memory-cap.sh"
capped="$tools/scripts/capped.sh"
threads="${OPUS_THREADS:-10}"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="${RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON:-派发提示写死编译 6G、跑 4G}"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-三方攻方腿模型复跑，在自己的工作目录里各跑各的}"
export CARGO_TARGET_DIR="$work/target"
tree="$work/tree"

harness_test() { # 段名、过滤串，其余是环境变量赋值
    local name="$1" filter="$2"; shift 2
    (cd "$tree" && env "$@" nice -n 19 bash "$cap" 4G bash "$capped" "$threads" \
        cargo test --release --offline -p singlefs-harness --test opus_b22_attack -- "$filter" --nocapture --test-threads 1) \
        >"$work/logs/$name.log" 2>&1
}

run_stage() {
    case "$1" in
    setup)
        mkdir -p "$tree"
        rsync -a --exclude target --exclude .git "$repo/crates" "$repo/Cargo.toml" "$repo/Cargo.lock" "$tree/"
        cp "$here/opus_b22_attack.rs" "$tree/crates/singlefs-harness/tests/opus_b22_attack.rs"
        mkdir -p "$work/crash/src"
        sed "s#@REPO@#$tree#g" "$here/crash/Cargo.toml.in" >"$work/crash/Cargo.toml"
        cp "$here/crash/src/main.rs" "$work/crash/src/main.rs"
        cp "$tree/Cargo.lock" "$work/crash/Cargo.lock"
        ln -sfn "$tree/crates/singlefs-harness/tests" "$work/crash/harness_tests"
        ;;
    build)
        (cd "$tree" && nice -n 19 bash "$cap" 6G bash "$capped" "$threads" \
            cargo build --release --offline -p singlefs-harness --test opus_b22_attack) >"$work/logs/build.log" 2>&1 ;;
    n4n7) harness_test n4n7 "n4_" OPUS_UNUSED=1 && harness_test n7 "n7_" OPUS_UNUSED=1 ;;
    lineage-n6) harness_test lineage "n5_a_lineage" OPUS_UNUSED=1 && harness_test n6 "n6_" OPUS_UNUSED=1 ;;
    scenarios) harness_test scenarios "n5_n9_scenarios" OPUS_UNUSED=1 ;;
    # 每批 50 段（长历史 18 段）：机器满载时一批也在几分钟里跑完。
    sweep-k0) harness_test sweep-k0 n5_n9_random OPUS_KNOB=0 OPUS_SEED_FROM=0 OPUS_HISTORIES=50 OPUS_STEPS=10 ;;
    sweep-k0-b) harness_test sweep-k0-b n5_n9_random OPUS_KNOB=0 OPUS_SEED_FROM=50 OPUS_HISTORIES=50 OPUS_STEPS=10 ;;
    sweep-k0-reissue) harness_test sweep-k0-reissue n5_n9_random OPUS_PREFIX=reissue OPUS_KNOB=0 OPUS_SEED_FROM=0 OPUS_HISTORIES=50 OPUS_STEPS=10 ;;
    sweep-k0-reissue-b) harness_test sweep-k0-reissue-b n5_n9_random OPUS_PREFIX=reissue OPUS_KNOB=0 OPUS_SEED_FROM=50 OPUS_HISTORIES=50 OPUS_STEPS=10 ;;
    sweep-k1) harness_test sweep-k1 n5_n9_random OPUS_KNOB=1 OPUS_SEED_FROM=1000 OPUS_HISTORIES=50 OPUS_STEPS=10 ;;
    sweep-k1-b) harness_test sweep-k1-b n5_n9_random OPUS_KNOB=1 OPUS_SEED_FROM=1050 OPUS_HISTORIES=50 OPUS_STEPS=10 ;;
    sweep-k0-long) harness_test sweep-k0-long n5_n9_random OPUS_KNOB=0 OPUS_SEED_FROM=2000 OPUS_HISTORIES=18 OPUS_STEPS=25 ;;
    sweep-k0-long-b) harness_test sweep-k0-long-b n5_n9_random OPUS_KNOB=0 OPUS_SEED_FROM=2018 OPUS_HISTORIES=18 OPUS_STEPS=25 ;;
    sweep-k1-long) harness_test sweep-k1-long n5_n9_random OPUS_KNOB=1 OPUS_SEED_FROM=2000 OPUS_HISTORIES=18 OPUS_STEPS=25 ;;
    sweep-k1-long-b) harness_test sweep-k1-long-b n5_n9_random OPUS_KNOB=1 OPUS_SEED_FROM=2018 OPUS_HISTORIES=18 OPUS_STEPS=25 ;;
    sweep-k2-reissue) harness_test sweep-k2-reissue n5_n9_random OPUS_PREFIX=reissue OPUS_KNOB=2 OPUS_SEED_FROM=3000 OPUS_HISTORIES=50 OPUS_STEPS=10 ;;
    sweep-k2-reissue-b) harness_test sweep-k2-reissue-b n5_n9_random OPUS_PREFIX=reissue OPUS_KNOB=2 OPUS_SEED_FROM=3050 OPUS_HISTORIES=50 OPUS_STEPS=10 ;;
    crash-build)
        (cd "$work/crash" && nice -n 19 bash "$cap" 6G bash "$capped" "$threads" cargo build --release --offline) \
            >"$work/logs/crash-build.log" 2>&1 ;;
    crash-count)
        (cd "$work/crash" && OPUS_COUNT_ONLY=1 nice -n 19 bash "$cap" 4G bash "$capped" "$threads" \
            "$work/target/release/opus-b22-crash") >"$work/logs/crash-count.log" 2>&1 ;;
    crash-boundaries)
        (cd "$work/crash" && OPUS_DOMAIN=boundaries:2 nice -n 19 bash "$cap" 4G bash "$capped" "$threads" \
            "$work/target/release/opus-b22-crash") >"$work/logs/crash-boundaries.log" 2>&1 ;;
    summary)
        # RESULT 行原样（测试名与它同一行时只取 RESULT 起的那一段），去掉随机器变的秒数；排序在 compare 里做。
        grep -ohE 'RESULT .*' "$work"/logs/{n4n7,n7,lineage,n6,scenarios,sweep-k0,sweep-k0-b,sweep-k0-reissue,sweep-k0-reissue-b,sweep-k1,sweep-k1-b,sweep-k0-long,sweep-k0-long-b,sweep-k1-long,sweep-k1-long-b,sweep-k2-reissue,sweep-k2-reissue-b,crash-count,crash-boundaries}.log \
            | sed -E 's/ (enumerate_)?seconds=[0-9.]+//g' >"$work/summary.txt"
        echo "summary: $(wc -l <"$work/summary.txt") 行 -> $work/summary.txt" ;;
    *) echo "不认得的段：$1" >&2; return 2 ;;
    esac
}

if [[ "$stage" == all ]]; then
    for s in setup build n4n7 lineage-n6 scenarios sweep-k0 sweep-k0-b sweep-k0-reissue sweep-k0-reissue-b sweep-k1 sweep-k1-b sweep-k0-long sweep-k0-long-b sweep-k1-long sweep-k1-long-b sweep-k2-reissue sweep-k2-reissue-b crash-build crash-count crash-boundaries summary; do
        echo "== $s"
        run_stage "$s"
    done
else
    run_stage "$stage"
fi
