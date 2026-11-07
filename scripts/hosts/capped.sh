#!/usr/bin/env bash
# admission: always 每一次调都判此刻的仓与这一次的参数，上一次的结论不替这一次作保
# run-condition: none 只是给一条命令套上线程上限再 exec 它，除了 bash 之外没有环境要求
# 给一条命令定线程上限：把编译、测试与各装置的并行度环境变量一次设成同一个数，再执行那条命令。
#
# 为什么：几个 agent 同时在一台机器上跑时，各按整机核数起线程会把机器超卖——测试变慢、变异撞超时、
# 计时不准（records/2026-09-16-subagent拆分提案.md 第四十节第 7 行）。上限默认是每台默认重活线程数：
# .claude/heavy-thread-default 的登记值（scripts/hosts/heavy-threads.sh 的 heavy_thread_default 取，SINGLEFS_THREAD_CAP 设了照它），
# 统一任务服务给每次尝试的命令外面套 capped.sh <每台默认重活线程数>（task-server/README.md），调用方不写线程上限。
#
# 只认服务直接起的那一层：task-server 起每一次尝试时给这一层设 CAPPED_ADMITTED_BY_TASK_SERVER=<任务号>.<第几次尝试>（task-server/task_server_lib.py 的
#   THREAD_ADMITTED_VARIABLE；非空就算设了，值不打、不记）。这里读掉它、exec 命令之前 unset，命令里看不到它；环境里没有它就拒、退 2、命令没跑。
#   于是任务里面的脚本再套一层 capped.sh 当场被拒：线程数由服务定，脚本照读服务交的 SINGLEFS_THREAD_CAP、CARGO_BUILD_JOBS 这些变量，不再套这一层。
#   --print 与 --selftest 不看它。
#
#   capped.sh <上限> <命令> [参数…]   # 设好变量后 exec 那条命令，退出码是那条命令的
#   capped.sh --print <上限>          # 只打印会设的变量（一行一个），不执行
#   capped.sh --selftest              # 自证：上限写错拒绝、变量都设上、退出码原样传回、default 取登记值
# <上限> 是正整数，或 default：取 heavy_thread_default 交的数（登记文件不在、写坏，SINGLEFS_THREAD_CAP 不是正整数，都拒、退 2，不退回按核数算）。
#   套在另一层 capped.sh <N> 里面的 default 取到的是 N（外层设了 SINGLEFS_THREAD_CAP）。
#
# 设的变量：cargo 的 CARGO_BUILD_JOBS、RUST_TEST_THREADS，crates/singlefs-verification 里各装置读的
# SINGLEFS_CRASH_POINT_REPLAY_THREADS、SINGLEFS_CRASH_INJECTION_THREADS、SINGLEFS_FAULT_INJECTION_WORKER_THREADS、
# SINGLEFS_BAD_DISK_INPUT_WORKER_THREADS、SINGLEFS_RANDOM_HISTORY_THREADS，验证包各处只读的 E161_THREADS（E161 装置）、
# SINGLEFS_CRASH_JUDGE_CPU_THREADS（GPU 对拍与记录核对器两份测试的 CPU 判器一侧）、SINGLEFS_CRASH_AMPLIFICATION_THREADS（崩溃放量的核对线程）、
# E165_THREADS（E165 装置）、E168_THREADS（E168 装置）、E175_WORKER_THREADS（E175 装置）、E180_THREADS、E182_THREADS、E183_THREADS、E184_THREADS、E185_THREADS
# （各自的装置与分片脚本）、GATE_MUTATION_KEY_WORKERS（crates 变异表门禁算行键的子进程数），以及通用的 SINGLEFS_THREAD_CAP
# （实验装置自己的线程变量没登记在这里的，加进本文件的 variable_names，或让装置读 SINGLEFS_THREAD_CAP；门禁 code-tooling 的 heavy-thread-defaults 格判：
# 装置读自己的线程变量，这里不设、文件里也没回退到 SINGLEFS_THREAD_CAP 或登记值的，红）。
# 弄坏开关（只给 --selftest 证红用；平常跑设了就在一切之前退 2、点名变量只在自证里认，自证起的子进程由 CAPPED_SELFTEST_MARK 认）：
#   CAPPED_BREAK=default-from-nproc  default 取本机核数、不读登记、不认 SINGLEFS_THREAD_CAP（本机核数不是 5、7、登记值时才红）
#       「✗ 自检：登记 7 ⇒ default 把」、「✗ 自检：登记 7 ⇒ --print default」、「✗ 自检：SINGLEFS_THREAD_CAP=5、登记 7 ⇒ default 设成 5」、
#       「✗ 自检：SINGLEFS_THREAD_CAP=0 ⇒ default 拒」、「✗ 自检：登记文件不在 ⇒ default 拒」、「✗ 自检：登记写坏 ⇒ default 拒」、
#       「✗ 自检：本仓的 default 与 heavy_thread_default 同值」
#   CAPPED_BREAK=e161-threads-unset  变量清单里去掉 E161_THREADS，装置只读它而这里不设（逐个变量的几格以清单为准，清单短了它们照绿，所以这一格单独钉）
#       「✗ 自检：装置只读的 E161_THREADS 要在变量清单里」
#   CAPPED_BREAK=crash-judge-cpu-threads-unset  变量清单里去掉 SINGLEFS_CRASH_JUDGE_CPU_THREADS（与 e161-threads-unset 同一格）
#       「✗ 自检：装置只读的 SINGLEFS_CRASH_JUDGE_CPU_THREADS 要在变量清单里」
#   CAPPED_BREAK=reject-says-default-for-all  上限写错的拒绝说明一律写「没给线程上限写 default」
#       「✗ 自检：上限写错的拒绝说明应当写」
#   CAPPED_BREAK=admission-mark-kept  exec 命令之前不 unset CAPPED_ADMITTED_BY_TASK_SERVER：标记随环境传给命令，任务里面再套一层照跑
#       「✗ 自检：被服务起的命令里面再调 capped.sh 应当拒」
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
source "$(dirname "${BASH_SOURCE[0]}")/heavy-threads.sh"

# 弄坏开关只在自证里认：命令行不是单独一个 --selftest、也不是自证起的子进程（CAPPED_SELFTEST_MARK 对得上）而变量非空，参数解析之前就退 2（不管值是什么）。
if [[ -n "${CAPPED_BREAK:-}" && ! ( "$#" -eq 1 && "${1:-}" == --selftest ) && "${CAPPED_SELFTEST_MARK:-}" != capped-selftest ]]; then
    echo "  ✗ CAPPED_BREAK=${CAPPED_BREAK} 是弄坏开关，只在自证（bash scripts/hosts/capped.sh --selftest）里认，平常跑设了就拒，这一趟什么都没做" >&2
    echo "  → 怎么办：去掉 CAPPED_BREAK 再跑；要证红就跑 CAPPED_BREAK=<项> bash scripts/hosts/capped.sh --selftest" >&2
    exit 2
fi

variable_names=(
    CARGO_BUILD_JOBS
    RUST_TEST_THREADS
    SINGLEFS_CRASH_POINT_REPLAY_THREADS
    SINGLEFS_CRASH_INJECTION_THREADS
    SINGLEFS_FAULT_INJECTION_WORKER_THREADS
    SINGLEFS_BAD_DISK_INPUT_WORKER_THREADS
    SINGLEFS_RANDOM_HISTORY_THREADS
    E161_THREADS
    SINGLEFS_CRASH_JUDGE_CPU_THREADS
    SINGLEFS_CRASH_AMPLIFICATION_THREADS
    E165_THREADS
    E168_THREADS
    E175_WORKER_THREADS
    E180_THREADS
    E182_THREADS
    E183_THREADS
    E184_THREADS
    E185_THREADS
    GATE_MUTATION_KEY_WORKERS
    SINGLEFS_THREAD_CAP
)
# 验证包各处只读的线程变量（文件头「设的变量」）：自证逐个核它们在清单里
device_only_variable_names=(E161_THREADS SINGLEFS_CRASH_JUDGE_CPU_THREADS SINGLEFS_CRASH_AMPLIFICATION_THREADS E165_THREADS E168_THREADS E175_WORKER_THREADS E180_THREADS E182_THREADS E183_THREADS E184_THREADS E185_THREADS)
# 弄坏开关 e161-threads-unset、crash-judge-cpu-threads-unset：清单里去掉那一个变量（只读它的装置，capped.sh N 到不了）
case "${CAPPED_BREAK:-}" in
    e161-threads-unset) dropped_name=E161_THREADS ;;
    crash-judge-cpu-threads-unset) dropped_name=SINGLEFS_CRASH_JUDGE_CPU_THREADS ;;
    *) dropped_name="" ;;
esac
if [[ -n "$dropped_name" ]]; then
    kept_names=()
    for name in "${variable_names[@]}"; do [[ "$name" == "$dropped_name" ]] || kept_names+=("$name"); done
    variable_names=("${kept_names[@]}")
fi

reject_cap() {
    local howto="线程上限不由调用方写：整条命令经 task-server/task submit 交，服务套 capped.sh <每台默认重活线程数>；default 取 .claude/heavy-thread-default 的登记值"
    # 弄坏开关 reject-says-default-for-all：没给线程上限的一律写 default（子 agent 也被指到 default 上）
    [[ "${CAPPED_BREAK:-}" == reject-says-default-for-all ]] && howto="派发提示给了「线程上限：N」照 N，没给写 default（取 .claude/heavy-thread-default 的登记值）"
    echo "  ✗ 线程上限要是正整数或 default，收到的是「$1」" >&2
    echo "  → 怎么办：写成 capped.sh <正整数|default> <命令>；$howto" >&2
    exit 2
}

check_cap() {
    [[ "$1" =~ ^[1-9][0-9]*$ ]] || reject_cap "$1"
}

# resolved_cap <上限>：default 换成每台默认重活线程数（heavy_thread_default，拒了它自己在 stderr 写原因与出路），别的原样；打到 stdout
resolved_cap() {
    if [[ "$1" != default ]]; then printf '%s\n' "$1"; return 0; fi
    if [[ "${CAPPED_BREAK:-}" == default-from-nproc ]]; then nproc; return 0; fi
    heavy_thread_default
}

run_selftest() {
    local failures=0 checked=0 output status
    export CAPPED_SELFTEST_MARK=capped-selftest   # 起的子进程（bash "$0" …，环境原样继承）靠它认：弄坏开关只在自证里认
    export CAPPED_ADMITTED_BY_TASK_SERVER=selftest-task.1   # 各格起的这一份都当服务直接起的那一层（文件头「只认服务直接起的那一层」）；要的格自己去掉
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
    # 拒绝说明里的出路：线程上限不由调用方写、整条经 task-server/task submit 交，default 只给脚本内部（task-server/README.md「提交写法」同一说法）
    checked=$((checked + 1))
    output="$(bash "$0" abc true 2>&1 || true)"
    if [[ "$output" != *"线程上限不由调用方写：整条命令经 task-server/task submit 交"* || "$output" != *"default 取 .claude/heavy-thread-default 的登记值"* ]]; then
        echo "  ✗ 自检：上限写错的拒绝说明应当写「线程上限不由调用方写：整条命令经 task-server/task submit 交」与「default 取 .claude/heavy-thread-default 的登记值」，实际「$output」"; failures=$((failures + 1))
        echo "  → 怎么办：reject_cap 的出路改回：线程上限不由调用方写、整条经 task-server/task submit 交，default 取登记值"
    fi
    # 只认服务直接起的那一层（文件头）：没有准入标记 ⇒ 拒、退 2、命令没跑；被服务起的命令里面再调一层 ⇒ 里层拒、退 2、命令没跑
    local admission_scratch
    admission_scratch="$(mktemp -d)"
    checked=$((checked + 1))
    if env -u CAPPED_ADMITTED_BY_TASK_SERVER bash "$0" 3 touch "$admission_scratch/unadmitted-ran" 2>"$admission_scratch/unadmitted.err"; then status=0; else status=$?; fi
    if [[ "$status" != 2 || -e "$admission_scratch/unadmitted-ran" ]] || ! grep -q '任务里面不再套包装：内存、线程、机器、卡由 task-server 定；删掉这一层' "$admission_scratch/unadmitted.err"; then
        echo "  ✗ 自检：没有服务给的准入标记应当拒（退 2、命令没跑、出路写「任务里面不再套包装」），实际退 $status：$(head -c 300 "$admission_scratch/unadmitted.err")"; failures=$((failures + 1))
        echo "  → 怎么办：run 那一段在 exec 之前判 CAPPED_ADMITTED_BY_TASK_SERVER，没有就拒"
    fi
    checked=$((checked + 1))
    if bash "$0" 3 bash "$0" 3 touch "$admission_scratch/nested-ran" 2>"$admission_scratch/nested.err"; then status=0; else status=$?; fi
    if [[ "$status" != 2 || -e "$admission_scratch/nested-ran" ]]; then
        echo "  ✗ 自检：被服务起的命令里面再调 capped.sh 应当拒（里层退 2、命令没跑），实际退 $status、命令$([[ -e "$admission_scratch/nested-ran" ]] && echo 跑了 || echo 没跑)"; failures=$((failures + 1))
        echo "  → 怎么办：exec 命令之前 unset CAPPED_ADMITTED_BY_TASK_SERVER，命令里看不到它；CAPPED_BREAK 设着的话这里本来就该红"
    fi
    rm -rf "${admission_scratch:?}"
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
    # 装置自己读的线程变量要在清单里（device_only_variable_names：E161 装置的 E161_THREADS、两份 CPU 判器测试的 SINGLEFS_CRASH_JUDGE_CPU_THREADS、
    # 崩溃放量的 SINGLEFS_CRASH_AMPLIFICATION_THREADS、E165 与 E168 装置的 E165_THREADS、E168_THREADS），清单里没有，capped.sh N 就到不了那一处；
    # 清单是上面循环的被测对象，这一格钉住「这几个在里面」，不让清单被改短
    checked=$((checked + 1))
    printed_with_three="$(bash "$0" --print 3)"
    for name in "${device_only_variable_names[@]}"; do
        if ! grep -qx "$name=3" <<<"$printed_with_three"; then
            echo "  ✗ 自检：装置只读的 $name 要在变量清单里，实际没有"; failures=$((failures + 1))
            echo "  → 怎么办：把 $name 加回 variable_names；装置读的别的线程变量也一样要在清单里，或让装置回退到 SINGLEFS_THREAD_CAP"
        fi
    done
    run_default_selftest
    if [ "$failures" -ne 0 ]; then
        echo "  ✗ capped.sh 自检 $failures 处不对（查了 $checked 项）"
        echo "  → 怎么办：照上面逐条改"
        exit 1
    fi
    echo "  ✓ capped.sh 自检通过（查了 $checked 项：上限写错两种拒绝、拒绝说明的出路、没有服务的准入标记拒、被服务起的命令里面再套一层拒、${#variable_names[@]} 个变量都设上、装置只读的 ${#device_only_variable_names[@]} 个线程变量在清单里、退出码原样传回、default 取登记值七格）"
}

# default 那七格：样本树里放这一份与 heavy-threads.sh 的拷贝、一份只交还参数的 preflight 桩（准入不在这里判），登记按格写；
# failures、checked 是 run_selftest 的，这里接着累加
run_default_selftest() {
    local scratch here tree output status name want_lines got_lines real_value
    here="$(cd "$(dirname "$0")" && pwd -P)"
    if ! scratch="$(mktemp -d)"; then
        echo "  ✗ 自检：建不了放样本树的临时目录"; failures=$((failures + 1))
        echo "  → 怎么办：看 \$TMPDIR 满没满、有没有写权限"
        return
    fi
    # default_tree <目录名> <登记内容，MISSING 是不建>：建一棵样本树，打树根（capped.sh 的拷贝在 <树根>/scripts/hosts/capped.sh）
    default_tree() {
        mkdir -p "$scratch/$1/scripts/hosts" "$scratch/$1/.claude/scripts"
        cp "$here/capped.sh" "$here/heavy-threads.sh" "$scratch/$1/scripts/hosts/"
        printf '%s\n' 'preflight() { shift; PREFLIGHT_ARGUMENTS=("$@"); }' > "$scratch/$1/.claude/scripts/preflight.sh"
        if [[ "$2" != MISSING ]]; then printf '%s\n' "$2" > "$scratch/$1/.claude/heavy-thread-default"; fi
        printf '%s\n' "$scratch/$1"
    }
    default_failed() {
        failures=$((failures + 1))
        echo "  ✗ 自检：$1"
        echo "  → 怎么办：default 要经 heavy-threads.sh 的 heavy_thread_default 取登记值（resolved_cap），拒了退 2、不起命令；CAPPED_BREAK 设着的话这里本来就该红"
    }
    tree="$(default_tree registered '7  # 样本理由')"
    checked=$((checked + 1))
    for name in "${variable_names[@]}"; do
        output="$(env -u SINGLEFS_THREAD_CAP bash "$tree/scripts/hosts/capped.sh" default printenv "$name" 2>&1 || true)"
        if [[ "$output" != 7 ]]; then default_failed "登记 7 ⇒ default 把 $name 设成 7，实际「$output」"; break; fi
    done
    checked=$((checked + 1))
    want_lines="$(for name in "${variable_names[@]}"; do echo "$name=7"; done)"
    got_lines="$(env -u SINGLEFS_THREAD_CAP bash "$tree/scripts/hosts/capped.sh" --print default 2>&1 || true)"
    [[ "$got_lines" == "$want_lines" ]] || default_failed "登记 7 ⇒ --print default 逐行打 <变量>=7，实际「$(head -n 1 <<<"$got_lines")」…"
    checked=$((checked + 1))
    output="$(env SINGLEFS_THREAD_CAP=5 bash "$tree/scripts/hosts/capped.sh" default printenv CARGO_BUILD_JOBS 2>&1 || true)"
    [[ "$output" == 5 ]] || default_failed "SINGLEFS_THREAD_CAP=5、登记 7 ⇒ default 设成 5，实际「$output」"
    checked=$((checked + 1))
    if env SINGLEFS_THREAD_CAP=0 bash "$tree/scripts/hosts/capped.sh" default touch "$scratch/ran-cap-zero" 2>"$scratch/cap-zero.stderr"; then status=0; else status=$?; fi
    [[ "$status" == 2 && ! -e "$scratch/ran-cap-zero" ]] || default_failed "SINGLEFS_THREAD_CAP=0 ⇒ default 拒、退 2、不起命令，实际退 $status"
    for case_name in missing broken; do
        checked=$((checked + 1))
        if [[ "$case_name" == missing ]]; then tree="$(default_tree missing MISSING)"; else tree="$(default_tree broken 'abc  # 样本理由')"; fi
        if env -u SINGLEFS_THREAD_CAP bash "$tree/scripts/hosts/capped.sh" default touch "$scratch/ran-$case_name" 2>"$scratch/$case_name.stderr"; then status=0; else status=$?; fi
        if [[ "$status" != 2 || -e "$scratch/ran-$case_name" ]] || ! grep -q '✗' "$scratch/$case_name.stderr"; then
            default_failed "登记$([[ "$case_name" == missing ]] && echo 文件不在 || echo 写坏) ⇒ default 拒（退 2、stderr 有拒绝标记、不起命令），实际退 $status"
        fi
    done
    checked=$((checked + 1))
    real_value="$(env -u SINGLEFS_THREAD_CAP bash -c 'source "$1"; heavy_thread_default' capped-selftest "$here/heavy-threads.sh" 2>&1)"
    output="$(env -u SINGLEFS_THREAD_CAP bash "$0" default printenv SINGLEFS_THREAD_CAP 2>&1 || true)"
    [[ "$real_value" =~ ^[1-9][0-9]*$ && "$output" == "$real_value" ]] || default_failed "本仓的 default 与 heavy_thread_default 同值：函数交「$real_value」，capped.sh default 设成「$output」"
    rm -rf "${scratch:?}"
}

case "${1:-}" in
    --selftest) run_selftest; exit 0 ;;
    --print)
        if ! cap="$(resolved_cap "${2:-}")"; then exit 2; fi
        check_cap "$cap"
        for name in "${variable_names[@]}"; do echo "$name=$cap"; done
        exit 0 ;;
    "") reject_cap "" ;;
esac

if ! cap="$(resolved_cap "$1")"; then exit 2; fi
shift
check_cap "$cap"
if [ "$#" -eq 0 ]; then
    echo "  ✗ 没给要执行的命令" >&2
    echo "  → 怎么办：写成 capped.sh $cap <命令> [参数…]" >&2
    exit 2
fi
# 只认服务直接起的那一层（文件头）：没有准入标记就拒；有就 unset，命令里看不到它（弄坏开关 admission-mark-kept 下照留）
if [[ -z "${CAPPED_ADMITTED_BY_TASK_SERVER:-}" ]]; then
    echo "  ✗ capped.sh 只认 task-server 起尝试时直接套的那一层，环境里没有服务给的 CAPPED_ADMITTED_BY_TASK_SERVER（任务里面又套了一层，或不经服务直接起），命令没跑" >&2
    echo "  → 怎么办：任务里面不再套包装：内存、线程、机器、卡由 task-server 定；删掉这一层（线程数照读服务交的 SINGLEFS_THREAD_CAP、CARGO_BUILD_JOBS 这些变量），整条经 task-server/task submit -- <命令…> 交" >&2
    exit 2
fi
[[ "${CAPPED_BREAK:-}" == admission-mark-kept ]] || unset CAPPED_ADMITTED_BY_TASK_SERVER
for name in "${variable_names[@]}"; do export "$name=$cap"; done
exec "$@"
