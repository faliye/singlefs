#!/usr/bin/env bash
# admission: always 每一次调都判此刻的仓与这一次的参数，上一次的结论不替这一次作保
# run-condition: none 只读会话记录与 /proc，除了 bash 与 python3 之外没有环境要求
# 看门狗的调用入口：主 agent 每次派发（含续做）之后照 .claude/main-agent.md「派出去之后」起它，调用时只填要盯的 agent 号，
# 会话目录由 agent-watch.py 按 agent 号自己找，阈值从 scripts/agents/watch.conf 读——不在命令行上手敲。
#
# 一律放进 Bash 工具的 run_in_background；这里不再自己放后台（放了之后跑完的那个叫不醒主 agent）。
#   bash scripts/agents/watch.sh                                       一个会话一个看门狗：从 CLAUDE_CODE_SESSION_ID 找会话目录，每次检查自己找还没结束的子 agent
#                                                                        （新派的自动接上，不用停旧的起新的）；已有一个在盯就退 5；--ack 记进这个会话的状态目录，下一次起读回
#   bash scripts/agents/watch.sh <agent 号>[,<agent 号>…] [<agent 号> …]   只盯这几个子 agent（--ack 只在这一次有效）
#   bash scripts/agents/watch.sh --processes                           只盯这个 Claude 实例底下已经在跑的长进程（主 agent 自己起的长命令），等它们都退出；进程卡不卡归 task-server 判，这里不报；
#                                                                        找得到当前会话（CLAUDE_CODE_SESSION_ID）就同时查交回之后后台还在跑
#   bash scripts/agents/watch.sh --report                              当前会话的全部子 agent 现在是什么样，报一次就退
#   bash scripts/agents/watch.sh … --ack <agent 号>:<告警名>            看过、判定只是慢的那一条告警，这一次看门狗里不再为它叫醒（可给多次；「交回之后后台还在跑」写 --ack 进程:<pid>）；
#                                                                        也可逗号连写：--ack 进程:<pid>,进程:<pid>,<agent 号>:<告警名>，拆成一条一个 --ack 交给 agent-watch.py。
#                                                                        逗号后面接的是「进程:」「类:」或「<agent 号>:」才算下一条；接的不是的，那个逗号算进前一条的告警名；
#                                                                        头一条就不是这几种形态的整个拒（拒绝说明给出路）
#                                                                        --ack 的写法一共三种（判法在 agent-watch.py 文件头「--ack 的写法」）：
#                                                                          进程:<pid>          确认那一个进程的告警（「交回之后后台还在跑」）
#                                                                          <agent 号>:<告警名>  确认那个子 agent 的那一类告警
#                                                                          类:<告警名>          确认这一类告警的全部，不分子 agent、不分进程（例：--ack 类:交回之后后台还在跑）
#   bash scripts/agents/watch.sh --dry-run …                           只打印要跑的命令，不跑
#   bash scripts/agents/watch.sh --selftest
# 弄坏开关（只给 --selftest 证红用；自证不逐个打开，打开后跑 --selftest 核红在点名那一格）：平常跑设了就在一切之前退 2、点名变量只在自证里认，自证起的子进程由 WATCH_SELFTEST_MARK 认。
#   WATCH_BREAK=rootfreekeydropped  root-free-gib-alert 这个键读了不转成选项（键表里新加的键自证不核的样子）
#       自检⑩（「✗ 自检⑩：键表里这些键没转成同名的长选项」）
#   WATCH_BREAK=ackcommajoined  --ack 的逗号连写不拆，整串原样当一条交过去（改前的样子：连写的几个进程照报）
#       自检⑨（「✗ 自检⑨：--ack 逗号连写没拆成一条一个 --ack」）、自检⑪（「✗ 自检⑪：--ack 类:<告警名> 没原样转过去」）
#   WATCH_BREAK=ackscopeform  --ack 只认「进程:」与「<agent 号>:」两种头（改前的样子：类: 被当成形态不对拒绝）
#       自检⑪（「✗ 自检⑪：--ack 类:<告警名> 没原样转过去」）、
#       自检⑬（「✗ 自检⑬：--ack 类:根分区余量过低 经 watch.sh 起的看门狗子进程应当确认那一类告警」、「✗ 自检⑬：--ack 类:交回之后后台还在跑 不该确认别的类的告警」）、
#       自检⑭（「✗ 自检⑭：watch.conf 里推荐的 --ack 类:根分区余量过低 过不了 watch.sh」、「✗ 自检⑭：agent-watch.py 文件头里推荐的 --ack 类:根分区余量过低 过不了 watch.sh」、
#       「✗ 自检⑭：看门狗打出的告警里推荐的 --ack 类:根分区余量过低 过不了 watch.sh」）
#   WATCH_BREAK=ackclassfiltered  自检⑬里真起的看门狗子进程带 AGENT_WATCH_BREAK=ackclassfiltered：agent-watch.py 的 acknowledgements() 滤掉 类: 开头的键（--ack 类:<告警名> 经真起的看门狗不起作用，被确认的那一类告警照样叫醒）
#       自检⑬（「✗ 自检⑬：--ack 类:根分区余量过低 经 watch.sh 起的看门狗子进程应当确认那一类告警」）
#   WATCH_BREAK=ackrecommendedform  自检⑭把 watch.conf、agent-watch.py 文件头与看门狗告警里推荐的 --ack 写法里的「类:」换回「本机:」再喂 --dry-run（改前的推荐写法，watch.sh 不认）
#       自检⑭（「✗ 自检⑭：watch.conf 里推荐的 --ack 本机:根分区余量过低 过不了 watch.sh」、「✗ 自检⑭：agent-watch.py 文件头里推荐的 --ack 本机:根分区余量过低 过不了 watch.sh」、
#       「✗ 自检⑭：看门狗打出的告警里推荐的 --ack 本机:根分区余量过低 过不了 watch.sh」）
#   WATCH_BREAK=session-directory-from-home  会话目录退回 $HOME 接本脚本所在的仓路径（改前的样子：设了 TASK_SERVER_SOURCE_REPOSITORY 也不用，HOME 指到别处就找错）
#       自检⑫（「✗ 自检⑫：设了 TASK_SERVER_SOURCE_REPOSITORY、HOME 指到别处时，--processes 没带来源仓的会话目录」、
#       「✗ 自检⑫：设了 TASK_SERVER_SOURCE_REPOSITORY、HOME 指到别处时，不给 agent 号的 --discover 没带来源仓的会话目录」、
#       「✗ 自检⑫：设了 TASK_SERVER_SOURCE_REPOSITORY、HOME 指到别处时，--report 没带来源仓的会话目录」、「✗ 自检⑫：来源仓根不是绝对路径时应当退 2 并写出路」）、
#       自检⑬（「✗ 自检⑬：没给 --ack 时真起的看门狗子进程应当报「根分区余量过低」」、「✗ 自检⑬：--ack 类:根分区余量过低 经 watch.sh 起的看门狗子进程应当确认那一类告警」、
#       「✗ 自检⑬：--ack 类:交回之后后台还在跑 不该确认别的类的告警」；真起的看门狗找不到会话目录就退 2）、自检⑭（「✗ 自检⑭：看门狗打出的告警里找不到推荐的 --ack 写法」）
# 会话目录（<家目录>/.claude/projects/<仓路径改写>/<会话号>）：统一任务服务给任务的来源仓根 TASK_SERVER_SOURCE_REPOSITORY 定仓路径与家目录，没有这个变量（服务外手跑）取本脚本所在的仓与本机家目录；
#   取法只在同目录 lib_session_directory.py 里（repository_session_directory 经 python 调它），不用 $HOME 拼。
set -uo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
# 弄坏开关只在自证里认：命令行不是单独一个 --selftest、也不是自证起的子进程（WATCH_SELFTEST_MARK 对得上）而变量非空，参数解析之前就退 2（不管值是什么）。
if [[ -n "${WATCH_BREAK:-}" && ! ( "$#" -eq 1 && "${1:-}" == --selftest ) && "${WATCH_SELFTEST_MARK:-}" != watch-selftest ]]; then
  echo "  ✗ WATCH_BREAK=${WATCH_BREAK} 是弄坏开关，只在自证（bash scripts/agents/watch.sh --selftest）里认，平常跑设了就拒，这一趟什么都没做" >&2
  echo "     → 怎么办：去掉 WATCH_BREAK 再跑；要证红就跑 WATCH_BREAK=<项> bash scripts/agents/watch.sh --selftest" >&2
  exit 2
fi
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
conf="${WATCH_CONF:-$here/watch.conf}"
watchdog="$here/agent-watch.py"

die() { echo "  ✗ $1" >&2; echo "     → 怎么办：$2" >&2; exit 2; }

# 本仓的会话目录（<家目录>/.claude/projects/<仓路径把 / 换成 ->，各会话的 <会话号> 目录在它底下）：统一任务服务给任务的来源仓根 TASK_SERVER_SOURCE_REPOSITORY 定仓路径与家目录
# （本机与第二台都给本机那份仓的绝对路径，会话记录在第二台挂在同一绝对路径，第二台的 $HOME 是另一个，不用它）；没有这个变量（服务外手跑）取本脚本所在的仓与本机家目录。
# 取法只在 lib_session_directory.py 里；取不到它已把 ✗ 与出路写到标准错误，这里退 2。弄坏开关 WATCH_BREAK=session-directory-from-home 退回改前的 $HOME 接仓路径（自检⑫红）。
repository_session_directory() {
  if [[ "${WATCH_BREAK:-}" == session-directory-from-home ]]; then
    printf '%s\n' "$HOME/.claude/projects/${root//\//-}"
    return 0
  fi
  python3 "$here/lib_session_directory.py" session-directory --own-repository "$root"
}

# 配置里认的键（agent-watch.py 的长选项名去掉 --）：新加的键写在这一处，自证⑩逐个核它们转得成同名选项、且都是 agent-watch.py 认的选项
KNOWN_CONFIG_KEYS=" interval-seconds max-minutes tool-minutes wait-loop-minutes idle-minutes repeat-count context-tokens not-started-minutes active-minutes detection-poll-seconds leftover-background-grace-minutes ask-every-minutes progress-stale-minutes kept-copy-hours scratch-idle-hours root-free-gib-alert "
# 配置转成选项；不认识的 key、不是整数的值都拒绝，免得一个拼错的 key 被安静地丢掉
config_options() {
  local line key value known="$KNOWN_CONFIG_KEYS"
  [[ -f "$conf" ]] || return 0
  while IFS= read -r line || [[ -n "$line" ]]; do
    line="${line%%#*}"
    line="${line//[[:space:]]/}"
    [[ -z "$line" ]] && continue
    [[ "$line" == *=* ]] || die "$conf 里有一行不是 key=value：$line" "照文件头的写法改成 key=value，或者删掉这一行"
    key="${line%%=*}"
    value="${line#*=}"
    [[ "$known" == *" $key "* ]] || die "$conf 里的 key「$key」agent-watch.py 不认识" "key 用 agent-watch.py 的长选项名去掉 --，可写的那几个列在 $conf 文件头"
    [[ "$value" =~ ^[0-9]+$ ]] || die "$conf 里「$key」的值「$value」不是整数" "改成整数"
    [[ "${WATCH_BREAK:-}" == rootfreekeydropped && "$key" == root-free-gib-alert ]] && continue   # 弄坏开关：这一个键读了不转（自检⑩红）
    printf -- '--%s\n%s\n' "$key" "$value"
  done < "$conf"
}

run_or_print() {
  if ((dry_run)); then
    printf '%s ' "$@"
    echo
    return 0
  fi
  exec "$@"
}

selftest() {
  local scratch fail=0 out checked=0
  export WATCH_SELFTEST_MARK=watch-selftest   # 自证起的子进程（bash "$0" --dry-run …，环境原样继承）靠它认：弄坏开关只在自证里认
  scratch="$(mktemp -d)"
  # 中途报错、被打断也删（成功、失败都删；local 变量到退出时已不在，路径在这里就展开）
  trap "rm -rf -- '$scratch'" EXIT
  # ① 配置里的 key 原样转成选项、agent 号逗号与空格都认
  printf 'tool-minutes = 12\n# 注释\n\nmax-minutes=30\nleftover-background-grace-minutes=3\nkept-copy-hours=5\nscratch-idle-hours=4\nroot-free-gib-alert=100\n' > "$scratch/ok.conf"
  out="$(WATCH_CONF="$scratch/ok.conf" bash "$0" --dry-run a0000000000000001,a0000000000000002 a0000000000000003 2>&1)"
  [[ "$out" == *"--agents a0000000000000001,a0000000000000002,a0000000000000003"* && "$out" == *"--tool-minutes 12"* && "$out" == *"--max-minutes 30"* \
     && "$out" == *"--leftover-background-grace-minutes 3"* && "$out" == *"--kept-copy-hours 5"* && "$out" == *"--scratch-idle-hours 4"* \
     && "$out" == *"--root-free-gib-alert 100"* ]] \
    || { echo "  ✗ 自检①：配置或 agent 号没转对：$out"; fail=1; }   # gate-lint:detail
  checked=$((checked + 1))
  # ② 拼错的 key 必须拒绝，不许安静丢掉
  printf 'tool-minute=12\n' > "$scratch/bad.conf"
  if WATCH_CONF="$scratch/bad.conf" bash "$0" --dry-run a0000000000000001 >/dev/null 2>&1; then
    echo "  ✗ 自检②：拼错的 key 没被拒绝"; fail=1   # gate-lint:detail
  fi
  checked=$((checked + 1))
  # ③ 不像 agent 号的参数必须拒绝
  if WATCH_CONF="$scratch/ok.conf" bash "$0" --dry-run nohup >/dev/null 2>&1; then
    echo "  ✗ 自检③：不像 agent 号的参数没被拒绝"; fail=1   # gate-lint:detail
  fi
  checked=$((checked + 1))
  # ④ --processes 走只盯进程那条路；找不到当前会话时不带 --session-dir（agent-watch.py 在报告里写明这一类没查）
  out="$(env -u CLAUDE_CODE_SESSION_ID WATCH_CONF="$scratch/ok.conf" bash "$0" --dry-run --processes 2>&1)"
  [[ "$out" == *"--processes-only"* && "$out" != *"--agents"* && "$out" != *"--session-dir"* ]] \
    || { echo "  ✗ 自检④：--processes 没转成 --processes-only，或找不到会话却带了 --session-dir：$out"; fail=1; }   # gate-lint:detail
  checked=$((checked + 1))
  # ⑤ --processes 找得到当前会话就带 --session-dir，好同时查交回之后后台还在跑
  mkdir -p "$scratch/home/.claude/projects/${root//\//-}/selftest-session"
  out="$(env -u TASK_SERVER_SOURCE_REPOSITORY HOME="$scratch/home" CLAUDE_CODE_SESSION_ID=selftest-session WATCH_CONF="$scratch/ok.conf" bash "$0" --dry-run --processes 2>&1)"   # 没有来源仓根（服务外手跑）：本脚本所在的仓与 HOME
  [[ "$out" == *"--processes-only --session-dir $scratch/home/.claude/projects/${root//\//-}/selftest-session "* ]] \
    || { echo "  ✗ 自检⑤：--processes 找得到当前会话却没带 --session-dir：$out"; fail=1; }   # gate-lint:detail
  checked=$((checked + 1))
  # ⑥ --ack 原样转过去；形态不对的拒绝
  out="$(WATCH_CONF="$scratch/ok.conf" bash "$0" --dry-run --ack a0000000000000001:上下文过大 a0000000000000001 2>&1)"
  [[ "$out" == *"--ack a0000000000000001:上下文过大"* ]] || { echo "  ✗ 自检⑥：--ack 没原样转过去：$out"; fail=1; }   # gate-lint:detail
  if WATCH_CONF="$scratch/ok.conf" bash "$0" --dry-run --ack 上下文过大 a0000000000000001 >/dev/null 2>&1; then
    echo "  ✗ 自检⑥：形态不对的 --ack 没被拒绝"; fail=1   # gate-lint:detail
  fi
  checked=$((checked + 1))
  # ⑦ 旧的调用写法照旧：交回之后后台还在跑的 --ack 进程:<pid> 与书记员汇总告警的 --ack <agent 号>:<告警名> 原样转过去，agent 号照旧跟在后面
  out="$(WATCH_CONF="$scratch/ok.conf" bash "$0" --dry-run --ack 进程:4242 --ack a0000000000000001:书记员攒下的没跟上检出 a0000000000000001 2>&1)"
  [[ "$out" == *"--agents a0000000000000001 "* && "$out" == *"--ack 进程:4242 --ack a0000000000000001:书记员攒下的没跟上检出"* ]] \
    || { echo "  ✗ 自检⑦：进程告警（--ack 进程:<pid>）或书记员汇总告警（--ack <agent 号>:书记员攒下的没跟上检出）的确认没原样转过去：$out"; fail=1; }   # gate-lint:detail
  checked=$((checked + 1))
  # ⑧ 不给 agent 号：找得到当前会话就走 --discover（一个会话一个看门狗）；找不到会话就拒绝
  out="$(env -u TASK_SERVER_SOURCE_REPOSITORY HOME="$scratch/home" CLAUDE_CODE_SESSION_ID=selftest-session WATCH_CONF="$scratch/ok.conf" bash "$0" --dry-run --ack a0000000000000001:该交接 2>&1)"
  [[ "$out" == *"watch --discover --session-dir $scratch/home/.claude/projects/${root//\//-}/selftest-session "* && "$out" == *"--ack a0000000000000001:该交接"* ]] \
    || { echo "  ✗ 自检⑧：不给 agent 号、找得到会话时没走 --discover 或没带上会话目录与 --ack：$out"; fail=1; }   # gate-lint:detail
  if env -u CLAUDE_CODE_SESSION_ID WATCH_CONF="$scratch/ok.conf" bash "$0" --dry-run >/dev/null 2>&1; then
    echo "  ✗ 自检⑧：不给 agent 号、又找不到会话时没被拒绝"; fail=1   # gate-lint:detail
  fi
  checked=$((checked + 1))
  # ⑨ --ack 逗号连写拆成一条一个 --ack（进程与 agent 号的告警混写也拆）；告警名里的逗号（后面接的不是「进程:」或 agent 号）留在那一条里；头一条形态不对的整个拒
  out="$(WATCH_CONF="$scratch/ok.conf" bash "$0" --dry-run --processes --ack 进程:3162530,进程:3170253,a0000000000000001:上下文过大,带逗号 2>&1)"
  [[ "$out" == *"--ack 进程:3162530 --ack 进程:3170253 --ack a0000000000000001:上下文过大,带逗号"* && "$out" != *"--ack 进程:3162530,"* ]] \
    || { echo "  ✗ 自检⑨：--ack 逗号连写没拆成一条一个 --ack（或把告警名里的逗号也拆了）：$out"; fail=1; }   # gate-lint:detail
  if WATCH_CONF="$scratch/ok.conf" bash "$0" --dry-run --processes --ack 上下文过大,进程:3162530 >/dev/null 2>&1; then
    echo "  ✗ 自检⑨：逗号连写头一条形态不对没被拒绝"; fail=1   # gate-lint:detail
  fi
  checked=$((checked + 1))
  # ⑩ 键表里的每一个键都转成同名的长选项、且都是 agent-watch.py 认的选项（新加的键不在上面①的样本里也核到）
  : > "$scratch/all-keys.conf"
  for key in $KNOWN_CONFIG_KEYS; do printf '%s=7\n' "$key" >> "$scratch/all-keys.conf"; done
  out="$(WATCH_CONF="$scratch/all-keys.conf" bash "$0" --dry-run a0000000000000001 2>&1)"
  help_text="$(python3 "$watchdog" --help 2>&1)" || { echo "  ✗ 自检⑩：agent-watch.py --help 没退 0：${help_text: -300}"; fail=1; }   # gate-lint:detail
  missing_forward="" missing_option=""
  for key in $KNOWN_CONFIG_KEYS; do
    [[ "$out" == *"--$key 7"* ]] || missing_forward+=" $key"
    [[ "$help_text" == *"--$key"* ]] || missing_option+=" $key"
  done
  [[ -z "$missing_forward" ]] || { echo "  ✗ 自检⑩：键表里这些键没转成同名的长选项：$missing_forward"; fail=1; }   # gate-lint:detail
  [[ -z "$missing_option" ]] || { echo "  ✗ 自检⑩：键表里这些键不是 agent-watch.py 的选项（--help 里找不到）：$missing_option"; fail=1; }   # gate-lint:detail
  checked=$((checked + 1))
  # ⑪ --ack 的类头照 agent 号、进程一样原样转过去，逗号连写里夹着也拆；类: 的告警名里的逗号规则同⑨
  out="$(WATCH_CONF="$scratch/ok.conf" bash "$0" --dry-run --processes --ack 类:交回之后后台还在跑,进程:7 --ack 类:根分区余量过低 2>&1)"
  [[ "$out" == *"--ack 类:交回之后后台还在跑 --ack 进程:7 --ack 类:根分区余量过低"* ]] \
    || { echo "  ✗ 自检⑪：--ack 类:<告警名> 没原样转过去（或逗号连写没拆）：$out"; fail=1; }   # gate-lint:detail
  checked=$((checked + 1))
  # ⑫ 来源仓根 TASK_SERVER_SOURCE_REPOSITORY 设着（统一任务服务给每次尝试带的，本机与第二台一个值）：HOME 指到别处、别处还有一份本脚本所在的仓的会话目录（找错了会找到一个在但不对的），
  # 会话目录仍取来源仓的，三种起法（--processes、不给 agent 号的 --discover、--report）一样；弄坏开关 WATCH_BREAK=session-directory-from-home 改回 $HOME 接仓路径，这一格红
  source_repository="$scratch/source-home/code/source-repository"
  expected_session="$scratch/source-home/.claude/projects/${source_repository//\//-}/selftest-session"
  mkdir -p "$source_repository" "$expected_session" "$scratch/other-home/.claude/projects/${root//\//-}/selftest-session"
  session_environment=(HOME="$scratch/other-home" TASK_SERVER_SOURCE_REPOSITORY="$source_repository" CLAUDE_CODE_SESSION_ID=selftest-session WATCH_CONF="$scratch/ok.conf")
  out="$(env "${session_environment[@]}" bash "$0" --dry-run --processes 2>&1)"
  [[ "$out" == *"--processes-only --session-dir $expected_session "* ]] \
    || { echo "  ✗ 自检⑫：设了 TASK_SERVER_SOURCE_REPOSITORY、HOME 指到别处时，--processes 没带来源仓的会话目录 $expected_session：$out"; fail=1; }   # gate-lint:detail
  out="$(env "${session_environment[@]}" bash "$0" --dry-run --ack a0000000000000001:该交接 2>&1)"
  [[ "$out" == *"watch --discover --session-dir $expected_session "* ]] \
    || { echo "  ✗ 自检⑫：设了 TASK_SERVER_SOURCE_REPOSITORY、HOME 指到别处时，不给 agent 号的 --discover 没带来源仓的会话目录 $expected_session：$out"; fail=1; }   # gate-lint:detail
  out="$(env "${session_environment[@]}" bash "$0" --dry-run --report 2>&1)"
  [[ "$out" == *" report --session-dir $expected_session "* ]] \
    || { echo "  ✗ 自检⑫：设了 TASK_SERVER_SOURCE_REPOSITORY、HOME 指到别处时，--report 没带来源仓的会话目录 $expected_session：$out"; fail=1; }   # gate-lint:detail
  out="$(env "${session_environment[@]}" TASK_SERVER_SOURCE_REPOSITORY=relative/repository bash "$0" --dry-run --report 2>&1)"
  if [[ "$out" == *"--session-dir"* || "$out" != *"怎么办"* ]]; then
    echo "  ✗ 自检⑫：来源仓根不是绝对路径时应当退 2 并写出路，不带会话目录往下跑：$out"; fail=1   # gate-lint:detail
  fi
  checked=$((checked + 1))
  # ⑬ --ack 类:<告警名> 经真起的看门狗子进程：watch.sh 不给 agent 号（走 --discover）起真的 agent-watch.py，三趟各用自己的状态目录：
  # 不给 --ack 时报「根分区余量过低」退 3；--ack 类:根分区余量过低 时那一条只进报告（max-minutes=0，第一轮整查完就到点，退 4）、不再叫醒；
  # --ack 的是别的类（类:交回之后后台还在跑）时「根分区余量过低」照报、退 3。告警靠环境变量把根分区可用量设成 40 GiB 造出来（阈值 128 GiB 下过线），
  # 状态目录、会话登记、草稿根都指到临时目录，会话目录是 ⑫ 造的空目录，不连真服务。
  # 弄坏开关 WATCH_BREAK=ackclassfiltered 给这几只子进程带 AGENT_WATCH_BREAK=ackclassfiltered（acknowledgements() 滤掉 类: 开头的键），第二趟的确认失效，这一格红
  watchdog_selftest_mark="agent-watch.py --selftest 起的子进程"   # 与 agent-watch.py 的 SELFTEST_MARK_CONTENT 同一串：带 AGENT_WATCH_BREAK 的子进程靠它认是自证起的
  grep -qF "SELFTEST_MARK_CONTENT = \"$watchdog_selftest_mark\"" "$watchdog" \
    || { echo "  ✗ 自检⑬：agent-watch.py 的 SELFTEST_MARK_CONTENT 不是「$watchdog_selftest_mark」，真起的看门狗子进程带弄坏开关时认不出这是自证"; fail=1; }   # gate-lint:detail
  agent_break=""
  [[ "${WATCH_BREAK:-}" == ackclassfiltered ]] && agent_break=ackclassfiltered
  printf 'max-minutes=0\nroot-free-gib-alert=128\n' > "$scratch/ack-class.conf"
  ack_class_run() {   # $1：这一趟的状态目录名；其余原样交给 watch.sh
    local state_name="$1"
    shift
    timeout 120 env TASK_SERVER_SOURCE_REPOSITORY="$source_repository" CLAUDE_CODE_SESSION_ID=selftest-session WATCH_CONF="$scratch/ack-class.conf" \
      AGENT_WATCH_STATE_ROOT="$scratch/$state_name" AGENT_WATCH_ROOT_FREE_BYTES=$((40 * 1024 * 1024 * 1024)) \
      AGENT_WATCH_SESSIONS_DIR="$scratch/no-session-registry" AGENT_WATCH_SCRATCH_ROOTS="$scratch/no-scratch-roots" \
      AGENT_WATCH_SELFTEST_MARK="$watchdog_selftest_mark" AGENT_WATCH_BREAK="$agent_break" bash "$0" "$@" 2>&1
  }
  unacked_output="$(ack_class_run state-unacked)"; ack_code=$?
  if ((ack_code != 3)) || [[ "$unacked_output" != *"⚠️ 根分区余量过低（本机）："* ]]; then
    echo "  ✗ 自检⑬：没给 --ack 时真起的看门狗子进程应当报「根分区余量过低」、退 3，实际退出码 $ack_code：${unacked_output: -400}"; fail=1   # gate-lint:detail
  fi
  acked_output="$(ack_class_run state-acked --ack 类:根分区余量过低)"; ack_code=$?
  if ((ack_code != 4)) || [[ "$acked_output" != *"已确认接着盯（--ack 本机:根分区余量过低）"* || "$acked_output" == *"⚠️ 根分区余量过低（本机）："* ]]; then
    echo "  ✗ 自检⑬：--ack 类:根分区余量过低 经 watch.sh 起的看门狗子进程应当确认那一类告警（退 4 定时回报、写「已确认接着盯」、不再报「根分区余量过低」），实际退出码 $ack_code：${acked_output: -400}"; fail=1   # gate-lint:detail
  fi
  other_output="$(ack_class_run state-other --ack 类:交回之后后台还在跑)"; ack_code=$?
  if ((ack_code != 3)) || [[ "$other_output" != *"⚠️ 根分区余量过低（本机）："* ]]; then
    echo "  ✗ 自检⑬：--ack 类:交回之后后台还在跑 不该确认别的类的告警，「根分区余量过低」应当照报、退 3，实际退出码 $ack_code：${other_output: -400}"; fail=1   # gate-lint:detail
  fi
  checked=$((checked + 1))
  # ⑭ 文件与告警里推荐的 --ack 写法，原样喂 watch.sh --dry-run 看收不收：watch.conf 里根分区余量那一条、agent-watch.py 文件头的那一句、⑬ 里真起的看门狗打出的告警出路；
  # 推荐写法过不了 watch.sh（改前推荐 --ack 本机:根分区余量过低，被拒「不是 --ack 认的形态」）就红，找不到推荐写法的也红（没有东西可判）。
  # 弄坏开关 WATCH_BREAK=ackrecommendedform 把取到的写法里的「类:」换回「本机:」再喂，这一格红
  recommended_ack_accepted() {   # $1：推荐写法出在哪；$2：从那里取出来的写法，一行一个
    local label="$1" forms="$2" form dry_output
    if [[ -z "$forms" ]]; then
      echo "  ✗ 自检⑭：${label}找不到推荐的 --ack 写法，这一格没有东西可判"; fail=1   # gate-lint:detail
      return
    fi
    while IFS= read -r form; do
      [[ -n "$form" ]] || continue
      [[ "${WATCH_BREAK:-}" == ackrecommendedform ]] && form="${form/类:/本机:}"
      dry_output="$(WATCH_CONF="$scratch/ok.conf" bash "$0" --dry-run --processes --ack "$form" </dev/null 2>&1)" && [[ "$dry_output" == *"--ack $form"* ]] \
        || { echo "  ✗ 自检⑭：${label}推荐的 --ack $form 过不了 watch.sh：$dry_output"; fail=1; }   # gate-lint:detail
    done <<<"$forms"
  }
  recommended_ack_accepted "watch.conf 里" "$(grep -oP -- '--ack \K\S+' "$here/watch.conf")"
  recommended_ack_accepted "agent-watch.py 文件头里" "$(grep -oP -- '^条件类告警：--ack \K\S+' "$watchdog")"
  recommended_ack_accepted "看门狗打出的告警里" "$(grep -oP -- '确认接着跑的用 --ack \K\S+' <<<"$unacked_output")"
  checked=$((checked + 1))
  rm -rf "${scratch:?}"
  if ((fail)); then
    echo "  → 怎么办：按上面那一条改 scripts/agents/watch.sh 里对应的那段"
    exit 1
  fi
  echo "  ✓ watch.sh 自检通过：配置转选项、拼错的 key 拒绝、不像 agent 号的参数拒绝、--processes 只盯进程（找得到当前会话就带上会话目录）、--ack 原样转过去且形态不对就拒绝、进程与书记员汇总两类告警的 --ack 照旧转过去、不给 agent 号找得到会话就走 --discover 找不到就拒绝、--ack 逗号连写拆成一条一个、--ack 类:<告警名> 原样转过去、会话目录取来源仓根的（HOME 指到别处也找对，来源仓根不是绝对路径就拒）、--ack 类:<告警名> 经真起的看门狗子进程只确认那一类告警、文件与告警里推荐的 --ack 写法都过得了 watch.sh（判了 $checked 条）"
  exit 0
}

dry_run=0
mode="agents"
agents=()
acks=()
ack_head='^(a[0-9a-f]{8,}|进程|类):.+$'   # --ack 认的头：<agent 号>、进程、类
[[ "${WATCH_BREAK:-}" == ackscopeform ]] && ack_head='^(a[0-9a-f]{8,}|进程):.+$'   # 弄坏开关：只认两种头（自检⑪红）
while (($#)); do
  argument="$1"
  shift
  case "$argument" in
    --selftest) selftest ;;
    --dry-run) dry_run=1 ;;
    --processes) mode="processes" ;;
    --report) mode="report" ;;
    --ack)
      (($#)) || die "--ack 后面没给要确认的告警" "写成 --ack <agent 号>:<告警名>，进程告警写 --ack 进程:<pid>，一类告警写 --ack 类:<告警名>"
      [[ "$1" =~ $ack_head ]] || die "「$1」不是 --ack 认的形态" "写成 <agent 号>:<告警名>（告警名照看门狗报出来的那个，例如 上下文过大），或 进程:<pid>、类:<告警名>；几条可逗号连写，头一条也要是这几种形态"
      if [[ "${WATCH_BREAK:-}" == ackcommajoined ]]; then
        acks+=(--ack "$1")   # 弄坏开关：逗号连写不拆（改前的样子）
      else
        # 逗号连写（文件头 --ack 那一行）：逗号后面接「进程:」「类:」或「<agent 号>:」的才是下一条，别的逗号留在前一条的告警名里
        ack_entries=()
        IFS=',' read -r -a ack_pieces <<<"$1"
        for ack_piece in "${ack_pieces[@]}"; do
          if [[ "$ack_piece" =~ $ack_head || ${#ack_entries[@]} -eq 0 ]]; then
            ack_entries+=("$ack_piece")
          else
            ack_entries[${#ack_entries[@]} - 1]+=",$ack_piece"
          fi
        done
        for ack_entry in "${ack_entries[@]}"; do acks+=(--ack "$ack_entry"); done
      fi
      shift
      ;;
    -*) die "不认识的选项：$argument" "只有 --processes、--report、--ack、--dry-run、--selftest；阈值写进 $conf，不在命令行上给" ;;
    *)
      IFS=',' read -r -a parts <<<"$argument"
      for part in "${parts[@]}"; do
        [[ -z "$part" ]] && continue
        [[ "$part" =~ ^a[0-9a-f]{8,}$ ]] || die "「$part」不像 agent 号" "agent 号是派发结果里的 agentId（a 加一串十六进制）"
        agents+=("$part")
      done
      ;;
  esac
done

mapfile -t options < <(config_options)
# config_options 里的 die 在子 shell 里退出，这里要把它的失败接住
config_options >/dev/null || exit 2

case "$mode" in
  agents)
    if ((${#agents[@]} == 0)); then
      # 不给 agent 号：一个会话一个看门狗，自己找还没结束的子 agent
      [[ -n "${CLAUDE_CODE_SESSION_ID:-}" ]] || die "没给 agent 号，环境里也没有 CLAUDE_CODE_SESSION_ID，找不到当前会话" "在 Claude Code 的会话里起它；或者给要盯的 agent 号"
      session_base="$(repository_session_directory)" || exit 2
      session_dir="$session_base/$CLAUDE_CODE_SESSION_ID"
      [[ -d "$session_dir" ]] || die "会话目录不存在：$session_dir" "核一下 CLAUDE_CODE_SESSION_ID 与项目路径；或者给要盯的 agent 号"
      run_or_print python3 "$watchdog" watch --discover --session-dir "$session_dir" "${options[@]}" "${acks[@]}"
      exit 0
    fi
    joined="$(IFS=,; echo "${agents[*]}")"
    run_or_print python3 "$watchdog" watch --agents "$joined" "${options[@]}" "${acks[@]}"
    ;;
  processes)
    ((${#agents[@]})) && die "--processes 不带 agent 号" "盯子 agent 就不加 --processes"
    # 找得到当前会话就把会话目录交过去，同时查交回之后后台还在跑；找不到时 agent-watch.py 在报告里写明这一类没查
    session_options=()
    if [[ -n "${CLAUDE_CODE_SESSION_ID:-}" ]]; then
      session_base="$(repository_session_directory)" || exit 2
      session_dir="$session_base/$CLAUDE_CODE_SESSION_ID"
      [[ -d "$session_dir" ]] && session_options=(--session-dir "$session_dir")
    fi
    run_or_print python3 "$watchdog" watch --processes-only "${session_options[@]}" "${options[@]}" "${acks[@]}"
    ;;
  report)
    [[ -n "${CLAUDE_CODE_SESSION_ID:-}" ]] || die "环境里没有 CLAUDE_CODE_SESSION_ID，找不到当前会话" "在 Claude Code 的会话里跑；要看别的会话就直接调 agent-watch.py report --session-dir"
    session_base="$(repository_session_directory)" || exit 2
    session_dir="$session_base/$CLAUDE_CODE_SESSION_ID"
    [[ -d "$session_dir" ]] || die "会话目录不存在：$session_dir" "核一下 CLAUDE_CODE_SESSION_ID 与项目路径"
    run_or_print python3 "$watchdog" report --session-dir "$session_dir"
    ;;
esac
