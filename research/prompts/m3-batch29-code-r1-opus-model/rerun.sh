#!/usr/bin/env bash
# m3-batch29-code-r1 攻方模型复跑（串行：同一时刻只起一条 cargo 编译或测试；仓副本只建一份）。
# 用法：
#   SINGLEFS_REPO=<冻结副本或仓，含 crates/ 与 Cargo.*> SINGLEFS_TOOLS=<仓根> bash rerun.sh <空工作目录> <段|all>
#   bash rerun.sh compare <工作目录>   # 各段 summary 里的 RESULT 行排序之后与 expected-summary.txt（同样排序）逐行比；不读 SINGLEFS_REPO、SINGLEFS_TOOLS
# 变量：
#   SINGLEFS_REPO   被判的源码（冻结副本 ~/.local/share/singlefs/work/m3-batch29-code-r1/tree）。
#   SINGLEFS_TOOLS  包装脚本 scripts/hosts/run-with-memory-cap.sh、capped.sh 所在的仓根：先找 $SINGLEFS_REPO/scripts/，没有就取它。
#   OPUS_THREADS    线程上限（默认 31，经 capped.sh 设；派发时主 agent 给的上限）。
#   OPUS_STAGE_BUDGET  harness-build、harness-probe 两段一次调用最多起新件的挂钟秒数（默认 190；过了就不再起下一件、不停正在跑的，退 0 并打印 STOP budget；
#                   再调同一段从没做完的那一件接着做，打印 DONE 才算这一段做完）。all 会把这两段反复调到 DONE。
#   包装另读的变量（RUN_WITH_MEMORY_CAP_*）这里不设，取包装自己的默认值。
# 段（all 依次跑全部；除 harness-build、harness-probe 外每段单跑在四分钟以内）：
#   setup         tree = SINGLEFS_REPO 的 crates 与 Cargo.*；checker_known_bad_images.rs 末尾追加 opus_b29_kbi_append.rs，
#                 container_trees_grow_a_level_when_the_root_is_full.rs 末尾追加 opus_b29_ctg_append.rs；make_variants.py 造 walk 的两份变体
#                 （probe：只计数的探针，不改判定；fix：probe 之上加攻方改法 F-L0）；tree 里的 walk.rs 换成 probe 那份。crash/ 原型 crate 指向 tree。
#   kbi-new / ctg-new   编、跑 opus_b29 打头的用例（walk = probe），日志 logs/<段>.log，摘要 logs/<段>.summary。
#   harness-build / harness-probe   singlefs-harness 每个测试目标逐个编、逐个跑（walk = probe，设 OPUS_B29_PROBE_LOG），摘要 logs/harness.summary。
#   crash-build / crash-count   编原型、只数每段历史的崩溃状态数（closed_form_state_count），不枚举。
#   kbi-fix / ctg-fix   walk 换成 fix 那份，再编、跑 opus_b29 打头的用例；跑完换回 probe。
#   kbi-anylen / ctg-anylen   同上，walk 换成 anylen 那份（攻方对照臂 F-任一长）。
#   existing-fix / existing-anylen   现有 checker_known_bad_images、container_trees_grow_a_level_when_the_root_is_full、
#                 pool_checker_judges_the_deletion_state_of_every_walked_version 三个目标在 fix / anylen 那份 walk 下整份跑（改法会不会把现有用例弄红）。
#   l2            python3 l2_chain_model.py（读码模型，推的）。
# 内存：编译 6G、跑 4G，经 run-with-memory-cap.sh；线程经 capped.sh。编译目录在工作目录里（target、target-crash）。
set -uo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ "${1:-}" == compare ]]; then
    work="$(cd "${2:?要给工作目录}" && pwd)"
    cat "$work"/logs/*.summary | sed -E 's/ seconds=[0-9.]+//g' | LC_ALL=C sort >"$work/summary.txt"
    if diff <(LC_ALL=C sort "$here/expected-summary.txt") "$work/summary.txt" >"$work/compare.diff"; then
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
[[ -f "$tools/scripts/hosts/run-with-memory-cap.sh" ]] || tools="${SINGLEFS_TOOLS:?SINGLEFS_REPO 下没有 scripts，要设 SINGLEFS_TOOLS}"
cap="$tools/scripts/hosts/run-with-memory-cap.sh"
capped="$tools/scripts/hosts/capped.sh"
threads="${OPUS_THREADS:-31}"
budget="${OPUS_STAGE_BUDGET:-190}"
tree="$work/tree"
walk="$tree/crates/singlefs-checker/src/walk.rs"
run_build() { CARGO_TARGET_DIR="$work/target" nice -n 19 bash "$cap" 6G bash "$capped" "$threads" "$@"; }
run_test() { CARGO_TARGET_DIR="$work/target" nice -n 19 bash "$cap" 4G bash "$capped" "$threads" "$@"; }
summarize() { grep -ohE 'RESULT .*' "$work/logs/$1.log" | sed -E "s/^RESULT /RESULT stage=$1 /" >"$work/logs/$1.summary"; }

stage_setup() {
    rm -rf "$tree" "$work/variants" "$work/crash"
    mkdir -p "$tree" "$work/variants" "$work/crash"
    rsync -a --exclude target --exclude .git "$repo/crates" "$repo/Cargo.toml" "$repo/Cargo.lock" "$tree/"
    cat "$here/opus_b29_kbi_append.rs" >>"$tree/crates/singlefs-harness/tests/checker_known_bad_images.rs"
    cat "$here/opus_b29_ctg_append.rs" >>"$tree/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs"
    python3 "$here/make_variants.py" "$repo/crates/singlefs-checker/src/walk.rs" "$work/variants/walk-probe.rs" "$work/variants/walk-fix.rs" "$work/variants/walk-anylen.rs"
    cp "$work/variants/walk-probe.rs" "$walk"
    cp -r "$here/crash/src" "$work/crash/"
    sed "s#@REPO@#$tree#g" "$here/crash/Cargo.toml.in" >"$work/crash/Cargo.toml"
    cp "$tree/Cargo.lock" "$work/crash/Cargo.lock"
    sha256sum "$work/variants/walk-probe.rs" "$work/variants/walk-fix.rs" "$work/variants/walk-anylen.rs" | sed -E "s#$work/##" | sed -E 's/^/RESULT stage=setup sha256 /' >"$work/logs/setup.summary"
}

stage_opus_tests() { # $1 段名  $2 测试目标
    (cd "$tree" && run_build cargo test --offline -p singlefs-harness --test "$2" --no-run) >"$work/logs/$1.build.log" 2>&1 || { echo "编译失败：$1"; return 1; }
    (cd "$tree" && run_test cargo test --offline -p singlefs-harness --test "$2" -- opus_b29 --nocapture --test-threads=1) >"$work/logs/$1.log" 2>&1
    local rc=$?
    summarize "$1"
    echo "RESULT stage=$1 exit=$rc" >>"$work/logs/$1.summary"
}

harness_targets() { ls "$tree/crates/singlefs-harness/tests/" | sed -n 's/\.rs$//p' | LC_ALL=C sort; }

stage_harness_build() {
    local start; start=$(date +%s)
    touch "$work/logs/harness-build.progress"
    for target in $(harness_targets); do
        grep -qx "$target" "$work/logs/harness-build.progress" && continue
        (( $(date +%s) - start > budget )) && { echo "STOP budget"; return 0; }
        (cd "$tree" && run_build cargo test --offline -p singlefs-harness --test "$target" --no-run) >"$work/logs/harness-build-last.log" 2>&1 || { echo "编译失败：$target"; return 1; }
        echo "$target" >>"$work/logs/harness-build.progress"
    done
    echo "DONE harness-build"
}

stage_harness_probe() {
    local start; start=$(date +%s)
    mkdir -p "$work/logs/probe" "$work/logs/harness-out"
    touch "$work/logs/harness-probe.progress"
    for target in $(harness_targets); do
        grep -q "^$target " "$work/logs/harness-probe.progress" && continue
        (( $(date +%s) - start > budget )) && { echo "STOP budget"; return 0; }
        local bin
        bin=$(cd "$tree" && CARGO_TARGET_DIR="$work/target" cargo test --offline -p singlefs-harness --test "$target" --no-run --message-format=json 2>/dev/null | python3 -c 'import json,sys
for line in sys.stdin:
    m=json.loads(line)
    if m.get("reason")=="compiler-artifact" and m.get("executable") and m["target"]["kind"]==["test"]:
        print(m["executable"])')
        [[ -n "$bin" ]] || { echo "找不到测试二进制：$target"; return 1; }
        OPUS_B29_PROBE_LOG="$work/logs/probe/$target.log" run_test "$bin" >"$work/logs/harness-out/$target.log" 2>&1
        echo "$target rc=$?" >>"$work/logs/harness-probe.progress"
    done
    python3 "$here/harness_probe_summary.py" "$work/logs" >"$work/logs/harness.summary"
    echo "DONE harness-probe"
}

stage_crash_build() {
    (cd "$work/crash" && CARGO_TARGET_DIR="$work/target-crash" nice -n 19 bash "$cap" 6G bash "$capped" "$threads" cargo build --offline --release) >"$work/logs/crash-build.log" 2>&1 || { echo "编译失败：crash"; return 1; }
}

stage_crash_count() {
    OPUS_COUNT_ONLY=1 nice -n 19 bash "$cap" 4G bash "$capped" "$threads" "$work/target-crash/release/opus-b29-crash" >"$work/logs/crash-count.log" 2>&1
    summarize crash-count
}

stage_variant() { # $1 段名  $2 测试目标  $3 变体（fix / anylen）
    cp "$work/variants/walk-$3.rs" "$walk"
    stage_opus_tests "$1" "$2"
    cp "$work/variants/walk-probe.rs" "$walk"
}

stage_existing() { # $1 段名  $2 变体（fix / anylen）：现有三个测试目标在这一份 walk 下整份跑
    cp "$work/variants/walk-$2.rs" "$walk"
    (cd "$tree" && run_test cargo test --offline -p singlefs-harness --test checker_known_bad_images --test container_trees_grow_a_level_when_the_root_is_full --test pool_checker_judges_the_deletion_state_of_every_walked_version) >"$work/logs/$1.log" 2>&1
    local rc=$?
    cp "$work/variants/walk-probe.rs" "$walk"
    grep -E "^test result" "$work/logs/$1.log" | sed -E "s/; finished in [0-9.]+s//; s/^/RESULT stage=$1 /" >"$work/logs/$1.summary"
    echo "RESULT stage=$1 exit=$rc" >>"$work/logs/$1.summary"
}

stage_l2() {
    python3 "$here/l2_chain_model.py" >"$work/logs/l2.log" 2>&1
    summarize l2
}

run_stage() {
    case "$1" in
    setup) stage_setup ;;
    kbi-new) stage_opus_tests kbi-new checker_known_bad_images ;;
    ctg-new) stage_opus_tests ctg-new container_trees_grow_a_level_when_the_root_is_full ;;
    harness-build) stage_harness_build ;;
    harness-probe) stage_harness_probe ;;
    crash-build) stage_crash_build ;;
    crash-count) stage_crash_count ;;
    kbi-fix) stage_variant kbi-fix checker_known_bad_images fix ;;
    ctg-fix) stage_variant ctg-fix container_trees_grow_a_level_when_the_root_is_full fix ;;
    kbi-anylen) stage_variant kbi-anylen checker_known_bad_images anylen ;;
    ctg-anylen) stage_variant ctg-anylen container_trees_grow_a_level_when_the_root_is_full anylen ;;
    existing-fix) stage_existing existing-fix fix ;;
    existing-anylen) stage_existing existing-anylen anylen ;;
    l2) stage_l2 ;;
    *) echo "不认得的段：$1"; return 2 ;;
    esac
}

if [[ "$stage" == all ]]; then
    for one in setup kbi-new ctg-new; do run_stage "$one" || exit 1; done
    for one in harness-build harness-probe; do
        while true; do
            out=$(run_stage "$one")
            echo "$out"
            case "$out" in
            *DONE*) break ;;
            *"STOP budget"*) continue ;;
            *) exit 1 ;;
            esac
        done
    done
    for one in crash-build crash-count kbi-fix ctg-fix kbi-anylen ctg-anylen existing-fix existing-anylen l2; do run_stage "$one" || exit 1; done
    echo "ALL DONE"
else
    run_stage "$stage"
fi
