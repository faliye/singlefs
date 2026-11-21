"""task_server_lib.py 拆出的一份：经 unix socket 调服务、尝试的在不在与进展读数、结局说明与回话的行、异常表、进程表、会话登记（总说明在 task_server_lib.py 文件头）。"""
import os
import base64
import contextlib
import http.client
import inspect
import json
import math
import re
import shlex
import signal
import socket
import time
from dataclasses import dataclass
from task_server_lib_constants import (
    ANOMALY_GENERIC_ADVICE, ANOMALY_ROW, ANOMALY_TABLE_HEAD, ANOMALY_TABLE_LOADED, ANOMALY_TABLE_TEXT, ANOMALY_TASK_PLACEHOLDER,
    ATTEMPT_PRESENCE_PROBE_MARK, ATTEMPT_PROGRESS_PROBE_MARK, ATTENTION_MARK, ATTENTION_TYPE, ATTENTION_WHY_LIMIT, BOUNCE_NEW_TASK_REMINDER,
    CANCEL_ORIGIN_TEXTS, CAP_GROWTH_FACTOR, CONFIRM_NEEDED_LINE, CONFIRM_NEEDED_MARK, CRASH_CASE_ITEM_PREFIX, CRASH_DOMAIN_ARGUMENT,
    CRASH_SCOPE_ENTRIES, EXIT_TYPES, EXTERNAL_KILL_LINE, EXTERNAL_KILL_OUTCOME, GIBIBYTE, HOST_NAMES, INSTALL_COMMAND, LIBTEST_VALUE_OPTIONS,
    LOOP_LEAK_BOUNCE_CAP_HITS, LOOP_LEAK_CHECK_HEAD, LOOP_LEAK_KINDS, LOOP_LEAK_OUTCOMES, MEBIBYTE, MEMORY_CGROUP_KILL_TEXT, NAMED_SCOPE_OPTIONS,
    OUTCOME_TYPES, PEER_BUILD_KEPT_MARK, PEER_COMMAND_END_LINE, PRESENCE_ALIVE, PRESENCE_GONE, PRESENCE_UNVERIFIED, REASON_ELEMENTS,
    REASON_LOG_NOISE, REASON_MARK, REPOSITORY_ROOT, RESULT_MARK, SCOPE_INSIDE_SCRIPT_TEXT, SESSIONS_DIRECTORY_OVERRIDE, SESSION_REGISTRY_LOADED,
    SESSION_REGISTRY_SCRIPT, STALL_AT_CAP_FRACTION, START_COMMAND, SUSPECT_MARK, UNIT_NAME, WHOLE_SCOPE_TEXT,
)
from task_server_lib_configuration import Unavailable, broken, command_word_index, key_words, load_module, selftest_override


# ── 经 unix socket 调服务 ──
class ServerUnreachable(Exception):
    pass


class UnixSocketConnection(http.client.HTTPConnection):
    def __init__(self, socket_path, timeout):
        super().__init__("task-server", timeout=timeout)
        self.socket_path = socket_path

    def connect(self):
        connection = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        connection.settimeout(self.timeout)
        connection.connect(self.socket_path)
        self.sock = connection


def call_server(socket_path, method, path, body=None, timeout=30.0):
    """→ (HTTP 状态码, 解开的 JSON)；连不上、读不完抛 ServerUnreachable。"""
    connection = UnixSocketConnection(socket_path, timeout)
    try:
        payload = None if body is None else json.dumps(body, ensure_ascii=False).encode("utf-8")
        connection.request(method, path, body=payload, headers={"Content-Type": "application/json"} if payload is not None else {})
        response = connection.getresponse()
        data = response.read()
        status = response.status
    except (OSError, http.client.HTTPException) as error:
        raise ServerUnreachable(f"{type(error).__name__}: {error}") from None
    finally:
        connection.close()
    try:
        decoded = json.loads(data.decode("utf-8")) if data else {}
    except ValueError:
        decoded = {"detail": data.decode("utf-8", "replace")[:500]}
    return status, decoded


def unreachable_error(configuration, reason):
    return Unavailable(f"task-server 不通（{configuration.socket}：{reason}）",
                       f"看服务在不在：systemctl --user status {UNIT_NAME}；没起就 {START_COMMAND}；没装就 {INSTALL_COMMAND}；"
                       "不退回直接执行命令，服务好了再提交")


def require_server(configuration, timeout=10.0):
    """服务通不通：/health 回 ok 交它的 JSON，不通抛 Unavailable。"""
    try:
        status, health = call_server(configuration.socket, "GET", "/health", timeout=timeout)
    except ServerUnreachable as error:
        raise unreachable_error(configuration, str(error)) from None
    if status != 200 or not health.get("ok"):
        raise Unavailable(f"task-server 回话了但不健康（HTTP {status}：{health.get('problem') or health}）",
                          f"看服务的日志：journalctl --user -u {UNIT_NAME} -n 100；修好之后 python3 task-server/control check 回读到退 0")
    return health


# ── 错误类型与重试（纯函数）──
def classify_attempt(exit_code, host, log_text, cancelled, environment=None):
    """一次尝试的错误类型（文件头「错误类型」）。exit_code 是 None：单元不在了又没留结果。"""
    if cancelled:
        return "cancelled"
    if exit_code is None:
        return "launcher-lost"
    if exit_code == 255 and host == "peer" and not broken("peer-exit-unsplit", environment):
        return "peer-run-failed"
    if exit_code == 75 and broken("prune-server-exit-unsplit", environment):
        return "command-failed"   # 弄坏开关：改前的样子，75 不单列
    if exit_code in EXIT_TYPES:
        return EXIT_TYPES[exit_code]
    if broken("unlisted-exit-swallowed", environment):
        return "succeeded"
    return "command-failed"


def memory_cgroup_kill_line(line, environment=None):
    """这一行是内核在某个内存组到了它自己的上限时杀进程的记录（带 MEMORY_CGROUP_KILL_TEXT）：不算被外部信号杀掉。弄坏开关 memory-cgroup-kill-counted 下一律交 False。"""
    return MEMORY_CGROUP_KILL_TEXT in line and not broken("memory-cgroup-kill-counted", environment)


def external_kill_pid(line, environment=None):
    """journalctl 里一行是 earlyoom（「sending SIGTERM to process <号> …」）或整机内存不够时内核 OOM（「Killed process <号> (…」）杀进程的记录，交被杀的进程号；
    不是这两种行、或是内存组到了它自己上限的那一种（memory_cgroup_kill_line）交 None。"""
    if memory_cgroup_kill_line(line, environment):
        return None
    found = EXTERNAL_KILL_LINE.search(line)
    return int(found.group(1) or found.group(2)) if found else None


# ── 读不到单元状态时不经 systemd 核进程（task_server.py 文件头 ①「读不到单元状态」）──
def attempt_presence_readings(request):
    """不经 systemd 核一次尝试在这一台上还有没有进程：只读 /proc 与 cgroupfs，不起 systemctl、不发信号（第二台经 attempt_presence_probe_text 把这一段送过去跑，两台同一段）。
    request：process_ids（记下的进程号）、process_id_files（存着一个进程号的文件：第二台 multi-host-run.sh 记的 runs/<标签>.pid；文件不在算没起过那一层）、
    cgroup_directories（记下的 cgroup 目录：attempt_runner 写的 scope 文件里那一个）、cgroup_names（照名字在这一台用户级管理器的 cgroup 树里找的目录：尝试的单元）、
    cgroup_name_files（存着一个 cgroup 名字的文件，读出的名字接上 cgroup_name_suffix 照 cgroup_names 找：第二台 multi-host-run.sh 记的 runs/<标签>.scope）、
    label_entry（「<尝试标签变量>=<标签>」：树里名字以 label_scope_prefix 起头、以 .scope 收尾的 cgroup，里面有进程的环境带这一项的也算这一次的）、
    tree_depth（在树里往下找几层）、cgroup_root、proc_root（不给是 /sys/fs/cgroup、/proc）、absent_cgroup_unconfirmed（只给弄坏开关：cgroup 目录不在不算空、算读不了）。
    用户级管理器的 cgroup 树是 <cgroup_root>/user.slice/user-<uid>.slice/user@<uid>.service（uid 是起这一段的那个用户的）。
    交回 {"processes": {进程号: 在不在（僵尸算不在；进程表列不出是 None）}, "cgroups": {目录或名字: [进程号…]（目录不在而往上一层列得出算空）| None（读不了）},
    "files": {文件: 读到的内容（不在是空串）}, "problems": [核不动的那几句]}。"""
    import os as os_module

    proc_root = request.get("proc_root") or "/proc"
    cgroup_root = (request.get("cgroup_root") or "/sys/fs/cgroup").rstrip("/")
    absent_unconfirmed = bool(request.get("absent_cgroup_unconfirmed"))
    label_entry = (request.get("label_entry") or "").encode("utf-8")
    prefix = request.get("label_scope_prefix") or ""
    readings = {"processes": {}, "cgroups": {}, "files": {}, "problems": []}

    def read_small_file(path):
        """→ 去掉首尾空白的内容；文件不在而所在目录列得出交空串（没写过），读不了交 None、记一句。"""
        try:
            with open(path, encoding="utf-8", errors="replace") as handle:
                return handle.read().strip()
        except FileNotFoundError:
            try:
                os_module.listdir(os_module.path.dirname(path) or ".")
            except OSError as error:
                readings["problems"].append(f"{path} 不在，所在的目录也列不出（{type(error).__name__}）")
                return None
            return ""
        except OSError as error:
            readings["problems"].append(f"{path} 读不了（{type(error).__name__}）")
            return None

    def members(directory):
        """cgroup 目录此刻的进程号；目录不在而往上第一层在的目录列得出交 []（cgroup 已收掉），读不了交 None、记一句。"""
        try:
            with open(os_module.path.join(directory, "cgroup.procs"), encoding="utf-8") as handle:
                return handle.read().split()
        except FileNotFoundError:
            pass
        except OSError as error:
            readings["problems"].append(f"{directory}/cgroup.procs 读不了（{type(error).__name__}）")
            return None
        if os_module.path.isdir(directory):
            readings["problems"].append(f"{directory} 在而没有 cgroup.procs")
            return None
        if absent_unconfirmed:
            readings["problems"].append(f"{directory} 不在（弄坏开关 unit-unreadable-absent-cgroup-unconfirmed：不算空）")
            return None
        ancestor = os_module.path.dirname(directory.rstrip("/"))
        while ancestor == cgroup_root or ancestor.startswith(cgroup_root + "/"):
            if os_module.path.isdir(ancestor):
                try:
                    os_module.listdir(ancestor)
                except OSError as error:
                    readings["problems"].append(f"{directory} 不在，上一层 {ancestor} 列不出（{type(error).__name__}）")
                    return None
                return []
            ancestor = os_module.path.dirname(ancestor)
        readings["problems"].append(f"{directory} 不在，往上到 cgroup 根 {cgroup_root} 都不在")
        return None

    def labelled(processes):
        """→ (有一个进程的环境带 label_entry, 有读不了环境的)；读的时候刚退出的跳过。"""
        unreadable = False
        for process in processes:
            try:
                with open(os_module.path.join(proc_root, process, "environ"), "rb") as handle:
                    if label_entry in handle.read().split(b"\0"):
                        return True, False
            except FileNotFoundError:
                continue
            except OSError:
                unreadable = True
        return False, unreadable

    try:
        os_module.listdir(proc_root)
        proc_readable = True
    except OSError as error:
        proc_readable = False
        readings["problems"].append(f"进程表 {proc_root} 列不出（{type(error).__name__}）")
    process_ids = [str(process) for process in request.get("process_ids") or []]
    for path in request.get("process_id_files") or []:
        text = read_small_file(path)
        if text is None:
            continue
        readings["files"][path] = text
        if text.isdigit():
            process_ids.append(text)
        elif text:
            readings["problems"].append(f"{path} 里不是进程号（{text[:40]}）")
    for process in process_ids:
        if not proc_readable:
            readings["processes"][process] = None
            continue
        try:
            with open(os_module.path.join(proc_root, process, "stat"), encoding="utf-8", errors="replace") as handle:
                readings["processes"][process] = handle.read().rpartition(")")[2].split()[:1] != ["Z"]
        except FileNotFoundError:
            readings["processes"][process] = False
        except OSError:
            readings["processes"][process] = True   # 目录在、stat 读不了：当在
    names = list(request.get("cgroup_names") or [])
    for path in request.get("cgroup_name_files") or []:
        text = read_small_file(path)
        if text is None:
            continue
        readings["files"][path] = text
        if text:
            names.append(text + (request.get("cgroup_name_suffix") or ""))
    directories = list(request.get("cgroup_directories") or [])
    if names or (label_entry and prefix):
        uid = os_module.getuid()
        manager = f"{cgroup_root}/user.slice/user-{uid}.slice/user@{uid}.service"
        found, candidates, errors = {}, [], []

        def walk_error(error):
            if not isinstance(error, FileNotFoundError):   # 走的时候刚收掉的目录不算读不了
                errors.append(f"{getattr(error, 'filename', '') or manager} 列不出（{type(error).__name__}）")

        if not os_module.path.isdir(manager):
            errors.append(f"用户级管理器的 cgroup 目录 {manager} 不在")
        else:
            for current, children, _ in os_module.walk(manager, onerror=walk_error):
                for name in children:
                    if name in names:
                        found.setdefault(name, os_module.path.join(current, name))
                    elif label_entry and prefix and name.startswith(prefix) and name.endswith(".scope"):
                        candidates.append(os_module.path.join(current, name))
                if current[len(manager):].count("/") + 1 >= int(request.get("tree_depth") or 1):
                    children[:] = []
        readings["problems"] += errors
        for name in names:
            if errors:
                readings["cgroups"][name] = None
            elif name in found:
                directories.append(found[name])
            elif absent_unconfirmed:
                readings["problems"].append(f"cgroup {name} 在 {manager} 下找不到（弄坏开关 unit-unreadable-absent-cgroup-unconfirmed：不算空）")
                readings["cgroups"][name] = None
            else:
                readings["cgroups"][name] = []   # 树走全了也没有：这个单元或 scope 的 cgroup 已收掉
        for path in candidates:
            processes = members(path)
            if processes is None:
                readings["cgroups"][path] = None
                continue
            ours, unreadable = labelled(processes) if processes else (False, False)
            if ours:
                readings["cgroups"][path] = processes
            elif unreadable:
                readings["problems"].append(f"{path} 里有进程的环境读不了，认不出是不是这一次的")
                readings["cgroups"][path] = None
    for directory in directories:
        readings["cgroups"][directory] = members(directory)
    return readings


ATTEMPT_PRESENCE_READINGS_SOURCE = inspect.getsource(attempt_presence_readings)   # 载入时取一次：运行时再按行号读磁盘上的源码，文件被改过就取到别的函数


def attempt_presence_probe_text(request):
    """在第二台上起 attempt_presence_readings 的那一条 bash 命令（经 peer-host-lib.sh 的 peer_host_run 起）：
    源码与请求各 base64 一份，读数打成一行 ATTEMPT_PRESENCE_PROBE_MARK 加 JSON。"""
    program = (ATTEMPT_PRESENCE_READINGS_SOURCE
               + "\nimport base64 as probe_base64, json as probe_json, sys as probe_sys\n"
               + "probe_request = probe_json.loads(probe_base64.b64decode(probe_sys.argv[1]).decode('utf-8'))\n"
               + f"print({ATTEMPT_PRESENCE_PROBE_MARK!r} + probe_json.dumps(attempt_presence_readings(probe_request)))\n")
    encoded = base64.b64encode(program.encode("utf-8")).decode("ascii")
    argument = base64.b64encode(json.dumps(request).encode("utf-8")).decode("ascii")
    return f'python3 -B -c "$(printf %s {encoded} | base64 -d)" {argument}'


def attempt_presence_probe_reply(stdout):
    """第二台那一趟的标准输出 → 读数；没有那一行或解不开交 None。"""
    line = next((line for line in reversed((stdout or "").splitlines()) if line.startswith(ATTEMPT_PRESENCE_PROBE_MARK)), None)
    if line is None:
        return None
    try:
        readings = json.loads(line[len(ATTEMPT_PRESENCE_PROBE_MARK):])
    except ValueError:
        return None
    return readings if isinstance(readings, dict) else None


# ── 日志不涨时的进展读数（task_server.py 文件头 ①「要处置的提醒」的 quiet-log-busy）──
def attempt_progress_readings(request):
    """一次尝试在这一台上此刻的进展读数：这一趟进程树里各进程打开写着的普通文件与大小。只读 /proc，不发信号
    （第二台经 attempt_progress_probe_text 把这一段送过去跑，两台同一段）。
    request：root_process_ids（进程树的根）、root_process_id_files（存着一个根进程号的文件：第二台 multi-host-run.sh 记的 runs/<标签>.pid）、
    excluded_files（不算的文件：尝试的顶层日志，它的大小里有起法那一层每轮打的进度行，服务另照 log_content 判）、proc_root（不给是 /proc）、
    file_limit、
    write_mode_unchecked（只给弄坏开关 open-file-write-mode-unchecked：只读打开的也算）。
    进程树照 <proc_root>/<号>/stat 的父进程号从根往下；写着的文件：<号>/fd/<fd> 指到的绝对路径（不是已删的），<号>/fdinfo/<fd> 的 flags 里访问方式是只写或读写，
    指到的是普通文件。→ {"files": {路径: 字节数}, "processes": 进程树里几个进程, "problems": [读不了的那几句]}。"""
    import os as os_module
    import stat as stat_module

    proc_root = request.get("proc_root") or "/proc"
    limit = int(request.get("file_limit") or 256)
    readings = {"files": {}, "processes": 0, "problems": []}
    excluded = set()
    for path in request.get("excluded_files") or []:
        excluded.add(os_module.path.realpath(path))
    roots = [int(process) for process in request.get("root_process_ids") or [] if str(process).isdigit()]
    for path in request.get("root_process_id_files") or []:
        try:
            with open(path, encoding="utf-8") as handle:
                text = handle.read().strip()
        except OSError as error:
            readings["problems"].append(f"{path} 读不了（{type(error).__name__}）")
            continue
        if text.isdigit():
            roots.append(int(text))
        elif text:
            readings["problems"].append(f"{path} 里不是进程号（{text[:40]}）")
    children = {}
    try:
        names = os_module.listdir(proc_root)
    except OSError as error:
        readings["problems"].append(f"进程表 {proc_root} 列不出（{type(error).__name__}）")
        names = []
    for name in names:
        if not name.isdigit():
            continue
        try:
            with open(os_module.path.join(proc_root, name, "stat"), encoding="utf-8", errors="replace") as handle:
                parent = int(handle.read().rsplit(")", 1)[1].split()[1])
        except (OSError, IndexError, ValueError):
            continue
        children.setdefault(parent, []).append(int(name))
    family, frontier = set(), [root for root in roots if os_module.path.isdir(os_module.path.join(proc_root, str(root)))]
    family.update(frontier)
    while frontier:
        for child in children.get(frontier.pop(), []):
            if child not in family:
                family.add(child)
                frontier.append(child)
    readings["processes"] = len(family)
    for process in sorted(family):
        if len(readings["files"]) >= limit:
            readings["problems"].append(f"写着的文件记满 {limit} 份，余下的进程不再读")
            break
        directory = os_module.path.join(proc_root, str(process), "fd")
        try:
            descriptors = os_module.listdir(directory)
        except OSError:
            continue   # 刚退出、或读不了别人的进程：这一个不算
        for descriptor in descriptors:
            try:
                target = os_module.readlink(os_module.path.join(directory, descriptor))
            except OSError:
                continue
            if not target.startswith("/") or target.endswith(" (deleted)"):
                continue
            if not request.get("write_mode_unchecked"):
                try:
                    with open(os_module.path.join(proc_root, str(process), "fdinfo", descriptor), encoding="utf-8") as handle:
                        flags = next((int(line.split()[1], 8) for line in handle if line.startswith("flags:")), None)
                except (OSError, ValueError, IndexError):
                    continue
                if flags is None or flags & 3 not in (1, 2):   # O_ACCMODE：1 只写、2 读写
                    continue
            real = os_module.path.realpath(target)
            if real in excluded or real in readings["files"]:
                continue
            try:
                found = os_module.stat(real)
            except OSError:
                continue
            if not stat_module.S_ISREG(found.st_mode):
                continue
            if len(readings["files"]) >= limit:
                break
            readings["files"][real] = found.st_size
    return readings


ATTEMPT_PROGRESS_READINGS_SOURCE = inspect.getsource(attempt_progress_readings)   # 载入时取一次（同 ATTEMPT_PRESENCE_READINGS_SOURCE）


def attempt_progress_probe_text(request):
    """在第二台上起 attempt_progress_readings 的那一条 bash 命令（经 peer-host-lib.sh 的 peer_host_run 起，同 attempt_presence_probe_text 的送法）：
    源码与请求各 base64 一份，读数打成一行 ATTEMPT_PROGRESS_PROBE_MARK 加 JSON。"""
    program = (ATTEMPT_PROGRESS_READINGS_SOURCE
               + "\nimport base64 as probe_base64, json as probe_json, sys as probe_sys\n"
               + "probe_request = probe_json.loads(probe_base64.b64decode(probe_sys.argv[1]).decode('utf-8'))\n"
               + f"print({ATTEMPT_PROGRESS_PROBE_MARK!r} + probe_json.dumps(attempt_progress_readings(probe_request)))\n")
    encoded = base64.b64encode(program.encode("utf-8")).decode("ascii")
    argument = base64.b64encode(json.dumps(request).encode("utf-8")).decode("ascii")
    return f'python3 -B -c "$(printf %s {encoded} | base64 -d)" {argument}'


def attempt_progress_probe_reply(stdout):
    """第二台那一趟的标准输出 → 读数；没有那一行或解不开交 None。"""
    line = next((line for line in reversed((stdout or "").splitlines()) if line.startswith(ATTEMPT_PROGRESS_PROBE_MARK)), None)
    if line is None:
        return None
    try:
        readings = json.loads(line[len(ATTEMPT_PROGRESS_PROBE_MARK):])
    except ValueError:
        return None
    return readings if isinstance(readings, dict) else None


def attempt_presence_verdict(parts):
    """一次尝试在各台的读数合起来判（task_server.py 文件头 ①「读不到单元状态」）。parts：[(台（HOST_NAMES 的键）, attempt_presence_readings 交的读数或 None, 读不成的那一句)]。
    → (PRESENCE_ALIVE：有一个记下的进程号在、或有一个 cgroup 里有进程；PRESENCE_GONE：都读得到、cgroup 都空或已收掉、记下的进程号都不在；
    PRESENCE_UNVERIFIED：没有在的、又有一样读不成或核不动, 说明：逐台写核到的进程号与 cgroup)。"""
    alive, unverified, pieces = False, False, []
    for host, readings, problem in parts:
        name = HOST_NAMES.get(host, host)
        if readings is None:
            unverified = True
            pieces.append(f"{name}：{problem or '读不成'}")
            continue
        facts = []
        for process, present in sorted((readings.get("processes") or {}).items()):
            if present is None:
                unverified = True
                facts.append(f"进程号 {process} 读不了")
            elif present:
                alive = True
                facts.append(f"进程号 {process} 在（/proc/{process}）")
            else:
                facts.append(f"进程号 {process} 不在")
        for group, processes in sorted((readings.get("cgroups") or {}).items()):
            if processes is None:
                unverified = True
                facts.append(f"cgroup {group} 读不了")
            elif processes:
                alive = True
                facts.append(f"cgroup {group} 里 {len(processes)} 个进程（{' '.join(processes[:5])}{' …' if len(processes) > 5 else ''}）")
            else:
                facts.append(f"cgroup {group} 空或已收掉")
        if readings.get("problems"):
            unverified = True
            facts.append("核不动：" + "；".join(str(line) for line in readings["problems"][:3]))
        if readings.get("note"):
            facts.append(str(readings["note"]))
        pieces.append(f"{name}：{'，'.join(facts) or '没有要核的'}")
    verdict = PRESENCE_ALIVE if alive else PRESENCE_UNVERIFIED if unverified else PRESENCE_GONE
    return verdict, "；".join(pieces)


def peer_command_exit(log_text, environment=None):
    """第二台那条命令自己的退出码：日志里最后一行「MULTI_HOST_RUN_STEP step=peer-command event=end … exit=<码>」（multi-host-run.sh 在第二台那条命令
    等到结局时打）的码；没有这一行交 None（multi-host-run.sh 那一层没跑到命令结束）。弄坏开关 peer-step-exit-ignored 下一律 None（改前的样子）。"""
    if broken("peer-step-exit-ignored", environment):
        return None
    found = None
    for line in (log_text or "").splitlines():
        matched = PEER_COMMAND_END_LINE.match(line.strip())
        if matched:
            found = int(matched.group(1))
    return found


def peer_marked_bytes(log_text, mark):
    """日志里最后一行「<mark> bytes=<十进制字节数>」的字节数（multi-host-run.sh 打的第二台那一层的峰值）；没有交 None。"""
    pattern = re.compile(rf"^{re.escape(mark)} bytes=([0-9]+)$")
    found = None
    for line in (log_text or "").splitlines():
        matched = pattern.match(line.strip())
        if matched:
            found = int(matched.group(1))
    return found


def peer_kept_build_label(log_text):
    """日志里最后一行「PEER_BUILD_KEPT_MARK label=<标签>」的标签（multi-host-run.sh 留下了第二台这一趟的编译目录）；没有交 None。"""
    pattern = re.compile(rf"^{re.escape(PEER_BUILD_KEPT_MARK)} label=([A-Za-z0-9][A-Za-z0-9._-]*)$")
    found = None
    for line in (log_text or "").splitlines():
        matched = pattern.match(line.strip())
        if matched:
            found = matched.group(1)
    return found


def final_exit_code(outcome, attempt_exit):
    code, _ = OUTCOME_TYPES[outcome]
    if code is not None:
        return code
    return attempt_exit if isinstance(attempt_exit, int) and 0 < attempt_exit < 256 else 1


def grown_cap(cap_bytes):
    return math.ceil(cap_bytes * CAP_GROWTH_FACTOR / MEBIBYTE) * MEBIBYTE


def mebibytes_text(byte_count):
    return f"{math.ceil(byte_count / MEBIBYTE)}M"


def maximum_next_step(host, cap_text, peak_text, environment=None):
    """memory-cap-at-maximum 的出路（文件头「撞顶到两台最大」）：撞了哪台、到了多少、峰值多少，后面接异常表里这一种的修改意见（核实范围、分片与同时起的个数）。
    wait 的末行、打回的理由与发给会话的正文都用这一句。"""
    where = HOST_NAMES.get(host, host or "本机")
    return (f"撞到动态上限、打回：{where}已给到 {cap_text}（峰值 {peak_text}），再乘 1.5 就到能派的每一台的可用上限；"
            + anomaly_advice("memory-cap-at-maximum", "", environment))


def pinned_caps_text(reply):
    """钉值回话（task_server.py 的 pin_cap 交的那一份）里两台此刻的钉值：「本机 20480M、第二台 30720M」；读不到写「此刻读不到（原因）」（task status 与 control pin-cap 打）。"""
    caps = reply.get("bytes_by_host")
    if isinstance(caps, dict) and caps and all(isinstance(value, int) for value in caps.values()):
        shown = "、".join(f"{HOST_NAMES.get(host, host)} {mebibytes_text(value)}" for host, value in sorted(caps.items()))
        return f"{shown}；{reply['problem']}" if reply.get("problem") else shown   # 哪一台量不到余量：给得出的照给，量不到的写原因（task_server.py 的 pin_value）
    return f"此刻读不到（{reply.get('problem')}）"


def size_text(byte_count):
    if not isinstance(byte_count, int):
        return "没量到"
    return f"{byte_count / GIBIBYTE:.2f} GiB" if byte_count >= GIBIBYTE else f"{byte_count / MEBIBYTE:.0f} MiB"


def log_lines(log_text):
    """日志里人读的那几行（去掉空行与 REASON_LOG_NOISE 那几种进度行）。"""
    return [line.strip() for line in (log_text or "").splitlines() if line.strip() and not line.strip().startswith(REASON_LOG_NOISE)]


def log_last_line(log_text, fallback="日志里没有输出"):
    lines = log_lines(log_text)
    return lines[-1][:300] if lines else fallback


def trailing_cap_hits(attempts):
    """尝试记录里末尾连着结局是 memory-cap-hit 的那几次（文件头「打回的理由」的 loop_leak=、「异常表」repeated-cap-hits）→ [尝试…]，照第几次从小到大。
    只数已经收尾的（state 是 ended）；在跑、在起的那一次不数、也不断开；中间夹着别的结局（被挤掉、第二台没跑成这类）的从那之后数起。"""
    ended = sorted((attempt for attempt in attempts if attempt.get("state") == "ended"), key=lambda attempt: int(attempt.get("number") or 0))
    run = []
    for attempt in ended:
        run = run + [attempt] if attempt.get("outcome") == "memory-cap-hit" else []
    return run


LOOP_LEAK_READING_MISMATCH = "峰值远低于上限：多半是命令自己开跑前判装不下、自己退的 250，看日志里命令自己的那一句，不要据此判泄漏"


def loop_leak_check_text(cap_hits):
    """「检查代码有没有逻辑循环或内存泄漏」那一段（文件头「打回的理由」）：cap_hits 是 trailing_cap_hits 交的那几次（至少一次）。
    写要查的两类（LOOP_LEAK_KINDS），再写依据：各次的上限与峰值逐次列出、峰值的走势（严格变大、全部相等，或都不是）、每次是不是都顶到上限（峰值不低于上限的 STALL_AT_CAP_FRACTION）、
    最后一次内存最高那一刻的明细（task_server.py 的 finish_attempt 记的 peak_memory_note）；每一次都没顶到的另写 LOOP_LEAK_READING_MISMATCH。理由行的 loop_leak=、结局正文与 repeated-cap-hits 的提醒都用这一句。
    弄坏开关 loop-leak-equal-peaks-growing 下相等也写一次比一次大，loop-leak-reading-mismatch-silent 下不写那一句。"""
    shown = "、".join(f"第 {attempt.get('number')} 次上限 {attempt.get('memory_cap_text') or '没记'} 峰值 {size_text(attempt.get('memory_peak'))}" for attempt in cap_hits)
    peaks = [attempt.get("memory_peak") for attempt in cap_hits]
    if not all(isinstance(peak, int) and peak > 0 for peak in peaks):
        trend = "有几次峰值没量到，比不出是不是一次比一次大"
    elif all(later >= earlier if broken("loop-leak-equal-peaks-growing") else later > earlier for earlier, later in zip(peaks, peaks[1:])):
        trend = "峰值一次比一次大"
    elif all(later == earlier for earlier, later in zip(peaks, peaks[1:])):
        trend = "峰值没变"
    else:
        trend = "峰值不是一次比一次大"
    short = [str(attempt.get("number")) for attempt in cap_hits
             if not (isinstance(attempt.get("memory_peak"), int) and isinstance(attempt.get("memory_cap"), int) and attempt["memory_cap"] > 0
                                                 and attempt["memory_peak"] >= STALL_AT_CAP_FRACTION * attempt["memory_cap"])]
    topped = (f"每次都顶到上限（峰值不低于上限的 {STALL_AT_CAP_FRACTION:.0%}）" if not short
              else f"第 {'、'.join(short)} 次没顶到上限的 {STALL_AT_CAP_FRACTION:.0%}（或峰值没量到）")
    stat = cap_hits[-1].get("peak_memory_note") or "最后一次内存最高那一刻的明细没采到（命令比一轮还短，或读不了 scope 的 memory.current、memory.stat）"
    mismatch = f"；{LOOP_LEAK_READING_MISMATCH}" if len(short) == len(cap_hits) and not broken("loop-leak-reading-mismatch-silent") else ""
    return f"{LOOP_LEAK_CHECK_HEAD}连着撞了 {len(cap_hits)} 次内存上限，要查两类：{LOOP_LEAK_KINDS}。依据：{shown}；{trend}；{topped}；{stat}{mismatch}"


def reason_fields(task, attempts, log_text):
    """打回的理由（文件头「错误类型」表后的「打回的理由」）：→ [(栏, 值)]，type 在头、why 与 next 在尾，中间是 REASON_ELEMENTS 给这一种的要素。
    attempts 是结局所在的那几次尝试（分片的是没成那一步的子任务的），log_text 是最后那一次日志的尾巴。"""
    outcome = task.get("outcome") or ""
    task_id = task.get("id")
    note = (task.get("note") or "").strip()
    last = attempts[-1] if attempts else {}
    facts = {
        "exit": str(task.get("exit_code")),
        "host": HOST_NAMES.get(last.get("host") or task.get("maximum_host") or "", last.get("host") or "本机"),
        "last_line": log_last_line(log_text),
        "cap": last.get("memory_cap_text") or "没起",
        "peak": size_text(last.get("memory_peak")) if attempts else "没量到",
        "reached": task.get("maximum_cap_text") or last.get("memory_cap_text") or "?",
        "by": last.get("squeezed_by") or "同台没有别的任务在跑（slice 里回收不掉的用量把它挤掉）",
        "unit": last.get("unit") or "没起单元",
        "judged": note or "没有说明",
        "kill": last.get("external_kill_note") or "没有说明",
        "rule": note or "没有说明",
        "what": note or "任务记录",
    }
    if outcome == "cancelled":   # 谁撤的照子任务的 cancel_origin 写（CANCEL_ORIGIN_TEXTS；弄坏开关 cancel-origin-unread 下一律写提交它的会话）
        origin = "" if broken("cancel-origin-unread") else str(task.get("cancel_origin") or "")
        facts["by"] = CANCEL_ORIGIN_TEXTS.get(origin, CANCEL_ORIGIN_TEXTS[""])
    counts = task.get("outcome_counts") or {}
    whys = {
        "succeeded": "命令退 0",
        "command-failed": f"命令自己退 {facts['exit']}（不是服务判的；日志最后一行见 last_line）" + (f"；{last['signal_note']}" if last.get("signal_note") else ""),
        "memory-cap-hit": f"撞了这一次的内存上限 {facts['cap']}（量到的峰值 {facts['peak']}），撞了 {counts.get('memory-cap-hit', 1)} 次" + (f"：{note}" if note else ""),
        "memory-cap-at-maximum": ((f"{facts['host']}给到 {facts['reached']} 还撞（峰值 {facts['peak']}），再乘 1.5 就到能派的每台的总上限" + (f"：{note}" if note else ""))
                                  if attempts else (note or "撞上限而再乘 1.5 就到能派的每台的总上限")),
        "squeezed-out": f"{facts['host']}的 slice 总上限满了，这一条（上限 {facts['cap']}）被内核挤掉 {counts.get('squeezed-out', 1)} 次；同台在跑的：{facts['by']}",
        "timed-out": f"内存包装的限时到了（253，上限 {facts['cap']}）",
        "wrapper-refused": "内存包装用法错或 scope 起不来（251）",
        "run-condition-unmet": note or f"命令退 78（运行条件不满足）：{facts['last_line']}",
        "not-run": "命令报本次未跑（77）",
        "peer-run-failed": "第二台那一趟没跑成（multi-host-run.sh 退 255、日志里没有第二台那条命令的结束行：连不上、ssh 被重置或那边的准备没过），重排过也没成",
        "prune-server-unavailable": "命令自己退 75：prune-server（判定库服务）不可用；服务另起了一次 python3 prune-server/control check，不通的写了告警",
        "launcher-lost": f"尝试的单元 {facts['unit']} 不在了、又没留结果，服务对不回来",
        "cancelled": note or "提交它的会话取消、单元已停",
        "record-tampered": f"库里这条记录校验不过（{facts['what']}），一次都不起",
        "never-fits": f"哪台都等多久也放不下，或点名第二台而它用不了：{facts['judged']}",
        "unit-start-failed": f"尝试的单元起不来：{facts['judged']}",
        "resubmit-required": f"服务规则改了，还在排的这一条照现行提交规则不再收：{facts['rule']}",
        "stalled": f"卡住，服务停了它：{facts['judged']}",
        "shard-plan-refused": f"分片的计划那一步退 0 而它写的计划读不通：{facts['judged']}",
        "over-four-hours": f"到了四小时硬上限，服务打回：{facts['judged']}",
        "killed-externally": f"命令被不是服务发的信号杀掉 {counts.get(EXTERNAL_KILL_OUTCOME, 1)} 次（{facts['kill']}），重排过也没成",
    }
    why = whys.get(outcome, note or f"结局 {outcome}")
    # 出路照异常表这一种那一行的修改意见（文件头「异常表」）；memory-cap-at-maximum 前面另接撞了哪台、到了多少（maximum_next_step）
    if outcome == "succeeded":
        next_step = "无"
    elif outcome == "memory-cap-at-maximum":
        next_step = maximum_next_step(task.get("maximum_host") or last.get("host") or "", facts["reached"], facts["peak"])
    elif outcome == "launcher-lost":
        next_step = anomaly_advice(outcome, task_id).replace("<unit= 点名的单元>", facts["unit"])
    else:
        next_step = anomaly_advice(outcome, task_id)
    # 连着撞上限之后打回的另带 loop_leak=（文件头「打回的理由」）：末尾连着撞上限不少于 LOOP_LEAK_BOUNCE_CAP_HITS 次才带，排队那一刻就打回的（一次都没起）不带
    cap_hits = trailing_cap_hits(attempts) if outcome in LOOP_LEAK_OUTCOMES and not broken("loop-leak-check-omitted") else []
    loop_leak = [("loop_leak", loop_leak_check_text(cap_hits))] if len(cap_hits) >= LOOP_LEAK_BOUNCE_CAP_HITS else []
    return [("type", outcome)] + [(name, facts[name]) for name in REASON_ELEMENTS.get(outcome, ())] + loop_leak + [("why", why), ("next", next_step)]


def reason_line(task, attempts, log_text):
    """REASON_MARK 那一行（值照 shlex 引，shlex.split 拆得回）。"""
    return " ".join([REASON_MARK, f"task={task.get('id')}"] + [f"{name}={shlex.quote(str(value))}" for name, value in reason_fields(task, attempts, log_text)])


# ── 一条任务的回话（GET /tasks/<号> 与 wait 的 summary）照 wait 的写法拼成几行：客户端打它，服务发消息给会话（task_server.py 文件头 ⑥）照同一份拼 ──
LOG_REASON_BYTES = 65536   # 打回的理由取日志最后这么多字节（与 task_server.py 的 LOG_TAIL_BYTES 同一个数）
ROLE_NAMES = {"plan": "计划", "merge": "合并"}


def child_prefix(child_task):
    role = child_task.get("role")
    if role == "shard":
        return f"第 {child_task.get('shard_index')}/{child_task.get('shard_count')} 片（子任务 {child_task.get('id')}）"
    return f"{ROLE_NAMES.get(role, role)}（子任务 {child_task.get('id')}）"


def reply_attempts(reply):
    """[(前缀, 尝试)]：普通任务是它自己的；分片的父任务是各子任务的，按子任务号。"""
    if reply.get("children"):
        return [(child_prefix(child["task"]), attempt) for child in reply["children"] for attempt in child.get("attempts", [])]
    return [("", attempt) for attempt in reply.get("attempts", [])]


def outcome_attempts(reply):
    """结局所在的那几次尝试：分片的有一步没成的是那一步的子任务的，别的是全部（末行的 log= 与打回的理由取同一份）。"""
    failed = reply["task"].get("failed_step") or {}
    attempts = [attempt for _, attempt in reply_attempts(reply)]
    if failed and reply.get("children"):
        attempts = next((child.get("attempts", []) for child in reply["children"] if child["task"].get("id") == failed.get("task")), attempts)
    return attempts


def log_tail_text(path):
    """日志最后 LOG_REASON_BYTES 字节（读不了交空串）。"""
    if not path:
        return ""
    try:
        with open(path, "rb") as handle:
            handle.seek(0, os.SEEK_END)
            handle.seek(max(0, handle.tell() - LOG_REASON_BYTES))
            return handle.read().decode("utf-8", "replace")
    except OSError:
        return ""


def reply_reason_fields(reply):
    """打回的理由那几栏（reason_fields）：日志取结局那一次的最后 LOG_REASON_BYTES 字节。"""
    attempts = outcome_attempts(reply)
    return reason_fields(reply["task"], attempts, log_tail_text(attempts[-1].get("log")) if attempts else "")


def reply_reason_line(reply):
    """末行之前那一行 REASON_MARK（task 文件头 wait 那一段）。"""
    return " ".join([REASON_MARK, f"task={reply['task'].get('id')}"] + [f"{name}={shlex.quote(str(value))}" for name, value in reply_reason_fields(reply)])


def reply_result_line(reply, environment=None):
    """末行 RESULT_MARK（task 文件头 wait 那一段）。弄坏开关 maximum-line-vague 下撞顶的不写撞了哪台、到了多少、改什么。"""
    task = reply["task"]
    pairs = reply_attempts(reply)
    failed = task.get("failed_step") or {}
    log_attempts = outcome_attempts(reply)
    log = log_attempts[-1].get("log", "") if log_attempts else ""
    line = (f"{RESULT_MARK} task={task.get('id')} type={task.get('outcome')} exit={task.get('exit_code')} attempts={len(pairs)} "
            f"log={log or '无（一次都没起）'}")
    if task.get("sharded_entry"):
        line += f" shards={task.get('shard_count', 0)}"
    if failed.get("role") == "shard":
        line += f" shard={failed.get('index')}/{failed.get('count')}"
    elif failed.get("role") in ("plan", "merge"):
        line += f" step={failed['role']}"
    if task.get("outcome") == "memory-cap-at-maximum" and not broken("maximum-line-vague", environment):
        host = failed.get("maximum_host") or task.get("maximum_host", "")
        cap_text = failed.get("maximum_cap_text") or task.get("maximum_cap_text", "")
        peak_text = size_text(log_attempts[-1].get("memory_peak")) if log_attempts else "没量到"
        line += f" host={HOST_NAMES.get(host, host or '本机')} reached={cap_text or '?'} next={maximum_next_step(host, cap_text or '?', peak_text)}"
    return line


def suspect_line(task_id, suspect):
    """报疑似卡住的那一行 SUSPECT_MARK（task 文件头 wait 那一段）。"""
    return (f"{SUSPECT_MARK} task={task_id} attempt={suspect.get('attempt')} host={HOST_NAMES.get(suspect.get('host'), suspect.get('host') or '本机')} "
            f"window={suspect.get('window_seconds')} cores={suspect.get('cores')} log={suspect.get('log') or '无'} why={suspect.get('why')}")


def attention_line(task_id, item):
    """报要处置的提醒的那一行 ATTENTION_MARK（task 文件头 wait 那一段）：值照 shlex 引（同 REASON_MARK，shlex.split 拆得回）；part= 只在提醒说的是分片的子任务时写它的任务号；
    带 loop_leak 的那一种（repeated-cap-hits，文件头「错误类型」）在 basis= 之后另写 loop_leak=；时长判的那几种（项里有 reminded_after_seconds）在 kind= 之后写 reminded_after_seconds=（这一回跨过的那道提醒门槛，秒）。why= 超过 ATTENTION_WHY_LIMIT 个字符的按字符截到那么长、尾巴接「…」
    （全文在发给会话的那一条的「原因：」一行）；弄坏开关 attention-why-unclipped 下不截。"""
    part = f" part={item.get('task')}" if item.get("task") not in (None, task_id) else ""
    values = [(name, item.get(name)) for name in ("basis", "loop_leak", "why", "next") if name != "loop_leak" or item.get(name)]
    why = str(item.get("why"))
    if len(why) > ATTENTION_WHY_LIMIT and not broken("attention-why-unclipped"):
        values = [(name, why[:ATTENTION_WHY_LIMIT - 1] + "…" if name == "why" else value) for name, value in values]
    after = f" reminded_after_seconds={item['reminded_after_seconds']}" if item.get("reminded_after_seconds") is not None else ""
    return (f"{ATTENTION_MARK} task={task_id}{part} type={ATTENTION_TYPE} kind={item.get('kind')}{after} "
            + " ".join(f"{name}={shlex.quote(str(value))}" for name, value in values))


@dataclass(frozen=True)
class AnomalyRow:
    kind: str
    action: str
    criterion: str
    advice: str
    unhandled: str


def anomaly_rows(text):
    """异常表那一段（text 里从 ANOMALY_TABLE_HEAD 起头的那一行起、到下一个空行为止）逐行读成 [AnomalyRow]，次序照表；
    行首两个空格、照「｜」切出五栏的才算一行，栏的前后不留空白。没有这一段交 []。"""
    rows, inside = [], False
    for line in (text or "").splitlines():
        if not inside:
            inside = line.startswith(ANOMALY_TABLE_HEAD)
            continue
        if not line.strip():
            break
        found = ANOMALY_ROW.match(line)
        if found:
            rows.append(AnomalyRow(*(part.strip() for part in found.groups())))
    return rows


def anomaly_table():
    """{种类: AnomalyRow}：ANOMALY_TABLE_TEXT 那张表（anomaly_rows 读，读过一次就留着）。"""
    if not ANOMALY_TABLE_LOADED:
        ANOMALY_TABLE_LOADED.append({row.kind: row for row in anomaly_rows(ANOMALY_TABLE_TEXT)})
    return ANOMALY_TABLE_LOADED[0]


RETIRED_OUTCOME_KINDS = ("over-two-hours",)   # 库里的旧任务还记着、服务不再判出的结局：表里没有这一行，修改意见交通用那一句

def anomaly_advice(kind, task, environment=None):
    """异常表里这一种的修改意见（next= 那一句的头一段），{task} 换成任务号；表里没有这一种抛 KeyError（种类与表对不上是代码错，不悄悄退回一句通用的）。
    弄坏开关 attention-next-generic 下每一种都交 ANOMALY_GENERIC_ADVICE；retired-outcome-kinds-unknown 下退役的结局照表里没有抛 KeyError；弄坏开关 bounce-new-id-reminder-omitted 下抹掉「不要使用原来的任务号，用 task-server/task submit 创建新的任务号」。"""
    retired = kind in RETIRED_OUTCOME_KINDS and not broken("retired-outcome-kinds-unknown", environment)
    template = ANOMALY_GENERIC_ADVICE if broken("attention-next-generic", environment) or retired else anomaly_table()[kind].advice
    text = template.replace(ANOMALY_TASK_PLACEHOLDER, str(task))
    if broken("bounce-new-id-reminder-omitted", environment):
        text = text.replace(f"（{BOUNCE_NEW_TASK_REMINDER}）", "").replace(BOUNCE_NEW_TASK_REMINDER, "")
    return text


PROCESS_LIST_COLUMNS = "pid=,ppid=,etimes=,times=,stat=,cgroup:300=,comm="   # 任务之外的进程空转读的进程表（task_server.py 的 process_listings）
TASK_SLICE_MARK = "singlefs.slice"   # cgroup 里带这一段的是任务 slice 里的进程


def parsed_process_list(listing):
    """PROCESS_LIST_COLUMNS 那几列的输出 → [{pid, ppid, elapsed, cpu, stat, cgroup, command}…]；列不够、数不对的行不要。"""
    rows = []
    for line in listing.splitlines():
        fields = line.split(None, 6)
        if len(fields) == 7 and all(field.isdigit() for field in fields[:4]):
            rows.append({"pid": int(fields[0]), "ppid": int(fields[1]), "elapsed": int(fields[2]), "cpu": int(fields[3]), "stat": fields[4], "cgroup": fields[5], "command": fields[6]})
    return rows


def process_parents(listing):
    """进程表 → {进程号: 父进程号}。"""
    return {row["pid"]: row["ppid"] for row in parsed_process_list(listing)}


def runaway_processes(listing, minimum_cpu_seconds, minimum_fraction, environment=None):
    """任务之外的进程空转的判据（task_server.py 文件头 ⑥）：不是内核线程、不在任务 slice 里、累计 CPU 满 minimum_cpu_seconds、累计 CPU 占已跑时长不低于 minimum_fraction。
    弄坏开关 runaway-slice-counted 下任务 slice 里的也算。"""
    found = []
    for row in parsed_process_list(listing):
        if row["pid"] == 2 or row["ppid"] == 2:
            continue
        if TASK_SLICE_MARK in row["cgroup"] and not broken("runaway-slice-counted", environment):
            continue
        if row["cpu"] >= minimum_cpu_seconds and row["cpu"] >= minimum_fraction * max(row["elapsed"], 1):
            found.append(row)
    return found


STOP_PROCESS_POLL_SECONDS = 0.1   # stop_process_here 回读 /proc 的间隔，同 stop_process_script 的 sleep 0.1
RUNAWAY_PROTECTED_COMMANDS = ("claude", "MainThread", "node", "code", "sshd", "systemd", "prune-server", "Xorg", "gnome-shell", "tmux: server")   # 认不出发起者也不碰的命令名（会话、编辑器、登录与桌面、判定库服务）


def runaway_unowned_stoppable(row, environment=None):
    """认不出发起者的空转进程能不能直接终止：只碰登录会话 scope 里的（systemd 单元与系统服务不碰），RUNAWAY_PROTECTED_COMMANDS 点名的不碰。"""
    return "/session-" in row["cgroup"] and row["cgroup"].endswith(".scope") and row["command"] not in RUNAWAY_PROTECTED_COMMANDS


def stop_process_script(pid, grace_seconds):
    """按进程号停的 shell（第二台经 ssh 跑；本机的同一个次序与判法是 stop_process_here）：先 TERM，过 grace_seconds 秒还在发 KILL，再等一会儿；退 0 是回读已不在（僵尸算不在）。"""
    number = int(pid)
    gone = f'gone() {{ s=$(sed -E "s/.*\\) (.).*/\\1/" /proc/{number}/stat 2>/dev/null); [ -z "$s" ] || [ "$s" = Z ]; }}'
    wait_gone = f"for i in $(seq 1 {int(grace_seconds) * 10}); do gone && exit 0; sleep 0.1; done"
    return f"{gone}; kill -TERM {number} 2>/dev/null; {wait_gone}; kill -KILL {number} 2>/dev/null; {wait_gone}; gone"


def process_gone_here(pid):
    """本机这个进程号回读已不在：/proc/<号>/stat 读不到，或状态字是 Z（僵尸算不在，同 stop_process_script 的 gone）。"""
    try:
        with open(f"/proc/{int(pid)}/stat", encoding="utf-8", errors="replace") as handle:
            return handle.read().rsplit(")", 1)[1].split()[0] == "Z"
    except (OSError, IndexError):
        return True


def stop_process_here(pid, grace_seconds):
    """本机按进程号停，与 stop_process_script（第二台跑的那一段）同一个次序与判法：先 TERM，过 grace_seconds 秒还在发 KILL，再等同样久；→ 回读已不在。发信号本身的错不算数，只看回读。"""
    number = int(pid)
    for signal_number in (signal.SIGTERM, signal.SIGKILL):
        with contextlib.suppress(OSError):
            os.kill(number, signal_number)
        deadline = time.monotonic() + grace_seconds
        while time.monotonic() < deadline:
            if process_gone_here(number):
                return True
            time.sleep(STOP_PROCESS_POLL_SECONDS)
    return process_gone_here(number)


COMMAND_CLASS_INTERPRETERS = ("bash", "sh", "python3", "python")


def command_classes(argv):
    """一条命令的同类（task_server.py 的 class_average_seconds）→ 从窄到宽的名字：同一脚本（cargo 是 cargo test 这样带子命令）、同一程序。开头的 env 与变量赋值不算；`bash -c` 这种没有脚本的交空表。"""
    words = list(argv)
    if words and os.path.basename(words[0]) == "env":
        words = words[1:]
    while words and ("=" in words[0] and not words[0].startswith("-") or words[0].startswith("-")):
        words = words[1:]
    if not words:
        return []
    program = os.path.basename(words[0])
    if program in COMMAND_CLASS_INTERPRETERS:
        if len(words) < 2 or words[1].startswith("-"):
            return []
        return [f"{program} {os.path.basename(words[1])}", program]
    if program == "cargo" and len(words) > 1:
        return [f"cargo {words[1]}", program]
    return [program]


def session_registry_library():
    """载 scripts/agents/lib_session_registry.py（会话登记的唯一取法）：交模块；载不进抛它的错（服务把这一回当找不到收件 socket）。"""
    if not SESSION_REGISTRY_LOADED:
        SESSION_REGISTRY_LOADED.append(load_module("lib_session_registry_for_task_server", SESSION_REGISTRY_SCRIPT))
    return SESSION_REGISTRY_LOADED[0]


def session_inbox_socket(session, environment=None):
    """按会话号找这个会话此刻的收件 socket（task_server.py 文件头 ⑥「收件地址」）→ (路径, 找不到时的那一句)：读会话登记目录
    （~/.claude/sessions，自证换成 SESSIONS_DIRECTORY_OVERRIDE 点名的）里会话号相同、进程还活着、带 messagingSocketPath 的那几份，取启动最晚的一份；
    读法与判活照 lib_session_registry.py（不另读 .key 文件，不另判别的字段）。没有会话（人手）、目录读不了、没有这样的一份，交 ("", 那一句)。"""
    if not session:
        return "", "这条任务没有会话（人手交的）"
    try:
        registry = session_registry_library()
        directory = selftest_override(SESSIONS_DIRECTORY_OVERRIDE, environment) or registry.registry_directory({})
        entries = registry.session_registry_entries(directory)
    except Exception as error:   # 载不进、读的时候抛错：这一回找不到
        return "", f"读会话登记出了错（{type(error).__name__}: {error}）"
    if entries is None:
        return "", f"会话登记目录 {directory} 读不了，或里面一份能解的登记都没有"
    alive = [entry for entry in entries if entry.session_id == session and entry.is_alive and entry.messaging_socket_path]
    if not alive:
        return "", f"会话登记目录 {directory} 里没有会话 {session} 进程还活着、带收件 socket 的那一份（会话已关）"
    return max(alive, key=registry.started_or_earliest).messaging_socket_path, ""


def confirm_command(task_id, items, tasks):
    """确认单打的确认命令（task_server.py 文件头 ⑦）：客户端照它打，钩子自证照它喂（.claude/hooks/lib_project_code_runs.py）。"""
    return f"task-server/task confirm {task_id} --items {items} --tasks {tasks}"


LONG_RUN_WHY_PLACEHOLDER = "<为什么要跑这么久、范围为什么是这些>"
LONG_RUN_WHY_MINIMUM_CHARACTERS = 12   # task confirm <号> --why 去掉空白至少这么多个字（task_server.py 文件头 ⑦「跑满两小时」）


def long_run_confirm_command(task_id, why=LONG_RUN_WHY_PLACEHOLDER):
    """跑满两小时提醒那一条「对」的出路整行（task_server.py 文件头 ⑦「跑满两小时」）：status 与拒绝说明照它打，自证照它喂。"""
    return f"task-server/task confirm {task_id} --why {shlex.quote(why)}"


def long_run_confirmation_problem(why, environment=None):
    """长跑确认的 --why（task confirm <号> --why '<…>'；task_server.py 文件头 ⑦「跑满两小时」）：去掉空白至少 LONG_RUN_WHY_MINIMUM_CHARACTERS 个字。
    → 不合的那一句，合的交空串。弄坏开关 long-run-short-why-accepted 下不判字数。"""
    if broken("long-run-short-why-accepted", environment):
        return ""
    stated = "".join((why or "").split())
    if len(stated) < LONG_RUN_WHY_MINIMUM_CHARACTERS:
        return f"--why 去掉空白只有 {len(stated)} 个字，至少 {LONG_RUN_WHY_MINIMUM_CHARACTERS} 个：写为什么要跑这么久、范围为什么是这些"
    return ""


def confirm_needed_line(task_id, items, tasks, timeout_seconds):
    """确认单末尾那一行 CONFIRM_NEEDED_MARK（机器读的；数是服务此刻算的）。"""
    return f"{CONFIRM_NEEDED_MARK} task={task_id} items={items} tasks={tasks} timeout={float(timeout_seconds):.0f}"


def confirm_needed_fields(text):
    """text 里最后一行 CONFIRM_NEEDED_MARK 的 (任务号, 项数, 条数)；没有交 None。"""
    found = list(CONFIRM_NEEDED_LINE.finditer(text or ""))
    return tuple(int(value) for value in found[-1].groups()) if found else None


def shell_inner_commands(command, environment=None):
    """命令是 bash / sh -c '<脚本>'（key_words 剥过前缀之后）时，脚本里按 && || ; | & 与括号切出的每一条命令（词表，重定向和它的目标去掉）→ [[词…]…]；不是 -c 的、引号不配对拆不开的交 []。
    确认单（named_scope、crash_scope_text、task_server.py 的 scope_is_whole）用它读 bash -c 里命令位置上的脚本与参数，不读被起脚本的正文。
    弄坏开关 shell-inner-scope-unread 下交 []（bash -c 里的范围一律认不出）。"""
    words = key_words(command, environment)
    index = command_word_index(words)
    if broken("shell-inner-scope-unread", environment) or index >= len(words) - 1 or words[index] != "-c":
        return []
    try:
        lexer = shlex.shlex(words[index + 1], posix=True, punctuation_chars=True)
        lexer.whitespace_split = True
        tokens = list(lexer)
    except ValueError:
        return []
    commands, current, skip_target = [], [], False
    for token in tokens:
        if skip_target:   # 重定向的目标不是命令的参数
            skip_target = False
            continue
        if token and set(token) <= set("&|;()"):
            commands.append(current)
            current = []
        elif token and set(token) <= set("<>&") and ("<" in token or ">" in token):
            skip_target = True
            if current and current[-1].isdigit():   # 2>&1 的 2 是文件描述符，不是参数
                current.pop()
        else:
            current.append(token)
    commands.append(current)
    return [inner for inner in commands if inner]


def named_scope(command, environment=None):
    """确认单的项数（task_server.py 文件头 ⑦）：命令（key_words 剥过不改跑法的前缀与临时 prune-server 包装）里点名的 --item、--check（值照逗号拆）、
    --test、--bin、--lib 与 `--` 之后带了 --exact 时的测试名（LIBTEST_VALUE_OPTIONS 的值不算）各几个 → [(选项, 值)…]，次序照命令；一个都没点的交 []。
    命令是 bash / sh -c 的，外层一个都没点时读脚本里每一条命令位置上点的（shell_inner_commands）。"""
    named = named_scope_words(command, environment)
    if not named:
        for inner in shell_inner_commands(command, environment):
            named += named_scope_words(inner, environment)
    return named


def named_scope_words(command, environment=None):
    """named_scope 对一条命令（不钻 bash -c）点名的项。"""
    words = key_words(command, environment)
    before, after = (words[:words.index("--")], words[words.index("--") + 1:]) if "--" in words else (words, [])
    named, position = [], 0
    while position < len(before):
        name, joined, value = before[position].partition("=")
        if name in NAMED_SCOPE_OPTIONS:
            if not joined:
                value = before[position + 1] if position + 1 < len(before) else ""
                position += 1
            named += [(name, piece) for piece in (value.split(",") if name == "--check" else [value]) if piece]
        elif before[position] == "--lib":
            named.append(("--lib", ""))
        position += 1
    if "--exact" in after:
        skip = False
        for word in after:
            if skip or word in LIBTEST_VALUE_OPTIONS:
                skip = not skip
                continue
            if not word.startswith("-"):
                named.append(("--exact", word))
    return named


def named_scope_text(named, whole, environment=None):
    """named_scope 交的那几项写成「--item 3、--test 1」（各选项照头一次出现的次序）；一个都没有的，whole 为真（task_server.py 的 scope_is_whole）写 WHOLE_SCOPE_TEXT，
    别的写 SCOPE_INSIDE_SCRIPT_TEXT。弄坏开关 scope-inside-script-unnamed 下一律写 WHOLE_SCOPE_TEXT。"""
    counts = {}
    for name, _ in named:
        counts[name] = counts.get(name, 0) + 1
    if counts:
        return "、".join(f"{name} {count}" for name, count in counts.items())
    return WHOLE_SCOPE_TEXT if whole or broken("scope-inside-script-unnamed", environment) else SCOPE_INSIDE_SCRIPT_TEXT


def crash_scope_text(command, cwd, environment=None):
    """确认单「崩溃用例与域」那一行（task_server.py 文件头 ⑦）：命令里点名的 crash-case 项数与 --crash-expand-up-to 的值；CRASH_SCOPE_ENTRIES 起的、命令里都没写的写 SCOPE_INSIDE_SCRIPT_TEXT；
    别的命令一样都没写的交空串（不打这一行）。弄坏开关 crash-scope-unreported 下一律空串。"""
    if broken("crash-scope-unreported", environment):
        return ""
    inner_results = [found for found in (crash_scope_text(inner, cwd, environment) for inner in shell_inner_commands(command, environment)) if found]
    if inner_results:   # bash -c 的脚本里命令位置上点了崩溃用例或域的：读那一条（脚本里有几条，取头一条给出确实点了的）
        pointed = [found for found in inner_results if found != SCOPE_INSIDE_SCRIPT_TEXT]
        return (pointed or inner_results)[0]
    words = key_words(command, environment)
    cases = [value for name, value in named_scope(command, environment) if name == "--item" and value.startswith(CRASH_CASE_ITEM_PREFIX)]
    domain = ""
    for position, word in enumerate(words):
        if word == CRASH_DOMAIN_ARGUMENT and position + 1 < len(words):
            domain = words[position + 1]
        elif word.startswith(CRASH_DOMAIN_ARGUMENT + "="):
            domain = word[len(CRASH_DOMAIN_ARGUMENT) + 1:]
    found = entry_word(words, cwd)
    entry = found is not None and any(entry_is(found, relative) for relative in CRASH_SCOPE_ENTRIES)
    if not cases and not domain:
        return SCOPE_INSIDE_SCRIPT_TEXT if entry else ""
    case_text = f"{len(cases)} 条（命令里点名）" if cases else SCOPE_INSIDE_SCRIPT_TEXT
    domain_text = f"{CRASH_DOMAIN_ARGUMENT} {domain}" if domain else f"命令里没给 {CRASH_DOMAIN_ARGUMENT}，由脚本定"
    return f"用例 {case_text}；域 {domain_text}"


def named_scope_entry(command, environment=None):
    """点的项属于哪一条入口（比两条命令点的项交不交叠只在同一条入口里比）：剥过前缀之后命令位置上那个词，cargo 另接 -p / --package 点的包（排过序）。"""
    words = key_words(command, environment)
    index = command_word_index(words)
    if index >= len(words):
        return ""
    if os.path.basename(words[index]) != "cargo":
        return words[index]
    packages = sorted(words[position + 1] for position in range(index, len(words) - 1) if words[position] in ("-p", "--package"))
    return " ".join(["cargo", *packages])


def entry_word(argv, cwd):
    """命令位置上起的那一个词（剥法同 sharded_entry）：→ (词, 真路径, 后面的参数)；没有命令词交 None。"""
    index = command_word_index(argv)
    if index >= len(argv):
        return None
    return argv[index], os.path.realpath(os.path.join(cwd or REPOSITORY_ROOT, argv[index])), argv[index + 1:]


def entry_is(found, relative):
    """entry_word 交的那个词起的是不是仓根下的 relative 那一份。"""
    word, script, _ = found
    return script.endswith("/" + relative) or word == relative
