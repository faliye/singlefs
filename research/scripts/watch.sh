#!/usr/bin/env bash
# 看门狗的调用入口：主 agent 每次派发（含续做）之后照 .claude/main-agent.md「派出去之后」起它，调用时只填要盯的 agent 号，
# 会话目录由 agent-watch.py 按 agent 号自己找，阈值从 research/scripts/watch.conf 读——不在命令行上手敲。
#
# 一律放进 Bash 工具的 run_in_background；这里不再自己放后台（放了之后跑完的那个叫不醒主 agent）。
#   bash research/scripts/watch.sh <agent 号>[,<agent 号>…] [<agent 号> …]   盯这几个子 agent 与它们起的长进程
#   bash research/scripts/watch.sh --processes                           只盯这个 Claude 实例底下已经在跑的长进程（主 agent 自己起的长命令）；
#                                                                        找得到当前会话（CLAUDE_CODE_SESSION_ID）就同时查交回之后后台还在跑
#   bash research/scripts/watch.sh --report                              当前会话的全部子 agent 现在是什么样，报一次就退
#   bash research/scripts/watch.sh … --ack <agent 号>:<告警名>            看过、判定只是慢的那一条告警，这一次看门狗里不再为它叫醒（可给多次；进程告警写 --ack 进程:<pid>）
#   bash research/scripts/watch.sh --dry-run …                           只打印要跑的命令，不跑
#   bash research/scripts/watch.sh --selftest
set -uo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
conf="${WATCH_CONF:-$here/watch.conf}"
watchdog="$here/agent-watch.py"

die() { echo "  ✗ $1" >&2; echo "     → 怎么办：$2" >&2; exit 2; }

# 配置转成选项；不认识的 key、不是整数的值都拒绝，免得一个拼错的 key 被安静地丢掉
config_options() {
  local line key value known=" interval-seconds max-minutes tool-minutes wait-loop-minutes idle-minutes repeat-count context-tokens process-report-minutes process-stale-minutes process-max-minutes not-started-minutes active-minutes detection-poll-seconds leftover-background-grace-minutes "
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
  scratch="$(mktemp -d)"
  # ① 配置里的 key 原样转成选项、agent 号逗号与空格都认
  printf 'tool-minutes = 12\n# 注释\n\nmax-minutes=30\nleftover-background-grace-minutes=3\n' > "$scratch/ok.conf"
  out="$(WATCH_CONF="$scratch/ok.conf" bash "$0" --dry-run a0000000000000001,a0000000000000002 a0000000000000003 2>&1)"
  [[ "$out" == *"--agents a0000000000000001,a0000000000000002,a0000000000000003"* && "$out" == *"--tool-minutes 12"* && "$out" == *"--max-minutes 30"* \
     && "$out" == *"--leftover-background-grace-minutes 3"* ]] \
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
  out="$(HOME="$scratch/home" CLAUDE_CODE_SESSION_ID=selftest-session WATCH_CONF="$scratch/ok.conf" bash "$0" --dry-run --processes 2>&1)"
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
  rm -rf "${scratch:?}"
  if ((fail)); then
    echo "  → 怎么办：按上面那一条改 research/scripts/watch.sh 里对应的那段"
    exit 1
  fi
  echo "  ✓ watch.sh 自检通过：配置转选项、拼错的 key 拒绝、不像 agent 号的参数拒绝、--processes 只盯进程（找得到当前会话就带上会话目录）、--ack 原样转过去且形态不对就拒绝（判了 $checked 条）"
  exit 0
}

dry_run=0
mode="agents"
agents=()
acks=()
while (($#)); do
  argument="$1"
  shift
  case "$argument" in
    --selftest) selftest ;;
    --dry-run) dry_run=1 ;;
    --processes) mode="processes" ;;
    --report) mode="report" ;;
    --ack)
      (($#)) || die "--ack 后面没给要确认的告警" "写成 --ack <agent 号>:<告警名>，进程告警写 --ack 进程:<pid>"
      [[ "$1" =~ ^(a[0-9a-f]{8,}|进程):.+$ ]] || die "「$1」不是 --ack 认的形态" "写成 <agent 号>:<告警名>（告警名照看门狗报出来的那个，例如 上下文过大），或 进程:<pid>"
      acks+=(--ack "$1")
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
    ((${#agents[@]})) || die "没给要盯的 agent 号" "bash research/scripts/watch.sh <agent 号>[,<agent 号>…]；只盯主 agent 自己的长进程用 --processes"
    joined="$(IFS=,; echo "${agents[*]}")"
    run_or_print python3 "$watchdog" watch --agents "$joined" "${options[@]}" "${acks[@]}"
    ;;
  processes)
    ((${#agents[@]})) && die "--processes 不带 agent 号" "盯子 agent 就不加 --processes"
    # 找得到当前会话就把会话目录交过去，同时查交回之后后台还在跑；找不到时 agent-watch.py 在报告里写明这一类没查
    session_options=()
    session_dir="$HOME/.claude/projects/${root//\//-}/${CLAUDE_CODE_SESSION_ID:-}"
    [[ -n "${CLAUDE_CODE_SESSION_ID:-}" && -d "$session_dir" ]] && session_options=(--session-dir "$session_dir")
    run_or_print python3 "$watchdog" watch --processes-only "${session_options[@]}" "${options[@]}" "${acks[@]}"
    ;;
  report)
    [[ -n "${CLAUDE_CODE_SESSION_ID:-}" ]] || die "环境里没有 CLAUDE_CODE_SESSION_ID，找不到当前会话" "在 Claude Code 的会话里跑；要看别的会话就直接调 agent-watch.py report --session-dir"
    session_dir="$HOME/.claude/projects/${root//\//-}/$CLAUDE_CODE_SESSION_ID"
    [[ -d "$session_dir" ]] || die "会话目录不存在：$session_dir" "核一下 CLAUDE_CODE_SESSION_ID 与项目路径"
    run_or_print python3 "$watchdog" report --session-dir "$session_dir"
    ;;
esac
