#!/usr/bin/env bash
# admission: always 每次调都在这一刻的树上照服务给的那一步跑：分片时各片判自己那一份、直接写进 prune-server，合并时本机起核账那一趟，不分片时本机整份；上一次的结论不替这一次作保，块的复用在判定库（prune-server）里
# run-condition: command bash python3 rsync
# gate-similar: scripts/crash/crash-amplification-two-hosts.sh 不另起一套分片跑法：崩溃点重放的一条崩溃枚举用例也是崩溃放量的一条流（整条流一个崩溃点），这一份只把那一条用例交给它的同一步、再把核账那一趟（或不分片那一趟）的日志交回门禁给的日志文件
# 崩溃点重放一条崩溃枚举用例交崩溃放量的分片驱动 scripts/crash/crash-amplification-two-hosts.sh 跑（引擎读同样的 SHARE，判的块直接写进 prune-server，用例自己断言），
# 核账那一趟（或不分片时本机整份那一趟）的日志抄进给的日志文件，门禁照单机的判法判（scripts/gate/admission.py crash-case-judge）。
# 分不分片、哪一片去哪台机器、上限、线程、分卡与清场都归 task-server（records/2026-10-30-编译绕过内存总上限.md 收尾批 #5）；这一份不自己连第二台、不核第二台、不量速度、不排权重。
#
#   crash-point-replay-shard-run.sh --merged-log <崩溃枚举用例的键> <树根> <输入指纹> <日志文件> <测试目标> <用例函数>
#       键与指纹只进标签与输出，测试目标与用例函数交驱动的 <测试目标> 与 --test-filter。
#   crash-point-replay-shard-run.sh --selftest
#       拿假的分片驱动（CRASH_POINT_REPLAY_SHARD_RUN_TWO_HOSTS_DRIVER）走一遍：交的参数（不交库目录）、经 task-server 分片的三步各转给驱动的同一步、合并与不分片时日志抄回、
#       发现日志、退出码照驱动、编译目录跑完删；认不出的选项退 2；基线和开关轮每一轮各用一份自己的工作目录同时跑（并行度取每台默认重活线程数），
#       逐个打开弄坏开关核点名的格红。只跑假的那几格，不起 cargo（.claude/hooks/selftest-registry.tsv 登记 selftest）；两台真分判不在自证里，由门禁 crash-point-replay 不指定范围那一趟跑到（登记了 shard=across-machines 的用例经这一份交服务）。
#       退出码：0 全对；1 有格判错；
#       2 不在临时 prune-server 底下起（scripts/prune-server/prune_server_client_selftest.py --with-temporary-service 起；共用库 crash_amplification_selftest_service_problem 判），一格都不跑。
#
# 经 task-server 分片（task-server/task_server_lib.py 文件头「分片」，SHARDED_ENTRIES 登记了 --merged-log 这一路；
#   提交 task-server/task submit -- bash scripts/crash/crash-point-replay-shard-run.sh --merged-log …）：服务拆成计划、各片、合并几个子任务，
#   这一份照环境里的 TASK_SERVER_SHARD_PHASE 把同一组参数转给驱动（驱动从环境读同样的变量，只跑那一步；三步的做法在驱动文件头「经 task-server 分片」）：
#   plan、shard  只转给驱动，不碰日志文件；
#   merge        转给驱动之后，把 <放量家目录>/runs/<计划里的标签>/merge.log（驱动核账那一趟的日志；标签是计划那一步写进计划目录的 label，驱动的合并照它）抄进 <日志文件>。
#   环境里没有 TASK_SERVER_SHARD_PHASE（门禁 crash-point-replay 在服务起的一件里调到这里、或直接起）：驱动本机整份跑，
#   把 <放量家目录>/runs/<这一趟的标签>/single.log 抄进 <日志文件>。TASK_SERVER_SHARD_PHASE 是别的值时驱动退 2，这里照它退。
# 判定库：崩溃判定只在 prune-server 里那一份库（驱动文件头「判定库」）：不交库目录、不送库的变量，引擎经客户端直接读写。
#   放量家目录照共用库 crash_amplification_home_directory（SINGLEFS_CRASH_AMPLIFICATION_HOME，没设是工作根下的 crash-amplification：共享根里、两台同一份；只放运行目录与日志）。
# 编译目录（自己用完自己删，不跨趟复用）：除计划那一步外，建好 <放量家目录>/runs/<这一趟的标签>/ 之后，调用方没设 CARGO_TARGET_DIR 时
#   在里面建 target、export 给驱动；正常结束、驱动退非 0、被 TERM / INT 停（等手里那一步退出）都在 EXIT 上删
#   （scripts/hosts/run-build-directory.sh；删之前核本机没有进程开着里面的文件，有就不删、报出进程、退非 0）。调用方设了的照它、不删（经 task-server 起时编译目录由服务给）。
# 发现日志：合并与不分片的那一趟在环境里设 SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE=<日志文件>.findings.tsv 交驱动；只有判完整条流的那一趟写一节（各片只判自己领的块、不写，也不交）。
# 标签：crash-point-replay-<用例名>-<开跑时刻>-<进程号>；分片时各步照计划那一步写下的那一个。
# 退出码：驱动的退出码（含义见驱动文件头「退出码」）；2 用法错；1 驱动退 0 而该抄的那份日志不在（驱动没跑到那一步却退 0：不当绿）。
# 驱动跑 验证包用例：只在提交时或用户同意后由主 agent 经 task-server/task submit 交；--selftest 不起 cargo。
#
# 只供 --selftest（自证起的每一趟里层设）：
#   CRASH_POINT_REPLAY_SHARD_RUN_TWO_HOSTS_DRIVER=<脚本>   换掉分片驱动（调法相同：<测试目标> --test-filter … --label …，读同样的环境变量）
# 弄坏开关 CRASH_POINT_REPLAY_SHARD_RUN_BREAK=<项>（只在 --selftest 里认：命令行第一个参数不是 --selftest、又不是自证起的子进程（自证设 CRASH_POINT_REPLAY_SHARD_RUN_SELFTEST=1）时
# 设了它，在参数解析之前退 2、点名它只在自证里认；几个用逗号连）。逐项写开关名、弄坏哪一处，下一行起写打开后判红的格（格名是自证里 selftest_cells 的名字）：
#   CRASH_POINT_REPLAY_SHARD_RUN_BREAK=log-not-copied       合并与不分片的那一趟的日志不抄进日志文件
#       log-copied、phases-forwarded
#   CRASH_POINT_REPLAY_SHARD_RUN_BREAK=exit-swallowed       驱动退非 0 也退 0
#       driver-exit-kept、run-target-red
#   CRASH_POINT_REPLAY_SHARD_RUN_BREAK=merge-log-of-own-label  合并时照这一趟自己的标签找核账那一趟的日志（不读计划目录里的标签）
#       phases-forwarded
#   CRASH_POINT_REPLAY_SHARD_RUN_BREAK=findings-file-for-shards  各片也交发现日志（各片与合并那一趟都交）
#       phases-forwarded
#   CRASH_POINT_REPLAY_SHARD_RUN_BREAK=run-target-kept      跑完不删这一趟自己建的编译目录（驱动本机那几趟在树根照树根的 target 编、跨趟留着）
#       run-target-green、run-target-red、run-target-term
#   CRASH_POINT_REPLAY_SHARD_RUN_BREAK=unknown-option-accepted  认不出的第一个参数也退 0（不退 2）
#       unknown-option-refused
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
shard_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$shard_script_directory/crash-amplification-lib.sh"

shard_break_is() { [[ ",${CRASH_POINT_REPLAY_SHARD_RUN_BREAK:-}," == *",$1,"* ]]; }
source "$shard_script_directory/../hosts/run-build-directory.sh"

# 收尾（EXIT trap，--merged-log 那一趟装）：删这一趟自己建的编译目录（文件头「编译目录」），没删成而本来要退 0 的改退 1。弄坏开关 run-target-kept：不删
shard_finish() {
  local finish_exit=$?
  trap - EXIT
  if ! run_build_directory_end $(shard_break_is run-target-kept && echo keep); then
    (( finish_exit != 0 )) || finish_exit=1
  fi
  exit "$finish_exit"
}

usage_error() {
  echo "  ✗ 用法不对：$1" >&2
  echo "  → 怎么办：bash scripts/crash/crash-point-replay-shard-run.sh --merged-log <崩溃枚举用例的键> <树根> <输入指纹> <日志文件> <测试目标> <用例函数>，或 --selftest" >&2
  exit 2
}

# run_merged_log <键> <树根> <指纹> <日志> <测试目标> <用例函数>：照 TASK_SERVER_SHARD_PHASE 交驱动那一步，合并与不分片时把那一趟的日志抄进 <日志>；退出码照文件头「退出码」
run_merged_log() {
  local key="$1" root="$2" fingerprint="$3" log="$4" target="$5" function="$6"
  [[ -d "$root" ]] || usage_error "树根 $root 不在"
  [[ "$fingerprint" =~ ^[0-9a-f]{64}$ ]] || usage_error "输入指纹要 64 位十六进制：$fingerprint"
  [[ "$key" == crash-case:* && -n "$target" && -n "$function" ]] || usage_error "键要以 crash-case: 起头，测试目标与用例函数不许空"
  local phase="${TASK_SERVER_SHARD_PHASE:-}" amplification_home label driver driver_exit copied_log="" copied_label
  local -a findings=()
  amplification_home="$(crash_amplification_home_directory)" || return 2
  label="crash-point-replay-${key#crash-case:}-$(date -u +%Y%m%dT%H%M%SZ)-$$"
  # 这一趟的编译目录（文件头「编译目录」）：计划那一步不编、不建；收尾的 EXIT trap（shard_finish）删
  if [[ "$phase" != plan ]]; then
    run_build_directory_begin "$amplification_home/runs/$label" || return 78
  fi
  trap shard_finish EXIT
  trap 'exit 143' TERM
  trap 'exit 130' INT
  driver="${CRASH_POINT_REPLAY_SHARD_RUN_TWO_HOSTS_DRIVER:-$root/scripts/crash/crash-amplification-two-hosts.sh}"
  local -a driver_arguments=("$target" --test-filter "$function" --label "$label")
  # 抄哪一份日志（文件头「经 task-server 分片」）：合并照计划目录里的标签找 merge.log，不分片照这一趟的标签找 single.log；各片与计划那一步不抄
  case "$phase" in
    merge)
      copied_label="$label"
      if ! shard_break_is merge-log-of-own-label && [[ -s "${TASK_SERVER_SHARD_PLAN_DIRECTORY:-}/label" ]]; then copied_label="$(head -n 1 -- "$TASK_SERVER_SHARD_PLAN_DIRECTORY/label")"; fi
      copied_log="$amplification_home/runs/$copied_label/merge.log"
      findings=(SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE="$log.findings.tsv") ;;
    "")
      copied_log="$amplification_home/runs/$label/single.log"
      findings=(SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE="$log.findings.tsv") ;;
    *)
      # 弄坏开关 findings-file-for-shards：各片也交
      if [[ "$phase" == shard ]] && shard_break_is findings-file-for-shards; then findings=(SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE="$log.findings.tsv"); fi ;;
  esac
  echo "  · 崩溃点重放：$key（输入指纹 ${fingerprint:0:16}…）交崩溃放量的分片驱动（${phase:-不分片，本机整份}）：标签 $label，判定写进 prune-server"
  ( cd "$root" && env -u SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE "${findings[@]+"${findings[@]}"}" bash "$driver" "${driver_arguments[@]}" ); driver_exit=$?
  if [[ -n "$copied_log" ]]; then
    if [[ -f "$copied_log" ]]; then
      if ! shard_break_is log-not-copied; then
        cp -- "$copied_log" "$log"
        echo "  · 那一趟的日志抄到 $log（驱动那几份在 $(dirname -- "$copied_log")/）"
      fi
    elif [[ "$driver_exit" == 0 ]]; then
      echo "  ✗ 分片驱动退 0，该抄的那一趟日志 $copied_log 却不在：没跑到那一步，不当绿"
      echo "     → 怎么办：看 $(dirname -- "$copied_log")/ 里有哪几份日志、驱动的输出停在哪一步；修好之后重投。"
      return 1
    else
      : > "$log"
      echo "  · 分片驱动退 $driver_exit，该抄的那一趟日志不在（$copied_log）：日志文件留空，门禁按驱动的退出码判红"
    fi
  fi
  if shard_break_is exit-swallowed; then return 0; fi
  return "$driver_exit"
}

# ============================== 自证 ==============================

FAKE_TWO_HOSTS_DRIVER='#!/usr/bin/env bash
# 假的分片驱动：记下收到的参数、这一步（TASK_SERVER_SHARD_PHASE，没设记 single）、发现日志、编译目录；照这一步写驱动该写的东西，退码照 $FAKE_DRIVER_CONTROL/exit（没写退 0）
phase="${TASK_SERVER_SHARD_PHASE:-single}"
printf "%s\n" "$*" > "${FAKE_DRIVER_CONTROL:?}/arguments"
printf "%s\n" "$*" > "$FAKE_DRIVER_CONTROL/arguments-$phase"
printf "%s\n" "${SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE-unset}" > "$FAKE_DRIVER_CONTROL/findings-file-$phase"
printf "%s\n" "${CARGO_TARGET_DIR-unset}" > "$FAKE_DRIVER_CONTROL/target"
if [[ -n "${CARGO_TARGET_DIR:-}" ]]; then mkdir -p "$CARGO_TARGET_DIR/release" && echo built > "$CARGO_TARGET_DIR/release/artifact"; fi
if [[ -f "$FAKE_DRIVER_CONTROL/sleep" ]]; then sleep "$(cat "$FAKE_DRIVER_CONTROL/sleep")"; fi
label=""
while (($#)); do
  if [[ "$1" == --label ]]; then label="$2"; shift 2; else shift; fi
done
case "$phase" in
  plan) printf "%s\n" "$label" > "${TASK_SERVER_SHARD_PLAN_DIRECTORY:?}/label" ;;
  merge)
    planned="$(head -n 1 "${TASK_SERVER_SHARD_PLAN_DIRECTORY:?}/label")"
    if [[ ! -f "$FAKE_DRIVER_CONTROL/no-copied-log" ]]; then
      mkdir -p "${SINGLEFS_CRASH_AMPLIFICATION_HOME:?}/runs/$planned"
      printf "CRASH_POINT_REPLAY_PARALLEL_FINISHED states=29 slices=1 worker_threads=2\nCRASH_POINT_REPLAY states=29 exhaustive=true merged=yes\n" > "$SINGLEFS_CRASH_AMPLIFICATION_HOME/runs/$planned/merge.log"
    fi ;;
  single)
    if [[ ! -f "$FAKE_DRIVER_CONTROL/no-copied-log" ]]; then
      mkdir -p "${SINGLEFS_CRASH_AMPLIFICATION_HOME:?}/runs/$label"
      printf "CRASH_POINT_REPLAY_PARALLEL_FINISHED states=29 slices=1 worker_threads=2\nCRASH_POINT_REPLAY states=29 exhaustive=true\n" > "$SINGLEFS_CRASH_AMPLIFICATION_HOME/runs/$label/single.log"
    fi ;;
esac
exit "$(cat "$FAKE_DRIVER_CONTROL/exit" 2>/dev/null || echo 0)"
'
selftest_cells=(selftest-refused-outside-temporary-service unknown-option-refused driver-arguments log-copied driver-exit-kept no-copied-log-red findings-file phases-forwarded
  run-target-green run-target-red run-target-term run-target-caller)
selftest_zero_fingerprint="0000000000000000000000000000000000000000000000000000000000000000"

# 每格从空起：控制目录、库的家目录、树根、日志都重建
selftest_reset() {
  rm -rf -- "${selftest_work:?}/control" "${selftest_work:?}/home" "${selftest_work:?}/root" "${selftest_work:?}/plan"
  rm -f -- "${selftest_work:?}/log" "${selftest_work:?}/log.findings.tsv" "${selftest_work:?}/output"
  mkdir -p -- "$selftest_work/control" "$selftest_work/home" "$selftest_work/root" "$selftest_work/plan"
}

# selftest_run_wrapper <日志> [环境 键=值…]：起这一份 --merged-log，输出写 <自证目录>/output，退出码放进 selftest_exit（直接调，不在 $(…) 里）
selftest_exit=""
selftest_wrapper_environment=()
# selftest_build_wrapper_environment [环境 键=值…]：env 要的参数（先清掉会串味的变量、再设自证用的假驱动），放进全局数组 selftest_wrapper_environment
selftest_build_wrapper_environment() {
  selftest_wrapper_environment=(-u SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE -u CARGO_TARGET_DIR
    -u TASK_SERVER_SHARD_PHASE -u TASK_SERVER_SHARD -u TASK_SERVER_SHARD_PLAN_DIRECTORY -u TASK_SERVER_SHARD_OUTPUT_DIRECTORY -u TASK_SERVER_SHARD_OUTPUTS -u TASK_SERVER_SHARD_HOST
    "SINGLEFS_CRASH_AMPLIFICATION_HOME=$selftest_work/home" "FAKE_DRIVER_CONTROL=$selftest_work/control"
    "CRASH_POINT_REPLAY_SHARD_RUN_TWO_HOSTS_DRIVER=$selftest_work/fake-driver.sh" "$@")
}
selftest_run_wrapper() {
  local log="$1"
  shift
  selftest_build_wrapper_environment "$@"
  if env "${selftest_wrapper_environment[@]}" bash "$0" --merged-log crash-case:demo "$selftest_work/root" "$selftest_zero_fingerprint" "$log" demo_target demo_function >"$selftest_work/output" 2>&1; then
    selftest_exit=0
  else
    selftest_exit=$?
  fi
}

selftest_arguments() { cat "$selftest_work/control/arguments" 2>/dev/null; }
selftest_last_output() { cat "$selftest_work/output" 2>/dev/null; }

# 编译目录那几格（文件头「编译目录」）：调用方没设 CARGO_TARGET_DIR，假驱动收到的是 <家目录>/runs/crash-point-replay-demo-*/target，跑完不在了
selftest_run_target_gone() { # <要的退出码>
  local seen
  seen="$(cat "$selftest_work/control/target" 2>/dev/null)"
  [[ "$selftest_exit" == "$1" && "$seen" == "$selftest_work"/home/runs/crash-point-replay-demo-*/target && ! -e "$seen" ]]
}
cell_run_target_green() {
  selftest_reset
  selftest_run_wrapper "$selftest_work/log"
  selftest_run_target_gone 0
}
cell_run_target_red() {
  selftest_reset
  echo 1 > "$selftest_work/control/exit"
  selftest_run_wrapper "$selftest_work/log"
  selftest_run_target_gone 1
}
cell_run_target_term() {   # 假驱动睡 3 秒时只给这一份发 TERM：等驱动退出后走 EXIT、退 143
  local wrapper_pid attempt
  selftest_reset
  echo 3 > "$selftest_work/control/sleep"
  # 直接把 --merged-log 那一趟放后台（env 换成 bash，$! 就是它）：TERM 只发给自己起的这一个进程号，不从子进程里按命令行挑
  selftest_build_wrapper_environment
  env "${selftest_wrapper_environment[@]}" bash "$0" --merged-log crash-case:demo "$selftest_work/root" "$selftest_zero_fingerprint" "$selftest_work/log" demo_target demo_function \
    >"$selftest_work/output" 2>&1 &
  wrapper_pid=$!
  for (( attempt = 0; attempt < 600; attempt++ )); do [[ -s "$selftest_work/control/target" ]] && break; sleep 0.1; done
  kill -TERM "$wrapper_pid" 2>/dev/null
  if wait "$wrapper_pid"; then selftest_exit=0; else selftest_exit=$?; fi
  selftest_run_target_gone 143
}
cell_run_target_caller() {   # 调用方设了 CARGO_TARGET_DIR：假驱动照收到它，跑完不删
  selftest_reset
  mkdir -p "$selftest_work/caller-target"
  selftest_run_wrapper "$selftest_work/log" CARGO_TARGET_DIR="$selftest_work/caller-target"
  [[ "$selftest_exit" == 0 && "$(cat "$selftest_work/control/target" 2>/dev/null)" == "$selftest_work/caller-target" && -f "$selftest_work/caller-target/release/artifact" ]]
}
cell_selftest_refused_outside_temporary_service() {   # 摘掉临时 prune-server 的标记再起一份自证：退 2、说自证没跑，一格都不跑（文件头「用法」--selftest 的退出码 2）
  env -u PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET -u CRASH_POINT_REPLAY_SHARD_RUN_BREAK bash "$0" --selftest >"$selftest_work/bare-selftest.output" 2>&1
  [[ "$?" == 2 ]] && grep -q "crash-point-replay-shard-run.sh 自证没跑：" "$selftest_work/bare-selftest.output" && ! grep -q "开关轮" "$selftest_work/bare-selftest.output"
}
cell_unknown_option_refused() {   # 第一个参数不是 --merged-log 也不是 --selftest（用一个从没用过的名字）：退 2、点名正确用法、什么都不起（弄坏开关 unknown-option-accepted 下退 0）
  # 开关原样传给这一趟：子进程在自证的环境里（CRASH_POINT_REPLAY_SHARD_RUN_SELFTEST=1），不撞「开关只在自证里认」那一道，开关才到得了被测的那一支
  bash "$0" --option-never-used-before >"$selftest_work/unknown-option.output" 2>&1
  [[ "$?" == 2 ]] && grep -q "第一个参数要是 --merged-log 或 --selftest" "$selftest_work/unknown-option.output"
}
cell_driver_arguments() {   # 交给驱动的参数只有测试目标、点名的用例与标签（判定库在 prune-server 里，不交库目录）
  selftest_reset
  selftest_run_wrapper "$selftest_work/log"
  [[ "$selftest_exit" == 0 && "$(selftest_arguments)" =~ ^demo_target\ --test-filter\ demo_function\ --label\ crash-point-replay-demo-[0-9TZ]+-[0-9]+$ ]]
}
cell_log_copied() {   # 不分片：本机整份那一趟的日志抄进日志文件
  selftest_reset
  selftest_run_wrapper "$selftest_work/log"
  [[ "$selftest_exit" == 0 && -f "$selftest_work/log" && "$(cat "$selftest_work/log")" == *"CRASH_POINT_REPLAY states=29 exhaustive=true"* ]]
}
cell_driver_exit_kept() {
  selftest_reset
  echo 1 > "$selftest_work/control/exit"
  selftest_run_wrapper "$selftest_work/log"
  [[ "$selftest_exit" == 1 ]]
}
cell_no_copied_log_red() {   # 驱动退 0 而该抄的那一趟日志不在：退 1、说不当绿
  selftest_reset
  : > "$selftest_work/control/no-copied-log"
  selftest_run_wrapper "$selftest_work/log"
  [[ "$selftest_exit" == 1 && "$(selftest_last_output)" == *"不当绿"* ]]
}
cell_findings_file() {   # 不分片：发现日志交给驱动
  selftest_reset
  selftest_run_wrapper "$selftest_work/log"
  [[ "$selftest_exit" == 0 && "$(cat "$selftest_work/control/findings-file-single" 2>/dev/null)" == "$selftest_work/log.findings.tsv" ]]
}
# 经 task-server 分片（文件头「经 task-server 分片」）：照服务起的三步逐个起这一份——计划、一片、合并，每一步都把同一组参数转给驱动、驱动看到的是同一步；
#   计划那一步驱动写下它的标签；片那一步不交发现日志、不碰日志文件；合并照计划里的标签（不是合并这一趟自己的）把 merge.log 抄进日志文件、交发现日志
cell_phases_forwarded() {
  local plan="$selftest_work/plan" planned
  selftest_reset
  selftest_run_wrapper "$selftest_work/log" TASK_SERVER_SHARD_PHASE=plan TASK_SERVER_SHARD_PLAN_DIRECTORY="$plan"
  [[ "$selftest_exit" == 0 && -s "$plan/label" && "$(cat "$selftest_work/control/arguments-plan" 2>/dev/null)" == "demo_target --test-filter demo_function --label crash-point-replay-demo-"* ]] || return 1
  planned="$(cat "$plan/label")"
  selftest_run_wrapper "$selftest_work/log" TASK_SERVER_SHARD_PHASE=shard TASK_SERVER_SHARD=1/2 TASK_SERVER_SHARD_HOST=peer TASK_SERVER_SHARD_PLAN_DIRECTORY="$plan" \
    TASK_SERVER_SHARD_OUTPUT_DIRECTORY="$selftest_work/output-1"
  [[ "$selftest_exit" == 0 && ! -e "$selftest_work/log" && "$(cat "$selftest_work/control/findings-file-shard" 2>/dev/null)" == unset ]] || return 1
  # 合并这一趟自己的标签带它自己的进程号，与计划里那一个不同：照自己的标签找就找不到计划里那一份
  selftest_run_wrapper "$selftest_work/log" TASK_SERVER_SHARD_PHASE=merge TASK_SERVER_SHARD_PLAN_DIRECTORY="$plan" TASK_SERVER_SHARD_OUTPUTS="$selftest_work/outputs.tsv"
  [[ "$selftest_exit" == 0 && "$(cat "$selftest_work/log" 2>/dev/null)" == *"merged=yes"* && -f "$selftest_work/home/runs/$planned/merge.log" \
     && "$(cat "$selftest_work/control/findings-file-merge" 2>/dev/null)" == "$selftest_work/log.findings.tsv" ]]
}

# run_selftest_cells：跑全部格，红的格名打在 stdout（空格隔开）
run_selftest_cells() {
  local cell red=()
  for cell in "${selftest_cells[@]}"; do
    "cell_${cell//-/_}" || red+=("$cell")
  done
  echo "${red[*]+"${red[*]}"}"
}

# 弄坏开关只在自证里认（文件头「弄坏开关」）：命令行第一个参数不是 --selftest、又不是自证起的子进程（CRASH_POINT_REPLAY_SHARD_RUN_SELFTEST=1，只有 --selftest 在自己的环境里设）
# 而 CRASH_POINT_REPLAY_SHARD_RUN_BREAK 非空（不管值是什么），退 2 并点名它只在自证里认；在参数解析之前判，一步没做
if [[ "${1:-}" != --selftest && "${CRASH_POINT_REPLAY_SHARD_RUN_SELFTEST:-}" != 1 && -n "${CRASH_POINT_REPLAY_SHARD_RUN_BREAK:-}" ]]; then
  echo "  ✗ CRASH_POINT_REPLAY_SHARD_RUN_BREAK 只在自证里认（弄坏开关只给 --selftest 证红用），这一趟不是自证，一步都没做" >&2
  echo "  → 怎么办：去掉 CRASH_POINT_REPLAY_SHARD_RUN_BREAK 再跑；要证红就跑 CRASH_POINT_REPLAY_SHARD_RUN_BREAK=<项> bash $0 --selftest" >&2
  exit 2
fi

if [[ "${1:-}" == --selftest ]]; then
  [[ $# -eq 1 ]] || usage_error "--selftest 不带别的参数"
  # 只在临时 prune-server 上跑（文件头「用法」--selftest 的退出码 2），不是就退 2、一格都不跑
  selftest_service_problem_said="$(crash_amplification_selftest_service_problem)"
  if [[ -n "$selftest_service_problem_said" ]]; then
    echo "  ✗ crash-point-replay-shard-run.sh 自证没跑：$selftest_service_problem_said" >&2
    echo "     → 怎么办：照 scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service 那一条，在临时服务底下起这一份自证（带 --server-binary 就不编）" >&2
    exit 2
  fi
  export CRASH_POINT_REPLAY_SHARD_RUN_SELFTEST=1   # 自证起的每一趟里层（--merged-log 与开关轮）凭它认弄坏开关
  selftest_work="$(mktemp -d)"
  trap 'rm -rf -- "${selftest_work:?}"' EXIT
  selftest_base="$selftest_work"
  # selftest_prepare_work <目录>：把这一份（基线与每个开关轮各一份）自己的工作目录建好——假驱动；selftest_work 设成这个目录，之后的格都在里面做
  selftest_prepare_work() {
    selftest_work="$1"
    mkdir -p -- "$selftest_work/root"
    printf '%s' "$FAKE_TWO_HOSTS_DRIVER" > "$selftest_work/fake-driver.sh"
    chmod +x "$selftest_work/fake-driver.sh"
  }
  selftest_prepare_work "$selftest_base"
  selftest_failed=0
  # 开关轮每一轮一份自己的工作目录，同时跑（互不共用状态）；并行度取每台默认重活线程数（scripts/hosts/heavy-threads.sh），取不到取 4。
  # 一轮一行：说明、环境赋值（空格隔开）、该红的格、判错时的出路
  round_labels=() round_environments=() round_expected=() round_howto=()
  add_selftest_round() { round_labels+=("$1"); round_environments+=("$2"); round_expected+=("$3"); round_howto+=("$4"); }
  add_selftest_round "基线（不设开关）" "CRASH_POINT_REPLAY_SHARD_RUN_BREAK=" "" "对着文件头「用法」「经 task-server 分片」与这几格的判法看交给驱动的参数、日志抄回、退出码哪一样变了。"
  declare -A switch_cells=([log-not-copied]="log-copied phases-forwarded" [exit-swallowed]="driver-exit-kept run-target-red"
    [merge-log-of-own-label]="phases-forwarded" [findings-file-for-shards]="phases-forwarded"
    [unknown-option-accepted]="unknown-option-refused" [run-target-kept]="run-target-green run-target-red run-target-term")
  for switch in log-not-copied exit-swallowed merge-log-of-own-label findings-file-for-shards unknown-option-accepted run-target-kept; do
    add_selftest_round "CRASH_POINT_REPLAY_SHARD_RUN_BREAK=$switch" "CRASH_POINT_REPLAY_SHARD_RUN_BREAK=$switch" "${switch_cells[$switch]}" \
      "文件头「弄坏开关」那一张表与 run_merged_log 里 shard_break_is 的分支对一遍。"
  done
  # selftest_round <序号> <环境赋值，空格隔开>：在这一轮自己的目录里重建假的东西、设上开关、跑全部格，红的格名单写 round-<序号>.red
  selftest_round() {
    local assignment
    selftest_prepare_work "$selftest_base/round-$1"
    for assignment in $2; do export "$assignment"; done
    run_selftest_cells > "$selftest_base/round-$1.red"
  }
  round_count="${#round_labels[@]}"
  round_parallel="$( (source "$shard_script_directory/../hosts/heavy-threads.sh" && heavy_thread_default) 2>/dev/null)"
  [[ "$round_parallel" =~ ^[1-9][0-9]*$ ]] || round_parallel=4
  for (( round_start = 0; round_start < round_count; round_start += round_parallel )); do
    for (( round_index = round_start; round_index < round_count && round_index < round_start + round_parallel; round_index++ )); do
      rm -f -- "$selftest_base/round-$round_index.exit"
      { selftest_round "$round_index" "${round_environments[round_index]}"; echo "$?" > "$selftest_base/round-$round_index.exit"; } &
    done
    wait   # shell-lint:exit-collected 退出码在各轮自己的 round-<序号>.exit 里，下面按次序逐个回读、数份数
  done
  collected_rounds=0
  for (( round_index = 0; round_index < round_count; round_index++ )); do
    if [[ "$(cat "$selftest_base/round-$round_index.exit" 2>/dev/null)" != 0 || ! -f "$selftest_base/round-$round_index.red" ]]; then
      echo "  ✗ 自证第 $round_index 轮（${round_labels[round_index]}）没跑完（退出码文件「$(cat "$selftest_base/round-$round_index.exit" 2>/dev/null)」）"
      echo "  → 怎么办：这一轮的工作目录在自证的临时目录里随退出删掉；单跑 ${BASH_SOURCE[0]} --selftest 看有没有别的输出，磁盘与进程数是不是满了。"
      selftest_failed=1
      continue
    fi
    collected_rounds=$((collected_rounds + 1))
    red_cells="$(cat "$selftest_base/round-$round_index.red")"
    if (( round_index == 0 )); then
      if [[ -n "$red_cells" ]]; then
        echo "  ✗ crash-point-replay-shard-run 自证：这几格红：$red_cells（最后一趟的输出：$(cat "$selftest_base/round-$round_index/output" 2>/dev/null))"
        echo "  → 怎么办：${round_howto[round_index]}"
        selftest_failed=1
      fi
      continue
    fi
    echo "  · 开关轮：${round_labels[round_index]} 红的格：${red_cells:-（无）}"
    if [[ "$red_cells" != "${round_expected[round_index]}" ]]; then
      echo "  ✗ 开关轮：打开 ${round_labels[round_index]} 该红的是「${round_expected[round_index]}」，实际红的是「$red_cells」"
      echo "  → 怎么办：${round_howto[round_index]}"
      selftest_failed=1
    fi
  done
  if (( collected_rounds != round_count )); then selftest_failed=1; fi
  switch_count=$(( round_count - 1 ))
  if (( selftest_failed )); then exit 1; fi
  light_summary="${#selftest_cells[@]} 格绿（不在临时 prune-server 底下起的自证退 2；认不出的选项退 2；假的分片驱动：参数（不交库目录）、不分片时日志抄回与发现日志、经 task-server 分片的三步各转给驱动的同一步（片不交发现日志、合并照计划里的标签抄 merge.log）、驱动退 0 而日志不在退 1、退出码照驱动、编译目录跑完删），开关轮 ${switch_count} 个开关各红在点名的格"
  echo "  ✓ crash-point-replay-shard-run 自证通过：$light_summary"
  exit 0
fi

if [[ "${1:-}" != --merged-log ]]; then
  shard_break_is unknown-option-accepted && exit 0   # 弄坏开关 unknown-option-accepted：认不出的选项也退 0
  usage_error "第一个参数要是 --merged-log 或 --selftest（给的是「${1:-}」）"
fi
[[ $# -eq 7 ]] || usage_error "--merged-log 后面要六个参数：键、树根、输入指纹、日志文件、测试目标、用例函数（给了 $(( $# - 1 )) 个）"
run_merged_log "$2" "$3" "$4" "$5" "$6" "$7"
exit $?
