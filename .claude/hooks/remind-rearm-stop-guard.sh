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
# 认「在等」：<名字>.pid 里的进程活着、命令行里带 remind.sh（进程号被别的进程复用了不算）。once=1 的登记不管。
# 已经因为这个钩子拦过一次（hook 输入 stop_hook_active 为真）还没重起的，放行，免得卡死；下一轮结束时照样再拦。
# 不读也不假设提醒周期：只看登记在不在（once=）、进程活没活；自证里写的 every= 与 --every 数字都只是样本值。
# 读不到输入、认不出会话号、没有登记目录时什么都不打、退 0。
# 退役判据：循环提醒不再用 remind.sh 登记，或 Claude Code 自己提供到点自启动时删掉。
#
#   remind-rearm-stop-guard.sh --selftest   在临时登记目录里喂造出来的 hook JSON，末尾逐个打开弄坏开关核它红在点名的格
# 弄坏开关 REMIND_REARM_STOP_BREAK=<项>（只给证红用，逗号连几个）。项名、弄坏哪一处、打开后红在哪一格（格名是「✗ 自证 <格名>：」里的那一段）：
#   ignore-once         once=1 的登记也当要重起                 once
#   ignore-waiting      不看有没有进程在等                       waiting
#   trust-pid-file      进程号文件在就当在等（不核进程活着、命令行带 remind.sh）   stale-pid
#   ignore-active       stop_hook_active 为真时仍然拦             active
#   all-sessions        不按会话号分，读所有会话的登记             other-session
#   first-only          只交出第一条要重起的命令                   several
set -uo pipefail
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
# python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import json, os, re, subprocess, sys, tempfile, time

BREAK_VARIABLE = "REMIND_REARM_STOP_BREAK"
SESSION_PATTERN = re.compile(r"^[A-Za-z0-9_-]+$")

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

def is_waiting(directory, name):
    pid_path = os.path.join(directory, name + ".pid")
    if not os.path.isfile(pid_path):
        return False
    if broken("trust-pid-file"):
        return True
    try:
        pid = int(open(pid_path).read().strip())
        os.kill(pid, 0)
        return b"remind.sh" in open(f"/proc/{pid}/cmdline", "rb").read()
    except (ValueError, OSError):
        return False

def sessions_to_read(root, session):
    if broken("all-sessions"):
        return [os.path.join(root, entry) for entry in sorted(os.listdir(root))] if os.path.isdir(root) else []
    return [os.path.join(root, session)]

def restart_commands(root, session):
    """这个会话里登记着、没人在等的循环提醒，各自照原样重起的命令。"""
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
            except OSError:
                continue
            if settings.get("once") == "1" and not broken("ignore-once"):
                continue
            if is_waiting(directory, name) and not broken("ignore-waiting"):
                continue
            if settings.get("restart"):
                commands.append(settings["restart"])
    return commands

def decide(hook_input):
    """hook 输入 → 要打到标准输出的 JSON 文本（放行时是 None）。"""
    session = str(hook_input.get("session_id") or "")
    if not SESSION_PATTERN.match(session):
        return None
    commands = restart_commands(state_root(), session)
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

def hook_main():
    try:
        hook_input = json.loads(sys.stdin.read())
    except ValueError:
        return 0
    if not isinstance(hook_input, dict):
        return 0
    output = decide(hook_input)
    if output is not None:
        print(output)
    return 0

# ── 自证：每一格造一个答案已知的输入
def selftest_cells(hook_dir, root):
    failures = []
    checked = [0]
    hook = os.path.join(hook_dir, "remind-rearm-stop-guard.sh")
    environment = dict(os.environ, REMIND_STATE_ROOT=root)
    environment.pop(BREAK_VARIABLE, None)
    if os.environ.get(BREAK_VARIABLE):
        environment[BREAK_VARIABLE] = os.environ[BREAK_VARIABLE]

    def check(cell, ok, detail):
        checked[0] += 1
        if not ok:
            failures.append(cell)
            print(f"✗ 自证 {cell}：{detail}")
            print(f"→ 怎么办：看 remind-rearm-stop-guard.sh 里 {cell} 对应的判定（restart_commands、is_waiting、decide）；{BREAK_VARIABLE} 设着的话这里本来就该红")

    def register(session, name, once, restart, pid=None):
        # 登记里的 every=30 与 restart 里的 --every 数字都只是样本值：钩子不读也不假设任何周期，只看登记在不在（once=）、进程活没活
        directory = os.path.join(root, session)
        os.makedirs(directory, exist_ok=True)
        with open(os.path.join(directory, name + ".conf"), "w") as handle:
            handle.write(f"every=30\nonce={once}\nmessage=试跑\nrestart={restart}\n")
        if pid is not None:
            with open(os.path.join(directory, name + ".pid"), "w") as handle:
                handle.write(f"{pid}\n")

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
    # 有进程在等（命令行带 remind.sh）：放行
    # 假的「在等」进程：只是一个活着的假进程，命令行里带 remind.sh，活多久与提醒周期无关，判完就杀。
    # exec -a 把进程名换成 remind.sh，整个只有这一个进程，杀它不会留下占着输出管道的孙进程；标准输入输出接空设备。
    waiter = subprocess.Popen(["bash", "-c", "exec -a remind.sh sleep infinity"], stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        register("s-wait", "a", 0, "bash /x/remind.sh start --every 30 --message 丙 --name a", pid=waiter.pid)
        code, output = feed({"session_id": "s-wait"})
        check("waiting", code == 0 and output == "", f"有进程在等应当放行，实际退 {code}，输出：{output!r}")
        # 进程号文件在，但进程是别的命令（进程号被复用）：照拦
        # 另一个活着的假进程（命令行里没有 remind.sh），同样与提醒周期无关，判完就杀
        other = subprocess.Popen(["sleep", "infinity"], stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        try:
            register("s-reused", "a", 0, "bash /x/remind.sh start --every 30 --message 丁 --name a", pid=other.pid)
            code, output = feed({"session_id": "s-reused"})
            check("stale-pid", code == 0 and blocked_reason(output) is not None, f"进程号对应的不是 remind.sh 应当照拦，实际退 {code}，输出：{output!r}")
        finally:
            other.kill()
            other.wait()
    finally:
        waiter.kill()
        waiter.wait()
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
    # 输入读不出、没有会话号、会话号带斜杠：放行不报错
    state = True
    for raw in ("", "不是 JSON", "[]", json.dumps({}), json.dumps({"session_id": "../s-loop"})):
        code, output = feed(None, raw=raw)
        state = state and code == 0 and output == ""
    check("bad-input", state, "读不出的输入、没有会话号、会话号带路径符号都应当放行（退 0、什么都不打）")
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

BREAK_TABLE = [("ignore-once", "once"), ("ignore-waiting", "waiting"), ("trust-pid-file", "stale-pid"),
               ("ignore-active", "active"), ("all-sessions", "other-session"), ("first-only", "several")]

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
    for switch, cell in BREAK_TABLE:
        completed = subprocess.run(["bash", hook, "--selftest-cells"], capture_output=True, text=True, env=dict(clean, **{BREAK_VARIABLE: switch}))
        if completed.returncode == 0 or f"✗ 自证 {cell}：" not in completed.stdout:
            failed += 1
            print(f"✗ 自证：打开弄坏开关 {switch} 之后应当红在「{cell}」格，实际退 {completed.returncode}，输出：{completed.stdout!r}")
            print(f"→ 怎么办：{switch} 弄坏的那一处要真影响「{cell}」格的判定；两边对不上就改开关或改格")
    if failed:
        return 1
    print(f"✓ remind-rearm-stop-guard.sh 自证：各格都对，{len(BREAK_TABLE)} 个弄坏开关各红在点名的格")
    return 0

def main():
    hook_dir = sys.argv[1]
    if sys.argv[2:] == ["--selftest"]:
        return selftest(hook_dir)
    if sys.argv[2:] == ["--selftest-cells"]:
        return selftest_run_cells(hook_dir)
    return hook_main()

sys.exit(main())
PY
