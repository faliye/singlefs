#!/usr/bin/env bash
# 三方腿模型目录里 tools/rerun.sh 的模板：腿把它拷成 <模型目录>/tools/rerun.sh，只改 rerun_steps 那一段。
# 仓根与冻结副本只从参数接，不从这份脚本自己的路径往上推：判决复跑时模型目录被拷进草稿目录跑
# （.claude/rules/three-way-inference.md 判决要引的复跑数那一条），从自身路径推出来的仓根在副本里就不是仓了。
#
# 用法（整条交 task-server 跑，工作目录随意；不套内存包装、不写线程：上限、线程与机器都由服务定）：
#   task-server/task submit -- bash <模型目录>/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录> [--model <模型目录>]
#   bash <这份脚本> --selftest
#     --repo    仓根：要有 .git 与 task-server/task（判是不是本仓）
#     --frozen  冻结副本（腿开工时拷出的那棵树）：要是一个目录
#     --work    新的草稿目录：不在、或是空目录；跑出来的全写在这里
#     --model   模型目录（原型、变异表、out/）：默认是本脚本的上一层——模型目录整份拷走时它跟着走；只拷这一份脚本时用它指过去
# 退出码：0 跑完；2 用法错（缺参数、仓根不像本仓、冻结副本不在、草稿目录已存在且非空、模型目录不在），stderr 列用法；
#         别的是 rerun_steps 里出错那一步的退出码。--selftest：0 全过，1 有格红。
#
# 准入与运行条件不写、不调 preflight：这份模板拷进模型目录、草稿目录跑，不从自身路径就找不到 .claude/scripts/preflight.sh；
# 环境要求（仓根、冻结副本、草稿目录）由 parse_arguments 自己判，不满足退 2。
#
# 弄坏开关（只给 --selftest 证红用）：
#   RERUN_TEMPLATE_BREAK=root-from-self-path  仓根改回从本脚本路径往上推四层（旧 rerun.sh 的写法），--repo 不认
#       「✗ 自检：拷进与仓无关的临时目录、传参应当跑通」那一格红
#       连带红的格：「✗ 自检：--work 指到已有输出的目录应当退 2、不在旧输出上接着写」
#   RERUN_TEMPLATE_BREAK=root-check-by-wrapper  判是不是本仓看 scripts/hosts/run-with-memory-cap.sh 在不在（不看 task-server/task）
#       「✗ 自检：--repo 指到只有内存包装、没有 task-server/task 的旧仓应当退 2」那一格红
#       连带红的格（自证里的假仓只有 task-server/task、没有内存包装，被当成不是本仓）：「✗ 自检：拷进与仓无关的临时目录、传参应当跑通」、「✗ 自检：--work 指到已有输出的目录应当退 2、不在旧输出上接着写」
set -euo pipefail

refuse() {   # refuse <卡在哪>：打出卡在哪、出路与用法，退 2
  echo "  ✗ $1" >&2
  echo "  → 怎么办：仓根与冻结副本都从参数给（脚本不从自己的路径推）；草稿目录给一个新的；整条交 task-server/task submit -- bash <这份脚本> … 跑" >&2
  cat >&2 <<'USAGE'
用法：rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录> [--model <模型目录>]
      rerun.sh --selftest
USAGE
  exit 2
}

parse_arguments() {
  REPO_ROOT="" FROZEN="" WORK="" MODEL=""
  while (($#)); do
    case "$1" in
      --repo|--frozen|--work|--model)
        (($# >= 2)) || refuse "$1 后面要跟一个路径"
        case "$1" in
          --repo) REPO_ROOT="$2" ;;
          --frozen) FROZEN="$2" ;;
          --work) WORK="$2" ;;
          --model) MODEL="$2" ;;
        esac
        shift 2 ;;
      *) refuse "认不出参数：$1" ;;
    esac
  done
  [[ -n "$REPO_ROOT" ]] || refuse "缺 --repo <仓根>"
  [[ -n "$FROZEN" ]] || refuse "缺 --frozen <冻结副本>"
  [[ -n "$WORK" ]] || refuse "缺 --work <新的草稿目录>"
  if [[ "${RERUN_TEMPLATE_BREAK:-}" == root-from-self-path ]]; then
    REPO_ROOT="$(dirname "${BASH_SOURCE[0]}")/../../../.."
  fi
  local repository_marker="task-server/task"
  [[ "${RERUN_TEMPLATE_BREAK:-}" != root-check-by-wrapper ]] || repository_marker="scripts/hosts/run-with-memory-cap.sh"
  [[ -e "$REPO_ROOT/.git" && -f "$REPO_ROOT/$repository_marker" ]] \
    || refuse "--repo $REPO_ROOT 不像本仓（要有 .git 与 $repository_marker）"
  REPO_ROOT="$(cd "$REPO_ROOT" && pwd)"
  [[ -d "$FROZEN" ]] || refuse "--frozen $FROZEN 不是目录"
  FROZEN="$(cd "$FROZEN" && pwd)"
  MODEL="${MODEL:-$(dirname "${BASH_SOURCE[0]}")/..}"
  [[ -d "$MODEL" ]] || refuse "模型目录 $MODEL 不在（只拷了这一份脚本的，用 --model 指过去）"
  MODEL="$(cd "$MODEL" && pwd)"
  if [[ -e "$WORK" && -n "$(ls -A "$WORK")" ]]; then
    refuse "--work $WORK 已存在且非空：给一个新的草稿目录，不在旧输出上接着写"
  fi
  mkdir -p "$WORK"
  WORK="$(cd "$WORK" && pwd)"
}

# ── 这一腿要跑的步骤：腿把这一段换成自己的 ──
# 路径只用 $REPO_ROOT、$FROZEN、$MODEL、$WORK 四个变量引，不写死、不从 $0 推；cargo 直接起（线程由 task-server 给整条套的 capped.sh <N> 定，这里不再套），
# 输出写进 $WORK，最后打出与 $MODEL/out/ 里哪一份逐行比。模板自带的这一段只拷冻结副本、打出四个路径，自证跑的就是它。
rerun_steps() {
  mkdir -p "$WORK/tree"
  rsync -a --exclude target --exclude .git "$FROZEN/" "$WORK/tree/"
  echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"
  local tests_directory="$WORK/tree/crates/singlefs-mount-runtime/tests"
  if [[ ! -d "$tests_directory" ]]; then
    # 模板自证拷来的假冻结副本没有运行库的测试目录：只验传参，不量。真跑时给错了 --frozen，下面 name=rerun_done 那一行不会出现。
    echo "name=rerun_skipped reason=frozen_copy_has_no_mount_runtime_tests"
    return 0
  fi
  mkdir -p "$WORK/out"
  cp "$MODEL/probes/mount_runtime_busy_window_foreground_wait.rs" "$tests_directory/"
  if [[ -n "${CARGO_TARGET_DIR:-}" && "$CARGO_TARGET_DIR" != /* ]]; then
    export CARGO_TARGET_DIR="$WORK/tree/$CARGO_TARGET_DIR"
  fi
  if ! (cd "$WORK/tree" && cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait --no-run) > "$WORK/out/build-busy.log" 2>&1; then
    tail -n 80 "$WORK/out/build-busy.log"
    return 1
  fi
  echo "name=build done"
  # 第二轮辩方腿的忙窗口基线探针（源文件原样拷来，不改）。补三样：替身卡每次调用睡 10、20 毫秒两档（真卡一次提交约 5.3 毫秒，
  # 一次调用里两份核各算一次、至少两次提交，约 10.6 毫秒，推的）；每个配置连跑三趟看趟与趟之间差多少；CPU 路径对照连跑三趟。
  # 读线程四条紧循环；写线程 0 与 1 两种。每一条 cargo 都写字面参数。
  local failures=0 expected_rows=0
  show_row() {   # show_row <标签>：打出这一趟的配置名与它那一行
    expected_rows=$((expected_rows + 1))
    echo "name=busy_config tag=$1"
    grep -h -o 'name=busy_window_foreground_wait .*' "$WORK/out/busy-$1.out" || { echo "  ✗ 没读到 $1 那一行（尾巴见 $WORK/out/busy-$1.out）"; tail -n 5 "$WORK/out/busy-$1.out"; failures=$((failures + 1)); }
  }
  (cd "$WORK/tree" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=0 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=0 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep0_readers4_writers0_trial1.out" 2>&1 || failures=$((failures + 1))
  show_row double_sleep0_readers4_writers0_trial1
  (cd "$WORK/tree" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=5 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=0 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep5_readers4_writers0_trial1.out" 2>&1 || failures=$((failures + 1))
  show_row double_sleep5_readers4_writers0_trial1
  (cd "$WORK/tree" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=10 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=0 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep10_readers4_writers0_trial1.out" 2>&1 || failures=$((failures + 1))
  show_row double_sleep10_readers4_writers0_trial1
  (cd "$WORK/tree" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=20 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=0 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep20_readers4_writers0_trial1.out" 2>&1 || failures=$((failures + 1))
  show_row double_sleep20_readers4_writers0_trial1
  (cd "$WORK/tree" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=50 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=0 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep50_readers4_writers0_trial1.out" 2>&1 || failures=$((failures + 1))
  show_row double_sleep50_readers4_writers0_trial1
  (cd "$WORK/tree" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=5 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=1 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep5_readers4_writers1_trial1.out" 2>&1 || failures=$((failures + 1))
  show_row double_sleep5_readers4_writers1_trial1
  (cd "$WORK/tree" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=10 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=1 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep10_readers4_writers1_trial1.out" 2>&1 || failures=$((failures + 1))
  show_row double_sleep10_readers4_writers1_trial1
  (cd "$WORK/tree" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=20 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=1 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep20_readers4_writers1_trial1.out" 2>&1 || failures=$((failures + 1))
  show_row double_sleep20_readers4_writers1_trial1
  (cd "$WORK/tree" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=50 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=1 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep50_readers4_writers1_trial1.out" 2>&1 || failures=$((failures + 1))
  show_row double_sleep50_readers4_writers1_trial1
  (cd "$WORK/tree" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=0 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=0 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_the_cpu_route --exact --nocapture --test-threads=1) > "$WORK/out/busy-cpu_route_readers4_writers0_trial1.out" 2>&1 || failures=$((failures + 1))
  show_row cpu_route_readers4_writers0_trial1
  (cd "$WORK/tree" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=0 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=0 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep0_readers4_writers0_trial2.out" 2>&1 || failures=$((failures + 1))
  show_row double_sleep0_readers4_writers0_trial2
  (cd "$WORK/tree" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=5 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=0 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep5_readers4_writers0_trial2.out" 2>&1 || failures=$((failures + 1))
  show_row double_sleep5_readers4_writers0_trial2
  (cd "$WORK/tree" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=10 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=0 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep10_readers4_writers0_trial2.out" 2>&1 || failures=$((failures + 1))
  show_row double_sleep10_readers4_writers0_trial2
  (cd "$WORK/tree" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=20 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=0 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep20_readers4_writers0_trial2.out" 2>&1 || failures=$((failures + 1))
  show_row double_sleep20_readers4_writers0_trial2
  (cd "$WORK/tree" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=50 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=0 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep50_readers4_writers0_trial2.out" 2>&1 || failures=$((failures + 1))
  show_row double_sleep50_readers4_writers0_trial2
  (cd "$WORK/tree" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=5 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=1 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep5_readers4_writers1_trial2.out" 2>&1 || failures=$((failures + 1))
  show_row double_sleep5_readers4_writers1_trial2
  (cd "$WORK/tree" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=10 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=1 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep10_readers4_writers1_trial2.out" 2>&1 || failures=$((failures + 1))
  show_row double_sleep10_readers4_writers1_trial2
  (cd "$WORK/tree" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=20 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=1 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep20_readers4_writers1_trial2.out" 2>&1 || failures=$((failures + 1))
  show_row double_sleep20_readers4_writers1_trial2
  (cd "$WORK/tree" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=50 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=1 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep50_readers4_writers1_trial2.out" 2>&1 || failures=$((failures + 1))
  show_row double_sleep50_readers4_writers1_trial2
  (cd "$WORK/tree" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=0 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=0 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_the_cpu_route --exact --nocapture --test-threads=1) > "$WORK/out/busy-cpu_route_readers4_writers0_trial2.out" 2>&1 || failures=$((failures + 1))
  show_row cpu_route_readers4_writers0_trial2
  (cd "$WORK/tree" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=0 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=0 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep0_readers4_writers0_trial3.out" 2>&1 || failures=$((failures + 1))
  show_row double_sleep0_readers4_writers0_trial3
  (cd "$WORK/tree" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=5 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=0 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep5_readers4_writers0_trial3.out" 2>&1 || failures=$((failures + 1))
  show_row double_sleep5_readers4_writers0_trial3
  (cd "$WORK/tree" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=10 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=0 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep10_readers4_writers0_trial3.out" 2>&1 || failures=$((failures + 1))
  show_row double_sleep10_readers4_writers0_trial3
  (cd "$WORK/tree" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=20 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=0 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep20_readers4_writers0_trial3.out" 2>&1 || failures=$((failures + 1))
  show_row double_sleep20_readers4_writers0_trial3
  (cd "$WORK/tree" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=50 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=0 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep50_readers4_writers0_trial3.out" 2>&1 || failures=$((failures + 1))
  show_row double_sleep50_readers4_writers0_trial3
  (cd "$WORK/tree" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=5 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=1 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep5_readers4_writers1_trial3.out" 2>&1 || failures=$((failures + 1))
  show_row double_sleep5_readers4_writers1_trial3
  (cd "$WORK/tree" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=10 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=1 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep10_readers4_writers1_trial3.out" 2>&1 || failures=$((failures + 1))
  show_row double_sleep10_readers4_writers1_trial3
  (cd "$WORK/tree" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=20 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=1 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep20_readers4_writers1_trial3.out" 2>&1 || failures=$((failures + 1))
  show_row double_sleep20_readers4_writers1_trial3
  (cd "$WORK/tree" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=50 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=1 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep50_readers4_writers1_trial3.out" 2>&1 || failures=$((failures + 1))
  show_row double_sleep50_readers4_writers1_trial3
  (cd "$WORK/tree" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=0 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=0 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_the_cpu_route --exact --nocapture --test-threads=1) > "$WORK/out/busy-cpu_route_readers4_writers0_trial3.out" 2>&1 || failures=$((failures + 1))
  show_row cpu_route_readers4_writers0_trial3
  # 判法自证：探针要分得出替身睡多久。睡 50 毫秒那三趟被挡取样最坏等待的中位数要大于睡 0 毫秒那三趟的；不大于，说明量到的不随 GPU 等变，数不算。
  median_of() {   # median_of <标签前缀>：三趟 blocked_wait_micros_max 的中位数
    { grep -h -o 'blocked_wait_micros_max=[0-9]*' "$WORK"/out/busy-"$1"_trial*.out || true; } | sed 's/.*=//' | sort -n | sed -n 2p
  }
  local median_sleep0 median_sleep50 rows
  median_sleep0="$(median_of double_sleep0_readers4_writers0)"
  median_sleep50="$(median_of double_sleep50_readers4_writers0)"
  echo "name=self_check median_blocked_max_sleep0=${median_sleep0:-none} median_blocked_max_sleep50=${median_sleep50:-none}"
  if [[ -z "$median_sleep0" || -z "$median_sleep50" || "$median_sleep50" -le "$median_sleep0" ]]; then
    echo "  ✗ 判法自证没过：睡 50 毫秒的被挡最坏等待中位数不大于睡 0 毫秒的（或没读到），探针分不出 GPU 等，这一趟的数不算"
    failures=$((failures + 1))
  fi
  rows="$({ grep -h -c '^name=busy_window_foreground_wait' "$WORK"/out/busy-*.out || true; } | paste -sd+ | bc)"
  echo "name=rerun_done busy_rows=$rows expected_rows=$expected_rows failures=$failures"
  [[ "$rows" -eq "$expected_rows" && "$failures" -eq 0 ]]
}

# 自证：模板拷进一个与仓无关的临时目录（<临时>/elsewhere/r1-opus-model/tools/rerun.sh），从另一个目录起它
selftest() {
  local scratch copy rc checked=0
  local -a failures=()
  # 删临时目录的 trap 在函数返回之后才触发，那时 local 已经没了：路径放进全局变量
  SELFTEST_SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/rerun-template-selftest.XXXXXX")"
  trap 'rm -rf "${SELFTEST_SCRATCH:?}"' EXIT
  scratch="$SELFTEST_SCRATCH"
  export RERUN_TEMPLATE_SELFTEST_MARK="rerun-template-selftest"   # 自证里拷出去起的子进程（环境原样继承）靠它认：弄坏开关只在自证里认
  mkdir -p "$scratch/fake-repo/.git" "$scratch/fake-repo/task-server" "$scratch/old-repo/.git" "$scratch/old-repo/scripts/hosts" "$scratch/frozen/crates/target" \
    "$scratch/elsewhere/r1-opus-model/tools" "$scratch/cwd"
  : > "$scratch/fake-repo/task-server/task"
  : > "$scratch/old-repo/scripts/hosts/run-with-memory-cap.sh"
  echo 冻结副本里的文件 > "$scratch/frozen/crates/marker.txt"
  echo 编译产物 > "$scratch/frozen/crates/target/junk"
  copy="$scratch/elsewhere/r1-opus-model/tools/rerun.sh"
  cp "${BASH_SOURCE[0]}" "$copy"

  checked=$((checked + 1))
  if (cd "$scratch/cwd" && bash "$copy" --repo "$scratch/fake-repo" --frozen "$scratch/frozen" --work "$scratch/work1") \
      > "$scratch/run1.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 0 ]] || ! grep -qF "name=rerun_roots repo=$scratch/fake-repo frozen=$scratch/frozen model=$scratch/elsewhere/r1-opus-model " "$scratch/run1.out" \
      || [[ ! -f "$scratch/work1/tree/crates/marker.txt" || -e "$scratch/work1/tree/crates/target" ]]; then
    failures+=("拷进与仓无关的临时目录、传参应当跑通（退 0、仓根取 --repo、冻结副本拷进草稿目录、target 不拷），实际退 $rc：$(tail -n 4 "$scratch/run1.out")")
  fi

  checked=$((checked + 1))
  if (cd "$scratch/cwd" && bash "$copy") > "$scratch/run2.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "用法：rerun.sh --repo" "$scratch/run2.out"; then
    failures+=("不传参应当退 2 并列出用法，实际退 $rc：$(tail -n 4 "$scratch/run2.out")")
  fi

  checked=$((checked + 1))
  if bash "$copy" --repo "$scratch/fake-repo" --work "$scratch/work3" > "$scratch/run3.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "缺 --frozen" "$scratch/run3.out"; then
    failures+=("缺 --frozen 应当退 2 并点名缺它，实际退 $rc：$(tail -n 4 "$scratch/run3.out")")
  fi

  checked=$((checked + 1))
  if bash "$copy" --repo "$scratch/cwd" --frozen "$scratch/frozen" --work "$scratch/work4" > "$scratch/run4.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "不像本仓" "$scratch/run4.out"; then
    failures+=("--repo 指到不是仓的目录应当退 2，实际退 $rc：$(tail -n 4 "$scratch/run4.out")")
  fi

  checked=$((checked + 1))
  if bash "$copy" --repo "$scratch/old-repo" --frozen "$scratch/frozen" --work "$scratch/work6" > "$scratch/run6.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "不像本仓" "$scratch/run6.out"; then
    failures+=("--repo 指到只有内存包装、没有 task-server/task 的旧仓应当退 2，实际退 $rc：$(tail -n 4 "$scratch/run6.out")")
  fi

  checked=$((checked + 1))
  if bash "$copy" --repo "$scratch/fake-repo" --frozen "$scratch/frozen" --work "$scratch/work1" > "$scratch/run5.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "已存在且非空" "$scratch/run5.out"; then
    failures+=("--work 指到已有输出的目录应当退 2、不在旧输出上接着写，实际退 $rc：$(tail -n 4 "$scratch/run5.out")")
  fi

  if ((${#failures[@]})); then
    local failure
    for failure in "${failures[@]}"; do
      echo "  ✗ 自检：$failure"   # gate-lint:detail
    done
    echo "    → 怎么办：看 parse_arguments() 与 rerun_steps()；RERUN_TEMPLATE_BREAK 设着的话这里本来就该红"
    return 1
  fi
  echo "  ✓ 自检通过（查了 $checked 格）：拷进与仓无关的临时目录、从别的目录起，传参跑通；不传参、缺 --frozen、--repo 不是仓、--repo 是没有 task-server/task 的旧仓、--work 非空都退 2 并说明"
  return 0
}

# 弄坏开关只在自证里认：命令行不是单独一个 --selftest、也不是自证起的子进程（RERUN_TEMPLATE_SELFTEST_MARK 对得上）而变量非空，参数解析之前就退 2（不管值是什么）。
if [[ -n "${RERUN_TEMPLATE_BREAK:-}" && ! ( "$#" -eq 1 && "${1:-}" == --selftest ) && "${RERUN_TEMPLATE_SELFTEST_MARK:-}" != rerun-template-selftest ]]; then
  echo "  ✗ RERUN_TEMPLATE_BREAK=${RERUN_TEMPLATE_BREAK} 是弄坏开关，只在自证（bash <这份脚本> --selftest）里认，平常跑设了就拒，这一趟什么都没做" >&2
  echo "     → 怎么办：去掉 RERUN_TEMPLATE_BREAK 再跑；要证红就跑 RERUN_TEMPLATE_BREAK=<项> bash scripts/agents/three-way-rerun-template.sh --selftest" >&2
  exit 2
fi
if [[ "${1:-}" == --selftest ]]; then
  if selftest; then exit 0; else exit 1; fi
fi
parse_arguments "$@"
rerun_steps
