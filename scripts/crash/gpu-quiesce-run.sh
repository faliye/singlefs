#!/usr/bin/env bash
# admission: always 每次调都在这一刻的卡与服务上清场、跑，上一次的结局不替这一次作保
# run-condition: command bash python3 flock
# 凡是起 GPU 的跑法都经这一份：照多机配置清场并回读、跑那条命令、交回那条命令的退出码。跑完、跑红或被打断都不复原：
#   清场停下的服务一直停着，要复原由用户手动跑私有复原脚本（多机配置的 QUIESCE_START_COMMAND，再拿 QUIESCE_STARTED_CHECK_COMMAND 回读到它退 0）。
# 多机配置里没写清场命令的不在文件头的 run-condition 里判（--selftest 不读多机配置，要在没有配置的机器上也跑得了），
#   在运行中照 .claude/singlefs-ai-sop/rules/preflight-discipline.md 的退出码退 78、写出路。
#
#   gpu-quiesce-run.sh -- <命令…>     清场 → 跑 <命令…>（不复原）；<命令…> 原样起（不经 sh -c），标准输入输出与环境原样交给它
#   gpu-quiesce-run.sh --selftest     拿假的清场命令（换掉多机配置）与假的命令走一遍各格，末尾一轮逐个打开弄坏开关、核它红在点名的格
#
# 清场：读多机配置（${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}，模板仓根 multi-host.env.example；主工作树的根按 git common-dir 取，
#   同 scripts/crash/crash-point-replay-shard-configuration-check.sh）的两个键 QUIESCE_STOP_COMMAND、QUIESCE_STOPPED_CHECK_COMMAND，两个都要写；
#   QUIESCE_START_COMMAND、QUIESCE_STARTED_CHECK_COMMAND 留给手动复原，这一份不读、写没写都照跑。
#   两条命令各用 bash -c 在调用方的工作目录里跑，命令与输出都不打到标准错误上（里面有私有的服务名）。
#   输出逐条追加进这一趟的本机日志 <日志目录>/<日期>-<登记名>.log（目录 700、文件 600，每条前后各一行「== <键> 起 <时刻>」「== <键> 退 <码>」）：
#   清场回读没过、或这一趟跑的清场命令退非 0 时留下，路径写进 GPU_QUIESCE 那一行的 log=；都过了就删掉。
# 上一趟清了场没复原、服务已经停着：这一趟照样跑清场命令（私有清场脚本见已经停干净就只回读、退 0），回读过了就照跑。
#   日志目录 ${GPU_QUIESCE_LOG_DIRECTORY:-<工作根>/gpu-quiesce-logs}：开机不清，工作根只在 .claude/hooks/lib_work_root.py 里定义（SINGLEFS_WORK_ROOT），
#   这里经 python3 那份库取，取不到照「状态目录建不了」退 78；--selftest 把它指到自己的临时目录，不落进真工作根。
#   这一份与它调的东西不停、不杀任何进程，也不按模式找进程：清场只停清场命令白名单里的服务单元；这一趟要用的卡上有不归白名单、也不在别的活着的登记之下的
#   计算进程（别的会话的 GPU 活）时清场回读本来就不过，照拒（退 78），出路是等那个进程跑完再起。
# 回读看哪几张卡、跳过哪些进程：这一份在进来时算好两样，经环境只交给清场那两条命令（QUIESCE_STOP_COMMAND、QUIESCE_STOPPED_CHECK_COMMAND），
#   私有回读脚本认它们；调用方带进来的同名变量进来时先清掉，不认、也不传给 <命令…>：
#   GPU_QUIESCE_CHECK_LOCAL_CARDS=<本机卡的 nvidia-smi 序号，逗号隔开>  这一趟要用的本机卡，回读只看这几张上的计算进程。
#     取法照用例那一侧（crates/singlefs-checker-tier/src/gpu_card_configuration.rs 的 cards_by_priority_from，清过场那一支）：环境变量 GPU_CARDS_BY_PRIORITY
#     设了照它（空串也算设了），没设读多机配置的 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED；取其中 local:<序号>:<额度> 那几项，peer 的不算。
#     环境变量 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED 与多机配置的 GPU_CARDS_BY_PRIORITY 不读（清场之下的用例不照它们取卡）。
#     两处都没写、有一项写法不对（用例那一侧照样报错不跑）、或一项本机卡都没有时不设：回读照旧查本机所有卡。
#   GPU_QUIESCE_CHECK_EXEMPT_PIDS=<进程号，空格隔开>  别的活着的登记（holders/ 下）各自的进程号与它们这一刻的全部子孙：在锁里、清场命令起之前，
#     按 /proc/<进程号>/stat 的父进程号一层层往下找（不按名字或模式找进程），回读不把它们算「没清干净」。没有别的活着的登记时不设。
#     子孙的父进程先退了、被收养到别处的，不再算子孙，照外人办。
#   两样都只管本机的卡与进程；第二台的回读不变。私有回读脚本两个变量都没给时与没有它们时一样查。标准错误上另打一句「回读看的卡：…；跳过 N 个进程」。
# 几趟同时用卡：每一趟进来时在状态目录（${GPU_QUIESCE_STATE_DIRECTORY:-${XDG_RUNTIME_DIR:-/tmp}/singlefs-gpu-quiesce}）的 holders/ 下登记一份
#   <进程号>-<进程的开机后起始时刻>，进出都在同一把 flock 锁里（至多等 GPU_QUIESCE_LOCK_SECONDS 秒，默认 900）。进来时没有别的活着的登记才跑清场命令，
#   有就只回读；出去时删掉自己的登记。登记的进程号不在了、或起始时刻对不上（进程号被重用）的登记是死的，进出时删掉；被 SIGKILL 的那一趟留下的登记由下一趟删。
# 被打断：收到 INT、TERM、HUP 时不停命令、不发任何信号，等前台那条命令自己结束再退；清场途中收到的，命令不起。都不复原。
#   要停这一条链，从最底层的命令起停（停这一份自己而命令还在跑时，它要等命令结束才退）。
# 卡与额度：不读显卡配置、不定额度、不按余量拒或排队：用哪几张卡、各给多少由调用方定（例：分项驱动按清场之后那一份显卡配置交给用例），
#   这一份只管清场、回读、跑。调用方给的环境（GPU_CARDS_BY_PRIORITY 这类）原样传给命令。
# 给调用方与派发器的接口（选哪台、查余量、定卡与额度都在调用方；这一份不读也不改它们）：
#   参数只有 -- <命令…>；命令的环境原样带下去（调用方定的卡与额度经环境交给命令，例 GPU_CARDS_BY_PRIORITY），另加一个 GPU_QUIESCE_HOLDER=<这一趟的登记文件>：
#   命令要核自己在清场之下，看这份文件在、文件名以它父进程的进程号加「-」起头（被包着的命令的父进程就是这一份）。
#   交给清场那两条命令的另有 GPU_QUIESCE_CHECK_LOCAL_CARDS、GPU_QUIESCE_CHECK_EXEMPT_PIDS（见「回读看哪几张卡、跳过哪些进程」），<命令…> 拿不到它们。
#   状态目录 GPU_QUIESCE_STATE_DIRECTORY、本机日志目录 GPU_QUIESCE_LOG_DIRECTORY 与锁的等待 GPU_QUIESCE_LOCK_SECONDS 可由调用方给；同一台机器上同时用卡的各趟要用同一个状态目录；
#   调用方的自证用假清场命令跑这一份时，把日志目录也指到自己的临时目录，不然失败那几趟的日志落进真工作根。
#   结局读退出码与下面的 GPU_QUIESCE 行：命令自己退 78 与这一份退 78 由 step=command 那一行分（not-started 是这一份拒的）。
# 输出：标准错误上逐步一行，给调用方与派发器读（字段 key=value，空格隔开）：
#   GPU_QUIESCE step=quiesce result=stopped-and-read-back|joined-and-read-back|read-back-failed holders=<进来时别的活着的登记数>[ stop-exit=<码>][ log=<本机日志>]
#     stop-exit 是这一趟跑的清场命令的退出码，回读没过或它退非 0 时打（回读过了而它退非 0，那一行是 stopped-and-read-back 带非 0 的 stop-exit，照跑）；
#     log 在留下本机日志时打
#   GPU_QUIESCE step=command result=exit-<码>|not-started
#   GPU_QUIESCE step=restore result=not-restored holders=<出去时别的活着的登记数>
#     清了场、没复原（清场回读没过、命令没起的那一趟也打这一行）；标准错误另打一句「清了场、没复原」与手动复原的写法
#   GPU_QUIESCE step=exit code=<这一份的退出码>
#   出错时另打「✗ <哪一步>」与下一行「→ 怎么办：…」。
# 退出码：
#   <命令的>  命令起了：交回命令的退出码；命令退 0 而这一份收到过 INT/TERM/HUP 时退 128+信号号
#   2         用法错：没给 -- 或 -- 后面没有命令
#   78        命令没起：多机配置里 QUIESCE_STOP_COMMAND、QUIESCE_STOPPED_CHECK_COMMAND 没写全、状态目录或本机日志建不了（含取不到工作根）、锁等不到、清场回读没过；
#             收到 INT/TERM/HUP 而命令还没起时退 128+信号号
# 弄坏开关 GPU_QUIESCE_BREAK=<项>（只给证红用，逗号连几个；--selftest 末尾那一轮从这张表现读，写法同 scripts/crash/crash-amplification.sh 文件头「弄坏开关」一段，
#   格名是 run_selftest 里「✗ 自证 <格名>：」的那一段）：
#   GPU_QUIESCE_BREAK=run-despite-failed-quiesce  清场回读没过也起命令
#       quiesce-read-back-fails（「清场回读没过却起了命令」）
#   GPU_QUIESCE_BREAK=restore-after-run  出去时（命令起没起、退几都算）又照多机配置跑 QUIESCE_START_COMMAND 与 QUIESCE_STARTED_CHECK_COMMAND 复原，restore 那一行写 restored
#       no-restore（「跑完调了复原命令」）
#   GPU_QUIESCE_BREAK=trust-stale-holders  死了的登记也当活着的
#       stale-holder（「死了的登记挡住了清场」）
#   GPU_QUIESCE_BREAK=accept-missing-quiesce-commands  多机配置里 QUIESCE_STOP_COMMAND、QUIESCE_STOPPED_CHECK_COMMAND 没写全也不清场照起命令
#       missing-commands（「多机配置没写全清场命令却没退 78」）
#   GPU_QUIESCE_BREAK=no-failure-log  回读没过也不留本机日志、GPU_QUIESCE 行不带 log=
#       failure-log（「回读没过没留本机日志」）
#   GPU_QUIESCE_BREAK=failure-log-on-stderr  清场命令的输出打到标准错误上，不进本机日志
#       failure-log（「清场命令的输出打到了标准错误上」）
#   GPU_QUIESCE_BREAK=failure-log-in-runtime-dir  没设 GPU_QUIESCE_LOG_DIRECTORY 时日志落进 ${XDG_RUNTIME_DIR:-/tmp}（重启就清），不落工作根
#       failure-log（「本机日志没落在工作根下」）
#   GPU_QUIESCE_BREAK=drop-stop-exit  GPU_QUIESCE 清场那一行不带 stop-exit=（清场命令退非 0 而回读过了时看不出它退了几）
#       stop-exit（「清场命令退非 0 而回读过了，那一行没带 stop-exit」）
#   GPU_QUIESCE_BREAK=no-holder-descendants  不给回读 GPU_QUIESCE_CHECK_EXEMPT_PIDS：别的活着的登记底下的进程也当外人（改前的样子）
#       holder-descendants（「前一趟登记下的进程在卡上，后一趟进来回读没过」）
#   GPU_QUIESCE_BREAK=exempt-holder-session  跳过的进程按会话号认：与别的活着的登记同一个会话的进程都跳过（不在任何登记之下的外人也跳过了）
#       outsider-not-exempt（「不在任何登记之下的进程在卡上，回读却过了」）
#   GPU_QUIESCE_BREAK=check-every-card  不给回读 GPU_QUIESCE_CHECK_LOCAL_CARDS：这一趟不用的卡也查（改前的样子）
#       unused-card-outsider（「这一趟只用卡 1，卡 0 上的外人挡住了回读」）、used-card-outsider（「回读拿到的卡不是只有卡 0」）
#   GPU_QUIESCE_BREAK=cards-from-unquiesced-key  没设 GPU_CARDS_BY_PRIORITY 时读多机配置的 GPU_CARDS_BY_PRIORITY（不清场那一份），不读 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED
#       used-card-outsider（「这一趟用卡 0，卡 0 上的外人却没挡住回读」）
#   GPU_QUIESCE_BREAK=keep-inherited-check-variables  调用方带进来的 GPU_QUIESCE_CHECK_LOCAL_CARDS、GPU_QUIESCE_CHECK_EXEMPT_PIDS 不清掉，这一份没设的那一样照它们传给回读
#       used-card-outsider（「这一趟用卡 0，卡 0 上的外人却没挡住回读」）
#   这几个开关别的脚本的自证也认（它们起的命令经这一份）：崩溃放量入口 scripts/crash/crash-amplification.sh、分项驱动 scripts/crash/crash-amplification-item.sh
#   与门禁 checker-tier-crash-point-replay 的样本，各红在哪一格写在那三份文件头。
# 自证末尾那一轮走 scripts/crash/crash-amplification-lib.sh 的 crash_amplification_switch_wheel（并行度 CRASH_AMPLIFICATION_SELFTEST_JOBS，没给取 SINGLEFS_THREAD_CAP，再没给取登记的重活默认并行度）。
#
# gate-similar: peer-host-lib.sh 它的 peer_host_quiesce / peer_host_quiesce_left 只在双机配置开着时读得到清场命令，也不管几趟同时用卡；改它会废掉崩溃点重放双机分片三条用例的全绿标记（admission.py 的 SHARD_DRIVER_FILES），所以这里另写清场与登记
# gate-similar: crash-amplification-two-hosts.sh 它的 --quiesce 经 peer-host-lib.sh 清场，只给单跑双机驱动时用；分项驱动与崩溃放量入口经这一份
# gate-similar: crash-point-replay-shard-run.sh 它在崩溃点重放双机分片（CPU）开跑前经 peer-host-lib.sh 清场；不上 GPU，不归这一份
# gate-similar: run-with-memory-cap.sh 同是「包装 -- 命令」的写法，它管内存上限，这里管清场；两样各自套，次序由调用方定
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

gpu_quiesce_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# 共用库：读配置的一个键（crash_amplification_configuration_value）与弄坏开关轮（--selftest 末尾那一轮）
source "$gpu_quiesce_script_directory/crash-amplification-lib.sh"

GPU_QUIESCE_KEYS=(QUIESCE_STOP_COMMAND QUIESCE_STOPPED_CHECK_COMMAND)
# 复原那一对只在弄坏开关 restore-after-run 下读（文件头「清场」：复原由用户手动做）
GPU_QUIESCE_RESTORE_KEYS=(QUIESCE_START_COMMAND QUIESCE_STARTED_CHECK_COMMAND)

gpu_quiesce_break_is() { [[ ",${GPU_QUIESCE_BREAK:-}," == *",$1,"* ]]; }
gpu_quiesce_line() { echo "GPU_QUIESCE $*" >&2; }

usage() {
  echo "  ✗ 用法：$0 -- <命令…>，或 $0 --selftest" >&2
  echo "  → 怎么办：要清场再跑的命令写在 -- 后面，例 $0 -- cargo test --release -p singlefs-checker-tier --features verdict-store,gpu --test <测试目标>" >&2
  exit 2
}

gpu_quiesce_configuration_file() { # 打多机配置的路径：SINGLEFS_MULTI_HOST_CONFIG，没设取主工作树的根（git common-dir 的上一层）下的 multi-host.env
  if [[ -n "${SINGLEFS_MULTI_HOST_CONFIG:-}" ]]; then printf '%s' "$SINGLEFS_MULTI_HOST_CONFIG"; return 0; fi
  local common_directory
  if common_directory="$(git -C "$gpu_quiesce_script_directory" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" && [[ -n "$common_directory" ]]; then
    printf '%s/multi-host.env' "${common_directory%/*}"
  else
    printf '%s/multi-host.env' "$(cd "$gpu_quiesce_script_directory/../.." && pwd)"
  fi
}

gpu_quiesce_process_start_time() { # <进程号>：打 /proc/<进程号>/stat 的第 22 段（开机后起始时刻）；进程不在交回 1
  local stat_line rest fields=()
  stat_line="$(cat "/proc/$1/stat" 2>/dev/null)" || return 1
  rest="${stat_line##*) }"
  read -r -a fields <<<"$rest"
  printf '%s' "${fields[19]:-}"
}

gpu_quiesce_other_live_holders() { # 打 holders/ 下别的活着的登记数；死了的登记删掉（在锁里调）
  local holder_file holder_entry holder_pid holder_start live=0
  for holder_file in "$gpu_quiesce_state_directory/holders"/*; do
    [[ -e "$holder_file" ]] || continue
    holder_entry="${holder_file##*/}"
    [[ "$holder_entry" == "$gpu_quiesce_holder_name" ]] && continue
    holder_pid="${holder_entry%%-*}"; holder_start="${holder_entry#*-}"
    if [[ "$(gpu_quiesce_process_start_time "$holder_pid")" == "$holder_start" ]] || gpu_quiesce_break_is trust-stale-holders; then
      live=$((live + 1))
    else
      rm -f -- "$holder_file"
    fi
  done
  printf '%s' "$live"
}

gpu_quiesce_local_cards_of() { # <显卡先后的值>：打其中 local 那几项的序号（逗号隔开，按出现次序、去重）；有一项写法不对或一项本机卡都没有交回 1
  local entries=() entry cards=","
  read -r -a entries <<<"$1"
  for entry in ${entries[@]+"${entries[@]}"}; do
    [[ "$entry" =~ ^(local|peer):([0-9]{1,9}):([0-9]{1,15})[GM]$ ]] || return 1
    (( 10#${BASH_REMATCH[3]} > 0 )) || return 1
    [[ "${BASH_REMATCH[1]}" == local ]] || continue
    [[ "$cards" == *",$((10#${BASH_REMATCH[2]})),"* ]] || cards+="$((10#${BASH_REMATCH[2]})),"
  done
  cards="${cards#,}"; cards="${cards%,}"
  [[ -n "$cards" ]] || return 1
  printf '%s' "$cards"
}

gpu_quiesce_holder_descendants() { # 打别的活着的登记各自的进程号与它们这一刻的全部子孙（/proc 的父进程号链），空格隔开；不按名字找进程（在锁里调）
  local holder_file holder_entry holder_pid stat_file stat_line rest pid child root index fields=() roots=() found=()
  local -A children=() session_of=() seen=()
  for holder_file in "$gpu_quiesce_state_directory/holders"/*; do
    [[ -e "$holder_file" ]] || continue
    holder_entry="${holder_file##*/}"
    [[ "$holder_entry" == "$gpu_quiesce_holder_name" ]] && continue
    holder_pid="${holder_entry%%-*}"
    [[ "$holder_pid" =~ ^[0-9]+$ ]] || continue
    if [[ "$(gpu_quiesce_process_start_time "$holder_pid")" == "${holder_entry#*-}" ]] || gpu_quiesce_break_is trust-stale-holders; then roots+=("$holder_pid"); fi
  done
  (( ${#roots[@]} > 0 )) || return 0
  for stat_file in /proc/[0-9]*/stat; do
    { read -r stat_line <"$stat_file"; } 2>/dev/null || continue
    pid="${stat_file#/proc/}"; pid="${pid%/stat}"
    rest="${stat_line##*) }"
    read -r -a fields <<<"$rest"
    # 去掉「进程号 (名字) 」之后：0 状态、1 父进程号、2 进程组、3 会话号
    children[${fields[1]:-0}]+=" $pid"
    session_of[$pid]="${fields[3]:-}"
  done
  found=("${roots[@]}")
  for ((index = 0; index < ${#found[@]}; index++)); do
    pid="${found[index]}"
    [[ -n "${seen[$pid]:-}" ]] && continue
    seen[$pid]=1
    for child in ${children[$pid]:-}; do found+=("$child"); done
  done
  if gpu_quiesce_break_is exempt-holder-session; then
    for pid in "${!session_of[@]}"; do
      for root in "${roots[@]}"; do
        [[ -n "${session_of[$root]:-}" && "${session_of[$pid]}" == "${session_of[$root]}" ]] && seen[$pid]=1
      done
    done
  fi
  printf '%s\n' "${!seen[@]}" | LC_ALL=C sort -n | tr '\n' ' ' | sed 's/ $//'
}

# 回读看的卡（文件头「回读看哪几张卡、跳过哪些进程」）：置 gpu_quiesce_local_cards（取不到为空）与 gpu_quiesce_cards_said（给标准错误那一句）
gpu_quiesce_choose_cards() {
  local key=GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED value source
  gpu_quiesce_local_cards=""
  if gpu_quiesce_break_is check-every-card; then gpu_quiesce_cards_said="本机所有卡"; return 0; fi
  if [[ -n "${GPU_CARDS_BY_PRIORITY+set}" ]]; then
    value="$GPU_CARDS_BY_PRIORITY"; source="环境变量 GPU_CARDS_BY_PRIORITY"
  else
    gpu_quiesce_break_is cards-from-unquiesced-key && key=GPU_CARDS_BY_PRIORITY
    value="$(crash_amplification_configuration_value "$gpu_quiesce_configuration" "$key")"; source="多机配置的 $key"
  fi
  if [[ -z "${value//[[:space:]]/}" ]]; then
    gpu_quiesce_cards_said="本机所有卡（环境变量 GPU_CARDS_BY_PRIORITY 与多机配置的 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED 都没点名卡）"
  elif gpu_quiesce_local_cards="$(gpu_quiesce_local_cards_of "$value")"; then
    gpu_quiesce_cards_said="本机卡 $gpu_quiesce_local_cards（$source）"
  else
    gpu_quiesce_local_cards=""
    gpu_quiesce_cards_said="本机所有卡（$source 有一项写法不对、或一张本机卡都没点名）"
  fi
}

gpu_quiesce_enter_lock() {
  exec {gpu_quiesce_lock_fd}>>"$gpu_quiesce_state_directory/lock" || return 1
  flock -w "${GPU_QUIESCE_LOCK_SECONDS:-900}" "$gpu_quiesce_lock_fd"
}
gpu_quiesce_leave_lock() {
  flock -u "$gpu_quiesce_lock_fd" 2>/dev/null
  exec {gpu_quiesce_lock_fd}>&-
}

gpu_quiesce_run_configured() { # <键>：bash -c 跑配置里那条命令，命令与输出不打到标准错误，输出追加进本机日志；交回它的退出码
  #   环境另加回读看的卡与跳过的进程号（gpu_quiesce_check_environment，进来时在锁里置；没置的那一样不设）
  local code
  if gpu_quiesce_break_is failure-log-on-stderr; then env ${gpu_quiesce_check_environment[@]+"${gpu_quiesce_check_environment[@]}"} bash -c "${gpu_quiesce_commands[$1]}" </dev/null >&2; return; fi
  printf '== %s 起 %s\n' "$1" "$(date -u +%FT%TZ)" >>"$gpu_quiesce_log_file"
  env ${gpu_quiesce_check_environment[@]+"${gpu_quiesce_check_environment[@]}"} bash -c "${gpu_quiesce_commands[$1]}" </dev/null >>"$gpu_quiesce_log_file" 2>&1; code=$?
  printf '== %s 退 %s\n' "$1" "$code" >>"$gpu_quiesce_log_file"
  return "$code"
}

gpu_quiesce_keep_log() { # 留下本机日志：gpu_quiesce_log_field 置成 GPU_QUIESCE 行末尾的「 log=<路径>」（开关 no-failure-log 打开时不留、置空）
  gpu_quiesce_log_field=""
  gpu_quiesce_break_is no-failure-log && return 0
  gpu_quiesce_log_kept=1
  gpu_quiesce_log_field=" log=$gpu_quiesce_log_file"
}
gpu_quiesce_finish_log() { # 出口：没留下的本机日志删掉
  [[ -n "${gpu_quiesce_log_file:-}" ]] || return 0
  (( gpu_quiesce_log_kept )) || rm -f -- "$gpu_quiesce_log_file"
}

# 出去：删自己的登记，不复原（文件头「清场」）：打一行 step=restore result=not-restored 与手动复原的写法。
#   弄坏开关 restore-after-run 下照改前在出去时复原并回读（只给证红用）
gpu_quiesce_leave() {
  (( gpu_quiesce_registered )) || return 0
  gpu_quiesce_registered=0
  local others="unknown" restore_key
  if gpu_quiesce_enter_lock; then
    rm -f -- "$gpu_quiesce_state_directory/holders/$gpu_quiesce_holder_name"
    others="$(gpu_quiesce_other_live_holders)"
    gpu_quiesce_leave_lock
  else
    echo "  · ${GPU_QUIESCE_LOCK_SECONDS:-900} 秒没拿到清场锁 $gpu_quiesce_state_directory/lock，自己的登记没删：下一趟进来时认出它是死的、删掉" >&2
  fi
  if gpu_quiesce_break_is restore-after-run; then
    for restore_key in "${GPU_QUIESCE_RESTORE_KEYS[@]}"; do
      gpu_quiesce_commands[$restore_key]="$(crash_amplification_configuration_value "$gpu_quiesce_configuration" "$restore_key")"
      [[ -n "${gpu_quiesce_commands[$restore_key]}" ]] && gpu_quiesce_run_configured "$restore_key"
    done
    gpu_quiesce_line "step=restore result=restored holders=$others"
    return 0
  fi
  gpu_quiesce_line "step=restore result=not-restored holders=$others"
  echo "  · 清了场、没复原：清场停下的服务一直停着；要复原手动跑私有复原脚本（多机配置的 QUIESCE_START_COMMAND），再跑 QUIESCE_STARTED_CHECK_COMMAND 回读到它退 0" >&2
}

run_selftest() {
  local work failures=0 cells=0 nested=0
  crash_amplification_switch_wheel_inner && nested=1
  work="$(mktemp -d "${TMPDIR:-/tmp}/gpu-quiesce-run-selftest.XXXXXX")"
  trap 'rm -rf -- "${work:?}"' EXIT
  # 本机日志一律落进这里的临时目录（run_wrapper 再按状态目录各给一份），不落进真工作根；格 failure-log 里查默认位置的那一趟另把工作根指到临时目录
  export GPU_QUIESCE_LOG_DIRECTORY="$work/logs"
  # 假的清场命令：每一步记一笔进 FAKE_CALLS；FAKE_STOPPED_CHECK / FAKE_STARTED_CHECK 写成 fail 时那一步回读不过；FAKE_STOP_EXIT 是清场命令的退出码（默认 0）；
  #   设了 FAKE_GPU_PROCESSES 的，回读再过一道假的「卡上的计算进程」（gpu-check.sh）
  printf '%s\n' \
    "QUIESCE_STOP_COMMAND='echo stop >> \"\$FAKE_CALLS\"; echo fake-private-stop-output; exit \"\${FAKE_STOP_EXIT:-0}\"'" \
    "QUIESCE_STOPPED_CHECK_COMMAND='echo stopped-check >> \"\$FAKE_CALLS\"; echo fake-private-unit-stdout; echo fake-private-unit-stderr >&2; [[ \"\${FAKE_STOPPED_CHECK:-ok}\" == ok ]] && { [[ -z \"\${FAKE_GPU_PROCESSES:-}\" ]] || bash \"$work/gpu-check.sh\"; }'" \
    "QUIESCE_START_COMMAND='echo start >> \"\$FAKE_CALLS\"'" \
    "QUIESCE_STARTED_CHECK_COMMAND='echo started-check >> \"\$FAKE_CALLS\"; [[ \"\${FAKE_STARTED_CHECK:-ok}\" == ok ]]'" >"$work/multi-host.env"
  grep -v '^QUIESCE_STOPPED_CHECK_COMMAND=' "$work/multi-host.env" >"$work/multi-host-missing.env"
  # 假的「卡上的计算进程」回读（各格 holder-descendants、outsider-not-exempt、unused-card-outsider、used-card-outsider）：FAKE_GPU_PROCESSES 每行「<本机卡序号> <进程号>」，
  #   照这一份给的 GPU_QUIESCE_CHECK_LOCAL_CARDS 只看点名的卡、跳过 GPU_QUIESCE_CHECK_EXEMPT_PIDS 里的进程号，剩下一个就退 1；那一刻看到的两个变量记进 FAKE_CHECK_SEEN
  cat >"$work/gpu-check.sh" <<'STUB'
#!/usr/bin/env bash
echo "cards=${GPU_QUIESCE_CHECK_LOCAL_CARDS-unset} exempt=${GPU_QUIESCE_CHECK_EXEMPT_PIDS-unset}" >> "${FAKE_CHECK_SEEN:-/dev/null}"
residue=0
while read -r card pid; do
  [[ -n "$card" ]] || continue
  [[ -n "${GPU_QUIESCE_CHECK_LOCAL_CARDS:-}" && ",$GPU_QUIESCE_CHECK_LOCAL_CARDS," != *",$card,"* ]] && continue
  [[ " ${GPU_QUIESCE_CHECK_EXEMPT_PIDS:-} " == *" $pid "* ]] && continue
  echo "fake-gpu-residue card=$card pid=$pid"; residue=1
done <"${FAKE_GPU_PROCESSES:?}"
exit "$residue"
STUB
  # 显卡先后写在多机配置里的一份（格 used-card-outsider）：清场之后那一份点名本机卡 0，不清场那一份点名本机卡 1
  { cat "$work/multi-host.env"; printf '%s\n' "GPU_CARDS_BY_PRIORITY='local:1:8G'" "GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED='peer:1:16G local:0:24G'"; } >"$work/multi-host-cards.env"
  # 复原那一对没写的多机配置（格 no-restore）：这一份不读它们，照跑
  grep -vE '^QUIESCE_START(ED_CHECK)?_COMMAND=' "$work/multi-host.env" >"$work/multi-host-without-restore.env"
  # 假的命令：<模式> 记开始与结束；fail 退 7，self-term 自己收 TERM 退出，wait-for:<文件> 等那份文件出现（至多 10 秒），sleep:<秒> 睡一会儿，
  #   hold <进程号文件> <放行文件> 起一个子进程（这一份的孙辈）、把它的进程号写进 <进程号文件>，等它见到 <放行文件>（至多 30 秒）退出
  cat >"$work/command.sh" <<'STUB'
#!/usr/bin/env bash
echo "command-$1-start" >> "${FAKE_CALLS:?}"
# 核 GPU_QUIESCE_HOLDER：文件在、文件名以父进程（包装）的进程号加「-」起头
if [[ "$1" == ok && -e "${GPU_QUIESCE_HOLDER:-}" && "${GPU_QUIESCE_HOLDER##*/}" == "$PPID-"* ]]; then echo holder-of-parent >> "$FAKE_CALLS"; fi
case "$1" in
  fail) echo "command-$1-end" >> "$FAKE_CALLS"; exit 7 ;;
  self-term) kill -s TERM "$$" ;;
  wait-for:*) for ((round = 0; round < 200; round++)); do [[ -e "${1#wait-for:}" ]] && break; sleep 0.05; done ;;
  sleep:*) sleep "${1#sleep:}" ;;
  hold) bash -c 'for ((round = 0; round < 600; round++)); do [[ -e "$1" ]] && break; sleep 0.05; done' hold-child "$3" &
        echo "$!" >"$2"; wait "$!" ;;
esac
echo "command-$1-end" >> "$FAKE_CALLS"
STUB
  selftest_cell_failed() { # <格名> <这一格判错的一句> <出路>
    echo "  ✗ 自证 $1：$2" >&2
    echo "     → 怎么办：$3" >&2
    failures=$((failures + 1))
  }
  run_wrapper() { # <calls 文件> <状态目录> [<环境 键=值>…] -- <命令…>：起一趟这一份，输出写进 <calls 文件>.out，本机日志进 <状态目录>/logs
    local calls="$1" state="$2"; shift 2
    local environment=()
    while (($#)) && [[ "$1" != -- ]]; do environment+=("$1"); shift; done
    # 调用方环境里的显卡先后与回读握手变量不带进来，要的格自己给
    env -u GPU_CARDS_BY_PRIORITY -u GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED -u GPU_QUIESCE_CHECK_LOCAL_CARDS -u GPU_QUIESCE_CHECK_EXEMPT_PIDS \
      SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host.env" GPU_QUIESCE_STATE_DIRECTORY="$state" GPU_QUIESCE_LOG_DIRECTORY="$state/logs" FAKE_CALLS="$calls" ${environment[@]+"${environment[@]}"} \
      bash "$0" "$@" >"$calls.out" 2>&1
  }
  # 调用次序（复原那两笔 start / started-check 不列：跑完调没调复原命令只由格 no-restore 判）
  calls_of() { { grep -vxE 'start|started-check' "$1" || true; } | tr '\n' ' '; }
  wait_for_line() { # <文件> <行>：至多等 10 秒那一行出现
    local round
    for ((round = 0; round < 200; round++)); do grep -qx -- "$2" "$1" 2>/dev/null && return 0; sleep 0.05; done
    return 1
  }
  local cell_exit calls no_restore_output

  # 格 quiesce-run：清场、回读、命令，次序对，退命令的 0；输出里不带配置里的命令
  cells=$((cells + 1)); calls="$work/green.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-green" -- bash "$work/command.sh" ok; cell_exit=$?
  if [[ "$cell_exit" != 0 || "$(calls_of "$calls")" != "stop stopped-check command-ok-start holder-of-parent command-ok-end " ]] \
     || ! grep -q '^GPU_QUIESCE step=quiesce result=stopped-and-read-back holders=0$' "$calls.out" \
     || grep -q 'FAKE_CALLS' "$calls.out"; then
    selftest_cell_failed quiesce-run "没有照「清场、回读、命令」走、没退 0，或输出里带了配置里的命令（退 $cell_exit，次序「$(calls_of "$calls")」）" "看主流程 gpu_quiesce_enter / gpu_quiesce_leave 与 gpu_quiesce_run_configured（命令与输出都不打）"
  fi
  # 格 missing-commands：多机配置里少 QUIESCE_STOPPED_CHECK_COMMAND ⇒ 退 78，命令没起、清场命令一条都没跑
  cells=$((cells + 1)); calls="$work/missing.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-missing" SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-missing.env" -- bash "$work/command.sh" ok; cell_exit=$?
  if [[ "$cell_exit" != 78 || -s "$calls" ]] || ! grep -q 'QUIESCE_STOPPED_CHECK_COMMAND' "$calls.out"; then
    selftest_cell_failed missing-commands "多机配置没写全清场命令却没退 78、点名缺的键，或照跑了（退 $cell_exit，次序「$(calls_of "$calls")」）" "看读配置那一段：清场那两个键缺一个就退 78、不起命令"
  fi
  # 格 quiesce-read-back-fails：清场回读不过 ⇒ 命令不起、退 78；清场命令跑过
  cells=$((cells + 1)); calls="$work/stop-fails.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-stop-fails" FAKE_STOPPED_CHECK=fail -- bash "$work/command.sh" ok; cell_exit=$?
  if [[ "$cell_exit" != 78 || "$(calls_of "$calls")" != "stop stopped-check " ]] || ! grep -q '^GPU_QUIESCE step=command result=not-started$' "$calls.out"; then
    selftest_cell_failed quiesce-read-back-fails "清场回读没过却起了命令、没退 78（退 $cell_exit，次序「$(calls_of "$calls")」）" "看 gpu_quiesce_enter 回读那一支：不过就不登记命令、照样走 gpu_quiesce_leave"
  fi
  # 格 stop-exit：清场命令退 1 而清场回读过了 ⇒ 照跑、退命令的 0，
  #   清场那一行是 stopped-and-read-back 带 stop-exit=1；清场回读不过那一趟（格 quiesce-read-back-fails）带 stop-exit=0
  cells=$((cells + 1)); calls="$work/stop-refused.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-stop-refused" FAKE_STOP_EXIT=1 -- bash "$work/command.sh" ok; cell_exit=$?
  if [[ "$cell_exit" != 0 || "$(calls_of "$calls")" != "stop stopped-check command-ok-start holder-of-parent command-ok-end " ]] \
     || ! grep -Eq '^GPU_QUIESCE step=quiesce result=stopped-and-read-back holders=0 stop-exit=1( |$)' "$calls.out" \
     || ! grep -Eq '^GPU_QUIESCE step=quiesce result=read-back-failed holders=0 stop-exit=0( |$)' "$work/stop-fails.calls.out"; then
    selftest_cell_failed stop-exit "清场命令退 1 而回读过了，清场那一行没带 stop-exit=1、没照跑或没退 0，或清场回读不过那一行没带 stop-exit=0（退 $cell_exit，次序「$(calls_of "$calls")」，清场那一行「$(grep '^GPU_QUIESCE step=quiesce ' "$calls.out")」）" \
      "看清场那一段的 gpu_quiesce_stop_exit 与 gpu_quiesce_stop_field：这一趟跑了清场命令、而回读没过或它退非 0 时，那一行带 stop-exit=<码>"
  fi
  # 格 failure-log：清场回读不过 ⇒ GPU_QUIESCE 清场那一行带 log=<本机日志>；日志 600、所在目录 700，里面有回读命令的两路输出与「退 1」那一行；
  #   清场命令退非 0 而回读过了（格 stop-exit 那一趟）同样留下、带 log=；私有输出不上标准错误；全绿那一趟（格 quiesce-run 的状态目录）不留日志；
  #   不设 GPU_QUIESCE_LOG_DIRECTORY 时日志在 <工作根>/gpu-quiesce-logs/ 下（工作根指到这里的临时目录），不在 XDG_RUNTIME_DIR 下
  cells=$((cells + 1))
  local failure_log failure_mode="" failure_directory_mode="" refused_log default_log runtime_files
  failure_log="$(sed -nE 's/^GPU_QUIESCE step=quiesce result=read-back-failed .* log=([^ ]+)$/\1/p' "$work/stop-fails.calls.out")"
  if [[ -n "$failure_log" && -f "$failure_log" ]]; then
    failure_mode="$(stat -c %a -- "$failure_log")"; failure_directory_mode="$(stat -c %a -- "${failure_log%/*}")"
  fi
  refused_log="$(sed -nE 's/^GPU_QUIESCE step=quiesce result=stopped-and-read-back .* log=([^ ]+)$/\1/p' "$work/stop-refused.calls.out")"
  mkdir -p "$work/runtime"
  env -u GPU_QUIESCE_LOG_DIRECTORY SINGLEFS_WORK_ROOT="$work/work-root" XDG_RUNTIME_DIR="$work/runtime" SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host.env" \
    GPU_QUIESCE_STATE_DIRECTORY="$work/state-default-log" FAKE_CALLS="$work/default-log.calls" FAKE_STOPPED_CHECK=fail \
    bash "$0" -- bash "$work/command.sh" ok >"$work/default-log.calls.out" 2>&1
  default_log="$(sed -nE 's/^GPU_QUIESCE step=quiesce result=read-back-failed .* log=([^ ]+)$/\1/p' "$work/default-log.calls.out")"
  runtime_files="$(find "$work/runtime" -type f | wc -l)"
  if [[ "$failure_mode" != 600 || "$failure_directory_mode" != 700 ]] || ! grep -q '^fake-private-unit-stdout$' "$failure_log" || ! grep -q '^fake-private-unit-stderr$' "$failure_log" \
     || ! grep -q '^fake-private-stop-output$' "$failure_log" || ! grep -q '^== QUIESCE_STOPPED_CHECK_COMMAND 退 1$' "$failure_log" \
     || [[ -z "$refused_log" || ! -f "$refused_log" ]] || ! grep -q '^== QUIESCE_STOP_COMMAND 退 1$' "$refused_log" \
     || grep -q 'fake-private-' "$work/stop-fails.calls.out" "$work/green.calls.out" "$work/stop-refused.calls.out" "$work/default-log.calls.out" \
     || compgen -G "$work/state-green/logs/*" >/dev/null \
     || [[ "$default_log" != "$work/work-root/gpu-quiesce-logs/"* || ! -f "$default_log" || "$runtime_files" != 0 ]]; then
    selftest_cell_failed failure-log "回读没过（或清场命令退非 0）没留本机日志、日志不是 600 或目录不是 700、缺清场与回读命令的输出、私有输出打到了标准错误上、全绿那一趟留了日志，或默认位置不在工作根下（清场回读不过的日志「${failure_log:-没打 log=}」权限「${failure_mode:-无}」目录「${failure_directory_mode:-无}」；清场退 1 的日志「${refused_log:-没打 log=}」；默认位置「${default_log:-没打 log=}」，XDG_RUNTIME_DIR 下 $runtime_files 份）" \
      "看 gpu_quiesce_run_configured（输出进本机日志）、gpu_quiesce_keep_log、gpu_quiesce_finish_log 与取日志目录那一段（默认 <工作根>/gpu-quiesce-logs，工作根经 .claude/hooks/lib_work_root.py 取）"
  fi
  # 格 command-fails：命令退 7 ⇒ 交回 7
  cells=$((cells + 1)); calls="$work/fail.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-fail" -- bash "$work/command.sh" fail; cell_exit=$?
  if [[ "$cell_exit" != 7 || "$(calls_of "$calls")" != "stop stopped-check command-fail-start command-fail-end " ]]; then
    selftest_cell_failed command-fails "命令失败之后没交回命令的退出码 7（退 $cell_exit，次序「$(calls_of "$calls")」）" "看末尾的退出码：命令起了就交回它的"
  fi
  # 格 interrupted：① 命令跑着时这一份收到 TERM（kill 自己起的那一份的进程号）⇒ 等命令结束、退 143；② 命令自己被 TERM 打断 ⇒ 退 143
  cells=$((cells + 1)); calls="$work/term.calls"; : >"$calls"
  # 直接起这一份（不经 run_wrapper 那层函数），$! 才是这一份自己的进程号
  env SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host.env" GPU_QUIESCE_STATE_DIRECTORY="$work/state-term" FAKE_CALLS="$calls" \
    bash "$0" -- bash "$work/command.sh" sleep:1 >"$calls.out" 2>&1 &
  local wrapper_pid=$! interrupted_exit=""
  if wait_for_line "$calls" "command-sleep:1-start"; then kill -TERM "$wrapper_pid"; fi
  wait "$wrapper_pid"; interrupted_exit=$?
  calls="$work/self-term.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-self-term" -- bash "$work/command.sh" self-term; cell_exit=$?
  if [[ "$interrupted_exit" != 143 || "$(calls_of "$work/term.calls")" != "stop stopped-check command-sleep:1-start command-sleep:1-end " ]] \
     || [[ "$cell_exit" != 143 || "$(calls_of "$calls")" != "stop stopped-check command-self-term-start " ]]; then
    selftest_cell_failed interrupted "被信号打断之后没等命令结束，或没退 143（这一份收到 TERM：退 $interrupted_exit，次序「$(calls_of "$work/term.calls")」；命令被 TERM：退 $cell_exit，次序「$(calls_of "$calls")」）" \
      "看 INT/TERM/HUP 的 trap（只记下信号、等前台命令结束）与末尾的退出码"
  fi
  # 格 overlapping-runs：A 先进来清场，B 跟着进来只回读（joined-and-read-back holders=1）；清场命令只跑一次，两趟都退 0
  cells=$((cells + 1)); calls="$work/overlap.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-overlap" -- bash "$work/command.sh" "wait-for:$work/b-started" &
  local first_pid=$! first_exit second_exit
  wait_for_line "$calls" "command-wait-for:$work/b-started-start"
  cp -- "$calls" "$work/overlap.before-b"
  env SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host.env" GPU_QUIESCE_STATE_DIRECTORY="$work/state-overlap" FAKE_CALLS="$calls" \
    bash "$0" -- bash -c "touch '$work/b-started'; for ((round = 0; round < 200; round++)); do [[ -e '$work/a-done' ]] && break; sleep 0.05; done; echo command-b-end >> '$calls'" \
    >"$work/overlap-b.out" 2>&1 &
  local second_pid=$!
  wait "$first_pid"; first_exit=$?
  touch "$work/a-done"
  wait "$second_pid"; second_exit=$?
  local overlap_calls; overlap_calls="$(calls_of "$calls")"
  if [[ "$first_exit" != 0 || "$second_exit" != 0 || "$(grep -cx stop "$calls")" != 1 ]] \
     || [[ "$overlap_calls" != *"command-b-end " ]] || ! grep -q '^GPU_QUIESCE step=quiesce result=joined-and-read-back holders=1$' "$work/overlap-b.out"; then
    selftest_cell_failed overlapping-runs "两趟重叠时没有只清一次、后来的那一趟没跟着用（A 退 $first_exit、B 退 $second_exit，次序「$overlap_calls」）" "看 gpu_quiesce_enter 里 gpu_quiesce_other_live_holders 那一处"
  fi
  # 格 stale-holder：holders/ 下有一份死了的登记（进程号已经不在）⇒ 不当它在用，照常清场，并删掉它
  cells=$((cells + 1)); calls="$work/stale.calls"; : >"$calls"
  mkdir -p "$work/state-stale/holders"
  bash -c 'exit 0' & local dead_pid=$!; wait "$dead_pid"
  : >"$work/state-stale/holders/$dead_pid-1"
  run_wrapper "$calls" "$work/state-stale" -- bash "$work/command.sh" ok; cell_exit=$?
  if [[ "$cell_exit" != 0 || "$(calls_of "$calls")" != "stop stopped-check command-ok-start holder-of-parent command-ok-end " || -e "$work/state-stale/holders/$dead_pid-1" ]]; then
    selftest_cell_failed stale-holder "死了的登记挡住了清场、或没删掉（退 $cell_exit，次序「$(calls_of "$calls")」）" "看 gpu_quiesce_other_live_holders：进程不在或起始时刻对不上的登记删掉、不计数"
  fi
  # 格 no-restore：跑完、跑红、清场回读没过、被打断都不调复原命令：上面各格那几趟的调用记录里一笔 start / started-check 都没有，
  #   出去时那一行是 step=restore result=not-restored；多机配置里没写复原那一对也照跑；同一个状态目录上一趟清了场没复原、
  #   接着再起一趟照常清场、回读、跑（服务已经停着时私有清场脚本只回读、退 0，这里的假清场命令照样退 0）
  cells=$((cells + 1)); calls="$work/no-restore.calls"; : >"$calls"
  local no_restore_first_exit restore_calls restore_lines_missing=""
  run_wrapper "$calls" "$work/state-no-restore" -- bash "$work/command.sh" ok; no_restore_first_exit=$?
  cp -- "$calls.out" "$work/no-restore-first.out"
  run_wrapper "$calls" "$work/state-no-restore" SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-without-restore.env" -- bash "$work/command.sh" ok; cell_exit=$?
  restore_calls="$(cat -- "$work"/*.calls | grep -cxE 'start|started-check')"
  for no_restore_output in green.calls.out fail.calls.out stop-fails.calls.out term.calls.out self-term.calls.out no-restore-first.out no-restore.calls.out; do
    grep -q '^GPU_QUIESCE step=restore result=not-restored holders=0$' "$work/$no_restore_output" || restore_lines_missing+="$no_restore_output "
  done
  if [[ "$no_restore_first_exit" != 0 || "$cell_exit" != 0 || "$restore_calls" != 0 || -n "$restore_lines_missing" ]] \
     || [[ "$(tr '\n' ' ' <"$calls")" != "stop stopped-check command-ok-start holder-of-parent command-ok-end stop stopped-check command-ok-start holder-of-parent command-ok-end " ]] \
     || ! grep -q '清了场、没复原' "$work/no-restore.calls.out"; then
    selftest_cell_failed no-restore "跑完调了复原命令（各格的调用记录里 start / started-check 共 $restore_calls 笔），或出去时那一行不是 result=not-restored（缺的：${restore_lines_missing:-无}），或没写复原那一对的配置、接着上一趟再起一趟没照跑（两趟退 $no_restore_first_exit、$cell_exit，次序「$(tr '\n' ' ' <"$calls")」）" \
      "看 gpu_quiesce_leave：只删自己的登记、打 step=restore result=not-restored，不跑 QUIESCE_START_COMMAND；读配置那一段只要清场那两个键"
  fi
  # 格 usage：没给 -- 或 -- 后面没有命令 ⇒ 退 2，什么都不跑
  cells=$((cells + 1)); calls="$work/usage.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-usage" -- ; cell_exit=$?
  local usage_second_exit
  env SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host.env" GPU_QUIESCE_STATE_DIRECTORY="$work/state-usage" FAKE_CALLS="$calls" \
    bash "$0" bash "$work/command.sh" ok >"$calls.out" 2>&1; usage_second_exit=$?
  if [[ "$cell_exit" != 2 || "$usage_second_exit" != 2 || -s "$calls" ]]; then
    selftest_cell_failed usage "用法错没退 2（-- 后面空着退 $cell_exit，没给 -- 退 $usage_second_exit），或跑了东西" "看参数那一段"
  fi

  # 回读看哪几张卡、跳过哪些进程（文件头那一段）四格。外人是这里起的一个进程：不在任何登记之下、与登记同一个会话，见到 outsider-release 退出（至多 60 秒）。
  #   回读过没过只看 GPU_QUIESCE 清场那一行（回读不过还起不起命令归格 quiesce-read-back-fails）
  local round outsider_pid holder_child first_holder_pid holder_exit descendants_exit outsider_seen
  seen_exempt() { printf ' %s ' "$(sed -n 's/^cards=[^ ]* exempt=//p' "$1" 2>/dev/null | head -n 1)"; }
  bash -c 'for ((round = 0; round < 1200; round++)); do [[ -e "$1" ]] && break; sleep 0.05; done' outsider "$work/outsider-release" &
  outsider_pid=$!
  # 格 holder-descendants：前一趟 A 在用卡（登记活着），它的孙辈进程在卡 0 上；B 进来只回读 ⇒ 回读过（joined-and-read-back holders=1）、退 0，
  #   回读拿到的 GPU_QUIESCE_CHECK_EXEMPT_PIDS 里有那个孙辈；A 放行之后退 0
  # 格 outsider-not-exempt：A 还在用卡，卡 0 上另有外人 ⇒ B 回读不过（read-back-failed），跳过的进程号里没有外人
  cells=$((cells + 2)); : >"$work/holder.calls"
  run_wrapper "$work/holder.calls" "$work/state-holders" -- bash "$work/command.sh" hold "$work/holder-child.pid" "$work/holder-release" &
  first_holder_pid=$!
  for ((round = 0; round < 200; round++)); do [[ -s "$work/holder-child.pid" ]] && break; sleep 0.05; done
  holder_child="$(cat -- "$work/holder-child.pid" 2>/dev/null)"
  printf '0 %s\n' "$holder_child" >"$work/gpu-descendant.list"
  printf '0 %s\n0 %s\n' "$holder_child" "$outsider_pid" >"$work/gpu-outsider.list"
  calls="$work/descendants.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-holders" FAKE_GPU_PROCESSES="$work/gpu-descendant.list" FAKE_CHECK_SEEN="$work/descendants.seen" -- bash "$work/command.sh" ok; descendants_exit=$?
  calls="$work/outsider.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-holders" FAKE_GPU_PROCESSES="$work/gpu-outsider.list" FAKE_CHECK_SEEN="$work/outsider.seen" -- bash "$work/command.sh" ok
  touch "$work/holder-release"
  wait "$first_holder_pid"; holder_exit=$?
  if [[ -z "$holder_child" || "$holder_exit" != 0 || "$descendants_exit" != 0 ]] \
     || ! grep -q '^GPU_QUIESCE step=quiesce result=joined-and-read-back holders=1$' "$work/descendants.calls.out" \
     || [[ "$(seen_exempt "$work/descendants.seen")" != *" $holder_child "* ]]; then
    selftest_cell_failed holder-descendants "前一趟登记下的进程在卡上，后一趟进来回读没过（A 的孙辈「${holder_child:-没起来}」，A 退 $holder_exit，B 退 $descendants_exit，B 的清场那一行「$(grep '^GPU_QUIESCE step=quiesce ' "$work/descendants.calls.out")」，回读看到的「$(head -n 1 "$work/descendants.seen" 2>/dev/null)」）" \
      "看 gpu_quiesce_holder_descendants（别的活着的登记与它们的子孙，按 /proc 的父进程号往下找）与进来那一段：它们经 GPU_QUIESCE_CHECK_EXEMPT_PIDS 交给回读"
  fi
  outsider_seen="$(seen_exempt "$work/outsider.seen")"
  if ! grep -Eq '^GPU_QUIESCE step=quiesce result=read-back-failed holders=1( |$)' "$work/outsider.calls.out" \
     || [[ ! -s "$work/outsider.seen" || "$outsider_seen" == *" $outsider_pid "* ]]; then
    selftest_cell_failed outsider-not-exempt "不在任何登记之下的进程在卡上，回读却过了（外人 $outsider_pid，清场那一行「$(grep '^GPU_QUIESCE step=quiesce ' "$work/outsider.calls.out")」，回读看到的「$(head -n 1 "$work/outsider.seen" 2>/dev/null)」）" \
      "看 gpu_quiesce_holder_descendants：只跳过别的活着的登记自己与它们的子孙，不按会话、进程组或名字认自家的"
  fi
  # 格 unused-card-outsider：没有别的登记，这一趟的 GPU_CARDS_BY_PRIORITY 是「peer:0:8G local:1:8G」，外人在本机卡 0 上 ⇒ 回读只看本机卡 1、过了，照跑、退 0
  printf '0 %s\n' "$outsider_pid" >"$work/gpu-card0.list"
  cells=$((cells + 1)); calls="$work/unused-card.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-unused-card" GPU_CARDS_BY_PRIORITY='peer:0:8G local:1:8G' FAKE_GPU_PROCESSES="$work/gpu-card0.list" FAKE_CHECK_SEEN="$work/unused-card.seen" -- bash "$work/command.sh" ok; cell_exit=$?
  if [[ "$cell_exit" != 0 || "$(head -n 1 "$work/unused-card.seen" 2>/dev/null)" != "cards=1 exempt=unset" ]] \
     || ! grep -q '^GPU_QUIESCE step=quiesce result=stopped-and-read-back holders=0$' "$calls.out" \
     || ! grep -q '回读看的卡：本机卡 1（环境变量 GPU_CARDS_BY_PRIORITY）' "$calls.out"; then
    selftest_cell_failed unused-card-outsider "这一趟只用卡 1，卡 0 上的外人挡住了回读（退 $cell_exit，清场那一行「$(grep '^GPU_QUIESCE step=quiesce ' "$calls.out")」，回读看到的「$(head -n 1 "$work/unused-card.seen" 2>/dev/null)」）" \
      "看 gpu_quiesce_choose_cards 与 gpu_quiesce_local_cards_of：GPU_CARDS_BY_PRIORITY 里 local 那几项的序号经 GPU_QUIESCE_CHECK_LOCAL_CARDS 交给回读，peer 的不算"
  fi
  # 格 used-card-outsider：没设 GPU_CARDS_BY_PRIORITY，多机配置清场之后那一份点名本机卡 0（不清场那一份点名卡 1），外人在卡 0 上；
  #   调用方另带进来 GPU_QUIESCE_CHECK_LOCAL_CARDS=1 与 GPU_QUIESCE_CHECK_EXEMPT_PIDS=<外人> ⇒ 不认它们，回读只看卡 0、不跳过外人、不过
  cells=$((cells + 1)); calls="$work/used-card.calls"; : >"$calls"
  run_wrapper "$calls" "$work/state-used-card" SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-cards.env" GPU_QUIESCE_CHECK_LOCAL_CARDS=1 GPU_QUIESCE_CHECK_EXEMPT_PIDS="$outsider_pid" \
    FAKE_GPU_PROCESSES="$work/gpu-card0.list" FAKE_CHECK_SEEN="$work/used-card.seen" -- bash "$work/command.sh" ok
  if [[ "$(head -n 1 "$work/used-card.seen" 2>/dev/null)" != "cards=0 exempt=unset" ]] \
     || ! grep -Eq '^GPU_QUIESCE step=quiesce result=read-back-failed holders=0( |$)' "$calls.out"; then
    selftest_cell_failed used-card-outsider "这一趟用卡 0，卡 0 上的外人却没挡住回读（清场那一行「$(grep '^GPU_QUIESCE step=quiesce ' "$calls.out")」，回读看到的「$(head -n 1 "$work/used-card.seen" 2>/dev/null)」）" \
      "看 gpu_quiesce_choose_cards（没设 GPU_CARDS_BY_PRIORITY 读多机配置的 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED）与参数之后清掉调用方带进来的 GPU_QUIESCE_CHECK_* 那一行"
  fi
  touch "$work/outsider-release"
  wait "$outsider_pid"

  if (( failures > 0 )); then
    echo "  ✗ --selftest：$cells 格里 $failures 格判错" >&2
    echo "     → 怎么办：照上面每一格给的出路改，再跑 $0 --selftest" >&2
    exit 1
  fi
  if (( nested )); then echo "  ✓ --selftest（开关轮里起的一份）：$cells 格都对"; exit 0; fi
  # 开关轮：文件头「弄坏开关」那张表现读，表里的开关与代码里认的开关要相等，逐个打开要红在点名的那一格
  local table assignment named round_failures=0 in_code in_table
  switch_wheel_assignments=(); switch_wheel_expected=()
  table="$(crash_amplification_read_switch_table "$0" GPU_QUIESCE_BREAK)"
  while IFS=$'\t' read -r assignment named; do
    [[ -n "$assignment" ]] || continue
    switch_wheel_assignments+=("$assignment"); switch_wheel_expected+=("$named")
  done <<<"$table"
  in_code="$(grep -oE 'gpu_quiesce_break_is [a-z0-9-]+' "$0" | awk '{ print $2 }' | LC_ALL=C sort -u)"
  in_table="$(printf '%s\n' ${switch_wheel_assignments[@]+"${switch_wheel_assignments[@]#*=}"} | LC_ALL=C sort -u)"
  if [[ -z "$table" || "$in_code" != "$in_table" ]]; then
    echo "  ✗ 开关轮：代码里认的开关与文件头表里列的不一样（代码里：$(tr '\n' ' ' <<<"$in_code")；表里：$(tr '\n' ' ' <<<"$in_table")）" >&2
    echo "     → 怎么办：代码里每一处 gpu_quiesce_break_is 认的开关都在文件头「弄坏开关」表里写一行，表里每一行都在代码里有对应的一处" >&2
    round_failures=$((round_failures + 1))
  fi
  crash_amplification_switch_wheel "$work" '✗ 自证 ([a-z-]+)：' 0 -- bash "$0" --selftest
  round_failures=$((round_failures + switch_wheel_failures))
  if (( round_failures > 0 )); then
    echo "  ✗ --selftest：$cells 格都对，开关轮有 $round_failures 处对不上" >&2
    echo "     → 怎么办：红的格与表里写的不同，要么改那一处让开关红在点名的格，要么照实改文件头那一行；单看一个开关：GPU_QUIESCE_BREAK=<开关> $0 --selftest" >&2
    exit 1
  fi
  echo "  ✓ --selftest：清场回读命令的次序、没写全清场命令退 78、清场回读不过不起命令、清场命令退非 0 而回读过了那一行带 stop-exit、回读没过或清场命令退非 0 时在工作根下留 600 的本机日志且输出不上标准错误、命令失败交回退出码、被信号打断等命令结束退 143、两趟重叠只清一次、死了的登记不挡、跑完不调复原命令且报 not-restored（没写复原那一对、接着上一趟再起都照跑）、用法错退 2、前一趟登记下的进程不挡后一趟回读而外人照挡、回读只看这一趟要用的本机卡（GPU_CARDS_BY_PRIORITY，没设取多机配置清场之后那一份；调用方带进来的回读变量不认），$cells 格都对；开关轮 ${#switch_wheel_assignments[@]} 个开关（文件头现读）逐个打开，收回 $switch_wheel_collected 个，各红在点名的那一格"
  exit 0
}

case "${1:-}" in
  --selftest) (($# == 1)) || usage; run_selftest ;;
  --) shift; (($# > 0)) || usage ;;
  *) usage ;;
esac

# 回读的两个握手变量只由这一份置、只给清场那两条命令：调用方带进来的清掉，不认、也不传给 <命令…>
gpu_quiesce_break_is keep-inherited-check-variables || unset GPU_QUIESCE_CHECK_LOCAL_CARDS GPU_QUIESCE_CHECK_EXEMPT_PIDS
gpu_quiesce_check_environment=()

# 读配置里清场那两个键：缺一个就不清场、不起命令，退 78
gpu_quiesce_configuration="$(gpu_quiesce_configuration_file)"
declare -A gpu_quiesce_commands=()
gpu_quiesce_missing=()
for gpu_quiesce_key in "${GPU_QUIESCE_KEYS[@]}"; do
  gpu_quiesce_commands[$gpu_quiesce_key]="$(crash_amplification_configuration_value "$gpu_quiesce_configuration" "$gpu_quiesce_key")"
  [[ -n "${gpu_quiesce_commands[$gpu_quiesce_key]}" ]] || gpu_quiesce_missing+=("$gpu_quiesce_key")
done
if (( ${#gpu_quiesce_missing[@]} > 0 )); then
  if gpu_quiesce_break_is accept-missing-quiesce-commands; then
    "$@"; exit $?
  fi
  gpu_quiesce_line "step=command result=not-started"
  gpu_quiesce_line "step=exit code=78"
  echo "  ✗ 多机配置 $gpu_quiesce_configuration 里没写 ${gpu_quiesce_missing[*]}（文件不在也算）：清不了场，命令没起" >&2
  echo "  → 怎么办：照仓根 multi-host.env.example 写全 QUIESCE_STOP_COMMAND 与 QUIESCE_STOPPED_CHECK_COMMAND（清场、清场回读；复原那一对这里不读）；配置在别处的设 SINGLEFS_MULTI_HOST_CONFIG 指到它（退 78）" >&2
  exit 78
fi

gpu_quiesce_state_directory="${GPU_QUIESCE_STATE_DIRECTORY:-${XDG_RUNTIME_DIR:-/tmp}/singlefs-gpu-quiesce}"
if ! mkdir -p -- "$gpu_quiesce_state_directory/holders"; then
  gpu_quiesce_line "step=command result=not-started"
  gpu_quiesce_line "step=exit code=78"
  echo "  ✗ 建不了清场的状态目录 $gpu_quiesce_state_directory/holders：命令没起" >&2
  echo "  → 怎么办：看 XDG_RUNTIME_DIR 可不可写，或设 GPU_QUIESCE_STATE_DIRECTORY 指到可写的目录；几趟同时用卡的要指同一个（退 78）" >&2
  exit 78
fi
gpu_quiesce_holder_name="$$-$(gpu_quiesce_process_start_time "$$")"
# 本机日志：清场那两条命令的输出进这里（目录 700、文件 600，日志目录开机不清）；取不到工作根、建不了就不起
gpu_quiesce_log_kept=0; gpu_quiesce_log_file=""
if [[ -n "${GPU_QUIESCE_LOG_DIRECTORY:-}" ]]; then gpu_quiesce_log_directory="$GPU_QUIESCE_LOG_DIRECTORY"
elif gpu_quiesce_break_is failure-log-in-runtime-dir; then gpu_quiesce_log_directory="${XDG_RUNTIME_DIR:-/tmp}/singlefs-gpu-quiesce/logs"
elif gpu_quiesce_work_root="$(python3 "$gpu_quiesce_script_directory/../../.claude/hooks/lib_work_root.py")" && [[ -n "$gpu_quiesce_work_root" ]]; then
  gpu_quiesce_log_directory="$gpu_quiesce_work_root/gpu-quiesce-logs"
else
  gpu_quiesce_line "step=command result=not-started"
  gpu_quiesce_line "step=exit code=78"
  echo "  ✗ 取不到工作根（.claude/hooks/lib_work_root.py 跑不起来），清场的本机日志没处放：命令没起" >&2
  echo "  → 怎么办：恢复那份库，或设 GPU_QUIESCE_LOG_DIRECTORY 指到开机不清的可写目录（退 78）" >&2
  exit 78
fi
gpu_quiesce_log_file="$gpu_quiesce_log_directory/$(TZ=Asia/Tokyo date +%F)-$gpu_quiesce_holder_name.log"
if ! { mkdir -p -m 700 -- "$gpu_quiesce_log_directory" && chmod 700 -- "$gpu_quiesce_log_directory" && ( umask 077 && : >"$gpu_quiesce_log_file" ); }; then
  gpu_quiesce_line "step=command result=not-started"
  gpu_quiesce_line "step=exit code=78"
  echo "  ✗ 建不了清场的本机日志 $gpu_quiesce_log_file：命令没起" >&2
  echo "  → 怎么办：看日志目录 $gpu_quiesce_log_directory 可不可写，或设 GPU_QUIESCE_LOG_DIRECTORY 指到开机不清的可写目录（退 78）" >&2
  exit 78
fi
gpu_quiesce_registered=0; gpu_quiesce_command_started=0; gpu_quiesce_command_exit=0; gpu_quiesce_interrupted=""
trap 'gpu_quiesce_interrupted=130' INT
trap 'gpu_quiesce_interrupted=143' TERM
trap 'gpu_quiesce_interrupted=129' HUP
trap 'gpu_quiesce_leave; gpu_quiesce_finish_log' EXIT

# 进来：锁里登记、没有别的活着的登记才清场，回读
if ! gpu_quiesce_enter_lock; then
  gpu_quiesce_line "step=command result=not-started"
  gpu_quiesce_line "step=exit code=78"
  echo "  ✗ ${GPU_QUIESCE_LOCK_SECONDS:-900} 秒没拿到清场锁 $gpu_quiesce_state_directory/lock：命令没起" >&2
  echo "  → 怎么办：别的经 gpu-quiesce-run.sh 的那一趟卡在清场命令上；等它结束再起（退 78）" >&2
  exit 78
fi
gpu_quiesce_others="$(gpu_quiesce_other_live_holders)"
: >"$gpu_quiesce_state_directory/holders/$gpu_quiesce_holder_name"
gpu_quiesce_registered=1
# 回读看的卡与跳过的进程号（文件头「回读看哪几张卡、跳过哪些进程」）：清场那一支与只回读那一支都算，在锁里、清场命令起之前
gpu_quiesce_choose_cards
[[ -n "$gpu_quiesce_local_cards" ]] && gpu_quiesce_check_environment+=("GPU_QUIESCE_CHECK_LOCAL_CARDS=$gpu_quiesce_local_cards")
gpu_quiesce_exempt_pids=""
gpu_quiesce_break_is no-holder-descendants || gpu_quiesce_exempt_pids="$(gpu_quiesce_holder_descendants)"
[[ -n "$gpu_quiesce_exempt_pids" ]] && gpu_quiesce_check_environment+=("GPU_QUIESCE_CHECK_EXEMPT_PIDS=$gpu_quiesce_exempt_pids")
gpu_quiesce_exempt_count=0
[[ -n "$gpu_quiesce_exempt_pids" ]] && gpu_quiesce_exempt_count="$(wc -w <<<"$gpu_quiesce_exempt_pids")"
echo "  · 回读看的卡：$gpu_quiesce_cards_said；跳过别的活着的登记下的 $gpu_quiesce_exempt_count 个进程" >&2
gpu_quiesce_stopped_here=0
gpu_quiesce_stop_exit=0
if (( gpu_quiesce_others == 0 )); then gpu_quiesce_run_configured QUIESCE_STOP_COMMAND; gpu_quiesce_stop_exit=$?; gpu_quiesce_stopped_here=1; fi
gpu_quiesce_read_back=0
gpu_quiesce_run_configured QUIESCE_STOPPED_CHECK_COMMAND && gpu_quiesce_read_back=1
gpu_quiesce_leave_lock
gpu_quiesce_quiesce_failed=0
gpu_quiesce_stop_field=""
if (( gpu_quiesce_stopped_here )) && { (( ! gpu_quiesce_read_back )) || (( gpu_quiesce_stop_exit != 0 )); }; then
  gpu_quiesce_keep_log
  gpu_quiesce_break_is drop-stop-exit || gpu_quiesce_stop_field=" stop-exit=$gpu_quiesce_stop_exit"
  gpu_quiesce_stop_field+="$gpu_quiesce_log_field"
fi
if (( gpu_quiesce_read_back )); then
  if (( gpu_quiesce_stopped_here )); then
    gpu_quiesce_line "step=quiesce result=stopped-and-read-back holders=0$gpu_quiesce_stop_field"
    if (( gpu_quiesce_stop_exit != 0 )); then
      echo "  · 照多机配置清了场，回读过了，照跑；清场命令自己退了 $gpu_quiesce_stop_exit，它报的原因在 log= 那份本机日志里" >&2
    else
      echo "  · 照多机配置清了场，回读过了" >&2
    fi
  else
    gpu_quiesce_line "step=quiesce result=joined-and-read-back holders=$gpu_quiesce_others"
    echo "  · 已有 $gpu_quiesce_others 趟清过场在用卡，回读过了，跟着用" >&2
  fi
else
  if [[ -z "$gpu_quiesce_stop_field" ]]; then gpu_quiesce_keep_log; gpu_quiesce_stop_field="$gpu_quiesce_log_field"; fi
  gpu_quiesce_line "step=quiesce result=read-back-failed holders=$gpu_quiesce_others$gpu_quiesce_stop_field"
  echo "  ✗ 照多机配置清场之后回读没过（QUIESCE_STOPPED_CHECK_COMMAND 退非 0）；清场与回读命令自己报的原因在 log= 那份本机日志里（不打到这里：里面有私有服务名）" >&2
  echo "  → 怎么办：卡上还有不归清场白名单的计算进程（别的会话的 GPU 活）时回读本来就不过：等那个进程跑完再起，不杀它；不是这样的，手动跑配置里的清场命令、回读到它退 0 为止（退 78）" >&2
  gpu_quiesce_break_is run-despite-failed-quiesce || gpu_quiesce_quiesce_failed=1
fi

# 跑：前台起，收到 INT/TERM/HUP 只记下、等它自己结束
if (( ! gpu_quiesce_quiesce_failed )) && [[ -z "$gpu_quiesce_interrupted" ]]; then
  gpu_quiesce_command_started=1
  GPU_QUIESCE_HOLDER="$gpu_quiesce_state_directory/holders/$gpu_quiesce_holder_name" "$@"; gpu_quiesce_command_exit=$?
  gpu_quiesce_line "step=command result=exit-$gpu_quiesce_command_exit"
else
  gpu_quiesce_line "step=command result=not-started"
fi

# 出去：删自己的登记，不复原
gpu_quiesce_leave
if (( gpu_quiesce_quiesce_failed )); then gpu_quiesce_final=78
elif [[ -n "$gpu_quiesce_interrupted" ]] && { (( ! gpu_quiesce_command_started )) || (( gpu_quiesce_command_exit == 0 )); }; then gpu_quiesce_final="$gpu_quiesce_interrupted"
else gpu_quiesce_final="$gpu_quiesce_command_exit"; fi
gpu_quiesce_line "step=exit code=$gpu_quiesce_final"
exit "$gpu_quiesce_final"
