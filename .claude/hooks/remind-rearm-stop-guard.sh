#!/usr/bin/env bash
# admission: always Claude Code 每一次结束一轮都要现看这个会话此刻登记着的提醒与等着的进程，上一次的结论不替这一次作保
# run-condition: command python3
# Stop hook：主 agent 要结束这一轮时，看这个会话登记着的循环提醒（scripts/agents/remind.sh start 不带 --once 登记的）有没有进程在等；
# 有登记、没在等的，拦下结束（decision=block），把登记里照原样重起的命令（restart= 那一行）交给主 agent，让它用 Bash 的 run_in_background 重起、再结束这一轮。
# 这就是「提醒到点之后自启动」那一步（用户 2026-10-14 定：「你让脚本定时提醒 你不需要挂了 脚本到时间提醒 然后自启动」「没有 --once 的时候就无限自启动或者被关闭」）。
# hook-events: Stop
# gate-similar: 无 同挂 Stop 事件的只有上游 .claude/singlefs-ai-sop/scripts/claude-hooks/gate-reuse-check.sh（收工时自检新建的门禁与钩子，本仓没注册）；它读 git 里的新文件，这里读提醒的登记目录与 /proc，判的对象不同
# gate-similar: timer-only-remind-guard.sh 同属「只许 remind.sh 定时」这一件事，但挂 PreToolUse 在执行前拒别的定时办法；这里挂 Stop 管 remind.sh 登记的循环提醒到点之后重起，事件和判的对象都不同，并进去要让一个文件同时挂两个事件、读两份互不相干的输入
#
# 登记目录与 remind.sh 同一份：${REMIND_STATE_ROOT:-~/.local/state/singlefs/reminders}/<会话号>/，会话号取 hook 输入的 session_id；只看这个会话的，不碰别的会话的登记。
# 认「在等」：判法与 remind.sh 共用 .claude/hooks/lib_remind_waiter.py——<名字>.pid 一行「<进程号> <启动时刻>」，进程活着、启动时刻与登记相同、
#   命令真是 bash …/remind.sh（解释器之后的脚本参数以 remind.sh 结尾）三样都对才算；进程号被复用、无关进程参数里带 remind.sh、
#   旧登记没有启动时刻的，都按没人在等，交出重起命令。once=1 的登记不管。
# 读不到 lib_remind_waiter.py 时判不了：照常放行（退 0），往 stderr 写一句，往检出记录（AGENT_HOOK_DETECTIONS，没设取同目录 lib_work_root.py 的默认）
#   记一条（结局只记，会话号从 hook 输入里取、读不出写「读不到」）；两样都取不到或写不进去时 stderr 另说一句没记上。
# 已经因为这个钩子拦过一次（hook 输入 stop_hook_active 为真）还没重起的，放行，免得卡死；下一轮结束时照样再拦。
# 不读也不假设提醒周期：只看登记在不在（once=）、进程活没活；自证里写的 every= 与 --every 数字都只是样本值。
# 没有登记目录时什么都不打、退 0。判不了的另几种同样照「放行但必记检出」：输入不是 JSON 对象、认不出会话号（没有、或带路径符号）、登记读不了、判的时候抛异常，
#   照常放行（退 0、不拦），stderr 写一行，检出记录记一条（record_unjudged，同上）。
# 退役判据：循环提醒不再用 remind.sh 登记，或 Claude Code 自己提供到点自启动时删掉。
#
#   remind-rearm-stop-guard.sh --selftest   在临时登记目录里喂造出来的 hook JSON，末尾逐个打开弄坏开关核它红在点名的格
# 弄坏开关 REMIND_REARM_STOP_BREAK=<项>（只给证红用，逗号连几个）。项名、弄坏哪一处、打开后红在哪一格（格名是「✗ 自证 <格名>：」里的那一段）：
#   ignore-once         once=1 的登记也当要重起                 once
#   ignore-waiting      不看有没有进程在等                       waiting
#   trust-pid-file      进程号文件在就当在等（不核进程活着、启动时刻与命令）     stale-pid
#   identity-by-name    认在等退回只看进程活着、命令行任意位置含 remind.sh       unrelated-argument、start-time-mismatch、legacy-registration
#   ignore-active       stop_hook_active 为真时仍然拦             active
#   all-sessions        不按会话号分，读所有会话的登记             other-session
#   first-only          只交出第一条要重起的命令                   several
#   failure-unrecorded  读不到 lib_remind_waiter.py 时只打 stderr、不记检出（改前的样子）     library-unreadable、library-unreadable-default-path
#   input-unrecorded    输入不是 JSON 对象时只打 stderr、不记检出（改前连 stderr 也不打）      bad-input-recorded
#   session-unrecorded  认不出会话号时只打 stderr、不记检出（同上）                           session-unrecognized-recorded
#   settings-unrecorded 登记读不了时只打 stderr、不记检出（同上）                             settings-unreadable-recorded
#   exception-unrecorded  判的时候抛异常不接住、照原样抛出（改前的样子：进程退 1，不拦、不记检出）   exception-recorded
set -uo pipefail
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
# python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import json, os, re, shutil, subprocess, sys, tempfile, time
from datetime import datetime, timezone

BREAK_VARIABLE = "REMIND_REARM_STOP_BREAK"
SESSION_PATTERN = re.compile(r"^[A-Za-z0-9_-]+$")
HOOK_NAME = "remind-rearm-stop-guard.sh"
DETECTIONS_VARIABLE = "AGENT_HOOK_DETECTIONS"

def broken(item):
    return item in os.environ.get(BREAK_VARIABLE, "").split(",")

def state_root():
    return os.environ.get("REMIND_STATE_ROOT") or os.path.expanduser("~/.local/state/singlefs/reminders")

def read_settings(path):
    values = {}
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            key, separator, value = line.rstrip("\n").partition("=")
            if separator:
                values.setdefault(key, value)
    return values

def load_waiter_library(hook_dir):
    """同目录的 lib_remind_waiter.py（与 remind.sh 共用的「在等」判法）。"""
    sys.dont_write_bytecode = True
    if hook_dir not in sys.path:
        sys.path.insert(0, hook_dir)
    import lib_remind_waiter
    return lib_remind_waiter

def read_hook_input():
    """读 hook 输入；读不出或不是 JSON 对象交 {}。"""
    try:
        data = json.loads(sys.stdin.read())
    except (ValueError, OSError):
        return {}
    return data if isinstance(data, dict) else {}

def detections_path(hook_dir):
    """检出记录的位置：AGENT_HOOK_DETECTIONS 设了照它，没设取同目录 lib_work_root.py 的默认；两样都取不到交 None。"""
    configured = os.environ.get(DETECTIONS_VARIABLE)
    if configured:
        return configured
    try:
        sys.dont_write_bytecode = True
        if hook_dir not in sys.path:
            sys.path.insert(0, hook_dir)
        import lib_work_root
        return lib_work_root.default_detections_path()
    except Exception:
        return None

def record_unjudged(hook_dir, hook_input, problem):
    """判不了、照常放行的这一次往检出记录追加一行（结局「只记」，看门狗照这一栏说下一步）；取不到位置或写不进去时往 stderr 说一句。"""
    session = hook_input.get("session_id")
    entry = {"time": datetime.now(timezone.utc).isoformat(), "hook": HOOK_NAME,
             "session_id": session if isinstance(session, str) and session else "读不到",
             "agent_id": hook_input.get("agent_id"), "agent_type": hook_input.get("agent_type") or "主 agent",
             "transcript_path": hook_input.get("transcript_path"), "command": "",
             "findings": [f"循环提醒的停止钩子判不了、照常放行：{problem}"], "outcome": "只记"}
    path = detections_path(hook_dir)
    try:
        if not path:
            raise OSError(f"没设 {DETECTIONS_VARIABLE}，也读不到 lib_work_root.py")
        os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
        with open(path, "a", encoding="utf-8") as handle:
            handle.write(json.dumps(entry, ensure_ascii=False) + "\n")
    except OSError as error:
        print(f"  ! {HOOK_NAME}：这一条检出没记上（{error}）", file=sys.stderr)

waiter_library = None

def is_waiting(directory, name):
    pid_path = os.path.join(directory, name + ".pid")
    if not os.path.isfile(pid_path):
        return False
    if broken("trust-pid-file"):
        return True
    return waiter_library.waiting_pid(directory, name, legacy_string_match=broken("identity-by-name")) is not None

def sessions_to_read(root, session):
    if broken("all-sessions"):
        return [os.path.join(root, entry) for entry in sorted(os.listdir(root))] if os.path.isdir(root) else []
    return [os.path.join(root, session)]

def restart_commands(root, session, unreadable=None):
    """这个会话里登记着、没人在等的循环提醒，各自照原样重起的命令。读不了的登记跳过，给了 unreadable（列表）的逐份往里记一句。"""
    commands = []
    for directory in sessions_to_read(root, session):
        if not os.path.isdir(directory):
            continue
        for file_name in sorted(os.listdir(directory)):
            if not file_name.endswith(".conf"):
                continue
            name = file_name[:-len(".conf")]
            try:
                settings = read_settings(os.path.join(directory, file_name))
            except OSError as error:
                if unreadable is not None:
                    unreadable.append(f"登记 {os.path.join(directory, file_name)} 读不了（{type(error).__name__}: {error}），那一条提醒要不要重起没判")
                continue
            if settings.get("once") == "1" and not broken("ignore-once"):
                continue
            if is_waiting(directory, name) and not broken("ignore-waiting"):
                continue
            if settings.get("restart"):
                commands.append(settings["restart"])
    return commands

def decide(hook_input, unjudged=None):
    """hook 输入 → 要打到标准输出的 JSON 文本（放行时是 None）。判不了的（认不出会话号、登记读不了）给了 unjudged（列表）的往里记 (弄坏开关名, 一句)。"""
    session = str(hook_input.get("session_id") or "")
    if not SESSION_PATTERN.match(session):
        if unjudged is not None:
            unjudged.append(("session-unrecorded", f"认不出会话号（{session[:80]!r}），这个会话登记着的循环提醒有没有进程在等没判"))
        return None
    unreadable = []
    commands = restart_commands(state_root(), session, unreadable)
    if unjudged is not None:
        unjudged += [("settings-unrecorded", text) for text in unreadable]
    if not commands:
        return None
    if hook_input.get("stop_hook_active") is True and not broken("ignore-active"):
        return None
    if broken("first-only"):
        commands = commands[:1]
    reason = "循环提醒没在等（到点提醒过了，要自启动）：用 Bash 的 run_in_background 照原样重起下面每一条，再结束这一轮："
    for command in commands:
        reason += "\n" + command
    reason += "\n要关掉哪条提醒就跑 bash scripts/agents/remind.sh stop --name <名字>（写法见 .claude/skills/remind/SKILL.md）。"
    return json.dumps({"decision": "block", "reason": reason}, ensure_ascii=False)

def hook_main(hook_dir):
    return judge_raw(sys.stdin.buffer.read(), hook_dir)

def judge_raw(raw, hook_dir):
    """hook 的原始输入（字节）→ 退出码（恒 0）；要拦就把 JSON 打到标准输出。判不了的照「放行但必记检出」（文件头那一段），各自的弄坏开关下只打 stderr、不记。"""
    try:
        hook_input = json.loads(raw.decode("utf-8"))
        problem = None if isinstance(hook_input, dict) else f"输入顶层是 {type(hook_input).__name__}，不是 JSON 对象"
    except ValueError as error:
        hook_input, problem = {}, f"输入不是合法的 JSON（{error}；前 200 字节：{raw[:200].decode('utf-8', 'replace')!r}）"
    if problem:
        report_unjudged(hook_dir, {}, problem + "，这个会话登记着的循环提醒有没有进程在等没判", "input-unrecorded")
        return 0
    unjudged = []
    try:
        output = decide(hook_input, unjudged)
    except Exception as error:
        if broken("exception-unrecorded"):   # 弄坏开关：不接住，照原样抛出（改前的样子：进程退 1，不拦、不记检出）
            raise
        report_unjudged(hook_dir, hook_input, f"判的时候抛异常（{type(error).__name__}: {error}），循环提醒有没有进程在等没判；手动看：bash scripts/agents/remind.sh list",
                        "exception-unrecorded")
        return 0
    for switch, problem in unjudged:
        report_unjudged(hook_dir, hook_input, problem, switch)
    if output is not None:
        print(output)
    return 0

def report_unjudged(hook_dir, hook_input, problem, switch):
    """判不了的一处：stderr 写一行，检出记录记一行（record_unjudged）；弄坏开关 switch 下只打 stderr、不记（改前连 stderr 也不打）。"""
    print(f"  ! {HOOK_NAME}：{problem}，照常放行", file=sys.stderr)
    if not broken(switch):
        record_unjudged(hook_dir, hook_input, problem)

# ── 自证：每一格造一个答案已知的输入
def selftest_cells(hook_dir, root):
    failures = []
    checked = [0]
    hook = os.path.join(hook_dir, "remind-rearm-stop-guard.sh")
    detections = os.path.join(root, "unjudged-detections.jsonl")   # 判不了的几格记的检出落在自证自己的临时目录，不碰真的检出记录
    environment = dict(os.environ, REMIND_STATE_ROOT=root, **{DETECTIONS_VARIABLE: detections})
    environment.pop(BREAK_VARIABLE, None)
    environment.pop("SINGLEFS_WORK_ROOT_BREAK", None)
    if os.environ.get(BREAK_VARIABLE):
        environment[BREAK_VARIABLE] = os.environ[BREAK_VARIABLE]

    def check(cell, ok, detail):
        checked[0] += 1
        if not ok:
            failures.append(cell)
            print(f"✗ 自证 {cell}：{detail}")
            print(f"→ 怎么办：看 remind-rearm-stop-guard.sh 里 {cell} 对应的判定（restart_commands、is_waiting、decide；library-unreadable 那两格看 main 与 record_unjudged，"
                  f"-recorded 结尾的几格看 judge_raw 与 report_unjudged）；{BREAK_VARIABLE} 设着的话这里本来就该红")

    def register(session, name, once, restart, identity=None):
        # 登记里的 every=30 与 restart 里的 --every 数字都只是样本值：钩子不读也不假设任何周期，只看登记在不在（once=）、进程活没活
        # identity 是 <名字>.pid 那一行：「<进程号> <启动时刻>」，旧登记只有进程号
        directory = os.path.join(root, session)
        os.makedirs(directory, exist_ok=True)
        with open(os.path.join(directory, name + ".conf"), "w") as handle:
            handle.write(f"every=30\nonce={once}\nmessage=试跑\nrestart={restart}\n")
        if identity is not None:
            with open(os.path.join(directory, name + ".pid"), "w") as handle:
                handle.write(f"{identity}\n")

    def identity(process, start_time_shift=0):
        return f"{process.pid} {int(waiter_library.process_start_time(process.pid)) + start_time_shift}"

    def start_holder(arguments):
        # 假进程只读自己的标准输入管道（没人写），整个只有这一个进程，判完就杀；活多久与提醒周期无关
        return subprocess.Popen(arguments, stdin=subprocess.PIPE, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    def stop_holder(process):
        process.kill()
        process.wait()
        process.stdin.close()

    def feed(payload, raw=None):
        completed = subprocess.run(["bash", hook], input=raw if raw is not None else json.dumps(payload), capture_output=True, text=True, env=environment)
        return completed.returncode, completed.stdout

    def blocked_reason(output):
        try:
            data = json.loads(output)
        except ValueError:
            return None
        return data.get("reason") if data.get("decision") == "block" else None

    # 没在等的循环提醒：拦，reason 里有照原样重起的命令
    register("s-loop", "a", 0, "bash /x/remind.sh start --every 30 --message 甲 --name a")
    code, output = feed({"session_id": "s-loop"})
    reason = blocked_reason(output)
    check("not-waiting", code == 0 and reason is not None and "bash /x/remind.sh start --every 30 --message 甲 --name a" in reason and "run_in_background" in reason,
          f"没在等的循环提醒应当拦下并交出重起命令，实际退 {code}，输出：{output!r}")
    # --once 的登记不管
    register("s-once", "a", 1, "bash /x/remind.sh start --every 1 --message 乙 --name a --once")
    code, output = feed({"session_id": "s-once"})
    check("once", code == 0 and output == "", f"once=1 的登记应当放行、什么都不打，实际退 {code}，输出：{output!r}")
    # 有进程在等（bash 跑一份叫 remind.sh 的脚本，登记的启动时刻对得上）：放行
    fake_script = os.path.join(root, "fake", "remind.sh")
    os.makedirs(os.path.dirname(fake_script), exist_ok=True)
    with open(fake_script, "w") as handle:
        handle.write("read -r _\n")
    waiter = start_holder(["bash", fake_script])
    try:
        register("s-wait", "a", 0, "bash /x/remind.sh start --every 30 --message 丙 --name a", identity=identity(waiter))
        code, output = feed({"session_id": "s-wait"})
        check("waiting", code == 0 and output == "", f"有进程在等应当放行，实际退 {code}，输出：{output!r}")
        # 进程号与启动时刻都对得上，但进程是别的命令（命令行里没有 remind.sh）：照拦
        other = subprocess.Popen(["sleep", "infinity"], stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        try:
            register("s-reused", "a", 0, "bash /x/remind.sh start --every 30 --message 丁 --name a", identity=identity(other))
            code, output = feed({"session_id": "s-reused"})
            check("stale-pid", code == 0 and blocked_reason(output) is not None, f"进程号对应的不是 remind.sh 应当照拦，实际退 {code}，输出：{output!r}")
        finally:
            other.kill()
            other.wait()
        # 无关进程：bash -c 跑命令串，remind.sh 只是它的一个参数；进程号与启动时刻写进登记，照拦、交出重起命令
        unrelated = start_holder(["bash", "-c", "read -r _", "remind.sh"])
        try:
            register("s-unrelated", "a", 0, "bash /x/remind.sh start --every 30 --message 庚 --name a", identity=identity(unrelated))
            code, output = feed({"session_id": "s-unrelated"})
            reason = blocked_reason(output) or ""
            check("unrelated-argument", code == 0 and "bash /x/remind.sh start --every 30 --message 庚 --name a" in reason,
                  f"参数里带 remind.sh 的无关进程 {unrelated.pid} 写进登记，应当判没人在等、交出重起命令，实际退 {code}，输出：{output!r}")
        finally:
            stop_holder(unrelated)
        # 命令确是 bash …/remind.sh，登记的启动时刻对不上（进程号被复用的样子）：照拦
        register("s-mismatch", "a", 0, "bash /x/remind.sh start --every 30 --message 辛 --name a", identity=identity(waiter, start_time_shift=1))
        code, output = feed({"session_id": "s-mismatch"})
        reason = blocked_reason(output) or ""
        check("start-time-mismatch", code == 0 and "bash /x/remind.sh start --every 30 --message 辛 --name a" in reason,
              f"登记的启动时刻与进程 {waiter.pid} 此刻的对不上，应当判没人在等、交出重起命令，实际退 {code}，输出：{output!r}")
        # 旧登记只有进程号、没有启动时刻：照拦
        register("s-legacy", "a", 0, "bash /x/remind.sh start --every 30 --message 壬 --name a", identity=str(waiter.pid))
        code, output = feed({"session_id": "s-legacy"})
        reason = blocked_reason(output) or ""
        check("legacy-registration", code == 0 and "bash /x/remind.sh start --every 30 --message 壬 --name a" in reason,
              f"旧登记只有进程号 {waiter.pid}、没有启动时刻，应当判没人在等、交出重起命令，实际退 {code}，输出：{output!r}")
    finally:
        stop_holder(waiter)
    # 已经拦过一次：放行
    code, output = feed({"session_id": "s-loop", "stop_hook_active": True})
    check("active", code == 0 and output == "", f"stop_hook_active 为真应当放行，实际退 {code}，输出：{output!r}")
    # 别的会话的登记不碰
    code, output = feed({"session_id": "s-nothing"})
    check("other-session", code == 0 and output == "", f"这个会话没有登记，别的会话的登记不该拦，实际退 {code}，输出：{output!r}")
    # 同一个会话几条都没在等：命令全交出
    register("s-several", "a", 0, "bash /x/remind.sh start --every 5 --message 戊 --name a")
    register("s-several", "b", 0, "bash /x/remind.sh start --every 7 --message 己 --name b")
    code, output = feed({"session_id": "s-several"})
    reason = blocked_reason(output) or ""
    check("several", "--name a" in reason and "--name b" in reason, f"两条都没在等应当两条命令都交出，实际：{output!r}")
    # 读不到 lib_remind_waiter.py：照常放行（退 0、不拦），stderr 写一句，检出记录记一行（结局只记、会话号照输入）；
    #   没设 AGENT_HOOK_DETECTIONS 时记进同目录 lib_work_root.py 的默认位置（这里把 SINGLEFS_WORK_ROOT 指进临时目录，不碰真的检出记录）
    bare = os.path.join(root, "bare-hooks")
    os.makedirs(bare, exist_ok=True)
    for library_name in ("remind-rearm-stop-guard.sh", "lib_work_root.py"):
        shutil.copy(os.path.join(hook_dir, library_name), bare)
    work_root_directory = os.path.join(root, "work-root")
    for cell, extra, path in (("library-unreadable", {DETECTIONS_VARIABLE: os.path.join(root, "detections.jsonl")}, os.path.join(root, "detections.jsonl")),
                              ("library-unreadable-default-path", {"SINGLEFS_WORK_ROOT": work_root_directory},
                               os.path.join(work_root_directory, "agent-hook-detections.jsonl"))):
        bare_environment = {key: value for key, value in environment.items() if key not in (DETECTIONS_VARIABLE, "SINGLEFS_WORK_ROOT_BREAK")}
        bare_environment.update(extra)
        completed = subprocess.run(["bash", os.path.join(bare, "remind-rearm-stop-guard.sh")], input=json.dumps({"session_id": "s-bare"}),
                                   capture_output=True, text=True, env=bare_environment)
        try:
            lines = [json.loads(line) for line in open(path, encoding="utf-8")]
        except (OSError, ValueError):
            lines = []
        recorded = (len(lines) == 1 and lines[0].get("outcome") == "只记" and lines[0].get("hook") == HOOK_NAME and lines[0].get("session_id") == "s-bare"
                    and "lib_remind_waiter.py" in " ".join(lines[0].get("findings") or []))
        check(cell, completed.returncode == 0 and completed.stdout == "" and "lib_remind_waiter.py" in completed.stderr and recorded,
              f"读不到 lib_remind_waiter.py 应当退 0、不拦、stderr 写一句、{path} 记一行（结局只记、会话号 s-bare），实际退 {completed.returncode}，"
              f"stdout {completed.stdout!r}，stderr {completed.stderr.strip()[:300]!r}，检出 {lines!r}")
    # 输入读不出、没有会话号、会话号带斜杠：放行不报错
    state = True
    for raw in ("", "不是 JSON", "[]", json.dumps({}), json.dumps({"session_id": "../s-loop"})):
        code, output = feed(None, raw=raw)
        state = state and code == 0 and output == ""
    check("bad-input", state, "读不出的输入、没有会话号、会话号带路径符号都应当放行（退 0、什么都不打）")

    def unjudged_lines():
        try:
            with open(detections, encoding="utf-8") as handle:
                return [json.loads(line) for line in handle if line.strip()]
        except (OSError, ValueError):
            return []

    def expect_recorded(cell, runs, sessions):
        """runs 里每一次（钩子入口或进程里的 judge_raw）交 (退出码, stdout, stderr)：都要退 0、不拦，检出记录照次序多出 sessions 那几行（结局只记）。"""
        before = len(unjudged_lines())
        outcomes = []
        for run in runs:
            try:
                outcomes.append(run())
            except Exception as error:   # 弄坏开关 exception-unrecorded：进程里照原样抛出
                outcomes.append((f"抛出 {type(error).__name__}", "", ""))
        added = unjudged_lines()[before:]
        check(cell, all(code == 0 and stdout == "" and HOOK_NAME in stderr for code, stdout, stderr in outcomes)
              and [entry.get("session_id") for entry in added] == sessions and all(entry.get("outcome") == "只记" and entry.get("hook") == HOOK_NAME for entry in added),
              f"应当各退 0、不拦、stderr 写一行、检出记录多 {len(sessions)} 行（结局只记、会话号 {sessions}）；实际 {outcomes!r:.300}，检出多了 {len(added)} 行：{added!r:.400}")

    def through_entry(raw):
        def run():
            completed = subprocess.run(["bash", hook], input=raw, capture_output=True, text=True, env=environment)
            return completed.returncode, completed.stdout, completed.stderr
        return run

    def raising_restart_commands(root_directory, session, unreadable=None):
        raise OSError("自证造的异常")

    def in_process_raising():
        import io
        saved = (globals()["restart_commands"], sys.stdout, sys.stderr, os.environ.get(DETECTIONS_VARIABLE))
        globals()["restart_commands"], sys.stdout, sys.stderr, os.environ[DETECTIONS_VARIABLE] = raising_restart_commands, io.StringIO(), io.StringIO(), detections
        try:
            code = judge_raw(json.dumps({"session_id": "s-raise"}).encode("utf-8"), hook_dir)
            return code, sys.stdout.getvalue(), sys.stderr.getvalue()
        finally:
            globals()["restart_commands"], sys.stdout, sys.stderr = saved[0], saved[1], saved[2]
            if saved[3] is None:
                os.environ.pop(DETECTIONS_VARIABLE, None)
            else:
                os.environ[DETECTIONS_VARIABLE] = saved[3]

    # 判不了的照「放行但必记检出」：输入不是 JSON 对象、认不出会话号、登记读不了、判的时候抛异常，都照常放行（退 0、不拦）并各记一条检出
    expect_recorded("bad-input-recorded", [through_entry("不是 JSON"), through_entry("[]")], ["读不到", "读不到"])
    expect_recorded("session-unrecognized-recorded", [through_entry(json.dumps({})), through_entry(json.dumps({"session_id": "../s-loop"}))], ["读不到", "../s-loop"])
    os.makedirs(os.path.join(root, "s-unreadable", "broken.conf"))   # 登记文件的位置被目录占着：读不了
    expect_recorded("settings-unreadable-recorded", [through_entry(json.dumps({"session_id": "s-unreadable"}))], ["s-unreadable"])
    expect_recorded("exception-recorded", [in_process_raising], ["s-raise"])
    return failures, checked[0]

def selftest_run_cells(hook_dir):
    root = tempfile.mkdtemp(prefix="remind-rearm-stop-selftest-")
    try:
        failures, checked = selftest_cells(hook_dir, root)
    finally:
        for directory, _, files in os.walk(root, topdown=False):
            for file_name in files:
                os.remove(os.path.join(directory, file_name))
            os.rmdir(directory)
    if failures:
        print(f"✗ 自证：{len(failures)} 格红（共判了 {checked} 格）")
        print("→ 怎么办：照上面每一格的下一步改，再单独跑 remind-rearm-stop-guard.sh --selftest")
        return 1
    print(f"✓ 各格都对（判了 {checked} 格）")
    return 0

BREAK_TABLE = [("ignore-once", ("once",)), ("ignore-waiting", ("waiting",)), ("trust-pid-file", ("stale-pid",)),
               ("ignore-active", ("active",)), ("all-sessions", ("other-session",)), ("first-only", ("several",)),
               ("identity-by-name", ("unrelated-argument", "start-time-mismatch", "legacy-registration")),
               ("failure-unrecorded", ("library-unreadable", "library-unreadable-default-path")),
               ("input-unrecorded", ("bad-input-recorded",)), ("session-unrecorded", ("session-unrecognized-recorded",)),
               ("settings-unrecorded", ("settings-unreadable-recorded",)), ("exception-unrecorded", ("exception-recorded",))]

def selftest(hook_dir):
    hook = os.path.join(hook_dir, "remind-rearm-stop-guard.sh")
    clean = {key: value for key, value in os.environ.items() if key != BREAK_VARIABLE}
    completed = subprocess.run(["bash", hook, "--selftest-cells"], capture_output=True, text=True, env=clean)
    if completed.returncode != 0:
        print("✗ 自证：不带弄坏开关时各格应当全绿，实际：")
        print(completed.stdout + completed.stderr)
        print("→ 怎么办：先让 --selftest-cells 不带弄坏开关跑绿，再回来跑本自证")
        return 1
    failed = 0
    for switch, cells in BREAK_TABLE:
        completed = subprocess.run(["bash", hook, "--selftest-cells"], capture_output=True, text=True, env=dict(clean, **{BREAK_VARIABLE: switch}))
        missing = [cell for cell in cells if f"✗ 自证 {cell}：" not in completed.stdout]
        if completed.returncode == 0 or missing:
            failed += 1
            print(f"✗ 自证：打开弄坏开关 {switch} 之后应当红在「{'」「'.join(cells)}」格，没红的：{'、'.join(missing) or '（都红了，但退 0）'}，实际退 {completed.returncode}，输出：{completed.stdout!r}")
            print(f"→ 怎么办：{switch} 弄坏的那一处要真影响「{'」「'.join(cells)}」格的判定；两边对不上就改开关或改格")
    if failed:
        return 1
    print(f"✓ remind-rearm-stop-guard.sh 自证：各格都对，{len(BREAK_TABLE)} 个弄坏开关各红在点名的格")
    return 0

def main():
    global waiter_library
    hook_dir = sys.argv[1]
    try:
        waiter_library = load_waiter_library(hook_dir)
    except (ImportError, OSError, SyntaxError) as error:
        if sys.argv[2:] in (["--selftest"], ["--selftest-cells"]):
            print(f"✗ 自证：读不到 {hook_dir}/lib_remind_waiter.py（{error!r}），判不了")
            print("→ 怎么办：它随仓走，被删了就从 git 找回来；改坏了先跑 python3 -m py_compile 看哪一行")
            return 1
        problem = (f"读不到 {hook_dir}/lib_remind_waiter.py（{error!r}），这一次判不了循环提醒有没有进程在等，照常放行；"
                   f"手动看：bash scripts/agents/remind.sh list")
        print(f"  ! remind-rearm-stop-guard.sh：{problem}", file=sys.stderr)
        if not broken("failure-unrecorded"):   # 弄坏开关：只打 stderr、不记检出（改前的样子）
            record_unjudged(hook_dir, read_hook_input(), problem)
        return 0
    if sys.argv[2:] == ["--selftest"]:
        return selftest(hook_dir)
    if sys.argv[2:] == ["--selftest-cells"]:
        return selftest_run_cells(hook_dir)
    return hook_main(hook_dir)

sys.exit(main())
PY
