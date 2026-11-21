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

# ── 这一腿要跑的步骤（设计三方 m4-gpu-wait-outside-the-lock-design-r2 辩方腿）──
# 冻结副本拷进 $WORK/copy，拼进探针与运行库的测试目标（tools/patch_the_copy.py，核心层与运行库一行不改），
# 一、`stale-base-kinds`：旧底上整理 × 前台改动种类（编 release 探针包，打出 RESULT 各行，判法自证不过退 1）；
# 二、忙窗口基线：运行库测试目标 mount_runtime_busy_window_foreground_wait 的六种配置（读线程数、写线程、替身卡睡多久），各打一行 name=busy_window_foreground_wait。
# 冻结副本要带运行库的 tests/common_pool_on_memory_devices（`scripts/hosts/minimal-copy.py <仓根> <副本> --with-test-inputs` 建的有）。
rerun_steps() {
  mkdir -p "$WORK/copy" "$WORK/out"
  rsync -a --exclude target --exclude .git "$FROZEN/" "$WORK/copy/"
  python3 "$MODEL/tools/patch_the_copy.py" "$WORK/copy" "$MODEL" > "$WORK/out/patch.out" 2>&1 || { cat "$WORK/out/patch.out"; return 1; }
  cat "$WORK/out/patch.out"
  # 编译目录：服务给了 CARGO_TARGET_DIR 就用它（相对路径按拷贝的根算），没给用拷贝里的 target。
  if [[ -n "${CARGO_TARGET_DIR:-}" && "$CARGO_TARGET_DIR" != /* ]]; then
    export CARGO_TARGET_DIR="$WORK/copy/$CARGO_TARGET_DIR"
  fi
  if ! (cd "$WORK/copy" && cargo build --offline --release -p gpu-wait-outside-the-lock-r2-probe) > "$WORK/out/build-kinds.log" 2>&1; then
    tail -n 80 "$WORK/out/build-kinds.log"
    return 1
  fi
  local binary="${CARGO_TARGET_DIR:-$WORK/copy/target}/release/gpu-wait-outside-the-lock-r2-probe"
  echo "name=build done binary=$binary"
  local rc_kinds=0
  "$binary" stale-base-kinds > "$WORK/out/stale-base-kinds.out" 2>&1 || rc_kinds=$?
  echo "name=exit_codes stale_base_kinds=$rc_kinds"
  grep -h -e "^RESULT" "$WORK/out/stale-base-kinds.out" || true
  if [[ "$rc_kinds" -ne 0 ]]; then
    echo "  ✗ stale-base-kinds 没跑完（退出码见上一行）"
    return 1
  fi
  local field kind
  field() {   # field <种类> <臂> <名>：那一格 RESULT 行里 <名>= 后面的数，没有就空
    grep -o "^RESULT name=stale_base_kinds kind=$1 arm=$2 .*" "$WORK/out/stale-base-kinds.out" | grep -o " $3=[0-9]*" | sed "s/.*=//"
  }
  for kind in overwrite_unit_of_the_first_file sequential_write_of_the_second_inode new_inodes; do
    echo "name=kind_check kind=$kind planted_lost=$(field "$kind" planted_rollback lost) planted_histories=$(field "$kind" planted_rollback histories) txg_guard_lost=$(field "$kind" txg_guard lost) stale_weak_lost=$(field "$kind" stale_weak lost) stale_weak_publish_made=$(field "$kind" stale_weak stale_publish_made) stale_weak_refused_by_the_release_check=$(field "$kind" stale_weak refused_by_the_release_check)"
    if [[ "$(field "$kind" txg_guard lost)" != 0 ]]; then
      echo "  ✗ 对照臂 txg_guard 在 $kind 上读回的与 current_base 不同：探针本身有错，数不算"
      return 1
    fi
  done
  local planted_first planted_first_histories
  planted_first="$(field overwrite_unit_of_the_first_file planted_rollback lost)"
  planted_first_histories="$(field overwrite_unit_of_the_first_file planted_rollback histories)"
  if [[ -z "$planted_first" || "$planted_first" != "$planted_first_histories" || "$planted_first" -eq 0 ]]; then
    echo "  ✗ 判法自证没红：覆盖写那一种上盘换回发之前那一份只读出 ${planted_first:-没读到}/${planted_first_histories:-没读到} 段丢了（该相等且大于 0），stale-base-kinds 的数不算"
    return 1
  fi
  if ! (cd "$WORK/copy" && cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait --no-run) > "$WORK/out/build-busy.log" 2>&1; then
    tail -n 80 "$WORK/out/build-busy.log"
    return 1
  fi
  local rc_busy=0
  echo "name=busy_config tag=reproduce_first_round"
  (cd "$WORK/copy" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=5 PROBE_FOREGROUND_READERS=1 PROBE_SAMPLE_GAP_MICROSECONDS=200 PROBE_FOREGROUND_WRITERS=0 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-reproduce_first_round.out" 2>&1 || rc_busy=$?
  grep -h '^name=busy_window_foreground_wait' "$WORK/out/busy-reproduce_first_round.out" || echo "  ✗ 没读到 reproduce_first_round 那一行"
  echo "name=busy_config tag=double_sleep0_readers4_tight"
  (cd "$WORK/copy" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=0 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=0 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep0_readers4_tight.out" 2>&1 || rc_busy=$?
  grep -h '^name=busy_window_foreground_wait' "$WORK/out/busy-double_sleep0_readers4_tight.out" || echo "  ✗ 没读到 double_sleep0_readers4_tight 那一行"
  echo "name=busy_config tag=double_sleep5_readers4_tight"
  (cd "$WORK/copy" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=5 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=0 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep5_readers4_tight.out" 2>&1 || rc_busy=$?
  grep -h '^name=busy_window_foreground_wait' "$WORK/out/busy-double_sleep5_readers4_tight.out" || echo "  ✗ 没读到 double_sleep5_readers4_tight 那一行"
  echo "name=busy_config tag=double_sleep50_readers4_tight"
  (cd "$WORK/copy" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=50 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=0 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep50_readers4_tight.out" 2>&1 || rc_busy=$?
  grep -h '^name=busy_window_foreground_wait' "$WORK/out/busy-double_sleep50_readers4_tight.out" || echo "  ✗ 没读到 double_sleep50_readers4_tight 那一行"
  echo "name=busy_config tag=double_sleep5_readers4_writer1"
  (cd "$WORK/copy" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=5 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=1 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep5_readers4_writer1.out" 2>&1 || rc_busy=$?
  grep -h '^name=busy_window_foreground_wait' "$WORK/out/busy-double_sleep5_readers4_writer1.out" || echo "  ✗ 没读到 double_sleep5_readers4_writer1 那一行"
  echo "name=busy_config tag=double_sleep50_readers4_writer1"
  (cd "$WORK/copy" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_GPU_SLEEP_MILLISECONDS=50 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=1 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_a_sleeping_gpu_double --exact --nocapture --test-threads=1) > "$WORK/out/busy-double_sleep50_readers4_writer1.out" 2>&1 || rc_busy=$?
  grep -h '^name=busy_window_foreground_wait' "$WORK/out/busy-double_sleep50_readers4_writer1.out" || echo "  ✗ 没读到 double_sleep50_readers4_writer1 那一行"
  echo "name=busy_config tag=cpu_route_readers4_tight"
  (cd "$WORK/copy" && env PROBE_REPEATS=6 PROBE_WINDOW_MILLISECONDS=3000 PROBE_FOREGROUND_READERS=4 PROBE_SAMPLE_GAP_MICROSECONDS=0 PROBE_FOREGROUND_WRITERS=0 cargo test --offline -p singlefs-mount-runtime --test mount_runtime_busy_window_foreground_wait -- busy_window_on_the_cpu_route --exact --nocapture --test-threads=1) > "$WORK/out/busy-cpu_route_readers4_tight.out" 2>&1 || rc_busy=$?
  grep -h '^name=busy_window_foreground_wait' "$WORK/out/busy-cpu_route_readers4_tight.out" || echo "  ✗ 没读到 cpu_route_readers4_tight 那一行"
  if [[ "$rc_busy" -ne 0 ]]; then
    echo "  ✗ 忙窗口基线至少有一趟测试没过（退出码 $rc_busy，日志在 $WORK/out/busy-*.out）"
    return 1
  fi
  echo "name=rerun_done work=$WORK model=$MODEL"
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
