#!/usr/bin/env python3
"""攻方腿第二轮探针：看门狗「知识同步提醒」两批修补之后，落盘、被忽略的条目、porcelain 加 -z、摘要没变不点名、给人抄的命令、命名管道几处的行为。

用法：
  knowledge_sync_r2_probe.py <agent-watch.py 的路径> <运行目录>
      在 <运行目录> 底下逐个场景建小仓与临时会话目录，导入被判的 agent-watch.py，直接调它的 take_repository_baseline、
      knowledge_sync_section、repository_status_section、repository_snapshot 与 knowledge_state_path（看门狗起来时与退出时调的就是这几个）；
      看门狗模块里的时钟换成探针自己拨的假时钟（只换 time.time，算摘要时限用的 time.monotonic 照真的走），一串退出时刻照场景喂。
      每个场景打一行 PROBE name=<场景> …；末行 PROBE name=done emitted=<前面 PROBE 行的条数> setup_failures=<探针自己搭场景失败的条数>。
  knowledge_sync_r2_probe.py --child <agent-watch.py 的路径> <小仓>
      只给本探针自己起：在小仓上记一次基线，打 child-finished 与列出的被忽略文件（目录里有命名管道时，弄坏开关下预期卡住）。
只碰 <运行目录> 底下自己建的小仓与会话目录：git 带 GIT_CEILING_DIRECTORIES=<运行目录>，不读用户与系统配置（配置那一格另给自造的全局配置）；
HOME 换成运行目录里的空目录、去掉 TASK_SERVER_SOURCE_REPOSITORY，按 agent 号找会话记录只找到运行目录里，不碰真的会话目录。
起的子进程（本探针的 --child、照抄跑的 bash）各在自己的进程组，到点只停那一组。
退出码：0 全部场景跑完（被判对象自己抛的异常算读数）；1 探针自己搭场景失败；2 用法错。
"""
import argparse
import hashlib
import importlib.util
import json
import os
import re
import signal
import socket
import subprocess
import sys
import time

NEUTRAL_GIT_CONFIG = {"GIT_CONFIG_GLOBAL": os.devnull, "GIT_CONFIG_NOSYSTEM": "1"}
GIT_IDENTITY = ["-c", "user.name=probe", "-c", "user.email=probe@example.invalid", "-c", "commit.gpgsign=false", "-c", f"core.hooksPath={os.devnull}"]
MAX_MINUTES = 60                 # watch.conf 没改 max-minutes，取 agent-watch.py 的默认 60
CHILD_SECONDS = 20               # --child 记一次基线的上限
SPARSE_BYTES = 8 << 30           # 被忽略的大文件那一格：8 GiB 稀疏文件
IGNORED_DIRECTORY = ".claude/singlefs-ai-sop/rules/"
EPOCH = time.time()
emitted_lines = 0
setup_failures = []


def emit(name, **fields):
    global emitted_lines
    emitted_lines += 1
    text = " ".join(f"{key}={value}" for key, value in fields.items())
    print(f"PROBE name={name} {text}", flush=True)


def as_json(value):
    return json.dumps(value, ensure_ascii=False)


class FakeClock:
    """只换 time.time：看门狗算「离上次提醒多久」、落盘 reminded_at 用的那一个；别的照真的 time 模块。"""

    def __init__(self):
        self.now = EPOCH

    def time(self):
        return self.now

    def at(self, minutes):
        self.now = EPOCH + minutes * 60
        return self.now

    def __getattr__(self, name):
        return getattr(time, name)


def load_watchdog(path):
    spec = importlib.util.spec_from_file_location("agent_watch_under_probe", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def git(repository, *arguments, check=True, extra_environment=None):
    environment = dict(os.environ, **NEUTRAL_GIT_CONFIG, **(extra_environment or {}))
    finished = subprocess.run(["git", *GIT_IDENTITY, "-C", repository, *arguments], capture_output=True,
                              errors="surrogateescape", env=environment, timeout=60, text=True)
    if check and finished.returncode != 0:
        raise RuntimeError(f"git {' '.join(arguments)} 退 {finished.returncode}：{finished.stderr.strip()}")
    return finished


def write_file(repository, relative_path, text):
    path = os.path.join(repository, relative_path)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)


def new_repository(run_directory, name, files):
    """建一个小仓：分支 main，files 一次提交成起点。"""
    repository = os.path.join(run_directory, name)
    os.makedirs(repository)
    git(repository, "init", "-q")
    git(repository, "symbolic-ref", "HEAD", "refs/heads/main")
    for relative_path, text in files.items():
        write_file(repository, relative_path, text)
    git(repository, "add", "-A")
    git(repository, "commit", "-q", "-m", "起点")
    return repository


def arguments_of(repository, limit=20):
    return argparse.Namespace(repository_report_lines=limit, repository_root=repository, max_minutes=MAX_MINUTES, session_dir=None)


def session_state(run_directory, name):
    """这一格自己的临时会话目录与落盘基线的路径（还没有文件）。"""
    directory = os.path.join(run_directory, "sessions", name)
    os.makedirs(directory)
    return os.path.join(directory, "agent-watch-knowledge-sync.json")


def start_watchdog(watchdog, clock, repository, state_path, minutes):
    """一只看门狗在第 minutes 分钟起来：与 run() 一样，起来那一刻（watch_started）交给 take_repository_baseline。"""
    return watchdog.take_repository_baseline(arguments_of(repository), clock.at(minutes), state_path)


def exit_watchdog(watchdog, clock, baseline, repository, minutes, on_alert):
    """这一只在第 minutes 分钟退出（告警退 3 是 on_alert，定时回报退 4 不是）：与 run() 一样调 knowledge_sync_section。"""
    clock.at(minutes)
    return watchdog.knowledge_sync_section(baseline, arguments_of(repository), on_alert)


def claimed_elapsed(lines):
    found = re.search(r"到此刻这 ([0-9.]+) 分钟里", lines[0]) if lines else None
    return float(found.group(1)) if found else None


def named_rows(watchdog, lines):
    return [list(row) for row in watchdog.named_knowledge_rows(lines)]


def stored_reminded_at(state_path):
    with open(state_path, encoding="utf-8") as handle:
        return round((json.load(handle)["reminded_at"] - EPOCH) / 60, 3)


# ── 第 1 问：落盘与按时带这一节 ──
def run_chain(watchdog, clock, repository, state_path, alerts, restart_delay, horizon, arm):
    """一串看门狗：每只起来时读落盘的基线，退出时刻取「下一次告警」与「定时回报」里早的那个，退出后隔 restart_delay 分钟起下一只。
    arm 是 as-is（定时回报在起来之后满 max-minutes，与 run() 的 report_deadline 相同）或 exit-when-due（零轮改法：
    定时回报的时刻再与「上次提醒 + max-minutes」取早的那个，已经满了就当场退）。→ 带了这一节的退出时刻（分钟）。"""
    reminders = []
    start = 0.0
    while start < horizon:
        baseline = start_watchdog(watchdog, clock, repository, state_path, start)
        deadline = start + MAX_MINUTES
        if arm == "exit-when-due":
            deadline = min(deadline, max(start, (baseline.reminded_at - EPOCH) / 60 + MAX_MINUTES))
        next_alert = next((moment for moment in alerts if moment > start), None)
        on_alert = next_alert is not None and next_alert < deadline
        moment = next_alert if on_alert else deadline
        if exit_watchdog(watchdog, clock, baseline, repository, moment, on_alert):
            reminders.append(round(moment, 3))
        start = moment + restart_delay
    return reminders


def scenario_exit_chains(watchdog, clock, run):
    """退出时刻那一串放开扫：告警的时刻表（环境给的）与重起的间隔（主 agent 给的）都扫，只固定 max-minutes 与「第一只起来时没有落盘文件」。"""
    repository = new_repository(run, "chain", {".claude/main-agent.md": "原文\n"})
    schedules = {"no-alerts": [], "one-alert-at-59.9": [59.9], "alerts-every-59.9": [59.9 * k for k in range(1, 8)],
                 "alerts-every-30": [30.0 * k for k in range(1, 14)], "alerts-every-10": [10.0 * k for k in range(1, 40)],
                 "alerts-at-1-and-61": [1.0, 61.0]}
    horizon = 360.0
    for arm in ("as-is", "exit-when-due"):
        worst = 0.0
        for label, alerts in schedules.items():
            for restart_delay in (0.1, 1.0, 5.0):
                state_path = session_state(run, f"chain-{arm}-{label}-{restart_delay}")
                reminders = run_chain(watchdog, clock, repository, state_path, alerts, restart_delay, horizon, arm)
                points = [0.0] + reminders
                gaps = [round(later - earlier, 3) for earlier, later in zip(points, points[1:])]
                biggest = max(gaps) if gaps else horizon
                worst = max(worst, biggest)
                emit("exit-chain", arm=arm, alerts=label, restart_delay=restart_delay, reminders=as_json(reminders), max_gap_minutes=biggest)
        emit("exit-chain-worst", arm=arm, max_gap_minutes=round(worst, 3), max_minutes=MAX_MINUTES)


def scenario_first_start(watchdog, clock, run):
    repository = new_repository(run, "first-start", {".claude/main-agent.md": "原文\n"})
    state_path = session_state(run, "first-start")
    baseline = start_watchdog(watchdog, clock, repository, state_path, 0.0)
    before = exit_watchdog(watchdog, clock, baseline, repository, 59.9, True)
    at_due = exit_watchdog(watchdog, clock, baseline, repository, 60.0, False)
    emit("first-start", state_file=os.path.exists(state_path), stored_reminded_at_minutes=stored_reminded_at(state_path),
         note=as_json(baseline.note), printed_at_59_9=bool(before), printed_at_60=bool(at_due))


def scenario_two_watchers(watchdog, clock, run):
    """--discover 与 --processes 两只读同一份落盘的基线（上次提醒在两小时前）：前一只在第 0 分钟告警退出，后一只在第 0.5 分钟定时回报退出。"""
    repository = new_repository(run, "two-watchers", {".claude/main-agent.md": "原文\n"})
    state_path = session_state(run, "two-watchers")
    seed = start_watchdog(watchdog, clock, repository, state_path, -120.0)
    first = start_watchdog(watchdog, clock, repository, state_path, -1.0)
    second = start_watchdog(watchdog, clock, repository, state_path, -0.5)
    first_lines = exit_watchdog(watchdog, clock, first, repository, 0.0, True)
    second_lines = exit_watchdog(watchdog, clock, second, repository, 0.5, False)
    emit("two-watchers", seed_note=as_json(seed.note), first_printed=bool(first_lines), second_printed=bool(second_lines),
         second_claims_minutes_since_last_reminder=claimed_elapsed(second_lines), actual_minutes_since_last_reminder=0.5,
         stored_reminded_at_minutes=stored_reminded_at(state_path))


def scenario_replaced_before_print(watchdog, clock, run):
    """knowledge_sync_section 返回之前就把落盘的基线换成此刻：返回的行没打出去（这一只在打印前被停），点名过的那一份之后谁都不再点。"""
    repository = new_repository(run, "replaced-before-print", {".claude/main-agent.md": "原文\n", ".claude/kb/k.md": "原文\n"})
    state_path = session_state(run, "replaced-before-print")
    baseline = start_watchdog(watchdog, clock, repository, state_path, 0.0)
    write_file(repository, ".claude/kb/k.md", "原文\n改了\n")
    lost_lines = exit_watchdog(watchdog, clock, baseline, repository, 60.0, False)
    replaced_at = stored_reminded_at(state_path)
    following = start_watchdog(watchdog, clock, repository, state_path, 60.5)
    later_lines = exit_watchdog(watchdog, clock, following, repository, 120.5, False)
    emit("replaced-before-print", named_in_unprinted_lines=as_json(named_rows(watchdog, lost_lines)),
         stored_reminded_at_minutes_before_any_print=replaced_at, named_by_next_watchdog=as_json(named_rows(watchdog, later_lines)))


def scenario_killed_inside_replace(watchdog, clock, run):
    """落盘时在 mkstemp 之后、os.replace 之前被停（这里用 os.replace 抛 SystemExit 顶替 SIGKILL：不走 except OSError 那条清理）：临时文件留不留。"""
    repository = new_repository(run, "killed-inside-replace", {".claude/main-agent.md": "原文\n"})
    state_path = session_state(run, "killed-inside-replace")
    real_replace = os.replace

    def killed(*_arguments, **_keywords):
        raise SystemExit("probe: 停在 os.replace 之前")
    os.replace = killed
    try:
        start_watchdog(watchdog, clock, repository, state_path, 0.0)
        outcome = "returned"
    except SystemExit:
        outcome = "stopped"
    finally:
        os.replace = real_replace
    directory = os.path.dirname(state_path)
    leftovers = sorted(name for name in os.listdir(directory) if name.endswith(".tmp"))
    following = start_watchdog(watchdog, clock, repository, state_path, 1.0)
    leftovers_after_next = sorted(name for name in os.listdir(directory) if name.endswith(".tmp"))
    emit("killed-inside-replace", outcome=outcome, leftover_temporary_files=len(leftovers), state_file=os.path.exists(state_path),
         next_note=as_json(following.note), leftover_after_next_watchdog=len(leftovers_after_next))


def scenario_hand_edited_reminded_at(watchdog, clock, run):
    """落盘那一份被手改了 reminded_at（别的字段照原样）：之后的看门狗还提醒不提醒。"""
    repository = new_repository(run, "hand-edited", {".claude/main-agent.md": "原文\n"})
    variants = {"NaN": float("nan"), "Infinity": float("inf"), "year-2100": 4102444800.0, "minus-Infinity": float("-inf"), "string-zero": "0", "null": None}
    for label, value in variants.items():
        state_path = session_state(run, f"hand-edited-{label}")
        start_watchdog(watchdog, clock, repository, state_path, 0.0)
        with open(state_path, encoding="utf-8") as handle:
            record = json.load(handle)
        record["reminded_at"] = value
        with open(state_path, "w", encoding="utf-8") as handle:
            json.dump(record, handle, ensure_ascii=False)
        try:
            baseline = start_watchdog(watchdog, clock, repository, state_path, 1.0)
            lines = exit_watchdog(watchdog, clock, baseline, repository, 600.0, True)
            with open(state_path, encoding="utf-8") as handle:
                after = json.load(handle)["reminded_at"]
            emit("hand-edited-reminded-at", value=label, note=as_json(baseline.note), printed_after_ten_hours=bool(lines),
                 stored_after=as_json(after if after == after and after not in (float("inf"), float("-inf")) else repr(after)))
        except Exception as error:   # 被判对象自己抛的算读数
            emit("hand-edited-reminded-at", value=label, exception=type(error).__name__, detail=as_json(str(error)[:160]))


def scenario_truncated_state(watchdog, clock, run):
    repository = new_repository(run, "truncated", {".claude/main-agent.md": "原文\n", ".claude/kb/k.md": "原文\n"})
    state_path = session_state(run, "truncated")
    start_watchdog(watchdog, clock, repository, state_path, 0.0)
    write_file(repository, ".claude/kb/k.md", "原文\n两只之间改了\n")
    with open(state_path, "rb") as handle:
        content = handle.read()
    with open(state_path, "wb") as handle:
        handle.write(content[: len(content) // 2])
    baseline = start_watchdog(watchdog, clock, repository, state_path, 10.0)
    lines = exit_watchdog(watchdog, clock, baseline, repository, 70.0, False)
    emit("truncated-state", note=as_json(baseline.note), named=as_json(named_rows(watchdog, lines)), printed=bool(lines),
         claims_minutes=claimed_elapsed(lines))


def scenario_unreadable_chains(watchdog, clock, run):
    """git 起不来（AGENT_WATCH_GIT_EXECUTABLE 指到不存在的路径，与自证「git 起不来」那一格同一个法子）一直不好：
    起来时就没查成（会话目录里还没有落盘文件）与先查成过一次、之后才没查成两种，各配一串告警退出。"""
    repository = new_repository(run, "unreadable", {".claude/main-agent.md": "原文\n"})
    missing_git = os.path.join(run, "no-such-git")
    for label, alerts in (("alerts-every-30", [30.0 * k for k in range(1, 7)]), ("alerts-every-59.9", [59.9 * k for k in range(1, 4)]),
                          ("alerts-every-10", [10.0 * k for k in range(1, 19)])):
        state_path = session_state(run, f"unreadable-from-start-{label}")
        os.environ["AGENT_WATCH_GIT_EXECUTABLE"] = missing_git
        try:
            reminders = run_chain(watchdog, clock, repository, state_path, alerts, 0.1, 180.0, "as-is")
            reminders_fix = run_chain(watchdog, clock, repository, session_state(run, f"unreadable-from-start-fix-{label}"), alerts, 0.1, 180.0, "exit-when-due")
        finally:
            os.environ.pop("AGENT_WATCH_GIT_EXECUTABLE", None)
        emit("unreadable-from-start", alerts=label, horizon_minutes=180, sections_printed=len(reminders), state_file=os.path.exists(state_path),
             sections_printed_with_exit_when_due=len(reminders_fix))
        state_path = session_state(run, f"unreadable-later-{label}")
        start_watchdog(watchdog, clock, repository, state_path, -60.0)
        os.environ["AGENT_WATCH_GIT_EXECUTABLE"] = missing_git
        try:
            reminders = run_chain(watchdog, clock, repository, state_path, alerts, 0.1, 180.0, "as-is")
        finally:
            os.environ.pop("AGENT_WATCH_GIT_EXECUTABLE", None)
        emit("unreadable-later", alerts=label, horizon_minutes=180, exits=len([moment for moment in alerts if moment < 180]) + 1,
             sections_printed=len(reminders), reminders=as_json(reminders))


def scenario_agents_without_transcript(watchdog, run):
    """watch.sh <agent 号> 起的 --agents（不带 --session-dir）：被看的子 agent 还没有会话记录时，落盘基线的路径。"""
    path = watchdog.knowledge_state_path(argparse.Namespace(session_dir=None), ["a0123456789abcdef"])
    emit("agents-without-transcript", state_path=as_json(path))


def scenario_other_repository_shared_session(watchdog, clock, run):
    """同一个会话目录里两只看门狗读不同的仓（一只从仓副本里的 watch.sh 起）：一只落的盘被另一只当成「别的仓」重新记、覆盖。"""
    first = new_repository(run, "shared-session-first", {".claude/main-agent.md": "原文\n", ".claude/kb/k.md": "原文\n"})
    second = new_repository(run, "shared-session-second", {".claude/main-agent.md": "原文\n"})
    state_path = session_state(run, "shared-session")
    start_watchdog(watchdog, clock, first, state_path, 0.0)
    write_file(first, ".claude/kb/k.md", "原文\n改了\n")
    other = start_watchdog(watchdog, clock, second, state_path, 10.0)
    exit_watchdog(watchdog, clock, other, second, 20.0, True)
    again = start_watchdog(watchdog, clock, first, state_path, 20.1)
    lines = exit_watchdog(watchdog, clock, again, first, 80.1, False)
    emit("other-repository-shared-session", second_note=as_json(other.note), again_note=as_json(again.note),
         named=as_json(named_rows(watchdog, lines)), claims_minutes=claimed_elapsed(lines))


def scenario_digest_timeout_chains(watchdog, clock, run):
    """算摘要超时（AGENT_WATCH_DIGEST_TIMEOUT_SECONDS=0，与自证「整次摘要超时走没查成」同一个法子；范围里有一份改着的文件才会去算）一直不好：
    起来时就超时与先查成过一次、之后才超时两种，各配一串告警退出。"""
    repository = new_repository(run, "digest-timeout", {".claude/main-agent.md": "原文\n", ".claude/kb/k.md": "原文\n"})
    write_file(repository, ".claude/kb/k.md", "改着\n")
    alerts = [10.0 * k for k in range(1, 19)]
    for kind in ("from-start", "later"):
        state_path = session_state(run, f"digest-timeout-{kind}")
        if kind == "later":
            start_watchdog(watchdog, clock, repository, state_path, -60.0)
        os.environ["AGENT_WATCH_DIGEST_TIMEOUT_SECONDS"] = "0"
        try:
            reminders = run_chain(watchdog, clock, repository, state_path, alerts, 0.1, 180.0, "as-is")
        finally:
            os.environ.pop("AGENT_WATCH_DIGEST_TIMEOUT_SECONDS", None)
        emit("digest-timeout-chain", kind=kind, alerts="alerts-every-10", horizon_minutes=180, sections_printed=len(reminders),
             reminders=as_json(reminders), state_file=os.path.exists(state_path))


# ── 第 2 问：范围里被 git 忽略的条目 ──
def ignored_repository(run, name):
    return new_repository(run, name, {".gitignore": "/.claude/singlefs-ai-sop/\n", ".claude/main-agent.md": "原文\n"})


def scenario_ignored_many(watchdog, clock, run):
    for count in (5000, 50000):
        repository = ignored_repository(run, f"ignored-many-{count}")
        for index in range(count):
            write_file(repository, f"{IGNORED_DIRECTORY}bulk/{index // 1000}/rule-{index}.md", "上游规则\n" * 10)
        state_path = session_state(run, f"ignored-many-{count}")
        started = time.monotonic()
        listed = watchdog.ignored_knowledge_paths(repository)
        listing_seconds = time.monotonic() - started
        started = time.monotonic()
        baseline = start_watchdog(watchdog, clock, repository, state_path, 0.0)
        baseline_seconds = time.monotonic() - started
        emit("ignored-many", files=count, listed=len(listed), listing_seconds=f"{listing_seconds:.2f}", baseline_seconds=f"{baseline_seconds:.2f}",
             unreadable=as_json(baseline.unreadable_reason), state_bytes=os.path.getsize(state_path) if os.path.exists(state_path) else 0)


def scenario_ignored_sparse(watchdog, clock, run):
    repository = ignored_repository(run, "ignored-sparse")
    os.makedirs(os.path.join(repository, IGNORED_DIRECTORY))
    with open(os.path.join(repository, IGNORED_DIRECTORY, "image.md"), "wb") as handle:
        handle.truncate(SPARSE_BYTES)
    started = time.monotonic()
    baseline = start_watchdog(watchdog, clock, repository, session_state(run, "ignored-sparse"), 0.0)
    seconds = time.monotonic() - started
    emit("ignored-sparse", bytes=SPARSE_BYTES, baseline_seconds=f"{seconds:.2f}", unreadable=as_json(baseline.unreadable_reason),
         gib_per_second=f"{SPARSE_BYTES / (1 << 30) / max(seconds, 1e-6):.2f}" if baseline.snapshot is not None else "timed-out")


def run_child(probe_path, watchdog_path, repository, extra_environment=None):
    """起本探针的 --child（自己的进程组），CHILD_SECONDS 秒到点停那一组 → (hung, 秒, 输出)。"""
    started = time.monotonic()
    child = subprocess.Popen([sys.executable, "-B", probe_path, "--child", watchdog_path, repository], stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                             text=True, errors="replace", env=dict(os.environ, **(extra_environment or {})), start_new_session=True)
    try:
        output, _ = child.communicate(timeout=CHILD_SECONDS)
        hung = False
    except subprocess.TimeoutExpired:
        os.killpg(child.pid, signal.SIGKILL)   # 自己用新会话起的那一个子进程，只停它自己那一组
        output, _ = child.communicate()
        hung = True
    return hung, time.monotonic() - started, output


def scenario_ignored_special_files(watchdog, run, probe_path, watchdog_path):
    """被忽略的目录里放命名管道、套接字、指到 /dev/zero 的符号链接与一份普通文件：列不列、卡不卡。"""
    repository = ignored_repository(run, "ignored-special")
    write_file(repository, IGNORED_DIRECTORY + "plain.md", "规则\n")
    os.mkfifo(os.path.join(repository, IGNORED_DIRECTORY, "channel.md"))
    os.symlink("/dev/zero", os.path.join(repository, IGNORED_DIRECTORY, "zero.md"))
    listener = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    socket_path = os.path.join(repository, IGNORED_DIRECTORY, "s.md")
    previous_directory = os.getcwd()
    try:
        os.chdir(os.path.dirname(socket_path))   # AF_UNIX 的路径上限约 108 字节：进那个目录按相对路径绑
        listener.bind(os.path.basename(socket_path))
    except OSError as error:
        print(f"  · ignored-special 没绑上套接字（{error}），这一格只看命名管道与符号链接", flush=True)
    finally:
        os.chdir(previous_directory)
    try:
        hung, seconds, output = run_child(probe_path, watchdog_path, repository)
    finally:
        listener.close()
    emit("ignored-special-files", hung=hung, seconds=f"{seconds:.2f}", child=as_json(output.strip().splitlines()[-1:] if output else []))


def scenario_ignored_transitions(watchdog, clock, run):
    """不改内容、只改忽略规则：被忽略的整个目录不再被忽略；未跟踪的知识类文件新被忽略。点名与标签。"""
    repository = ignored_repository(run, "ignored-unignored")
    for name in ("a.md", "b.md"):
        write_file(repository, IGNORED_DIRECTORY + name, "上游规则\n")
    baseline = start_watchdog(watchdog, clock, repository, None, 0.0)
    write_file(repository, ".gitignore", "")
    lines = exit_watchdog(watchdog, clock, baseline, repository, 60.0, False)
    emit("ignored-to-untracked", named=as_json(named_rows(watchdog, lines)),
         other=as_json([line.strip() for line in lines if line.startswith("    新出现") or line.startswith("    不在了") or line.startswith("    状态变了")]))
    repository = new_repository(run, "untracked-to-ignored", {".claude/main-agent.md": "原文\n", ".claude/kb/tracked.md": "原文\n"})
    write_file(repository, ".claude/kb/notes.md", "草稿\n")
    baseline = start_watchdog(watchdog, clock, repository, None, 0.0)
    write_file(repository, ".git/info/exclude", "/.claude/kb/notes.md\n")
    lines = exit_watchdog(watchdog, clock, baseline, repository, 60.0, False)
    emit("untracked-to-ignored", named=as_json(named_rows(watchdog, lines)))


def run_copied_command(command, directory):
    """一条给人抄的命令原样写进一份脚本交给 bash（与自证 run_selftest_shell_command 同一个法子），自己的进程组、60 秒到点停那一组 → (退出码, 标准输出)。"""
    script_directory = os.path.join(directory, f"copied-{hashlib.sha256(command.encode('utf-8', 'surrogateescape')).hexdigest()[:12]}")
    os.makedirs(script_directory, exist_ok=True)
    script = os.path.join(script_directory, "manual-command.sh")
    with open(script, "w", encoding="utf-8", errors="surrogateescape") as handle:
        handle.write(command + "\n")
    shell = subprocess.Popen(["bash", script], cwd=directory, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, errors="replace",
                             env=dict(os.environ, **NEUTRAL_GIT_CONFIG), start_new_session=True)
    try:
        output, error_output = shell.communicate(timeout=60)
    except subprocess.TimeoutExpired:
        os.killpg(shell.pid, signal.SIGKILL)   # 自己用新会话起的那一个子进程，只停它自己那一组
        shell.communicate()
        return "timeout", ""
    return shell.returncode, output + error_output


def copied_commands(watchdog, lines):
    return [line[len(watchdog.COMMAND_INDENT):] for line in lines if line.startswith(watchdog.COMMAND_INDENT + "git ")]


def scenario_timeout_commands_miss_ignored(watchdog, clock, run):
    """被忽略的规则改了、到点时算摘要超时：「没查成」下面给的两条命令照抄跑，看得见那一份吗。"""
    repository = ignored_repository(run, "timeout-commands")
    write_file(repository, IGNORED_DIRECTORY + "rule.md", "上游规则\n")
    baseline = start_watchdog(watchdog, clock, repository, None, 0.0)
    write_file(repository, IGNORED_DIRECTORY + "rule.md", "上游规则改了\n")
    os.environ["AGENT_WATCH_DIGEST_TIMEOUT_SECONDS"] = "0"
    try:
        lines = exit_watchdog(watchdog, clock, baseline, repository, 60.0, False)
    finally:
        os.environ.pop("AGENT_WATCH_DIGEST_TIMEOUT_SECONDS", None)
    commands = copied_commands(watchdog, lines)
    readings = [run_copied_command(command, run) for command in commands]
    emit("timeout-commands-miss-ignored", unreadable_line=as_json(next((line for line in lines if "没查成：" in line), "")),
         commands=len(commands), exits=as_json([code for code, _ in readings]),
         any_output_mentions_rule=any("rule.md" in output for _, output in readings))


# ── 第 3 问：porcelain 加 -z ──
def scenario_porcelain_configs(watchdog, run):
    """同一串改动在几种用户 git 配置下读出的 git status（路径元组 → 状态码）与点名是不是逐条相同。改动里有一条「改了源文件又复制一份暂存」。"""
    def reading(label, config_text):
        repository = new_repository(run, f"porcelain-{label}", {".claude/main-agent.md": "原文\n", ".claude/rules/source.md": "一条规则\n" * 30,
                                                                  ".claude/kb/k.md": "原文\n", "other.txt": "别的\n"})
        environment_saved = os.environ.get("GIT_CONFIG_GLOBAL")
        if config_text is not None:
            config_path = os.path.join(run, f"porcelain-{label}.gitconfig")
            with open(config_path, "w", encoding="utf-8") as handle:
                handle.write(config_text)
            os.environ["GIT_CONFIG_GLOBAL"] = config_path
        try:
            baseline = watchdog.take_repository_baseline(arguments_of(repository), EPOCH, None)
            write_file(repository, "other.txt", "暂存起来的\n")
            git(repository, "stash", "push", "-q", "--", "other.txt")
            write_file(repository, ".claude/kb/k.md", "原文\n改了\n")
            write_file(repository, ".claude/rules/source.md", "一条规则\n" * 30 + "加一行\n")
            write_file(repository, ".claude/rules/copy.md", "一条规则\n" * 30)
            git(repository, "add", "--", ".claude/rules/source.md", ".claude/rules/copy.md")
            write_file(repository, ".claude/kb/new.md", "新页\n")
            snapshot = watchdog.repository_snapshot(repository)
            lines = watchdog.repository_status_section(baseline, arguments_of(repository))[0]
        finally:
            if environment_saved is None:
                os.environ.pop("GIT_CONFIG_GLOBAL", None)
            else:
                os.environ["GIT_CONFIG_GLOBAL"] = environment_saved
        status = sorted([list(key), code] for key, code in snapshot.status_by_path.items())
        return status, sorted(named_rows(watchdog, lines)), list(watchdog.status_counts(snapshot.status_by_path))
    reference = reading("reference", None)
    emit("porcelain-reference", status=as_json(reference[0]), named=as_json(reference[1]), counts=as_json(reference[2]))
    configs = {"status-renames-copies": "[status]\n\trenames = copies\n", "diff-renames-copies": "[diff]\n\trenames = copies\n",
               "status-showStash": "[status]\n\tshowStash = true\n", "status-submoduleSummary": "[status]\n\tsubmoduleSummary = true\n",
               "status-aheadBehind-false": "[status]\n\taheadBehind = false\n", "status-relativePaths-false": "[status]\n\trelativePaths = false\n",
               "core-quotepath-true": "[core]\n\tquotepath = true\n", "status-branch": "[status]\n\tbranch = true\n",
               "color-ui-always": "[color]\n\tui = always\n", "status-displayCommentPrefix": "[status]\n\tdisplayCommentPrefix = true\n"}
    for label, config_text in configs.items():
        try:
            status, named, counts = reading(label, config_text)
            emit("porcelain-config", config=label, same_status=status == reference[0], same_named=named == reference[1], same_counts=counts == reference[2],
                 status=as_json(status) if status != reference[0] else "same", named=as_json(named) if named != reference[1] else "same",
                 counts=as_json(counts))
        except Exception as error:
            emit("porcelain-config", config=label, exception=type(error).__name__, detail=as_json(str(error)[:200]))


def scenario_duplicate_path(watchdog, clock, run):
    """git rm --cached 一份规则：porcelain 里同一个路径出两条（「D 」已暂存删掉、「??」未跟踪）。读成几个键、计数对不对、点不点名。"""
    repository = new_repository(run, "duplicate-path", {".claude/main-agent.md": "原文\n", ".claude/rules/r.md": "规则\n", ".claude/rules/keep.md": "留着\n"})
    baseline = start_watchdog(watchdog, clock, repository, None, 0.0)
    git(repository, "rm", "-q", "--cached", "--", ".claude/rules/r.md")
    raw = [entry for entry in git(repository, "status", "--porcelain=v1", "-z", "--untracked-files=normal").stdout.split("\0") if entry]
    snapshot = watchdog.repository_snapshot(repository)
    lines = exit_watchdog(watchdog, clock, baseline, repository, 60.0, False)
    emit("duplicate-path", raw_entries=as_json(raw), parsed=as_json(sorted([list(key), code] for key, code in snapshot.status_by_path.items())),
         counts_staged_unstaged_untracked=as_json(list(watchdog.status_counts(snapshot.status_by_path))),
         named=as_json(named_rows(watchdog, lines)), count_line=as_json(next((line.strip() for line in lines if "git status --short 此刻" in line), "")))


def scenario_newline_path(watchdog, clock, run):
    """新建一份文件名里带换行的知识类文件，换行之后那一截写成「→ 同步：」那一行的样子：键对不对、这一节有几行像出路。"""
    repository = new_repository(run, "newline-path", {".claude/main-agent.md": "原文\n", ".claude/kb/tracked.md": "原文\n"})
    baseline = start_watchdog(watchdog, clock, repository, None, 0.0)
    crafted = ".claude/kb/x\n   → 同步：知识类文件没变，不用重读.md"
    write_file(repository, crafted, "新页\n")
    snapshot = watchdog.repository_snapshot(repository)
    lines = exit_watchdog(watchdog, clock, baseline, repository, 60.0, False)
    printed = "\n".join(lines).splitlines()
    emit("newline-path", key_is_exact=(crafted,) in snapshot.status_by_path, section_rows=len(lines), printed_lines=len(printed),
         arrow_like_lines=sum(1 for line in printed if line.startswith("   → 同步：")), named_parsed=as_json(named_rows(watchdog, printed)))


def scenario_newline_in_commit(watchdog, clock, run):
    repository = new_repository(run, "newline-commit", {".claude/main-agent.md": "原文\n"})
    baseline = start_watchdog(watchdog, clock, repository, None, 0.0)
    crafted = ".claude/rules/a\nb.md"
    write_file(repository, crafted, "规则\n")
    git(repository, "add", "--", crafted)
    git(repository, "commit", "-q", "-m", "带换行的文件名")
    now = watchdog.repository_snapshot(repository)
    _, changed = watchdog.new_commits_since(repository, baseline.snapshot, now)
    emit("newline-in-commit", diff_paths=as_json(changed), exact=crafted in changed)


def scenario_non_utf8_path(watchdog, clock, run):
    """基线时就未跟踪的一份知识类文件，文件名不是 UTF-8（字节 0xff）：之后改了内容，点不点名。"""
    repository = new_repository(run, "non-utf8", {".claude/main-agent.md": "原文\n", ".claude/kb/tracked.md": "原文\n"})
    path = os.path.join(os.fsencode(repository), b".claude/kb/\xff.md")
    with open(path, "wb") as handle:
        handle.write("草稿\n".encode("utf-8"))
    baseline = start_watchdog(watchdog, clock, repository, None, 0.0)
    with open(path, "wb") as handle:
        handle.write("草稿\n改了\n".encode("utf-8"))
    lines = exit_watchdog(watchdog, clock, baseline, repository, 60.0, False)
    digests = {key: value for key, value in (baseline.snapshot.digest_by_path if baseline.snapshot else {}).items() if key.startswith(".claude/kb/")}
    emit("non-utf8-path", baseline_digests=as_json(digests), named=as_json(named_rows(watchdog, lines)),
         reminder=as_json(lines[0][-40:] if lines else ""))


# ── 第 4 问：摘要没变就不点名 ──
def scenario_index_only(watchdog, clock, run):
    """基线时已暂存（索引与工作区同一份内容）；之后只往索引里换一份别的内容（git update-index --cacheinfo），工作区一字没动。"""
    repository = new_repository(run, "index-only", {".claude/main-agent.md": "原文\n", ".claude/rules/r.md": "原文\n"})
    write_file(repository, ".claude/rules/r.md", "基线时已暂存\n")
    git(repository, "add", "--", ".claude/rules/r.md")
    baseline = start_watchdog(watchdog, clock, repository, None, 0.0)
    blob = subprocess.run(["git", "-C", repository, "hash-object", "-w", "--stdin"], input="索引里换了一份\n", capture_output=True, text=True,
                          env=dict(os.environ, **NEUTRAL_GIT_CONFIG), timeout=60, check=True).stdout.strip()
    git(repository, "update-index", "--cacheinfo", f"100644,{blob},.claude/rules/r.md")
    now_code = watchdog.codes_by_path(watchdog.repository_snapshot(repository).status_by_path).get(".claude/rules/r.md")
    lines = exit_watchdog(watchdog, clock, baseline, repository, 60.0, False)
    emit("index-only", now_code=as_json(now_code), named=as_json(named_rows(watchdog, lines)))


def scenario_changed_and_back(watchdog, clock, run):
    repository = new_repository(run, "changed-and-back", {".claude/main-agent.md": "原文\n", ".claude/kb/k.md": "原文\n"})
    write_file(repository, ".claude/kb/k.md", "基线时已改\n")
    baseline = start_watchdog(watchdog, clock, repository, None, 0.0)
    write_file(repository, ".claude/kb/k.md", "中间改过一次\n")
    write_file(repository, ".claude/kb/k.md", "基线时已改\n")
    lines = exit_watchdog(watchdog, clock, baseline, repository, 60.0, False)
    emit("changed-and-back", named=as_json(named_rows(watchdog, lines)))


def scenario_links_in_untracked_directory(watchdog, clock, run):
    """基线时就未跟踪的 .claude/kb/drafts/（git status 写成一条）里有一个指到目录的符号链接、一个指到文件的符号链接：之后各改指向。"""
    readings = {}
    for kind in ("directory-link", "file-link"):
        repository = new_repository(run, f"links-{kind}", {".claude/main-agent.md": "原文\n", "first/a.md": "甲\n", "second/a.md": "乙\n",
                                                           "first.md": "甲\n", "second.md": "乙\n", ".claude/kb/tracked.md": "原文\n"})
        write_file(repository, ".claude/kb/drafts/notes.md", "草稿\n")
        link = os.path.join(repository, ".claude/kb/drafts/ref")
        os.symlink("../../../first" if kind == "directory-link" else "../../../first.md", link)
        baseline = start_watchdog(watchdog, clock, repository, None, 0.0)
        os.remove(link)
        os.symlink("../../../second" if kind == "directory-link" else "../../../second.md", link)
        lines = exit_watchdog(watchdog, clock, baseline, repository, 60.0, False)
        readings[kind] = named_rows(watchdog, lines)
    emit("links-in-untracked-directory", directory_link_named=as_json(readings["directory-link"]), file_link_named=as_json(readings["file-link"]))


def scenario_unreadable_file(watchdog, clock, run):
    """基线时就未跟踪、权限 000 的一份知识类文件：之后内容改了（权限照旧 000）。"""
    repository = new_repository(run, "unreadable-file", {".claude/main-agent.md": "原文\n", ".claude/kb/tracked.md": "原文\n"})
    path = os.path.join(repository, ".claude/kb/locked.md")
    write_file(repository, ".claude/kb/locked.md", "草稿\n")
    os.chmod(path, 0)
    baseline = start_watchdog(watchdog, clock, repository, None, 0.0)
    os.chmod(path, 0o600)
    write_file(repository, ".claude/kb/locked.md", "草稿\n改了\n")
    os.chmod(path, 0)
    lines = exit_watchdog(watchdog, clock, baseline, repository, 60.0, False)
    os.chmod(path, 0o600)
    emit("unreadable-file", baseline_digest=as_json((baseline.snapshot.digest_by_path if baseline.snapshot else {}).get(".claude/kb/locked.md")),
         named=as_json(named_rows(watchdog, lines)), running_as_root=os.geteuid() == 0)


# ── 第 5 问：给人抄的命令 ──
def scenario_special_root_commands(watchdog, clock, run):
    """仓的路径里带单引号、$、反引号与空格：没查成那一节与清单超过上限时给的命令，原样写进脚本交给 bash 跑。"""
    parent = os.path.join(run, "q'uote $HOME `echo injected` dir")
    os.makedirs(parent)
    repository = new_repository(parent, "repo", {".claude/main-agent.md": "原文\n", ".claude/kb/old.md": "原文\n"})
    unread = watchdog.repository_status_section(watchdog.RepositoryBaseline(repository, None, "探针造的：起来时没读到", None, clock.at(0.0), None),
                                               arguments_of(repository))[0]
    baseline = watchdog.take_repository_baseline(arguments_of(repository, 1), clock.at(0.0), None)
    for index in range(2):
        write_file(repository, f".claude/kb/page-{index}.md", "页\n")
        write_file(repository, f"x-{index}.txt", "x\n")
        git(repository, "add", "--", f".claude/kb/page-{index}.md", f"x-{index}.txt")
        git(repository, "commit", "-q", "-m", f"新提交 {index}")
    for name in ("y.txt", "z.txt"):
        write_file(repository, name, "新出现\n")
    capped = watchdog.repository_status_section(baseline, arguments_of(repository, 1))[0]
    commands = copied_commands(watchdog, unread + capped)
    readings = [run_copied_command(command, run) for command in commands]
    emit("special-root-commands", commands=len(commands), exits=as_json([code for code, _ in readings]),
         injected_marker_seen=any("injected" in output and "echo injected" not in output for _, output in readings),
         who_touched_line=as_json(next((line for line in capped if "who-touched.py" in line), "")[-60:]))


# ── 第 7 问：命名管道 ──
def scenario_fifo_under_tracked_claude(run, probe_path, watchdog_path):
    """与自证「未跟踪知识类目录里有命名管道」同一个形状：被跟踪的 .claude/ 底下未跟踪的 .claude/kb/pipes/，里面一份普通文件与一个没有写端的命名管道。
    照常与弄坏开关 knowledgefiletypeunchecked 下各记一次基线。"""
    repository = new_repository(run, "fifo-under-tracked-claude", {".claude/main-agent.md": "原文\n"})
    write_file(repository, ".claude/kb/pipes/notes.md", "草稿\n")
    os.mkfifo(os.path.join(repository, ".claude/kb/pipes/channel"))
    status = git(repository, "status", "--porcelain=v1", "-z", "--untracked-files=normal").stdout.split("\0")
    for label, environment in (("as-is", {}), ("knowledgefiletypeunchecked", {"AGENT_WATCH_BREAK": "knowledgefiletypeunchecked"})):
        hung, seconds, output = run_child(probe_path, watchdog_path, repository, environment)
        emit("fifo-under-tracked-claude", variant=label, status=as_json([entry for entry in status if entry]), hung=hung, seconds=f"{seconds:.2f}",
             child=as_json(output.strip().splitlines()[-1:] if output else []))


def child_main(watchdog_path, repository):
    watchdog = load_watchdog(watchdog_path)
    baseline = watchdog.take_repository_baseline(arguments_of(repository), time.time(), None)
    snapshot = baseline.snapshot
    print("child-finished " + as_json({"unreadable": baseline.unreadable_reason,
                                        "digests": snapshot.digest_by_path if snapshot else None,
                                        "ignored": snapshot.ignored_digest_by_path if snapshot else None}), flush=True)
    return 0


def main(argv):
    if len(argv) == 4 and argv[1] == "--child":
        return child_main(argv[2], argv[3])
    if len(argv) != 3:
        print(__doc__, file=sys.stderr)
        return 2
    watchdog_path, run = os.path.abspath(argv[1]), os.path.abspath(argv[2])
    if not os.path.isfile(watchdog_path) or os.path.exists(run):
        print(f"✗ 要给在的 agent-watch.py 与一个还不在的运行目录：{watchdog_path}、{run}", file=sys.stderr)
        return 2
    os.makedirs(run)
    fake_home = os.path.join(run, "fake-home")
    os.makedirs(fake_home)
    os.environ.update(NEUTRAL_GIT_CONFIG)
    os.environ.update({"HOME": fake_home, "GIT_CEILING_DIRECTORIES": run, "AGENT_WATCH_STATE_ROOT": os.path.join(run, "watch-state"), "AGENT_WATCH_PROJECTS_ROOT": os.path.join(fake_home, "projects"),
                       "PYTHONDONTWRITEBYTECODE": "1"})
    for name in ("TASK_SERVER_SOURCE_REPOSITORY", "AGENT_WATCH_BREAK", "AGENT_WATCH_GIT_EXECUTABLE", "AGENT_WATCH_GIT_TIMEOUT_SECONDS",
                 "AGENT_WATCH_DIGEST_TIMEOUT_SECONDS", "AGENT_WATCH_REPOSITORY_ROOT", "GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_COMMON_DIR"):
        os.environ.pop(name, None)
    with open(watchdog_path, "rb") as handle:
        emit("watchdog", path=watchdog_path, sha256=hashlib.sha256(handle.read()).hexdigest())
    watchdog = load_watchdog(watchdog_path)
    clock = FakeClock()
    watchdog.time = clock
    probe_path = os.path.abspath(__file__)
    scenarios = [
        ("first-start", lambda: scenario_first_start(watchdog, clock, run)),
        ("exit-chains", lambda: scenario_exit_chains(watchdog, clock, run)),
        ("two-watchers", lambda: scenario_two_watchers(watchdog, clock, run)),
        ("replaced-before-print", lambda: scenario_replaced_before_print(watchdog, clock, run)),
        ("killed-inside-replace", lambda: scenario_killed_inside_replace(watchdog, clock, run)),
        ("hand-edited", lambda: scenario_hand_edited_reminded_at(watchdog, clock, run)),
        ("truncated", lambda: scenario_truncated_state(watchdog, clock, run)),
        ("unreadable-chains", lambda: scenario_unreadable_chains(watchdog, clock, run)),
        ("digest-timeout-chains", lambda: scenario_digest_timeout_chains(watchdog, clock, run)),
        ("agents-without-transcript", lambda: scenario_agents_without_transcript(watchdog, run)),
        ("other-repository", lambda: scenario_other_repository_shared_session(watchdog, clock, run)),
        ("ignored-many", lambda: scenario_ignored_many(watchdog, clock, run)),
        ("ignored-sparse", lambda: scenario_ignored_sparse(watchdog, clock, run)),
        ("ignored-special", lambda: scenario_ignored_special_files(watchdog, run, probe_path, watchdog_path)),
        ("ignored-transitions", lambda: scenario_ignored_transitions(watchdog, clock, run)),
        ("timeout-commands", lambda: scenario_timeout_commands_miss_ignored(watchdog, clock, run)),
        ("porcelain-configs", lambda: scenario_porcelain_configs(watchdog, run)),
        ("duplicate-path", lambda: scenario_duplicate_path(watchdog, clock, run)),
        ("newline-path", lambda: scenario_newline_path(watchdog, clock, run)),
        ("newline-in-commit", lambda: scenario_newline_in_commit(watchdog, clock, run)),
        ("non-utf8-path", lambda: scenario_non_utf8_path(watchdog, clock, run)),
        ("index-only", lambda: scenario_index_only(watchdog, clock, run)),
        ("changed-and-back", lambda: scenario_changed_and_back(watchdog, clock, run)),
        ("links", lambda: scenario_links_in_untracked_directory(watchdog, clock, run)),
        ("unreadable-file", lambda: scenario_unreadable_file(watchdog, clock, run)),
        ("special-root-commands", lambda: scenario_special_root_commands(watchdog, clock, run)),
        ("fifo-under-tracked-claude", lambda: scenario_fifo_under_tracked_claude(run, probe_path, watchdog_path)),
    ]
    for label, scenario in scenarios:
        started = time.monotonic()
        try:
            scenario()
        except Exception as error:   # 探针自己搭场景失败（或被判对象抛了没被那一格接住的异常）：记下、接着跑下一格
            setup_failures.append(f"{label}：{type(error).__name__}：{error}")
            print(f"  ! {label} 没跑完：{type(error).__name__}：{str(error)[:300]}", flush=True)
        print(f"  · {label} 用时 {time.monotonic() - started:.1f} 秒", flush=True)
    sparse = os.path.join(run, "ignored-sparse", IGNORED_DIRECTORY, "image.md")
    if os.path.exists(sparse):
        os.remove(sparse)   # 稀疏文件不留给复跑之后的人
    print(f"PROBE name=done emitted={emitted_lines} setup_failures={len(setup_failures)}", flush=True)
    return 1 if setup_failures else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
