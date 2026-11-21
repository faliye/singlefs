"""task_server_selftest.py 拆出的一份：判卡住、疑似卡住、失控进程、留下的进程、读不到单元状态的各格（总说明在 task_server_selftest.py 文件头）。"""
import os
import sys
import contextlib
import json
import re
import shlex
import shutil
import signal
import subprocess
import tempfile
import threading
import time
import types
import task_server_lib as lib
from task_server_selftest_fakes import (
    AT_CAP_PEAK_BYTES, AT_CAP_SEED_PEAK_BYTES, BLOCK_SCRIPT, GIBIBYTE, HOLD_UNTIL_RELEASED_SCRIPT, PLACEMENT_WAIT_SECONDS, POLL_SECONDS,
    SELFTEST_PEER_ADDRESS, SERVER, SESSION_A, SESSION_B, STATE_TIMEOUT_SECONDS,
)
from task_server_selftest_world import (
    ATTENTION_QUIET_SECONDS, ATTENTION_WAIT_SECONDS, FakeInbox, INBOX_QUIET_SECONDS, INBOX_TOKEN_SAMPLE, STARVED_HEALTH_TEXT, SelftestFailure,
    append_lines, attempt_has_run_for, attention_fields, attention_lines, attention_problems, blocking_command, describe, first_attempt,
    health_until, inbox_problems, peer_log_scheduler, process_gone, process_memory_stat, raise_cell_problems, read_text, reminder_head,
    reminders_after, restart_surviving_scheduler, start_inbox_server, wait_path, write_executable,
)


GRACE_CELL_STALL_SECONDS = 2              # stall-waits-for-grace：卡住阈值；宽限里判到静了它的两倍才放行
GRACE_CELL_GRACE_SECONDS = 20             # stall-waits-for-grace：宽限只要比「判到静了两倍阈值、放行、命令退出」这一下长

LOG_QUIET_PINNED_SECONDS = 3000   # stall-clock-from-log-mtime 把在跑那一条的日志修改时刻设到这么多秒以前

FALLBACK_TEMPORARY_BASE = "/tmp"


def cell_stalled_attempt_stopped(world):
    scope = os.path.join(world.root, "idle-scope")   # CPU 读得到、一直不涨
    os.makedirs(scope)
    with open(os.path.join(scope, "cpu.stat"), "w", encoding="utf-8") as handle:
        handle.write("usage_usec 1000\n")
    world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01",
                              "TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope})
    quiet = world.finished(world.submit(["bash", "-c", "echo started; sleep 30"]), timeout=40)
    chatty = world.finished(world.submit(["bash", "-c", "for i in 1 2 3 4 5 6; do echo tick $i; sleep 0.5; done"]), timeout=40)
    if quiet["task"].get("outcome") != "stalled" or chatty["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"日志 2 秒没涨的那一条应当被停、记 stalled，每半秒打一行的照跑完；实际不出声的 {describe(quiet)}、一直打的 {describe(chatty)}",
                              world.server_log_lines())
    return "日志满阈值没涨的那一条服务停掉、记 stalled；一直有输出的照跑完"


def cell_computing_quietly_not_stalled(world):
    outer, scope = os.path.join(world.root, "outer-scope"), os.path.join(world.root, "inner-scope")   # 外层不动，里层包装开的平级 scope 在涨
    for directory in (outer, scope):
        os.makedirs(directory)
        with open(os.path.join(directory, "cpu.stat"), "w", encoding="utf-8") as handle:
            handle.write("usage_usec 0\n")
    stat = os.path.join(scope, "cpu.stat")
    world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01", "TASK_SERVER_FAKE_SCOPE_DIRECTORY": outer + os.pathsep + scope})
    stop = threading.Event()

    def burn():   # 假 scope 的 CPU 每 0.2 秒涨 0.2 秒（一个核在算）
        used = 0
        while not stop.is_set():
            used += 200000
            with open(stat + ".partial", "w", encoding="utf-8") as handle:
                handle.write(f"usage_usec {used}\n")
            os.replace(stat + ".partial", stat)
            time.sleep(0.2)

    burner = threading.Thread(target=burn, daemon=True)
    burner.start()
    try:
        summary = world.finished(world.submit(["bash", "-c", "echo started; sleep 7"]), timeout=40)
    finally:
        stop.set()
    if summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"日志 7 秒不涨（阈值 2 秒）而 CPU 一直在涨：在算，应当照跑完，实际 {describe(summary)}", world.server_log_lines())
    return "日志过了阈值不涨而 CPU 一直在涨的照跑完（在算不算卡住）"


PEER_STALL_SECONDS = 2        # 第二台卡住判法那几格：卡住阈值（TASK_SERVER_SELFTEST_STALL_SECONDS）
PEER_HELD_WINDOWS = 4         # 第二台那条该跑完的命令：服务记的起跑时刻起，在跑满这么多个卡住窗口之后才放行（卡住的那几条被服务停了才算服务判过窗口）
PEER_HOLD_ROUNDS = 400        # 第二台那几条命令最多等放行这么多轮（每轮 0.5 秒），到了不当放行、退 9
PEER_STOP_QUIET_SECONDS = 30.0   # 第二台那几格：该被服务停的那几条等服务停它的安静时限（它们的日志一直在涨，世界一直在变，绝对上限是它的 WAIT_CEILING_FACTOR 倍）


def peer_hold_script(release, *round_lines):
    """第二台那几条命令的正文：每轮 0.5 秒、每轮打 round_lines（shell 命令，可用循环号 $round），放行文件在了就退 0；服务停它的那几条在放行之前就被停了。"""
    return (f'for round in $(seq {PEER_HOLD_ROUNDS}); do [[ -e {shlex.quote(release)} ]] && exit 0; ' + "; ".join(round_lines) + "; sleep 0.5; done; exit 9")


def cell_peer_stall_judged_by_peer_cpu(world):
    """第二台的尝试判卡住读第二台的 CPU（日志里最近一行 MULTI_HOST_RUN_PEER_CPU）：CPU 在涨的照跑完，CPU 不涨、只打 MULTI_HOST_RUN_PEER_CPU 与 STEP 行的记 stalled。
    两条都等放行文件才退，不靠命令自己跑完：CPU 不涨的那一条由服务停（记 stalled），服务停了它，说明服务判过了卡住窗口；在涨的那一条再跑满 PEER_HELD_WINDOWS 个窗口
    （按服务记的起跑时刻）才放行，这之间服务判过它几个窗口，被停就记 stalled。
    弄坏开关 cpu-ignored-for-stall 下 CPU 在涨的那一条也被停、记 stalled，stall-ignored 下 CPU 不涨的那一条不被停，这一格红。"""
    world.resources(local=(8, 30), peer=(100, 100), peer_cap_gibibytes=64)
    world.seed_peak("selftest-peer-stall", 10 * GIBIBYTE)
    world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": str(PEER_STALL_SECONDS), "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01"})
    release = os.path.join(world.root, "release-peer-stall")
    busy = "sleep 1; " + peer_hold_script(release, 'echo "MULTI_HOST_RUN_PEER_CPU poll=$round usage_usec=$((round*1000000)) read=yes"')   # 头 1 秒还没有 CPU 行（演第二台在拷树；短于阈值）
    idle = peer_hold_script(release, 'echo "MULTI_HOST_RUN_PEER_CPU poll=$round usage_usec=1000 read=yes"', 'echo "MULTI_HOST_RUN_STEP step=peer-command event=poll"')
    summaries = {}
    try:
        task_ids = {name: world.submit(["bash", "-c", script, name, "fake-peer-run"], extra={lib.KEY_VARIABLE: "selftest-peer-stall"}, cwd=world.repo)
                    for name, script in (("busy", busy), ("idle", idle))}
        summaries["idle"] = world.finished(task_ids["idle"], timeout=PEER_STOP_QUIET_SECONDS)
        world.wait_for(task_ids["busy"], lambda summary: summary["task"].get("state") == "finished" or attempt_has_run_for(summary, PEER_HELD_WINDOWS * PEER_STALL_SECONDS),
                       f"在第二台跑满 {PEER_HELD_WINDOWS} 个卡住窗口（或被停）")
    finally:
        open(release, "w").close()
    summaries["busy"] = world.finished(task_ids["busy"])
    hosts = {name: (summary["attempts"][0].get("host") if summary["attempts"] else None) for name, summary in summaries.items()}
    if hosts != {"busy": "peer", "idle": "peer"} or summaries["busy"]["task"].get("outcome") != "succeeded" or summaries["idle"]["task"].get("outcome") != "stalled":
        raise SelftestFailure(f"第二台的两条：只打 CPU 行且 CPU 在涨的应当照跑完，只打 CPU 不涨的行与 STEP 行的应当记 stalled；"
                              f"实际台 {hosts}、在涨的 {describe(summaries['busy'])}、不涨的 {describe(summaries['idle'])}", world.server_log_lines())
    return "第二台的尝试：MULTI_HOST_RUN_PEER_CPU 与 STEP 行不算日志在涨，CPU 在涨的照跑完、不涨的记 stalled"


def cell_stall_waits_for_grace(world):
    """起来后宽限里不判卡住：日志与 CPU 都不涨的那一条，服务在宽限里判过、日志已静了两个卡住阈值那么久时才放行它，它照跑完。
    放行看服务记的「宽限里判过一次」（elapsed 是从起尝试算的秒数），不看挂钟；宽限只要比放行那一下长就够。"""
    scope = os.path.join(world.root, "idle-scope")   # CPU 读得到、一直不涨：没有宽限就会被判卡住
    os.makedirs(scope)
    with open(os.path.join(scope, "cpu.stat"), "w", encoding="utf-8") as handle:
        handle.write("usage_usec 1000\n")
    world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": str(GRACE_CELL_STALL_SECONDS), "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": str(GRACE_CELL_GRACE_SECONDS),
                              "TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope})
    release = os.path.join(world.root, "release")
    task_id = world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "quiet", release])
    quiet_for = 2 * GRACE_CELL_STALL_SECONDS

    def judged_quiet_in_grace():
        for line in world.scheduler_reads():
            found = re.match(rf"^stall-grace task={task_id} attempt=1 elapsed=([0-9.]+)$", line)
            if found and float(found.group(1)) >= quiet_for:
                return True
        return False

    try:
        world.wait_for(task_id, lambda found: found["task"].get("state") == "finished" or judged_quiet_in_grace(), f"在宽限里判到静了 {quiet_for} 秒")
    finally:
        open(release, "w").close()
    summary = world.finished(task_id)
    if summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"起来后宽限 {GRACE_CELL_GRACE_SECONDS} 秒不判：日志与 CPU 静了 {quiet_for} 秒（卡住阈值 {GRACE_CELL_STALL_SECONDS} 秒的两倍）还在宽限里的那一条应当照跑完，"
                              f"实际 {describe(summary)}", world.server_log_lines())
    return f"起来后宽限里日志与 CPU 静了卡住阈值两倍那么久的照跑完（宽限过了才量窗口）"


def cell_running_status_shows_progress(world):
    key = "selftest-progress"
    release = os.path.join(world.root, "release")
    command = ["bash", "-c", BLOCK_SCRIPT, "block", release]
    world.seed_duration(key, 3, command)
    world.start_server()
    task_id = world.submit(command, extra={lib.KEY_VARIABLE: key})
    world.running(task_id)
    time.sleep(1)
    shown = world.client(["status", str(task_id)])
    open(release, "w").close()
    world.finished(task_id)
    if "在跑：已跑" not in shown.stdout or "没涨" not in shown.stdout:
        raise SelftestFailure(f"在跑的任务 status 应当报已跑多久、日志多久没涨、在用核数，实际「{shown.stdout.strip()[:400]}」")
    return "在跑的任务 status 报已跑多久、日志多久没涨与在用核数"


def cell_attempt_unit_nofile_raised(world):
    import resource
    hard = resource.getrlimit(resource.RLIMIT_NOFILE)[1]
    wanted = "unlimited" if hard == resource.RLIM_INFINITY else str(hard)
    world.resources(local=(30, 30))
    world.start_server()
    summary = world.finished(world.submit(["bash", "-c", "echo NOFILE=$(ulimit -Sn):$(ulimit -Hn)"]))
    log = summary["attempts"][0].get("log", "") if summary["attempts"] else ""
    text = open(log, encoding="utf-8", errors="replace").read() if log and os.path.exists(log) else ""
    if f"NOFILE={wanted}:{wanted}" not in text:
        found = [line for line in text.splitlines() if line.startswith("NOFILE=")]
        raise SelftestFailure(f"尝试单元里打开文件数的软、硬上限都应当是服务的硬上限 {wanted}，实际 {found or '没读到'}（{describe(summary)}）", world.server_log_lines())
    return f"尝试单元里打开文件数软、硬上限都是 {wanted}"


def cell_stall_clock_from_log_mtime(world):
    quick_grace = {"TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0"}
    world.start_server(extra=quick_grace)
    release = os.path.join(world.root, "release")
    task_id = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release])
    log = world.running(task_id)["attempts"][0].get("log", "")
    wait_path(world, log, "把尝试的日志建出来")   # 记了在跑和日志文件真建出来之间机器忙时隔得久：不等就去改修改时刻会撞上 FileNotFoundError
    world.stop_server()   # 在跑的尝试在它自己的单元里接着跑
    quiet_since = time.time() - LOG_QUIET_PINNED_SECONDS
    os.utime(log, (quiet_since, quiet_since))
    world.start_server(extra=quick_grace)
    try:
        summary = world.wait_for(task_id, lambda found: any((attempt.get("progress") or {}).get("log_quiet_seconds") is not None for attempt in found["attempts"]),
                                 "报日志多久没涨")
        quiet = next(attempt["progress"]["log_quiet_seconds"] for attempt in summary["attempts"] if (attempt.get("progress") or {}).get("log_quiet_seconds") is not None)
        shown = world.client(["status", str(task_id)]).stdout
    finally:
        open(release, "w").close()
    world.finished(task_id)
    if not LOG_QUIET_PINNED_SECONDS - 100 <= quiet <= LOG_QUIET_PINNED_SECONDS + 300 or "没涨" not in shown:
        raise SelftestFailure(f"日志最后写在 {LOG_QUIET_PINNED_SECONDS} 秒前、服务重启之后：日志多久没涨应当从日志的修改时刻算，实际 {quiet} 秒；status「{shown.strip()[-300:]}」",
                              world.server_log_lines())
    return f"服务重启之后在跑那一条的「日志多久没涨」从日志的修改时刻算：{quiet:.0f} 秒（修改时刻在 {LOG_QUIET_PINNED_SECONDS} 秒前）"


def idle_cpu_scope(world):
    """CPU 读得到、一直不涨的假 scope（判疑似卡住的那几格用）。"""
    scope = os.path.join(world.root, "idle-scope")
    os.makedirs(scope, exist_ok=True)
    with open(os.path.join(scope, "cpu.stat"), "w", encoding="utf-8") as handle:
        handle.write("usage_usec 1000\n")
    return scope


def suspect_lines(completed):
    return [line for line in completed.stdout.splitlines() if line.startswith(lib.SUSPECT_MARK + " ")]


def cell_suspected_stall_notified(world):
    """疑似卡住只通知、不停（task_server.py 的 check_suspect、judge_stall_stop）：窗口 6 秒（脚本每 0.25 秒打一行，输出间隔要比窗口小得多：机器忙时输出断档不到 6 秒，
    不会被判成日志不涨而停掉），假 scope 的 CPU 不涨，约 15 秒的脚本：头一回 wait 打 TASK_SERVER_SUSPECT、退 SUSPECT_EXIT，任务照跑、status 写疑似卡住、记告警；
    再起 wait 拿到 succeeded，同一段只报一次。弄坏开关 suspect-unreported 下不判，这一格红。"""
    world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "6", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01",
                              "TASK_SERVER_FAKE_SCOPE_DIRECTORY": idle_cpu_scope(world)})
    task_id = world.submit(["bash", "-c", 'for round in $(seq 1 60); do echo "tick $round"; sleep 0.25; done'])   # 日志一直在涨、CPU 不涨，约 15 秒
    first = world.client(["wait", str(task_id)], timeout=60)
    during = world.summary(task_id)
    shown = world.client(["status", str(task_id)])
    alerted = [alert for alert in world.alerts() if alert.get("kind") == "suspected-stall" and alert.get("task") == task_id]
    second = world.client(["wait", str(task_id)], timeout=90)
    problems = []
    if first.returncode != lib.SUSPECT_EXIT or not [line for line in suspect_lines(first) if line.startswith(f"{lib.SUSPECT_MARK} task={task_id} ")]:
        problems.append(f"日志在涨而 CPU 满窗口不涨：头一回 wait 应当打一行 {lib.SUSPECT_MARK} task={task_id} …、退 {lib.SUSPECT_EXIT}，"
                        f"实际退 {first.returncode}、「{first.stdout.strip()[-300:]}」")
    if during["task"].get("state") != "running" or not [attempt for attempt in during["attempts"] if attempt.get("state") == "running"]:
        problems.append(f"报了疑似卡住之后任务应当照跑（running），实际 {describe(during)}、状态 {during['task'].get('state')}")
    if "疑似卡住" not in shown.stdout:
        problems.append(f"status 应当写「疑似卡住」与依据，实际「{shown.stdout.strip()[-400:]}」")
    if not alerted:
        problems.append(f"应当记一条 suspected-stall 告警，实际告警 {world.alerts()}")
    if second.returncode != 0 or suspect_lines(second) or f"{lib.RESULT_MARK} task={task_id} type=succeeded " not in second.stdout:
        problems.append(f"再起 wait 应当拿到真结局（succeeded、退 0）、同一段不再报疑似，实际退 {second.returncode}、「{second.stdout.strip()[-300:]}」")
    if problems:
        raise SelftestFailure("疑似卡住只通知：" + "；".join(problems), world.server_log_lines())
    return (f"日志在涨、CPU 满窗口不涨：头一回 wait 打 {lib.SUSPECT_MARK}、退 {lib.SUSPECT_EXIT}，任务照跑、status 写疑似卡住、记了告警；"
            "再起 wait 拿到 succeeded，同一段只报一次")


LEFTOVER_GONE_SECONDS = 10.0   # leftover-processes-stopped：结局记下之后至多等这么久看留下的那个孙进程没了


def cell_leftovers_only_in_own_unit(world):
    """attempt_runner 停留下的进程只在自己待在这一次尝试的单元里时做（attempt_runner.py 的 stop_unit_leftovers）：自己的 cgroup 是起服务那一方的 scope
    （没起成单元：弄坏开关 no-detach）时，里面的进程一个不停。假 cgroup 根下放一份 cgroup.procs，点名一个真起的 sleep；不碰 systemd，开关打开也只杀这一个 sleep。"""
    import importlib.util
    spec = importlib.util.spec_from_file_location("attempt_runner_under_test", lib.RUNNER_SCRIPT)
    runner = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(runner)
    group = "/user.slice/singlefs-memory-cap-1-2.scope"
    os.makedirs(world.root + group, exist_ok=True)
    sleeper = subprocess.Popen(["sleep", "60"], stdin=subprocess.DEVNULL)
    try:
        with open(world.root + group + "/cgroup.procs", "w", encoding="utf-8") as handle:
            handle.write(f"{os.getpid()}\n{sleeper.pid}\n")
        runner.CGROUP_ROOT, runner.cgroup_of = world.root, (lambda process_id: group)
        result = runner.stop_unit_leftovers("singlefs-task-selftest-1-1.service", lib.broken("leftovers-outside-own-unit"))
        time.sleep(0.3)
        if sleeper.poll() is not None or result.get("processes"):
            raise SelftestFailure(f"自己的 cgroup 不是这一次尝试的单元时一个进程都不该停，实际停了 {result.get('processes')}、sleep {'已退' if sleeper.poll() is not None else '还在'}", [])
        result = runner.stop_unit_leftovers("singlefs-memory-cap-1-2.scope")
        sleeper.wait(timeout=15)
        if str(sleeper.pid) not in result.get("processes", []):
            raise SelftestFailure(f"自己就在点名的单元里时应当停掉留下的进程，实际 {result}", [])
    finally:
        if sleeper.poll() is None:
            sleeper.kill()
        sleeper.wait()
    return "自己的 cgroup 不是这一次尝试的单元时不停里面的进程；在自己的单元里照停"


RUNNER_THREADS_SPINNING = 3    # runner-threads-file-counts：真进程里空转的线程数（pbkdf2_hmac 在 OpenSSL 里算时放开 GIL，一直是 R）
RUNNER_THREADS_SLEEPING = 4    # 睡着等事件的线程数（S）
RUNNER_THREADS_NAME = "spin-and-sleep"   # 那个进程的 comm（不超过 15 个字节，内核不截）
RUNNER_THREADS_SECONDS = 20.0  # 等读数对上的安静时限
RUNNER_THREADS_PROGRAM = r'''
import hashlib, sys, threading
stop = threading.Event()
for _ in range(int(sys.argv[2])):
    threading.Thread(target=lambda: hashlib.pbkdf2_hmac("sha256", b"spin", b"salt", 2 ** 31 - 1), daemon=True).start()
for _ in range(int(sys.argv[3])):
    threading.Thread(target=stop.wait, daemon=True).start()
with open("/proc/self/comm", "w", encoding="utf-8") as handle:
    handle.write(sys.argv[4])
with open(sys.argv[1], "w", encoding="utf-8") as handle:
    handle.write("ready\n")
stop.wait()
'''


def cell_runner_threads_file_counts(world):
    """attempt_runner 每轮写的线程读数（attempt_runner.py 文件头 ③ 的 THREADS_FILE_NAME；note_threads）：起一个真的进程树，最上面是在 wait 的 bash，底下一个 python
    （3 个空转线程、4 个睡着的线程，主线程也睡着，comm 改成 spin-and-sleep），载入 attempt_runner.py 的 Sampler 对这棵树数：
    读数应当是 runnable 3、total 9（bash 1 个加 python 8 个）、top_threads 8、top_name spin-and-sleep、sample 是这一轮的轮数。整组在它自己的进程组里，收尾对整组发 KILL、核没留下进程。
    弄坏开关 runner-threads-state-unread 下（说明里 threads_state_unread）每个线程都算可跑的，runnable 成 9，这一格红。"""
    import importlib.util
    spec = importlib.util.spec_from_file_location("attempt_runner_under_test", lib.RUNNER_SCRIPT)
    runner = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(runner)
    directory = os.path.join(world.work, "runner-threads")
    os.makedirs(directory, exist_ok=True)
    ready = os.path.join(directory, "ready")
    total = 1 + 1 + RUNNER_THREADS_SPINNING + RUNNER_THREADS_SLEEPING
    process = subprocess.Popen(["bash", "-c", '"$1" -c "$2" "$3" "$4" "$5" "$6" & wait', "runner-threads", sys.executable, RUNNER_THREADS_PROGRAM, ready,
                                str(RUNNER_THREADS_SPINNING), str(RUNNER_THREADS_SLEEPING), RUNNER_THREADS_NAME], stdin=subprocess.DEVNULL, start_new_session=True)
    sampler = runner.Sampler({"directory": directory, "threads_state_unread": lib.broken("runner-threads-state-unread")}, process.pid, None)
    reading, family = {}, {}
    try:
        patience = world.patience(RUNNER_THREADS_SECONDS)
        while patience.waiting():
            if os.path.exists(ready):
                sampler.rounds += 1
                family = runner.descendants_with_depth(process.pid)
                sampler.note_threads(family)
                with open(os.path.join(directory, runner.THREADS_FILE_NAME), encoding="utf-8") as handle:
                    reading = json.load(handle)
                if reading == {"sample": sampler.rounds, "runnable": RUNNER_THREADS_SPINNING, "total": total, "top_threads": total - 1, "top_name": RUNNER_THREADS_NAME}:
                    break
            time.sleep(POLL_SECONDS)
    finally:
        with contextlib.suppress(ProcessLookupError):
            os.killpg(process.pid, signal.SIGKILL)   # 整组都是这一格自己起的
        process.wait()
    patience = world.patience(RUNNER_THREADS_SECONDS)
    while patience.waiting() and not all(process_gone(process_id) for process_id in family):
        time.sleep(POLL_SECONDS)
    left = [process_id for process_id in family if not process_gone(process_id)]
    wanted = {"sample": sampler.rounds, "runnable": RUNNER_THREADS_SPINNING, "total": total, "top_threads": total - 1, "top_name": RUNNER_THREADS_NAME}
    if reading != wanted or left:
        raise SelftestFailure(f"线程读数应当是 {wanted}（{RUNNER_THREADS_SPINNING} 个空转、{RUNNER_THREADS_SLEEPING} 个睡着、主线程与 bash 也睡着），收尾之后不留进程；"
                              f"实际最后一回 {reading or '没读到'}、留下的进程 {left}")
    return f"真进程树（{RUNNER_THREADS_SPINNING} 个空转、{RUNNER_THREADS_SLEEPING} 个睡着）的线程读数 {reading}"


LEFTOVERS_ERROR_EXIT = 3   # leftovers-error-isolated：那一条命令自己的退出码（不是 0，看得出照原样带回）
LEFTOVERS_ERROR_RUNNER = """import importlib.util, sys
specification = importlib.util.spec_from_file_location("attempt_runner_under_test", sys.argv[1])
runner = importlib.util.module_from_spec(specification)
specification.loader.exec_module(runner)
def raising(*arguments, **options):
    raise RuntimeError("自证：stop_unit_leftovers 抛错")
runner.stop_unit_leftovers = raising
sys.exit(runner.main(sys.argv[2:]))
"""   # leftovers-error-isolated：载入 attempt_runner.py、把 stop_unit_leftovers 换成一抛就错的，再照原样起 main


def cell_leftovers_error_isolated(world):
    """attempt_runner 停单元里留下的进程时抛了没料到的异常（records/2026-11-03-task-server与prune-server深入对攻审核.md「TS-02」；attempt_runner.py 的 main）：
    result.json 照写、exit_code 是命令自己的、unit_leftovers 是 null，attempt_runner 照命令的退出码退，日志里记一行告警。
    另起一个 python 载入 attempt_runner.py、把 stop_unit_leftovers 换成一抛就错的，照说明起 bash -c 'exit 3'（不起 systemd 单元）。
    弄坏开关 leftovers-error-uncaught 下（说明里 leftovers_error_uncaught）异常照改前抛出去、result.json 写不出，这一格红。"""
    directory = os.path.join(world.work, "leftovers-error")
    os.makedirs(directory, exist_ok=True)
    wrapper = os.path.join(world.work, "leftovers-error-runner.py")
    with open(wrapper, "w", encoding="utf-8") as handle:
        handle.write(LEFTOVERS_ERROR_RUNNER)
    log = os.path.join(directory, "attempt.log")
    specification_path = os.path.join(directory, "spec.json")
    with open(specification_path, "w", encoding="utf-8") as handle:
        json.dump({"argv": ["bash", "-c", f"exit {LEFTOVERS_ERROR_EXIT}"], "cwd": world.work, "environment": dict(os.environ), "log": log, "directory": directory,
                   "fast": True, "unit": "singlefs-task-selftest-1-1.service", "leftovers_error_uncaught": lib.broken("leftovers-error-uncaught")}, handle)
    completed = subprocess.run([sys.executable, "-B", wrapper, lib.RUNNER_SCRIPT, specification_path], stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=60)
    result, logged = None, ""
    with contextlib.suppress(OSError, ValueError):
        with open(os.path.join(directory, "result.json"), encoding="utf-8") as handle:
            result = json.load(handle)
    with contextlib.suppress(OSError):
        with open(log, encoding="utf-8", errors="replace") as handle:
            logged = handle.read()
    warned = "停单元里留下的进程时出了事（RuntimeError: 自证：stop_unit_leftovers 抛错）" in logged
    if (completed.returncode != LEFTOVERS_ERROR_EXIT or not isinstance(result, dict) or result.get("exit_code") != LEFTOVERS_ERROR_EXIT
            or result.get("unit_leftovers") is not None or not warned):
        raise SelftestFailure(f"停留下的进程时抛错：result.json 应当照写、exit_code {LEFTOVERS_ERROR_EXIT}、unit_leftovers 是 null，attempt_runner 退 {LEFTOVERS_ERROR_EXIT}、日志里记一行告警；"
                              f"实际退 {completed.returncode}、result.json {'写不出' if result is None else result}、日志里{'有' if warned else '没有'}那一行告警",
                              (completed.stdout + completed.stderr).splitlines()[-12:])
    return f"stop_unit_leftovers 抛错时 result.json 照写（exit_code {LEFTOVERS_ERROR_EXIT}、unit_leftovers 是 null），attempt_runner 照命令退 {LEFTOVERS_ERROR_EXIT}、日志记了告警"


def cell_leftover_processes_stopped(world):
    """尝试正常结束之后单元里还留着命令起的孙进程（task_server.py 的 stop_leftovers）：服务停掉它，结局说明写留下了几个、停了；
    不留进程的那一条说明里没有这一句。"""
    world.start_server()
    child_file = os.path.join(world.root, "leftover-child")
    left = world.finished(world.submit(["bash", "-c", 'sleep 120 & echo "$!" > "$1"; exit 0', "leftover", child_file]))
    clean = world.finished(world.submit(["bash", "-c", "exit 0", "clean"]))
    problems = []
    try:
        with open(child_file, encoding="utf-8") as handle:
            child = handle.read().strip()
    except OSError:
        child = ""
    patience = world.patience(LEFTOVER_GONE_SECONDS)
    while child and not process_gone(child) and patience.waiting():
        time.sleep(POLL_SECONDS)
    note = left["task"].get("note") or ""
    if left["task"].get("outcome") != "succeeded" or "还留着进程" not in note or "服务已停掉" not in note:
        problems.append(f"命令退 0 而留下一个后台 sleep：应当照 succeeded 收尾、结局说明写留下的进程与停了，实际 {describe(left)}、说明「{note}」")
    if not child or not process_gone(child):
        problems.append(f"留下的那个孙进程（{child or '没记下进程号'}）结局记下之后应当不在了")
    if "还留着进程" in (clean["task"].get("note") or ""):
        problems.append(f"没留下进程的那一条说明里不该有这一句，实际「{clean['task'].get('note')}」")
    if problems:
        raise SelftestFailure("留下的进程：" + "；".join(problems), world.server_log_lines())
    return "命令退出之后单元里留下的孙进程由服务停掉、结局说明写明；没留下的不写"


def cell_runaway_process_notified(world):
    """任务之外的进程空转（task_server.py 的 watch_runaway_processes）：进程表里只有不在任务 slice、不是内核线程、CPU 够久够满的那一条被报，恰好一条，写依据与按号停的出路；
    任务 slice 里的、内核线程、CPU 不够久的、占比不够的都不报。弄坏开关 runaway-process-unreported、runaway-slice-counted 下这一格红。"""
    listing = os.path.join(world.root, "process-list")
    inbox = FakeInbox(world, "a")
    other_inbox = FakeInbox(world, "b", session=SESSION_B, register=False)
    bystander = subprocess.Popen(["sleep", "600"])
    world.register_session(SESSION_B, other_inbox.path, "inbox-b", pid=bystander.pid)
    release = os.path.join(world.root, "release-runaway")
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_PROCESS_LIST": listing})
    head = "task-server：任务之外的进程空转"
    try:
        world.running(world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "runaway", release], extra=inbox.environment()))
        world.running(world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "runaway-other", release], session=SESSION_B, extra=other_inbox.environment()))
        with open(listing, "w", encoding="utf-8") as handle:
            handle.write(f"  501 {os.getpid()} 7200 7100 R 0::/user.slice/session-3.scope rev\n  502 {os.getpid()} 7200 7100 R 0::/user.slice/singlefs.slice/a.service in-slice\n"
                         f"  503 2 7200 7100 R - kworker\n  504 {os.getpid()} 7200 600 S 0::/user.slice/session-3.scope brief\n  505 {os.getpid()} 7200 3000 S 0::/user.slice/session-3.scope low-share\n"
                         )
        inbox.wait_for(world, head)
        time.sleep(INBOX_QUIET_SECONDS)
        found = inbox.headed(head)
        strayed = other_inbox.headed(head)
    finally:
        open(release, "w").close()
        bystander.kill()
        bystander.wait()
    problems = inbox_problems(found, head, "任务之外的进程空转")
    if strayed:
        problems.append(f"别的会话不该收到，实际收到 {len(strayed)} 条")
    text = found[0]["text"] if found else ""
    if found and ("进程 501" not in text.split("\n", 1)[0] or "7100 秒" not in text or "proc.py stop 501" not in text):
        problems.append(f"应当点名进程 501、写依据与按号停的出路，实际「{text[:400]}」")
    if problems:
        raise SelftestFailure("任务之外的进程空转：" + "；".join(problems), world.server_log_lines())
    return "五行进程表只报不在 slice、CPU 够久够满的那一条，恰好一次，写了依据与出路"


def cell_runaway_process_stopped_after_reports(world):
    """任务之外的进程空转（task_server.py 的 watch_runaway_processes）：认得出发起会话的先报两次，再由服务按号停掉真进程并告诉那个会话；认不出的（进程 1 的子进程）不等、直接终止，不发给任何会话；
    命令名在不碰名单里的、systemd 单元里的、登记的会话进程不碰。弄坏开关 runaway-process-not-stopped、runaway-unowned-not-stopped、runaway-protected-stopped 下这一格红。"""
    spinning = subprocess.Popen(["sleep", "600"])
    orphan = subprocess.Popen(["sleep", "600"])
    named, in_unit, session_process = (subprocess.Popen(["sleep", "600"]) for _ in range(3))
    world.register_session("selftest-session-keeper", "", "keeper", pid=session_process.pid)
    listing = os.path.join(world.root, "process-list-stop")
    inbox = FakeInbox(world, "a")
    release = os.path.join(world.root, "release-runaway-stop")
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_PROCESS_LIST": listing, "TASK_SERVER_SELFTEST_RUNAWAY_RENOTIFY_SECONDS": "0.3"})
    head = "task-server：任务之外的进程空转 已停"
    try:
        world.running(world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "runaway-stop", release], extra=inbox.environment()))
        with open(listing, "w", encoding="utf-8") as handle:
            handle.write(f"  {spinning.pid} {os.getpid()} 7200 7100 R 0::/user.slice/session-3.scope spinning\n  {orphan.pid} 1 7200 7100 R 0::/user.slice/session-3.scope orphan\n"
                         f"  {named.pid} 1 7200 7100 R 0::/user.slice/session-3.scope claude\n  {in_unit.pid} 1 7200 7100 R 0::/user.slice/user-1000.slice/user@1000.service/app.slice/a.service daemon\n"
                         f"  {session_process.pid} 1 7200 7100 R 0::/user.slice/session-3.scope registered\n")
        inbox.wait_for(world, head)
        time.sleep(0.5)
        stopped = inbox.headed(head)
        reported = inbox.headed(f"task-server：任务之外的进程空转 本机进程 {spinning.pid}")
        spinning_gone, orphan_gone = spinning.poll() is not None, orphan.poll() is not None
        kept_alive = [name for name, process in (("不碰名单", named), ("systemd 单元", in_unit), ("登记的会话进程", session_process)) if process.poll() is None]
        unowned_messages = [message for message in inbox.headed("task-server：任务之外的进程空转") if str(orphan.pid) in message["text"]]
    finally:
        open(release, "w").close()
        for process in (spinning, orphan, named, in_unit, session_process):
            process.kill()
            process.wait()
    problems = []
    if len(stopped) != 1 or str(spinning.pid) not in stopped[0]["text"] or "回读已不在" not in stopped[0]["text"]:
        problems.append(f"应当恰好一条「已停」点名进程 {spinning.pid}、写回读已不在，实际 {[message['text'][:200] for message in stopped]}")
    if len(reported) != 2:
        problems.append(f"停之前应当恰好报两次，实际 {len(reported)} 次")
    if not spinning_gone:
        problems.append("所属会话认得出的那个进程应当被服务停掉，实际还在")
    if not orphan_gone:
        problems.append("认不出发起者的那个进程应当被直接终止，实际还在")
    if unowned_messages:
        problems.append(f"认不出发起者的不该发给任何会话，实际收到 {len(unowned_messages)} 条")
    if len(kept_alive) != 3:
        problems.append(f"不碰的三类都该还在，实际还在的只有 {kept_alive}")
    if problems:
        raise SelftestFailure("报满次数之后按号停：" + "；".join(problems), world.server_log_lines())
    return "认得出发起会话的报两次之后被按号停掉、那个会话收到一条；认不出的直接终止、不发消息；不碰的三类都还在"


def cell_cancel_start_race(world):
    """TS-01（records/2026-11-01-task-server深入审核.md）：起单元那一侧判过 queued 之后、记尝试之前精确停住再取消：取消回 stopped 之后一次都不起、
    任务不从 finished 回到 running；记了尝试、起单元之前停住再取消：取消等单元起完、停了、回读过才回，结局 cancelled。"""
    pause = os.path.join(world.root, "pause")
    os.makedirs(pause)
    problems = []
    world.resources(local=(30, 30), hold=True)
    world.start_server(extra={"TASK_SERVER_SELFTEST_PAUSE_POINT": f"start-checked:{pause}"})
    marker = os.path.join(world.root, "race-ran")
    task_id = world.submit(["bash", "-c", ': > "$1"', "touch-marker", marker])
    open(os.path.join(pause, "arm-start-checked"), "w").close()
    world.resources(local=(30, 30))
    wait_path(world, os.path.join(pause, "reached-start-checked"), "停在判过 queued 之后、记尝试之前")
    completed = world.client(["cancel", str(task_id)])
    open(os.path.join(pause, "release-start-checked"), "w").close()
    world.next_pass()
    world.next_pass()
    summary = world.summary(task_id)
    if (completed.returncode != 0 or summary["task"].get("state") != "finished" or summary["task"].get("outcome") != "cancelled"
            or summary["attempts"] or os.path.exists(marker)):
        problems.append(f"判过 queued、记尝试之前取消：取消回了 stopped（退 {completed.returncode}）之后应当一次都不起、任务留在 finished/cancelled，"
                        f"实际 {summary['task'].get('state')}、{describe(summary)}、命令{'跑了' if os.path.exists(marker) else '没跑'}")
    world.stop_server()
    world.resources(local=(30, 30), hold=True)
    world.start_server(extra={"TASK_SERVER_SELFTEST_PAUSE_POINT": f"claimed:{pause}"})
    release = os.path.join(world.root, "race-release")
    task_id = world.submit(blocking_command(release))
    open(os.path.join(pause, "arm-claimed"), "w").close()
    world.resources(local=(30, 30))
    try:
        wait_path(world, os.path.join(pause, "reached-claimed"), "停在记了尝试之后、起单元之前")
        canceller = world.client(["cancel", str(task_id)], background=True)
        time.sleep(1.0)
        if canceller.poll() is not None:
            problems.append(f"记了尝试、单元还没起时取消：取消应当等单元起完再停（这时不回），实际已退 {canceller.returncode}")
        open(os.path.join(pause, "release-claimed"), "w").close()
        stdout, stderr = world.communicate(canceller, STATE_TIMEOUT_SECONDS)
        summary = world.summary(task_id)
        units = [attempt.get("unit", "") for attempt in summary["attempts"]]
        still = [unit for unit in units if world.unit_active(unit)]
        if canceller.returncode != 0 or still or not units:
            problems.append(f"记了尝试、单元还没起时取消：应当等单元起了、停了、回读过才退 0，实际退 {canceller.returncode}、单元 {units}、还在跑的 {still}（{stdout.strip()[-200:]}）")
        finished = world.finished(task_id)
        if finished["task"].get("outcome") != "cancelled":
            problems.append(f"那一件的结局应当是 cancelled，实际 {describe(finished)}")
    finally:
        open(release, "w").close()
    if problems:
        raise SelftestFailure("起单元与取消交错：" + "；".join(problems), world.server_log_lines())
    return "判过 queued 之后到的取消：回了 stopped 就一次都不起；记了尝试之后到的取消：等单元起完、停了、回读过才回，结局 cancelled"


ATTENTION_IO_WAIT_SECONDS = "3"      # io-wait-local-not-stalled：卡在 IO 上的提醒门槛（秒；TASK_SERVER_SELFTEST_IO_WAIT_ATTENTION_SECONDS），比卡住阈值 2 秒长
FAKE_BLOCKED_PROCESS = 4194301       # io-wait-local-not-stalled：假 scope 的 cgroup.procs 点名的进程号，假 /proc 下只有它一份 stat、是 D 状态（服务照假 /proc 读，与真进程号撞了也只读这一份）
IO_WAIT_SUSPECT_WAITS = 3            # io-wait-local-not-stalled：CPU 不涨先报的疑似卡住照报，至多再起这么多回 wait 等 io-wait 那一行


def quiet_log_busy_problems(waited, which):
    """io-wait-local-not-stalled 用：几回 wait 的输出里不该有 kind=quiet-log-busy 的那一行（服务不是因为 CPU 在涨或读不到才没停它）。"""
    lines = [line for completed in waited for line in attention_lines(completed) if attention_fields(line).get("kind") == "quiet-log-busy"]
    return [f"{which}不该报 quiet-log-busy，实际 wait 打了「{lines[0][:300]}」"] if lines else []


def cell_io_wait_local_not_stalled(world):
    """卡在 IO 上（task_server.py 文件头 ①「卡在 IO 上」；任务 8608、8609、8611、8619 在等 NFS 时被当成卡住停掉）：本机一条只打一行就不出声、跑到放行为止，
    假 scope 的 CPU 一直不涨（卡住阈值 2 秒），假 scope 的 cgroup.procs 点名一个进程号，假 /proc（TASK_SERVER_SELFTEST_PROC_ROOT）下它的 stat 是 D 状态：
    服务不判 stalled、不停；日志没涨的每一轮都有 D 状态的进程满提醒门槛（自证里 3 秒），wait 打一行 kind=io-wait、依据写 D 状态的进程数与 /proc/pressure/io 的 avg60
    （读不到照写读不到）、退 ATTENTION_EXIT（CPU 不涨先报的疑似卡住照报，再起 wait 接着等）；任务照旧在跑，放行之后跑完。
    没有 D 状态的照旧停 stalled 由 stalled-attempt-stopped 那一格判（同一段判法：假 /proc 不换，那一条的真进程都在睡、不在 D 状态）。
    日志长期不涨的提醒门槛同样压到 3 秒（ATTENTION_QUIET_SECONDS）：按 D 状态免停的不报 quiet-log-busy（task_server.py 的 quiet_log_items），各回 wait 都不该打那一种。
    弄坏开关 io-wait-ignored 下停掉、记 stalled，attention-io-wait-unreported 下 wait 不叫醒、等到命令自己退，quiet-log-busy-any-exemption 下 wait 另报 quiet-log-busy，这一格红。"""
    scope = os.path.join(world.root, "io-wait-scope")
    os.makedirs(scope)
    with open(os.path.join(scope, "cpu.stat"), "w", encoding="utf-8") as handle:   # CPU 读得到、一直不涨
        handle.write("usage_usec 1000\n")
    with open(os.path.join(scope, "cgroup.procs"), "w", encoding="utf-8") as handle:
        handle.write(f"{FAKE_BLOCKED_PROCESS}\n")
    proc_root = os.path.join(world.root, "io-wait-proc")
    os.makedirs(os.path.join(proc_root, str(FAKE_BLOCKED_PROCESS)))
    with open(os.path.join(proc_root, str(FAKE_BLOCKED_PROCESS), "stat"), "w", encoding="utf-8") as handle:   # 进程名带括号与空格：状态照最后一个右括号切
        handle.write(f"{FAKE_BLOCKED_PROCESS} (git (diff) D 1 {FAKE_BLOCKED_PROCESS} {FAKE_BLOCKED_PROCESS} 0 -1 4194560 0 0 0 0 0 0 0 0 20 0 1 0 0 0 0\n")
    release = os.path.join(world.root, "release-io-wait")
    problems, fields, done = [], {}, None
    try:
        world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01", "TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope,
                                  "TASK_SERVER_SELFTEST_IO_WAIT_ATTENTION_SECONDS": ATTENTION_IO_WAIT_SECONDS, "TASK_SERVER_SELFTEST_PROC_ROOT": proc_root,
                                  "TASK_SERVER_SELFTEST_QUIET_ATTENTION_SECONDS": ATTENTION_QUIET_SECONDS})
        task_id = world.submit(["bash", "-c", "echo started; " + BLOCK_SCRIPT, "io-wait", release])
        completed, waited = None, []
        for _ in range(IO_WAIT_SUSPECT_WAITS):
            completed = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
            waited.append(completed)
            if completed.returncode != lib.SUSPECT_EXIT:
                break
        during = world.summary(task_id)
        problems, fields = attention_problems(completed, task_id, "io-wait", lib.ATTENTION_EXIT)
        problems += quiet_log_busy_problems(waited, "按 D 状态免停的")
        basis = fields.get("basis", "")
        if "D 状态" not in basis or "1 个" not in basis or ("avg60=" not in basis and "读不到" not in basis):
            problems.append(f"依据应当写每一轮都有进程在「D 状态」、最近一轮「1 个」与 /proc/pressure/io 的 avg60（或读不到），实际「{basis}」")
        if during["task"].get("state") != "running" or not [attempt for attempt in during["attempts"] if attempt.get("state") == "running"]:
            problems.append(f"卡在 IO 上的应当照旧在跑（running、不判 stalled），实际 {describe(during)}、状态 {during['task'].get('state')}")
        open(release, "w").close()
        done = world.finished(task_id)
    finally:
        open(release, "w").close()
    if done is not None and done["task"].get("outcome") != "succeeded":
        problems.append(f"放行之后应当跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("本机卡在 IO 上：" + "；".join(problems), world.server_log_lines())
    return (f"本机日志与 CPU 都不涨而每一轮都有进程在 D 状态：不停，满 {ATTENTION_IO_WAIT_SECONDS} 秒 wait 打 {lib.ATTENTION_MARK} kind=io-wait"
            f"（依据「{fields.get('basis')}」）、退 {lib.ATTENTION_EXIT}，放行后跑完")


def peer_io_wait_script(release, blocked, read):
    """io-wait-peer-not-stalled 的第二台那条命令：每轮 0.5 秒，只打起法那一层的三种进度行（日志不算在涨），CPU 一直不涨，放行文件在了就退 0（peer_hold_script）；
    IO_WAIT 那一行打在 CPU 那一行前面（服务读到 CPU 的那一回 IO_WAIT 一定已在，不因两行之间那一下判成这一轮读不到）。"""
    return peer_hold_script(release, 'echo "MULTI_HOST_RUN_PEER_IO_WAIT poll=$round blocked=' + str(blocked) + ' read=' + read + '"',
                            'echo "MULTI_HOST_RUN_PEER_CPU poll=$round usage_usec=1000 read=yes"', 'echo "MULTI_HOST_RUN_PEER_MEMORY poll=$round current_bytes=0 peak_bytes=0 read=no"')


def cell_io_wait_peer_not_stalled(world):
    """第二台卡在 IO 上（task_server.py 文件头 ①「卡在 IO 上」）：三条派到第二台、日志与 CPU 都不涨（卡住阈值 2 秒），每轮打一行 MULTI_HOST_RUN_PEER_IO_WAIT，都等放行文件才退：
    blocked=2 read=yes 的不停、放行之后跑完；blocked=0 read=yes（没有 D 状态的进程）与 blocked=3 read=no（读不到）的照原判法由服务停、记 stalled。
    服务停了那两条，说明它判过卡住窗口；blocked=2 的那一条再跑满 PEER_HELD_WINDOWS 个窗口（按服务记的起跑时刻）才放行，这之间被停就记 stalled。
    弄坏开关 io-wait-ignored 下 blocked=2 的也停掉、记 stalled，这一格红。"""
    world.resources(local=(8, 30), peer=(100, 100), peer_cap_gibibytes=64)
    world.seed_peak("selftest-peer-io-wait", 10 * GIBIBYTE)
    world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": str(PEER_STALL_SECONDS), "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01"})
    expected = {"blocked": (2, "yes", "succeeded"), "unblocked": (0, "yes", "stalled"), "unread": (3, "no", "stalled")}
    release = os.path.join(world.root, "release-peer-io-wait")
    summaries = {}
    try:
        submitted = {name: world.submit(["bash", "-c", peer_io_wait_script(release, blocked, read), name, "fake-peer-run"], extra={lib.KEY_VARIABLE: "selftest-peer-io-wait"}, cwd=world.repo)
                     for name, (blocked, read, _) in expected.items()}
        for name in ("unblocked", "unread"):
            summaries[name] = world.finished(submitted[name], timeout=PEER_STOP_QUIET_SECONDS)
        world.wait_for(submitted["blocked"], lambda summary: summary["task"].get("state") == "finished" or attempt_has_run_for(summary, PEER_HELD_WINDOWS * PEER_STALL_SECONDS),
                       f"在第二台跑满 {PEER_HELD_WINDOWS} 个卡住窗口（或被停）")
    finally:
        open(release, "w").close()
    summaries["blocked"] = world.finished(submitted["blocked"])
    problems = []
    for name, (blocked, read, outcome) in expected.items():
        summary = summaries[name]
        host = summary["attempts"][0].get("host") if summary["attempts"] else None
        if host != "peer" or summary["task"].get("outcome") != outcome:
            problems.append(f"blocked={blocked} read={read} 的那一条应当在第二台跑、记 {outcome}，实际在 {host}、{describe(summary)}")
    if problems:
        raise SelftestFailure("第二台卡在 IO 上：" + "；".join(problems), world.server_log_lines())
    return "第二台日志与 CPU 都不涨：每轮 blocked=2 read=yes 的不停、跑完；blocked=0 与 read=no 的照原判法记 stalled"


PROGRESS_FREE_GIBIBYTES = 30            # progress-stall-reported：本机空 30G，排头一条没记录的（4G）照公式放得下
PROGRESS_WATCH_SECONDS = 4 * SERVER.FAST_PROGRESS_STALL_SECONDS   # 那一格等看进展报出来至多等这么久（过时限几倍）
PROGRESS_HEALTH_TEXT = "一条都没起（看进展"   # task_server.py 的 progress_problem 那一句里的几个字


def cell_progress_stall_reported(world):
    """看进展兜派活自己判错（task_server.py 的 progress_problem；队列卡死 4 个多小时、51 条排着只跑 3 条那一次）：本机空 30G、排头一条小的，
    派活被 TASK_SERVER_SELFTEST_PLACEMENT_REFUSED 演成一律判不能起：派活的结论是不能起（starved_problem 不报），看进展只读余量与起没起，
    照公式放得下最小那条而本机过了时限一条都没起，/health 回不健康、那一句点名本机、照公式还能记多少与最小那条的任务号；放开之后跑完、回 ok。
    弄坏开关 progress-unwatched 下 /health 一直回 ok，这一格红。"""
    world.resources(local=(PROGRESS_FREE_GIBIBYTES, PROGRESS_FREE_GIBIBYTES))
    refused = os.path.join(world.root, "placement-refused")
    open(refused, "w").close()
    world.start_server(extra={"TASK_SERVER_SELFTEST_PLACEMENT_REFUSED": refused})
    task_id = world.submit(["bash", "-c", "exit 0", "progress-stall"])
    waiting = world.wait_for(task_id, lambda found: "PLACEMENT_REFUSED" in (found.get("waiting_for") or "") or found["task"].get("state") != "queued", "判成不能起")
    health = health_until(world, lambda found: PROGRESS_HEALTH_TEXT in (found.get("problem") or ""), PROGRESS_WATCH_SECONDS)
    os.remove(refused)
    summary = world.finished(task_id)
    recovered = health_until(world, lambda found: found.get("ok"), 10)
    problem = health.get("problem") or ""
    problems = []
    if waiting["attempts"] or "PLACEMENT_REFUSED" not in (waiting.get("waiting_for") or ""):
        problems.append(f"派活一律判不能起时排头应当排着、排队原因写演法那一句，实际 {describe(waiting)}、在等「{waiting.get('waiting_for')}」")
    if health.get("ok") or PROGRESS_HEALTH_TEXT not in problem or "本机" not in problem or f"任务 {task_id}" not in problem or STARVED_HEALTH_TEXT in problem:
        problems.append(f"派活判错（一律判不能起）而本机照公式放得下：过了看进展的时限（{SERVER.FAST_PROGRESS_STALL_SECONDS:.0f} 秒）/health 应当回不健康、"
                        f"看进展那一句点名本机与任务 {task_id}，不报「{STARVED_HEALTH_TEXT}」（派活的结论是不能起），实际 {health}")
    if summary["task"].get("outcome") != "succeeded" or not recovered.get("ok"):
        problems.append(f"放开之后应当跑完、/health 回 ok，实际 {describe(summary)}、{recovered}")
    if problems:
        raise SelftestFailure("看进展兜派活判错：" + "；".join(problems), world.server_log_lines())
    return f"派活一律判不能起、本机照公式放得下：/health 报「{problem[:100]}」，放开之后跑完、回 ok"


RUNAWAY_RESTART_PID = 601          # 换服务接着数那两格的假空转进程号（进程表是假的：不起真进程、不停真进程）
RUNAWAY_RESTART_OLD_PID = 602      # runaway-reused-pid-counted-afresh 里换服务前后是同一个的那一个
RUNAWAY_RESTART_GAP_SECONDS = 10000.0   # runaway-reused-pid-counted-afresh：换服务之前那两次报离此刻这么久（服务停着的空档）


def runaway_restart_scheduler(world, database_name, listing):
    """restart_surviving_scheduler 再套上 watch_runaway_processes 读写的几样：进程表是这一格给的那一份、所属会话认得出（SESSION_A）、发消息与按号停都是假的，
    记下发了什么（sent）、停了哪几个（stopped）；隔多久再报是 0（每扫一次都到点）。"""
    scheduler = restart_surviving_scheduler(world, database_name)
    scheduler.runaway_renotify_seconds = 0.0
    scheduler.sent, scheduler.stopped = [], []
    scheduler.process_listings = lambda: [(SERVER.LOCAL, listing)]
    scheduler.runaway_owner = lambda pid, listing_text: SESSION_A
    scheduler.runaway_targets = lambda owner: [{"id": 1, "session": owner}]
    scheduler.send_to_session = lambda task, text, follow: scheduler.sent.append(text)

    def stop_runaway_process(host, pid):   # 假的按号停：只记下停了谁，回「回读已不在」
        scheduler.stopped.append(pid)
        return True

    scheduler.stop_runaway_process = stop_runaway_process
    return scheduler


def runaway_listing_line(pid, elapsed_seconds):
    """假进程表的一行（lib.parsed_process_list 的列）：登录会话 scope 里、已跑 elapsed_seconds 秒、累计 CPU 比它少 100 秒（够 RUNAWAY_MINIMUM_CPU_SECONDS 与占比）。"""
    return f"  {pid} 1 {elapsed_seconds} {elapsed_seconds - 100} R 0::/user.slice/session-3.scope spinning\n"


def cell_runaway_reports_survive_restart(world):
    """任务之外的空转进程报过几次换服务后接着数（task_server.py 的 watch_runaway_processes、restore_restart_surviving_state）：认得出所属会话的那一个
    扫两次报两次（RUNAWAY_REPORTS_BEFORE_STOP），服务停了再起（同一份库上另建调度器），下一次扫到就按号停、发「已停」，不再从零报两次。
    套在假调度器上，进程表与停进程都是假的，不起服务。弄坏开关 runaway-reports-memory-only 下换服务后又报一次、不停，这一格红。"""
    listing = runaway_listing_line(RUNAWAY_RESTART_PID, 7200)
    before = runaway_restart_scheduler(world, "runaway-restart.sqlite3", listing)
    for _ in range(SERVER.RUNAWAY_REPORTS_BEFORE_STOP):
        SERVER.Scheduler.watch_runaway_processes(before)
    after = runaway_restart_scheduler(world, "runaway-restart.sqlite3", listing)
    SERVER.Scheduler.watch_runaway_processes(after)
    problems = []
    if len(before.sent) != SERVER.RUNAWAY_REPORTS_BEFORE_STOP or before.stopped:
        problems.append(f"换服务之前应当报 {SERVER.RUNAWAY_REPORTS_BEFORE_STOP} 次、不停，实际报 {len(before.sent)} 次、停了 {before.stopped}")
    if after.stopped != [RUNAWAY_RESTART_PID] or len(after.sent) != 1 or "已停" not in after.sent[0]:
        problems.append(f"换服务后应当接着数、这一次就按号停 {RUNAWAY_RESTART_PID} 并发一条「已停」，实际停了 {after.stopped}、发了 {[text[:80] for text in after.sent]}")
    if problems:
        raise SelftestFailure("空转进程的报数换服务不丢：" + "；".join(problems))
    return f"报过 {SERVER.RUNAWAY_REPORTS_BEFORE_STOP} 次的进程换服务后接着数，下一次扫到就按号停"


def cell_runaway_reused_pid_counted_afresh(world):
    """换服务的空档里同号换了进程的从零数（task_server.py 的 watch_runaway_processes）：两个进程各报过两次，那两次离此刻 RUNAWAY_RESTART_GAP_SECONDS 秒（服务停着的空档）；
    服务再起时 602 已跑得比那两次早（同一个），按号停；601 已跑的时长短于那段空档（上次报它之后才起的，是号被复用的另一个），从零数、报一次、不停。
    套在假调度器上，进程表与停进程都是假的，不起服务。弄坏开关 runaway-reused-pid-carried 下 601 照号接着数、一露面就被停，这一格红。"""
    before_listing = runaway_listing_line(RUNAWAY_RESTART_PID, 7200) + runaway_listing_line(RUNAWAY_RESTART_OLD_PID, 7200)
    before = runaway_restart_scheduler(world, "runaway-reused.sqlite3", before_listing)
    for _ in range(SERVER.RUNAWAY_REPORTS_BEFORE_STOP):
        SERVER.Scheduler.watch_runaway_processes(before)
    for entry in before.runaway_notified.values():
        entry["reported_at"] -= RUNAWAY_RESTART_GAP_SECONDS
    SERVER.Scheduler.persist_restart_surviving_state(before, "runaway_notified")
    after_listing = (runaway_listing_line(RUNAWAY_RESTART_PID, int(RUNAWAY_RESTART_GAP_SECONDS // 2))
                     + runaway_listing_line(RUNAWAY_RESTART_OLD_PID, int(RUNAWAY_RESTART_GAP_SECONDS * 2)))
    after = runaway_restart_scheduler(world, "runaway-reused.sqlite3", after_listing)
    SERVER.Scheduler.watch_runaway_processes(after)
    reported = [text for text in after.sent if f"本机进程 {RUNAWAY_RESTART_PID}" in text.split("\n", 1)[0] and "已停" not in text]
    problems = []
    if after.stopped != [RUNAWAY_RESTART_OLD_PID]:
        problems.append(f"换服务后只该停跑得比上次报早的 {RUNAWAY_RESTART_OLD_PID}，号被复用的 {RUNAWAY_RESTART_PID} 不停，实际停了 {after.stopped}")
    if len(reported) != 1 or after.runaway_notified.get((SERVER.LOCAL, RUNAWAY_RESTART_PID), {}).get("count") != 1:
        problems.append(f"号被复用的 {RUNAWAY_RESTART_PID} 应当从零数、报一次（count 1），实际报了 {len(reported)} 次、"
                        f"账 {after.runaway_notified.get((SERVER.LOCAL, RUNAWAY_RESTART_PID))}")
    if problems:
        raise SelftestFailure("换服务的空档里同号换了进程：" + "；".join(problems))
    return f"同一个的 {RUNAWAY_RESTART_OLD_PID} 接着数、按号停；号被复用的 {RUNAWAY_RESTART_PID} 从零数、报一次"


def cell_stall_stop_spares_next_attempt(world):
    """顶在上限上卡住（同 stalled-at-cap-retried 的假 scope）判停、起了停单元的后台线程，它读库之前精确停住（TASK_SERVER_SELFTEST_PAUSE_POINT=stop-attempts-read）；
    这时第 1 次自己退 250（撞上限）、照重试规矩重排起了第 2 次（每 0.25 秒打一行，不卡）；放开之后后台线程只停判它的那一次（task_server.py 的 judged_attempt_only）：
    排队读数里那一行 stopped=-，第 2 次的单元照跑，放行之后跑完、各次 memory-cap-hit、succeeded。弄坏开关 stall-stop-by-task 下按任务号停、把第 2 次也停了，这一格红。"""
    scope = os.path.join(world.root, "at-cap-scope")
    os.makedirs(scope)
    for name, text in (("cpu.stat", "usage_usec 1000\n"), ("memory.peak", f"{AT_CAP_PEAK_BYTES}\n"), ("memory.stat", process_memory_stat(AT_CAP_PEAK_BYTES))):
        with open(os.path.join(scope, name), "w", encoding="utf-8") as handle:
            handle.write(text)
    world.seed_peak("selftest-stall-next", AT_CAP_SEED_PEAK_BYTES)
    pause = os.path.join(world.root, "pause")
    os.makedirs(pause)
    capped, retried = os.path.join(world.root, "release-capped-attempt"), os.path.join(world.root, "release-retried-attempt")
    world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01", "TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope,
                              "TASK_SERVER_SELFTEST_PAUSE_POINT": f"stop-attempts-read:{pause}"})
    open(os.path.join(pause, "arm-stop-attempts-read"), "w").close()
    # 第 1 次不出声地等（顶在上限上卡住），放行就退 250；重排之后那一次每 0.25 秒打一行、放行就退 0
    script = ('if [[ "$TASK_SERVER_ATTEMPT" == 1 ]]; then echo started; for round in $(seq 1 2000); do [[ -e "$1" ]] && exit 250; sleep 0.1; done; exit 9; fi; '
              'for round in $(seq 1 2000); do [[ -e "$2" ]] && exit 0; echo tick; sleep 0.25; done; exit 9')
    task_id = world.submit(["bash", "-c", script, "stall-next", capped, retried], extra={lib.KEY_VARIABLE: "selftest-stall-next"})
    problems = []
    try:
        wait_path(world, os.path.join(pause, "reached-stop-attempts-read"), "判停的后台线程读库之前停住")
        open(capped, "w").close()
        summary = world.wait_for(task_id, lambda summary: len(summary["attempts"]) >= 2 and summary["attempts"][-1].get("state") == "running", "重排起第 2 次")
        unit = summary["attempts"][-1].get("unit", "")
        open(os.path.join(pause, "release-stop-attempts-read"), "w").close()
        patience = world.patience(STATE_TIMEOUT_SECONDS)
        stops = []
        while patience.waiting() and not stops:
            stops = [line for line in world.scheduler_reads() if line.startswith(f"stop task={task_id} ")]
            time.sleep(POLL_SECONDS)
        current = world.summary(task_id)["attempts"][-1]
        if not stops or not stops[0].endswith(" stopped=-") or not world.unit_active(unit) or current.get("state") != "running":
            problems.append(f"判停的后台线程读库时第 1 次已自己退了、第 2 次在跑：它应当一个都不停（stopped=-）、第 2 次的单元照跑，"
                            f"实际停单元那一行 {stops[:1]}、第 2 次的单元{'在跑' if world.unit_active(unit) else '不在跑'}、状态 {current.get('state')}")
    finally:
        open(os.path.join(pause, "release-stop-attempts-read"), "w").close()
        open(capped, "w").close()
        open(retried, "w").close()
    done = world.finished(task_id, timeout=60)
    seen = [attempt.get("outcome") for attempt in done["attempts"]]
    if seen != ["memory-cap-hit", "succeeded"] or done["task"].get("outcome") != "succeeded":
        problems.append(f"放行之后应当各次 memory-cap-hit、succeeded，任务 succeeded，实际各次 {seen}、{describe(done)}")
    if problems:
        raise SelftestFailure("判停的后台线程只停判它的那一次：" + "；".join(problems), world.server_log_lines())
    return "判停的后台线程读库时第 1 次已退、第 2 次在跑：它一个都不停，第 2 次照跑完"


def release_all(paths):
    for path in paths:
        open(path, "w").close()


OUTER_UNIT_LAUNCHER = ("import json, os, sys\n"
                       "environment = json.load(open(sys.argv[1], encoding='utf-8'))\n"
                       "log = os.open(sys.argv[2], os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o600)\n"
                       "os.dup2(log, 1)\nos.dup2(log, 2)\n"
                       "os.execve(sys.executable, [sys.executable, '-B', sys.argv[3], 'serve'], environment)\n")   # 外层单元里起这一格的服务：照这一格服务的环境、输出进这一格的服务日志
OUTER_UNIT_STOP_SECONDS = 20   # selftest-units-stop-with-outer：停了外层单元之后，等尝试单元停下至多这么久（带 BindsTo 的当场停）
STOPPED_UNIT_STATES = ("inactive", "failed", "")   # systemctl --user show -p ActiveState 打这几样算停了（单元收掉了的打 inactive）
ENCLOSING_UNIT_SAMPLES = (("0::/user.slice/user-1000.slice/user@1000.service/app.slice/outer-sample.service\n", "outer-sample.service"),
                          ("0::/user.slice/user-1000.slice/user@1000.service/singlefs.slice/memory-cap-sample.scope/payload\n", "memory-cap-sample.scope"),
                          ("0::/user.slice/user-1000.slice/session-3.scope\n", ""), ("0::/system.slice/sample.service\n", ""))


def unit_active_state(unit):
    completed = subprocess.run(["systemctl", "--user", "show", "-p", "ActiveState", "--value", unit], capture_output=True, text=True, stdin=subprocess.DEVNULL, timeout=30)
    return completed.stdout.strip()


def cell_selftest_units_stop_with_outer(world):
    """自证里起的尝试单元随外层停（task_server.py 文件头 ③「套在外层单元里的服务」）：这一格的服务起在一个这一格自己起的外层单元里（systemd-run --user，单元名带这一格的进程号），
    交一条一直等放行文件的命令：它的尝试单元 BindsTo 那个外层单元；停外层单元之后 OUTER_UNIT_STOP_SECONDS 秒里尝试单元停了（不等命令自己退）。
    另核 lib.enclosing_user_unit 认几种 cgroup 写法。弄坏开关 nested-units-unbound 下不带 BindsTo、外层停了尝试单元照跑，这一格红。"""
    problems = [f"cgroup「{text.strip()}」应当认成「{unit}」，实际「{lib.enclosing_user_unit(text)}」" for text, unit in ENCLOSING_UNIT_SAMPLES
                if lib.enclosing_user_unit(text) != unit]
    outer = f"singlefs-task-selftest-outer-{os.getpid()}-{time.time_ns()}.service"
    environment_path = os.path.join(world.root, "outer-environment.json")
    with open(environment_path, "w", encoding="utf-8") as handle:
        json.dump(world.server_environment, handle)
    world.server_logs += 1
    log_path = os.path.join(world.root, f"server-{world.server_logs}.log")
    release = os.path.join(world.root, "release-outer")
    write_executable(os.path.join(world.work, "outer-hold.sh"), f"#!/usr/bin/env bash\n{HOLD_UNTIL_RELEASED_SCRIPT}\n")
    with open("/proc/self/cgroup", encoding="utf-8") as handle:
        own_unit = lib.enclosing_user_unit(handle.read())   # 外层单元自己也随这一格所在的单元停：整份自证被停时它不留下
    bound = ["-p", f"BindsTo={own_unit}", "-p", f"After={own_unit}"] if own_unit else []
    started = subprocess.run(["systemd-run", "--user", f"--unit={outer}", "--collect", "--quiet", "-p", "KillMode=mixed", *bound, f"--working-directory={world.root}", "--",
                              sys.executable, "-B", "-c", OUTER_UNIT_LAUNCHER, environment_path, log_path, lib.SERVER_SCRIPT],
                             capture_output=True, text=True, stdin=subprocess.DEVNULL, timeout=60)
    if started.returncode != 0:
        raise SelftestFailure(f"外层单元起不来（systemd-run 退 {started.returncode}）：{(started.stderr + started.stdout).strip()[-300:]}")
    unit = ""
    try:
        patience = world.patience(STATE_TIMEOUT_SECONDS)
        while patience.waiting():
            with contextlib.suppress(lib.ServerUnreachable):
                status, health = world.api("GET", "/health")
                if status == 200 and health.get("ok"):
                    break
            time.sleep(POLL_SECONDS)
        else:
            raise SelftestFailure(f"外层单元里的服务 {patience.elapsed():.0f} 秒没起来", world.server_log_lines())
        task_id = world.submit(["bash", "outer-hold.sh", release])
        unit = next((attempt.get("unit") for attempt in world.running(task_id)["attempts"] if attempt.get("state") == "running"), "") or ""
        binds = subprocess.run(["systemctl", "--user", "show", "-p", "BindsTo", "--value", unit], capture_output=True, text=True, stdin=subprocess.DEVNULL,
                               timeout=30).stdout.split()
        if outer not in binds:
            problems.append(f"尝试单元 {unit} 应当 BindsTo 外层单元 {outer}，实际 BindsTo「{' '.join(binds)}」")
        subprocess.run(["systemctl", "--user", "stop", outer], capture_output=True, stdin=subprocess.DEVNULL, timeout=120)
        deadline = time.monotonic() + OUTER_UNIT_STOP_SECONDS
        while unit_active_state(unit) not in STOPPED_UNIT_STATES and time.monotonic() < deadline:
            time.sleep(POLL_SECONDS)
        if unit_active_state(unit) not in STOPPED_UNIT_STATES:
            problems.append(f"外层单元停了 {OUTER_UNIT_STOP_SECONDS} 秒，尝试单元 {unit} 还是 {unit_active_state(unit)}（只能等命令自己超时退）")
    finally:
        release_all([release])
        subprocess.run(["systemctl", "--user", "stop", outer], capture_output=True, stdin=subprocess.DEVNULL, timeout=120)
        if unit and unit_active_state(unit) not in STOPPED_UNIT_STATES:
            subprocess.run(["systemctl", "--user", "stop", unit], capture_output=True, stdin=subprocess.DEVNULL, timeout=200)
    raise_cell_problems(world, problems, "自证里起的单元随外层停")
    return f"外层单元里的服务起的尝试单元 BindsTo 外层单元，停外层之后 {OUTER_UNIT_STOP_SECONDS} 秒里跟着停了"


HEARTBEAT_ONLY_SCRIPT = 'echo started; for round in $(seq 1 40); do echo "TASK_SERVER_HEARTBEAT 编译中 $round"; sleep 0.5; done'   # 20 秒只打心跳行
HEARTBEAT_MIXED_SCRIPT = 'echo started; for round in 1 2 3 4 5 6; do echo "TASK_SERVER_HEARTBEAT 编译中 $round"; echo "real output $round"; sleep 0.5; done'   # 心跳行里夹着真输出


def cell_heartbeat_lines_not_log_growth(world):
    """命令自己打的心跳行不算日志在涨（task_server.py 的 log_content、judge_stall_stop；task_server_lib.py 文件头「命令自己打的两种登记行」）：卡住窗口 2 秒、宽限 0.01 秒，
    假 scope 的 CPU 读得到、一直不涨：每 0.5 秒打一行 TASK_SERVER_HEARTBEAT、跑 20 秒的一条被停、记 stalled；心跳行里夹着每 0.5 秒一行真输出的一条照跑完；
    理由行里的「日志最后一行」跳过心跳行。弄坏开关 heartbeat-counted-as-log 下只打心跳行的那一条也照跑完，这一格红。"""
    scope = os.path.join(world.root, "idle-scope")
    os.makedirs(scope)
    with open(os.path.join(scope, "cpu.stat"), "w", encoding="utf-8") as handle:
        handle.write("usage_usec 1000\n")
    world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01", "TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope})
    only = world.finished(world.submit(["bash", "-c", HEARTBEAT_ONLY_SCRIPT, "heartbeat-only"]), timeout=45)
    mixed = world.finished(world.submit(["bash", "-c", HEARTBEAT_MIXED_SCRIPT, "heartbeat-mixed"]), timeout=45)
    problems = []
    if only["task"].get("outcome") != "stalled":
        problems.append(f"只打心跳行、CPU 不涨的那一条满 2 秒应当被停、记 stalled，实际 {describe(only)}")
    if mixed["task"].get("outcome") != "succeeded":
        problems.append(f"心跳行里夹着真输出的那一条应当照跑完，实际 {describe(mixed)}")
    last = lib.log_last_line("real output 6\nTASK_SERVER_HEARTBEAT 编译中 6\n")
    if last != "real output 6":
        problems.append(f"理由行里的日志最后一行应当跳过心跳行取「real output 6」，实际「{last}」")
    if problems:
        raise SelftestFailure("心跳行不算日志在涨：" + "；".join(problems), world.server_log_lines())
    return "只打心跳行、CPU 不涨的满窗口记 stalled；夹着真输出的照跑完；最后一行跳过心跳行"


VERDICT_PROGRESS_STEADY_SCRIPT = 'echo started; for round in $(seq 1 40); do echo "PRUNE_SERVER_CLIENT_PROGRESS exchanges=$round"; sleep 0.5; done'   # 20 秒里只打判定库进度行
VERDICT_PROGRESS_ONCE_SCRIPT = 'echo started; echo "PRUNE_SERVER_CLIENT_PROGRESS exchanges=1"; sleep 20'   # 只打一行判定库进度行就不再出声


def cell_verdict_library_progress_not_stalled(world):
    """判定库进度行算日志在涨（task_server.py 的 log_content、judge_stall_stop；task_server_lib.py 文件头「命令自己打的两种登记行」的 PRUNE_SERVER_CLIENT_PROGRESS）：
    卡住窗口 2 秒、宽限 0.01 秒，假 scope 的 CPU 读得到、一直不涨：每 0.5 秒打一行 PRUNE_SERVER_CLIENT_PROGRESS、没有别的输出、跑 20 秒的一条照跑完（只在等判定库回话的命令不被判卡住）；
    只打一行就不再出声的一条被停、记 stalled（进度行断了就照常判）；理由行里的「日志最后一行」跳过进度行。
    弄坏开关 verdict-library-progress-uncounted 下进度行也不算日志在涨，只打进度行的那一条被停、记 stalled，这一格红。"""
    scope = os.path.join(world.root, "idle-scope")
    os.makedirs(scope)
    with open(os.path.join(scope, "cpu.stat"), "w", encoding="utf-8") as handle:
        handle.write("usage_usec 1000\n")
    world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01", "TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope})
    steady = world.finished(world.submit(["bash", "-c", VERDICT_PROGRESS_STEADY_SCRIPT, "verdict-progress-steady"]), timeout=45)
    once = world.finished(world.submit(["bash", "-c", VERDICT_PROGRESS_ONCE_SCRIPT, "verdict-progress-once"]), timeout=45)
    problems = []
    if steady["task"].get("outcome") != "succeeded":
        problems.append(f"只打判定库进度行、CPU 不涨、跑 20 秒的那一条应当照跑完（进度行算日志在涨），实际 {describe(steady)}")
    if once["task"].get("outcome") != "stalled":
        problems.append(f"只打一行进度行就不再出声的那一条满 2 秒应当被停、记 stalled，实际 {describe(once)}")
    last = lib.log_last_line("real output 6\nPRUNE_SERVER_CLIENT_PROGRESS exchanges=3\n")
    if last != "real output 6":
        problems.append(f"理由行里的日志最后一行应当跳过判定库进度行取「real output 6」，实际「{last}」")
    if problems:
        raise SelftestFailure("判定库进度行算日志在涨：" + "；".join(problems), world.server_log_lines())
    return "只打进度行的照跑完；只打一行就不出声的满窗口记 stalled；最后一行跳过进度行"


def cell_progress_line_names_progress(world):
    """在跑进度里一次在跑的尝试那一行（task_server.py 的 progress_attempt_line）：起跑以来 90 分钟，日志 1800 秒没涨而 120 秒前读到子进程写着的文件在涨：
    那一行写起跑以来多久、写读到的进展与多少秒前读的、写日志末行。套在假调度器上（running_progress 换成定值），不起服务。
    弄坏开关 progress-line-progress-unnamed 下不写读到的进展，这一格红。"""
    log = os.path.join(world.root, "progress-line.log")
    with open(log, "w", encoding="utf-8") as handle:
        handle.write("头一行\n末一行在这里\n")
    note = "子进程写着的 /selftest/record.log 从 10 涨到 20 字节"
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.running_progress = lambda task, attempt, now: {"elapsed_seconds": 5400.0, "cores": 0.0, "log_quiet_seconds": 1800.0, "progress_note": note, "progress_age_seconds": 120.0}
    line = SERVER.Scheduler.progress_attempt_line(fake, {"id": 17}, {"id": 1, "task_id": 17, "host": SERVER.PEER, "log": log}, time.time())
    wanted = ["起跑以来 90 分钟", note, "120 秒前读到", "末一行在这里"]
    missing = [piece for piece in wanted if piece not in line]
    if missing:
        raise SelftestFailure(f"在跑进度那一行应当写 {wanted}，缺 {missing}，实际「{line}」")
    return f"那一行：{line[:160]}"


def cell_attempt_progress_readings(world):
    """日志不涨时的进展读数（task_server_lib.py 的 attempt_progress_readings、attempt_progress_probe_text）：假 /proc 里进程 4100（树根）→ 4101，另有不在树里的 4999；
    4100 只写着顶层日志（不算），4101 只写着 record.log、只读着 input.txt，4999 写着 outside.log：读数是 record.log 一份（大小照真文件）、
    进程树 2 个。送到第二台的那一份（源码与请求各 base64 一份的 bash 命令）在本机 bash 起，读数逐项相同。
    弄坏开关 open-file-write-mode-unchecked 下只读打开的 input.txt 也算写着的，这一格红。"""
    proc, files = os.path.join(world.root, "progress-proc"), os.path.join(world.root, "progress-files")
    os.makedirs(files, exist_ok=True)
    record, top, source, outside = (os.path.join(files, name) for name in ("record.log", "attempt.log", "input.txt", "outside.log"))
    for path, size in ((record, 10), (top, 20), (source, 30), (outside, 40)):
        with open(path, "w", encoding="utf-8") as handle:
            handle.write("x" * size)
    write_only, read_only = "0100001", "0100000"
    layout = {4100: (1, {"1": (top, write_only)}), 4101: (4100, {"1": (record, write_only), "3": (source, read_only)}), 4999: (1, {"1": (outside, write_only)})}
    for process, (parent, descriptors) in layout.items():
        for part in ("fd", "fdinfo"):
            os.makedirs(os.path.join(proc, str(process), part), exist_ok=True)
        with open(os.path.join(proc, str(process), "stat"), "w", encoding="utf-8") as handle:
            handle.write(f"{process} (fake) S {parent} 0 0\n")
        for descriptor, (target, flags) in descriptors.items():
            os.symlink(target, os.path.join(proc, str(process), "fd", descriptor))
            with open(os.path.join(proc, str(process), "fdinfo", descriptor), "w", encoding="utf-8") as handle:
                handle.write(f"pos:\t0\nflags:\t{flags}\nmnt_id:\t1\n")
    request = {"root_process_ids": [4100], "excluded_files": [top], "proc_root": proc, "file_limit": 16,
               "write_mode_unchecked": lib.broken("open-file-write-mode-unchecked")}
    direct = lib.attempt_progress_readings(request)
    problems = []
    if direct.get("files") != {os.path.realpath(record): 10} or direct.get("processes") != 2:
        problems.append(f"读数应当是写着的 record.log 一份（10 字节）、进程树 2 个，实际 {direct}")
    completed = subprocess.run(["bash", "-c", lib.attempt_progress_probe_text(request)], capture_output=True, text=True, stdin=subprocess.DEVNULL, timeout=60)
    shipped = lib.attempt_progress_probe_reply(completed.stdout)
    if shipped != direct:
        problems.append(f"送到第二台的那一份在本机起应当读数逐项相同，实际退 {completed.returncode}、读数 {shipped}、标准错误「{completed.stderr.strip()[-200:]}」")
    if problems:
        raise SelftestFailure("日志不涨时的进展读数：" + "；".join(problems))
    return "写着的 record.log 一份、进程树 2 个；送到第二台的那一份读数相同"


def progress_quiet_scheduler(readings):
    """quiet-log-busy-spared-by-progress 的假调度器：尝试 7（任务 70）的日志 5000 秒没涨、最近一个窗口因 CPU 在涨没停它（log_growth 的缘由 SPARED_CPU_BUSY），
    卡住阈值 600 秒、日志长期不涨的提醒门槛 1800 秒；本机的进展读数依次交 readings 里的各份（读到最后一份之后一直交它）。→ (假调度器, 读数开始的那一刻（monotonic）)。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.stall_seconds, fake.quiet_attention_seconds = 600.0, 1800.0
    now = time.monotonic()
    fake.log_growth = {7: (100, now - 5000, now - 600, 1.0, SERVER.SPARED_CPU_BUSY)}
    fake.progress_read_at, fake.progress_files, fake.progress_noted = {}, {}, {}
    pending = list(readings)
    fake.machines = {SERVER.LOCAL: types.SimpleNamespace(attempt_progress_readings=lambda attempt: (pending.pop(0) if len(pending) > 1 else pending[0], ""))}
    return fake, now


def cell_quiet_log_busy_spared_by_progress(world):
    """日志不涨而有进展的不报 quiet-log-busy（task_server.py 的 note_attempt_progress、quiet_log_items；任务 17334 顶层日志 66200 秒没涨而测试进程写着的 record.log 在涨）：
    尝试 7 日志 5000 秒没涨、CPU 在涨。子进程写着的 record.log 隔 400 秒读两回从 100 涨到 250 字节：记下进展（写出那份文件与两个大小）、不报；
    对照：record.log 两回都是 100 字节：报一项 quiet-log-busy。套在假调度器上，不起服务。
    弄坏开关 open-file-growth-ignored 下不看文件（头一种照报），quiet-log-busy-progress-ignored 下不看读到的进展（照报），这一格红。"""
    attempt = {"id": 7, "task_id": 70, "number": 1, "host": SERVER.LOCAL, "state": "running"}
    record = "/selftest-run/record.log"
    problems = []
    for what, readings, spared, wanted in (
            ("record.log 在涨", [{"files": {record: 100}}, {"files": {record: 250}}], True, [record, "100", "250"]),
            ("都没动", [{"files": {record: 100}}, {"files": {record: 100}}], False, [])):
        fake, now = progress_quiet_scheduler(readings)
        SERVER.Scheduler.note_attempt_progress(fake, attempt, now - 400)
        SERVER.Scheduler.note_attempt_progress(fake, attempt, now)
        noted = fake.progress_noted.get(7)
        items = SERVER.Scheduler.quiet_log_items(fake, {"id": 70}, [attempt])
        kinds = [item.get("kind") for item in items]
        if spared and (noted is None or any(piece not in noted[1] for piece in wanted) or kinds):
            problems.append(f"{what}：应当记下进展（写出 {wanted}）、不报 quiet-log-busy，实际记下「{noted[1] if noted else '没有'}」、报了 {kinds}")
        if not spared and (noted is not None or kinds != ["quiet-log-busy"]):
            problems.append(f"{what}：不该记进展、应当报一项 quiet-log-busy，实际记下「{noted[1] if noted else '没有'}」、报了 {kinds}")
    if problems:
        raise SelftestFailure("日志不涨而有进展：" + "；".join(problems))
    return "写着的 record.log 在涨的记下进展、不报 quiet-log-busy；没动的照报"


def cell_peer_ssh_multiplexed(world):
    """服务起的 ssh 一律复用连接（task_server_lib.py 的 enable_ssh_multiplexing、ssh_multiplex_options、peer_shell_timed、ssh_multiplexed_environment、ssh_shim_text；
    第二台 sshd 的 MaxStartups 被一条条新连接打满过）：在一个短的临时目录当服务状态目录起复用，控制目录是它下面的 ssh/：
    服务自己起的那一路（peer_shell，ssh 换成记参数的假 ssh）参数里带 ControlMaster=auto、ControlPath=<控制目录>/%C、ControlPersist=<秒>，主机名是调用方给的多机配置第二台那一项；
    host-dispatch.py 那一路（ssh_multiplexed_environment 之后的环境里 bash 起 PATH 上的 ssh）：PATH 头一项是控制目录下的 bin，复用层起 PATH 上下一个 ssh（记参数的假的），参数是这三项
    接原来的参数；复用层正文里没有写死的主机名。控制目录放不下 socket 路径的判出来（不复用），环境里没记控制目录的（control 这类不是服务起的）不接。
    弄坏开关 ssh-unmultiplexed 下两路都不接这三项，这一格红。"""
    # 控制 socket 路径有上限：状态目录要短（系统临时目录接上 tsm-<8 个字符>/ssh/<%C>.<临时后缀> 放不下的用 /tmp）
    base = tempfile.gettempdir()
    if len(os.fsencode(base)) + len("/tsm-12345678/") + len(lib.SSH_CONTROL_DIRECTORY_NAME) + 1 + lib.SSH_CONTROL_HASH_CHARACTERS + lib.SSH_CONTROL_TEMPORARY_SUFFIX_CHARACTERS \
            > lib.SOCKET_PATH_MAXIMUM_BYTES:
        base = FALLBACK_TEMPORARY_BASE
    state = tempfile.mkdtemp(prefix="tsm-", dir=base)
    problems = []
    try:
        environment = dict(os.environ)
        problem = lib.enable_ssh_multiplexing(environment, state)
        control = os.path.join(state, lib.SSH_CONTROL_DIRECTORY_NAME)
        expected = ["-o", "ControlMaster=auto", "-o", f"ControlPath={control}/%C", "-o", f"ControlPersist={lib.SSH_CONTROL_PERSIST_SECONDS}"]
        if problem or environment.get(lib.SSH_CONTROL_VARIABLE) != control:
            problems.append(f"短的状态目录应当起得了复用、记下控制目录 {control}，实际「{problem}」、记的 {environment.get(lib.SSH_CONTROL_VARIABLE)}")
        recorder = '#!/usr/bin/env bash\nprintf "%s\\n" "$@" > "$(dirname "$0")/argv"\n'
        direct_directory, next_directory = os.path.join(state, "direct"), os.path.join(state, "next")
        for directory in (direct_directory, next_directory):
            os.makedirs(directory)
            with open(os.path.join(directory, "ssh"), "w", encoding="utf-8") as handle:
                handle.write(recorder)
            os.chmod(os.path.join(directory, "ssh"), 0o755)

        def recorded(directory):
            with contextlib.suppress(OSError):
                with open(os.path.join(directory, "argv"), encoding="utf-8") as handle:
                    return handle.read().splitlines()
            return []
        lib.peer_shell(SELFTEST_PEER_ADDRESS, "true", dict(environment, TASK_SERVER_PEER_SSH=os.path.join(direct_directory, "ssh")))
        direct = recorded(direct_directory)
        if direct != [*lib.NFS_SSH_OPTIONS, *expected, SELFTEST_PEER_ADDRESS, "true"]:
            problems.append(f"服务自己起的 ssh 参数应当是 {[*lib.NFS_SSH_OPTIONS, *expected, SELFTEST_PEER_ADDRESS, 'true']}，实际 {direct}")
        merged = lib.ssh_multiplexed_environment({"PATH": os.pathsep.join([next_directory, "/usr/bin", "/bin"])}, environment)
        subprocess.run(["bash", "-c", 'ssh -o BatchMode=yes "$0" true', SELFTEST_PEER_ADDRESS], env=merged, capture_output=True, text=True, stdin=subprocess.DEVNULL, timeout=30)
        relayed = recorded(next_directory)
        if merged.get("PATH", "").split(os.pathsep)[0] != os.path.join(control, lib.SSH_SHIM_DIRECTORY_NAME) or relayed != [*expected, "-o", "BatchMode=yes", SELFTEST_PEER_ADDRESS, "true"]:
            problems.append(f"host-dispatch.py 那一路：PATH 头一项应当是 {os.path.join(control, lib.SSH_SHIM_DIRECTORY_NAME)}、下一个 ssh 收到的参数应当是三项接原来的参数，"
                            f"实际 PATH「{merged.get('PATH')}」、参数 {relayed}")
        with contextlib.suppress(OSError):
            with open(os.path.join(control, lib.SSH_SHIM_DIRECTORY_NAME, "ssh"), encoding="utf-8") as handle:
                if SELFTEST_PEER_ADDRESS in handle.read():
                    problems.append("复用层正文里不该写死主机名")
        if not lib.ssh_control_path_problem(os.path.join("/" + "x" * 80, lib.SSH_CONTROL_DIRECTORY_NAME)):
            problems.append("80 字节的状态目录接上 %C 与临时后缀放不下 socket 路径，应当判出来，实际没判")
        if lib.ssh_multiplex_options({}):
            problems.append(f"环境里没记控制目录的不该接复用的三项，实际 {lib.ssh_multiplex_options({})}")
    finally:
        shutil.rmtree(state, ignore_errors=True)
    if problems:
        raise SelftestFailure("服务起的 ssh 复用连接：" + "；".join(problems))
    return f"两路 ssh 都带 ControlMaster=auto、ControlPath=<状态目录>/ssh/%C、ControlPersist={lib.SSH_CONTROL_PERSIST_SECONDS}；主机名照调用方给的"


def unit_unreadable_lines(world, attempt_id, presence=""):
    """这一格各次服务输出里 TASK_UNIT_UNREADABLE 起头、点名这一次尝试（attempt=<记录号>）的那几行（task_server.py 的 note_unit_unreadable）；给了 presence 的只要 presence=<它> 的。"""
    lines = []
    for number in range(1, world.server_logs + 1):
        text = read_text(os.path.join(world.root, f"server-{number}.log"))
        lines += [line for line in text.splitlines() if line.startswith("TASK_UNIT_UNREADABLE ") and f" attempt={attempt_id} " in line
                  and (not presence or f" presence={presence} " in line)]
    return lines


def wait_unit_unreadable_line(world, attempt_id, presence=""):
    """等服务日志打出点名这一次尝试的 TASK_UNIT_UNREADABLE 行（PLACEMENT_WAIT_SECONDS 的安静时限）；交此刻有的那几行（没有交空表，调用处判）。"""
    patience = world.patience(PLACEMENT_WAIT_SECONDS)
    while not unit_unreadable_lines(world, attempt_id, presence) and patience.waiting():
        time.sleep(POLL_SECONDS)
    return unit_unreadable_lines(world, attempt_id, presence)


UNIT_UNREADABLE_WAIT_SCRIPT = 'for round in $(seq 1 2000); do [[ -e "$1" ]] && exit "$2"; sleep 0.1; done; exit 9'   # 等放行文件在了再退给的码（至多 200 秒）
UNIT_UNREADABLE_HOLD_SECONDS = 3.0   # unit-unreadable-unverified-health-reported：/health 报了之后再看这么久（对账十来轮、过了 1 秒的宽限），核不出的那一次仍该照记 running


def cell_unit_unreadable_exit_file_settles(world):
    """读不到单元状态、留了 result.json 的照它的退出码收尾（task_server.py 文件头 ①「读不到单元状态」）：一条命令跑起来之后世界目录里放 unit-states-unreadable
    （unit_states 一律交 None），等服务日志打出点名它的 TASK_UNIT_UNREADABLE 行（进了读不到的那一段）再放它退 3：收尾 command-failed、退 3、只一次。
    弄坏开关 unit-unreadable-exit-file-unread 下读不到的那一段不认 result.json，命令退了、单元收掉之后照没留结果记 launcher-lost，这一格红。"""
    world.start_server()
    unreadable, release = (os.path.join(world.world, name) for name in (SERVER.UNIT_STATES_UNREADABLE_FILE, "unit-unreadable-release"))
    problems = []
    task_id = world.submit(["bash", "-c", UNIT_UNREADABLE_WAIT_SCRIPT, "unit-unreadable-exit-file", release, "3"])
    try:
        attempt = first_attempt(world.running(task_id))
        open(unreadable, "w").close()
        if not wait_unit_unreadable_line(world, attempt.get("id")):
            problems.append(f"读不到单元状态时服务日志应当打点名第 1 次（attempt={attempt.get('id')}）的 TASK_UNIT_UNREADABLE 行，实际没有")
    finally:
        open(release, "w").close()
    try:
        done = world.finished(task_id, timeout=PLACEMENT_WAIT_SECONDS)
    finally:
        with contextlib.suppress(FileNotFoundError):
            os.remove(unreadable)
    first = first_attempt(done)
    if done["task"].get("outcome") != "command-failed" or first.get("exit_code") != 3 or len(done["attempts"]) != 1:
        problems.append(f"留了退出码的那一次应当照它收尾（command-failed、退 3、只一次），实际 {describe(done)}、第 1 次 {first.get('outcome')}、退 {first.get('exit_code')}、"
                        f"{len(done['attempts'])} 次")
    raise_cell_problems(world, problems, "读不到单元状态、留了退出码的那一次")
    return "读不到单元状态时留了 result.json 的照它的退出码收尾：command-failed、退 3、只一次"


def cell_unit_unreadable_gone_recorded_lost(world):
    """读不到单元状态、没留 result.json、不经 systemd 核到都没了的记 launcher-lost（task_server.py 文件头 ①「读不到单元状态」）：世界目录里先放 unit-states-unreadable，
    再交一条把起它的 attempt_runner 杀掉（kill -9 $PPID）、睡 2 秒的：起跑标记记的 runner 进程号不在、单元收掉之后它的 cgroup 在本机用户级管理器的 cgroup 树里找不到（算空），
    满宽限记 launcher-lost（第 1 次 ended、launcher-lost），服务日志有点名它的 presence=gone 那一行。
    弄坏开关 unit-unreadable-absent-cgroup-unconfirmed 下不在的 cgroup 目录算核不出，那一次一直照记 running、等收尾等满安静时限，这一格红。"""
    world.start_server()
    unreadable = os.path.join(world.world, SERVER.UNIT_STATES_UNREADABLE_FILE)
    open(unreadable, "w").close()
    problems = []
    try:
        task_id = world.submit(["bash", "-c", 'kill -9 "$PPID"; sleep 2', "unit-unreadable-gone"])
        done = world.finished(task_id, timeout=PLACEMENT_WAIT_SECONDS)
    finally:
        os.remove(unreadable)
    first = first_attempt(done)
    if done["task"].get("outcome") != "launcher-lost" or first.get("state") != "ended" or first.get("outcome") != "launcher-lost":
        problems.append(f"核到都没了的那一次应当记 launcher-lost，实际 {describe(done)}、第 1 次 {first.get('state')}/{first.get('outcome')}")
    if not unit_unreadable_lines(world, first.get("id"), lib.PRESENCE_GONE):
        problems.append(f"照没留结果收尾之前服务日志应当打点名它（attempt={first.get('id')}）、presence={lib.PRESENCE_GONE} 的 TASK_UNIT_UNREADABLE 行（核到都不在），"
                        f"实际点名它的是 {unit_unreadable_lines(world, first.get('id'))[-3:]}")
    raise_cell_problems(world, problems, "读不到单元状态、核到都没了的那一次")
    return "读不到单元状态、没留结果：不经 systemd 核到 runner 进程号不在、单元的 cgroup 已收掉之后记 launcher-lost，服务日志有 presence=gone 那一行"


def cell_unit_unreadable_running_reminded(world):
    """读不到单元状态、进程还在的不杀、照记 running、提醒发起任务的会话（task_server.py 文件头 ①「读不到单元状态」与「要处置的提醒」的 unit-unreadable）：
    会话的收件 socket（FakeInbox）在，一条命令跑起来之后世界目录里放 unit-states-unreadable：起跑标记记的 runner 进程号在、单元的 cgroup 里有进程，核到还在；
    满 FAST_UNIT_UNREADABLE_ATTENTION_SECONDS（2 秒）收到恰好一条「task-server：提醒（任务还在跑，不是结局）任务 <号> 读不到单元状态（unit-unreadable；第 1 回」（reminded_after_seconds=2），
    依据写本机、单元名、读不到什么与核到的进程号，再看几轮不多一条；满 4 秒再收一条 reminded_after_seconds=4 的；这一段里任务与那一次尝试都照记 running。放行之后照 result.json 收尾 succeeded。
    弄坏开关 unit-unreadable-lost-unverified 下读不到就当场照没留结果收尾、停掉单元里的进程（launcher-lost），unit-unreadable-unreminded 下收不到提醒，
    unit-unreadable-exit-file-unread 下放行之后不认 result.json、记 launcher-lost，这一格红。"""
    inbox = FakeInbox(world, "a")
    start_inbox_server(world)
    unreadable, release = (os.path.join(world.world, name) for name in (SERVER.UNIT_STATES_UNREADABLE_FILE, "unit-unreadable-release"))
    problems = []
    task_id = world.submit(["bash", "-c", UNIT_UNREADABLE_WAIT_SCRIPT, "unit-unreadable-running", release, "0"], extra=inbox.environment())
    try:
        attempt = first_attempt(world.running(task_id))
        open(unreadable, "w").close()
        head = reminder_head(task_id, "unit-unreadable")
        first_after, second_after = round(SERVER.FAST_UNIT_UNREADABLE_ATTENTION_SECONDS), round(2 * SERVER.FAST_UNIT_UNREADABLE_ATTENTION_SECONDS)
        inbox.wait_for(world, head)
        time.sleep(INBOX_QUIET_SECONDS)
        first = reminders_after(inbox.headed(head), first_after)
        problems += inbox_problems(first, head, f"读不到单元状态满 {first_after} 秒那一回")
        text = first[0]["text"] if first else ""
        for wanted in ("依据：", "本机读不到它的单元", str(attempt.get("unit")), "进程号", "出路：",
                       f"{lib.ATTENTION_MARK} task={task_id} type={lib.ATTENTION_TYPE} kind=unit-unreadable reminded_after_seconds={first_after} basis="):
            if wanted not in text:
                problems.append(f"读不到单元状态那一条的正文里应当有「{wanted}」，实际「{text[:500]}」")
        inbox.wait_for(world, head, count=2)
        time.sleep(INBOX_QUIET_SECONDS)
        problems += inbox_problems(reminders_after(inbox.headed(head), second_after), head, f"读不到单元状态满 {second_after} 秒那一回")
        if len(reminders_after(inbox.headed(head), first_after)) != 1:
            problems.append(f"满 {second_after} 秒之后，{first_after} 秒那一条仍该只有一条，实际 {len(reminders_after(inbox.headed(head), first_after))} 条")
        during = world.summary(task_id)
        if during["task"].get("state") != "running" or first_attempt(during).get("state") != "running":
            problems.append(f"读不到单元状态而进程还在的那一次应当照记 running，实际任务 {during['task'].get('state')}、第 1 次 {first_attempt(during).get('state')}"
                            f"/{first_attempt(during).get('outcome')}")
    finally:
        open(release, "w").close()
    try:
        done = world.finished(task_id, timeout=PLACEMENT_WAIT_SECONDS)
    finally:
        with contextlib.suppress(FileNotFoundError):
            os.remove(unreadable)
    if done["task"].get("outcome") != "succeeded" or len(done["attempts"]) != 1:
        problems.append(f"放行之后应当照 result.json 收尾 succeeded、只一次，实际 {describe(done)}、{len(done['attempts'])} 次")
    raise_cell_problems(world, problems, "读不到单元状态、进程还在的那一次")
    return "读不到单元状态而进程还在：不杀、照记 running，满 2 秒、满 4 秒各推恰好一条 unit-unreadable（依据写本机、单元与核到的进程号），放行后照退出码收尾 succeeded"


def cell_unit_unreadable_unverified_health_reported(world):
    """读不到单元状态、没留 result.json、cgroup 也读不到的一直等（task_server.py 文件头 ①「读不到单元状态」）：世界目录里先放 unit-states-unreadable 与 unit-cgroups-unreadable
    （本机核进程的 cgroup 根换成不在的一处），再交一条把起它的 attempt_runner 杀掉、睡 1 秒的：runner 进程号不在而 cgroup 核不出，不杀、不收尾，那一次照记 running；
    /health 回不健康，problem 写本机读不到在跑尝试的单元状态，unit_unreadable 栏点名这一条（presence=unverified）；control check 退 78、报同一句；
    再看 UNIT_UNREADABLE_HOLD_SECONDS 仍照记 running。撤掉两份文件之后照读得到的单元状态判，单元不在了记 launcher-lost。
    弄坏开关 unit-unreadable-health-unreported 下 /health 不报，unit-unreadable-lost-unverified 下当场记 launcher-lost，这一格红。"""
    world.start_server()
    flags = [os.path.join(world.world, name) for name in (SERVER.UNIT_STATES_UNREADABLE_FILE, SERVER.UNIT_CGROUPS_UNREADABLE_FILE)]
    release_all(flags)
    problems = []
    try:
        task_id = world.submit(["bash", "-c", 'kill -9 "$PPID"; sleep 1', "unit-unreadable-unverified"])
        health = health_until(world, lambda reading: any(entry.get("task") == task_id and entry.get("presence") == lib.PRESENCE_UNVERIFIED
                                                         for entry in reading.get("unit_unreadable") or []), PLACEMENT_WAIT_SECONDS)
        entries = [entry for entry in health.get("unit_unreadable") or [] if entry.get("task") == task_id]
        if (health.get("ok") or SERVER.UNIT_UNREADABLE_HEALTH_TEXT not in (health.get("problem") or "") or not entries
                or entries[0].get("presence") != lib.PRESENCE_UNVERIFIED):
            problems.append(f"读不到单元状态、cgroup 也读不到时 /health 应当回不健康、problem 写「{SERVER.UNIT_UNREADABLE_HEALTH_TEXT}」、unit_unreadable 栏点名任务 {task_id}"
                            f"（presence={lib.PRESENCE_UNVERIFIED}），实际 ok={health.get('ok')}、problem「{(health.get('problem') or '')[:300]}」、栏 {entries}")
        checked = world.control(["check"])
        if checked.returncode != 78 or SERVER.UNIT_UNREADABLE_HEALTH_TEXT not in checked.stderr:
            problems.append(f"control check 应当退 78、报「{SERVER.UNIT_UNREADABLE_HEALTH_TEXT}」，实际退 {checked.returncode}、「{(checked.stdout + checked.stderr).strip()[-300:]}」")
        time.sleep(UNIT_UNREADABLE_HOLD_SECONDS)
        during = world.summary(task_id)
        if during["task"].get("state") != "running" or first_attempt(during).get("state") not in SERVER.ACTIVE_ATTEMPT_STATES:
            problems.append(f"核不出的那一次应当一直照记 running（不杀、不收尾），实际任务 {during['task'].get('state')}、第 1 次 {first_attempt(during).get('state')}"
                            f"/{first_attempt(during).get('outcome')}")
    finally:
        for path in flags:
            with contextlib.suppress(FileNotFoundError):
                os.remove(path)
    done = world.finished(task_id, timeout=PLACEMENT_WAIT_SECONDS)
    if done["task"].get("outcome") != "launcher-lost":
        problems.append(f"撤掉两份文件之后单元不在了应当记 launcher-lost，实际 {describe(done)}")
    raise_cell_problems(world, problems, "读不到单元状态、cgroup 也读不到的那一次")
    return (f"读不到单元状态、cgroup 也读不到：照记 running 不杀，/health 报「{SERVER.UNIT_UNREADABLE_HEALTH_TEXT}…」、unit_unreadable 栏 presence={lib.PRESENCE_UNVERIFIED}，"
            "control check 退 78；读得到之后照单元不在记 launcher-lost")


def cell_peer_readings_merged_across_reads(world):
    """两次读之间读日志几回，IO 等待与线程读数一行不丢（task_server.py 的 read_log_content、merge_peer_io_rounds、merge_peer_thread_rounds）：读一回日志接到 blocked=0、
    runnable=9 那一行，再读一回接到 blocked=3、runnable=2 那一行，之后 io_blocked 交 0（取最少）、thread_reading 的可跑线程数 9（取最大）。套在假调度器上，不起服务。
    弄坏开关 peer-readings-overwritten 下后一回盖掉前一回（交 3 与 2），这一格红。"""
    fake, attempt = peer_log_scheduler(world, "merged", ["MULTI_HOST_RUN_PEER_IO_WAIT poll=1 blocked=0 read=yes",
                                                         "MULTI_HOST_RUN_PEER_THREADS poll=1 runnable=9 total=20 top=12:cargo read=yes"])
    SERVER.Scheduler.log_content(fake, attempt)
    append_lines(attempt["log"], ["MULTI_HOST_RUN_PEER_IO_WAIT poll=2 blocked=3 read=yes", "MULTI_HOST_RUN_PEER_THREADS poll=2 runnable=2 total=5 top=3:rustc read=yes"])
    SERVER.Scheduler.log_content(fake, attempt)
    machine = fake.machines[SERVER.PEER]
    blocked, threads = machine.io_blocked(attempt), machine.thread_reading(attempt)
    if blocked != 0 or threads is None or threads.runnable != 9:
        raise SelftestFailure(f"两回读到的 IO 等待应当并起来取最少（0）、可跑线程取最大（9），实际 {blocked}、{threads}")
    return "两回读到的 IO 等待并起来取 0、可跑线程取 9"


def cell_log_content_serialized(world):
    """同一次尝试同一时刻只一个线程读日志（task_server.py 的 log_content、LOG_CONTENT_LOCKS）：两个线程同时读（读那一段换成睡 0.3 秒、记同时在读几个的假的）：至多一个在读。
    套在假调度器上，不起服务。弄坏开关 log-content-unlocked 下两个同时读，这一格红。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    guard, active, peak = threading.Lock(), [0], [0]

    def slow_read(attempt):
        with guard:
            active[0] += 1
            peak[0] = max(peak[0], active[0])
        time.sleep(0.3)
        with guard:
            active[0] -= 1
        return 0
    fake.read_log_content = slow_read
    readers = [threading.Thread(target=SERVER.Scheduler.log_content, args=(fake, {"id": 7})) for _ in range(2)]
    for reader in readers:
        reader.start()
    for reader in readers:
        reader.join(10)
    if peak[0] != 1:
        raise SelftestFailure(f"同一次尝试的日志同一时刻应当只一个线程在读，实际同时 {peak[0]} 个")
    return "两个线程读同一次尝试的日志一个接一个"


def cell_log_noise_marks_not_growth(world):
    """第二台收尾那几行与 multi-host-run.sh 每一轮照打的显存行不算日志在涨（task_server.py 的 read_log_content 的 noise_marks）：日志里只有 MULTI_HOST_RUN_PEER_MEMORY_PEAK、
    MULTI_HOST_RUN_PEER_BUILD_PEAK、MULTI_HOST_RUN_PEER_BUILD_KEPT 与 MULTI_HOST_RUN_PEER_GPU 各一行：算「在涨」的字节数是 0；另接一行命令自己的输出：等于那一行的字节数。
    套在假调度器上，不起服务。弄坏开关 log-noise-marks-incomplete 下收尾那三行照算，这一格红。"""
    lines = [f"{lib.PEER_MEMORY_PEAK_MARK} bytes=1024", f"{lib.PEER_BUILD_PEAK_MARK} bytes=2048", f"{lib.PEER_BUILD_KEPT_MARK} label=task-selftest-1-1",
             "MULTI_HOST_RUN_PEER_GPU poll=1 used_mib=0 read=yes utilization=none on_cards=empty"]
    fake, attempt = peer_log_scheduler(world, "noise-marks", lines)
    quiet = SERVER.Scheduler.log_content(fake, attempt)
    output = "命令自己的一行输出"
    append_lines(attempt["log"], [output])
    grown = SERVER.Scheduler.log_content(fake, attempt)
    if quiet != 0 or grown != len((output + "\n").encode("utf-8")):
        raise SelftestFailure(f"收尾三行与显存行应当都不算在涨（0 字节）、接一行输出之后是那一行的字节数，实际 {quiet}、{grown}")
    return "收尾的峰值行、留着编译目录行与每轮的显存行不算日志在涨"


def cell_runaway_targets_skip_children(world):
    """空转进程的提醒找得到会话（task_server.py 的 runaway_targets）：会话 A 先交一条带收件地址的任务，之后它的分片任务建了 25 个子任务（没有收件地址）：
    找到的是那条带收件地址的任务。套在假调度器上（这一格临时库上的 Store），不起服务。弄坏开关 runaway-targets-counting-children 下子任务占满最近 20 行、找不到，这一格红。"""
    store = SERVER.Store(os.path.join(world.root, "runaway-targets.sqlite3"), bytes(32), dict(os.environ))
    parent = store.insert("tasks", {"task_id": 0, "state": "sharded", "session": SESSION_A, "command_key": "", "inbox": {"token": INBOX_TOKEN_SAMPLE}})
    for index in range(25):
        store.insert("tasks", {"task_id": parent["id"], "parent_id": parent["id"], "role": "shard", "shard_index": index, "state": "queued", "session": SESSION_A, "command_key": ""})
    fake = object.__new__(SERVER.Scheduler)
    fake.environment, fake.store = dict(os.environ), store
    fake.inbox_reachable = lambda task: bool(task.get("inbox"))
    found = [task.get("id") for task in SERVER.Scheduler.runaway_targets(fake, SESSION_A)]
    if found != [parent["id"]]:
        raise SelftestFailure(f"会话最近交了 25 片的分片任务：应当找到带收件地址的任务 {parent['id']}，实际 {found}")
    return "子任务不占最近那 20 行，找得到带收件地址的任务"


def cell_runner_start_error_recorded(world):
    """命令起不来照写 result.json（attempt_runner.py 的 main）：argv 头一个不在：attempt_runner 退 127、result.json 的 exit_code 127、带 start_error，日志记一行「命令起不来」。
    弄坏开关 runner-start-error-uncaught 下（说明里 start_error_uncaught）照抛出去、result.json 写不出，这一格红。"""
    directory = os.path.join(world.work, "runner-start-error")
    os.makedirs(directory, exist_ok=True)
    log = os.path.join(directory, "attempt.log")
    specification_path = os.path.join(directory, "spec.json")
    with open(specification_path, "w", encoding="utf-8") as handle:
        json.dump({"argv": [os.path.join(world.work, "no-such-command")], "cwd": world.work, "environment": dict(os.environ), "log": log, "directory": directory,
                   "fast": True, "unit": "", "start_error_uncaught": lib.broken("runner-start-error-uncaught")}, handle)
    completed = subprocess.run([sys.executable, "-B", lib.RUNNER_SCRIPT, specification_path], stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=60)
    result, logged = None, ""
    with contextlib.suppress(OSError, ValueError):
        with open(os.path.join(directory, "result.json"), encoding="utf-8") as handle:
            result = json.load(handle)
    with contextlib.suppress(OSError):
        with open(log, encoding="utf-8", errors="replace") as handle:
            logged = handle.read()
    if completed.returncode != 127 or not isinstance(result, dict) or result.get("exit_code") != 127 or not result.get("start_error") or "命令起不来" not in logged:
        raise SelftestFailure(f"命令起不来：应当退 127、result.json 的 exit_code 127 且带 start_error、日志记一行，实际退 {completed.returncode}、"
                              f"result.json {'写不出' if result is None else result}、日志「{logged.strip()[-200:]}」", (completed.stdout + completed.stderr).splitlines()[-12:])
    return "命令起不来：退 127，result.json 照写、带原因"


RUNNER_FINAL_SAMPLE_WRAPPER = """import importlib.util, json, os, sys
specification = importlib.util.spec_from_file_location("attempt_runner_under_test", sys.argv[1])
runner = importlib.util.module_from_spec(specification)
specification.loader.exec_module(runner)
record = sys.argv[2]
seen = []
def sample(self):
    seen.append(os.path.exists(f"/proc/{self.root_process_id}"))
    self.memory_peak = 1
runner.Sampler.sample = sample
code = runner.main(sys.argv[3:])
with open(record, "w", encoding="utf-8") as handle:
    json.dump(seen, handle)
sys.exit(code)
"""   # runner-final-sample-after-exit：载入 attempt_runner.py、把 Sampler.sample 换成记「这一刻起的那个子进程还在不在 /proc」的假的，再照原样起 main


def cell_runner_final_sample_after_exit(world):
    """命令退了之后再采一次（attempt_runner.py 的 main）：采样换成记下子进程此刻在不在 /proc 的假的（头一次就记上内存峰值），命令睡 0.3 秒退 0：采样记录里最后一次是子进程
    已收走之后（不在 /proc）。弄坏开关 runner-final-sample-skipped 下（说明里 final_sample_skipped）采到过就不补，最后一次还在跑着，这一格红。"""
    directory = os.path.join(world.work, "runner-final-sample")
    os.makedirs(directory, exist_ok=True)
    wrapper, record = os.path.join(world.work, "runner-final-sample.py"), os.path.join(directory, "samples.json")
    with open(wrapper, "w", encoding="utf-8") as handle:
        handle.write(RUNNER_FINAL_SAMPLE_WRAPPER)
    specification_path = os.path.join(directory, "spec.json")
    with open(specification_path, "w", encoding="utf-8") as handle:
        json.dump({"argv": ["bash", "-c", "sleep 0.3"], "cwd": world.work, "environment": dict(os.environ), "log": os.path.join(directory, "attempt.log"),
                   "directory": directory, "fast": True, "unit": "", "leftovers_kept": True, "final_sample_skipped": lib.broken("runner-final-sample-skipped")}, handle)
    completed = subprocess.run([sys.executable, "-B", wrapper, lib.RUNNER_SCRIPT, record, specification_path], stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=60)
    seen = None
    with contextlib.suppress(OSError, ValueError):
        with open(record, encoding="utf-8") as handle:
            seen = json.load(handle)
    if completed.returncode != 0 or not seen or seen[-1] is not False:
        raise SelftestFailure(f"命令退了之后应当再采一次（最后一次采样时子进程已不在），实际退 {completed.returncode}、采样记录 {seen}",
                              (completed.stdout + completed.stderr).splitlines()[-12:])
    return f"采了 {len(seen)} 次，最后一次在命令退了之后"
