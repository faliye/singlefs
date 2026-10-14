#!/usr/bin/env bash
# admission: always 每一次调都是用户或主 agent 要登记、关掉或查看一条提醒，登记目录随时在变，上一次的结论不替这一次作保
# run-condition: command sleep awk
# gate-similar: cache-keepalive.sh 同样是 run_in_background 起、到点退出叫醒会话的计时脚本，但它只给子 agent 续提示缓存（固定 230 秒、不登记、不循环）；这里按分钟登记、可循环、有 stop 与 list，到点之后靠 Stop 钩子 remind-rearm-stop-guard.sh 自启动，并进去会把两种用途的退出语义搅在一起
# 本工程唯一的定时提醒（用户 2026-10-14 定：「这个脚本可以输入提醒周期时间 提醒内容 和是否循环」「脚本到时间提醒 然后自启动」
# 「没有 --once 的时候就无限自启动或者被关闭」「禁止其他的设置时间的防止，仅仅允许这一个」）。别的定时办法（CronCreate、ScheduleWakeup、
# RemoteTrigger、只为定时而 sleep 的 Bash 与 Monitor）由 .claude/hooks/timer-only-remind-guard.sh 在执行前拒绝。写法与到点之后做什么在 .claude/skills/remind/SKILL.md。
#
# 用法（主 agent 用 Bash 的 run_in_background 起 start；它等到点、打一行「REMIND …」、退出，退出就叫醒主 agent）：
#   remind.sh start --every <分钟> --message <提醒内容> [--name <名字>] [--once]
#       登记一条提醒，等 <分钟>（正整数），打一行 REMIND，退出。名字缺省 default，只许字母、数字、下划线、连字符。
#       不带 --once：登记留着，循环提醒。这一次退出之后，主 agent 一结束这一轮，停止钩子 remind-rearm-stop-guard.sh 看到登记在、
#         等着的进程不在，就拦下结束、给出照原样重起的命令（登记里的 restart= 那一行），所以它一直自启动，直到 remind.sh stop。
#       带 --once：到点提醒一次就删掉登记，不再自启动。
#       同名的已经有一个在等，这一次不另起（退 0，说明在等哪个进程）。
#   remind.sh stop [--name <名字>]     关掉一条提醒：删登记，还在等的那个进程连同它的计时一并停掉
#   remind.sh list                    列这个会话登记着的提醒，各自在不在等
#   remind.sh --selftest              用秒级的周期走一遍，再逐个打开弄坏开关核它红在点名的格
#   remind.sh --selftest-cells        只走各格（--selftest 内部用，REMIND_BREAK 在这里生效）
# 登记放在 ${REMIND_STATE_ROOT:-~/.local/state/singlefs/reminders}/<会话号>/<名字>.conf，会话号取 CLAUDE_CODE_SESSION_ID（停止钩子按它认是哪个会话的）；
# 等着的进程号写在同目录的 <名字>.pid，已提醒的次数写在 <名字>.count。
# 退出码：0 做完；2 用法错或认不出会话（stderr 写卡在哪与下一步）；1 自证红；78 准入或运行条件不满足（preflight）。
# REMIND_PERIOD_UNIT_SECONDS 把「一分钟」缩成这么多秒（缺省 60），只给自证和试跑用。
#
# 弄坏开关 REMIND_BREAK=<项>（只给 --selftest-cells 证红用，逗号连几个；平常跑时不认）。项名、弄坏哪一处、打开后红在哪一格（格名是「✗ 自证 <格名>：」里的那一段）：
#   once-keeps-registration   --once 提醒完不删登记                             once
#   loop-drops-registration   循环提醒完把登记删了                               loop
#   count-resets              次数不跨重启累加（每次都算第 1 次）                 loop-count
#   restart-drops-message     登记里的 restart= 那一行丢了 --message            loop-count
#   no-duplicate-guard        同名已有一个在等时仍另起一个                       duplicate
#   list-always-waiting       list 把没在等的也写成在等                          list
#   stop-leaves-waiter        stop 只删登记、不停等着的进程                       stop
#   trap-leaves-sleeper       等着的进程被停时不停它的计时子进程                   stop-sleeper
#   accept-bad-every          --every 不校验（0、前导零、非数字都收）             usage-every
set -u
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

STATE_ROOT=${REMIND_STATE_ROOT:-$HOME/.local/state/singlefs/reminders}
SESSION=${CLAUDE_CODE_SESSION_ID:-}
SELF="$(readlink -f "${BASH_SOURCE[0]}")"
PERIOD_UNIT_SECONDS=${REMIND_PERIOD_UNIT_SECONDS:-60}

broken() { [[ ",${REMIND_BREAK:-}," == *",$1,"* ]]; }

refuse() {   # refuse <卡在哪> <下一步>：打出卡在哪与下一步，退 2
  echo "✗ $1" >&2
  echo "→ 怎么办：$2" >&2
  exit 2
}

usage() {
  refuse "$1" "用法：remind.sh start --every <分钟> --message <提醒内容> [--name <名字>] [--once] | stop [--name <名字>] | list | --selftest；用 Bash 的 run_in_background 起 start，细节看 .claude/skills/remind/SKILL.md"
}

need_session() {
  [[ -n "$SESSION" ]] || refuse "没有 CLAUDE_CODE_SESSION_ID：认不出这是哪个会话的提醒" "在 Claude Code 会话的 Bash 里起它；会话外试跑自己设 CLAUDE_CODE_SESSION_ID=<任意名字> 与 REMIND_STATE_ROOT=<临时目录>"
}

# 这一条提醒等着的进程号（活着、命令行里确是 remind.sh 才算），没有就什么都不打
waiting_pid() {   # waiting_pid <登记目录> <名字>
  local pid
  [[ -f "$1/$2.pid" ]] || return 0
  pid="$(cat "$1/$2.pid")"
  [[ "$pid" =~ ^[0-9]+$ ]] || return 0
  if kill -0 "$pid" 2>/dev/null && grep -q remind.sh "/proc/$pid/cmdline" 2>/dev/null; then echo "$pid"; fi
}

start() {
  local every="" message="" name=default once=0 has_every=0 has_message=0
  while (( $# )); do
    case "$1" in
      --every) (( $# >= 2 )) || usage "--every 后面缺值"; every="$2"; has_every=1; shift 2 ;;
      --message) (( $# >= 2 )) || usage "--message 后面缺值"; message="$2"; has_message=1; shift 2 ;;
      --name) (( $# >= 2 )) || usage "--name 后面缺值"; name="$2"; shift 2 ;;
      --once) once=1; shift ;;
      *) usage "start 认不出参数：$1" ;;
    esac
  done
  (( has_every )) || usage "start 缺 --every <分钟>"
  (( has_message )) || usage "start 缺 --message <提醒内容>"
  if ! broken accept-bad-every; then
    [[ "$every" =~ ^[1-9][0-9]*$ ]] || usage "--every 要是正整数分钟（不带前导零），收到「$every」"
  fi
  [[ -n "$message" ]] || usage "--message 不能是空的"
  [[ "$message" != *$'\n'* ]] || usage "--message 要写成一行（登记按行存），收到的里面有换行"
  [[ "$name" =~ ^[A-Za-z0-9_-]+$ ]] || usage "--name 只许字母、数字、下划线、连字符，收到「$name」"
  need_session
  local dir="$STATE_ROOT/$SESSION" conf pidfile count_file count=0 existing sleeper="" seconds
  mkdir -p "$dir"
  conf="$dir/$name.conf"; pidfile="$dir/$name.pid"; count_file="$dir/$name.count"
  existing="$(waiting_pid "$dir" "$name")"
  if [[ -n "$existing" ]] && ! broken no-duplicate-guard; then
    echo "· 提醒「$name」已经有一个在等（进程 $existing），这一次不另起；要换内容或周期先 remind.sh stop --name $name"
    exit 0
  fi
  {
    printf 'every=%s\n' "$every"; printf 'once=%s\n' "$once"; printf 'message=%s\n' "$message"
    if broken restart-drops-message; then
      printf 'restart=bash %q start --every %q --name %q%s\n' "$SELF" "$every" "$name" "$([[ $once == 1 ]] && echo ' --once')"
    else
      printf 'restart=bash %q start --every %q --message %q --name %q%s\n' "$SELF" "$every" "$message" "$name" "$([[ $once == 1 ]] && echo ' --once')"
    fi
  } >"$conf.new"
  mv "$conf.new" "$conf"
  if [[ -f "$count_file" ]] && ! broken count-resets; then count="$(cat "$count_file")"; fi
  seconds="$(awk -v every="$every" -v unit="$PERIOD_UNIT_SECONDS" 'BEGIN { print every * unit }')"
  # 先装好停止时的收尾，再起计时子进程，最后才写进程号：stop 读得到进程号时，收尾一定已经在位
  if broken trap-leaves-sleeper; then
    trap 'rm -f "$pidfile"; exit 143' TERM INT
  else
    trap 'kill "$sleeper" 2>/dev/null; rm -f "$pidfile"; exit 143' TERM INT
  fi
  sleep "$seconds" &
  sleeper=$!
  echo $$ >"$pidfile"
  wait "$sleeper"
  count=$((count + 1)); echo "$count" >"$count_file"
  echo "REMIND $(TZ=Asia/Tokyo date '+%m-%d %H:%M JST') 「$name」第 $count 次：$message"   # clock-times:allow 提醒行带到点的时刻，是运行时输出
  rm -f "$pidfile"
  if (( once )); then
    if ! broken once-keeps-registration; then rm -f "$conf" "$count_file"; fi
    echo "· 只提醒一次（--once），登记已删"
  else
    if broken loop-drops-registration; then rm -f "$conf"; fi
    echo "· 循环提醒：这一轮结束时停止钩子会让主 agent 照原样重起；要关：bash $SELF stop --name $name"
  fi
}

stop() {
  local name=default dir pid
  while (( $# )); do
    case "$1" in
      --name) (( $# >= 2 )) || usage "--name 后面缺值"; name="$2"; shift 2 ;;
      *) usage "stop 认不出参数：$1" ;;
    esac
  done
  [[ "$name" =~ ^[A-Za-z0-9_-]+$ ]] || usage "--name 只许字母、数字、下划线、连字符，收到「$name」"
  need_session
  dir="$STATE_ROOT/$SESSION"
  if [[ ! -e "$dir/$name.conf" && ! -e "$dir/$name.pid" ]]; then
    echo "· 没有叫「$name」的提醒（已经关了，或名字不对）；bash $SELF list 看登记着哪些"
    return 0
  fi
  pid="$(waiting_pid "$dir" "$name")"
  rm -f "$dir/$name.conf" "$dir/$name.pid" "$dir/$name.count"
  if [[ -n "$pid" ]] && ! broken stop-leaves-waiter; then kill "$pid" 2>/dev/null; fi
  echo "✓ 提醒「$name」关了（登记删了，等着的进程停了）"
}

list() {
  need_session
  local dir="$STATE_ROOT/$SESSION" listed=0 conf name pid waiting kind
  for conf in "$dir"/*.conf; do
    [[ -f "$conf" ]] || continue
    listed=$((listed + 1))
    name="$(basename "$conf" .conf)"
    pid="$(waiting_pid "$dir" "$name")"
    waiting=没在等
    if [[ -n "$pid" ]]; then waiting="在等（进程 $pid）"; fi
    if broken list-always-waiting; then waiting="在等（进程 ${pid:-0}）"; fi
    kind=循环
    if [[ "$(sed -n 's/^once=//p' "$conf")" == 1 ]]; then kind=只一次; fi
    echo "· $name：每 $(sed -n 's/^every=//p' "$conf") 分钟，$kind，$waiting；内容：$(sed -n 's/^message=//p' "$conf")"
  done
  echo "✓ 这个会话登记着 $listed 条提醒"
}

# ── 自证：每一格造一个答案已知的输入，判出来与答案相同；失败行「✗ 自证 <格名>：…」后面跟一行「→」
cells_failed=0
cells_checked=0
cell_fail() {   # cell_fail <格名> <现象>
  cells_failed=$((cells_failed + 1))
  echo "✗ 自证 $1：$2"
  echo "→ 怎么办：看 remind.sh 里 $1 对应的那一段（start 的登记与收尾、stop、list、参数校验）；REMIND_BREAK 设着的话这里本来就该红"
}
cell_check() {   # cell_check <格名> <条件成立与否 0/1> <现象>
  cells_checked=$((cells_checked + 1))
  [[ "$2" == 0 ]] || cell_fail "$1" "$3"
}
run_remind() {   # run_remind <环境前缀> <参数…>：在自证的临时登记里跑本脚本，输出与退出码落到 $cell_out、$cell_rc
  local prefix="$1"; shift
  cell_out="$(env $prefix REMIND_STATE_ROOT="$selftest_root" CLAUDE_CODE_SESSION_ID=selftest-session bash "$SELF" "$@" 2>&1)"
  cell_rc=$?
}
own_children_of() { ps -o pid= --ppid "$1" 2>/dev/null | tr -d ' '; }

run_cells() {
  local dir="$selftest_root/selftest-session" pid sleeper conf flag state
  local tricky='含 空格 和 "引号" 与 $变量 的内容'
  mkdir -p "$dir"
  # 只提醒一次：打一行、删登记
  run_remind "REMIND_PERIOD_UNIT_SECONDS=0.2" start --every 1 --message 试跑一次 --name once-case --once
  state=1; if [[ $cell_rc == 0 && "$cell_out" == *"「once-case」第 1 次：试跑一次"* && ! -e "$dir/once-case.conf" && ! -e "$dir/once-case.count" ]]; then state=0; fi
  cell_check once $(( state )) "--once 应当提醒一次并删掉登记与次数，实际退 $cell_rc，输出：$cell_out"
  # 循环：提醒、登记留着、有照原样重起的一行
  run_remind "REMIND_PERIOD_UNIT_SECONDS=0.2" start --every 1 --message "$tricky" --name loop-case
  conf="$dir/loop-case.conf"
  state=1; if [[ $cell_rc == 0 && "$cell_out" == *"「loop-case」第 1 次：$tricky"* && -f "$conf" ]] && grep -q '^once=0$' "$conf" && grep -q '^restart=bash .*remind.sh start --every 1 ' "$conf"; then state=0; fi
  cell_check loop $(( state )) "循环的应当提醒、登记留着（once=0、有 restart= 一行），实际退 $cell_rc，输出：$cell_out"
  # 重起：登记里的 restart= 原样跑，次数接着数，内容（带空格、引号、美元符）一字不差
  state=1
  if [[ -f "$conf" ]]; then
    cell_out="$(env REMIND_PERIOD_UNIT_SECONDS=0.2 REMIND_STATE_ROOT="$selftest_root" CLAUDE_CODE_SESSION_ID=selftest-session bash -c "$(sed -n 's/^restart=//p' "$conf")" 2>&1)"
    cell_rc=$?
    if [[ $cell_rc == 0 && "$cell_out" == *"「loop-case」第 2 次：$tricky"* ]]; then state=0; fi
  fi
  cell_check loop-count $(( state )) "按登记里的 restart= 重起应当是第 2 次、内容原样，实际退 $cell_rc，输出：$cell_out"
  # 下面的 REMIND_PERIOD_UNIT_SECONDS=30 只是让假的等待够久、能被 stop 停掉，与真实的提醒周期无关，判完就停。
  # 同名已有一个在等：不另起、退 0、说出在等哪个进程；list 把它写成在等、没在等的写成没在等
  env REMIND_PERIOD_UNIT_SECONDS=30 REMIND_STATE_ROOT="$selftest_root" CLAUDE_CODE_SESSION_ID=selftest-session bash "$SELF" start --every 1 --message 等着的 --name wait-case >/dev/null 2>&1 &
  pid=$!
  for _ in $(seq 1 100); do [[ -f "$dir/wait-case.pid" ]] && break; sleep 0.05; done
  sleeper="$(own_children_of "$pid")"
  cell_out="$(env REMIND_PERIOD_UNIT_SECONDS=30 REMIND_STATE_ROOT="$selftest_root" CLAUDE_CODE_SESSION_ID=selftest-session timeout 3 bash "$SELF" start --every 1 --message 另一个 --name wait-case 2>&1)"
  cell_rc=$?
  state=1; if [[ $cell_rc == 0 && "$cell_out" == *"已经有一个在等（进程 $pid）"* ]]; then state=0; fi
  cell_check duplicate $(( state )) "同名在等时应当退 0 并说「已经有一个在等（进程 $pid）」，实际退 $cell_rc，输出：$cell_out"
  run_remind "" list
  state=1; if [[ "$cell_out" == *"wait-case：每 1 分钟，循环，在等（进程 $pid）"* && "$cell_out" == *"loop-case：每 1 分钟，循环，没在等"* && "$cell_out" == *"登记着 2 条"* ]]; then state=0; fi
  cell_check list $(( state )) "list 应当列出在等的 wait-case（进程 $pid）与没在等的 loop-case，共 2 条，实际：$cell_out"
  # 停：登记删了、等着的进程与它的计时子进程都停了
  sleeper="$(own_children_of "$pid")"
  run_remind "" stop --name wait-case
  for _ in $(seq 1 40); do kill -0 "$pid" 2>/dev/null || break; sleep 0.05; done
  state=1; if [[ $cell_rc == 0 && "$cell_out" == *"「wait-case」关了"* && ! -e "$dir/wait-case.conf" && ! -e "$dir/wait-case.pid" ]] && ! kill -0 "$pid" 2>/dev/null; then state=0; fi
  cell_check stop $(( state )) "stop 之后登记应当删了、等着的进程 $pid 应当停了，实际退 $cell_rc，输出：$cell_out"
  state=1; if [[ -n "$sleeper" ]] && ! kill -0 "$sleeper" 2>/dev/null; then state=0; fi
  cell_check stop-sleeper $(( state )) "等着的进程停时它的计时子进程 ${sleeper:-（没取到）} 应当一并停了"
  # 收自己起的：只对自己起的进程号发信号
  kill "$pid" 2>/dev/null
  if [[ -n "$sleeper" ]]; then kill "$sleeper" 2>/dev/null; fi
  wait "$pid" 2>/dev/null
  # 用法错：每一种都退 2、stderr 带下一步
  run_remind "" bogus
  state=1; if [[ $cell_rc == 2 && "$cell_out" == *"→ 怎么办"* ]]; then state=0; fi
  cell_check usage-subcommand $(( state )) "认不出的子命令应当退 2 并带下一步，实际退 $cell_rc，输出：$cell_out"
  state=0
  for bad_every in 0 08 abc -3 1.5 ""; do
    run_remind "REMIND_PERIOD_UNIT_SECONDS=0.01" start --every "$bad_every" --message x --name bad-every
    if [[ $cell_rc != 2 || "$cell_out" != *"→ 怎么办"* ]]; then state=1; echo "  （--every「$bad_every」实际退 $cell_rc）"; fi
  done
  cell_check usage-every $(( state )) "--every 不是正整数分钟（0、前导零、非数字、负数、小数、空）应当都退 2 并带下一步"
  state=0
  run_remind "" start --every 1 --message ""; [[ $cell_rc == 2 ]] || state=1
  run_remind "" start --every 1; [[ $cell_rc == 2 ]] || state=1
  run_remind "" start --every 1 --message $'两行\n内容'; [[ $cell_rc == 2 ]] || state=1
  cell_check usage-message $(( state )) "--message 空、缺、带换行都应当退 2"
  state=0
  run_remind "" start --every 1 --message x --name "带 空格"; [[ $cell_rc == 2 && "$cell_out" == *"→ 怎么办"* ]] || state=1
  run_remind "" stop --name "../x"; [[ $cell_rc == 2 ]] || state=1
  cell_check usage-name $(( state )) "--name 带空格或斜杠应当退 2 并带下一步"
  cell_out="$(env -u CLAUDE_CODE_SESSION_ID REMIND_STATE_ROOT="$selftest_root" bash "$SELF" list 2>&1)"; cell_rc=$?
  state=1; if [[ $cell_rc == 2 && "$cell_out" == *"CLAUDE_CODE_SESSION_ID"* && "$cell_out" == *"→ 怎么办"* ]]; then state=0; fi
  cell_check usage-session $(( state )) "没有会话号应当退 2 并带下一步，实际退 $cell_rc，输出：$cell_out"
  # 关一条没有的：退 0、说没有、给 list 的出路
  run_remind "" stop --name nothing-here
  state=1; if [[ $cell_rc == 0 && "$cell_out" == *"没有叫「nothing-here」的提醒"* ]]; then state=0; fi
  cell_check stop-unknown $(( state )) "关一条不存在的应当退 0 并说没有，实际退 $cell_rc，输出：$cell_out"
}

SELFTEST_BREAKS=(
  "once-keeps-registration once" "loop-drops-registration loop" "count-resets loop-count" "restart-drops-message loop-count"
  "no-duplicate-guard duplicate" "list-always-waiting list" "stop-leaves-waiter stop" "trap-leaves-sleeper stop-sleeper" "accept-bad-every usage-every"
)

selftest_cells() {
  selftest_root="$(mktemp -d)"
  trap 'rm -rf "${selftest_root:?}"' EXIT
  run_cells
  if (( cells_failed )); then
    echo "✗ 自证：$cells_failed 格红（共判了 $cells_checked 格）"
    echo "→ 怎么办：照上面每一格的下一步改 remind.sh，再单独跑 bash $SELF --selftest"
    exit 1
  fi
  echo "✓ 各格都对（判了 $cells_checked 格）"
}

selftest() {
  local out rc entry switch cell broken_checked=0 failed=0
  out="$(bash "$SELF" --selftest-cells 2>&1)"; rc=$?
  if (( rc != 0 )); then
    echo "✗ 自证：不带弄坏开关时各格应当全绿，实际退 $rc："
    echo "$out" | sed 's/^/    /'   # gate-lint:detail
    echo "→ 怎么办：先让 --selftest-cells 不带 REMIND_BREAK 跑绿，再回来跑本自证"
    exit 1
  fi
  for entry in "${SELFTEST_BREAKS[@]}"; do
    switch="${entry%% *}"; cell="${entry##* }"
    out="$(REMIND_BREAK="$switch" bash "$SELF" --selftest-cells 2>&1)"; rc=$?
    broken_checked=$((broken_checked + 1))
    if (( rc == 0 )) || [[ "$out" != *"✗ 自证 $cell："* ]]; then
      failed=$((failed + 1))
      echo "✗ 自证：打开弄坏开关 $switch 之后应当红在「$cell」格，实际退 $rc，输出："
      echo "$out" | sed 's/^/    /'   # gate-lint:detail
      echo "→ 怎么办：$switch 在 remind.sh 里弄坏的那一处要真影响「$cell」格的判定；两边对不上就改开关或改格"
    fi
  done
  if (( failed )); then exit 1; fi
  echo "✓ remind.sh 自证：各格都对，$broken_checked 个弄坏开关各红在点名的格"
}

case "${1:-}" in
  start) shift; start "$@" ;;
  stop) shift; stop "$@" ;;
  list) shift; (( $# == 0 )) || usage "list 不带参数，收到：$*"; list ;;
  --selftest) selftest ;;
  --selftest-cells) selftest_cells ;;
  "") usage "没给子命令" ;;
  *) usage "认不出的子命令：$1" ;;
esac
