#!/usr/bin/env python3
# admission: always 起的是调用方这一次给的命令，上一次起过、收干净过不替这一次作保
# run-condition: none 只用 python 标准库起子进程、发信号给自己起的那一组、读 /proc，不要别的工具
# gate-similar: verus-proof-check.py 它的 run_streamed 同样放进自己的进程组、退了停组里留下的，但两路输出边跑边收给进度行用、只起 Verus 那一种命令；这一份是门禁侧起阶段、自证与探针的共用起法，不并进去
# gate-similar: doc-registries.sh 它有一段内联的 start_new_session 加 killpg，在那道门禁的格正文里、只给那一格用；这一份抽给 gate-structure-check.py 与 code-tooling.sh 的两格共用，那一段这一批不动
"""门禁侧起子进程的共用起法：每条命令放进它自己的会话与进程组，超时、异常、收到信号与正常退出之后都收整组，
再在 /proc 里核这个会话里没有留下进程（.claude/singlefs-ai-sop/rules/command-safety.md「装置起的样本进程，放进它自己的进程组」）。

调它的：scripts/gate/gate-structure-check.py 的 start_stage（起各道门禁的 --list 与 --check，超时限 LIST_TIMEOUT_SECONDS 秒）；
.claude/gate.d/code-tooling.sh 的 stage-break-switch-replay 格（逐条开关起 stage-selftest.sh，不限时）与
break-switch-outside-selftest 格（设上弄坏开关起探针，限 30 秒）。两边都调这一份，不各写一份起法。

run_in_own_session(命令, timeout=秒或 None, leader_only=False, **交给 Popen 的参数) → SessionRun
  起法：subprocess.Popen(start_new_session=True)，stdout、stderr 各接一根管道，按 UTF-8 解（坏字节换成 U+FFFD）。
  收组：先对整组发 SIGTERM，过 TERM_GRACE_SECONDS 秒组里还有没退出的（僵尸不算）再发 SIGKILL。
    超时：收组，再收管道里剩下的输出（至多 SETTLE_SECONDS 秒；管道还被组外的进程拿着就不再等）。
    正常退出：会话里还有进程的，记进 left_behind，再收组。
    异常（含 KeyboardInterrupt 与下面信号处理抛的 SystemExit）：收组之后照原样抛出。
    收到信号：install_stop_on_signals 装了处理之后，TERM、INT、HUP 到了先记一笔，各线程里在跑的这一条至多 POLL_SECONDS 秒后醒来收自己那一组、抛 SystemExit；
      主线程里由信号处理当场抛 SystemExit(128 + 信号号)。
  核：收组之后在 /proc 里按会话号找，还在的（不在这一组里、SIGKILL 没打到的，或停不掉的）记进 stuck。
  leader_only=True 只给 gate-structure-check.py 的弄坏开关 timeout-kills-leader-only 用：不开新会话，
    subprocess.run(timeout=…) 超时只 SIGKILL 直接起的那一个、不收组、不核，left_behind 与 stuck 都空着。
SessionRun 字段：returncode（超时是 None）、stdout、stderr、timed_out、left_behind、stuck；
  left_behind 与 stuck 是「<进程号> <命令行前 120 个字符>」的列表；problem() 把这两样拼成一句失败信息（都空交 None），调用方把它写进那条样本的失败信息。
install_stop_on_signals()：在主线程调一次（不在主线程交 False、什么都不装）；run_in_own_session 在主线程里第一次被调时也装一次，
  只换掉还是缺省处理的那几个信号。起子进程的线程池在开池之前由调用方在主线程里调它。
"""
import os
import signal
import subprocess
import threading
import time

TERM_GRACE_SECONDS = 8      # 比 .claude/gate.d/lib/stage-cells.sh 收格的宽限（STAGE_CELLS_TERM_GRACE_SECONDS，5 秒）长：被收的是门禁阶段时，它先把正在跑的那一格收完、自己退出
SETTLE_SECONDS = 2          # 收组之后再等管道收尾、/proc 里的进程退干净的秒数
POLL_SECONDS = 0.5          # 等命令的时候每隔这么久醒一次，看超时与信号
STOP_SIGNALS = (signal.SIGTERM, signal.SIGINT, signal.SIGHUP)
COMMAND_LINE_SHOWN = 120

_stop_requested = threading.Event()
_stop_signal_number = []


class SessionRun:
    __slots__ = ("returncode", "stdout", "stderr", "timed_out", "left_behind", "stuck")

    def __init__(self, returncode, stdout, stderr, timed_out, left_behind, stuck):
        self.returncode, self.stdout, self.stderr = returncode, stdout, stderr
        self.timed_out, self.left_behind, self.stuck = timed_out, left_behind, stuck

    def problem(self):
        """left_behind 与 stuck 拼成的一句失败信息；两样都空交 None。"""
        parts = []
        if self.left_behind:
            parts.append(f"退出之后它起的进程还留着 {len(self.left_behind)} 个（已对整组 TERM、过宽限 KILL）：{'；'.join(self.left_behind)}")
        if self.stuck:
            parts.append(f"收组之后会话里还剩 {len(self.stuck)} 个停不掉或不在这一组里的进程：{'；'.join(self.stuck)}")
        return "，".join(parts) if parts else None


def _text(data):
    if data is None:
        return ""
    if isinstance(data, bytes):
        return data.decode("utf-8", errors="replace")
    return data


def _stop_on_signal(signal_number, _frame):
    _stop_signal_number.append(signal_number)
    _stop_requested.set()
    raise SystemExit(128 + signal_number)


def install_stop_on_signals(only_defaults=False):
    """装 TERM、INT、HUP 的处理（只能在主线程装）：记下收到了、在主线程抛 SystemExit(128 + 信号号)。
    only_defaults=True 只换还是缺省处理的那几个（调用方自己装过的不动）。不在主线程交 False。"""
    if threading.current_thread() is not threading.main_thread():
        return False
    for signal_number in STOP_SIGNALS:
        current = signal.getsignal(signal_number)
        if only_defaults and current not in (signal.SIG_DFL, signal.default_int_handler, None):
            continue
        signal.signal(signal_number, _stop_on_signal)
    return True


def _described(pid):
    try:
        with open(f"/proc/{pid}/cmdline", "rb") as handle:
            command_line = handle.read().replace(b"\0", b" ").decode("utf-8", errors="replace").strip()
    except OSError:
        command_line = "（读不到命令行）"
    return f"{pid} {command_line[:COMMAND_LINE_SHOWN]}"


def _live_processes(field_index, wanted):
    """/proc 里 stat 第 field_index 段（状态之后数，0 是状态）等于 wanted、又不是僵尸的进程号。只读，不发信号。"""
    found = []
    for entry in os.listdir("/proc"):
        if not entry.isdigit():
            continue
        try:
            with open(f"/proc/{entry}/stat", "rb") as handle:
                fields = handle.read().rsplit(b")", 1)[1].split()
        except (OSError, IndexError):
            continue
        if len(fields) > field_index and fields[0] not in (b"Z", b"X") and int(fields[field_index]) == wanted:
            found.append(int(entry))
    return sorted(found)


def group_members(group_id):
    """这个进程组里还没退出的进程号（僵尸不算）。"""
    return _live_processes(2, group_id)


def session_members(session_id):
    """这个会话里还没退出的进程号（僵尸不算）。"""
    return _live_processes(3, session_id)


def _wait_until_empty(members_of, identifier, seconds):
    deadline = time.monotonic() + seconds
    while members_of(identifier) and time.monotonic() < deadline:
        time.sleep(0.1)
    return members_of(identifier)


def run_in_own_session(command, timeout=None, leader_only=False, **popen_arguments):
    if leader_only:
        try:
            done = subprocess.run(command, capture_output=True, encoding="utf-8", errors="replace", timeout=timeout, **popen_arguments)
            return SessionRun(done.returncode, done.stdout, done.stderr, False, [], [])
        except subprocess.TimeoutExpired as expired:
            return SessionRun(None, _text(expired.stdout), _text(expired.stderr), True, [], [])
    install_stop_on_signals(only_defaults=True)
    process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, encoding="utf-8", errors="replace",
                               start_new_session=True, **popen_arguments)

    def stop_group():
        """对这条命令的整组先 SIGTERM，过宽限还有没退出的再 SIGKILL（组号就是它的进程号）。"""
        try:
            os.killpg(process.pid, signal.SIGTERM)
        except ProcessLookupError:
            return
        if _wait_until_empty(group_members, process.pid, TERM_GRACE_SECONDS):
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass

    deadline = None if timeout is None else time.monotonic() + timeout
    stdout = stderr = ""
    timed_out = False
    left_behind = []
    try:
        while True:
            if _stop_requested.is_set():
                stop_group()
                raise SystemExit(128 + (_stop_signal_number[0] if _stop_signal_number else signal.SIGTERM))
            slice_seconds = POLL_SECONDS if deadline is None else max(0.0, min(POLL_SECONDS, deadline - time.monotonic()))
            try:
                stdout, stderr = process.communicate(timeout=slice_seconds)
                break
            except subprocess.TimeoutExpired as expired:
                if deadline is not None and time.monotonic() >= deadline:
                    timed_out = True
                    stdout, stderr = _text(expired.stdout), _text(expired.stderr)
                    break
    except BaseException:
        stop_group()
        raise
    if timed_out:
        stop_group()
        try:
            stdout, stderr = process.communicate(timeout=SETTLE_SECONDS)
        except subprocess.TimeoutExpired:
            for stream in (process.stdout, process.stderr):
                stream.close()
            process.wait()
    else:
        left_behind = [_described(pid) for pid in session_members(process.pid)]
        if left_behind:
            stop_group()
    stuck = [_described(pid) for pid in _wait_until_empty(session_members, process.pid, SETTLE_SECONDS)]
    return SessionRun(None if timed_out else process.returncode, stdout or "", stderr or "", timed_out, left_behind, stuck)
