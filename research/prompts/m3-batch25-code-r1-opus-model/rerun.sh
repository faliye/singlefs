#!/usr/bin/env bash
# m3-batch25-code-r1 攻方模型复跑。
#   SINGLEFS_REPO=<仓或冻结副本，含 crates/ 与 Cargo.*> [SINGLEFS_TOOLS=<仓根>] bash rerun.sh <空工作目录> <段|all>
#   bash rerun.sh compare <工作目录>        # 各段日志里的 RESULT 行（去掉秒数）排序，与 expected-summary.txt 比；不依赖 SINGLEFS_REPO
# 段：setup build a b1 b2 c fix-setup fix-build fix-a fix-kbi summary（fix-* 是攻方改法 F-N 的副本：fix-FN.diff 打在 tree 的拷贝上）
# 包装脚本（run-with-memory-cap.sh、capped.sh）取 $SINGLEFS_REPO/scripts/，没有就取 $SINGLEFS_TOOLS/scripts/
# （SINGLEFS_REPO 是只含 crates/ 的冻结副本时，要另设变量 SINGLEFS_TOOLS=<仓根>）。
# 编译与跑都经 run-with-memory-cap.sh（编译 6G、跑 4G），线程经 capped.sh（OPUS_THREADS，默认 10；并行的批每件 2）。
# 每段前台四分钟以内跑完。
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
logs_in_order=(a b1-S0 b1-SX b1-SK b1-SR b2-S0 b2-SX b2-SK b2-SR c-T2b c-T2k2 c-L2b c-I0b fix-a fix-kbi)
if [[ "$stage" == summary ]]; then
    # 每行前面带段名（同一形状在冻结副本与 F-N 副本上各一行，比的时候分得开），去掉随机器变的秒数。
    { for name in "${logs_in_order[@]}"; do
          grep -ohE 'RESULT .*' "$work/logs/$name.log" | sed -E 's/ (enumerate_|judge_)?seconds=[0-9.]+//g' | sed -E "s/^/[$name] /" || true
      done
      grep -ohE '^test [a-z0-9_]+ \.\.\. FAILED$|test result: [^;]*;[^;]*;' "$work/logs/fix-kbi.log" | sed -E 's/^/[fix-kbi] /'; } >"$work/summary.txt"
    echo "summary: $(wc -l <"$work/summary.txt") 行 -> $work/summary.txt"
    exit 0
fi
repo="${SINGLEFS_REPO:?要设 SINGLEFS_REPO}"
tools="$repo"
[[ -f "$tools/scripts/run-with-memory-cap.sh" ]] || tools="${SINGLEFS_TOOLS:?SINGLEFS_REPO 下没有 scripts，要设 SINGLEFS_TOOLS}"
cap="$tools/scripts/run-with-memory-cap.sh"
capped="$tools/scripts/capped.sh"
threads="${OPUS_THREADS:-10}"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="${RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON:-派发提示写死编译 6G、跑 4G}"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-三方攻方腿模型复跑，在自己的工作目录里各跑各的}"
# 别的会话占着内存时排队等一会儿（退 252 的那一次输出不算，原样参数重跑那一段）。
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS="${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-150}"

run_test() { # 日志名、线程数、过滤串，其余是环境变量赋值；直接跑编好的测试二进制
    local log="$1" n="$2" filter="$3"; shift 3
    env "$@" nice -n 19 bash "$cap" 4G bash "$capped" "$n" "$(cat "$work/target/opus-bin.txt")" "$filter" --nocapture --test-threads 1 \
        >"$work/logs/$log.log" 2>&1
}

run_batch() { # 批名，之后每件「日志名|过滤串|变量…」：并行起，每件退出码写 logs/<批名>-<件号>.rc，不带参数的 wait 等齐，数过再按次序读
    local batch="$1"; shift
    rm -f "$work/logs/$batch"-*.rc
    local index=0
    for item in "$@"; do
        index=$((index + 1))
        local name="${item%%|*}" rest="${item#*|}"
        local filter="${rest%%|*}" vars="${rest#*|}"
        # shellcheck disable=SC2086
        { set +e; run_test "$name" 2 "$filter" $vars; echo "$?" >"$work/logs/$batch-$index.rc"; } &
    done
    wait
    local count
    count=$(find "$work/logs" -maxdepth 1 -name "$batch-*.rc" | wc -l)
    if [[ "$count" -ne "$#" ]]; then
        echo "批 $batch：退出码文件 $count 个，派出去 $# 件，整批作废" >&2
        return 1
    fi
    index=0
    for item in "$@"; do
        index=$((index + 1))
        echo "批 $batch 件 $index ${item%%|*} 退出码 $(cat "$work/logs/$batch-$index.rc")"
    done
}

run_stage() {
    case "$1" in
    setup)
        mkdir -p "$work/tree"
        rsync -a --exclude target --exclude .git "$repo/crates" "$repo/Cargo.toml" "$repo/Cargo.lock" "$work/tree/"
        cp "$here/opus_b25_attack.rs" "$work/tree/crates/singlefs-harness/tests/opus_b25_attack.rs"
        ;;
    build)
        (cd "$work/tree" && CARGO_TARGET_DIR="$work/target" nice -n 19 bash "$cap" 6G bash "$capped" "$threads" \
            cargo test --release --offline -p singlefs-harness --test opus_b25_attack --no-run) >"$work/logs/build.log" 2>&1
        grep -oE '\(/[^)]*/opus_b25_attack-[0-9a-f]+\)' "$work/logs/build.log" | tr -d '()' | tail -1 >"$work/target/opus-bin.txt"
        ;;
    a) run_test a "$threads" a_gate_shapes OPUS_UNUSED=1 ;;
    b1) run_batch b1 \
        "b1-S0|b_one_birth|OPUS_START=S0 OPUS_SEED_FROM=101 OPUS_HISTORIES=30 OPUS_STEPS=12" \
        "b1-SX|b_one_birth|OPUS_START=SX OPUS_SEED_FROM=201 OPUS_HISTORIES=30 OPUS_STEPS=12" \
        "b1-SK|b_one_birth|OPUS_START=SK OPUS_SEED_FROM=301 OPUS_HISTORIES=30 OPUS_STEPS=12" \
        "b1-SR|b_one_birth|OPUS_START=SR OPUS_SEED_FROM=401 OPUS_HISTORIES=4 OPUS_STEPS=8" ;;
    b2) run_batch b2 \
        "b2-S0|b_one_birth|OPUS_START=S0 OPUS_SEED_FROM=131 OPUS_HISTORIES=14 OPUS_STEPS=25" \
        "b2-SX|b_one_birth|OPUS_START=SX OPUS_SEED_FROM=231 OPUS_HISTORIES=14 OPUS_STEPS=25" \
        "b2-SK|b_one_birth|OPUS_START=SK OPUS_SEED_FROM=331 OPUS_HISTORIES=14 OPUS_STEPS=25" \
        "b2-SR|b_one_birth|OPUS_START=SR OPUS_SEED_FROM=405 OPUS_HISTORIES=4 OPUS_STEPS=8" ;;
    c) run_batch c \
        "c-T2b|c_second_sample|OPUS_POOL=T2b" \
        "c-T2k2|c_second_sample|OPUS_POOL=T2k2" \
        "c-L2b|c_second_sample|OPUS_POOL=L2b" \
        "c-I0b|c_second_sample|OPUS_POOL=I0b" ;;
    fix-setup)
        mkdir -p "$work/tree-fix"
        rsync -a "$work/tree/" "$work/tree-fix/"
        patch -d "$work/tree-fix" -p1 <"$here/fix-FN.diff" >"$work/logs/fix-setup.log" 2>&1 ;;
    fix-build)
        (cd "$work/tree-fix" && CARGO_TARGET_DIR="$work/target-fix" nice -n 19 bash "$cap" 6G bash "$capped" "$threads" \
            cargo test --release --offline -p singlefs-harness --test opus_b25_attack --test checker_known_bad_images --no-run) >"$work/logs/fix-build.log" 2>&1
        grep -oE '\(/[^)]*/opus_b25_attack-[0-9a-f]+\)' "$work/logs/fix-build.log" | tr -d '()' | tail -1 >"$work/target-fix/opus-bin.txt"
        grep -oE '\(/[^)]*/checker_known_bad_images-[0-9a-f]+\)' "$work/logs/fix-build.log" | tr -d '()' | tail -1 >"$work/target-fix/kbi-bin.txt" ;;
    fix-a)
        nice -n 19 bash "$cap" 4G bash "$capped" "$threads" "$(cat "$work/target-fix/opus-bin.txt")" a_gate_shapes --nocapture --test-threads 1 \
            >"$work/logs/fix-a.log" 2>&1 ;;
    fix-kbi)
        # 这一段退出码不是 0 是预期的（F-N 改了 N5-b 那条用例钉的判定）：结论看 summary 里 [fix-kbi] 那几行。
        nice -n 19 bash "$cap" 4G bash "$capped" "$threads" "$(cat "$work/target-fix/kbi-bin.txt")" --test-threads "$threads" \
            >"$work/logs/fix-kbi.log" 2>&1 || echo "fix-kbi 退出码 $?" ;;
    *) echo "不认得的段：$1" >&2; return 2 ;;
    esac
}

if [[ "$stage" == all ]]; then
    for s in setup build a b1 b2 c fix-setup fix-build fix-a fix-kbi; do
        echo "== $s"
        run_stage "$s"
    done
    bash "$0" "$work" summary
else
    run_stage "$stage"
fi
