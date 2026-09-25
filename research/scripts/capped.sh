#!/usr/bin/env bash
# 给一条命令定线程上限：把编译、测试与各装置的并行度环境变量一次设成同一个数，再执行那条命令。
#
# 为什么：几个 agent 同时在一台机器上跑时，各按整机核数起线程会把机器超卖——测试变慢、变异撞超时、
# 计时不准（records/2026-09-16-subagent拆分提案.md 第四十节第 7 行）。主 agent 派发时按
# 「整机核数 ÷ 同时在跑的重活数」给每个 agent 一个上限，agent 用这个脚本把它落到每条重命令上。
#
#   capped.sh <上限> <命令> [参数…]   # 设好变量后 exec 那条命令，退出码是那条命令的
#   capped.sh --print <上限>          # 只打印会设的变量（一行一个），不执行
#   capped.sh --selftest              # 自证：上限写错拒绝、变量都设上、退出码原样传回
#
# 设的变量：cargo 的 CARGO_BUILD_JOBS、RUST_TEST_THREADS，crates/singlefs-harness 里各装置读的
# SINGLEFS_LAYER0_THREADS、SINGLEFS_CRASH_INJECTION_THREADS、SINGLEFS_FAULT_INJECTION_WORKER_THREADS、
# SINGLEFS_BAD_DISK_INPUT_WORKER_THREADS、SINGLEFS_RANDOM_HISTORY_THREADS，以及通用的 SINGLEFS_THREAD_CAP
# （实验装置自己的线程变量没登记在这里的，照派发提示另设，或让装置读 SINGLEFS_THREAD_CAP）。
set -euo pipefail

variable_names=(
    CARGO_BUILD_JOBS
    RUST_TEST_THREADS
    SINGLEFS_LAYER0_THREADS
    SINGLEFS_CRASH_INJECTION_THREADS
    SINGLEFS_FAULT_INJECTION_WORKER_THREADS
    SINGLEFS_BAD_DISK_INPUT_WORKER_THREADS
    SINGLEFS_RANDOM_HISTORY_THREADS
    SINGLEFS_THREAD_CAP
)

reject_cap() {
    echo "  ✗ 线程上限要是正整数，收到的是「$1」" >&2
    echo "  → 怎么办：写成 capped.sh <正整数> <命令>；上限按派发提示里的「线程上限：N」取" >&2
    exit 2
}

check_cap() {
    [[ "$1" =~ ^[1-9][0-9]*$ ]] || reject_cap "$1"
}

run_selftest() {
    local failures=0 checked=0 output status
    checked=$((checked + 1))
    if bash "$0" 0 true 2>/dev/null; then
        echo "  ✗ 自检：上限 0 应当被拒绝"; failures=$((failures + 1))
        echo "  → 怎么办：check_cap 的正则要拒绝 0"
    fi
    checked=$((checked + 1))
    if bash "$0" abc true 2>/dev/null; then
        echo "  ✗ 自检：上限 abc 应当被拒绝"; failures=$((failures + 1))
        echo "  → 怎么办：check_cap 的正则要拒绝非数字"
    fi
    for name in "${variable_names[@]}"; do
        checked=$((checked + 1))
        output="$(bash "$0" 3 printenv "$name" || true)"
        if [ "$output" != 3 ]; then
            echo "  ✗ 自检：$name 应当是 3，实际「$output」"; failures=$((failures + 1))
            echo "  → 怎么办：variable_names 里的每个名字都要 export 成上限"
        fi
    done
    checked=$((checked + 1))
    if bash "$0" 2 bash -c 'exit 7'; then status=0; else status=$?; fi
    if [ "$status" != 7 ]; then
        echo "  ✗ 自检：退出码应当原样传回 7，实际 $status"; failures=$((failures + 1))
        echo "  → 怎么办：设完变量要 exec 那条命令，不能吞掉它的退出码"
    fi
    if [ "$failures" -ne 0 ]; then
        echo "  ✗ capped.sh 自检 $failures 处不对（查了 $checked 项）"
        echo "  → 怎么办：照上面逐条改"
        exit 1
    fi
    echo "  ✓ capped.sh 自检通过（查了 $checked 项：上限写错两种拒绝、${#variable_names[@]} 个变量都设上、退出码原样传回）"
}

case "${1:-}" in
    --selftest) run_selftest; exit 0 ;;
    --print)
        check_cap "${2:-}"
        for name in "${variable_names[@]}"; do echo "$name=$2"; done
        exit 0 ;;
    "") reject_cap "" ;;
esac

cap="$1"; shift
check_cap "$cap"
if [ "$#" -eq 0 ]; then
    echo "  ✗ 没给要执行的命令" >&2
    echo "  → 怎么办：写成 capped.sh $cap <命令> [参数…]" >&2
    exit 2
fi
for name in "${variable_names[@]}"; do export "$name=$cap"; done
exec "$@"
