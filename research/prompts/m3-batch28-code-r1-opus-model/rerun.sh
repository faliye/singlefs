#!/usr/bin/env bash
# m3-batch28-code-r1 攻方模型复跑（串行：同一时刻只起一条 cargo 编译或测试；仓副本同时最多两份）。
#   SINGLEFS_REPO=<冻结副本或仓，含 crates/ 与 Cargo.*> SINGLEFS_TOOLS=<仓根> bash rerun.sh <空工作目录> <段|all>
#   bash rerun.sh compare <工作目录>     # 各段日志里的 RESULT 行去掉计时字段、排序之后与 expected-summary.txt（同样排序）比；不读 SINGLEFS_REPO、SINGLEFS_TOOLS
# 变量：
#   SINGLEFS_REPO   被判的源码（冻结副本 /tmp/claude-1000/m3-batch28-code-r1/tree 或仓根）。
#   SINGLEFS_TOOLS  包装脚本 run-with-memory-cap.sh、capped.sh 所在的仓根：先找 $SINGLEFS_REPO/research/scripts/，没有就取
#                   $SINGLEFS_TOOLS/research/scripts/（SINGLEFS_REPO 是冻结副本时必须给）。
#   OPUS_THREADS    线程上限（默认 10，经 capped.sh 设）。
#   包装另读的：RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON、RUN_WITH_MEMORY_CAP_PARALLEL_REASON、RUN_WITH_MEMORY_CAP_WAIT_SECONDS（没设时这里给默认值）。
# 段（all 依次跑全部；每段单跑在四分钟以内）：
#   setup        tree = SINGLEFS_REPO 的 crates 与 Cargo.*，checker_known_bad_images.rs 末尾追加 opus_b28_append.rs；walk.rs 原样存一份 walk-new.rs；
#                tree-probe = 同一份源码打 probe.diff（walk.rs 里加只计数的原子计数器，不改判定）；crash/ 原型 crate 指向 tree-probe。
#   kbi-new / kbi-old / kbi-fix   把 tree 的 walk.rs 换成这一份（new 原样；old 反打 batch28-walk.patch；fix 打 fix-FW.diff），
#                编 checker_known_bad_images（编译目录 target-kbi 共用、增量），二进制拷到 bin/kbi-<代号>，跑 opus_b28_ 打头的用例，日志 logs/hv-<代号>.log。
#   hvsum        analyze.py 把三份日志按形对齐，写 logs/hvsum.log。
#   crash-build  编原型 crate（编译目录 target-crash）。
#   crash-<名>   跑原型：名见 crash_runs（每一档的环境变量写在那里），日志 logs/crash-<名>.log。
# 内存：编译 6G、跑 4G，经 run-with-memory-cap.sh；线程经 capped.sh。
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ "${1:-}" == compare ]]; then
    work="$(cd "${2:?要给工作目录}" && pwd)"
    grep -ohE 'RESULT .*' "$work"/logs/*.log | sed -E 's/ seconds=[0-9.]+//; s/ enumerate_seconds=[0-9.]+//' | LC_ALL=C sort >"$work/summary.txt"
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
mkdir -p "$work/logs" "$work/bin"
work="$(cd "$work" && pwd)"
repo="${SINGLEFS_REPO:?要设 SINGLEFS_REPO}"
tools="$repo"
[[ -f "$tools/research/scripts/run-with-memory-cap.sh" ]] || tools="${SINGLEFS_TOOLS:?SINGLEFS_REPO 下没有 research/scripts，要设 SINGLEFS_TOOLS}"
cap="$tools/research/scripts/run-with-memory-cap.sh"
capped="$tools/research/scripts/capped.sh"
threads="${OPUS_THREADS:-10}"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="${RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON:-派发提示写死编译 6G、跑 4G}"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-三方攻方腿模型复跑，在自己的工作目录里串行编、跑}"
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS="${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-200}"
walk_rel=crates/singlefs-checker/src/walk.rs

# 原型的各档：名、环境变量（OPUS_MODE、OPUS_STARTS、OPUS_SEED_FROM、OPUS_HISTORIES、OPUS_STEPS、OPUS_STATE_BUDGET）
#   count-*   只按 closed_form_state_count 数状态、不枚举（几秒）；final-*  不枚举崩溃状态，每段历史每做完一步跑一次池级 checker（每段三分钟上下）；
#   s0m / sdm 层 0 全量枚举（每一段都展开）一段「再挂」历史，393238 / 327696 个状态，按实测约 420 状态 / 秒估 16 / 13 分钟，
#             超过前台四分钟，交主 agent 跑（不在 all 里）。
crash_runs() {
    case "$1" in
    count-it) echo "OPUS_COUNT_ONLY=1 OPUS_MODE=it" ;;
    count-legal) echo "OPUS_COUNT_ONLY=1 OPUS_MODE=legal OPUS_STARTS=S0,SD,SL OPUS_SEED_FROM=1 OPUS_HISTORIES=8 OPUS_STEPS=1" ;;
    final-s0) echo "OPUS_MODE=final OPUS_STARTS=S0 OPUS_SEED_FROM=1 OPUS_HISTORIES=150 OPUS_STEPS=8" ;;
    final-sd1) echo "OPUS_MODE=final OPUS_STARTS=SD OPUS_SEED_FROM=1 OPUS_HISTORIES=60 OPUS_STEPS=8" ;;
    final-sd2) echo "OPUS_MODE=final OPUS_STARTS=SD OPUS_SEED_FROM=61 OPUS_HISTORIES=60 OPUS_STEPS=8" ;;
    final-sl) echo "OPUS_MODE=final OPUS_STARTS=SL OPUS_SEED_FROM=1 OPUS_HISTORIES=80 OPUS_STEPS=8" ;;
    s0m) echo "OPUS_MODE=legal OPUS_STARTS=S0 OPUS_SEED_FROM=4 OPUS_HISTORIES=1 OPUS_STEPS=1 OPUS_STATE_BUDGET=1000000" ;;
    sdm) echo "OPUS_MODE=legal OPUS_STARTS=SD OPUS_SEED_FROM=4 OPUS_HISTORIES=1 OPUS_STEPS=1 OPUS_STATE_BUDGET=1000000" ;;
    *) return 2 ;;
    esac
}
crash_names="count-it count-legal final-s0 final-sd1 final-sd2 final-sl"

kbi() { # 代号
    local code="$1"
    cp "$work/walk-new.rs" "$work/tree/$walk_rel"
    case "$code" in
    new) ;;
    old) patch -R -p1 -d "$work/tree" <"$here/batch28-walk.patch" >"$work/logs/patch-old.txt" 2>&1 ;;
    fix) patch -p1 -d "$work/tree" <"$here/fix-FW.diff" >"$work/logs/patch-fix.txt" 2>&1 ;;
    esac
    (cd "$work/tree" && CARGO_TARGET_DIR="$work/target-kbi" nice -n 19 bash "$cap" 6G bash "$capped" "$threads" \
        cargo test --release --offline -p singlefs-harness --test checker_known_bad_images --no-run) >"$work/logs/build-$code.txt" 2>&1
    cp "$(grep -oE '\(/[^)]*/checker_known_bad_images-[0-9a-f]+\)' "$work/logs/build-$code.txt" | tr -d '()' | tail -1)" "$work/bin/kbi-$code"
    env OPUS_CODE="$code" nice -n 19 bash "$cap" 4G bash "$capped" "$threads" "$work/bin/kbi-$code" opus_b28_ \
        --nocapture --test-threads 2 >"$work/logs/hv-$code.log" 2>&1
}

run_stage() {
    case "$1" in
    setup)
        mkdir -p "$work/tree"
        rsync -a --exclude target --exclude .git "$repo/crates" "$repo/Cargo.toml" "$repo/Cargo.lock" "$work/tree/"
        cp "$work/tree/$walk_rel" "$work/walk-new.rs"
        rsync -a "$work/tree/" "$work/tree-probe/"
        patch -p1 -d "$work/tree-probe" <"$here/probe.diff" >"$work/logs/patch-probe.txt" 2>&1
        cat "$here/opus_b28_append.rs" >>"$work/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs"
        mkdir -p "$work/crash/src"
        sed "s#@REPO@#$work/tree-probe#g" "$here/crash/Cargo.toml.in" >"$work/crash/Cargo.toml"
        cp "$here/crash/src/main.rs" "$work/crash/src/main.rs"
        cp "$work/tree-probe/Cargo.lock" "$work/crash/Cargo.lock"
        ln -sfn "$work/tree-probe/crates/singlefs-harness/tests" "$work/crash/harness_tests"
        ;;
    kbi-new | kbi-old | kbi-fix) kbi "${1#kbi-}" ;;
    hvsum) python3 "$here/analyze.py" "$work" >"$work/logs/hvsum-run.txt" ;;
    crash-build)
        (cd "$work/crash" && CARGO_TARGET_DIR="$work/target-crash" nice -n 19 bash "$cap" 6G bash "$capped" "$threads" \
            cargo build --release --offline) >"$work/logs/crash-build.txt" 2>&1
        ;;
    crash-*)
        local name="${1#crash-}" settings
        settings="$(crash_runs "$name")" || { echo "不认得的原型档：$name" >&2; return 2; }
        # shellcheck disable=SC2086
        env $settings nice -n 19 bash "$cap" 4G bash "$capped" "$threads" "$work/target-crash/release/opus-b28-crash" \
            >"$work/logs/crash-$name.log" 2>&1
        ;;
    *) echo "不认得的段：$1" >&2; return 2 ;;
    esac
}

if [[ "$stage" == all ]]; then
    for s in setup kbi-new kbi-old kbi-fix hvsum crash-build $(for n in $crash_names; do echo "crash-$n"; done); do
        echo "== $s"
        run_stage "$s"
    done
else
    run_stage "$stage"
fi
