#!/usr/bin/env bash
# admission: always Claude Code 每一次触发都要现判这一次调用，上一次的结论不替这一次作保
# run-condition: command python3
# PreToolUse hook（Bash）：门禁（.claude/gate.d/*.sh、gate.sh）、cargo test、四个入口脚本（replay.sh、crash-point-replay.sh、crash-amplification.sh、merge-and-verify.sh）
#   与 gate-staged.sh 的输出，用 >、>>、&>（含 2> 之类的 fd 重定向）或 | tee（含 -a、|&）写到 .log 的，执行前拒绝（退出 2，stderr 写给 agent 的说明），
#   说明里给出 bash scripts/hosts/two-logs.sh <名> -- <原命令> 的写法：日志出两份，<名>.log 全量，<名>.problems.log 只收错误与警告，一般只看后一份。
#   主 agent 与子 agent 一样判。
# hook-events: PreToolUse:Bash
# gate-similar: bash-command-detector.sh 同挂 PreToolUse[Bash]、同用 lib_shell_words.py 切词；它判命令的形状（等待循环、放出追踪、覆盖 research/results 下的产物、终止进程、就地改文件），这里判被盯的那几条命令的输出落在哪种文件，判据不同，也不在别的会话正改着的那份里追加
# gate-similar: heavy-test-guard.sh 同挂 PreToolUse[Bash]、同用 lib_shell_words.py 与 lib_heavy_tests.py；它判谁能跑重型测试、要不要带前缀，这里不管能不能跑，只管输出写法，从它那里借的是 gate_stage_name、located_in、cargo_subcommand 这几个认命令的函数
# gate-similar: run-scope-guard.sh 同挂 PreToolUse[Bash]、同样认门禁阶段与入口脚本、同用 lib_shell_words.py；它判范围声明写没写对，这里判输出重定向写没写成 .log，两件事互不依赖
# gate-similar: timer-only-remind-guard.sh 的 matcher 含 Bash、同用 lib_shell_words.py；它拒 remind.sh 之外的定时写法，与输出落盘无关
# gate-similar: host-resources-reminder.sh 同挂 PreToolUse[Bash]；它认出重活与 GPU 命令时查各台资源出提醒、从不拒，这里拒日志写法，不查外部资源
# gate-similar: pattern-process-guard.sh 上游的钩子，同挂 PreToolUse[Bash]；它拒按模式找进程，判据与 shell-lint 同一份，与日志写法无关，也不许在上游副本里加本工程的判据
#
# 为什么：用户 2026-10-21「过万的日志太长了，日志以后输出两份，一份全量，一份错误警告的，一般就看错误警告的就好了」。入口脚本与门禁里自己写的 .log 已经改成经
#   scripts/hosts/two-logs.sh；这一道拦的是手敲或脚本之外的命令文本里仍然 > x.log、| tee x.log 的写法，免得又回到一份上万行的全量日志。
#
# 判什么（判法在同目录的 lib_two_logs_guard.py，切词与认命令从 lib_shell_words.py、lib_heavy_tests.py 导入，不另写一份）：
#   被盯的命令（前缀与包装剥掉之后）：.claude/gate.d/ 顶层的 *.sh、gate.sh、cargo test（含 cargo t）、scripts/experiment-apparatus/replay.sh、scripts/crash/crash-point-replay.sh、
#     scripts/crash/crash-amplification.sh、scripts/gate/merge-and-verify.sh、scripts/gate/gate-staged.sh；带 --selftest、--help、-h、--list、--list-items、--admission-only 的只查清单与自证，不盯。
#   拒：① 被盯的命令自己的输出重定向（>、>>、>|、&>、&>>）目标是日志；② 同一条管道里有一段 tee（含 -a）、文件参数是日志；③ bash -c '…' 里按这两条再判（递归）；
#     ④ 整组的输出重定向（`{ …; } > x.log`、`( … ) > x.log`）与整组经管道接的 tee（`{ …; } | tee x.log`）挂到组里每一条被盯的命令上，照同一判据（嵌套的组各算一组，
#     `名字=(…)` 数组与 `f()` 的括号不算组；lib_two_logs_guard.py 的 grouped_commands）。
#   「目标是日志」：目标词里有 .log（后面不接字母数字下划线），或引用名字里带 log 的变量（$log、${LOG_FILE}）；/dev/null 与 &1、&2 这类 fd 不算；写到 .txt、.out 的不拦。
#   包在 two-logs.sh 里的命令（two-logs.sh <名> -- <命令>）不判：它自己出两份日志；它外面再 | tee 屏幕副本也放。
# 罩不住的：exec > x.log、变量里拼出来的命令、eval、python 的 subprocess、驱动脚本里写的重定向（只判这一条命令文本）、if / while / for 这类复合命令整体的重定向
#   （`if …; fi > x.log`、`for …; done > x.log`，只认花括号组与圆括号子 shell）；写到不叫 .log 的文件（.txt、.out）的不拦。
# 拒绝说明（stderr，写给 agent）：逐条写命令是什么、写到哪个日志；再写 two-logs.sh 的写法（<名> 去掉 .log 后缀、输出照常转打、退出码是原命令的、只落盘加 --quiet）、
#   照抄的一条（前缀与包装原样放在 two-logs.sh 前面与 -- 后面）、已有全量日志时现筛的 --from-log。被拒的整条命令记一行进检出记录（AGENT_HOOK_DETECTIONS，
#   没设是 lib_work_root.py 的 default_detections_path；session_id、outcome 被拒，看门狗只记不叫醒）。判的时候出异常、读不到判法文件，照常放行并往 stderr 报一句：
#   这个钩子挂在每一条 Bash 上，坏了不许把所有命令都拒掉。判的时候出异常与 hook 输入不是 JSON 对象（诊断带开头若干字节，会话号读不到照实写读不到）另记一行检出
#   （findings 是 stderr 那一句、outcome「只记」）；读不到判法文件那一支由这份入口自己记一行（结局只记，会话号照 hook 输入、读不出写「读不到」；
#   AGENT_HOOK_DETECTIONS 没设时取同目录 lib_work_root.py 的默认，两样都取不到时 stderr 另说一句没记上）。
#   判法文件在而没跑完（导入就出错、被信号停掉，退 0 与 2 以外的码）同样照常放行、stderr 报一句、由这份入口记一行（hook 的 JSON 先读进变量，判法与记检出各读一遍）。
#
# 用法：
#   注册：.claude/settings.json 的 hooks.PreToolUse 里 matcher "Bash" 那一条的 hooks 加 {"type": "command", "command": "bash \"$CLAUDE_PROJECT_DIR\"/.claude/hooks/two-logs-guard.sh"}
#   自证：bash .claude/hooks/two-logs-guard.sh --selftest（登记在 .claude/hooks/selftest-registry.tsv，light）
# 自证：内存里逐格判（被盯的命令各一条拒，套了 nice / 声明 / capped.sh / timeout 的、bash -c 里的、管道 tee 的各一格；two-logs.sh 包着、写 .txt、写 /dev/null、
#   --list / --selftest、cargo build、别处同名脚本各一格放），另起这份入口喂 hook 的 JSON 核退出码、拒绝说明里的写法、检出记录、判法文件读不到时放行并记一条检出（设与没设 AGENT_HOOK_DETECTIONS 各一格）、
#   平常跑设了弄坏开关退 2（不带标记、标记文件内容对不上各一格）与带标记的自证子进程照认开关；关守卫项与别的项同写，平常跑与带标记的子进程都退 2 并点名它（各一格）。
#   末了开关轮：下面每个开关逐个打开再判一遍，红的格与下面写的逐格相等。
# 弄坏开关 TWO_LOGS_GUARD_BREAK=<项>（几个用逗号连；只给自证判红用；自证与它起的子进程里认不出的项抛异常，钩子照常放行并报一句；平常跑设了照下一句拒）。
#   只在自证里认（门禁 code-tooling 的 break-switch-outside-selftest 格）：平常跑（不是 --selftest，也不是自证起的子进程）设了它，入口在找判法文件、读 hook 的 JSON 之前
#   就退 2，stderr 点名它只在自证里认；自证起的子进程由环境里的 TWO_LOGS_GUARD_SELFTEST_MARK 认（指的那份文件在、内容是「two-logs-guard.sh --selftest 起的子进程」，
#   同一句是 lib_two_logs_guard.py 的 SELFTEST_MARK_CONTENT）。钩子的环境是 Claude Code 进程的环境，Bash 命令里的 export / unset 改不到它：带着这个变量起的会话里
#   每一条 Bash 都被拒，出路是退出会话、在起 claude 的 shell 里 unset 它（settings 的 env 里写了的删掉）再起。这不算「坏了把所有命令都拒掉」：
#   开关是有人设上的，设着照常放行就是让这道闸带着弄坏的那一处判每一条命令。
#   设了开关再跑 --selftest：照它判一遍、列出红的格、退 1，开关轮不跑；不设时末了跑开关轮，逐个打开、核红的格与这里写的逐格相等。逐项写开关、弄坏哪一处、红的格（写全）：
#   TWO_LOGS_GUARD_BREAK=redirect-unchecked  不判被盯命令的输出重定向
#       红：「bash -c 里的 cargo test > 写 .log 拒」「cargo test &> 写 .log 拒」「gate.sh >> 写 .log 拒」「merge-and-verify.sh > 写 .log 拒」「timeout 包着的 gate-staged.sh > 写 .log 拒」「入口：门禁阶段 > 写 .log 拒（退出码 2）」「套了 capped.sh 的 cargo test > 写 .log 拒」「套了 nice 与声明的 gate-staged.sh > 写 .log 拒」「崩溃放量入口 > 写 .log 拒」「崩溃点重放入口 > 写变量名带 log 的目标拒」「拒绝说明逐条写被盯的命令与目标」「门禁阶段 2>&1 重定向写 .log 拒」「门禁阶段 > 写 .log 拒」
#   TWO_LOGS_GUARD_BREAK=tee-unchecked  不判管道里的 tee
#       红：「bash -c 里的管道 tee 写 .log 拒」「gate-staged.sh |& tee 写 .log 拒」「replay.sh | tee -a 写 .log 拒」「入口：拒绝说明写 two-logs.sh 的写法、被拒的记一行」「拒绝说明逐条写被盯的命令与目标」「门禁阶段 | tee 写 .log 拒」
#   TWO_LOGS_GUARD_BREAK=nested-code-unread  bash -c 里的代码不递归判
#       红：「bash -c 里的 cargo test > 写 .log 拒」「bash -c 里的管道 tee 写 .log 拒」
#   TWO_LOGS_GUARD_BREAK=query-forms-guarded  只查清单与自证的写法也盯
#       红：「崩溃放量 --list-items | tee 写 .log 放」「门禁阶段 --list > 写 .log 放」「门禁阶段 --selftest | tee 写 .log 放」
#   TWO_LOGS_GUARD_BREAK=variable-names-ignored  名字里带 log 的变量目标不算日志
#       红：「崩溃点重放入口 > 写变量名带 log 的目标拒」
#   TWO_LOGS_GUARD_BREAK=all-cargo-guarded  cargo 的别的子命令也盯
#       红：「cargo build > 写 .log 放」
#   TWO_LOGS_GUARD_BREAK=non-log-targets-guarded  写到什么文件都拒
#       红：「门禁阶段 > /dev/null 放」「门禁阶段 > 写 .txt 放」「花括号组 > 写 .txt 放」
#   TWO_LOGS_GUARD_BREAK=no-way-out  拒绝说明不写 two-logs.sh 的出路
#       红：「入口：拒绝说明写 two-logs.sh 的写法、被拒的记一行」
#   TWO_LOGS_GUARD_BREAK=group-redirect-unchecked  不判整组的输出重定向与整组接的 tee（改前的样子）
#       红：「花括号组 > 写 .log 拒」「圆括号子 shell > 写 .log 拒」「花括号组 2>&1 | tee 写 .log 拒」「子 shell 里套数组赋值、组外 > 写 .log 拒」「入口：圆括号子 shell 整组 > 写 .log 拒（退出码 2）」
#   TWO_LOGS_GUARD_BREAK=failure-unrecorded  判不了的三支（hook 输入不是 JSON 对象、判的时候出异常、读不到判法文件）只打 stderr、不记检出（改前的样子）
#       红：「入口：hook 输入不是 JSON 对象（坏 JSON、数组）放行、stderr 写诊断、各记一条检出」「入口：判的时候抛异常放行、stderr 报一句、记一条检出」「入口：判法文件读不到时记一条检出（结局只记、会话号照输入）」「入口：判法文件读不到、没设 AGENT_HOOK_DETECTIONS 时记进 lib_work_root.py 的默认位置」
#   TWO_LOGS_GUARD_BREAK=judge-crash-unrecorded  判法文件在而没跑完（导入就出错、被信号停掉，退 0 与 2 以外的码）时只打 stderr、不记检出（改前的样子：exec 判法，退几都照常放行）
#       红：「入口：判法文件在而导入就出错时放行、stderr 报一句、记一条检出（结局只记、会话号照输入）」
#   TWO_LOGS_GUARD_BREAK=honor-outside-selftest  平常跑（不是 --selftest、也不是自证起的子进程）设了开关也照认、不拒（改前的样子）；它是关守卫项，只在单独一项时认，与别的项同写照拒
#       红：「入口：平常跑设了弄坏开关退 2、点名它只在自证里认」「入口：标记文件内容对不上照拒」
#   TWO_LOGS_GUARD_BREAK=honor-outside-selftest-with-others  带标记的自证子进程里，关守卫项与别的项同写也照认（改前的样子：honor-outside-selftest 写在哪一项都认；只在带标记的子进程里认，平常跑同写照拒）
#       红：「入口：关守卫项与别的项同写：带标记的子进程里也退 2、点名它」
#   开关与红的格以 lib_two_logs_guard.py 的 BREAK_SWITCH_CELLS 为准，开关轮核每个开关名都写在这一段里。
# 退役判据：入口脚本与门禁都改成经 two-logs.sh 出两份日志、连续四周没有一次拒绝记录，或日志约定改了，删掉这个钩子与它的注册。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
# 弄坏开关只在自证里认（文件头「弄坏开关」一段）：找判法文件、读 hook 的 JSON 之前先判；关守卫项 honor-outside-selftest 单独一项时不拒（改前的样子），
#   与别的项同写一律拒、点名它（带标记的子进程里也拒，同写着 honor-outside-selftest-with-others 的除外；records/2026-10-23-subagent拆分提案.md 第一百二十二节 ②）
if [[ -n "${TWO_LOGS_GUARD_BREAK:-}" && "${1:-}" != "--selftest" ]]; then
  two_logs_mark="${TWO_LOGS_GUARD_SELFTEST_MARK:-}"
  two_logs_marked=0
  [[ -n "$two_logs_mark" && -f "$two_logs_mark" && "$(< "$two_logs_mark")" == "two-logs-guard.sh --selftest 起的子进程" ]] && two_logs_marked=1
  two_logs_guard_off=0 two_logs_lenient=0 two_logs_others=()
  IFS=, read -r -a two_logs_switches <<< "$TWO_LOGS_GUARD_BREAK"
  for two_logs_switch in "${two_logs_switches[@]}"; do
    [[ -z "$two_logs_switch" ]] && continue
    if [[ "$two_logs_switch" == honor-outside-selftest ]]; then
      two_logs_guard_off=1
    else
      two_logs_others+=("$two_logs_switch")
      [[ "$two_logs_switch" == honor-outside-selftest-with-others ]] && two_logs_lenient=1
    fi
  done
  if (( two_logs_guard_off == 1 && ${#two_logs_others[@]} > 0 && ! (two_logs_marked == 1 && two_logs_lenient == 1) )); then
    echo "  ✗ two-logs-guard.sh：TWO_LOGS_GUARD_BREAK=${TWO_LOGS_GUARD_BREAK}：关守卫项 honor-outside-selftest 只在它是唯一一项时认，与别的项（${two_logs_others[*]}）同写照拒；弄坏开关只在自证里认，这一条命令没判、不执行" >&2
    echo "  → 怎么办：钩子的环境是 Claude Code 进程的环境，Bash 命令里 unset 改不到它：退出这个会话，在起 claude 的 shell 里 unset TWO_LOGS_GUARD_BREAK 再起；要看某一项红在哪一格，跑 TWO_LOGS_GUARD_BREAK=<项> bash .claude/hooks/two-logs-guard.sh --selftest，一次只设那一项" >&2
    exit 2
  fi
  if (( two_logs_marked == 0 && ! (two_logs_guard_off == 1 && ${#two_logs_others[@]} == 0) )); then
    echo "  ✗ two-logs-guard.sh：TWO_LOGS_GUARD_BREAK=${TWO_LOGS_GUARD_BREAK} 是弄坏开关，只在自证（two-logs-guard.sh --selftest 与它起的子进程）里认；平常跑设了就拒，这一条命令没判、不执行" >&2
    echo "  → 怎么办：钩子的环境是 Claude Code 进程的环境，Bash 命令里 unset 改不到它：退出这个会话，在起 claude 的 shell 里 unset TWO_LOGS_GUARD_BREAK（settings 的 env 里写了的删掉）再起；要看开关红在哪一格，跑 TWO_LOGS_GUARD_BREAK=<项> bash .claude/hooks/two-logs-guard.sh --selftest（开关写在文件头）" >&2
    exit 2
  fi
fi
HOOK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
JUDGE="$HOOK_DIR/lib_two_logs_guard.py"
# 照「放行但必记检出」：入口自己往检出记录记一行（结局只记；AGENT_HOOK_DETECTIONS 设了记那里，没设取同目录 lib_work_root.py 的默认，
#   两样都取不到或写不进去时 stderr 另说一句没记上）。$1 是卡在哪；标准输入是 hook 的 JSON（读不出时会话号写「读不到」）
record_unjudged() {
  python3 /dev/fd/3 "$HOOK_DIR" two-logs-guard.sh "$1" 3<<'PY' || echo "  ! two-logs-guard.sh：这一条检出没记上（python3 退 $?）" >&2
import json, os, sys
from datetime import datetime, timezone
sys.dont_write_bytecode = True
hook_dir, hook_name, problem = sys.argv[1:4]
try:
    known = json.loads(sys.stdin.buffer.read())
except Exception:
    known = None
known = known if isinstance(known, dict) else {}
tool_input = known.get("tool_input") if isinstance(known.get("tool_input"), dict) else {}
command = tool_input.get("command") or ""
try:
    path = os.environ.get("AGENT_HOOK_DETECTIONS")
    if not path:
        sys.path.insert(0, hook_dir)
        import lib_work_root
        path = lib_work_root.default_detections_path()
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "a", encoding="utf-8") as handle:
        handle.write(json.dumps({"time": datetime.now(timezone.utc).isoformat(), "hook": hook_name, "session_id": known.get("session_id") or "读不到",
                                 "agent_id": known.get("agent_id"), "agent_type": known.get("agent_type") or "主 agent",
                                 "transcript_path": known.get("transcript_path"), "command": (command if isinstance(command, str) else repr(command))[:2000],
                                 "findings": [f"{hook_name} {problem}"], "outcome": "只记"}, ensure_ascii=False) + "\n")
except Exception as error:
    print(f"  ! {hook_name}：这一条检出没记上（{error!r}）", file=sys.stderr)
PY
}
if [[ ! -f "$JUDGE" ]]; then
  echo "  ! two-logs-guard.sh 读不到判法 $JUDGE，这条命令照常执行（日志写法没判）" >&2
  if [[ "${1:-}" == "--selftest" ]]; then
    echo "  ✗ 自检：判法文件 $JUDGE 不在" >&2
    echo "  → 怎么办：把 lib_two_logs_guard.py 放回 .claude/hooks/，再跑 bash .claude/hooks/two-logs-guard.sh --selftest" >&2
    exit 1
  fi
  # 弄坏开关 failure-unrecorded 下只打 stderr（改前的样子）
  if [[ ",${TWO_LOGS_GUARD_BREAK:-}," != *",failure-unrecorded,"* ]]; then
    record_unjudged "读不到判法 $JUDGE，这条命令照常执行（日志写法没判）"
  fi
  exit 0
fi
if [[ "${1:-}" == "--selftest" ]]; then
  exec python3 "$JUDGE" "$HOOK_DIR" "$@"
fi
# 判法文件在而没跑完（导入就出错、被信号停掉：退 0 与 2 以外的码）：照常放行，stderr 报一句、记一条检出。
#   hook 的 JSON 先读进变量（命令替换只去掉末尾的换行），判法与记检出各读一遍；弄坏开关 judge-crash-unrecorded 下只打 stderr、不记（改前的样子：exec 判法，退几都照常放行）
hook_input_text="$(cat)"
printf '%s' "$hook_input_text" | python3 "$JUDGE" "$HOOK_DIR" "$@"
judge_status=${PIPESTATUS[1]}
if (( judge_status == 0 || judge_status == 2 )); then
  exit "$judge_status"
fi
echo "  ! two-logs-guard.sh 的判法 $JUDGE 没跑完（退 $judge_status），这条命令照常执行（日志写法没判）" >&2
if [[ ",${TWO_LOGS_GUARD_BREAK:-}," != *",judge-crash-unrecorded,"* ]]; then
  printf '%s' "$hook_input_text" | record_unjudged "判法 $JUDGE 没跑完（退 $judge_status），这条命令照常执行（日志写法没判）"
fi
exit 0
