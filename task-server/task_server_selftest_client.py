"""task_server_selftest.py 拆出的一份：客户端、control、错误类型与启动的各格（总说明在 task_server_selftest.py 文件头）。"""
import os
import sys
import contextlib
import hashlib
import io
import json
import re
import sqlite3
import subprocess
import threading
import time
import types
import task_server_lib as lib
from task_server_selftest_fakes import (
    BLOCK_SCRIPT, CONFIRM_RETRIES, GIBIBYTE, HOLD_UNTIL_RELEASED_SCRIPT, MEBIBYTE, POLL_SECONDS, SEEDED_ENDED_AT, SERVER, SESSION_A, SESSION_B,
    SOCKET_ROOT_PREFIX, STATE_TIMEOUT_SECONDS, WAIT_CEILING_FACTOR, WORLD_SOCKET_PREFIX,
)
from task_server_selftest_world import (
    FakeInbox, SelftestFailure, World, blocking_command, describe, first_attempt, raw_column_update, seed_ended_attempt, server_modules,
    shard_children, shown_local_pin, systemctl_start_count, verified_row, wait_last_line,
)


SESSION_FORGED = "selftest-session-forged"

SOCKET_CONNECT_SECONDS = 10.0     # restart-keeps-running：等 wait 连上服务的安静时限
DISCONNECT_NOTICE_SECONDS = 20.0  # restart-keeps-running：服务停着时等 wait 打「连不上」的安静时限（它隔 RECONNECT_SECONDS 2 秒重连一次，留够机器忙时的余地）
UNREACHABLE_NOTICE = "task-server 连不上"   # task 的 wait 重连时往标准错误打的那一句的开头
SLOW_WORLD_BASE_SECONDS = 6.0     # slow-world-patient-wait：这一格的安静时限
SLOW_WORLD_STEP_SECONDS = 2       # 慢世界里命令每一步睡多久（小于安静时限，大于 0）
SLOW_WORLD_STEPS = 5              # 一共几步：总共跑 SLOW_WORLD_STEP_SECONDS × SLOW_WORLD_STEPS 秒，超过安静时限

LONGEST_RUNTIME_NAME = "install-run"   # 各格 socket 目录里最长的那一个名字（World 的 run 与 install 那一格的 install-run）
RANDOM_NAME_CHARACTERS = 8             # tempfile.mkdtemp 在前缀后面接的随机字符数
SOCKET_PATH_ALLOWANCE = (len("/" + SOCKET_ROOT_PREFIX) + RANDOM_NAME_CHARACTERS + len("/" + WORLD_SOCKET_PREFIX) + RANDOM_NAME_CHARACTERS
                         + len("/" + LONGEST_RUNTIME_NAME + "/") + len(lib.SOCKET_NAME))

TOO_DEEP_SOCKET_DIRECTORY = "socket-directory-made-too-deep"

# 钉住 task 的 LOG_TAIL_LINES（log 不带 --all 打最后几行）：改它先改这里；每次尝试打的行比它多，截得出来
LOG_TAIL_PINNED_LINES = 200
LOG_LINES_PER_ATTEMPT = 250
# 第 1 次尝试命令自己退 254（被总上限挤掉，同一上限重排），第 2 次退 0：一件两次尝试，各打 LOG_LINES_PER_ATTEMPT 行「attempt-<第几次> line-<行号>」
LOG_SCRIPT = 'for line in $(seq 1 "$1"); do echo "attempt-$TASK_SERVER_ATTEMPT line-$line"; done; [[ "$TASK_SERVER_ATTEMPT" == 1 ]] && exit 254; exit 0'
ABSENT_TASK_ID = 9999   # 这一格的库里没有这个号（一格只交几件）

# 错误类型 → 喂到它的那一格（type-table-covered 静态核这张表与 task_server_lib.OUTCOME_TYPES 逐个对上）
EXERCISED_BY = {
    "succeeded": "error-types", "command-failed": "error-types", "memory-cap-hit": "error-types", "memory-cap-at-maximum": "memory-cap-at-maximum",
    "squeezed-out": "error-types", "timed-out": "error-types",
    "wrapper-refused": "error-types", "run-condition-unmet": "error-types", "not-run": "error-types", "peer-run-failed": "error-types",
    "launcher-lost": "error-types", "cancelled": "cancel-own-session-only", "record-tampered": "tampered-row-not-started",
    "never-fits": "pinned-peer-unavailable", "unit-start-failed": "unit-start-failed", "shard-plan-refused": "shard-retry-isolated",
    "stalled": "stalled-attempt-stopped", "resubmit-required": "queued-revalidated-on-restart",
    "prune-server-unavailable": "prune-server-unavailable-outcome", "over-four-hours": "run-hard-limit-stopped",
    "killed-externally": "external-kill-recorded-and-retried",
}

HEAVY_THREADS = lib.load_module("heavy_threads", os.path.join(lib.HOSTS_DIRECTORY, "heavy_threads.py"))


# ══ 各格 ══
def cell_check_missing(world):
    absent = os.path.join(world.root, "absent-multi-host.env")
    completed = world.control(["check"], extra={lib.MULTI_HOST_CONFIG_VARIABLE: absent})
    if completed.returncode != 78 or "multi-host.env.example" not in completed.stderr or "cp multi-host.env.example multi-host.env" not in completed.stderr:
        raise SelftestFailure(f"多机配置不在时 control check 应当退 78、出路写 cp multi-host.env.example multi-host.env，实际退 {completed.returncode}", [completed.stdout, completed.stderr])
    completed = world.control(["check"])
    if completed.returncode != 78 or "不通" not in completed.stderr or "control start" not in completed.stderr:
        raise SelftestFailure(f"服务没起时 control check 应当退 78、报不通、出路写 control start，实际退 {completed.returncode}", [completed.stderr])
    world.start_server()
    leftover = [path for path in (os.path.join(world.root, "task-server.env"), os.path.join(lib.REPOSITORY_ROOT, "task-server.env")) if os.path.exists(path)]
    completed = world.control(["check"])
    if completed.returncode != 0:
        raise SelftestFailure(f"多机配置在、服务起了之后 control check 应当退 0（task-server.env 不在也照常，此刻在的：{leftover or '无'}），实际退 {completed.returncode}",
                              [completed.stdout, completed.stderr])
    completed = world.control(["check"], extra={"TASK_SERVER_SELFTEST_MISSING_MODULES": "fastapi"})
    if completed.returncode != 78 or "sudo apt install python3-fastapi python3-uvicorn" not in completed.stderr:
        raise SelftestFailure(f"fastapi 不在时 control check 应当退 78、出路写 sudo apt install python3-fastapi python3-uvicorn，实际退 {completed.returncode}",
                              [completed.stderr])
    return "多机配置不在、服务没起、fastapi 不在各退 78 且出路写全，多机配置在、服务起了退 0（不读 task-server.env）"


def cell_install(world):
    state = os.path.join(world.root, "install-state")
    runtime = os.path.join(world.short_root, LONGEST_RUNTIME_NAME)
    key = os.path.join(world.root, "install-config", "signing.key")
    build = os.path.join(world.root, "install-builds")
    config = os.path.join(world.root, "install-multi-host.env")
    World.write_config(config, state, runtime, key, build, world.shared_root)
    extra = {lib.MULTI_HOST_CONFIG_VARIABLE: config, "TASK_SERVER_CONTROL_SYSTEMCTL": os.path.join(world.bin, "fake-systemctl"),
             "XDG_CONFIG_HOME": os.path.join(world.root, "xdg-config")}
    unit = os.path.join(world.root, "xdg-config", "systemd", "user", lib.UNIT_NAME)
    calls = os.path.join(world.world, "systemctl-calls")
    completed = world.control(["install", "--dry-run"], extra=extra)
    written = [path for path in (key, state, runtime, build, unit, calls) if os.path.exists(path)]
    if (completed.returncode != 0 or "ExecStart=" not in completed.stdout or f"Environment={lib.MULTI_HOST_CONFIG_VARIABLE}={config}" not in completed.stdout
            or written):
        raise SelftestFailure(f"install --dry-run 应当退 0、打出单元全文（钉住这一份多机配置）、一个文件不写、systemctl 不起，实际退 {completed.returncode}、写了 {written}",
                              [completed.stdout[-600:], completed.stderr])
    completed = world.control(["install", "--dry-run"], extra=dict(extra, **{lib.MULTI_HOST_CONFIG_VARIABLE: os.path.join(world.root, "absent-multi-host.env")}))
    if completed.returncode != 78 or "cp multi-host.env.example multi-host.env" not in completed.stderr:
        raise SelftestFailure(f"多机配置不在时 install 应当退 78、不替用户拷模板，实际退 {completed.returncode}", [completed.stderr])
    completed = world.control(["install"], extra=extra)
    modes = {path: oct(os.stat(path).st_mode & 0o777) for path in (key, os.path.dirname(key), state, runtime) if os.path.exists(path)}
    calls_text = ""
    with contextlib.suppress(OSError):
        with open(calls, encoding="utf-8") as handle:
            calls_text = handle.read()
    if (completed.returncode != 1 or "没回话" not in completed.stderr or modes.get(key) != "0o600" or modes.get(os.path.dirname(key)) != "0o700"
            or modes.get(state) != "0o700" or not os.path.isfile(unit) or "daemon-reload" not in calls_text or "--user start" not in calls_text or "enable --now" in calls_text):
        raise SelftestFailure(f"服务没起来时 install 应当写好密钥（600，目录 700）、状态目录（700）与单元、跑 daemon-reload 与 start（不 enable）、回读不通退 1，"
                              f"实际退 {completed.returncode}、权限 {modes}、单元{'在' if os.path.isfile(unit) else '不在'}、systemctl 收到「{calls_text.strip()}」",
                              [completed.stderr])
    with open(key, "rb") as handle:
        digest = hashlib.sha256(handle.read()).hexdigest()
    world.start_server(extra={lib.MULTI_HOST_CONFIG_VARIABLE: config}, runtime=runtime)
    completed = world.control(["install"], extra=extra)
    with open(key, "rb") as handle:
        digest_after = hashlib.sha256(handle.read()).hexdigest()
    if completed.returncode != 0 or digest_after != digest:
        raise SelftestFailure(f"服务起着、密钥已在时再 install 应当退 0、密钥不动，实际退 {completed.returncode}、密钥{'没动' if digest_after == digest else '被重写了'}",
                              [completed.stdout[-400:], completed.stderr])
    return "--dry-run 只打不写；install 写密钥 600、目录 700、单元，回读不通退 1；服务起着再装退 0、密钥不重写"


def cell_control_readback(world):
    extra = {"TASK_SERVER_CONTROL_SYSTEMCTL": os.path.join(world.bin, "fake-systemctl")}
    with open(os.path.join(world.world, "systemctl-state"), "w", encoding="utf-8") as handle:
        handle.write("active\n")
    open(os.path.join(world.world, "systemctl-stop-leaves-active"), "w").close()
    completed = world.control(["stop"], extra=extra)
    if completed.returncode != 1 or "回读不对" not in completed.stderr:
        raise SelftestFailure(f"systemctl stop 退 0 而 is-active 仍打 active 时 control stop 应当退 1、报回读不对，实际退 {completed.returncode}", [completed.stderr])
    os.remove(os.path.join(world.world, "systemctl-stop-leaves-active"))
    completed = world.control(["stop"], extra=extra)
    if completed.returncode != 0:
        raise SelftestFailure(f"停下了（is-active 打 inactive、cgroup 里没有进程）时 control stop 应当退 0，实际退 {completed.returncode}", [completed.stderr])
    return "停了而回读仍在跑退 1，回读停了退 0"


def cell_server_unreachable(world):
    marker = os.path.join(world.root, "ran-directly")
    completed = world.client(["submit", "--", "bash", "-c", ': > "$1"', "touch-marker", marker])
    time.sleep(0.5)
    if completed.returncode != 78 or "不通" not in completed.stderr or "control start" not in completed.stderr or os.path.exists(marker):
        raise SelftestFailure(f"服务不通时 submit 应当退 78、出路写 control start、命令一行不跑，实际退 {completed.returncode}、命令{'跑了' if os.path.exists(marker) else '没跑'}",
                              [completed.stdout, completed.stderr])
    completed = world.client(["wait", "1"])
    if completed.returncode != 78:
        raise SelftestFailure(f"服务一直不通时 wait 应当重连一阵之后退 78，实际退 {completed.returncode}", [completed.stderr])
    return "submit 退 78、命令没跑；wait 重连不上退 78"


def cell_exit_code_passthrough(world):
    world.start_server()
    task_id = world.submit(["bash", "-c", "echo selftest-output; exit 7"])
    completed = world.client(["wait", str(task_id)])
    last = completed.stdout.strip().splitlines()[-1] if completed.stdout.strip() else ""
    found = re.match(rf"{lib.RESULT_MARK} task={task_id} type=command-failed exit=7 attempts=1 log=(\S+)$", last)
    log = found.group(1) if found else ""
    content = ""
    with contextlib.suppress(OSError):
        with open(log, encoding="utf-8") as handle:
            content = handle.read()
    if completed.returncode != 7 or not found or not lib.inside(log, world.state) or "selftest-output" not in content:
        raise SelftestFailure(f"命令退 7：wait 应当退 7、末行 type=command-failed exit=7 attempts=1、日志在状态目录里、有命令的输出，实际退 {completed.returncode}、末行「{last}」",
                              [completed.stderr])
    calls = world.calls_of(task_id)
    if not calls or f"admitted={task_id}.1" not in calls[0]:
        raise SelftestFailure(f"本机的尝试应当给内存包装设 {lib.ADMITTED_VARIABLE}={task_id}.1，实际调用「{calls}」")
    return f"wait 退 7、末行对、日志在状态目录 {os.path.basename(log)}、内存包装收到已准入的开关"


def server_socket_count(world):
    """服务进程此刻开着的 socket 数（/proc/<进程号>/fd 里指到 socket: 的）：监听的、自管道的，加每个还连着的客户端一个；读不到交 -1。"""
    directory = f"/proc/{world.server.pid}/fd"
    count = 0
    try:
        names = os.listdir(directory)
    except OSError:
        return -1
    for name in names:
        with contextlib.suppress(OSError):
            if os.readlink(os.path.join(directory, name)).startswith("socket:"):
                count += 1
    return count


def cell_restart_keeps_running(world):
    world.start_server()
    release = os.path.join(world.root, "release")
    task_id = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release])
    summary = world.running(task_id)
    unit = summary["attempts"][0].get("unit", "")
    samples = []
    for _ in range(3):   # 没有客户端连着时服务开着几个 socket：隔一会儿采三次取最小，躲开刚断开还没收掉的那一个
        time.sleep(0.1)
        samples.append(server_socket_count(world))
    idle_sockets = min(samples)
    # wait 照平常的定时：缩短的定时下断线满 2 秒（task 的 FAST_STARTUP_RETRY_SECONDS）就退 78，停服务、隔 0.5 秒、再起到 /health 回话在机器忙时比它长
    # （任务 4519、4521 与 4586–4595 里的 78 都是「task-server 不通 … server.sock：FileNotFoundError」），这一格判的是重启时 wait 接着重连，不是缩短的那个时限
    waiter = world.client(["wait", str(task_id)], background=True, extra={"TASK_SERVER_SELFTEST_FAST": "0"})
    # 等 wait 真连上服务（服务多开了一个 socket）再停服务：机器忙时客户端起得慢，固定睡 1 秒停服务，它可能到服务重启之后才连上，没碰上断线，
    # 弄坏开关 wait-gives-up-on-disconnect（断线就退）就弄不红这一格
    patience = world.patience(SOCKET_CONNECT_SECONDS)
    while waiter.poll() is None and server_socket_count(world) <= idle_sockets and patience.waiting():
        time.sleep(POLL_SECONDS)
    time.sleep(0.5)   # 连上之后再给它一点时间发出长等的请求
    world.stop_server()
    # 服务停着时等 wait 真碰上断线：照平常它重连时打一句「task-server 连不上」（弄坏开关下它就此退 78）才再起服务。停服务时那一问可能回 HTTP 500、
    # wait 隔 RECONNECT_SECONDS 再问，服务要是在那之前已重新起来，它一次断线都没碰上，开关弄不红这一格（任务 7290）
    noticed = b""
    descriptor = waiter.stderr.fileno()
    os.set_blocking(descriptor, False)
    patience = world.patience(DISCONNECT_NOTICE_SECONDS)
    while waiter.poll() is None and UNREACHABLE_NOTICE.encode("utf-8") not in noticed and patience.waiting():
        with contextlib.suppress(BlockingIOError):
            noticed += os.read(descriptor, 65536)
        time.sleep(POLL_SECONDS)
    os.set_blocking(descriptor, True)
    if not world.unit_active(unit):
        raise SelftestFailure(f"服务停下之后在跑的那一次尝试的单元应当照跑，实际单元「{unit or '（没有单元）'}」不在跑")
    if waiter.poll() is not None:
        raise SelftestFailure(f"服务停下时 wait 应当接着重连、不退，实际退了 {waiter.returncode}",
                              [waiter.stdout.read(), noticed.decode("utf-8", "replace") + waiter.stderr.read()])
    if UNREACHABLE_NOTICE.encode("utf-8") not in noticed:
        raise SelftestFailure(f"服务停着时 wait 应当重连、打「{UNREACHABLE_NOTICE}」，安静 {DISCONNECT_NOTICE_SECONDS:.0f} 秒没打（它没碰上断线，这一格判不出断线就退）",
                              [noticed.decode("utf-8", "replace")])
    world.start_server()
    open(release, "w").close()
    try:
        stdout, stderr = world.communicate(waiter, STATE_TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired:
        raise SelftestFailure("服务重启、任务跑完之后 wait 60 秒没退") from None
    if waiter.returncode != 0 or f"{lib.RESULT_MARK} task={task_id} type=succeeded" not in stdout:
        raise SelftestFailure(f"服务重启之后 wait 应当拿到真结局 succeeded、退 0，实际退 {waiter.returncode}", [stdout, stderr])
    return "服务停了单元照跑、wait 重连，重启之后拿到 succeeded"


def cell_start_once(world):
    for point in ("before-unit", "after-unit"):
        world.stop_server()
        arm = os.path.join(world.root, f"arm-{point}")
        counter = os.path.join(world.root, f"count-{point}")
        world.resources(local=(30, 30), hold=True)
        world.start_server(extra={"TASK_SERVER_SELFTEST_CRASH_POINT": f"{point}:{arm}"})
        task_id = world.submit(["bash", "-c", 'printf x >> "$1"', "count", counter])
        open(arm, "w").close()
        world.resources(local=(30, 30))
        code = world.wait_server_exit()
        if code != 70:
            raise SelftestFailure(f"自证造的崩溃（{point}）应当让服务退 70，实际退 {code}", world.server_log_lines())
        patience = world.patience(STATE_TIMEOUT_SECONDS)
        while point == "after-unit" and patience.waiting() and (not os.path.exists(counter) or any(world.unit_active(unit) for unit in world.database_units())):
            time.sleep(POLL_SECONDS)
        world.start_server()
        summary = world.finished(task_id)
        with open(counter, encoding="utf-8") as handle:
            count = handle.read()
        if summary["task"].get("outcome") != "succeeded" or count != "x" or len(summary["attempts"]) != 1:
            raise SelftestFailure(f"服务在{'写了单元名、起单元之前' if point == 'before-unit' else '起了单元之后、标记起了之前'}没了：重启之后应当只起这一次（命令跑 1 次、尝试 1 次、succeeded），"
                                  f"实际命令跑了 {len(count)} 次、{describe(summary)}", world.server_log_lines())
    return "写了单元名、起单元之前与起了之后各崩一次：重启之后命令各只跑 1 次"


def cell_error_types(world):
    """表里的错误类型逐个喂一个假命令或假包装的退出码（EXERCISED_BY 点名这一格的那几种），记下的类型与表对得上；表外的 199 记 command-failed，
    退 245（表里没有这个码）同样记 command-failed。"""
    world.resources(local=(100, 100))   # 头一段没有第二台：本机的那几种都在本机起（派得到第二台的照余量挑台，不再因工作目录只在本机）
    world.seed_peak("selftest-peer-only", 120 * GIBIBYTE)
    world.start_server()
    plain = ["bash", "-c", "exit 0"]
    cases = [("succeeded", plain, {}, None, None), ("command-failed", ["bash", "-c", "exit 3"], {}, None, 3),
             ("command-failed", ["bash", "-c", "exit 199"], {}, None, 199), ("command-failed", ["bash", "-c", "exit 245"], {}, None, 245),
             ("memory-cap-hit", plain + ["fake-wrapper-exits=250,run"], {}, None, None), ("squeezed-out", plain + ["fake-wrapper-exits=254,run"], {}, None, None),
             ("timed-out", plain + ["fake-wrapper-exits=253"], {}, None, None), ("wrapper-refused", plain + ["fake-wrapper-exits=251"], {}, None, None),
             ("run-condition-unmet", ["bash", "-c", "exit 78"], {}, None, None), ("not-run", ["bash", "-c", "exit 77"], {}, None, None),
             ("launcher-lost", ["bash", "-c", "kill -9 $PPID"], {}, None, None)]
    submitted = [(expected, world.submit(command, extra=extra, cwd=cwd), exit_code) for expected, command, extra, cwd, exit_code in cases]
    summaries = {task_id: world.finished(task_id) for _, task_id, _ in submitted}
    # 后一段有第二台：记录 120G 比本机可用上限 98G 大、第二台装得下的那一条不派本机（hosts_short_of_record）、派第二台，假第二台退 255 记 peer-run-failed
    world.resources(local=(100, 100), peer=(200, 200), peer_cap_gibibytes=190)
    peer_case = ("peer-run-failed", plain + ["fake-peer-exit=255"], {"RUN_WITH_MEMORY_CAP_KEY": "selftest-peer-only"}, world.repo, None)
    cases.append(peer_case)
    peer_id = world.submit(peer_case[1], extra=peer_case[2], cwd=peer_case[3])
    summaries[peer_id] = world.finished(peer_id)
    submitted.append((peer_case[0], peer_id, peer_case[4]))
    wrong = []
    for expected, task_id, exit_code in submitted:
        summary = summaries[task_id]
        attempts = summary["attempts"]
        seen = attempts[0].get("outcome") if attempts else summary["task"].get("outcome")
        if seen != expected or (exit_code is not None and (attempts[0].get("exit_code") != exit_code or summary["task"].get("exit_code") != exit_code)):
            wrong.append(f"任务 {task_id} 应当是 {expected}{'、退 ' + str(exit_code) if exit_code is not None else ''}，实际第一次尝试 {seen}、{describe(summary)}")
        if expected == "peer-run-failed" and attempts and attempts[0].get("host") != "peer":
            wrong.append(f"任务 {task_id} 应当派到第二台，实际 {attempts[0].get('host')}")
    if wrong:
        raise SelftestFailure("错误类型对不上：" + "；".join(wrong), world.server_log_lines())
    return f"{len(cases)} 个假命令或假包装记下的类型都与表对得上（表外的 199、245 记 command-failed）"


def cell_unit_start_failed(world):
    world.start_server(extra={"TASK_SERVER_SYSTEMD_RUN": os.path.join(world.bin, "fake-systemd-run-refusing")})
    marker = os.path.join(world.root, "ran")
    task_id = world.submit(["bash", "-c", ': > "$1"', "touch-marker", marker])
    summary = world.finished(task_id)
    if summary["task"].get("outcome") != "unit-start-failed" or summary["task"].get("exit_code") != 235 or os.path.exists(marker):
        raise SelftestFailure(f"systemd-run 起不来时应当记 unit-start-failed、退 235、命令没跑，实际 {describe(summary)}、命令{'跑了' if os.path.exists(marker) else '没跑'}")
    return "systemd-run 起不来记 unit-start-failed、命令没跑"


def cell_tampered_row_not_started(world):
    world.start_server()
    marker = os.path.join(world.root, "bypass-ran")
    record = {"command": ["bash", "-c", ': > "$1"', "touch-marker", marker], "cwd": world.work, "environment": {}, "session": SESSION_A, "submitter_pid": 0,
              "submitted_at": time.time(), "command_key": "selftest-bypass", "state": "queued", "outcome": "", "exit_code": None, "collected": False,
              "next_cap": None, "next_host": "", "outcome_counts": {}, "note": "", "cancel_requested": False, "attempt_count": 0}
    task_id = world.insert_bypass_row(record)
    summary = world.finished(task_id, timeout=20)
    alerted = [alert for alert in world.alerts() if alert.get("kind") == "record-tampered" and alert.get("task") == task_id]
    if summary["task"].get("outcome") != "record-tampered" or os.path.exists(marker) or world.calls_of(task_id) or not alerted:
        raise SelftestFailure(f"绕过服务直接写进库的任务应当不起、记 record-tampered、写告警，实际 {describe(summary)}、命令{'跑了' if os.path.exists(marker) else '没跑'}、"
                              f"告警 {len(alerted)} 条")
    return "绕过服务写进库的行不起、记 record-tampered、写了告警"


def cell_forged_session(world):
    world.start_server()
    forged = world.submit(["bash", "-c", "exit 0"], session=SESSION_A, forged=SESSION_FORGED)
    human = world.submit(["bash", "-c", "exit 0"], session=None)
    sessions = [world.summary(task_id)["task"].get("session") for task_id in (forged, human)]
    if sessions != [SESSION_A, ""]:
        raise SelftestFailure(f"客户端自己环境里的会话号是伪造的：记的应当是进程树上读到的 {SESSION_A}，人手的记空串，实际 {sessions}")
    return "伪造的会话号不认，记进程树上读到的；没有会话号的记空串"


def cell_environment_whitelist(world):
    world.start_server()
    completed = world.client(["submit", "--", "bash", "-c", "exit 0"], extra={"SELFTEST_UNLISTED_VARIABLE": "1"})
    if completed.returncode != 3 or "SELFTEST_UNLISTED_VARIABLE" not in completed.stderr:
        raise SelftestFailure(f"白名单外的变量应当当场拒（退 3）并点名，实际退 {completed.returncode}", [completed.stdout, completed.stderr])
    completed = world.client(["submit", "--", "bash", "-c", "exit 0"], extra={"RUN_WITH_MEMORY_CAP_WAIT_SECONDS": "5"})
    if completed.returncode != 3 or "RUN_WITH_MEMORY_CAP_WAIT_SECONDS" not in completed.stderr or "排队归 task-server" not in completed.stderr:
        raise SelftestFailure(f"排队类变量应当当场拒（退 3）并说明排队归服务，实际退 {completed.returncode}", [completed.stdout, completed.stderr])
    probe = os.path.join(world.root, "scope-seen")
    task_id = world.submit(["bash", "-c", 'printf "%s" "$SINGLEFS_RUN_SCOPE" > "$1"', "probe", probe], extra={"SINGLEFS_RUN_SCOPE": "selftest-scope"})
    world.finished(task_id)
    seen = ""
    with contextlib.suppress(OSError):
        with open(probe, encoding="utf-8") as handle:
            seen = handle.read()
    if seen != "selftest-scope" or world.calls()[:-1]:
        raise SelftestFailure(f"白名单里的 SINGLEFS_RUN_SCOPE 应当带进任务、被拒的两次一条都没起，实际任务看到「{seen}」、包装调用 {world.calls()}")
    return "白名单外与排队类变量当场拒并点名，白名单里的带进去"


def cell_retired_variables_refused(world):
    """退役的名字（lib.RETIRED_VARIABLES，文件头「环境变量」）：环境里设了的、命令外层写了「名=值」的（env 后面、bash -c 引号里命令位置前）都当场拒（task submit 退 3、不给任务号），
    说明点名它、写的是它那一行的出路（不是「白名单外」），一条都没起。"""
    world.start_server()
    wrong = {}
    for name, why in lib.RETIRED_VARIABLES.items():
        completed = world.client(["submit", "--", "bash", "-c", "exit 0"], extra={name: "1"})
        if completed.returncode != 3 or name not in completed.stderr or why not in completed.stderr or "白名单外" in completed.stderr:
            wrong[f"环境里的 {name}"] = (completed.returncode, completed.stderr.strip()[-300:])
    for words, name in ((["env", "SINGLEFS_HEAVY_TESTS=commit", "true"], "SINGLEFS_HEAVY_TESTS"),
                        (["bash", "-c", "GATE_MUTATION_START_OVER=1 true"], "GATE_MUTATION_START_OVER"),
                        (["GATE_MUTATION_START_OVER=1", "bash", "-c", "exit 0"], "GATE_MUTATION_START_OVER"),
                        (["env", "LANG=C", "SINGLEFS_HEAVY_TESTS=commit", "bash", "-c", "exit 0"], "SINGLEFS_HEAVY_TESTS")):
        completed = world.client(["submit", "--", *words])
        if completed.returncode != 3 or f"{name}=" not in completed.stderr or "去掉这个赋值" not in completed.stderr:
            wrong[" ".join(words)] = (completed.returncode, completed.stderr.strip()[-300:])
    started = [line for line in world.calls() if line.startswith("wrapper ")]
    if wrong or started:
        raise SelftestFailure(f"退役的名字设在环境里、写在命令外层都应当当场拒（退 3、点名它、写它那一行的出路）、一条都不起，实际 {wrong}、包装调用 {started}")
    return f"退役的 {len(lib.RETIRED_VARIABLES)} 个名字设在环境里、写在命令外层与 bash -c 里的都当场拒并写出路，一条都没起"


def cell_nested_dispatch_refused(world):
    world.start_server()
    completed = world.client(["submit", "--", "bash", "scripts/hosts/run-with-memory-cap.sh", "4G", "true"])
    if completed.returncode != 3 or "内存上限由服务定" not in completed.stderr:
        raise SelftestFailure(f"命令自己套内存包装应当被拒（退 3），实际退 {completed.returncode}", [completed.stdout, completed.stderr])
    completed = world.client(["submit", "--", "python3", "scripts/hosts/host-dispatch.py", "--", "true"])
    if completed.returncode != 3:
        raise SelftestFailure(f"命令自己套派发器应当被拒（退 3），实际退 {completed.returncode}", [completed.stdout, completed.stderr])
    refused = {}
    for words, mark in ((["nice", "-n", "5", "true"], "优先级"), (["timeout", "10", "true"], "限时"),
                        (["bash", "scripts/hosts/capped.sh", "4", "true"], "线程"), (["bash", "scripts/crash/gpu-quiesce-run.sh", "--", "true"], "清场"),
                        (["bash", "scripts/gate/merge-and-verify.sh", "--memory", "16G"], "--memory"), (["bash", "x.sh", "--threads=8"], "--threads"),
                        (["bash", "x.sh", "--peer"], "--peer"), (["env", "SINGLEFS_THREAD_CAP=7", "true"], "SINGLEFS_THREAD_CAP"),
                        (["bash", "-c", "cd x && bash scripts/hosts/run-with-memory-cap.sh 8G cargo test"], "内存上限由服务定"),
                        (["bash", "-c", "true; timeout 10 cargo build"], "限时"),
                        (["bash", "scripts/hosts/multi-host-run.sh", "--verdict-library-route", "--", "true"], "不套派发器"),
                        (["python3", "scripts/hosts/host-dispatch.py", "--plan", "--", "true"], "不套派发器")):
        completed = world.client(["submit", "--", *words])
        if completed.returncode != 3 or mark not in completed.stderr:
            refused[" ".join(words)] = (completed.returncode, completed.stderr.strip()[-200:])
    if refused:
        raise SelftestFailure(f"包装前缀与上限、线程、挑台参数应当当场拒（退 3、点名那一层或那个参数），实际 {refused}")
    task_id = world.submit(["bash", "scripts/hosts/run-with-memory-cap.sh", "--selftest", "fake-wrapper-exits=0"])
    summary = world.finished(task_id)
    if summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"内存包装自己的 --selftest 不算套包装，应当照收、跑完，实际 {describe(summary)}")
    return ("套内存包装、派发器（带不带 --plan、--verdict-library-route 都一样）、nice、timeout、capped.sh、清场包装与写 --memory、--threads、--peer、"
            "SINGLEFS_THREAD_CAP 的都拒，包装自己的 --selftest 照收")


def cell_operation_log(world):
    world.start_server()
    release = os.path.join(world.root, "release")
    task_id = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release], session=SESSION_A)
    world.running(task_id)
    world.client(["cancel", str(task_id)], session=SESSION_B)
    completed = world.client(["cancel", str(task_id)], session=SESSION_A)
    open(release, "w").close()
    world.finished(task_id)
    seen = [(record.get("action"), record.get("session")) for record in world.operations() if record.get("task") == task_id]
    if seen != [("submit", SESSION_A), ("confirm", SESSION_A), ("cancel-refused", SESSION_B), ("cancel", SESSION_A)]:
        raise SelftestFailure(f"增加、确认与取消都应当记操作日志（submit、confirm、cancel-refused、cancel 与各自的会话），实际 {seen}", [completed.stderr])
    return "提交、确认、被拒的取消、取消都记进了仓外的操作日志"


def cell_control_watch_removed(world):
    """control watch 已删（task-server/control 文件头）：退 2，标准错误打「这个子命令已删，task-server 自己盯、出事直接发消息给发起任务的会话」，出路点名 control check。"""
    completed = world.control(["watch"])
    wanted = "这个子命令已删，task-server 自己盯、出事直接发消息给发起任务的会话"
    if completed.returncode != lib.USAGE_EXIT or wanted not in completed.stderr or "control check" not in completed.stderr:
        raise SelftestFailure(f"control watch 应当退 {lib.USAGE_EXIT}、打「{wanted}」与出路，实际退 {completed.returncode}、「{completed.stderr.strip()[-300:]}」")
    return f"control watch 退 {lib.USAGE_EXIT}、打出出路句"


def cell_state_placement(world):
    environment = {name: value for name, value in world.client_environment.items() if name != "TASK_SERVER_SELFTEST_ALLOW_TMPFS"}
    in_repository = os.path.join(lib.REPOSITORY_ROOT, ".task-server-selftest-state")
    cases = (("tmpfs", os.path.join("/dev/shm", f"task-server-selftest-{os.getpid()}-state"), world.builds),
             ("落在仓里", in_repository, world.builds),
             ("运行时目录", os.path.join(world.runtime, "state"), world.builds),
             ("编译目录", world.state, os.path.join(lib.REPOSITORY_ROOT, ".task-server-selftest-builds")))
    for number, (what, state, build) in enumerate(cases, 1):
        config = os.path.join(world.root, f"placement-{number}.env")
        World.write_config(config, state, world.runtime, world.key, build, world.shared_root)
        saved = world.client_environment
        world.client_environment = dict(environment, **{lib.MULTI_HOST_CONFIG_VARIABLE: config})
        if what == "编译目录":
            world.client_environment["TASK_SERVER_SELFTEST_ALLOW_TMPFS"] = "1"   # 这一例只判编译目录：状态目录是这一格的临时目录，可能在 tmpfs 上
        try:
            completed = world.control(["check"])
        finally:
            world.client_environment = saved
        if completed.returncode != 78 or what not in completed.stderr or os.path.exists(state if what != "编译目录" else build):
            raise SelftestFailure(f"状态目录放在 {state}、编译目录放在 {build}（{what}）应当退 78 并说明，实际退 {completed.returncode}", [completed.stderr])
    return "状态目录在 tmpfs 上、仓里、运行时目录下，编译目录在仓里，都拒"


TICK_PROMPT_TICK_SECONDS = 30      # thirty-second-tick-prompt：主循环一轮的间隔（TASK_SERVER_TICK_SECONDS）
WAIT_LONG_POLL_SECONDS = 20.0      # thirty-second-tick-prompt：task 客户端 wait 平常一问等多久（task 的 WAIT_SERVER_SECONDS；自证里缩短的 FAST 定时下不是它）


def cell_thirty_second_tick_prompt(world):
    """主循环 30 秒一轮、wait 平常 20 秒一问时，任务一结束就叫醒：runner 写完结果叫醒主循环（服务记下结局），服务记下结局叫醒在等的 wait。
    命令等放行文件才退，wait 连上服务之后才放行，放行前叫主循环转一轮（POST /wake），所以放行落在刚转完一轮之后、wait 刚进长等之后：
    不叫醒就要分别等主循环下一轮（30 秒）与 wait 这一问到点（20 秒）。判据取服务与客户端各自记下的前后两段，每一段与它自己的间隔比，不拿提交起算的挂钟比：
    命令结束到服务记下结局（尝试的 ended_at 到任务的 finished_at）要短于主循环间隔的一半；服务记下结局到 wait 拿到结局要短于 wait 一问的一半。
    弄坏开关 runner-wake-skipped 下第一段等主循环下一轮、wait-notify-skipped 下第二段等 wait 这一问到点，这一格红。"""
    world.start_server(extra={"TASK_SERVER_TICK_SECONDS": str(TICK_PROMPT_TICK_SECONDS)})
    release = os.path.join(world.root, "release-tick-prompt")
    task_id = world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "tick-prompt", release])
    world.running(task_id)
    samples = []
    for _ in range(3):   # 没有客户端连着时服务开着几个 socket：隔一会儿采三次取最小，躲开刚断开还没收掉的那一个
        time.sleep(0.1)
        samples.append(server_socket_count(world))
    idle_sockets = min(samples)
    waiter = world.client(["wait", str(task_id)], background=True, extra={"TASK_SERVER_SELFTEST_FAST": "0"})   # 照平常 20 秒一问
    try:
        patience = world.patience(SOCKET_CONNECT_SECONDS)
        while waiter.poll() is None and server_socket_count(world) <= idle_sockets and patience.waiting():
            time.sleep(POLL_SECONDS)
        time.sleep(0.5)   # 连上之后再给它一点时间发出长等的请求
        world.api("POST", "/wake")   # 主循环转一轮：之后下一轮在 TICK_PROMPT_TICK_SECONDS 秒以后
    finally:
        open(release, "w").close()
    try:
        stdout, stderr = world.communicate(waiter, STATE_TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired:
        stdout, stderr = world.stop_client(waiter)
        raise SelftestFailure(f"放行之后 wait 安静 {STATE_TIMEOUT_SECONDS:.0f} 秒都没拿到结局（任务在放行之后应当马上结束）", [stdout or "", stderr or ""] + world.server_log_lines()) from None
    returned_at = time.time()
    summary = world.summary(task_id)
    ended_at = float((summary["attempts"][-1] if summary["attempts"] else {}).get("ended_at") or 0.0)
    finished_at = float(summary["task"].get("finished_at") or 0.0)
    recorded_after, reported_after = finished_at - ended_at, returned_at - finished_at
    if (waiter.returncode != 0 or not ended_at or not finished_at or recorded_after >= TICK_PROMPT_TICK_SECONDS / 2 or reported_after >= WAIT_LONG_POLL_SECONDS / 2):
        raise SelftestFailure(f"主循环 {TICK_PROMPT_TICK_SECONDS} 秒一轮、wait 一问 {WAIT_LONG_POLL_SECONDS:.0f} 秒时，命令结束到服务记下结局应当不到 {TICK_PROMPT_TICK_SECONDS / 2:.0f} 秒"
                              f"（runner 写完结果叫醒主循环），服务记下结局到 wait 拿到结局应当不到 {WAIT_LONG_POLL_SECONDS / 2:.0f} 秒（记下结局叫醒 wait）；"
                              f"实际 wait 退 {waiter.returncode}、尝试结束 {ended_at}、任务结局记在 {finished_at}、前一段 {recorded_after:.1f} 秒、后一段 {reported_after:.1f} 秒",
                              [stdout or "", stderr or ""] + world.server_log_lines())
    return (f"主循环 {TICK_PROMPT_TICK_SECONDS} 秒一轮、wait 一问 {WAIT_LONG_POLL_SECONDS:.0f} 秒：命令结束 {recorded_after:.1f} 秒后服务记下结局、再 {reported_after:.1f} 秒后 wait 拿到结局"
            f"（结束即叫醒、wait 不轮询）")


def cell_socket_path_too_long(world):
    runtime = os.path.join(world.root, "runtime")
    while len(os.fsencode(os.path.join(runtime, lib.SOCKET_NAME))) <= lib.SOCKET_PATH_MAXIMUM_BYTES:
        runtime = os.path.join(runtime, TOO_DEEP_SOCKET_DIRECTORY)
    socket_path = os.path.join(runtime, lib.SOCKET_NAME)
    length = len(os.fsencode(socket_path))
    config = os.path.join(world.root, "too-long-socket-multi-host.env")
    World.write_config(config, world.state, runtime, world.key, world.builds, world.shared_root)
    named = (socket_path, f"{length} 字节", f"{lib.SOCKET_PATH_MAXIMUM_BYTES} 字节")
    server = subprocess.Popen([sys.executable, "-B", lib.SERVER_SCRIPT, "serve"], env=dict(world.server_environment, **{lib.MULTI_HOST_CONFIG_VARIABLE: config}),
                              stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, start_new_session=True)
    try:
        _, server_stderr = world.communicate(server, STATE_TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired:
        server.kill()   # 只停自己起的那一个进程号
        _, server_stderr = server.communicate(timeout=10)
        raise SelftestFailure(f"socket 路径 {length} 字节（上限 {lib.SOCKET_PATH_MAXIMUM_BYTES}）时服务应当当场退出，实际 {STATE_TIMEOUT_SECONDS:.0f} 秒还没退",
                              [server_stderr]) from None
    client = world.client(["status"], extra={lib.MULTI_HOST_CONFIG_VARIABLE: config})
    problems = []
    if server.returncode == 0 or not all(text in server_stderr for text in named):
        problems.append(f"服务应当退非 0、stderr 点名 socket 路径、它的 {length} 字节与上限 {lib.SOCKET_PATH_MAXIMUM_BYTES} 字节，实际退 {server.returncode}、"
                        f"stderr 里缺 {[text for text in named if text not in server_stderr]}")
    if client.returncode != lib.RUN_CONDITION_EXIT or not all(text in client.stderr for text in named):
        problems.append(f"客户端 status 应当退 78、stderr 点名同样三样，实际退 {client.returncode}、stderr 里缺 {[text for text in named if text not in client.stderr]}")
    if os.path.lexists(socket_path):
        problems.append(f"socket {socket_path} 不该建出来，实际在")
    if problems:
        raise SelftestFailure("socket 路径太长：" + "；".join(problems), [server_stderr, client.stderr])
    return f"socket 路径 {length} 字节（上限 {lib.SOCKET_PATH_MAXIMUM_BYTES}）：服务退 {server.returncode}、客户端退 78，都点名路径、字节数与上限"


def cell_threads_fixed(world):
    world.start_server()
    expected = str(HEAVY_THREADS.default_heavy_threads({}))
    seen = {}
    probe = os.path.join(world.root, "threads-registered")
    world.finished(world.submit(["bash", "-c", 'printf "%s" "${SINGLEFS_THREAD_CAP:-}" > "$1"', "probe", probe]))
    with contextlib.suppress(OSError):
        with open(probe, encoding="utf-8") as handle:
            seen["registered"] = handle.read()
    given = world.client(["submit", "--", "bash", "-c", "exit 0"], extra={"SINGLEFS_THREAD_CAP": "7"})
    if seen != {"registered": expected} or given.returncode == 0 or "SINGLEFS_THREAD_CAP" not in given.stderr:
        raise SelftestFailure(f"每一件都经 capped.sh 设登记的 {expected} 个线程，提交时带 SINGLEFS_THREAD_CAP=7 的当场拒并点名；"
                              f"实际命令看到 {seen}、带 7 的那次退 {given.returncode}", [given.stdout, given.stderr])
    return f"命令看到的线程数是登记的 {expected}；提交时带 SINGLEFS_THREAD_CAP 的当场拒"


def log_printed(completed):
    """task log 打的日志行（去掉头一行「== 任务 … 的日志 …」）。"""
    lines = completed.stdout.splitlines()
    return lines[1:] if lines[:1] and lines[0].startswith("== 任务 ") else lines


def attempt_lines(number, first=1, last=LOG_LINES_PER_ATTEMPT):
    """LOG_SCRIPT 在第 number 次尝试里打的第 first 到 last 行。"""
    return [f"attempt-{number} line-{line}" for line in range(first, last + 1)]


def cell_task_log(world):
    FakeInbox(world, "a")   # 会话登进会话登记：结局发得出去，最后一次尝试的日志末尾不多出发不出去的那一行（lib.INBOX_LOG_MARK）
    world.resources(local=(30, 30), hold=True)
    world.start_server()
    problems = []
    queued_id = world.submit(["bash", "-c", "exit 0"])
    completed = world.client(["log", str(queued_id)])
    if completed.returncode != 4 or "没有任何一次尝试" not in completed.stderr:
        problems.append(f"还没起过的任务 {queued_id}：log 应当退 4、说没有任何一次尝试，实际退 {completed.returncode}、stderr「{completed.stderr.strip()[-200:]}」")
    world.resources(local=(30, 30))
    world.finished(queued_id)
    task_id = world.submit(["bash", "-c", LOG_SCRIPT, "log-lines", str(LOG_LINES_PER_ATTEMPT)])
    summary = world.finished(task_id)
    outcomes = [attempt.get("outcome") for attempt in summary["attempts"]]
    if outcomes != ["squeezed-out", "succeeded"]:
        raise SelftestFailure("task log：" + "；".join(problems + [f"要读日志的那一件应当有两次尝试（第 1 次命令退 254 记 squeezed-out、重排的第 2 次 succeeded），"
                                                                 f"实际 {describe(summary)}"]))
    for arguments, what, header, expected in (
            (["log", str(task_id)], "不带参数", "第 2 次尝试", attempt_lines(2, LOG_LINES_PER_ATTEMPT - LOG_TAIL_PINNED_LINES + 1)),
            (["log", str(task_id), "--all"], "--all", "第 2 次尝试", attempt_lines(2)),
            (["log", str(task_id), "--attempt", "1", "--all"], "--attempt 1 --all", "第 1 次尝试", attempt_lines(1))):
        completed = world.client(arguments)
        printed = log_printed(completed)
        head = completed.stdout.splitlines()[0] if completed.stdout else ""
        if completed.returncode != 0 or header not in head or printed != expected:
            problems.append(f"log {what} 应当退 0、打{header}的 {len(expected)} 行（{expected[0]} 到 {expected[-1]}），实际退 {completed.returncode}、"
                            f"头一行「{head[:160]}」、打了 {len(printed)} 行（{printed[:1]} 到 {printed[-1:]}）")
    for arguments, what, said in ((["log", str(task_id), "--attempt", "3"], "要第 3 次尝试（只有两次）", "没有第 3 次尝试"),
                                  (["log", str(ABSENT_TASK_ID)], f"任务 {ABSENT_TASK_ID} 不在", f"没有任务 {ABSENT_TASK_ID}")):
        completed = world.client(arguments)
        if completed.returncode != 4 or said not in completed.stderr:
            problems.append(f"log {what}：应当退 4、说「{said}」，实际退 {completed.returncode}、stderr「{completed.stderr.strip()[-200:]}」")
    if problems:
        raise SelftestFailure("task log：" + "；".join(problems))
    return (f"log 默认打最后一次尝试的最后 {LOG_TAIL_PINNED_LINES} 行、--all 打整份、--attempt 1 取第 1 次；"
            "要的那一次不在、任务不在、还没起过都退 4")


def cell_control_start(world):
    extra = {"TASK_SERVER_CONTROL_SYSTEMCTL": os.path.join(world.bin, "fake-systemctl")}
    problems = []
    completed = world.control(["start"], extra=extra)
    if completed.returncode != 1 or "起了但回读不对" not in completed.stderr or systemctl_start_count(world) != 1:
        problems.append(f"systemctl start 退 0 而服务一直不回话：control start 应当在限时里回读不到、退 1、说回读不对，实际退 {completed.returncode}、"
                        f"systemctl start 收到 {systemctl_start_count(world)} 回、stderr「{completed.stderr.strip()[-200:]}」")
    waiting = world.control(["start"], extra=dict(extra, TASK_SERVER_SELFTEST_FAST="0"), background=True)   # 不缩定时：回读限时 30 秒，等得到服务起来
    patience = world.patience(STATE_TIMEOUT_SECONDS)
    while systemctl_start_count(world) < 2 and waiting.poll() is None and patience.waiting():
        time.sleep(POLL_SECONDS)
    exited_early = waiting.poll() is not None
    world.start_server()
    try:
        stdout, stderr = world.communicate(waiting, STATE_TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired:
        raise SelftestFailure(f"服务起来之后 control start {STATE_TIMEOUT_SECONDS:.0f} 秒没退", ["；".join(problems)]) from None
    if waiting.returncode != 0 or exited_early or "/health 回 ok" not in stdout:
        problems.append(f"systemctl start 之后服务才起来：control start 应当等到 is-active 打 active、/health 回 ok 再退 0，实际退 {waiting.returncode}、"
                        f"{'服务起来之前就退了' if exited_early else '等到了服务起来'}、stdout「{stdout.strip()[-200:]}」、stderr「{stderr.strip()[-200:]}」")
    open(os.path.join(world.world, "systemctl-start-fails"), "w").close()
    completed = world.control(["start"], extra=extra)
    if completed.returncode != 1 or f"systemctl --user start {lib.UNIT_NAME} 退 1" not in completed.stderr:
        problems.append(f"systemctl start 退 1（服务此刻照跑）：control start 应当退 1、说 systemctl start 退了 1，不照回读的结论退 0，实际退 {completed.returncode}、"
                        f"stderr「{completed.stderr.strip()[-200:]}」")
    if problems:
        raise SelftestFailure("control start：" + "；".join(problems))
    return "control start 等到服务起来、回读 is-active 与 /health 都对退 0；服务一直不回话退 1；systemctl start 退非 0 退 1"


def cell_environment_prefix_stripped_from_key(world):
    """命令前面带 env 与不改跑法的变量（多机配置路径、LANG）的，与不带的落同一个命令键，钉值对两条都生效；带了名单外变量的不并（lib.key_words）。"""
    world.set_slice_cap("20G")
    world.resources(local=(100, 100))
    world.start_server()
    plain = ["bash", "-c", "exit 0", "prefix-probe"]
    prefixed = ["env", f"SINGLEFS_MULTI_HOST_CONFIG={os.path.join(world.root, 'multi-host.env')}", "LANG=C", *plain]
    unlisted = ["env", "SELFTEST_UNLISTED_VARIABLE=1", "LANG=C", *plain]
    pinned = world.control(["pin-cap", "--", *plain])
    shown = shown_local_pin(pinned)
    wanted = f"{shown}M" if shown is not None else "pin-cap 没报本机的钉值"   # 钉值的口径由 pinned-cap-by-host 钉数，这一格照服务报的此刻钉值核
    summaries = {name: world.finished(world.submit(command)) for name, command in (("不带前缀的", plain), ("带前缀的", prefixed), ("带名单外变量的", unlisted))}
    keys = {name: summary["task"].get("command_key") for name, summary in summaries.items()}
    caps = {name: first_attempt(summary).get("memory_cap_text") for name, summary in summaries.items()}
    if (pinned.returncode != 0 or shown is None or keys["带前缀的"] != keys["不带前缀的"] or keys["带名单外变量的"] == keys["不带前缀的"]
            or caps["不带前缀的"] != wanted or caps["带前缀的"] != wanted or caps["带名单外变量的"] == wanted):
        raise SelftestFailure(f"带 env 与名单里变量前缀的应当与不带的同一个命令键、都吃到钉值（pin-cap 报的 {wanted}），带名单外变量的另一个键、不吃钉值；"
                              f"实际 pin-cap 退 {pinned.returncode}、键 {keys}、上限 {caps}", world.server_log_lines())
    return f"带 env 前缀与不带的同一个命令键、都吃到钉值 {wanted}；带名单外变量的另算（上限 {caps['带名单外变量的']}）"


def cell_selftest_variables_refused(world):
    outside = {name: value for name, value in world.client_environment.items() if name != lib.SELFTEST_MARK_VARIABLE}
    outside[lib.BREAK_VARIABLE] = "exit-code-dropped"
    for script in (lib.CLIENT_SCRIPT, lib.CONTROL_SCRIPT, lib.SERVER_SCRIPT):
        # 参数从表里取（折不出）：闭包把三份脚本整份连进来，各格经 World.client 起它们时走的是 bash 的 "$@"，闭包不沿那一条认脚本
        arguments = {lib.CLIENT_SCRIPT: ["status"], lib.CONTROL_SCRIPT: ["check"], lib.SERVER_SCRIPT: ["serve"]}[script]
        completed = subprocess.run([sys.executable, "-B", script, *arguments], env=outside, cwd=world.work, stdin=subprocess.DEVNULL, capture_output=True, text=True,
                                   timeout=60)
        if completed.returncode != 2 or "只在自证" not in completed.stderr:
            raise SelftestFailure(f"平常跑设了 {lib.BREAK_VARIABLE}，{os.path.basename(script)} 应当退 2、点名它只在自证里认，实际退 {completed.returncode}", [completed.stderr])
    return "客户端、control、服务平常跑设了弄坏开关都退 2"


def cell_association_column_verified(world):
    """TS-05：只改库里一行尝试的 task_id 列（正文与签名不动），把任务 A 的尝试挂到任务 B 名下：B 的记录读回时那一行校验不过（对账判出之后照正文挂回 A、记 ended、单元由对账停），
    取消 B 回不了 stopped；只改一个子任务的 task_id 列把它挂到另一件分片名下：取消那一件时对账还没判出的整件回 409、它自己的片不停，
    判出之后那一行照正文挂回原来那一件（核得过、记 record-tampered、挂过来那一片的单元由对账停），取消只停那一件自己的片。
    两条在跑的尝试各有自己的单元名（单元由对账停的几句都靠它核）；弄坏开关 no-detach 下尝试不起成单元、单元名是空，这一格红（不看对账先到还是取消先到）。"""
    releases = [os.path.join(world.root, f"release-association-{index}") for index in range(4)]
    world.start_server()
    problems = []
    try:
        owner = world.submit(blocking_command(releases[0]))
        other = world.submit(blocking_command(releases[1]))
        owner_summary, other_summary = world.running(owner), world.running(other)
        owner_unit, other_unit = owner_summary["attempts"][0].get("unit", ""), other_summary["attempts"][0].get("unit", "")
        if not owner_unit or not other_unit:
            problems.append(f"任务 {owner}、{other} 在跑的尝试应当各起成一个单元（脱离服务），实际单元名 {owner_unit!r}、{other_unit!r}")
        raw_column_update(world, "attempts", 0, other_summary["attempts"][0]["id"])   # (task_id, number) 唯一索引只管 task_id > 0：先把任务 B 自己的 1 号尝试挪到 0 名下，腾出那一号
        raw_column_update(world, "attempts", other, owner_summary["attempts"][0]["id"])
        cancelled = world.client(["cancel", str(other)])
        listed = world.summary(other)["attempts"]
        if cancelled.returncode == 0:
            problems.append(f"尝试的 task_id 列改到任务 {other} 名下：取消 {other} 应当不退 0，实际退 {cancelled.returncode}")
        patience = world.patience(STATE_TIMEOUT_SECONDS)
        while patience.waiting() and (world.unit_active(owner_unit) or world.unit_active(other_unit)):
            time.sleep(POLL_SECONDS)
        if world.unit_active(owner_unit) or world.unit_active(other_unit):
            problems.append(f"关联列被改的两次尝试的单元应当由对账停掉，实际 {owner} 的单元{'在跑' if world.unit_active(owner_unit) else '停了'}、{other} 的单元{'在跑' if world.unit_active(other_unit) else '停了'}")
        if any(attempt.get("id") == owner_summary["attempts"][0]["id"] and attempt.get("valid") is not False for attempt in listed):
            problems.append(f"任务 {other} 名下读回的那一行（正文是任务 {owner} 的）应当校验不过（或已被对账挂回 {owner}、不在 {other} 名下），"
                            f"实际 {[(attempt.get('id'), attempt.get('valid')) for attempt in listed]}")
        first = world.submit(world.write_spec("spec-association-first.json", {"shards": [{"block": releases[2]}]}), cwd=world.repo)
        second = world.submit(world.write_spec("spec-association-second.json", {"shards": [{"block": releases[3]}]}), cwd=world.repo)
        running = lambda summary: len(shard_children(summary)) == 1 and any(attempt.get("state") == "running" for child in shard_children(summary).values()
                                                                             for attempt in child["attempts"])
        first_child = list(shard_children(world.wait_for(first, running, "那一片跑起来")).values())[0]
        second_child = list(shard_children(world.wait_for(second, running, "那一片跑起来")).values())[0]
        raw_column_update(world, "tasks", second, first_child["task"]["id"])
        cancelled = world.client(["cancel", str(second)])
        units = [first_child["attempts"][-1].get("unit", ""), second_child["attempts"][-1].get("unit", "")]
        _, repaired, column = verified_row(world, "tasks", first_child["task"]["id"])
        if (cancelled.returncode == 0 and not (repaired and column == first)) or (cancelled.returncode != 0 and not world.unit_active(units[1])):
            problems.append(f"子任务 {first_child['task']['id']} 的 task_id 列改到分片的任务 {second} 名下：没判出时整件不退 0、"
                            f"它自己的片不停，判出之后那一行挂回 {first}、核得过，实际退 {cancelled.returncode}、在跑的单元 {[unit for unit in units if world.unit_active(unit)]}、"
                            f"那一行核{'得过' if repaired else '不过'}、关联列 {column}")
    finally:
        for release in releases:
            open(release, "w").close()
    if problems:
        raise SelftestFailure("关联列：" + "；".join(problems), world.server_log_lines())
    return "只改尝试或子任务的 task_id 列：挂过去的那一行读回校验不过，取消那一件回不了 stopped，单元由对账停"


def cell_prune_server_unavailable_outcome(world):
    """命令自己退 75（prune-server 不可用）：结局单列 prune-server-unavailable、wait 退 75、末行 type 带它，不重排；服务起一次 prune-server/control check
    （假的，照 prune-check-exit 退）：不通写一条告警 prune-server-unavailable，通了不写。"""
    world.start_server(extra={"TASK_SERVER_PRUNE_SERVER_CHECK": os.path.join(world.bin, "fake-prune-check")})
    problems = []
    for name, check_exit, alerted in (("check 不通", "78", True), ("check 通", "0", False)):
        with open(os.path.join(world.world, "prune-check-exit"), "w", encoding="utf-8") as handle:
            handle.write(check_exit + "\n")
        checks = len([line for line in world.calls() if line.startswith("prune-check ")])
        task_id = world.submit(["bash", "-c", "exit 75"])
        summary = world.finished(task_id)
        wait_code, last = wait_last_line(world, task_id)
        patience = world.patience(STATE_TIMEOUT_SECONDS)
        while patience.waiting() and len([line for line in world.calls() if line.startswith("prune-check ")]) <= checks:
            time.sleep(POLL_SECONDS)
        time.sleep(0.5)   # check 退了之后写告警那一下
        written = [alert for alert in world.alerts() if alert.get("kind") == lib.PRUNE_SERVER_ALERT_KIND and alert.get("task") == task_id]
        if (summary["task"].get("outcome") != lib.PRUNE_SERVER_UNAVAILABLE_OUTCOME or summary["task"].get("exit_code") != 75 or len(summary["attempts"]) != 1
                or wait_code != 75 or f"type={lib.PRUNE_SERVER_UNAVAILABLE_OUTCOME} " not in last + " "):
            problems.append(f"{name}：命令退 75 应当记 {lib.PRUNE_SERVER_UNAVAILABLE_OUTCOME}、退 75、不重排、wait 退 75、末行 type 带它，实际 {describe(summary)}、"
                            f"wait 退 {wait_code}、末行「{last}」")
        if bool(written) != alerted:
            problems.append(f"{name}：应当{'写' if alerted else '不写'}一条 {lib.PRUNE_SERVER_ALERT_KIND} 告警，实际 {len(written)} 条（check 起了 "
                            f"{len([line for line in world.calls() if line.startswith('prune-check ')]) - checks} 次）")
    if problems:
        raise SelftestFailure("退 75：" + "；".join(problems), world.server_log_lines())
    return f"命令退 75 记 {lib.PRUNE_SERVER_UNAVAILABLE_OUTCOME}、wait 退 75；check 不通写告警、通了不写"


def raw_record_update(world, table, row_id, changes):
    """绕过服务只改一行的正文（并进 changes；关联列与签名不动）：读回时那一行校验不过。"""
    with contextlib.closing(sqlite3.connect(os.path.join(world.state, lib.DATABASE_NAME), timeout=30, isolation_level=None)) as connection:
        (text,) = connection.execute(f"SELECT record FROM {table} WHERE id = ?", (row_id,)).fetchone()
        connection.execute(f"UPDATE {table} SET record = ? WHERE id = ?", (lib.canonical_json(dict(json.loads(text), **changes)), row_id))


def cell_child_rows_verified(world):
    """children_of 不丢校验位（TS-05）：一件分片跑完之后绕过服务改它那一片子任务行的正文：task clean 回 409、一份都不删；status 不列那一行、
    children_unverified 点名它，写了 record-tampered 告警。弄坏开关 child-rows-unverified 下照收那一行，这一格红。"""
    world.start_server()
    task_id = world.submit(world.write_spec("spec-child-rows.json", {"shards": [{}]}), cwd=world.repo)
    finished = world.finished(task_id)
    children = shard_children(finished)
    if finished["task"].get("outcome") != "succeeded" or 0 not in children:
        raise SelftestFailure(f"一片的分片应当成了，实际 {describe(finished)}、各片 {sorted(children)}", world.server_log_lines())
    child_id = children[0]["task"]["id"]
    raw_record_update(world, "tasks", child_id, {"note": "自证：绕过服务改了正文"})
    cleaned = world.client(["clean", str(task_id)])
    status = world.summary(task_id)
    listed = [child["task"].get("id") for child in status.get("children", [])]
    alerted = [alert for alert in world.alerts() if alert.get("kind") == "record-tampered" and alert.get("task") == task_id]
    problems = []
    if cleaned.returncode == 0:
        problems.append(f"子任务行 {child_id} 校验不过：task clean 应当退非 0、一份都不删，实际退 0（{cleaned.stdout.strip()[-200:]}）")
    if f"行 [{child_id}]" not in status.get("children_unverified", "") or child_id in listed or not alerted:
        problems.append(f"status 应当不列子任务行 {child_id}、children_unverified 点名它、写 record-tampered 告警，实际列了 {listed}、"
                        f"children_unverified「{status.get('children_unverified', '')}」、告警 {len(alerted)} 条")
    if problems:
        raise SelftestFailure("子任务行校验：" + "；".join(problems), world.server_log_lines())
    return "子任务行的正文被改过：clean 回 409 不删，status 不列它、点名它、写告警"


CONTROL_CHECK_LIMIT_SECONDS = 3   # control-check-probe-bounded：现探的总时长上限（--limit）
CONTROL_CHECK_EXTRA_SECONDS = 8   # 到点之后再多给这么久退出（点名卡住那一项、收子进程）


def cell_control_check_probe_bounded(world):
    """control check --probe 的总时长有界、分步可见（task-server/control 的 probed_nfs、task_server_lib.py 的 NfsProbeBudget；两台都堵时 control check 一百秒以上不出结论，
    各会话以为服务卡死）：服务起着、假 ssh 把挂载那一句（stat -f）睡死（fake-ssh-hang）；control check --probe --limit 3：到点退 1（不退 0、不退 78）、不超过 3 秒加 8 秒才退，
    标准输出里打了「目录」「导出」各一对开始与结束、「挂载」开始，标准错误点名卡在哪一个根的「挂载」、写总时长到点。
    弄坏开关 check-probe-unbounded 下现探不设总时长，一直等下去（客户端不在时限里退），这一格红。"""
    world.start_server()
    hang = os.path.join(world.world, "fake-ssh-hang")
    open(hang, "w").close()
    problems = []
    try:
        started = time.monotonic()
        checked = world.control(["check", "--probe", "--limit", str(CONTROL_CHECK_LIMIT_SECONDS)], timeout=CONTROL_CHECK_LIMIT_SECONDS + CONTROL_CHECK_EXTRA_SECONDS)
        took = time.monotonic() - started
    finally:
        with contextlib.suppress(FileNotFoundError):
            os.remove(hang)
    mount = f"共享根 {world.shared_root} 的「挂载」"
    if checked.returncode != 1 or took > CONTROL_CHECK_LIMIT_SECONDS + CONTROL_CHECK_EXTRA_SECONDS:
        problems.append(f"总时长 {CONTROL_CHECK_LIMIT_SECONDS} 秒到点应当退 1、{CONTROL_CHECK_LIMIT_SECONDS + CONTROL_CHECK_EXTRA_SECONDS} 秒内退出，实际退 {checked.returncode}、用了 {took:.0f} 秒")
    for step in (f"共享根 {world.shared_root} 的「目录」开始", f"共享根 {world.shared_root} 的「目录」结束", f"共享根 {world.shared_root} 的「导出」结束", f"{mount}开始"):
        if step not in checked.stdout:
            problems.append(f"现探每一项开始、结束各打一行：标准输出里应当有「{step}」，实际「{checked.stdout.strip()[-400:]}」")
    if f"卡在 {mount}" not in checked.stderr or "到点" not in checked.stderr:
        problems.append(f"到点应当打出卡在「{mount}」、写总时长到点，实际标准错误「{checked.stderr.strip()[-400:]}」")
    if f"{mount}结束" in checked.stdout:
        problems.append("睡死的那一项不该打「结束」")
    if problems:
        raise SelftestFailure("control check --probe 总时长有界、分步可见：" + "；".join(problems), world.server_log_lines())
    return f"挂载那一句睡死：--limit {CONTROL_CHECK_LIMIT_SECONDS} 到点退 1（用了 {took:.0f} 秒），逐项打了开始与结束、点名卡在「挂载」"


def cell_control_check_reads_service_nfs_report(world):
    """control check 默认不现探，读服务 /health 的 nfs 一栏（task-server/control 的 reported_nfs；写明是多久以前判的）：服务起着、NFS 四项都过，假 ssh 把挂载那一句睡死之后
    control check 照样几秒内退 0，输出写「秒前判的前三项」与「两向探针」「要现探加 --probe」，没有「开始」「结束」那种现探的逐项行。
    弄坏开关 check-nfs-probed-live-by-default 下不带 --probe 也现探，睡死的那一句让它一直等，这一格红。"""
    world.start_server()
    hang = os.path.join(world.world, "fake-ssh-hang")
    open(hang, "w").close()
    problems = []
    try:
        started = time.monotonic()
        checked = world.control(["check"], timeout=CONTROL_CHECK_EXTRA_SECONDS + CONTROL_CHECK_LIMIT_SECONDS)
        took = time.monotonic() - started
    finally:
        with contextlib.suppress(FileNotFoundError):
            os.remove(hang)
    if checked.returncode != 0 or took > CONTROL_CHECK_EXTRA_SECONDS + CONTROL_CHECK_LIMIT_SECONDS:
        problems.append(f"默认读服务最近一份结论、第二台睡死也应当几秒内退 0，实际退 {checked.returncode}、用了 {took:.0f} 秒、「{(checked.stdout + checked.stderr).strip()[-300:]}」")
    for text in ("秒前判的前三项", "两向探针", "要现探加 --probe"):
        if text not in checked.stdout:
            problems.append(f"输出应当写明是多久以前判的（「{text}」），实际「{checked.stdout.strip()[-400:]}」")
    if "开始" in checked.stdout:
        problems.append(f"默认路径不该现探（不该有逐项「开始」行），实际「{checked.stdout.strip()[-400:]}」")
    if problems:
        raise SelftestFailure("control check 默认读服务最近一份 NFS 结论：" + "；".join(problems), world.server_log_lines())
    return f"第二台睡死时 control check 默认读 /health 的 nfs 一栏、{took:.0f} 秒退 0，写明多久以前判的，没有现探的逐项行"


def cell_slow_world_patient_wait(world):
    """慢世界（上面常量处「等待的判法」）：一条命令每 SLOW_WORLD_STEP_SECONDS 秒往日志与世界目录各写一笔、共 SLOW_WORLD_STEPS 步，总共跑得比安静时限
    SLOW_WORLD_BASE_SECONDS 久；等它收尾的 wait_for 与客户端的 wait 都拿这个安静时限等：世界一直在变，应当等到收尾、判绿。另一头：收尾之后世界不再变，
    等一个永远不成立的条件，应当安静满时限就判红（不是拖到绝对上限）。弄坏开关 wait-limit-fixed 下等待退回写死的时限，前一头在安静时限处判红。"""
    world.start_server()
    beat = os.path.join(world.world, "slow-beat")
    script = f'for step in $(seq 1 {SLOW_WORLD_STEPS}); do echo "slow step $step"; echo x >> "$1"; sleep {SLOW_WORLD_STEP_SECONDS}; done; exit 0'
    problems = []

    def slow_task(name):
        return world.submit(["bash", "-c", script, name, beat])

    first_id = slow_task("slow-world-first")
    began = time.monotonic()
    try:
        first = world.wait_for(first_id, lambda found: found["task"].get("state") == "finished", "收尾", timeout=SLOW_WORLD_BASE_SECONDS)
        took = time.monotonic() - began
        if first["task"].get("outcome") != "succeeded" or took <= SLOW_WORLD_BASE_SECONDS:
            problems.append(f"慢世界里 wait_for 应当等到收尾（命令跑 {SLOW_WORLD_STEPS * SLOW_WORLD_STEP_SECONDS} 秒、比安静时限 {SLOW_WORLD_BASE_SECONDS:.0f} 秒久），"
                            f"实际 {describe(first)}、等了 {took:.1f} 秒")
    except SelftestFailure as failure:
        problems.append(f"慢世界里 wait_for 应当一直等到收尾（世界一直在变），实际在安静时限处判了红：{failure}")
    second_id = slow_task("slow-world-second")
    began = time.monotonic()
    try:
        completed = world.client(["wait", str(second_id)], timeout=SLOW_WORLD_BASE_SECONDS)
        took = time.monotonic() - began
        if completed.returncode != 0 or took <= SLOW_WORLD_BASE_SECONDS:
            problems.append(f"慢世界里客户端 wait 应当等到收尾、退 0，实际退 {completed.returncode}、等了 {took:.1f} 秒")
    except SelftestFailure as failure:
        problems.append(f"慢世界里客户端 wait 应当一直等到收尾（世界一直在变），实际在安静时限处判了红：{failure}")
    world.finished(second_id)   # 判红的话命令还在跑，等它收尾再测安静的那一头（这一句用平常的安静时限）
    began = time.monotonic()
    ceiling = SLOW_WORLD_BASE_SECONDS * WAIT_CEILING_FACTOR
    try:
        world.wait_for(second_id, lambda found: False, "永远不成立", timeout=SLOW_WORLD_BASE_SECONDS)
        problems.append("收尾之后世界不再变，等一个永远不成立的条件应当判红，实际等到了")
    except SelftestFailure:
        quiet = time.monotonic() - began
        if quiet < SLOW_WORLD_BASE_SECONDS or quiet >= ceiling - 1:
            problems.append(f"世界不再变时应当安静满 {SLOW_WORLD_BASE_SECONDS:.0f} 秒就判红（绝对上限 {ceiling:.0f} 秒之前），实际等了 {quiet:.1f} 秒")
    if problems:
        raise SelftestFailure("慢世界里的等待：" + "；".join(problems), world.server_log_lines())
    return (f"命令跑 {SLOW_WORLD_STEPS * SLOW_WORLD_STEP_SECONDS} 秒、安静时限 {SLOW_WORLD_BASE_SECONDS:.0f} 秒：wait_for 与客户端 wait 都等到了收尾；"
            f"收尾之后世界不再变，{quiet:.1f} 秒就判红（绝对上限 {ceiling:.0f} 秒）")


DROP_HISTORY_LIMIT_GIBIBYTES = 1           # control-drop-history-oversized-records：--attempt-records-over-gib 1
DROP_HISTORY_CAP_BYTES = 800 * MEBIBYTE    # 撞上限那几行的上限：本身不过 1G，乘 1.5 的 1200M 过 1G
DROP_HISTORY_NEAR_CAP_PEAK_BYTES = 790 * MEBIBYTE   # 峰值到了上限（不小于一半）：按上限进定额度
DROP_HISTORY_LOW_PEAK_BYTES = 100 * MEBIBYTE        # 峰值远低于上限（不到一半）：按峰值进定额度
DROP_HISTORY_SMALL_PEAK_BYTES = 500 * MEBIBYTE


def drop_history_rows(world):
    """control-drop-history-oversized-records：照服务读回的规则读 attempts，交 [(命令键, 结局)]（照行号先后）。"""
    return [(record.get("command_key"), record.get("outcome")) for record, _ in world.signed_rows("attempts")]


def cell_control_drop_history_oversized_records(world):
    """control drop-history --attempt-records-over-gib（task-server/control 的 drop_history；撞上限那一行进定额度的量照 task_server_lib.py 的 cap_hit_record_bytes 算，
    服务的 cap_hit_measurement 经同一处）：库里造六行——跑完、峰值 2G（过 1G）；撞了 800M 上限、峰值到了上限、之后这把键没跑完过（进定额度 1200M，过 1G）；
    跑完、峰值 500M；撞了 800M 上限而峰值只有 100M（远低于上限，按峰值进 100M）；同一把键撞了 800M 上限、之后又跑完一次 500M（撞的那一行按上限本身进 800M）两行。
    服务在跑（is-active 打 active）时拒：退 1、说服务还没停、一行不删、不拷备份；停了之后 --attempt-records-over-gib 1 删前两行、其余四行都在，
    删之前的库拷在 .before-drop-history- 那一份里、六行都在。
    弄坏开关 drop-history-cap-hits-ignored 下撞上限那一行不删，cap-hit-low-peak-counted-at-cap 下峰值 100M 的那一行也删，这一格红。"""
    world.start_server()
    rows = (("selftest-drop-peak-over", "succeeded", 2 * GIBIBYTE, None),
            ("selftest-drop-cap-hit-over", "memory-cap-hit", DROP_HISTORY_NEAR_CAP_PEAK_BYTES, DROP_HISTORY_CAP_BYTES),
            ("selftest-drop-small", "succeeded", DROP_HISTORY_SMALL_PEAK_BYTES, None),
            ("selftest-drop-cap-hit-low-peak", "memory-cap-hit", DROP_HISTORY_LOW_PEAK_BYTES, DROP_HISTORY_CAP_BYTES),
            ("selftest-drop-cap-hit-resolved", "memory-cap-hit", DROP_HISTORY_NEAR_CAP_PEAK_BYTES, DROP_HISTORY_CAP_BYTES),
            ("selftest-drop-cap-hit-resolved", "succeeded", DROP_HISTORY_SMALL_PEAK_BYTES, None))
    for offset, (key, outcome, peak, cap) in enumerate(rows):
        seed_ended_attempt(world, key, SEEDED_ENDED_AT + offset, outcome, memory_peak=peak, memory_cap=cap)
    world.stop_server()
    seeded = drop_history_rows(world)
    extra = {"TASK_SERVER_CONTROL_SYSTEMCTL": os.path.join(world.bin, "fake-systemctl")}
    state_file = os.path.join(world.world, "systemctl-state")
    backup_prefix = lib.DATABASE_NAME + ".before-drop-history-"
    arguments = ["drop-history", "--attempt-records-over-gib", str(DROP_HISTORY_LIMIT_GIBIBYTES)]
    problems = []
    if len(seeded) != len(rows):
        problems.append(f"造的 {len(rows)} 行应当都进了库，实际读回 {seeded}")
    with open(state_file, "w", encoding="utf-8") as handle:
        handle.write("active\n")
    refused = world.control(arguments, extra=extra)
    refused_text = refused.stdout + refused.stderr
    backups = [name for name in os.listdir(world.state) if name.startswith(backup_prefix)]
    if refused.returncode != 1 or "服务还没停干净" not in refused_text or drop_history_rows(world) != seeded or backups:
        problems.append(f"服务在跑（is-active 打 active）时应当拒：退 1、说服务还没停干净、一行不删、不拷备份；实际退 {refused.returncode}、"
                        f"库里 {drop_history_rows(world)}、备份 {backups}、输出「{refused_text.strip()[-300:]}」")
    with open(state_file, "w", encoding="utf-8") as handle:
        handle.write("inactive\n")
    dropped = world.control(arguments, extra=extra)
    dropped_text = dropped.stdout + dropped.stderr
    left = drop_history_rows(world)
    wanted = [row for row in seeded if row[0] not in ("selftest-drop-peak-over", "selftest-drop-cap-hit-over")]
    if dropped.returncode != 0 or left != wanted or f"进定额度超过 {DROP_HISTORY_LIMIT_GIBIBYTES} GiB 的尝试 2 行" not in dropped_text:
        problems.append(f"停了之后 --attempt-records-over-gib {DROP_HISTORY_LIMIT_GIBIBYTES} 应当退 0、只删峰值 2G 的与撞了 800M 上限而之后没跑完过的那两行（报删了 2 行），"
                        f"峰值 500M 的、峰值远低于上限的撞上限那一行、之后跑完过的撞上限那一行与它跑完的那一行都留着；实际退 {dropped.returncode}、库里 {left}、"
                        f"输出「{dropped_text.strip()[-400:]}」")
    backups = [name for name in os.listdir(world.state) if name.startswith(backup_prefix)]
    if len(backups) != 1:
        problems.append(f"删之前的库应当拷一份 {backup_prefix}<时刻>，实际 {backups}")
    else:
        with contextlib.closing(sqlite3.connect(os.path.join(world.state, backups[0]), timeout=30)) as connection:
            kept = connection.execute("SELECT COUNT(*) FROM attempts").fetchone()[0]
        if kept != len(rows):
            problems.append(f"备份 {backups[0]} 里应当是删之前的 {len(rows)} 行，实际 {kept} 行")
    if problems:
        raise SelftestFailure("drop-history 删进定额度超限的尝试行：" + "；".join(problems))
    return f"服务在跑时拒；停了之后删进定额度超过 {DROP_HISTORY_LIMIT_GIBIBYTES} GiB 的两行、留四行、备份六行都在"


RUNNING_ATTEMPTS_QUERY = "state IN ('starting', 'running')"
RUNNING_ATTEMPTS_QUERY_HOLDERS = ["reconcile", "running_attempts"]


def code_objects_of(module):
    """module 里定义的函数与方法（staticmethod、classmethod、property 拆开），连同套在里面的函数的代码对象。读编译出来的代码对象、不读源码原文：原文进这一份判定的键，注释一改键就变。"""
    pending, found = [], []
    for value in vars(module).values():
        members = vars(value).values() if isinstance(value, type) and value.__module__ == module.__name__ else (value,)
        for member in members:
            unwrapped = getattr(member, "__func__", member)
            function = getattr(unwrapped, "fget", unwrapped)
            if isinstance(function, types.FunctionType) and function.__module__ == module.__name__:
                pending.append(function.__code__)
    while pending:
        code = pending.pop()
        found.append(code)
        pending += [constant for constant in code.co_consts if isinstance(constant, types.CodeType)]
    return found


def cell_running_attempts_verified_only(world):
    """取在跑尝试只收校验过的行（缺陷五；task_server.py 的 Store.running_attempts，十处记账都调它）：这一格的临时库里两条在跑的尝试，后一条只改了关联列：
    running_attempts 只交前一条；server_pending（内存）、server_disk_pending 都只算前一条（套在假调度器上，不起服务）；
    task_server.py 里取在跑尝试的那一句查询只在 running_attempts 与对账（reconcile）两个函数里（读编译出来的代码对象，code_objects_of）。弄坏开关 running-attempts-unverified 下连那一行一起算，这一格红。"""
    store = SERVER.Store(os.path.join(world.root, "running.sqlite3"), bytes(32), dict(os.environ))
    common = {"state": "running", "session": "", "command_key": "", "host": SERVER.LOCAL, "number": 1, "started_at": time.time()}
    honest = store.insert("attempts", dict(common, task_id=1, memory_cap=4 * GIBIBYTE, disk_booked=2 * GIBIBYTE, threads=4))
    tampered = store.insert("attempts", dict(common, task_id=2, memory_cap=8 * GIBIBYTE, disk_booked=3 * GIBIBYTE, threads=8))
    with contextlib.closing(sqlite3.connect(store.path, timeout=30, isolation_level=None)) as connection:
        connection.execute("UPDATE attempts SET task_id = ? WHERE id = ?", (3, tampered["id"]))   # 只改关联列（TS-05 那一种）
    fake = object.__new__(SERVER.Scheduler)
    fake.store, fake.environment = store, dict(os.environ)
    fake.unit_stopped = set()   # booking_attempts 不数的、单元已确认停了的尝试号（Scheduler.__init__ 建的）
    fake.machines = {SERVER.LOCAL: types.SimpleNamespace(build_bytes=lambda attempt: 0)}
    fake.report_scope_usage = lambda report: {}
    problems = []
    listed = [attempt.get("id") for attempt in store.running_attempts()]
    if listed != [honest["id"]]:
        problems.append(f"running_attempts 应当只交校验过的那一条 {honest['id']}，实际 {listed}")
    checks = (("server_pending", SERVER.Scheduler.server_pending(fake, SERVER.LOCAL, None), 4 * GIBIBYTE),
              ("server_disk_pending", SERVER.Scheduler.server_disk_pending(fake, SERVER.LOCAL), 2 * GIBIBYTE))
    problems += [f"{name} 应当只算校验过的那一条（{wanted}），实际 {seen}" for name, seen, wanted in checks if seen != wanted]
    holders = sorted(code.co_name for module in server_modules() for code in code_objects_of(module) if RUNNING_ATTEMPTS_QUERY in code.co_consts)
    if holders != RUNNING_ATTEMPTS_QUERY_HOLDERS:
        problems.append(f"task_server.py 里取在跑尝试的查询应当只在 {RUNNING_ATTEMPTS_QUERY_HOLDERS} 里，实际 {holders}：别处各写的一份不看校验位")
    if problems:
        raise SelftestFailure("在跑尝试只收校验过的：" + "；".join(problems))
    return "关联列被改的那一条在跑尝试不进 running_attempts，内存与盘的记账都不算它；那一句查询只在两个函数里"


def cell_resident_entries_forgotten(world):
    """按任务号记在内存里的表随结局回收（缺陷九；task_server.py 的 forget_settled_entries，套在假调度器上，不起服务）：这一格的临时库里五条任务——
    有了结局并被取走的、有了结局没人取而结局才一分钟的、有了结局没人取而结局满 RESIDENT_RETENTION_SECONDS 的、还在排的、库里没有的；
    七张按任务号记的表与 inbox_failed 里各有它们的项，清一回只留没人取的新结局与还在排的；按尝试号记的只留在跑的那一次。
    弄坏开关 resident-entries-kept 下一项都不删，这一格红。"""
    store = SERVER.Store(os.path.join(world.root, "resident.sqlite3"), bytes(32), dict(os.environ))
    now = time.time()

    def task(**fields):
        return store.insert("tasks", dict({"task_id": 0, "state": "finished", "session": "", "command_key": "", "collected": False, "finished_at": now - 60}, **fields))["id"]

    collected, fresh = task(collected=True), task()
    stale, queued = task(finished_at=now - SERVER.RESIDENT_RETENTION_SECONDS - 60), task(state="queued", finished_at=None)
    gone = queued + 1000
    running = store.insert("attempts", {"task_id": queued, "number": 1, "state": "running", "session": "", "command_key": ""})["id"]
    ended = store.insert("attempts", {"task_id": collected, "number": 1, "state": "ended", "session": "", "command_key": ""})["id"]
    ids = (collected, fresh, stale, queued, gone)
    fake = object.__new__(SERVER.Scheduler)
    fake.store, fake.environment, fake.resident_pruned_at, fake.resident_prune_seconds = store, dict(os.environ), 0.0, 0.0
    fake.alerted_child_refusals = {(task_id, "名下有校验不过的子任务记录") for task_id in ids}
    fake.reminded_thresholds = {(task_id, "quiet-log-busy"): 1 for task_id in ids}
    fake.reminder_histories = {(task_id, "quiet-log-busy"): [] for task_id in ids}
    fake.workspace_write_checks = {task_id: None for task_id in ids}
    fake.advance_episodes = {task_id: 1 for task_id in ids}
    # 删了 advance_failures、advance_episodes 的项，库里那一份跟着写（persist_restart_surviving_state）：Scheduler.__init__ 建的那几样照建
    fake.advance_failures = {task_id: {"episode": 1, "why": "推不动", "count": 1} for task_id in ids}
    fake.attention_lock = threading.Lock()
    fake.restart_surviving_lock, fake.restart_surviving_written = threading.Lock(), {}
    fake.unclaimed_repushed = {task_id: 1 for task_id in ids}
    fake.suspect_reported, fake.suspect_lock, fake.health_notified = {running: 1, ended: 1}, threading.Lock(), {}   # 报过的段随记进库（persist_restart_surviving_state）：这几样照建
    fake.booking_drift_notified, fake.booking_drift_lock = {}, threading.Lock()
    fake.inbox_failed = {("outcome", task_id) for task_id in ids} | {(task_id, "quiet-log-busy", 1) for task_id in ids} | {("suspect", running, 1), ("suspect", ended, 1)}
    SERVER.Scheduler.forget_settled_entries(fake, now)
    kept = {fresh, queued}
    seen = {name: {key[position] if position is not None else key for key in vars(fake)[name]} for name, position in SERVER.RESIDENT_TASK_TABLES.items()}
    problems = [f"{name} 应当只留任务 {sorted(kept)} 的项，实际 {sorted(found)}" for name, found in seen.items() if found != kept]
    if set(fake.suspect_reported) != {running}:
        problems.append(f"suspect_reported 应当只留在跑的那一次 {running}，实际 {sorted(fake.suspect_reported)}")
    wanted_events = {("outcome", task_id) for task_id in kept} | {(task_id, "quiet-log-busy", 1) for task_id in kept} | {("suspect", running, 1)}
    if fake.inbox_failed != wanted_events:
        problems.append(f"inbox_failed 应当只留 {sorted(map(str, wanted_events))}，实际 {sorted(map(str, fake.inbox_failed))}")
    if problems:
        raise SelftestFailure("内存里的表随结局回收：" + "；".join(problems))
    return "取走的、结局满保留时限的、库里没有的任务的项都删了，没人取的新结局与在排的留着；按尝试号记的只留在跑的"


def cell_retired_outcome_status_generic(world):
    """库里记着退役结局的旧任务（lib.RETIRED_OUTCOME_KINDS，结局 over-two-hours）：task status <号> 不崩、退 0，TASK_SERVER_REASON 的 next= 用通用那一句（lib.ANOMALY_GENERIC_ADVICE）。
    弄坏开关 retired-outcome-kinds-unknown 下抛 KeyError，这一格红。"""
    world.start_server()
    outcome = lib.RETIRED_OUTCOME_KINDS[0]
    task_id = world.insert_signed_row("tasks", {"command": ["bash", "-c", "exit 0", "retired-outcome"], "cwd": world.work, "environment": {}, "session": SESSION_A, "submitter_pid": 0,
                                                "submitted_at": time.time(), "finished_at": time.time(), "command_key": "selftest-retired-outcome", "state": "finished", "outcome": outcome,
                                                "exit_code": 1, "collected": True, "next_cap": None, "next_host": "", "outcome_counts": {}, "note": "", "cancel_requested": False,
                                                "attempt_count": 0, "task_id": 0})
    shown = world.client(["status", str(task_id)])
    advice = lib.ANOMALY_GENERIC_ADVICE.replace(lib.ANOMALY_TASK_PLACEHOLDER, str(task_id))
    reason = next((line for line in shown.stdout.splitlines() if line.startswith(lib.REASON_MARK)), "")
    if shown.returncode != 0 or f"type={outcome}" not in reason or advice not in reason:
        raise SelftestFailure(f"结局是 {outcome} 的旧任务：status 应当退 0、理由行 next= 用通用那一句「{advice}」，实际退 {shown.returncode}、理由行「{reason}」、"
                              f"输出「{(shown.stdout + shown.stderr).strip()[-400:]}」", world.server_log_lines())
    return f"结局是 {outcome} 的旧任务 status 不崩、next= 用通用那一句"


def query_plan_text(store, query, parameters):
    """一条查询在这份库上的查询计划（EXPLAIN QUERY PLAN 各行的说明接起来）。"""
    return " ".join(str(row[-1]) for row in store.connection().execute("EXPLAIN QUERY PLAN " + query, parameters).fetchall())


def cell_lookup_indexes_used(world):
    """查得勤的几种走索引（task_server.py 的 Store.LOOKUP_INDEXES；loop 审核发现 5）：按 task_id 取尝试与子任务、按命令键取结束了的尝试、按会话取任务，
    查询计划里各用 attempts_task、tasks_task、attempts_key_state、tasks_session，不整表扫。
    弄坏开关 lookup-indexes-skipped 下不建，这一格红。"""
    store = SERVER.Store(os.path.join(world.root, "lookup-indexes.sqlite3"), bytes(32), dict(os.environ))
    problems = []
    for query, parameters, index in (("SELECT id FROM attempts WHERE task_id = ? ORDER BY id", (1,), "attempts_task"),
                                     ("SELECT id FROM tasks WHERE task_id = ? ORDER BY id", (1,), "tasks_task"),
                                     ("SELECT id FROM attempts WHERE command_key = ? AND state = 'ended' ORDER BY id DESC LIMIT 200", ("selftest",), "attempts_key_state"),
                                     ("SELECT id FROM tasks WHERE session = ? ORDER BY id", ("selftest",), "tasks_session")):
        plan = query_plan_text(store, query, parameters)
        if index not in plan:
            problems.append(f"「{query}」应当用 {index}，实际查询计划「{plan}」")
    if problems:
        raise SelftestFailure("查得勤的几种走索引：" + "；".join(problems))
    return "按 task_id、命令键、会话取的四种都走索引"


def cell_running_attempts_read_once(world):
    """在跑的尝试库没变就不重读（task_server.py 的 Store.running_attempts；loop 审核发现 8）：连读两次只查一次库；本进程写过一行之后、别的连接提交过之后各重读一次；
    改了交出来的那一份不染到下一次。弄坏开关 running-attempts-reread 下每次都查，这一格红。"""
    path = os.path.join(world.root, "running-read-once.sqlite3")
    store = SERVER.Store(path, bytes(32), dict(os.environ))
    attempt = store.insert("attempts", {"state": "running", "session": "", "command_key": "", "task_id": 0, "host": SERVER.LOCAL})
    reads = []
    original = store.select

    def counted(*arguments, **options):
        reads.append(arguments[0] if arguments else "")
        return original(*arguments, **options)
    store.select = counted
    problems = []
    first = store.running_attempts()
    store.running_attempts()
    if len(reads) != 1:
        problems.append(f"库没变连读两次：应当只查 1 次，实际 {len(reads)} 次")
    first[0]["host"] = SERVER.PEER
    if store.running_attempts()[0].get("host") != SERVER.LOCAL:
        problems.append("改了交出来的那一份：下一次交的不应当跟着变")
    store.update("attempts", dict(attempt, note="自证"))
    store.running_attempts()
    if len(reads) != 2:
        problems.append(f"本进程写过一行之后：应当重读（共 2 次），实际 {len(reads)} 次")
    with contextlib.closing(sqlite3.connect(path, timeout=5)) as connection:
        connection.execute("INSERT OR REPLACE INTO meta(name, value) VALUES('selftest-poke', '1')")
        connection.commit()
    store.running_attempts()
    if len(reads) != 3:
        problems.append(f"别的连接提交过之后：应当重读（共 3 次），实际 {len(reads)} 次")
    if problems:
        raise SelftestFailure("在跑的尝试库没变就不重读：" + "；".join(problems))
    return "库没变只读一次；本进程写过、别的连接提交过各重读一次"


def key_stats_seeder(world):
    """control key-stats 那几格往库里记尝试的帮手：每记一次结束时刻往后推 100 秒（起跑是结束前 1 秒，seed_attempt），交 run(键, 结局, 峰值 MiB, 任务号, 第几次, 峰值口径) → 这一次的起跑时刻。"""
    clock = [SEEDED_ENDED_AT]

    def run(key, outcome, peak=None, task_id=0, number=1, metric=lib.PROCESS_MEMORY_PEAK_METRIC):
        clock[0] += 100
        fields = {"command_key": key, "outcome": outcome, "ended_at": clock[0], "task_id": task_id, "number": number}
        if peak is not None:
            fields["memory_peak"] = peak * MEBIBYTE
        world.seed_attempt(fields, metric=metric)
        return clock[0] - 1
    return run


def cell_control_key_stats_reported(world):
    """control key-stats 只读打命令键的统计（task-server/control 的 key_stats_report，task_server_lib.py 的 key_stats、key_stats_lines；records/2026-11-17-subagent账本自上而下清欠.md 八十节 ①④ 回看要数的几样）：
    库里造六把键，每次尝试各是一个任务（任务号各不相同、都是第 1 次）——A 撞上限、跑成、撞上限、跑成（交替；撞过上限而最近一个任务跑成；跑成两次峰值各 100 MiB）；B 撞上限、跑成、跑成；
    C 跑成三次峰值 100、150、200 MiB；D 只跑过一次；E 跑成两次峰值 100、110 MiB；F 键里带工作根下副本路径、只跑过一次。读数：首行筛前筛后都是 14 行（没给 --since）、进统计的 14 行；
    命令键 6 个（只跑过一次的 2 个）、带副本路径的 1 个；按任务数交替的键 1 个；按任务数撞过上限、最近一个任务跑成的键 2 个；按尝试数交替的键 1 个；
    跑成两次以上的 4 个键最大比最小：中位 1.00 倍、第 90 百分位 2.00 倍、到 1.5 倍以上的 1 个；不打告警那一行。服务在跑时用（库只读打开），退 0。
    弄坏开关 key-stats-alternation-ignored 下按任务数交替的键数成 0，这一格红。"""
    world.start_server()
    run, task = key_stats_seeder(world), [0]

    def one(key, outcome, peak=None):
        task[0] += 1
        run(key, outcome, peak, task_id=task[0])

    for outcome, peak in (("memory-cap-hit", None), ("succeeded", 100), ("memory-cap-hit", None), ("succeeded", 100)):
        one("selftest-stats-a", outcome, peak)
    for outcome, peak in (("memory-cap-hit", None), ("succeeded", 100), ("succeeded", 100)):
        one("selftest-stats-b", outcome, peak)
    for peak in (100, 150, 200):
        one("selftest-stats-c", "succeeded", peak)
    one("selftest-stats-d", "succeeded", 100)
    for peak in (100, 110):
        one("selftest-stats-e", "succeeded", peak)
    one("selftest-stats-copy-singlefs/work/draft/f", "succeeded", 100)
    completed = world.control(["key-stats"])
    text = completed.stdout + completed.stderr
    problems = []
    if completed.returncode != 0:
        problems.append(f"只读的 key-stats 应当退 0，实际退 {completed.returncode}：「{text.strip()[-300:]}」")
    for wanted in ("attempts 筛前 14 行、筛后 14 行（没给 --since，不筛起跑时刻）", f"峰值口径是「{lib.PROCESS_MEMORY_PEAK_METRIC}」的）14 行",
                   "命令键 6 个（只跑过一次的 2 个），其中带工作根下副本或草稿目录路径的 1 个", "近 5 个任务里撞上限与跑成交替至少两回的键 1 个",
                   "近 5 个任务里撞过上限、最近一个任务跑成的键 2 个", "按尝试数：近 5 次尝试里撞上限与跑成交替至少两回的键 1 个",
                   "中位 1.00 倍、第 90 百分位 2.00 倍", "到 1.5 倍以上的 1 个"):
        if wanted not in text:
            problems.append(f"读数里应当有「{wanted}」")
    if "告警" in text:
        problems.append("读数里不该有告警那一行（服务不写这种告警）")
    if problems:
        raise SelftestFailure("control key-stats：" + "；".join(problems) + f"；实际输出：{text.strip()[-800:]}", world.server_log_lines())
    return "key-stats 只读打出：筛前筛后 14 行、6 把键（单跑 2、副本路径 1）、按任务数交替 1、撞过又跑成 2、按尝试数交替 1、峰值离散度中位 1.00 倍与第 90 百分位 2.00 倍、1.5 倍以上 1 个"


def cell_control_key_stats_by_task(world):
    """control key-stats 只取这一版峰值口径的尝试、交替按任务数（task_server_lib.py 的 key_stats、key_stats_after、current_peak_metric；
    research/prompts/ledger80-key-records-r1-main-verification.md「三、改法」第 2 条）：库里造四把键——
    「中间重试」任务甲跑成、任务乙第一次撞上限重试跑成、任务丙跑成（按尝试数成撞成成、交替；按任务数成撞成、也交替）；
    「起头重试」任务甲第一次撞、重试成，之后两个任务都成（按尝试数撞成成成、按任务数撞成成，都不交替；按任务数撞过而最近一个任务跑成）；
    「只有重试」两个任务都是第一次撞、重试成（按尝试数撞成撞成、交替；按任务数撞撞，不交替）；「老口径」三个任务撞、成、撞，尝试没有峰值口径一栏（不进统计）。
    读数：筛前筛后 15 行、进统计的 12 行；命令键 3 个；按任务数交替的键 1 个（中间重试）；按任务数撞过、最近一个任务跑成的 2 个；按尝试数交替的 2 个。
    --since 给「起头重试」第一次尝试的起跑时刻：筛后 11 行、进统计的 8 行、命令键 2 个、按任务数交替的 0 个；--since 给种下的那一天的下一天（YYYY-MM-DD）筛后 0 行；认不出的 --since 退 2。
    弄坏开关 key-stats-metric-unfiltered 下老口径的进统计（命令键 4 个），key-stats-retries-unfolded 下重试不折（按任务数交替的 2 个），
    key-stats-alternation-ignored 下按任务数交替的 0 个，key-stats-since-ignored 下 --since 不筛，这一格红。"""
    world.start_server()
    run = key_stats_seeder(world)
    for task_id, number, outcome in ((101, 1, "succeeded"), (102, 1, "memory-cap-hit"), (102, 2, "succeeded"), (103, 1, "succeeded")):
        run("selftest-stats-retry-middle", outcome, task_id=task_id, number=number)
    since = None
    for task_id, number, outcome in ((201, 1, "memory-cap-hit"), (201, 2, "succeeded"), (202, 1, "succeeded"), (203, 1, "succeeded")):
        started = run("selftest-stats-retry-first", outcome, task_id=task_id, number=number)
        since = started if since is None else since
    for task_id, number, outcome in ((301, 1, "memory-cap-hit"), (301, 2, "succeeded"), (302, 1, "memory-cap-hit"), (302, 2, "succeeded")):
        run("selftest-stats-retries-only", outcome, task_id=task_id, number=number)
    for task_id, outcome in ((401, "memory-cap-hit"), (402, "succeeded"), (403, "memory-cap-hit")):
        run("selftest-stats-old-metric", outcome, task_id=task_id, metric=None)
    metric = f"峰值口径是「{lib.PROCESS_MEMORY_PEAK_METRIC}」的）"
    next_day = time.strftime("%Y-%m-%d", time.gmtime(SEEDED_ENDED_AT + 86400))
    problems = []
    for what, arguments, wanted in (
            ("不筛起跑时刻", [], ("attempts 筛前 15 行、筛后 15 行", f"{metric}12 行", "命令键 3 个（只跑过一次的 0 个）",
                                  "近 5 个任务里撞上限与跑成交替至少两回的键 1 个（按任务数：同一任务的几次尝试折成一条，取第一次尝试的结局）：selftest-stats-retry-middle",
                                  "近 5 个任务里撞过上限、最近一个任务跑成的键 2 个", "按尝试数：近 5 次尝试里撞上限与跑成交替至少两回的键 2 个")),
            ("--since 起头重试的第一次", ["--since", f"{since:.0f}"], ("attempts 筛前 15 行、筛后 11 行", f"{metric}8 行", "命令键 2 个",
                                                                     "近 5 个任务里撞上限与跑成交替至少两回的键 0 个")),
            (f"--since {next_day}", ["--since", next_day], ("attempts 筛前 15 行、筛后 0 行", f"{metric}0 行", "命令键 0 个"))):
        completed = world.control(["key-stats", *arguments])
        text = completed.stdout + completed.stderr
        missing = [line for line in wanted if line not in text]
        if completed.returncode != 0 or missing:
            problems.append(f"{what}：应当退 0、读数里有{'、'.join('「' + line + '」' for line in missing)}，实际退 {completed.returncode}、输出「{text.strip()[-600:]}」")
    refused = world.control(["key-stats", "--since", "yesterday"])
    if refused.returncode != lib.USAGE_EXIT or "--since" not in refused.stderr:
        problems.append(f"认不出的 --since 应当退 {lib.USAGE_EXIT}、点名 --since，实际退 {refused.returncode}、「{(refused.stdout + refused.stderr).strip()[-300:]}」")
    if problems:
        raise SelftestFailure("control key-stats 按任务数、筛口径与起跑时刻：" + "；".join(problems), world.server_log_lines())
    return ("按任务数：中间重试的交替、起头重试与只有重试的不交替，按尝试数交替 2 个；老口径的键不进统计；--since 筛到 11 行、下一天筛到 0 行；认不出的 --since 退 2")


def cell_control_key_stats_dump(world):
    """control key-stats --dump 只读导出收尾了的尝试（task-server/control 的 key_stats_dump，task_server_lib.py 的 key_stats_dump_rows、KEY_STATS_DUMP_COLUMNS；
    research/prompts/ledger80-key-records-r1-main-verification.md「三、改法」第 3 条）：服务在跑，库里造 3 条尝试（任务 11 跑成、没记上限；任务 12 第一次撞上限、记了 stalled_at_cap，
    第二次跑成、记了 shard_count 与 item_plan）：导出 3 行，每行的列与 KEY_STATS_DUMP_COLUMNS 一样、次序一样，任务号与第几次照记的，尝试没有的栏写 null；
    同一个文件再导退 2、文件一个字节不动；--since 给任务 12 第一次的起跑时刻导到新文件里只有 2 行。
    弄坏开关 key-stats-dump-overwrites 下已在的照覆盖、退 0，key-stats-since-ignored 下 --since 那一份有 3 行，这一格红。"""
    world.start_server()
    world.seed_attempt({"command_key": "selftest-dump-a", "outcome": "succeeded", "exit_code": 0, "memory_peak": 100 * MEBIBYTE, "task_id": 11, "ended_at": SEEDED_ENDED_AT + 100})
    world.seed_attempt({"command_key": "selftest-dump-b", "outcome": "memory-cap-hit", "exit_code": 250, "memory_cap": GIBIBYTE, "stalled_at_cap": True,
                        "task_id": 12, "ended_at": SEEDED_ENDED_AT + 200})
    world.seed_attempt({"command_key": "selftest-dump-b", "outcome": "succeeded", "exit_code": 0, "memory_peak": 300 * MEBIBYTE, "shard_count": 4,
                        "item_plan": {"selftest-item": {"to_judge": 1, "reusable": 2}}, "task_id": 12, "number": 2, "ended_at": SEEDED_ENDED_AT + 300})
    since = SEEDED_ENDED_AT + 200 - 1   # 任务 12 第一次的起跑时刻（seed_attempt：结束前 1 秒）
    path, narrowed = os.path.join(world.root, "key-stats-dump.jsonl"), os.path.join(world.root, "key-stats-dump-since.jsonl")
    problems = []
    completed = world.control(["key-stats", "--dump", path])
    rows = []
    with contextlib.suppress(OSError, ValueError):
        with open(path, encoding="utf-8") as handle:
            rows = [json.loads(line) for line in handle]
    columns = list(lib.KEY_STATS_DUMP_COLUMNS)
    if completed.returncode != 0 or len(rows) != 3 or any(list(row) != columns for row in rows):
        problems.append(f"导出应当退 0、写 3 行、每行的列依次是 {'、'.join(columns)}，实际退 {completed.returncode}、{len(rows)} 行、"
                        f"列 {[list(row) for row in rows][:1]}、输出「{(completed.stdout + completed.stderr).strip()[-300:]}」")
    elif ([(row["task_id"], row["number"], row["command_key"]) for row in rows] != [(11, 1, "selftest-dump-a"), (12, 1, "selftest-dump-b"), (12, 2, "selftest-dump-b")]
          or rows[1]["stalled_at_cap"] is not True or rows[2]["shard_count"] != 4 or rows[0]["stalled_at_cap"] is not None
          or rows[2]["item_plan"] != {"selftest-item": {"to_judge": 1, "reusable": 2}} or rows[0]["memory_cap"] is not None
          or rows[0]["memory_peak"] != 100 * MEBIBYTE or rows[0]["peak_metric"] != lib.PROCESS_MEMORY_PEAK_METRIC or not all(isinstance(row["id"], int) for row in rows)):
        problems.append(f"导出的内容不对：应当按行号排、任务 11、12、12，记了的栏照记、没有的写 null，实际 {rows}")
    with open(path, "rb") as handle:
        before = handle.read()
    again = world.control(["key-stats", "--dump", path])
    with open(path, "rb") as handle:
        after = handle.read()
    if again.returncode != lib.USAGE_EXIT or after != before:
        problems.append(f"文件已在时应当退 {lib.USAGE_EXIT}、一个字节不动，实际退 {again.returncode}、文件{'没动' if after == before else '变了'}")
    narrowed_run = world.control(["key-stats", "--since", f"{since:.0f}", "--dump", narrowed])
    kept = []
    with contextlib.suppress(OSError, ValueError):
        with open(narrowed, encoding="utf-8") as handle:
            kept = [json.loads(line) for line in handle]
    if narrowed_run.returncode != 0 or [(row["task_id"], row["number"]) for row in kept] != [(12, 1), (12, 2)]:
        problems.append(f"--since 给任务 12 第一次的起跑时刻，应当只导任务 12 那 2 行，实际退 {narrowed_run.returncode}、{[(row.get('task_id'), row.get('number')) for row in kept]}")
    if problems:
        raise SelftestFailure("control key-stats --dump：" + "；".join(problems), world.server_log_lines())
    return "服务在跑：导出 3 行、列齐、次序对、没有的栏写 null；文件已在退 2 不动；--since 只导 2 行"


def cell_confirm_retries_follow_churn(world):
    """自证帮手确认单（World.confirm_submitted）数对不上时按同会话在排在跑的条数放宽重试回数（有界）：服务核数在会话锁里算了就比（task_server.py 的 confirm、counts_mismatch），
    每回对不上都是同会话又有一条收了，机器忙、十几条一齐收尾时三回追不上。假客户端头 8 回回 409（带新的条数，从 9 条一回少一条）、第 9 回回 0：帮手应当确认成功，共 9 回；
    头 14 回回 409（多于回数）、第 15 回回 0：回数用完之后打回的还带着新的确认单，再追到时限（CONFIRM_BUDGET_SECONDS），确认成功共 15 回；
    一直回 409 而时限给 0 的：在 CONFIRM_RETRIES 加头一回条数那么多回之后判红（有界）。弄坏开关 confirm-retries-fixed 下只重试 CONFIRM_RETRIES 回，这一格红。"""
    def needed(tasks):
        return subprocess.CompletedProcess([], 0, f"{lib.CONFIRM_NEEDED_MARK} task=5 items=1 tasks={tasks} timeout=1800\n", "")

    def stub(mismatches):
        calls = []

        def client(arguments, **options):
            calls.append(arguments)
            if len(calls) <= mismatches:
                return subprocess.CompletedProcess([], 3, needed(max(1, 9 - len(calls))).stdout, "mismatch")
            return subprocess.CompletedProcess([], 0, "confirmed", "")
        fake = object.__new__(World)
        fake.client, fake.server_log_lines = client, lambda: []
        return fake, calls
    problems = []
    fake, calls = stub(8)
    reply = fake.confirm_submitted(needed(9))
    if reply is None or reply.returncode != 0 or len(calls) != 9:
        problems.append(f"头 8 回对不上、第 9 回对上：应当确认成功、共 9 回，实际 {len(calls)} 回、{reply}")
    fake, calls = stub(14)
    reply = fake.confirm_submitted(needed(9))
    if reply is None or reply.returncode != 0 or len(calls) != 15:
        problems.append(f"14 回对不上（多于回数 {CONFIRM_RETRIES + 9}）、第 15 回对上：回数用完之后服务打回的还带着新的确认单，应当再追到时限、确认成功、共 15 回，实际 {len(calls)} 回、{reply}")
    fake, calls = stub(1000)
    fake.confirm_budget_seconds = 0
    try:
        fake.confirm_submitted(needed(9))
        problems.append("一直对不上的应当判红")
    except SelftestFailure:
        if len(calls) != CONFIRM_RETRIES + 9:
            problems.append(f"一直对不上：应当在 {CONFIRM_RETRIES + 9} 回之后判红（有界），实际 {len(calls)} 回")
    if problems:
        raise SelftestFailure("确认单重试随同会话条数放宽：" + "；".join(problems))
    return f"同会话条数 9 的时候头 8 回对不上、第 9 回确认成功；头 14 回对不上的追到第 15 回确认成功；一直对不上而时限给 0 的 {CONFIRM_RETRIES + 9} 回判红"


SUBMIT_TRACE_STEPS = ("进线程", "提交时拒", "认会话", "认任务", "记库", "提交时警告", "确认单", "回话")   # submit-trace-logged：一条收下的提交走过的几步，照这个先后
SUBMIT_TRACE_RECEIVED = re.compile(r"\d\d:\d\d:\d\d\.\d{3} 收到；")                                  # 那一行里收到的时刻（时、分、秒与毫秒）


def cell_submit_trace_logged(world):
    """提交那一路的进出日志（task_server.py 的 SubmitTrace、POST /tasks 的 submit 与 submit_in_thread、Scheduler.submit 的 trace）：真服务收下一条提交，服务日志里恰好一行
    以 SUBMIT_TRACE_TEXT 起头、写「任务 <号>，收下」、收到的时刻，与 SUBMIT_TRACE_STEPS 那几步照先后各带距收到多少秒；另在这一格的进程里造一条 SubmitTrace、
    演上一步之后过了 SUBMIT_TRACE_SLOW_SECONDS 以上：那一步当场另打一行「…这一步用了 …秒…回话还没出去」，回话时照样打那一行。
    弄坏开关 submit-trace-silent 下一行都不打，这一格红。"""
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "submit-trace"])
    done = world.finished(task_id)
    lines = []
    for number in range(1, world.server_logs + 1):
        with open(os.path.join(world.root, f"server-{number}.log"), encoding="utf-8", errors="replace") as handle:
            lines += [line for line in handle.read().splitlines() if SERVER.SUBMIT_TRACE_TEXT in line]
    mine = [line for line in lines if f"任务 {task_id}，收下" in line]
    problems = []
    if done["task"].get("outcome") != "succeeded":
        problems.append(f"那一条应当跑成，实际 {describe(done)}")
    if len(mine) != 1:
        problems.append(f"服务日志里应当恰好一行「{SERVER.SUBMIT_TRACE_TEXT}（任务 {task_id}，收下）」，实际 {len(mine)} 行：{lines[-5:]}")
    else:
        positions = [mine[0].find(f"{step} +") for step in SUBMIT_TRACE_STEPS]
        if not SUBMIT_TRACE_RECEIVED.search(mine[0]) or -1 in positions or positions != sorted(positions):
            problems.append(f"那一行应当写收到的时刻、照先后写 {'、'.join(SUBMIT_TRACE_STEPS)} 各距收到多少秒，实际「{mine[0]}」")
    printed = io.StringIO()
    with contextlib.redirect_stderr(printed):
        trace = SERVER.SubmitTrace(dict(os.environ))
        trace.previous -= SERVER.SUBMIT_TRACE_SLOW_SECONDS + 1   # 演上一步之后过了这么久
        trace.step("提交时拒")
        trace.finish("收下")
    shown = printed.getvalue().splitlines()
    if len(shown) != 2 or "提交时拒这一步用了" not in shown[0] or "回话还没出去" not in shown[0] or SERVER.SUBMIT_TRACE_TEXT not in shown[1] or "回话 +" not in shown[1]:
        problems.append(f"一步用了 {SERVER.SUBMIT_TRACE_SLOW_SECONDS + 1:g} 秒以上：应当当场另打一行写那一步慢、回话时再打进出那一行，实际 {shown}")
    if problems:
        raise SelftestFailure("提交那一路的进出日志：" + "；".join(problems), world.server_log_lines())
    return f"收下的提交在服务日志里恰好一行：收到的时刻与 {'、'.join(SUBMIT_TRACE_STEPS)} 各距收到多少秒；一步慢的当场另打一行"


def cell_store_write_rolls_back_on_error(world):
    """库的写事务出事先回滚（task_server.py 的 Store.write_transaction；health_write、delete_row 这几处都经它）：meta 表上一个触发器让写 health 那一行 RAISE(ABORT)
    （语句回滚、事务还开着，同 SQLITE_BUSY、IO 错那几种），operations 表上一个让删行 RAISE(ABORT)：health_write 交写不进、delete_row 抛出来，
    之后这条连接不在事务里，另一条连接 BEGIN IMMEDIATE 0.5 秒内拿得到写锁。弄坏开关 store-transaction-left-open 下出事不回滚：连接留在事务里、别的连接拿不到写锁，这一格红。"""
    path = os.path.join(world.root, "rollback.sqlite3")
    store = SERVER.Store(path, bytes(32), dict(os.environ))
    row = store.insert("operations", {"kind": "selftest"})
    with contextlib.closing(sqlite3.connect(path, timeout=30, isolation_level=None)) as connection:
        connection.execute("CREATE TRIGGER selftest_health_fails BEFORE UPDATE ON meta WHEN NEW.name = 'health' BEGIN SELECT RAISE(ABORT, 'selftest health write fails'); END")
        connection.execute("CREATE TRIGGER selftest_delete_fails BEFORE DELETE ON operations BEGIN SELECT RAISE(ABORT, 'selftest delete fails'); END")
    problems = []
    for what, action in (("health_write", lambda: store.health_write()), ("delete_row", lambda: store.delete_row("operations", row["id"]))):
        raised = ""
        try:
            outcome = action()
        except sqlite3.Error as error:
            outcome, raised = None, f"{type(error).__name__}: {error}"
        open_after = store.connection().in_transaction
        locked = ""
        with contextlib.closing(sqlite3.connect(path, timeout=0.5, isolation_level=None)) as other:
            try:
                other.execute("BEGIN IMMEDIATE")
                other.execute("ROLLBACK")
            except sqlite3.OperationalError as error:
                locked = str(error)
        failed = (outcome is not None and outcome[0] is False) if what == "health_write" else bool(raised)
        if not failed or open_after or locked:
            problems.append(f"{what} 撞上 RAISE(ABORT)：应当{'交写不进' if what == 'health_write' else '抛出来'}、之后连接不在事务里、别的连接拿得到写锁，"
                            f"实际交 {outcome}{('、抛 ' + raised) if raised else ''}、连接{'还在事务里' if open_after else '不在事务里'}、别的连接{'拿不到写锁（' + locked + '）' if locked else '拿得到写锁'}")
    if problems:
        raise SelftestFailure("写事务出事先回滚：" + "；".join(problems))
    return "health_write 与 delete_row 撞上 RAISE(ABORT) 之后连接不在事务里，别的连接照样拿得到写锁"


COMMAND_WORD_SAMPLES = (   # command-word-index-skips-option-values：(argv, 命令词的位置)
    (["env", "-C", "/tmp", "cargo", "build"], 3), (["env", "A=1", "-u", "B", "cargo"], 4), (["nice", "-n10", "cargo", "build"], 2),
    (["nice", "--adjustment", "10", "cargo"], 3), (["nice", "-n", "5", "cargo"], 3), (["timeout", "-k", "5", "60", "cargo"], 4),
    (["python3", "-W", "ignore", "x.py"], 3), (["bash", "-o", "pipefail", "x.sh"], 3),
)


def cell_command_word_index_skips_option_values(world):
    """剥外层包装时连同选项值跳过（task_server_lib.py 的 command_word_index、whole_package_cargo_test、CARGO_VALUE_OPTIONS）：COMMAND_WORD_SAMPLES 每条的命令词位置照表；
    cargo test -p 包 --exclude 别的 不当点名了测试（--exclude 的值不是测试名）；-p包 连写的照旧不吞下一个词（后面那个是测试名）。
    弄坏开关 command-wrapper-values-unskipped 下停在选项值上，cargo-exclude-value-unskipped 下 --exclude 的值当测试名，这一格红。"""
    wrong = [(argv, wanted, lib.command_word_index(argv)) for argv, wanted in COMMAND_WORD_SAMPLES if lib.command_word_index(argv) != wanted]
    problems = [f"{' '.join(argv)}：命令词应当在第 {wanted} 个位置，实际 {found}" for argv, wanted, found in wrong]
    if not lib.whole_package_cargo_test(["-p", "singlefs-core", "--exclude", "singlefs-verification"]):
        problems.append("cargo test -p singlefs-core --exclude singlefs-verification 应当判整包（--exclude 的值不是测试名），实际当点名了测试")
    if lib.whole_package_cargo_test(["-psinglefs-core", "some_test"]):
        problems.append("cargo test -psinglefs-core some_test 应当判点名了测试，实际判整包")
    if problems:
        raise SelftestFailure("剥包装的选项值：" + "；".join(problems))
    return f"{len(COMMAND_WORD_SAMPLES)} 种带值选项的写法命令词位置都对；--exclude 的值不当测试名"


STAGED_PEAK_BYTES = 777 * MEBIBYTE   # attempt-row-not-overwritten：演对账在这一会儿记下的那一次的峰值


def cell_attempt_row_not_overwritten(world):
    """HTTP 线程手里是先读的那份 running 的尝试，这一会儿对账把那一次记成 ended（结局 cancelled、峰值记下）：拿那份 running 的旧字典直接 Store.update
    抛 StoreRefused("ended")，库里那一次照旧 ended、结局与峰值都在，按 state IN ('starting', 'running') 数的记账不再算它。只用 task_server.py 的 Store，不起服务。
    弄坏开关 attempt-row-overwritten 下 Store.update 不守、整行写回 running，这一格红。"""
    store = SERVER.Store(os.path.join(world.root, "attempt-row.sqlite3"), bytes(32), dict(os.environ))
    store.insert("attempts", {"task_id": 1, "number": 1, "host": SERVER.LOCAL, "state": "running", "session": SESSION_A, "command_key": "selftest-attempt-row",
                              "memory_cap": GIBIBYTE})
    stale = store.attempts_of(1)[0][0]
    store.update("attempts", dict(stale, state="ended", outcome="cancelled", ended_at=round(time.time(), 3), seconds=1.0, memory_peak=STAGED_PEAK_BYTES))
    problems = []
    try:
        store.update("attempts", dict(stale))
        problems.append(f"拿那份 running 的旧字典 Store.update 应当被拒（库里已 ended），实际写进去了：库里此刻 {store.attempts_of(1)[0][0].get('state')}")
    except SERVER.StoreRefused as refused:
        if refused.kind != "ended":
            problems.append(f"Store.update 拒的种类应当是 ended，实际 {refused.kind}（{refused}）")
    found = store.attempts_of(1)[0][0]
    seen = {key: found.get(key) for key in ("state", "outcome", "memory_peak")}
    wanted = {"state": "ended", "outcome": "cancelled", "memory_peak": STAGED_PEAK_BYTES}
    if seen != wanted:
        problems.append(f"库里那一次应当是 {wanted}，实际 {seen}")
    active = [record.get("id") for record, _ in store.select("attempts", "state IN ('starting', 'running')")]
    if active:
        problems.append(f"收尾之后按在跑数的记账一条都不算，实际算在跑的尝试 {active}")
    if problems:
        raise SelftestFailure("尝试表不整行写回：" + "；".join(problems))
    return "对账记了 ended 之后，拿先读的 running 旧字典整行写回被拒，结局与峰值都在、不算在跑"
