#!/usr/bin/env bash
# admission: always 每次调都在这一刻的树上两台各跑一份、取回、本机带导入再跑一趟，上一次的结论不替这一次作保；块的复用在判定库里
# run-condition: command bash python3
# gate-similar: scripts/crash/crash-amplification-two-hosts.sh 不另起一套双机跑法：崩溃点重放的一条崩溃枚举用例也是崩溃放量的一条流（整条流一个崩溃点），这一份只把门禁 checker-tier-crash-point-replay 交的那一条用例交给它、再把导入那一趟的日志交回门禁
# 崩溃点重放按双机跑一条崩溃枚举用例：交崩溃放量的双机驱动 scripts/crash/crash-amplification-two-hosts.sh（本机 SINGLEFS_CRASH_AMPLIFICATION_SHARE=0/2、
# 第二台 1/2 各写各的库，第二台的库取回，本机再带 SINGLEFS_CRASH_AMPLIFICATION_IMPORT=<取回的库> 跑一趟 0/2：导入第二台判的块、从库重建整条流的计数、
# 用例自己断言）；导入那一趟的日志抄进门禁给的日志文件，门禁照单机的判法判（scripts/gate/admission.py crash-case-judge）、写同一格全绿标记。
#
#   crash-point-replay-shard-run.sh --merged-log <崩溃枚举用例的键> <树根> <输入指纹> <日志文件> <测试目标> <用例函数>
#       门禁 checker-tier-crash-point-replay --full 在双机配置开着时调（scripts/crash/crash-point-replay-shard-configuration-check.sh 判得过）；键与指纹只进标签与输出，
#       测试目标与用例函数交驱动的 <测试目标> 与 --test-filter。
#   crash-point-replay-shard-run.sh --selftest
#       拿假的双机驱动（CRASH_POINT_REPLAY_SHARD_RUN_TWO_HOSTS_DRIVER）走一遍：交的参数、库目录、--quiesce、日志抄回、退出码照驱动；末尾开关轮逐个打开弄坏开关核点名的格红。
#       命令里设了 SINGLEFS_HEAVY_TESTS（非空）时，假的那几格与开关轮都对了再跑一格真跑 real-two-hosts-run（下面「真跑那一格」）；没设时这一格报本次未跑，不算绿。
#       退出码：0 全对，真跑那一格本次未跑（没设变量；或设了变量而双机没开、驱动退 78）也是 0，输出里写明、报进 GATE_NOT_RUN_FILE；1 有格判错。
#
# 真跑那一格（real-two-hosts-run）：经真驱动（本仓的 crash-amplification-two-hosts.sh，不换）跑 checker 档一条标 ignore 的崩溃枚举用例，
#   起法与门禁 checker-tier-crash-point-replay --full 双机那一路相同（本脚本 --merged-log）。用例是 crash-case:new-pool-file-creation-stream：登记了 shard=across-machines 的四条里态数最小的
#   （闭式 16777260 态；另三条 multi-record-publish-stream 1358954634、unlink-after-the-multi-record-publish-stream 1463812278、overwrite-fixed-script-stream 1678377204，
#   各取自测试文件里断言的 closed_form；没登记 shard 的那几条不走双机驱动的两段与导入）。测试目标与用例函数从 .claude/gate.d/stage-inputs.tsv 那一行的 test= 现读。
#   库的家目录 SINGLEFS_CRASH_AMPLIFICATION_HOME 指到自证的临时目录、带 SINGLEFS_CRASH_AMPLIFICATION_LIBRARY_IS_TEMPORARY=1（判定库拒绝落在 /tmp 下，这一份是一次性的）；
#   多机配置是这台机器那一份（crash-amplification-lib.sh 的 crash_amplification_multi_host_configuration_file）拷进临时目录、ENABLE_GPU 改成 0、去掉显卡先后那两个键
#   （不上卡、不交 --quiesce 清场）；双机没开（ENABLE_ACROSS_MACHINES 不是 1）这一格本次未跑。本机编译目录
#   CARGO_TARGET_DIR=${GATE_CROSS_RUN_TMPDIR:-${TMPDIR:-/tmp}}/crash-point-replay-shard-run-selftest-target（跨轮复用，不删）；第二台的照驱动（树副本上一层的 crash-amplification-target）。
#   判：包装退 0；runs/<标签>/record.log 里有 CRASH_LIBRARY_RECORDED class=crash_state_enumeration … library=…/crash-state-enumeration-library（录入段真跑、那一行真打）；
#   抄回的日志（导入那一趟）里 CRASH_POINT_REPLAY_PARALLEL_START 带 imported_blocks=<正数> shard=0/2、有一行不带 blocks_left_to_other_hosts= 的 CRASH_POINT_REPLAY_PARALLEL_FINISHED（导入之后对账为空）、
#   CRASH_POINT_REPLAY 计数行 states 等于 closed_form、violations=0、exhaustive=true。驱动退 78（运行条件：第二台连不上这一类）记本次未跑。判错时 runs/<标签>/ 下的日志拷到
#   ${TMPDIR:-/tmp}/crash-point-replay-shard-run-selftest-logs-<进程号>/ 留着、打出路径。
#   必红的样本：CRASH_AMPLIFICATION_ITEM_BREAK=include-ignored-dropped（录入段一条用例都不跑，驱动退 1）、library-marker-not-accepted（录入行不认，驱动退 1），各红在这一格。
#   驱动本机那几趟经 run-with-memory-cap.sh 起、上限是驱动的默认 38G：整条套在 run-with-memory-cap.sh 里跑时外层上限要不小于它（里层比外层大，包装退 251），
#   所以套在外层里时按下面「内存包装」一段把外层的上限交给驱动。
#
# 库目录：SINGLEFS_CRASH_AMPLIFICATION_LIBRARY 设了（门禁 --start-over 时是那一份开跑前清空的库）交驱动的 --library；没设交
#   <库的家目录>/crash-state-enumeration-library（崩溃枚举用例那一类的长期库，crates/singlefs-checker-tier/src/crash_library.rs；家目录照 SINGLEFS_CRASH_AMPLIFICATION_HOME，
#   没设是 ~/.local/share/singlefs/crash-amplification）。第二台那一份库由驱动交相对路径、落在树副本里再取回（崩溃枚举用例那一类认库目录变量）。
# 内存包装：套在 scripts/hosts/run-with-memory-cap.sh 里跑时（门禁 --full 整条套在 16G 里），包装给命令设了 RUN_WITH_MEMORY_CAP_OUTER_CAP=<外层的上限>
#   （run-with-memory-cap.sh 文件头「套在包装里」）；这一趟把它照原样交驱动的 --memory-cap（驱动本机那几趟与没给 --peer-memory-cap 时的第二台都用它）：
#   里层上限比外层大，包装按用法错退 251、命令没跑。没设、写法不认得的不交（驱动用自己的默认 38G）。
# 线程：SINGLEFS_CRASH_POINT_REPLAY_THREADS 设了交 --threads（本机那两趟的核对线程数；用例先读 SINGLEFS_CRASH_POINT_REPLAY_THREADS、再读 SINGLEFS_CRASH_AMPLIFICATION_THREADS）；第二台的由驱动现算。
# GPU：多机配置（${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}）写了 ENABLE_GPU=1 交 --quiesce（驱动照配置清场、跑完不复原、两台带 gpu 特性编）。
# 发现日志：环境里设 SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE=<日志文件>.findings.tsv 带给驱动的本机那两趟；只有导入那一趟判完整条流、写一节（0/2 那一趟只判自己领的块、不写）。
# 日志：驱动那几份在 <库的家目录>/runs/<标签>/（local.log、第二台那几份、import.log），标签 crash-point-replay-<用例名>-<开跑时刻>-<进程号>；跑完不删。
# 退出码：驱动的退出码（导入那一趟 cargo test 的退出码；77、78、248、249、250–254 的含义见驱动文件头）；2 用法错；
#   1 驱动退 0 而导入那一趟的日志不在（驱动没跑到那一步却退 0：不当绿）。
# 重型：驱动跑 checker 档用例，只在提交时或用户要求时跑（SINGLEFS_HEAVY_TESTS 由调用方带着）；--selftest 没设 SINGLEFS_HEAVY_TESTS 时不起 cargo，不算重型，
#   设了就起真跑那一格（.claude/hooks/selftest-registry.tsv 登记 heavy-if-set:SINGLEFS_HEAVY_TESTS）。
#
# 只供 --selftest（自证起的每一趟里层设）：
#   CRASH_POINT_REPLAY_SHARD_RUN_TWO_HOSTS_DRIVER=<脚本>   换掉双机驱动（调法相同：<测试目标> --test-filter … --label … [--library …] [--threads …] [--quiesce]）
# 弄坏开关 CRASH_POINT_REPLAY_SHARD_RUN_BREAK=<项>（只给 --selftest 证红；几个用逗号连）。一项一行，下一行起写打开后判红的格（格名是自证里 selftest_cells 的名字）：
#   library-not-passed   库目录不交成 --library
#       library-default、library-from-environment
#   log-not-copied       导入那一趟的日志不抄进日志文件
#       log-copied
#   exit-swallowed       驱动退非 0 也退 0
#       driver-exit-kept
#   quiesce-not-passed   ENABLE_GPU=1 也不交 --quiesce
#       quiesce-when-gpu
#   ignore-outer-cap     套在外层内存包装里也不把外层的上限交成 --memory-cap（改前的样子：驱动用自己的默认 38G，大过外层 16G，退 251）
#       outer-memory-cap-to-driver
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
shard_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$shard_script_directory/crash-amplification-lib.sh"

shard_break_is() { [[ ",${CRASH_POINT_REPLAY_SHARD_RUN_BREAK:-}," == *",$1,"* ]]; }

usage_error() {
  echo "  ✗ 用法不对：$1" >&2
  echo "  → 怎么办：bash scripts/crash/crash-point-replay-shard-run.sh --merged-log <崩溃枚举用例的键> <树根> <输入指纹> <日志文件> <测试目标> <用例函数>，或 --selftest" >&2
  exit 2
}

# run_merged_log <键> <树根> <指纹> <日志> <测试目标> <用例函数>：交双机驱动，导入那一趟的日志抄进 <日志>；退出码照文件头「退出码」。
run_merged_log() {
  local key="$1" root="$2" fingerprint="$3" log="$4" target="$5" function="$6"
  [[ -d "$root" ]] || usage_error "树根 $root 不在"
  [[ "$fingerprint" =~ ^[0-9a-f]{64}$ ]] || usage_error "输入指纹要 64 位十六进制：$fingerprint"
  [[ "$key" == crash-case:* && -n "$target" && -n "$function" ]] || usage_error "键要以 crash-case: 起头，测试目标与用例函数不许空"
  local configuration_file gpu library_home library label driver import_log driver_exit
  configuration_file="$(crash_amplification_multi_host_configuration_file "$root")"
  gpu="$(crash_amplification_configuration_value "$configuration_file" ENABLE_GPU)"
  library_home="${SINGLEFS_CRASH_AMPLIFICATION_HOME:-$HOME/.local/share/singlefs/crash-amplification}"
  library="${SINGLEFS_CRASH_AMPLIFICATION_LIBRARY:-$library_home/crash-state-enumeration-library}"
  label="crash-point-replay-${key#crash-case:}-$(date -u +%Y%m%dT%H%M%SZ)-$$"
  driver="${CRASH_POINT_REPLAY_SHARD_RUN_TWO_HOSTS_DRIVER:-$root/scripts/crash/crash-amplification-two-hosts.sh}"
  local -a driver_arguments=("$target" --test-filter "$function" --label "$label")
  shard_break_is library-not-passed || driver_arguments+=(--library "$library")
  if [[ -n "${SINGLEFS_CRASH_POINT_REPLAY_THREADS:-}" ]]; then driver_arguments+=(--threads "$SINGLEFS_CRASH_POINT_REPLAY_THREADS"); fi
  if [[ "$gpu" == 1 ]] && ! shard_break_is quiesce-not-passed; then driver_arguments+=(--quiesce); fi
  # 外层内存包装给的上限（文件头「内存包装」）：写法不认得的当没给
  local outer_cap="${RUN_WITH_MEMORY_CAP_OUTER_CAP:-}"
  [[ "$outer_cap" =~ ^[1-9][0-9]*[KMGT]$ ]] || outer_cap=""
  shard_break_is ignore-outer-cap && outer_cap=""
  if [[ -n "$outer_cap" ]]; then driver_arguments+=(--memory-cap "$outer_cap"); fi
  echo "  · 崩溃点重放双机：$key（输入指纹 ${fingerprint:0:16}…）交崩溃放量的双机驱动：标签 $label，库 $library，ENABLE_GPU=${gpu:-0}；本机 0/2、第二台 1/2 各写各的库，取回，本机带导入再跑一趟"
  ( cd "$root" && env SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE="$log.findings.tsv" bash "$driver" "${driver_arguments[@]}" ); driver_exit=$?
  import_log="$library_home/runs/$label/import.log"
  if [[ -f "$import_log" ]]; then
    if ! shard_break_is log-not-copied; then
      cp -- "$import_log" "$log"
      echo "  · 导入那一趟的日志抄到 $log（驱动那几份在 $library_home/runs/$label/）"
    fi
  elif [[ "$driver_exit" == 0 ]]; then
    echo "  ✗ 双机驱动退 0，导入那一趟的日志 $import_log 却不在：没跑到导入那一步，不当绿"
    echo "     → 怎么办：看 $library_home/runs/$label/ 里有哪几份日志、驱动的输出停在哪一步；修好之后重跑 --full。"
    return 1
  else
    : > "$log"
    echo "  · 双机驱动退 $driver_exit，导入那一趟的日志不在（$import_log）：日志文件留空，门禁按驱动的退出码判红"
  fi
  if shard_break_is exit-swallowed; then return 0; fi
  return "$driver_exit"
}

# ============================== 自证 ==============================

FAKE_TWO_HOSTS_DRIVER='#!/usr/bin/env bash
printf "%s\n" "$*" > "${FAKE_DRIVER_CONTROL:?}/arguments"
printf "%s\n" "${SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE-unset}" > "$FAKE_DRIVER_CONTROL/findings-file"
label=""
local_cap=""
while (($#)); do
  if [[ "$1" == --label ]]; then label="$2"; shift 2
  elif [[ "$1" == --memory-cap ]]; then local_cap="$2"; shift 2
  else shift; fi
done
# 本机那几趟的上限：没给 --memory-cap 就取真驱动文件里 memory_cap="<数><单位>" 的第一处（驱动的默认），照真驱动的规矩
if [[ -z "$local_cap" ]]; then
  default_pattern="memory_cap=\"([0-9]+[KMGT])\""
  while IFS= read -r driver_line; do
    if [[ "$driver_line" =~ $default_pattern ]]; then local_cap="${BASH_REMATCH[1]}"; break; fi
  done < "${FAKE_DRIVER_REAL_DRIVER:?}"
fi
printf "%s\n" "$local_cap" > "$FAKE_DRIVER_CONTROL/local-memory-cap"
size_in_bytes() {
  local number="${1%[KMGT]}" unit="${1: -1}"
  case "$unit" in K) echo $((number * 1024)) ;; M) echo $((number * 1024 ** 2)) ;; G) echo $((number * 1024 ** 3)) ;; T) echo $((number * 1024 ** 4)) ;; esac
}
# 内存包装的规矩（scripts/hosts/run-with-memory-cap.sh 文件头「套在包装里」）：套在外层里而里层上限比外层大，按用法错退 251、命令没跑
echo "fake-driver: local memory cap=$local_cap outer=${RUN_WITH_MEMORY_CAP_OUTER_CAP-unset}"
if [[ "${RUN_WITH_MEMORY_CAP_OUTER_CAP:-}" =~ ^[1-9][0-9]*[KMGT]$ ]] && (( $(size_in_bytes "$local_cap") > $(size_in_bytes "$RUN_WITH_MEMORY_CAP_OUTER_CAP") )); then
  echo "run-with-memory-cap: 用法错：里层上限 $local_cap 比外层 $RUN_WITH_MEMORY_CAP_OUTER_CAP 大"
  exit 251
fi
if [[ ! -f "$FAKE_DRIVER_CONTROL/no-import-log" ]]; then
  mkdir -p "${SINGLEFS_CRASH_AMPLIFICATION_HOME:?}/runs/$label"
  printf "CRASH_POINT_REPLAY_PARALLEL_FINISHED states=29 slices=1 worker_threads=2\nCRASH_POINT_REPLAY states=29 exhaustive=true\n" > "$SINGLEFS_CRASH_AMPLIFICATION_HOME/runs/$label/import.log"
fi
exit "$(cat "$FAKE_DRIVER_CONTROL/exit" 2>/dev/null || echo 0)"
'
selftest_cells=(library-default library-from-environment log-copied driver-exit-kept quiesce-when-gpu no-import-log-red findings-file threads-passed outer-memory-cap-to-driver)
selftest_zero_fingerprint="0000000000000000000000000000000000000000000000000000000000000000"

selftest_reset() {
  rm -rf -- "${selftest_work:?}/control" "${selftest_work:?}/home"
  rm -f -- "${selftest_work:?}/log" "${selftest_work:?}/log.findings.tsv" "${selftest_work:?}/output"
  mkdir -p -- "$selftest_work/control" "$selftest_work/home" "$selftest_work/root"
  printf 'ENABLE_GPU=0\n' > "$selftest_work/root/multi-host.env"
}

# selftest_run_wrapper <日志> [环境 键=值…]：起这一份 --merged-log，输出写 <自证目录>/output，退出码放进 selftest_exit（直接调，不在 $(…) 里）
selftest_exit=""
selftest_run_wrapper() {
  local log="$1"
  shift
  local -a wrapper_environment=(-u SINGLEFS_CRASH_AMPLIFICATION_LIBRARY -u SINGLEFS_CRASH_POINT_REPLAY_THREADS -u SINGLEFS_MULTI_HOST_CONFIG
    -u RUN_WITH_MEMORY_CAP_OUTER_UNIT -u RUN_WITH_MEMORY_CAP_OUTER_CAP
    "SINGLEFS_CRASH_AMPLIFICATION_HOME=$selftest_work/home" "FAKE_DRIVER_CONTROL=$selftest_work/control"
    "FAKE_DRIVER_REAL_DRIVER=$shard_script_directory/crash-amplification-two-hosts.sh"
    "CRASH_POINT_REPLAY_SHARD_RUN_TWO_HOSTS_DRIVER=$selftest_work/fake-driver.sh" "$@")
  if env "${wrapper_environment[@]}" bash "$0" --merged-log crash-case:demo "$selftest_work/root" "$selftest_zero_fingerprint" "$log" demo_target demo_function >"$selftest_work/output" 2>&1; then
    selftest_exit=0
  else
    selftest_exit=$?
  fi
}

selftest_arguments() { cat "$selftest_work/control/arguments" 2>/dev/null; }
selftest_last_output() { cat "$selftest_work/output" 2>/dev/null; }

cell_library_default() {
  selftest_reset
  selftest_run_wrapper "$selftest_work/log"
  [[ "$selftest_exit" == 0 && "$(selftest_arguments)" == "demo_target --test-filter demo_function --label crash-point-replay-demo-"*"--library $selftest_work/home/crash-state-enumeration-library"* ]]
}
cell_library_from_environment() {
  selftest_reset
  selftest_run_wrapper "$selftest_work/log" SINGLEFS_CRASH_AMPLIFICATION_LIBRARY="$selftest_work/start-over-library"
  [[ "$selftest_exit" == 0 && "$(selftest_arguments)" == *"--library $selftest_work/start-over-library"* ]]
}
cell_log_copied() {
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
cell_quiesce_when_gpu() {
  selftest_reset
  selftest_run_wrapper "$selftest_work/log"
  [[ "$selftest_exit" == 0 && "$(selftest_arguments)" != *"--quiesce"* ]] || return 1
  printf 'ENABLE_GPU=1\n' > "$selftest_work/root/multi-host.env"
  selftest_run_wrapper "$selftest_work/log"
  [[ "$selftest_exit" == 0 && "$(selftest_arguments)" == *"--quiesce"* ]]
}
cell_no_import_log_red() {
  selftest_reset
  : > "$selftest_work/control/no-import-log"
  selftest_run_wrapper "$selftest_work/log"
  [[ "$selftest_exit" == 1 && "$(selftest_last_output)" == *"不当绿"* ]]
}
cell_findings_file() {
  selftest_reset
  selftest_run_wrapper "$selftest_work/log"
  [[ "$selftest_exit" == 0 && "$(cat "$selftest_work/control/findings-file")" == "$selftest_work/log.findings.tsv" ]]
}
cell_threads_passed() {
  selftest_reset
  selftest_run_wrapper "$selftest_work/log" SINGLEFS_CRASH_POINT_REPLAY_THREADS=5
  [[ "$selftest_exit" == 0 && "$(selftest_arguments)" == *"--threads 5"* ]]
}
# 外层内存包装给了 16G（RUN_WITH_MEMORY_CAP_OUTER_CAP=16G，门禁 --full 整条套在 16G 里）：驱动收到的本机上限不大于 16G、包装不退 251；
# 没套在外层里不交 --memory-cap（驱动用自己的默认）；外层上限写法不认得的当没给
cell_outer_memory_cap_to_driver() {
  selftest_reset
  selftest_run_wrapper "$selftest_work/log" RUN_WITH_MEMORY_CAP_OUTER_CAP=16G
  [[ "$selftest_exit" == 0 && "$(selftest_arguments)" == *"--memory-cap 16G"* && "$(cat "$selftest_work/control/local-memory-cap")" == 16G ]] || return 1
  selftest_reset
  selftest_run_wrapper "$selftest_work/log"
  [[ "$(selftest_arguments)" != *"--memory-cap"* ]] || return 1
  selftest_reset
  selftest_run_wrapper "$selftest_work/log" RUN_WITH_MEMORY_CAP_OUTER_CAP=sixteen
  [[ "$(selftest_arguments)" != *"--memory-cap"* ]]
}

# 真跑那一格（文件头「真跑那一格」）：real_cell_outcome 置成 green、red 或 not-run，real_cell_detail 是一句说明
REAL_CELL_CASE_KEY="crash-case:new-pool-file-creation-stream"
real_cell_outcome="" real_cell_detail=""
real_two_hosts_run() {
  local repository_root real configuration_source across row test_field target function label_directories runs import_copy record_log wrapper_exit
  repository_root="$(cd "$shard_script_directory/../.." && pwd)"
  real="$selftest_work/real"
  mkdir -p -- "$real/home"
  configuration_source="$(crash_amplification_multi_host_configuration_file "$repository_root")"
  across="$(crash_amplification_configuration_value "$configuration_source" ENABLE_ACROSS_MACHINES)"
  if [[ ! -f "$configuration_source" || "$across" != 1 ]]; then
    real_cell_outcome=not-run real_cell_detail="多机配置 $configuration_source 不在、或没写 ENABLE_ACROSS_MACHINES=1：双机没开，真驱动跑不了双机"
    return 0
  fi
  # 拷一份：ENABLE_GPU 改成 0、去掉显卡先后那两个键（不上卡、不清场）；拷贝在自证的临时目录里，退出时删
  { grep -vE '^[[:space:]]*(export[[:space:]]+)?(ENABLE_GPU|GPU_CARDS_BY_PRIORITY|GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED)[[:space:]]*=' "$configuration_source"; echo ENABLE_GPU=0; } >"$real/multi-host.env"
  row="$(awk -F'\t' -v key="$REAL_CELL_CASE_KEY" '$1 == key { print $3; exit }' "$repository_root/.claude/gate.d/stage-inputs.tsv")"
  test_field="$(tr ' ' '\n' <<<"$row" | grep -m 1 '^test=')"
  target="$(cut -d: -f2 <<<"${test_field#test=}")" function="$(cut -d: -f3 <<<"${test_field#test=}")"
  if [[ -z "$target" || -z "$function" || " $row " != *" shard=across-machines "* ]]; then
    real_cell_outcome=red real_cell_detail=".claude/gate.d/stage-inputs.tsv 里 $REAL_CELL_CASE_KEY 那一行读不出 test=<包>:<测试目标>:<用例函数>、或没登记 shard=across-machines（读到「$row」）"
    return 0
  fi
  echo "  · 真跑那一格：$REAL_CELL_CASE_KEY（$target::$function）经真驱动双机跑，库的家目录 $real/home，编译目录 ${GATE_CROSS_RUN_TMPDIR:-${TMPDIR:-/tmp}}/crash-point-replay-shard-run-selftest-target"
  if env -u SINGLEFS_CRASH_AMPLIFICATION_LIBRARY -u CRASH_POINT_REPLAY_SHARD_RUN_TWO_HOSTS_DRIVER -u CRASH_POINT_REPLAY_SHARD_RUN_BREAK \
      SINGLEFS_CRASH_AMPLIFICATION_HOME="$real/home" SINGLEFS_CRASH_AMPLIFICATION_LIBRARY_IS_TEMPORARY=1 SINGLEFS_MULTI_HOST_CONFIG="$real/multi-host.env" \
      CARGO_TARGET_DIR="${GATE_CROSS_RUN_TMPDIR:-${TMPDIR:-/tmp}}/crash-point-replay-shard-run-selftest-target" \
      bash "$0" --merged-log "$REAL_CELL_CASE_KEY" "$repository_root" "$selftest_zero_fingerprint" "$real/merged.log" "$target" "$function" >"$real/output" 2>&1; then
    wrapper_exit=0
  else
    wrapper_exit=$?
  fi
  label_directories=("$real/home/runs/crash-point-replay-${REAL_CELL_CASE_KEY#crash-case:}-"*)
  runs="${label_directories[0]}"
  record_log="$runs/record.log" import_copy="$real/merged.log"
  if [[ "$wrapper_exit" == 78 ]]; then
    real_cell_outcome=not-run real_cell_detail="驱动退 78（运行条件不满足：第二台连不上、库没取回、线程数现算不出这一类；输出尾部：$(tail -n 4 "$real/output" | tr '\n' '|')）"
  elif [[ "$wrapper_exit" != 0 ]]; then
    real_cell_outcome=red real_cell_detail="包装退 $wrapper_exit（输出尾部：$(tail -n 6 "$real/output" | tr '\n' '|')）"
  elif (( ${#label_directories[@]} != 1 )) || [[ ! -d "$runs" ]]; then
    real_cell_outcome=red real_cell_detail="$real/home/runs/ 下这一趟的日志目录不是恰好一个（${label_directories[*]}）"
  elif ! grep -qE '^CRASH_LIBRARY_RECORDED class=crash_state_enumeration item=[^ ]+ manifest=[0-9a-f]+ library=.*/crash-state-enumeration-library$' "$record_log" 2>/dev/null; then
    real_cell_outcome=red real_cell_detail="录入段那一趟的日志 $record_log 里没有 CRASH_LIBRARY_RECORDED class=crash_state_enumeration … library=…/crash-state-enumeration-library 那一行（录入段没真跑，或录进了别的库）"
  elif ! grep -qE '^CRASH_POINT_REPLAY_PARALLEL_START .* imported_blocks=[1-9][0-9]* shard=0/2 ' "$import_copy"; then
    real_cell_outcome=red real_cell_detail="抄回的导入那一趟日志里 CRASH_POINT_REPLAY_PARALLEL_START 不带 imported_blocks=<正数> shard=0/2（第二台判的块没导进来）"
  elif [[ -z "$(grep '^CRASH_POINT_REPLAY_PARALLEL_FINISHED ' "$import_copy" | grep -v 'blocks_left_to_other_hosts=')" ]]; then
    real_cell_outcome=red real_cell_detail="抄回的导入那一趟日志里没有不带 blocks_left_to_other_hosts= 的 CRASH_POINT_REPLAY_PARALLEL_FINISHED 行（导入之后对账不为空）"
  elif ! grep -qE '^CRASH_POINT_REPLAY states=([0-9]+) closed_form=\1 violations=0 .*exhaustive=true' "$import_copy"; then
    real_cell_outcome=red real_cell_detail="抄回的导入那一趟日志里 CRASH_POINT_REPLAY 计数行不是 states 等于 closed_form、violations=0、exhaustive=true（$(grep -m 1 '^CRASH_POINT_REPLAY ' "$import_copy" | cut -c1-200)）"
  else
    real_cell_outcome=green
    real_cell_detail="$(grep -m 1 '^CRASH_POINT_REPLAY states=' "$import_copy" | cut -d' ' -f1-4)；导入 $(grep -m 1 -oE 'imported_blocks=[0-9]+' "$import_copy")"
  fi
  if [[ "$real_cell_outcome" == red && -d "${runs:-}" ]]; then
    local kept="${TMPDIR:-/tmp}/crash-point-replay-shard-run-selftest-logs-$$"
    mkdir -p -- "$kept" && cp -- "$runs"/*.log "$real/output" "$kept/" 2>/dev/null
    real_cell_detail+="；日志拷到 $kept/ 留着"
  fi
  return 0
}

# run_selftest_cells：跑全部格，红的格名打在 stdout（空格隔开）
run_selftest_cells() {
  local cell red=()
  for cell in "${selftest_cells[@]}"; do
    "cell_${cell//-/_}" || red+=("$cell")
  done
  echo "${red[*]+"${red[*]}"}"
}

if [[ "${1:-}" == --selftest ]]; then
  [[ $# -eq 1 ]] || usage_error "--selftest 不带别的参数"
  selftest_work="$(mktemp -d)"
  trap 'rm -rf -- "${selftest_work:?}"' EXIT
  mkdir -p -- "$selftest_work/root"
  printf '%s' "$FAKE_TWO_HOSTS_DRIVER" > "$selftest_work/fake-driver.sh"
  chmod +x "$selftest_work/fake-driver.sh"
  selftest_failed=0
  red_cells="$(CRASH_POINT_REPLAY_SHARD_RUN_BREAK= run_selftest_cells)"
  if [[ -n "$red_cells" ]]; then
    echo "  ✗ crash-point-replay-shard-run 自证：这几格红：$red_cells（最后一趟的输出：$(selftest_last_output))"
    echo "  → 怎么办：对着文件头「用法」与这几格的判法看交给驱动的参数、日志抄回、退出码哪一样变了。"
    selftest_failed=1
  fi
  declare -A switch_cells=([library-not-passed]="library-default library-from-environment" [log-not-copied]="log-copied" [exit-swallowed]="driver-exit-kept" [quiesce-not-passed]="quiesce-when-gpu" [ignore-outer-cap]="outer-memory-cap-to-driver")
  switch_count=0
  for switch in library-not-passed log-not-copied exit-swallowed quiesce-not-passed ignore-outer-cap; do
    switch_count=$((switch_count + 1))
    red_cells="$(CRASH_POINT_REPLAY_SHARD_RUN_BREAK="$switch" run_selftest_cells)"
    if [[ "$red_cells" != "${switch_cells[$switch]}" ]]; then
      echo "  ✗ 开关轮：打开 $switch 该红的是「${switch_cells[$switch]}」，实际红的是「$red_cells」"
      echo "  → 怎么办：文件头「弄坏开关」那一张表与 run_merged_log 里 shard_break_is 的分支对一遍。"
      selftest_failed=1
    fi
  done
  if (( selftest_failed )); then exit 1; fi
  light_summary="${#selftest_cells[@]} 格绿（假的双机驱动：参数、库目录、--quiesce、日志抄回、退出码、发现日志、线程、外层内存上限），开关轮 ${switch_count} 个开关各红在点名的格"
  if [[ -z "${SINGLEFS_HEAVY_TESTS:-}" ]]; then
    echo "  ✓ crash-point-replay-shard-run 自证（假的那几格）通过：$light_summary"
    echo "  ! 真跑那一格 real-two-hosts-run 本次未跑（这不是通过）：命令里没设 SINGLEFS_HEAVY_TESTS；要跑照文件头「真跑那一格」带 SINGLEFS_HEAVY_TESTS=user-request 起 --selftest"
    if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then echo "crash-point-replay-shard-run.sh --selftest real-two-hosts-run：没设 SINGLEFS_HEAVY_TESTS" >>"$GATE_NOT_RUN_FILE"; fi
    exit 0
  fi
  real_two_hosts_run
  case "$real_cell_outcome" in
    green)
      echo "  ✓ crash-point-replay-shard-run 自证通过：$light_summary；真跑那一格 real-two-hosts-run 绿（$REAL_CELL_CASE_KEY 经真驱动双机跑：录入段真跑、CRASH_LIBRARY_RECORDED 真打、导入之后对账为空；$real_cell_detail），共 $(( ${#selftest_cells[@]} + 1 )) 格"
      exit 0 ;;
    not-run)
      echo "  ! 真跑那一格 real-two-hosts-run 本次未跑（这不是通过）：$real_cell_detail；假的那几格：$light_summary"
      if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then echo "crash-point-replay-shard-run.sh --selftest real-two-hosts-run：$real_cell_detail" >>"$GATE_NOT_RUN_FILE"; fi
      exit 0 ;;
    *)
      echo "  ✗ crash-point-replay-shard-run 自证 real-two-hosts-run：$real_cell_detail"
      echo "  → 怎么办：对着文件头「真跑那一格」的判法看那几份日志：录入段（record.log）、第二台那一份（share-1.log）、导入那一趟（import.log）哪一步没做到；假的那几格：$light_summary"
      exit 1 ;;
  esac
fi

[[ "${1:-}" == --merged-log ]] || usage_error "第一个参数要是 --merged-log 或 --selftest（给的是「${1:-}」）"
[[ $# -eq 7 ]] || usage_error "--merged-log 后面要六个参数：键、树根、输入指纹、日志文件、测试目标、用例函数（给了 $(( $# - 1 )) 个）"
run_merged_log "$2" "$3" "$4" "$5" "$6" "$7"
