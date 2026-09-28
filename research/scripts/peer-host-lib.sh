#!/usr/bin/env bash
# 双机分片共用的第二台那几段：读本地配置、在第二台上跑命令、拷文件、比工具链、找规范副本、清场与复原、
# 脱离 ssh 会话起一份活并轮询、按进程号连同后代停一棵进程树、把输出里配置的主机与目录换成「第二台」。
# 被 source 的函数库，只定义函数与几个全局变量、不改 shell 选项，不单独调；research/scripts/mutation-shard-run.sh（门禁 checker-tier-crates-mutation-replay 的双机分片驱动）source 它。
# gate-overlap:copy-kept layer0-shard-run.sh 改它会让登记了 shard=across-machines 的三条崩溃枚举用例的全绿标记作废（admission.py 的 SHARD_DRIVER_FILES 按内容进输入指纹），KV 接入那一批再把它迁到这一份上
#
# 配置与层 0 分片同一份：research/scripts/layer0-shard-configuration-check.sh --emit-assignments（只调）。配置里的主机、目录与清场命令是私有的：
# 这里的函数打出来的话一律写「第二台」，第二台上的命令与拷文件的错误输出经 peer_host_redact 换掉再打。
#
# 全局变量（peer_host_load_configuration 设）：PEER_SSH_HOST PEER_REPOSITORY_DIRECTORY PEER_CARGO_BIN_DIRECTORY PEER_MEMORY_CAP
#   QUIESCE_STOP_COMMAND QUIESCE_STOPPED_CHECK_COMMAND QUIESCE_START_COMMAND QUIESCE_STARTED_CHECK_COMMAND、peer_host_is_this_machine；
#   peer_host_quiesced（peer_host_quiesce 清过场置 1，peer_host_restore 复原后置 0）；peer_host_toolchain_text（peer_host_compare_toolchains 设）；
#   peer_host_detached_outcome（peer_host_wait_detached 设：exit <码>、vanished、unreachable、never-started）。
# 只供测试的开关 SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1（照层 0 分片）：「第二台」是本机上的另一个目录，命令在本机跑，
#   环境照 ssh 那样只给最少几样（HOME、USER、LOGNAME、LANG、XDG_RUNTIME_DIR、DBUS_SESSION_BUS_ADDRESS、CARGO_HOME、RUSTUP_HOME 与 PATH），本机别的环境变量不漏过去。
# 弄坏开关 PEER_HOST_LIB_BREAK=no-redact（只给证红用）：peer_host_redact 原样转出、不换私有的字（mutation-shard-run.sh 的自证里「输出里没有配置的主机」那一格判错）。

peer_host_is_this_machine=0
peer_host_quiesced=0
peer_host_toolchain_text=""
peer_host_detached_outcome=""

peer_host_load_configuration() { # <配置判法脚本> <树根>：判得过就设配置的键，退 0；判法退 3（双机：关，配置没写 ENABLE_ACROSS_MACHINES=1）
  # 什么都不设、不打，退 3，由调用方报「双机：关」；判不过打一行原因（不含配置内容）与出路，退 1
  local assignments configuration_exit=0
  assignments="$(bash "$1" --emit-assignments "$2" 2>/dev/null)" || configuration_exit=$?
  if (( configuration_exit == 3 )); then return 3; fi
  if (( configuration_exit != 0 )); then
    echo "  ✗ 双机分片的本地配置判不过（原因不在这里打：配置里的主机与目录是私有的）"
    echo "     → 怎么办：单跑 bash research/scripts/layer0-shard-configuration-check.sh 看原因；配置照仓根 multi-host.env.example 写"
    return 1
  fi
  eval "$assignments"
  peer_host_is_this_machine="${SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE:-0}"
}

peer_host_redact() { # 标准输入原样转到标准输出，配置里第二台的目录、cargo 目录与主机名换成「第二台的目录」「第二台的 cargo 目录」「第二台」
  if [[ ",${PEER_HOST_LIB_BREAK:-}," == *,no-redact,* ]]; then cat; return; fi
  PEER_HOST_REDACT_HOST="${PEER_SSH_HOST:-}" PEER_HOST_REDACT_DIRECTORY="${PEER_REPOSITORY_DIRECTORY:-}" PEER_HOST_REDACT_CARGO="${PEER_CARGO_BIN_DIRECTORY:-}" \
    python3 -c '
import os, re, sys
replacements = [(os.environ["PEER_HOST_REDACT_DIRECTORY"], "第二台的目录"), (os.environ["PEER_HOST_REDACT_CARGO"], "第二台的 cargo 目录")]
replacements = sorted((pair for pair in replacements if pair[0]), key=lambda pair: -len(pair[0]))
host = os.environ["PEER_HOST_REDACT_HOST"]
host_pattern = re.compile(r"(?<![\w.-])" + re.escape(host) + r"(?![\w-])") if host else None
for line in sys.stdin:
    for secret, shown in replacements:
        line = line.replace(secret, shown)
    if host_pattern:
        line = host_pattern.sub("第二台", line)
    sys.stdout.write(line)
    sys.stdout.flush()'
}

peer_host_run() { # <目录> <命令文本>：在第二台的这个目录里 bash -c 跑（PATH 前面加 PEER_CARGO_BIN_DIRECTORY），标准错误换掉私有的字再转出；退出码是那条命令的
  if [[ "$peer_host_is_this_machine" == 1 ]]; then
    # 和 ssh 一样等标准输出关上才返回（经 cat 转出）：留在后台、还握着输出的进程会让它一直等
    (cd "$1" && env -i HOME="$HOME" USER="${USER:-}" LOGNAME="${LOGNAME:-}" LANG="${LANG:-C.UTF-8}" XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-}" \
        DBUS_SESSION_BUS_ADDRESS="${DBUS_SESSION_BUS_ADDRESS:-}" ${CARGO_HOME:+CARGO_HOME="$CARGO_HOME"} ${RUSTUP_HOME:+RUSTUP_HOME="$RUSTUP_HOME"} \
        PATH="$PEER_CARGO_BIN_DIRECTORY:$PATH" bash -c "$2") 2> >(peer_host_redact >&2) | cat
    return "${PIPESTATUS[0]}"
  fi
  ssh -o BatchMode=yes "$PEER_SSH_HOST" "cd $(printf '%q' "$1") && PATH=$(printf '%q' "$PEER_CARGO_BIN_DIRECTORY"):\$PATH bash -c $(printf '%q' "$2")" \
    </dev/null 2> >(peer_host_redact >&2)
}

peer_host_copy() { # <rsync 的参数…>：rsync -a（不带 --delete），第二台那头的路径用 peer_host_path 写；错误输出换掉私有的字
  if [[ "$peer_host_is_this_machine" == 1 ]]; then
    rsync -a "$@" 2> >(peer_host_redact >&2)
  else
    rsync -a -e "ssh -o BatchMode=yes" "$@" 2> >(peer_host_redact >&2)
  fi
}

peer_host_path() { # <第二台上的路径>：rsync 的参数写法
  if [[ "$peer_host_is_this_machine" == 1 ]]; then printf '%s' "$1"; else printf '%s:%s' "$PEER_SSH_HOST" "$1"; fi
}

peer_host_compare_toolchains() { # 两台 rustc -Vv 前三行与 host 行、cargo -V 相同退 0（peer_host_toolchain_text 设成本机 rustc 那一行）；不同或报不出打原因与出路，退 1
  local description_command='rustc -Vv | head -3 && rustc -Vv | grep "^host: " && cargo -V' local_toolchain peer_toolchain
  if ! local_toolchain="$(bash -c "$description_command" 2>/dev/null)"; then
    echo "  ✗ 本机 rustc / cargo 报不出版本"
    echo "     → 怎么办：看 PATH 里有没有 rustc 与 cargo"
    return 1
  fi
  if ! peer_toolchain="$(peer_host_run / "$description_command" 2>/dev/null)"; then
    echo "  ✗ 第二台 rustc / cargo 报不出版本"
    echo "     → 怎么办：看配置的 PEER_CARGO_BIN_DIRECTORY 下有没有 rustc 与 cargo、ssh 连不连得上"
    return 1
  fi
  if [[ "$local_toolchain" != "$peer_toolchain" ]]; then
    echo "  ✗ 两台的工具链不同：本机「${local_toolchain//$'\n'/；}」，第二台「${peer_toolchain//$'\n'/；}」"
    echo "     → 怎么办：两台装同一个 rustup 工具链（rustc -Vv 的 commit 相同）再跑"
    return 1
  fi
  peer_host_toolchain_text="${local_toolchain%%$'\n'*}"
}

peer_host_sop_source() { # <树根> <驱动所在的仓根>：打要拷到第二台的规范副本目录——树里有就用树里的，没有用驱动所在仓的，再没有用那份仓的 git common-dir 所在的主仓的；都没有退 1
  local candidate common_directory
  local candidates=("$1/.claude/singlefs-ai-sop" "$2/.claude/singlefs-ai-sop")
  if common_directory="$(git -C "$2" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" && [[ -n "$common_directory" ]]; then
    candidates+=("${common_directory%/*}/.claude/singlefs-ai-sop")
  fi
  for candidate in "${candidates[@]}"; do
    if [[ -f "$candidate/scripts/preflight.py" ]]; then printf '%s' "$candidate"; return 0; fi
  done
  return 1
}

peer_host_quiesce() { # 照配置清场（配置里写了才做），跑完回读；清场与回读的命令与输出都不打（里面有服务名）。回读没过退 1
  [[ -n "${QUIESCE_STOP_COMMAND:-}" ]] || return 0
  peer_host_quiesced=1
  bash -c "$QUIESCE_STOP_COMMAND" >/dev/null 2>&1
  if bash -c "$QUIESCE_STOPPED_CHECK_COMMAND" >/dev/null 2>&1; then
    echo "  · 照配置清了场，回读过了"
    return 0
  fi
  echo "  ✗ 照配置清场之后回读没过（配置的 QUIESCE_STOPPED_CHECK_COMMAND 退非 0）"
  echo "     → 怎么办：手动跑配置里的清场命令，回读到它退 0 为止；复原命令在退出时照跑"
  return 1
}

peer_host_restore() { # 清过场就照配置复原并回读（命令与输出不打）；调用方挂在 EXIT 上
  (( peer_host_quiesced )) || return 0
  peer_host_quiesced=0
  bash -c "$QUIESCE_START_COMMAND" >/dev/null 2>&1
  if bash -c "$QUIESCE_STARTED_CHECK_COMMAND" >/dev/null 2>&1; then
    echo "  · 照配置复原了，回读过了"
  else
    echo "  ✗ 照配置复原之后回读没过（配置的 QUIESCE_STARTED_CHECK_COMMAND 退非 0）"
    echo "     → 怎么办：手动跑配置里的复原命令，回读到它退 0 为止"
  fi
}

# 下面四个函数也经 declare -f 带到第二台上跑：只用 bash、ps 与 /proc，不引这四个之外的函数
peer_host_signal_one() { # <信号名> <进程号>：给一个写死的进程号发一个信号，一次一个（.claude/singlefs-ai-sop/rules/command-safety.md「pkill -f / killall 一律禁用」；
  # 停一棵树时逐个进程号调它，kill 不写进循环里）。进程号不是正整数、或是 0、1，不发、退 2；发不出去（进程已经没了）退 kill 的码
  [[ "$2" =~ ^[0-9]+$ ]] && (( $2 > 1 )) || return 2
  kill -"$1" "$2" 2>/dev/null
}
peer_host_process_is_alive() { # <进程号>：还在、且不是僵尸（僵尸算停了）
  [[ -d "/proc/$1" ]] && [[ "$(awk '{print $3}' "/proc/$1/stat" 2>/dev/null)" != Z ]]
}
peer_host_process_tree() { # <进程号>：打这个进程与它的全部后代的进程号，一行一个，父在子前（按 ps --ppid 一层层往下找）
  local queue=("$1") current child
  while (( ${#queue[@]} > 0 )); do
    current="${queue[0]}"
    queue=("${queue[@]:1}")
    [[ -d "/proc/$current" ]] || continue
    echo "$current"
    for child in $(ps -o pid= --ppid "$current" 2>/dev/null); do queue+=("$child"); done
  done
}
peer_host_stop_process_tree() { # <进程号>：这个进程连同全部后代，逐个按进程号冻住（STOP）、从最底层起 TERM、放开（CONT），至多等 30 秒，
  # 还在的逐个 KILL；回读一个不剩退 0，还有剩的打出来退 1。冻住之后再列一遍：列与冻之间新起的子进程也收进来
  local pids=() pid index waited survivors
  mapfile -t pids < <(peer_host_process_tree "$1")
  if (( ${#pids[@]} == 0 )); then echo "进程 $1 已经不在"; return 0; fi
  for pid in "${pids[@]}"; do peer_host_signal_one STOP "$pid"; done
  mapfile -t pids < <(peer_host_process_tree "$1")
  for pid in "${pids[@]}"; do peer_host_signal_one STOP "$pid"; done
  for (( index = ${#pids[@]} - 1; index >= 0; index-- )); do peer_host_signal_one TERM "${pids[index]}"; done
  for pid in "${pids[@]}"; do peer_host_signal_one CONT "$pid"; done
  for (( waited = 0; waited < 300; waited++ )); do
    survivors=()
    for pid in "${pids[@]}"; do peer_host_process_is_alive "$pid" && survivors+=("$pid"); done
    (( ${#survivors[@]} == 0 )) && break
    sleep 0.1
  done
  for pid in ${survivors[@]+"${survivors[@]}"}; do peer_host_signal_one KILL "$pid"; done
  sleep 0.5
  survivors=()
  for pid in "${pids[@]}"; do peer_host_process_is_alive "$pid" && survivors+=("$pid"); done
  if (( ${#survivors[@]} > 0 )); then echo "进程 $1 那一棵里 ${survivors[*]} 发了 TERM、KILL 还在"; return 1; fi
  echo "进程 $1 连同 $(( ${#pids[@]} - 1 )) 个后代都停了"
}

peer_host_start_detached() { # <目录> <进程号文件> <退出码文件> <输出文件> <命令文本>：在第二台的这个目录里脱离 ssh 会话起（setsid nohup），
  # 起的那个 bash 先把自己的进程号写进进程号文件，命令的输出写进输出文件，跑完把退出码写进退出码文件；ssh 马上返回
  local directory="$1" pid_file="$2" exit_file="$3" output_file="$4" command_text="$5"
  local detached_text
  detached_text="echo \$\$ > $(printf '%q' "$pid_file"); $command_text > $(printf '%q' "$output_file") 2>&1; echo \$? > $(printf '%q' "$exit_file")"
  # 只让 setsid nohup 那一条进后台（它的输入输出都重定向了）：整串放进后台的话，后台那个子进程握着 ssh 的输出通道，ssh 要等它跑完才返回
  peer_host_run "$directory" "rm -f -- $(printf '%q' "$pid_file") $(printf '%q' "$exit_file") $(printf '%q' "$output_file") || exit 1; setsid nohup bash -c $(printf '%q' "$detached_text") < /dev/null > /dev/null 2>&1 &"
}

peer_host_wait_detached() { # <进程号文件> <退出码文件> <轮询秒数>：等第二台那一份跑完，结局写进 peer_host_detached_outcome：
  # exit <码>；vanished（进程没了、没留退出码，连续 3 次）；unreachable（连续 60 次 ssh 不通）；never-started（连续 60 次既没有进程号文件也没有退出码）
  local pid_file="$1" exit_file="$2" poll_seconds="$3" poll unreachable_rounds=0 vanished_rounds=0 starting_rounds=0 previous=""
  peer_host_detached_outcome=""
  while [[ -z "$peer_host_detached_outcome" ]]; do
    if poll="$(peer_host_run / "if [[ -s $(printf '%q' "$exit_file") ]]; then echo \"exit \$(cat $(printf '%q' "$exit_file"))\"; elif [[ -s $(printf '%q' "$pid_file") && -d /proc/\$(cat $(printf '%q' "$pid_file")) ]]; then echo running; elif [[ -s $(printf '%q' "$pid_file") ]]; then echo vanished; else echo starting; fi" 2>/dev/null)"; then
      unreachable_rounds=0
    else
      poll=unreachable; unreachable_rounds=$((unreachable_rounds + 1))
    fi
    case "$poll" in
      "exit "*) peer_host_detached_outcome="$poll" ;;
      vanished) vanished_rounds=$((vanished_rounds + 1)); (( vanished_rounds < 3 )) || peer_host_detached_outcome=vanished ;;
      unreachable) (( unreachable_rounds < 60 )) || peer_host_detached_outcome=unreachable ;;
      starting) starting_rounds=$((starting_rounds + 1)); (( starting_rounds < 60 )) || peer_host_detached_outcome=never-started ;;
      *) vanished_rounds=0 ;;
    esac
    if [[ "$poll" != "$previous" && -z "$peer_host_detached_outcome" ]]; then
      echo "  … 第二台那一份：$poll"
      previous="$poll"
    fi
    [[ -n "$peer_host_detached_outcome" ]] || sleep "$poll_seconds"
  done
}

peer_host_stop_detached() { # <目录> <进程号文件>：按第二台上记下的进程号停那一份（连同后代）；进程号文件要等那一份起来才有，至多等 10 秒；
  # 核它的工作目录是这个目录（进程号没被别的进程重用）。打一句结局；没停下退 1
  peer_host_run "$1" "$(declare -f peer_host_signal_one peer_host_process_is_alive peer_host_process_tree peer_host_stop_process_tree)
for ((pid_rounds = 0; pid_rounds < 100; pid_rounds++)); do [[ -s $(printf '%q' "$2") ]] && break; sleep 0.1; done
detached_pid=\"\$(cat $(printf '%q' "$2") 2>/dev/null)\"
if [[ -z \"\$detached_pid\" ]]; then echo '没有进程号文件：那一份没起来'
elif [[ \"\$(readlink /proc/\$detached_pid/cwd 2>/dev/null)\" != \"\$(pwd -P)\" ]]; then echo \"进程号 \$detached_pid 已经不在（或不是这一趟的）\"
else peer_host_stop_process_tree \"\$detached_pid\"; fi"
}
