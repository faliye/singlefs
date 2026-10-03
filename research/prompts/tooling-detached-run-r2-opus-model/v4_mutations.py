#!/usr/bin/env python3
"""V4：detached-run.sh 的几种「看起来对、其实错」的实现。用法：v4_mutations.py <原脚本> <写到哪> <名>
名：control（只改自证前缀）、a-linger-unreadable-silent、b-default-stop-timeout-unset、c-stop-nonblocking、
    f-query-check-after-archive、g-watch-skips-unknown。每种都先把自证前缀 selftest-<pid>- 换成 opusr2-<pid>-（单元名 singlefs-detached-opusr2-…）。"""
import sys
source, destination, name = sys.argv[1:4]
text = open(source, encoding="utf-8").read()
def swap(old, new, count=1):
    global text
    assert text.count(old) == count, (name, old[:80], text.count(old))
    text = text.replace(old, new)
swap('prefix="selftest-$$-"', 'prefix="opusr2-$$-"', 2)
if name == "a-linger-unreadable-silent":   # 读不到 Linger 时不警告（Ⓛ 只用替身打 no / yes）
    swap('    echo "  ! 读不到 loginctl show-user $user 的 Linger：判不出最后一个登录会话结束时用户级 systemd 会不会连这里起的单元一起停；照起" >&2\n',
         '    :\n')
elif name == "b-default-stop-timeout-unset":   # 只有明写 --stop-timeout 才设进单元；默认 180 只写进登记（Ⓣ 只测明写的 2 秒）
    swap('stop_timeout="$2"; shift 2 ;;', 'stop_timeout="$2"; stop_timeout_given=1; shift 2 ;;')
    swap('stop_timeout="${1#--stop-timeout=}"; shift ;;', 'stop_timeout="${1#--stop-timeout=}"; stop_timeout_given=1; shift ;;')
    swap('  if ! detached_run_break_is stop-timeout-ignored; then properties+=',
         '  if ! detached_run_break_is stop-timeout-ignored && [[ -n "${stop_timeout_given:-}" ]]; then properties+=')
elif name == "c-stop-nonblocking":   # stop 不等停机任务（--no-block），至多等 5 秒 exit 与单元停下，等不到就记 stop-killed
    swap('  if ! systemctl --user stop "$unit"; then', '  if ! systemctl --user stop --no-block "$unit"; then')
    swap('''  state="$(detached_run_unit_state "$unit")"
  if detached_run_is_active_state "$state"; then
    echo "  ✗ 停过之后单元 $unit 还是 $state" >&2
    echo "     → 怎么办：systemctl --user status $unit 看它卡在哪；别按名字或进程组去停，交主 agent 判断" >&2
    exit 1
  fi
''', '')
    swap('''  while [[ ! -f "$record/exit" ]] && ((tries < 50)); do
    sleep 0.1
    tries=$((tries + 1))
  done
  rc="$(detached_run_field "$record" exit)"''', '''  while ((tries < 50)); do
    state="$(detached_run_unit_state "$unit")"
    if [[ -f "$record/exit" ]] && ! detached_run_is_active_state "$state"; then break; fi
    sleep 0.1
    tries=$((tries + 1))
  done
  state="$(detached_run_unit_state "$unit")"
  rc="$(detached_run_field "$record" exit)"''')
elif name == "f-query-check-after-archive":   # 查不到的判挪到「挪走旧登记」之后（Ⓠ 的 start 用的是新名，没有旧登记）
    block = '''  if [[ "$state" == "$DETACHED_RUN_QUERY_FAILED" ]]; then
    detached_run_query_failed "$unit" "判不出同名的还在不在跑：这一次什么都没起，旧登记没动"
  fi
'''
    swap(block, '')
    swap('  if ! mkdir -- "$record"; then', block + '  if ! mkdir -- "$record"; then')
elif name == "g-watch-skips-unknown":   # watch 只记还在跑的、不记查不到的（Ⓠ 只核 watch 不退）
    swap('''    detached_run_observe ${DETACHED_RUN_RUNNING_RECORDS[@]+"${DETACHED_RUN_RUNNING_RECORDS[@]}"} \\
      ${DETACHED_RUN_UNKNOWN_RECORDS[@]+"${DETACHED_RUN_UNKNOWN_RECORDS[@]}"}''',
         '''    detached_run_observe ${DETACHED_RUN_RUNNING_RECORDS[@]+"${DETACHED_RUN_RUNNING_RECORDS[@]}"}''')
elif name != "control":
    sys.exit(f"认不出 {name}")
open(destination, "w", encoding="utf-8").write(text)
print(f"{name} → {destination}")
