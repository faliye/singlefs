#!/usr/bin/env bash
# m3-batch26-code-r1 攻方模型复跑。
#   SINGLEFS_REPO=<仓或冻结副本，含 crates/ 与 Cargo.*> SINGLEFS_TOOLS=<仓根> bash rerun.sh <空工作目录> <段|all>
#   bash rerun.sh compare <工作目录>        # 各段日志里的 RESULT 行排序，与 expected-summary.txt 比；不依赖 SINGLEFS_REPO
# 变量：SINGLEFS_REPO 是被判的源码（冻结副本只有 crates/ 与 Cargo.*）；包装脚本 run-with-memory-cap.sh、capped.sh 先找 $SINGLEFS_REPO/research/scripts/，
#   没有就取 $SINGLEFS_TOOLS/research/scripts/（SINGLEFS_REPO 是冻结副本时必须给 SINGLEFS_TOOLS）。OPUS_THREADS 线程上限（默认 10）。
# 段：setup build-new build-old build-fix h-new h-old h-fix u-new u-old u-fix v p a2 k b1 b2 kbi summary
#   tree-new = SINGLEFS_REPO 原样；tree-old = 反打 batch26-walk.patch（第 26 批 walk.rs 那两处改动之前）；tree-fix = tree-new 打 fix-FB-FV.diff（攻方改法，零轮）。
# 编译与跑都经 run-with-memory-cap.sh（编译 6G、跑 4G），线程经 capped.sh；b1、b2 四件并行、每件 capped 2。每段前台四分钟以内。
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
pools=(T1 T2 T3 L1 L2 I0 I0r I2)
upools=(T1 T2 L1 L2 I0 I0r I2)
starts=(S0 SX SK SI)
summary_logs() {
    for code in new old fix; do for p in "${pools[@]}"; do echo "h-$code-$p"; done; done
    for code in new old fix; do for p in "${upools[@]}"; do echo "u-$code-$p"; done; done
    echo v-new v-old v-fix p-1 p-2 p-3 p-4 a2 k
    for b in b1 b2; do for s in "${starts[@]}"; do echo "$b-$s"; done; done
    echo kbi-new kbi-fix
}
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
    : >"$work/summary.txt"
    for name in $(summary_logs); do
        # RESULT 行原样（测试名与它同一行时只取 RESULT 起的那一段），前面标上日志名；测试框架的耗时去掉。
        grep -oE 'RESULT .*|test result: .*' "$work/logs/$name.log" | sed -E "s/; finished in [0-9.]+s//; s/^/[$name] /" >>"$work/summary.txt"
    done
    echo "summary: $(wc -l <"$work/summary.txt") 行 -> $work/summary.txt"
    exit 0
fi
repo="${SINGLEFS_REPO:?要设 SINGLEFS_REPO}"
tools="$repo"
[[ -f "$tools/research/scripts/run-with-memory-cap.sh" ]] || tools="${SINGLEFS_TOOLS:?SINGLEFS_REPO 下没有 research/scripts，要设 SINGLEFS_TOOLS}"
cap="$tools/research/scripts/run-with-memory-cap.sh"
capped="$tools/research/scripts/capped.sh"
threads="${OPUS_THREADS:-10}"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="${RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON:-派发提示写死编译 6G、跑 4G}"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-三方攻方腿模型复跑，在自己的工作目录里各跑各的}"
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS="${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-150}"

build() { # 代号：编 opus_b26_attack 与 checker_known_bad_images，记下两份测试二进制的路径
    local code="$1"
    (cd "$work/tree-$code" && CARGO_TARGET_DIR="$work/target-$code" nice -n 19 bash "$cap" 6G bash "$capped" "$threads" \
        cargo test --release --offline -p singlefs-harness --test opus_b26_attack --test checker_known_bad_images --no-run) \
        >"$work/logs/build-$code.log" 2>&1
    grep -oE '\(/[^)]*/opus_b26_attack-[0-9a-f]+\)' "$work/logs/build-$code.log" | tr -d '()' | tail -1 >"$work/target-$code/opus-bin.txt"
    grep -oE '\(/[^)]*/checker_known_bad_images-[0-9a-f]+\)' "$work/logs/build-$code.log" | tr -d '()' | tail -1 >"$work/target-$code/kbi-bin.txt"
}

run() { # 代号、日志名、线程、过滤串，其余是环境变量赋值；直接跑编好的测试二进制
    local code="$1" log="$2" n="$3" filter="$4"; shift 4
    env OPUS_CODE="$code" "$@" nice -n 19 bash "$cap" 4G bash "$capped" "$n" "$(cat "$work/target-$code/opus-bin.txt")" "$filter" --nocapture --test-threads 1 \
        >"$work/logs/$log.log" 2>&1
}

run_batch() { # 批号，之后每件「日志名|变量…」：并行起 b2_legal，每件退出码写 logs/<日志名>.rc，不带参数的 wait 等齐，数过再按次序读
    local batch="$1"; shift
    local names=()
    for item in "$@"; do names+=("${item%%|*}"); done
    for name in "${names[@]}"; do rm -f "$work/logs/$name.rc"; done
    for item in "$@"; do
        local name="${item%%|*}" vars="${item#*|}"
        # shellcheck disable=SC2086
        { set +e; run new "$name" 2 b2_legal $vars; echo "$?" >"$work/logs/$name.rc"; } &
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
        mkdir -p "$work/tree-new"
        rsync -a --exclude target --exclude .git "$repo/crates" "$repo/Cargo.toml" "$repo/Cargo.lock" "$work/tree-new/"
        cp "$here/opus_b26_attack.rs" "$work/tree-new/crates/singlefs-harness/tests/opus_b26_attack.rs"
        rsync -a "$work/tree-new/" "$work/tree-old/"
        patch -R -p1 -d "$work/tree-old" <"$here/batch26-walk.patch" >"$work/logs/setup-old.log" 2>&1
        rsync -a "$work/tree-new/" "$work/tree-fix/"
        patch -p0 -d "$work/tree-fix" <"$here/fix-FB-FV.diff" >"$work/logs/setup-fix.log" 2>&1
        ;;
    build-new | build-old | build-fix) build "${1#build-}" ;;
    h-new | h-old | h-fix) for p in "${pools[@]}"; do run "${1#h-}" "$1-$p" "$threads" h_copy_states OPUS_POOL="$p"; done ;;
    u-new | u-old | u-fix) for p in "${upools[@]}"; do run "${1#u-}" "$1-$p" "$threads" u_unreadable OPUS_POOL="$p"; done ;;
    v) for code in new old fix; do run "$code" "v-$code" "$threads" v_both; done ;;
    p)
        run new p-1 "$threads" p_details OPUS_POOL=I0 OPUS_ENTRY=1 OPUS_MOD=Flip OPUS_C0=S2 OPUS_C1=R
        run new p-2 "$threads" p_details OPUS_POOL=I2 OPUS_MOD=Flip OPUS_C0=S3 OPUS_C1=R
        run new p-3 "$threads" p_details OPUS_POOL=T2 OPUS_MOD=Flip OPUS_C0=Z OPUS_C1=Z
        run new p-4 "$threads" p_details OPUS_POOL=I0 OPUS_MOD=Keep OPUS_C0=Z OPUS_C1=Z
        ;;
    a2) run new a2 "$threads" a2_second ;;
    k) run new k "$threads" k_count ;;
    b1) run_batch 1 \
        "b1-S0|OPUS_START=S0 OPUS_SEED_FROM=9101 OPUS_HISTORIES=25 OPUS_STEPS=12" \
        "b1-SX|OPUS_START=SX OPUS_SEED_FROM=9201 OPUS_HISTORIES=25 OPUS_STEPS=12" \
        "b1-SK|OPUS_START=SK OPUS_SEED_FROM=9301 OPUS_HISTORIES=25 OPUS_STEPS=12" \
        "b1-SI|OPUS_START=SI OPUS_SEED_FROM=9401 OPUS_HISTORIES=25 OPUS_STEPS=12" ;;
    b2) run_batch 2 \
        "b2-S0|OPUS_START=S0 OPUS_SEED_FROM=9501 OPUS_HISTORIES=12 OPUS_STEPS=25" \
        "b2-SX|OPUS_START=SX OPUS_SEED_FROM=9601 OPUS_HISTORIES=12 OPUS_STEPS=25" \
        "b2-SK|OPUS_START=SK OPUS_SEED_FROM=9701 OPUS_HISTORIES=12 OPUS_STEPS=25" \
        "b2-SI|OPUS_START=SI OPUS_SEED_FROM=9801 OPUS_HISTORIES=12 OPUS_STEPS=25" ;;
    kbi)
        for code in new fix; do
            nice -n 19 bash "$cap" 4G bash "$capped" "$threads" "$(cat "$work/target-$code/kbi-bin.txt")" --test-threads "$threads" \
                >"$work/logs/kbi-$code.log" 2>&1
        done ;;
    *) echo "不认得的段：$1" >&2; return 2 ;;
    esac
}

if [[ "$stage" == all ]]; then
    for s in setup build-new build-old build-fix h-new h-old h-fix u-new u-old u-fix v p a2 k b1 b2 kbi; do
        echo "== $s"
        run_stage "$s"
    done
    bash "$0" "$work" summary
else
    run_stage "$stage"
fi
