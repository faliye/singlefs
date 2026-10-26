#!/usr/bin/env bash
# admission: always Claude Code 每一次调用相关工具都要现判这一次的工具名与命令文本，上一次的结论不替这一次作保
# run-condition: command python3
# PreToolUse hook（CronCreate、ScheduleWakeup、RemoteTrigger、Skill、Monitor、Bash）：本工程设定时只许 scripts/agents/remind.sh 这一个办法，别的在执行前拒绝（退出 2，stderr 写原因与出路）。
# 用户 2026-10-14 定：「禁止其他的设置时间的防止，仅仅允许这一个」「这是本工程的定时脚本 放到本工程来」。主 agent 与子 agent 一样拒。
# hook-events: PreToolUse:CronCreate PreToolUse:ScheduleWakeup PreToolUse:RemoteTrigger PreToolUse:Skill PreToolUse:Monitor PreToolUse:Bash
# gate-similar: bash-command-detector.sh 同挂 PreToolUse[Bash]、同用 lib_shell_words.py 切词，它的 ② 拒前台没超时的等待循环、记 run_in_background 里的等待循环；这里判的是「循环或单条 sleep 之外只剩 echo 之类的闹钟写法」，并且还要管 Monitor 与五个非 Bash 的工具，那个文件只挂 Bash、已有 3000 多行，并进去要给它加工具挂载、改它的 matcher 与自证，两边的判据（等条件成立 / 等时间到）也没有一条相同
# gate-similar: heavy-test-guard.sh 同挂 PreToolUse[Bash]、同用 lib_shell_words.py；它只判重型测试与内存包装，与定时无关
# gate-similar: host-resources-reminder.sh 同挂 PreToolUse[Bash]；它认重活出资源提醒从不拒，这里拒定时的别的办法
# gate-similar: remind-rearm-stop-guard.sh 同属「只许 remind.sh 定时」，但挂 Stop 管 remind.sh 登记的循环提醒到点之后重起；事件和输入都不同
#
# 拒的（每一种在自证里各有一个样本）：
#   工具层：CronCreate、ScheduleWakeup、RemoteTrigger；Skill 工具点名的技能是 loop 或 schedule（带不带插件前缀都认，如 `x:loop`）。CronList、CronDelete 只读或收尾，不拒。
#   命令层（Bash 与 Monitor 的 command）：命令位置上的命令除了 sleep 只剩 echo / printf / date / true / : / 循环关键字 / wait / 响铃类（notify-send、tput、paplay），
#     且 sleep 的总时长不短于 TIMER_MINIMUM_SECONDS 秒（时长是变量、算式、infinity 这类算不出的按长算），或写在 `while true` / `while :` / `until false` / `for ((;;))` 的无限循环里（不论时长）：
#     例 `sleep 1500; echo "wake"`、`while true; do sleep N; echo …; done`、`timeout 3600 bash -c 'until false; do sleep 60; done; echo x'`、
#     `until [ "$(date +%H%M)" = 1530 ]; do sleep 30; done; echo wake`（`[` / `test` 的参数里有 date 或 SECONDS 才算时间比较）。
#     crontab（除 -l 之外）、at、batch、带 --on-* 或 OnCalendar= / OnActiveSec= 属性的 systemd-run、对 *.timer 的 systemctl start / enable / restart。
# 不拒的：sleep 之外还有别的真活（`until ! kill -0 <pid>; do sleep 20; done`、`sleep 600; cat log`、`until grep -q DONE log; do sleep 30; done`、`until [ -f 旗标 ]; do …`）、
#   不到 TIMER_MINIMUM_SECONDS 的有限 sleep、`proc.py wait`、看门狗 watch.sh、`detached-run.sh watch`、Monitor 盯日志（`tail -f … | grep`）、remind.sh 自己的命令、
#   引号里和 heredoc 正文里当数据写的、脚本文件里的 sleep（`bash x.sh` 起的脚本不读进去）。
# 判不到的：变量里拼出来的命令、eval、python 的 time.sleep 加 print、脚本文件里的 sleep、MCP 工具提供的定时器。
# 判不了的照「放行但必记检出」：输入不是 JSON 对象（解不开、顶层不是对象）、读不到 lib_shell_words.py、Bash / Monitor 的 command 不是字符串、命令切不开、
#   判的时候抛异常，照常退 0，stderr 写一行卡在哪，检出记录（AGENT_HOOK_DETECTIONS，没设取同目录 lib_work_root.py 的 default_detections_path，
#   写法在它的 record_unjudged_pass）追加一行、结局「只记」，findings 以「定时闸没判、照常放行」起头；读不到 lib_work_root.py 时 stderr 另说一句没记成。
# 退役判据：remind.sh 不再是本工程唯一的定时办法，或 Claude Code 不再提供这几个工具时删掉。
#
#   timer-only-remind-guard.sh --selftest   在内存里判造出来的 hook JSON，再起一次进程核退出码与 stderr；末尾逐个打开弄坏开关核它红在点名的格
# 弄坏开关 TIMER_ONLY_REMIND_GUARD_BREAK=<项>（只给证红用，逗号连几个；平常跑时不认）。项名、弄坏哪一处、打开后红在哪一格（格名是「✗ 自证 <格名>：」里的那一段）：
#   allow-cron-tools     CronCreate、ScheduleWakeup、RemoteTrigger 放行                     tools
#   allow-skills         loop、schedule 技能放行                                           skills
#   ignore-monitor       Monitor 的命令不判                                                monitor
#   allow-sleep-echo     sleep 加 echo 的单条闹钟放行                                      sleep-echo
#   allow-infinite-loop  无限循环里的 sleep 放行                                           sleep-loop
#   allow-date-test      按时刻比较的 until 放行                                           date-loop
#   allow-system-timers  crontab、at、systemd-run 的定时属性放行                           system-timers
#   refuse-wait-pid      有 sleep 就拒（连等进程结束的循环也拒）                             wait-pid
#   refuse-real-work     长 sleep 后面接真活也拒                                           sleep-then-work
#   refuse-short         不到阈值的有限 sleep 也拒                                          short-sleep
#   refuse-remind        连 remind.sh 自己的命令也拒                                       remind-start
#   refuse-log-watch     Monitor 盯日志的命令也拒                                          log-watch
#   no-next-step         拒绝信息里不写出路                                                next-step
#   input-unrecorded     输入不是 JSON 对象时只打 stderr、不记检出（改前连 stderr 也不打）   input-recorded
#   library-unrecorded   读不到 lib_shell_words.py 时只打 stderr、不记检出（同上）          library-recorded
#   command-shape-unrecorded  command 不是字符串时只打 stderr、不记检出（同上）            command-shape-recorded
#   split-unrecorded     命令切不开时只打 stderr、不记检出（同上）                         split-recorded
#   exception-unrecorded 判的时候抛异常不接住、照原样抛出（改前的样子：进程退 1，照常放行）   exception-recorded
set -uo pipefail
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
# python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import importlib.util, json, math, os, re, shutil, subprocess, sys, tempfile

BREAK_VARIABLE = "TIMER_ONLY_REMIND_GUARD_BREAK"
HOOK_NAME = "timer-only-remind-guard.sh"
UNJUDGED_PREFIX = "定时闸没判、照常放行"
TIMER_MINIMUM_SECONDS = 60
REFUSED_TOOLS = ("CronCreate", "ScheduleWakeup", "RemoteTrigger")
REFUSED_SKILLS = ("loop", "schedule")
COMMAND_TOOLS = ("Bash", "Monitor")
# 闹钟里陪着 sleep 的角色：打一行、响一下、读时钟、循环的骨架
FILLER_COMMANDS = {"echo", "printf", "date", "true", "false", ":", "wait", "notify-send", "tput", "paplay", "exit",
                   "done", "for", "while", "until", "do", "then", "else", "elif", "fi", "if", "esac", "case", "select", "in", "{", "}", "!", "(", ")"}
SLEEP_ARGUMENT = re.compile(r"^(\d+(?:\.\d*)?|\.\d+)([smhd]?)$")
UNIT_SECONDS = {"": 1, "s": 1, "m": 60, "h": 3600, "d": 86400}
INFINITE_LOOP = re.compile(r"\bwhile\s+(?:true|:)\s*(?:;|\n|\bdo\b)|\buntil\s+false\s*(?:;|\n|\bdo\b)|\bfor\s*\(\(\s*;\s*;\s*\)\)")
SYSTEMD_TIMER_PROPERTY = re.compile(r"^(?:--on-[a-z-]+(?:=.*)?|(?:-p|--property)=?.*On(?:Calendar|ActiveSec|BootSec|StartupSec|UnitActiveSec|UnitInactiveSec)=.*|On(?:Calendar|ActiveSec|BootSec|StartupSec|UnitActiveSec|UnitInactiveSec)=.*)$")

def broken(item):
    return item in os.environ.get(BREAK_VARIABLE, "").split(",")

def load_shell_words(hook_dir):
    spec = importlib.util.spec_from_file_location("lib_shell_words", os.path.join(hook_dir, "lib_shell_words.py"))
    library = importlib.util.module_from_spec(spec)
    sys.dont_write_bytecode = True
    spec.loader.exec_module(library)
    return library

def sleep_seconds(arguments):
    """sleep 的参数合起来多少秒；算不出的（变量、算式、infinity）按无限长算。"""
    total = 0.0
    for argument in arguments:
        if argument.startswith("-"):
            continue
        matched = SLEEP_ARGUMENT.match(argument)
        if not matched:
            return math.inf
        total += float(matched.group(1)) * UNIT_SECONDS[matched.group(2)]
    return total

def is_time_comparison(words):
    return os.path.basename(words[0]) in ("[", "[[", "test") and any("date" in word or "SECONDS" in word for word in words[1:]) and not broken("allow-date-test")

def system_timer(words):
    """crontab（除 -l 外）、at、batch、systemd-run 的定时属性、对 *.timer 的 systemctl：这一条是不是在设定时。"""
    if broken("allow-system-timers"):
        return None
    name, arguments = os.path.basename(words[0]), words[1:]
    if name in ("at", "batch"):
        return name
    if name == "crontab" and "-l" not in arguments:
        return "crontab"
    if name == "systemd-run" and any(SYSTEMD_TIMER_PROPERTY.match(argument) for argument in arguments):
        return "systemd-run 的定时属性"
    if name == "systemctl" and any(argument.endswith(".timer") for argument in arguments) and any(argument in ("start", "enable", "restart", "--now") for argument in arguments):
        return "systemctl 起 .timer 单元"
    return None

def timer_in_command(shell_words, text):
    """命令文本是不是只为定时：交出一句话说它是什么，不是就交 None。"""
    scan = shell_words.commands_at_command_position(text)
    sleeps, compares_clock, real_work = [], False, False
    for command in scan.commands:
        timer = system_timer(command.words)
        if timer:
            return f"用 {timer} 设定时"
        name = os.path.basename(command.words[0])
        if name == "sleep":
            sleeps.append(sleep_seconds(command.words[1:]))
        elif name in FILLER_COMMANDS:
            continue
        elif is_time_comparison(command.words):
            compares_clock = True
        else:
            real_work = True
    if not sleeps:
        return None
    if real_work:
        # sleep 之外还有真活（等进程、等文件、等日志里一行字、睡醒之后看日志）：不是只为定时
        if broken("refuse-wait-pid") and kill_probe_loop(scan.commands):
            return "等进程结束的循环"
        if broken("refuse-real-work") and sum(sleeps) >= TIMER_MINIMUM_SECONDS:
            return "长 sleep 之后接别的命令"
        return None
    infinite = INFINITE_LOOP.search(text) is not None and not broken("allow-infinite-loop")
    long_enough = sum(sleeps) >= TIMER_MINIMUM_SECONDS
    if infinite:
        return "无限循环里 sleep、除此之外只打字或响铃"
    if compares_clock:
        return "按时刻比较的循环里 sleep"
    if long_enough and not broken("allow-sleep-echo"):
        return f"sleep 满 {TIMER_MINIMUM_SECONDS} 秒以上、除此之外只打字或响铃"
    if broken("refuse-short"):
        return "有限的短 sleep"
    return None

def kill_probe_loop(commands):
    return any(os.path.basename(command.words[0]) == "kill" and "-0" in command.words[1:] for command in commands)

def skill_name(tool_input):
    value = str(tool_input.get("skill") or tool_input.get("name") or "").strip().lstrip("/")
    return value.rsplit(":", 1)[-1]

def refusal_reason(shell_words, hook_input, unjudged=None):
    """要拒就交一句话（卡在哪），放行交 None。command 不是字符串、命令切不开（判不了）的放行，给了 unjudged（列表）的往里记 (弄坏开关名, 一句)。"""
    tool = hook_input.get("tool_name")
    tool_input = hook_input.get("tool_input") or {}
    if tool in REFUSED_TOOLS and not broken("allow-cron-tools"):
        return f"工具 {tool} 是别的定时办法"
    if tool == "Skill" and skill_name(tool_input) in REFUSED_SKILLS and not broken("allow-skills"):
        return f"技能 {skill_name(tool_input)} 是别的定时办法"
    if tool in COMMAND_TOOLS and not isinstance(tool_input.get("command"), str) and unjudged is not None:
        unjudged.append(("command-shape-unrecorded", f"{tool} 的 tool_input.command 不是字符串（{type(tool_input.get('command')).__name__}），是不是定时的别的写法没判"))
    if tool in COMMAND_TOOLS and isinstance(tool_input.get("command"), str):
        if tool == "Monitor" and broken("ignore-monitor"):
            return None
        if broken("refuse-remind") and "remind.sh" in tool_input["command"]:
            return "remind.sh 自己的命令"
        if broken("refuse-log-watch") and tool == "Monitor" and "tail" in tool_input["command"]:
            return "Monitor 盯日志"
        try:
            kind = timer_in_command(shell_words, tool_input["command"])
        except Exception as error:
            if unjudged is not None:
                unjudged.append(("split-unrecorded", f"{tool} 的命令切不开（{type(error).__name__}: {error}），是不是定时的别的写法没判"))
            return None
        if kind:
            return f"{tool} 的命令是定时的别的写法（{kind}）"
    return None

def refusal_text(reason):
    lines = [f"✗ 本工程设定时只许 scripts/agents/remind.sh，这一次被拒：{reason}"]
    if not broken("no-next-step"):
        lines.append("→ 怎么办：定时提醒只用 bash scripts/agents/remind.sh start --every <分钟> --message <提醒内容> [--name <名字>] [--once]，用 Bash 的 run_in_background 起；循环 / --once / stop / list 的写法与到点之后做什么看 .claude/skills/remind/SKILL.md")
        lines.append("→ 不是定时的写法不受影响：等一个进程结束用 `until ! kill -0 <pid>; do sleep 20; done` 或 `python3 .claude/singlefs-ai-sop/scripts/proc.py wait <pid> --timeout <秒>`；盯日志用 Monitor 跑 `tail -f <日志> | grep --line-buffered <模式>`；等后台任务结束本轮等完成通知")
    return "\n".join(lines)

def record_unjudged(hook_dir, hook_input, problem, switch):
    """判不了、照常放行的这一次：stderr 写一行，照「放行但必记检出」往检出记录追加一行（lib_work_root.py 的 record_unjudged_pass，结局只记）。
    弄坏开关 switch 下只打 stderr、不记（改前是连 stderr 也不打）；读不到 lib_work_root.py、写不进去时 stderr 说一句没记成。"""
    print(f"! {HOOK_NAME}：{problem}，这一次没判、照常放行", file=sys.stderr)
    if broken(switch):
        return
    tool_input = hook_input.get("tool_input") if isinstance(hook_input, dict) and isinstance(hook_input.get("tool_input"), dict) else {}
    tool = hook_input.get("tool_name") if isinstance(hook_input, dict) else None
    try:
        sys.dont_write_bytecode = True
        if hook_dir not in sys.path:
            sys.path.insert(0, hook_dir)
        import lib_work_root
        _, said = lib_work_root.record_unjudged_pass(HOOK_NAME, hook_input, f"{tool or '?'}：{tool_input.get('command') or tool_input.get('skill') or ''}",
                                                     [f"{UNJUDGED_PREFIX}：{problem}"])
    except Exception as error:   # 文件不在、语法错都算读不到
        said = f"检出没记成：读不到同目录的 lib_work_root.py（{error!r}）"
    print(f"  {said}", file=sys.stderr)

def hook_main(hook_dir):
    return judge_raw(sys.stdin.buffer.read(), hook_dir)

def judge_raw(raw, hook_dir):
    """hook 的原始输入（字节）→ 退出码；拒的往 stderr 写原因与出路，判不了的照「放行但必记检出」（文件头「判不了的」一段）。"""
    try:
        hook_input = json.loads(raw.decode("utf-8"))
        problem = None if isinstance(hook_input, dict) else f"输入顶层是 {type(hook_input).__name__}，不是 JSON 对象"
    except ValueError as error:
        hook_input, problem = None, f"输入不是合法的 JSON（{error}；前 200 字节：{raw[:200].decode('utf-8', 'replace')!r}）"
    if problem:
        record_unjudged(hook_dir, None, problem, "input-unrecorded")
        return 0
    try:
        shell_words = load_shell_words(hook_dir)
    except Exception as error:
        record_unjudged(hook_dir, hook_input, f"读不到同目录的 lib_shell_words.py（{error!r}）", "library-unrecorded")
        return 0
    unjudged = []
    try:
        reason = refusal_reason(shell_words, hook_input, unjudged)
    except Exception as error:
        if broken("exception-unrecorded"):   # 弄坏开关：不接住，照原样抛出（改前的样子：进程退 1，照常放行、不记检出）
            raise
        record_unjudged(hook_dir, hook_input, f"判的时候抛异常（{type(error).__name__}: {error}）", "exception-unrecorded")
        return 0
    for switch, problem in unjudged:
        record_unjudged(hook_dir, hook_input, problem, switch)
    if reason is None:
        return 0
    print(refusal_text(reason), file=sys.stderr)
    return 2

# ── 自证：每一格造答案已知的输入。红样本必须被拒（交出 reason），绿样本必须放行
def bash(command, **extra):
    return dict({"tool_name": "Bash", "tool_input": {"command": command}}, **extra)

def monitor(command):
    return {"tool_name": "Monitor", "tool_input": {"command": command, "description": "试跑", "timeout_ms": 60000}}

def tool(name, **tool_input):
    return {"tool_name": name, "tool_input": tool_input}

CELLS = [
    # (格名, 该拒的样本, 该放行的样本)；样本是 (标签, hook 输入)
    ("tools", [("CronCreate", tool("CronCreate", cron="*/30 * * * *", prompt="x")), ("ScheduleWakeup", tool("ScheduleWakeup", delaySeconds=1500, prompt="x")),
               ("RemoteTrigger", tool("RemoteTrigger", action="create")), ("子 agent 同样拒", dict(tool("CronCreate", prompt="x"), agent_type="implementation-writer"))],
     [("CronList", tool("CronList")), ("CronDelete", tool("CronDelete", id="a")), ("Read", tool("Read", file_path="/x"))]),
    ("skills", [("loop", tool("Skill", skill="loop", args="5m check")), ("schedule", tool("Skill", skill="schedule")), ("带插件前缀", tool("Skill", skill="plugin:loop")),
                ("带斜杠", tool("Skill", skill="/schedule"))],
     [("gate", tool("Skill", skill="gate")), ("remind", tool("Skill", skill="remind"))]),
    ("monitor", [("Monitor 里 sleep 加 echo", monitor('sleep 1500; echo "wake: 半小时到了"'))], []),
    ("sleep-echo", [("sleep 1500; echo", bash('sleep 1500; echo "wake"')), ("run_in_background 的 sleep 25m && echo", bash("sleep 25m && echo ding", run_in_background=True)),
                    ("sleep 变量", bash('sleep "$N"; echo done')), ("sleep 加 printf", bash("sleep 3600; printf 'alarm\\n'")),
                    ("timeout 套 bash -c", bash("timeout 7200 bash -c 'sleep 1800; echo x'")), ("括号子 shell 放后台", bash("(sleep 90; echo x) &"))], []),
    ("sleep-loop", [("while true", bash("while true; do sleep 600; echo tick; done")), ("while :", bash("while :; do sleep 5; echo tick; done")),
                    ("until false 套 timeout", bash("timeout 3600 bash -c 'until false; do sleep 60; done; echo x'")), ("for 空条件", bash("for ((;;)); do sleep 9; echo t; done")),
                    ("Monitor 里 while true", monitor("while true; do sleep 300; echo tick; done"))], []),
    ("date-loop", [("until 比时刻", bash('until [ "$(date +%H%M)" = 1530 ]; do sleep 30; done; echo wake'))], []),
    ("system-timers", [("crontab -e", bash("crontab -e")), ("管道进 crontab", bash("echo '*/5 * * * * x' | crontab -")), ("at", bash("echo x | at now + 1 minute")),
                       ("systemd-run --on-active", bash("systemd-run --user --on-active=30m echo hi")), ("systemd-run -p OnCalendar", bash("systemd-run --user -p OnCalendar=hourly echo hi")),
                       ("systemctl start .timer", bash("systemctl --user start x.timer"))],
     [("crontab -l", bash("crontab -l")), ("systemd-run 起服务", bash("systemd-run --user --unit x -- sleep 5")), ("list-timers", bash("systemctl --user list-timers"))]),
    ("wait-pid", [], [("等进程结束", bash("until ! kill -0 12345 2>/dev/null; do sleep 20; done")), ("proc.py wait", bash("python3 .claude/singlefs-ai-sop/scripts/proc.py wait 123 --timeout 600")),
                      ("while kill -0", bash("while kill -0 12345; do sleep 5; done; echo 结束"))]),
    ("sleep-then-work", [], [("长 sleep 后看日志", bash("sleep 1500; cat /tmp/out.log")), ("等文件出现", bash("until [ -f /tmp/flag ]; do sleep 120; done")),
                             ("等日志里一行字", bash("timeout 3600 bash -c 'until grep -q DONE /tmp/x.log; do sleep 30; done; echo finished'"))]),
    ("short-sleep", [], [("sleep 2 && echo", bash("sleep 2 && echo ok")), ("sleep 30; echo", bash("sleep 30; echo ok")), ("sleep 0.5", bash("sleep 0.5; ls"))]),
    ("remind-start", [], [("remind.sh start", bash('bash scripts/agents/remind.sh start --every 30 --message "sleep 1500; echo wake" --name a', run_in_background=True)),
                          ("remind.sh stop", bash("bash scripts/agents/remind.sh stop --name a"))]),
    ("log-watch", [], [("tail -f 管 grep", monitor("tail -f /tmp/x.log | grep --line-buffered ERROR")), ("看门狗", bash("bash scripts/agents/watch.sh", run_in_background=True)),
                       ("detached-run watch", bash("bash scripts/hosts/detached-run.sh watch", run_in_background=True)), ("缓存计时脚本", bash("bash scripts/agents/cache-keepalive.sh 230", run_in_background=True))]),
    ("data-not-command", [], [("echo 里的字", bash('echo "sleep 1500; echo wake"')), ("git 提交说明", bash('git commit -m "sleep 1500; echo wake"')),
                              ("heredoc 数据", bash("cat > /tmp/x.txt <<'EOF'\nsleep 1500; echo wake\nEOF")), ("grep 的参数", bash("grep -rn 'sleep 1500' scripts")),
                              ("普通命令", bash("ls -l && git status --short")), ("坏输入没有 command", {"tool_name": "Bash", "tool_input": {}})]),
]
BREAK_TABLE = [("allow-cron-tools", "tools"), ("allow-skills", "skills"), ("ignore-monitor", "monitor"), ("allow-sleep-echo", "sleep-echo"),
               ("allow-infinite-loop", "sleep-loop"), ("allow-date-test", "date-loop"), ("allow-system-timers", "system-timers"),
               ("refuse-wait-pid", "wait-pid"), ("refuse-real-work", "sleep-then-work"), ("refuse-short", "short-sleep"),
               ("refuse-remind", "remind-start"), ("refuse-log-watch", "log-watch"), ("no-next-step", "next-step"),
               ("input-unrecorded", "input-recorded"), ("library-unrecorded", "library-recorded"), ("command-shape-unrecorded", "command-shape-recorded"),
               ("split-unrecorded", "split-recorded"), ("exception-unrecorded", "exception-recorded")]

def selftest_cells(hook_dir):
    shell_words = load_shell_words(hook_dir)
    failures, checked = [], 0
    def fail(cell, detail):
        failures.append(cell)
        print(f"✗ 自证 {cell}：{detail}")
        print(f"→ 怎么办：看 timer-only-remind-guard.sh 里 {cell} 对应的判定（refusal_reason、timer_in_command、system_timer）；{BREAK_VARIABLE} 设着的话这里本来就该红")
    for cell, red_samples, green_samples in CELLS:
        for label, hook_input in red_samples:
            checked += 1
            if refusal_reason(shell_words, hook_input) is None:
                fail(cell, f"「{label}」应当被拒，实际放行了")
        for label, hook_input in green_samples:
            checked += 1
            reason = refusal_reason(shell_words, hook_input)
            if reason is not None:
                fail(cell, f"「{label}」应当放行，实际被拒：{reason}")
    # 进程层：红样本退 2、stderr 带 remind.sh 与 SKILL.md 的出路，绿样本退 0 且 stderr 空
    hook = os.path.join(hook_dir, "timer-only-remind-guard.sh")
    scratch = tempfile.mkdtemp(prefix="timer-only-remind-guard-selftest-")
    detections = os.path.join(scratch, "detections.jsonl")   # 检出记在自证自己的临时目录，不碰真的检出记录
    environment = dict({key: value for key, value in os.environ.items() if key != "SINGLEFS_WORK_ROOT_BREAK"}, AGENT_HOOK_DETECTIONS=detections)
    for label, payload, want_code in (("CronCreate", tool("CronCreate", prompt="x"), 2), ("等进程结束", bash("until ! kill -0 1; do sleep 20; done"), 0)):
        checked += 1
        completed = subprocess.run(["bash", hook], input=json.dumps(payload), capture_output=True, text=True, env=environment)
        if completed.returncode != want_code:
            fail("process", f"「{label}」应当退 {want_code}，实际退 {completed.returncode}，stderr：{completed.stderr!r}")
        elif want_code == 2:
            checked += 1
            if "bash scripts/agents/remind.sh start" not in completed.stderr or ".claude/skills/remind/SKILL.md" not in completed.stderr:
                fail("next-step", f"拒绝信息里应当写出 remind.sh start 的写法与 SKILL.md，实际 stderr：{completed.stderr!r}")
        elif completed.stderr:
            fail("process", f"「{label}」放行时 stderr 应当是空的，实际：{completed.stderr!r}")

    def recorded():
        try:
            with open(detections, encoding="utf-8") as handle:
                return [json.loads(line) for line in handle if line.strip()]
        except OSError:
            return []

    def expect_recorded(cell, run, want, session):
        """run() 起一次（钩子入口或进程里的 judge_raw）交 (退出码, stderr)；要退 0、stderr 写一行、检出记录多 want 行（结局只记、会话号 session）。"""
        nonlocal checked
        checked += 1
        before = len(recorded())
        try:
            code, stderr = run()
        except Exception as error:   # 弄坏开关 exception-unrecorded：进程里照原样抛出
            code, stderr = f"抛出 {type(error).__name__}", ""
        added = recorded()[before:]
        if not (code == 0 and HOOK_NAME in stderr and len(added) == want and all(
                entry.get("outcome") == "只记" and entry.get("session_id") == session and entry.get("hook") == HOOK_NAME
                and all(str(finding).startswith(UNJUDGED_PREFIX) for finding in entry.get("findings") or []) for entry in added)):
            fail(cell, f"应当退 0、stderr 写一行、检出记录多 {want} 行（结局只记、会话号 {session}）；实际退 {code}，stderr {stderr.strip()[:200]!r}，检出多了 {len(added)} 行：{added!r:.300}")

    def through_entry(script, raw):
        def run():
            completed = subprocess.run(["bash", script], input=raw, capture_output=True, text=True, env=environment)
            return completed.returncode, completed.stderr
        return run

    def in_process(raw, replaced=None):
        """在进程里照钩子入口调 judge_raw；replaced 给了的把那几个函数临时换掉（造切不开、抛异常）。stderr 收进字符串。"""
        def run():
            import io
            saved = {name: globals()[name] for name in (replaced or {})}
            saved_stderr, saved_detections = sys.stderr, os.environ.get("AGENT_HOOK_DETECTIONS")
            globals().update(replaced or {})
            sys.stderr, os.environ["AGENT_HOOK_DETECTIONS"] = io.StringIO(), detections
            try:
                code = judge_raw(raw.encode("utf-8"), hook_dir)
                return code, sys.stderr.getvalue()
            finally:
                captured, sys.stderr = sys.stderr, saved_stderr
                globals().update(saved)
                if saved_detections is None:
                    os.environ.pop("AGENT_HOOK_DETECTIONS", None)
                else:
                    os.environ["AGENT_HOOK_DETECTIONS"] = saved_detections
        return run

    def raising(*arguments, **keywords):
        raise RuntimeError("自证造的异常")

    # 判不了的照「放行但必记检出」
    expect_recorded("input-recorded", through_entry(hook, "不是 JSON"), 1, "读不到")
    expect_recorded("input-recorded", through_entry(hook, "[]"), 1, "读不到")
    bare = os.path.join(scratch, "bare")   # 只拷入口与 lib_work_root.py：读不到 lib_shell_words.py
    os.makedirs(bare)
    for name in ("timer-only-remind-guard.sh", "lib_work_root.py"):
        shutil.copy(os.path.join(hook_dir, name), bare)
    expect_recorded("library-recorded", through_entry(os.path.join(bare, "timer-only-remind-guard.sh"),
                                                      json.dumps(dict(bash("sleep 1500; echo wake"), session_id="s-bare"))), 1, "s-bare")
    expect_recorded("command-shape-recorded", through_entry(hook, json.dumps({"tool_name": "Bash", "session_id": "s-shape", "tool_input": {"command": ["sleep", "1500"]}})),
                    1, "s-shape")
    expect_recorded("split-recorded", in_process(json.dumps(dict(bash("sleep 1500; echo wake"), session_id="s-split")), {"timer_in_command": raising}), 1, "s-split")
    expect_recorded("exception-recorded", in_process(json.dumps(dict(tool("CronList"), session_id="s-raise")), {"REFUSED_TOOLS": None}), 1, "s-raise")   # tool in None 抛 TypeError
    shutil.rmtree(scratch, ignore_errors=True)
    return failures, checked

def main():
    hook_dir = sys.argv[1]
    if sys.argv[2:] == ["--selftest-cells"]:
        failures, checked = selftest_cells(hook_dir)
        if failures:
            print(f"✗ 自证：{len(failures)} 处红（共判了 {checked} 个样本）")
            print("→ 怎么办：照上面每一格的下一步改，再单独跑 timer-only-remind-guard.sh --selftest")
            return 1
        print(f"✓ 各格都对（判了 {checked} 个样本）")
        return 0
    if sys.argv[2:] == ["--selftest"]:
        hook = os.path.join(hook_dir, "timer-only-remind-guard.sh")
        clean = {key: value for key, value in os.environ.items() if key != BREAK_VARIABLE}
        completed = subprocess.run(["bash", hook, "--selftest-cells"], capture_output=True, text=True, env=clean)
        if completed.returncode != 0:
            print("✗ 自证：不带弄坏开关时各格应当全绿，实际：")
            print(completed.stdout + completed.stderr)
            print("→ 怎么办：先让 --selftest-cells 不带弄坏开关跑绿，再回来跑本自证")
            return 1
        sample_line = completed.stdout.strip().splitlines()[-1]
        failed = 0
        for switch, cell in BREAK_TABLE:
            broken_run = subprocess.run(["bash", hook, "--selftest-cells"], capture_output=True, text=True, env=dict(clean, **{BREAK_VARIABLE: switch}))
            if broken_run.returncode == 0 or f"✗ 自证 {cell}：" not in broken_run.stdout:
                failed += 1
                print(f"✗ 自证：打开弄坏开关 {switch} 之后应当红在「{cell}」格，实际退 {broken_run.returncode}，输出：{broken_run.stdout!r}")
                print(f"→ 怎么办：{switch} 弄坏的那一处要真影响「{cell}」格的判定；两边对不上就改开关或改格")
        if failed:
            return 1
        print(f"✓ timer-only-remind-guard.sh 自证：{sample_line.removeprefix('✓ ')}；{len(BREAK_TABLE)} 个弄坏开关各红在点名的格")
        return 0
    return hook_main(hook_dir)

sys.exit(main())
PY
