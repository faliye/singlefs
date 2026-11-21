"""task_server.py 拆出的一份：本机与第二台（Machine、LocalMachine、PeerMachine）、资源读数（Resources）、一轮的视图与判疑似卡住的窗口、读进程与单元状态的函数（总说明在 task_server.py 文件头）。"""
import os
import task_server_lib as lib
import contextlib
import dataclasses
import hashlib
import re
import secrets
import shlex
import shutil
import signal
import subprocess
import threading
import time
import traceback
from dataclasses import dataclass, field
from task_server_constants import (
    ACTIVITY_MEMORY_BYTES, DISK_FLOOR_BYTES, DISK_FLOOR_KIND, DISK_REPORT_KEY, FAST_PEER_PROBE_REFRESH_SECONDS, IO_PRESSURE_PATH,
    IO_PRESSURE_REPORT_KEY, LOCAL, PEER, PEER_BUILD_REMOVE_SECONDS, PEER_DISK_PROBE_TEXT, PEER_HOST_SPECIFIC_VARIABLES, PEER_IO_PRESSURE_PROBE_TEXT,
    PEER_PROBE_FAILURE_HOLD_SECONDS, PEER_PROBE_REFRESH_SECONDS, PEER_PROBE_STALE_SECONDS, PEER_PROBE_TIMEOUT_SECONDS, PEER_RECHECK_SECONDS,
    PEER_STARTS_PER_PASS, QUOTED_WORD_PATH, RESOURCE_CACHE_SECONDS, SHELL_COMMAND_NAMES, SHELL_OPTIONS_WITH_VALUE, SHELL_WORD_PATH,
    UNIT_CGROUPS_UNREADABLE_FILE, UNIT_STATES_UNREADABLE_FILE, read_json_file, say,
)


MACHINE_ORDER = (LOCAL, PEER)   # 挑台时放下之后剩余占比一样大取排在前面的；分片的各片倒过来（candidate_hosts）


@dataclass
class Snapshot:
    reports: dict                                 # {台: 余量报告}：这一轮量得到、能派的台
    memory_caps: dict                             # {台: 那台总上限另收到的数（字节）}：Machine.configured_cap 交得出的台才有
    reachable: dict                               # {台: 此刻连不连得上}（服务所在这一台恒为真；另一台照多机配置判法与 NFS 复核）
    down: dict                                    # {台: 这一轮没给报告的原因}
    hold: bool = False                            # 只在自证里：假余量里写了 hold，一条都不起
    expected: dict = field(default_factory=dict)  # {台: 用不了要不要告警}（note_peer）：只有自证里没给那一台的格是 False
    down_kind: dict = field(default_factory=dict)  # {台: down 那一句是哪一种}（note_peer 按它去重）：盘余量不到下限是 DISK_FLOOR_KIND，别的没有（按原因的字面去重）

    def total(self, host):
        """这台的总上限（字节）：那台 slice 的总上限，另有收到的数（memory_caps）取小者。减那台此刻的余量在 Scheduler.host_capacity。"""
        total = int(self.reports[host]["memory"].get("slice_total") or 0)
        cap = self.memory_caps.get(host)
        return min(total, cap) if cap else total

    def maxima(self):
        """每台的总上限（total）：连得上而这一轮没报告的是 None（查不到），连不上的不在里面。减那台此刻的余量在 Scheduler.usable_maxima（retry_decision 照它比）。"""
        result = {}
        for host in MACHINE_ORDER:
            if host in self.reports:
                result[host] = self.total(host)
            elif self.reachable.get(host):
                result[host] = None
        return result


def peer_build_root(declarations):
    """第二台的编译根：多机配置 BUILD_ROOT_BY_HOST 里第二台那一项（host-dispatch.py 的 Peer 读出的 declare -p 那几行里的 PEER_HOST_BUILD_ROOT，
    peer-host-lib.sh 按地址取），去掉末尾的 /；读不到交空串。"""
    found = re.search(r"PEER_HOST_BUILD_ROOT=\"?([^\"\n]+?)\"?$", declarations or "", re.MULTILINE)
    return found.group(1).rstrip("/") if found else ""


def df_available_bytes(output):
    """df -B1 --output=avail 的输出（表头一行、数一行）：末行的字节数；认不出交 None。"""
    lines = [line.strip() for line in (output or "").splitlines() if line.strip()]
    return int(lines[-1]) if lines and lines[-1].isdigit() else None


def gibibytes_text(byte_count):
    return f"{byte_count / lib.GIBIBYTE:.1f}G"


def reminder_threshold(value, first):
    """要处置的提醒跨过了几道提醒门槛（文件头 ①「要处置的提醒」）：门槛是 first、2 × first、4 × first……；value 不到 first 交 0，到了 first 交 1，
    到了 2 × first 交 2，到了 4 × first 交 3……"""
    if first <= 0 or value < first:
        return 0
    crossed = 1
    while value >= first * 2 ** crossed:
        crossed += 1
    return crossed


def next_reminder_seconds(crossed, first):
    """跨过 crossed 道提醒门槛（reminder_threshold）之后，下一回提醒的那道门槛：first × 2 的 crossed 次方。"""
    return first * 2 ** crossed


def attention_event(item):
    """发消息那一路认一项提醒的键（inbox_failed、attention_held）：(任务号, 种类, 提醒门槛)；再推的那一回另接第几回（同一道门槛再推的每一回各算一件，发失败的那一回不连累下一回）。"""
    key = (item["task"], item["kind"], item["threshold"])
    return key + (item["repush"],) if item.get("repush") else key


def task_id_ranges(task_ids):
    """几个任务号写成区间（合成的那一条提醒点名几片子任务，Scheduler.merged_attention_text）：从小到大，连着的写「头–尾」、单个的写号，几段用「、」接。"""
    numbers = sorted({int(task_id) for task_id in task_ids})
    pieces, start = [], 0
    for index, number in enumerate(numbers):
        if index + 1 == len(numbers) or numbers[index + 1] != number + 1:
            pieces.append(str(numbers[start]) if start == index else f"{numbers[start]}–{number}")
            start = index + 1
    return "、".join(pieces)


def build_directory_name(tree, exists=os.path.lexists, parent=""):
    """本机组里新开的编译目录的名字：「tree-<树路径的 sha256 前 8 位>-<6 位随机十六进制>」，不带任务号与尝试标签。
    编译目录归树的那一组、同一棵树上之后交的任务接着用（task_server_lib.py 文件头「编译目录」），名字若照开它的那一次尝试的标签起（task-<库标签>-<任务号>-<尝试号>），
    之后每一条接着用它的任务的进程在 ps 里都挂着别的任务的号，认错是哪条任务的。parent 是它所在的目录，exists 判这个名字在那里有没有被占。"""
    stem = "tree-" + hashlib.sha256(tree.encode("utf-8")).hexdigest()[:8]
    while True:
        name = f"{stem}-{secrets.token_hex(3)}"
        if not exists(os.path.join(parent, name)):
            return name


# ── 两台：这一台怎么读、怎么起（文件头 ②「两台同一段逻辑」）──
@dataclasses.dataclass
class BuildDirectory:
    """交给那一台起法的这一次编译目录（Machine.command 的 build）。"""
    directory: str   # target 的上一层
    reused: bool     # 接着用这棵树在这台的组里已有的那一个（task_server_lib.py 文件头「编译目录」）；False 是这一次新开的


@dataclass(frozen=True)
class ThreadReading:
    """一次尝试某一轮的线程读数（Machine.thread_reading；文件头 ②「CPU 闸」的可跑线程数）：sample 是那一侧的采样号（本机是 attempt_runner 的轮数，第二台是 poll=），
    runnable 是状态 R 的线程数，total 是线程总数，top_threads、top_name 是占线程最多的那个进程的线程数与进程名。"""
    sample: int
    runnable: int
    total: int
    top_threads: int
    top_name: str


def thread_reading_of(value):
    """本机尝试目录里 lib.THREADS_FILE_NAME 那一份（attempt_runner.py 写的 JSON）→ ThreadReading；几样不是非负整数、进程名不是字符串的交 None（这一轮读不到）。"""
    if not isinstance(value, dict):
        return None
    numbers = [value.get(name) for name in ("sample", "runnable", "total", "top_threads")]
    if not all(isinstance(number, int) and not isinstance(number, bool) and number >= 0 for number in numbers) or not isinstance(value.get("top_name"), str):
        return None
    return ThreadReading(*numbers, value["top_name"])


class Machine:
    """一台机器在服务这一侧的接口：记账、判放不放得下、判卡住、记峰值、上限、编译目录、挑台与盘余量对两台走同一段（Scheduler 里），
    按台不同的只在这里——怎么连上它、怎么量它的余量与盘、怎么读一次尝试此刻的 CPU 与编译目录占盘、怎么起那一次、它的编译目录由谁建。
    两台型号相同（用户 2026-11-02：「两台型号都是一样的 根本没有必要写 if 第一台 else 第二台这种代码」）：这里只有摸到那台的路不同，判法不分台。"""
    name = ""
    always_reachable = False     # 服务所在这一台总连得上；另一台要照多机配置判法与 NFS 复核
    starts_per_pass = None       # 一轮至多在这台起几条（None 不限）
    admission_variable = ""      # 这一台最外层那一层只认服务起的准入标记（task_server_lib.py 的 ADMITTED_VARIABLE / PEER_ADMITTED_VARIABLE）
    admission_break = None       # 只给自证证红：照改前不设这一台准入标记的弄坏开关

    def __init__(self, scheduler):
        self.scheduler = scheduler

    @property
    def label(self):
        return lib.HOST_NAMES[self.name]

    def address(self, addresses):
        """这一台在多机配置 HOSTS 里的地址（Addresses 是 task_server_lib.host_addresses 交的）。"""
        raise NotImplementedError

    def reachable(self, resources):
        """→ (此刻连不连得上, 连不上的原因)。"""
        raise NotImplementedError

    def probe(self, resources):
        """→ (这台此刻的余量报告或 None, 查不成的原因)；报告里另带 DISK_REPORT_KEY（这台编译目录所在盘的余量，量得到才带）。不读卡：卡归 GPU 批处理验证服务。"""
        raise NotImplementedError

    def disk_free(self, resources):
        """报告里没带盘余量时现量：→ (退出码, 输出, 错误)，输出末行是字节数（同 df -B1 --output=avail）。"""
        raise NotImplementedError

    def io_pressure(self, resources):
        """这台此刻 /proc/pressure/io 的原文（Resources.note_io_pressure 取 full 那一行的 avg300）：→ (退出码, 输出, 错误)。"""
        raise NotImplementedError

    def disk_advice(self):
        raise NotImplementedError

    def configured_cap(self, resources):
        """这台的总上限另收到的数（字节）：多机配置 SLICE_MEMORY_CAP_BY_HOST 里这台那一项照它现读得到的；没有另收的交 None。"""
        return None

    def cpu_usec(self, attempt):
        """这一次尝试此刻累计用了多少 CPU（微秒）；读不到交 None。"""
        raise NotImplementedError

    def build_bytes(self, attempt):
        """这一次尝试的编译目录此刻的占盘（字节）；量不到交 None（记账照起的时候记的算）。"""
        return None

    def memory_now(self, attempt):
        """这一次尝试外层那一层内存包装 scope 此刻的内存与峰值（字节）：→ (此刻, 峰值)，读不到的那一样交 None（文件头 ①「顶在上限上卡住」）。"""
        return None, None

    def process_ids_seen(self, attempt, result):
        """这一次尝试进程树里见过的进程号（集合）：本机是 attempt_runner.py 每轮采样记进 result.json 的 process_ids；拿不到交 None（第二台的进程号服务这一侧读不到，
        external_kill_note 对第二台的尝试不下判）。"""
        return None

    def attempt_progress_readings(self, attempt):
        """日志不涨时的进展读数（lib.attempt_progress_readings；文件头 ①「要处置的提醒」的 quiet-log-busy）：→ (读数或 None, 读不成的那一句)。"""
        return None, "这一台不读进展"

    def io_blocked(self, attempt):
        """这一次尝试此刻在 D 状态（不可中断的 IO 等待）的进程数；读不到交 None（文件头 ①「卡在 IO 上」：照原判法判）。"""
        return None

    def thread_reading(self, attempt):
        """这一次尝试最近一轮的线程读数（ThreadReading；文件头 ②「CPU 闸」的可跑线程数）；读不到交 None（这一轮 CPU 闸只照在用核数）。"""
        return None

    def attempt_presence_readings(self, request):
        """读不到单元状态时不经 systemd 核一次尝试在这一台上还有没有进程（文件头 ①「读不到单元状态」）：在这一台上跑 task_server_lib.attempt_presence_readings
        → (读数或 None, 读不成的那一句)。两台跑同一段，只是摸到那台的路不同。"""
        raise NotImplementedError

    def command_exit(self, exit_code, tail):
        """→ (定结局用的退出码, classify_attempt 的 host, 起法那一层自己退的码或 None)。"""
        return exit_code, self.name, None

    def memory_stat_at_peak(self, attempt, result):
        """这一次尝试内存最高那一刻的 memory.stat（文件头 ①「内存最高那一刻」）：本机是 attempt_runner.py 交回的 memory_stat_at_peak；没采到交 None。"""
        return (result or {}).get("memory_stat_at_peak")

    def ended_peaks(self, attempt, tail):
        """尝试结束时起法那一层打进日志的峰值：{memory_peak, build_peak}（没打的不带）。"""
        return {}

    def kept_build_label(self, tail):
        """尝试结束时起法那一层报的「这一次的编译目录留着」（第二台是 multi-host-run.sh 打的 PEER_BUILD_KEPT_MARK 那一行）：→ 留着的那一份的标签或 None。"""
        return None

    def command(self, task, placement, label, environment, capped, build):
        """build 是这一次的编译目录（BuildDirectory；服务不给编译目录的是 None）。"""
        raise NotImplementedError

    def build_directory(self, label, tree):
        """这台上新开的组里那一个编译目录（task_server_lib.py 文件头「编译目录」）：→ (目录, 是不是服务在本机建)。目录是 target 的上一层；本机那一个的名字不带标签
        （build_directory_name），第二台那一个由 multi-host-run.sh 照开它的那一次尝试的标签起名（--label 是它起名的唯一来源）；给不出交空串。tree 是这棵树的仓根。"""
        raise NotImplementedError

    def build_directory_exists(self, directory):
        """组里那一个编译目录此刻还在不在（接着用之前核）。"""
        raise NotImplementedError

    def remove_build_directory(self, directory):
        """删组里那一个编译目录（树不在了那一扫）：→ (删了、回读不在了, 没删成时的说明行)。有进程开着里面的文件就不删。"""
        raise NotImplementedError

    def removal_howto(self, directory):
        """没删成时告警的出路。"""
        raise NotImplementedError


class LocalMachine(Machine):
    """服务所在这一台：余量照 host-dispatch.py 的 probe_local 现读，CPU 读 attempt_runner 写的 cpu_usec，起法是内存包装。"""
    name = LOCAL
    always_reachable = True
    admission_variable = lib.ADMITTED_VARIABLE

    def address(self, addresses):
        return addresses.this

    def reachable(self, resources):
        return True, ""

    def probe(self, resources):
        report, why = resources.host_dispatch.probe_local(resources.settings, False)
        if report is not None:
            with contextlib.suppress(OSError):   # 本机编译目录所在盘（TASK_SERVER_BUILD_DIRECTORY）的余量
                report[DISK_REPORT_KEY] = shutil.disk_usage(resources.build_directory).free
        return report, why

    def disk_free(self, resources):
        try:
            return 0, f"Avail\n{shutil.disk_usage(resources.build_directory).free}\n", ""
        except OSError as error:
            return 1, "", f"{type(error).__name__}: {error}"

    def io_pressure(self, resources):
        try:
            with open(IO_PRESSURE_PATH, encoding="utf-8") as handle:
                return 0, handle.read(), ""
        except OSError as error:
            return 1, "", f"{type(error).__name__}: {error}"

    def disk_advice(self):
        return ("先删掉不用了的仓副本（它的编译目录组在服务下一回扫树时跟着删，task_server_lib.py 文件头「编译目录」；多机配置 TASK_SERVER_BUILD_DIRECTORY 所在盘）"
                "与别的大目录")

    def cpu_usec(self, attempt):
        return self.scheduler.scope_cpu_usec(attempt)

    def process_ids_seen(self, attempt, result):
        ids = (result or {}).get("process_ids")
        return {process_id for process_id in ids if isinstance(process_id, int) and not isinstance(process_id, bool)} if isinstance(ids, list) else None

    def memory_now(self, attempt):
        return self.scheduler.scope_memory_current(attempt), self.scheduler.scope_memory_peak(attempt)

    def io_blocked(self, attempt):
        return self.scheduler.scope_blocked_processes(attempt)

    def thread_reading(self, attempt):
        return self.scheduler.scope_thread_reading(attempt)

    def build_bytes(self, attempt):
        with contextlib.suppress(OSError, ValueError):
            with open(os.path.join(attempt.get("directory", ""), "build_bytes"), encoding="utf-8") as handle:
                return int(handle.read().strip())
        return None

    def attempt_progress_readings(self, attempt):
        # 服务就在这一台：直接调。进程树从起跑标记记的 runner 进程号往下，顶层日志不算（它的大小里有起法那一层的进度行，服务照 log_content 判）。
        # 只在自证里：/proc 换成 TASK_SERVER_SELFTEST_PROC_ROOT
        scheduler = self.scheduler
        environment = scheduler.environment
        runner, problem = recorded_text(scheduler.attempt_paths(attempt)[1], "起跑标记")
        if problem or not runner.isdigit():
            return None, problem or "起跑标记还没写进程号，读不了进程树"
        request = {"root_process_ids": [int(runner)], "excluded_files": [attempt["log"]] if attempt.get("log") else [],
                   "proc_root": lib.selftest_override("TASK_SERVER_SELFTEST_PROC_ROOT", environment) or "/proc",
                   "file_limit": lib.ATTEMPT_PROGRESS_FILE_LIMIT,
                   "write_mode_unchecked": lib.broken("open-file-write-mode-unchecked", environment)}
        try:
            return lib.attempt_progress_readings(request), ""
        except Exception as error:   # 读法自己出事照读不成办：这一回没有读数
            return None, f"本机读进展出了异常（{type(error).__name__}: {error}）"

    def attempt_presence_readings(self, request):
        # 服务就在这一台：直接调。只在自证里：世界目录里有 UNIT_CGROUPS_UNREADABLE_FILE 时 cgroup 根换成世界目录里不在的一处（演 cgroup 也读不到）
        world = lib.selftest_override("TASK_SERVER_SELFTEST_WORLD", self.scheduler.environment)
        if world and os.path.exists(os.path.join(world, UNIT_CGROUPS_UNREADABLE_FILE)):
            request = dict(request, cgroup_root=os.path.join(world, UNIT_CGROUPS_UNREADABLE_FILE + "-root"))
        try:
            return lib.attempt_presence_readings(request), ""
        except Exception as error:   # 读法自己出事照读不成办：核不出，下一回再核
            return None, f"本机核进程出了异常（{type(error).__name__}: {error}）"

    def command(self, task, placement, label, environment, capped, build):
        # 编译目录（组里拿到的那一个）经 environment 的 CARGO_TARGET_DIR 交给命令（start_attempt），这里不用 build
        return ["bash", self.scheduler.memory_wrapper, lib.mebibytes_text(placement.cap)] + capped + task["command"]

    def build_directory(self, label, tree):
        """弄坏开关 build-directory-named-by-label 下照开它的那一次尝试的标签起名。"""
        root = self.scheduler.configuration.build_directory
        if lib.broken("build-directory-named-by-label", self.scheduler.environment):
            return os.path.join(root, label), True
        return os.path.join(root, build_directory_name(tree, parent=root)), True

    def build_directory_exists(self, directory):
        return os.path.isdir(directory)

    def remove_build_directory(self, directory):
        removed, lines = self.scheduler.build_directories.remove_run_build_directory(directory)
        return removed and not os.path.lexists(directory), lines

    def removal_howto(self, directory):
        return "照上面那句停掉开着它的进程，再 python3 scripts/hosts/run_build_directory.py remove " + directory


class PeerMachine(Machine):
    """多机配置 HOSTS 里的另一台：经 host-dispatch.py 的 Peer（多机配置判法）连、probe_peer 在那台上量余量；CPU 与内存读日志里 multi-host-run.sh 每轮打的
    MULTI_HOST_RUN_PEER_CPU 与 MULTI_HOST_RUN_PEER_MEMORY；起法是 multi-host-run.sh --peer，编译目录是这棵树在那台的组里那一个：新开的由它在那台本地盘上建、
    接着用的经 --reuse-build-directory 点名，都带 --keep-build-directory 跑完留着（multi-host-run.sh 文件头「留着的编译目录」；task_server_lib.py 文件头「编译目录」）。"""
    name = PEER
    starts_per_pass = PEER_STARTS_PER_PASS
    admission_variable = lib.PEER_ADMITTED_VARIABLE
    admission_break = "peer-admission-unmarked"

    def address(self, addresses):
        return addresses.peer

    def reachable(self, resources):
        return resources.peer.available, resources.peer.why

    def probe(self, resources):
        return resources.host_dispatch.probe_peer(resources.peer, False, timeout=PEER_PROBE_TIMEOUT_SECONDS)   # 查不成的原因留给 note_peer 告警

    def disk_free(self, resources):
        directory = peer_build_root(resources.peer.declarations)
        if not directory:
            return 1, "", "多机配置 BUILD_ROOT_BY_HOST 里读不到第二台那一项"
        return resources.peer.run(PEER_DISK_PROBE_TEXT.format(directory=shlex.quote(directory)), timeout=PEER_PROBE_TIMEOUT_SECONDS)

    def io_pressure(self, resources):
        return resources.peer.run(PEER_IO_PRESSURE_PROBE_TEXT, timeout=PEER_PROBE_TIMEOUT_SECONDS)

    def disk_advice(self):
        return ("先删掉不用了的仓副本（它在第二台的编译目录组在服务下一回扫树时跟着删），再删第二台的编译根（多机配置 BUILD_ROOT_BY_HOST 里它那一项）下 builds/ 里"
                "multi-host-run.sh 没删成的本地目录（它当时报了原因）与别的大目录")

    def configured_cap(self, resources):
        return resources.peer.memory_cap_bytes if resources.peer is not None else None

    def cpu_usec(self, attempt):
        return self.scheduler.log_offsets.get(attempt.get("id"), (0, 0, None))[2]

    def attempt_progress_readings(self, attempt):
        # 经多机配置的读法（host-dispatch.py 的 Peer.run，同 attempt_presence_readings）在第二台起同一段 attempt_progress_readings：进程树从清理回执点名的
        # runs/<标签>.pid 记的进程号往下（multi-host-run.sh 在第二台起的那一层；它的标准输出经 ssh 接回本机，不是第二台上的文件）
        resources = self.scheduler.resources
        if resources.fake_path:   # 自证不连第二台（Resources 的假余量世界里 self.peer 一直是 None）
            return None, "自证里不连第二台"
        path = attempt.get("peer_cleanup_file") or ""
        receipt = read_json_file(path) if path else None
        pid_file = (receipt or {}).get("pid_file")
        if (receipt or {}).get("status") != "pending" or not isinstance(pid_file, str) or not pid_file.startswith("/"):
            return None, f"清理回执 {path or '（这一次没有）'} 不是 pending 或没点名进程号文件，第二台那一趟读不了进展"
        resources.refresh_peer()
        peer = resources.peer
        if peer is None or not peer.available:
            return None, f"第二台连不上（{peer.why if peer is not None else '多机配置里的第二台没读出来'}）"
        request = {"root_process_id_files": [pid_file], "file_limit": lib.ATTEMPT_PROGRESS_FILE_LIMIT,
                   "write_mode_unchecked": lib.broken("open-file-write-mode-unchecked", self.scheduler.environment)}
        status, stdout, stderr = peer.run(lib.attempt_progress_probe_text(request), timeout=lib.ATTEMPT_PROGRESS_PEER_SECONDS)
        readings = lib.attempt_progress_probe_reply(stdout)
        if status != 0 or readings is None:
            return None, f"第二台读进展没成（退 {status}：{stderr.strip()[-200:]}）"
        return readings, ""

    def attempt_presence_readings(self, request):
        # 经多机配置的读法（host-dispatch.py 的 Peer.run，同 attempt_progress_readings）在第二台起同一段 attempt_presence_readings
        resources = self.scheduler.resources
        if resources.fake_path:   # 自证不连第二台（Resources 的假余量世界里 self.peer 一直是 None）
            return None, "自证里不连第二台"
        cooling = resources.peer_cooling_down()   # 上一回读不成还在冷却里：不去连
        if cooling:
            return None, cooling
        resources.refresh_peer()
        peer = resources.peer
        if peer is None or not peer.available:
            return None, f"第二台连不上（{peer.why if peer is not None else '多机配置里的第二台没读出来'}）"
        status, stdout, stderr = peer.run(lib.attempt_presence_probe_text(request), timeout=lib.ATTEMPT_PRESENCE_PEER_SECONDS)
        readings = lib.attempt_presence_probe_reply(stdout)
        if status != 0 or readings is None:
            return None, f"第二台核进程没成（退 {status}：{stderr.strip()[-200:]}）"
        return readings, ""

    def memory_stat_at_peak(self, attempt, result):
        # 日志里 multi-host-run.sh 每轮打的 MULTI_HOST_RUN_PEER_MEMORY_STAT 里 current_bytes 最大的那一行（log_content 读进 peer_memory_stat；文件头 ①「内存最高那一刻」）
        return self.scheduler.peer_memory_stat.get(attempt.get("id"))

    def memory_now(self, attempt):
        # 日志里 multi-host-run.sh 每轮打的最近一行 MULTI_HOST_RUN_PEER_MEMORY（log_content 读进 peer_memory）；弄坏开关 peer-memory-line-unread 下不读
        if lib.broken("peer-memory-line-unread", self.scheduler.environment):
            return None, None
        return self.scheduler.peer_memory.get(attempt.get("id"), (None, None))

    def io_blocked(self, attempt):
        # 日志里 multi-host-run.sh 每轮打的 MULTI_HOST_RUN_PEER_IO_WAIT（log_content 接进 peer_io_rounds）：上一次读之后接到的几行取最少的，有一行 read=no 记 None
        return self.scheduler.merge_peer_io_rounds(attempt)

    def thread_reading(self, attempt):
        # 日志里 multi-host-run.sh 每轮打的 MULTI_HOST_RUN_PEER_THREADS（log_content 接进 peer_thread_rounds）：上一次读之后接到的几行里 read=yes 的取最大
        return self.scheduler.merge_peer_thread_rounds(attempt)

    def command_exit(self, exit_code, tail):
        # 第二台那条命令自己的退出码（multi-host-run.sh 打的 MULTI_HOST_RUN_STEP step=peer-command event=end … exit=<码>）：有这一行照它定结局，
        # 命令自己退 255 记 command-failed；没有这一行才是 multi-host-run.sh 那一层没跑成（task_server_lib.py 文件头「错误类型」）
        command_exit = lib.peer_command_exit(tail, self.scheduler.environment) if exit_code is not None else None
        if command_exit is None:
            return exit_code, self.name, None
        return command_exit, lib.PEER_COMMAND_HOST, exit_code

    def ended_peaks(self, attempt, tail):
        # multi-host-run.sh 在第二台那条命令结束时打的 MULTI_HOST_RUN_PEER_MEMORY_PEAK 与 MULTI_HOST_RUN_PEER_BUILD_PEAK；读不到不打
        environment = self.scheduler.environment
        peaks = {"memory_peak": None if lib.broken("peer-memory-peak-ignored", environment) else lib.peer_marked_bytes(tail, lib.PEER_MEMORY_PEAK_MARK),
                 "build_peak": lib.peer_marked_bytes(tail, lib.PEER_BUILD_PEAK_MARK)}
        return {name: value for name, value in peaks.items() if value is not None}

    def kept_build_label(self, tail):
        return lib.peer_kept_build_label(tail)

    def build_directory_exists(self, directory):
        # 第二台本地盘上的那一个服务这一侧看不到：组里记着 multi-host-run.sh 报了留着的就当在（它接着用之前在第二台上核在不在，不在退 255，
        # 那一次收尾没报留着，组里删掉它）
        return True

    def remove_build_directory(self, directory):
        # multi-host-run.sh --remove-build-directory 在第二台上核没有进程开着里面的文件、删、回读不在了才退 0（它文件头「留着的编译目录」）；
        # 第二台上一回读不成还在冷却里的不去连（Resources.peer_cooling_down），照没删成交回
        cooling = self.scheduler.resources.peer_cooling_down()
        if cooling:
            return False, [cooling]
        try:
            completed = subprocess.run(["bash", self.scheduler.peer_runner, "--remove-build-directory", os.path.basename(directory)], capture_output=True, text=True,
                                       env=lib.ssh_multiplexed_environment(self.scheduler.environment, self.scheduler.environment),   # 服务起的 ssh 复用连接（task_server_lib.py 文件头「主机」）
                                       stdin=subprocess.DEVNULL, timeout=PEER_BUILD_REMOVE_SECONDS)
        except (OSError, subprocess.TimeoutExpired) as error:
            return False, [f"multi-host-run.sh --remove-build-directory 起不了或超过 {PEER_BUILD_REMOVE_SECONDS} 秒：{error}"]
        return completed.returncode == 0, (completed.stdout + completed.stderr).splitlines()[-5:]

    def removal_howto(self, directory):
        return ("照上面那句到第二台停掉开着它的进程（python3 .claude/singlefs-ai-sop/scripts/proc.py stop <进程号>），再 bash scripts/hosts/multi-host-run.sh --remove-build-directory "
                + os.path.basename(directory))

    def command(self, task, placement, label, environment, capped, build):
        scheduler = self.scheduler
        # 两个共享根照来源仓根那样原样带过去（task_server_lib.py 文件头「共享根」）；弄坏开关 shared-roots-not-passed 下照改前不带
        shared_roots = () if lib.broken("shared-roots-not-passed", scheduler.environment) else lib.SHARED_ROOT_VARIABLES
        carried_names = ("TASK_SERVER_TASK", "TASK_SERVER_ATTEMPT", lib.ATTEMPT_LABEL_VARIABLE, lib.THREAD_ADMITTED_VARIABLE, lib.MEMORY_CAP_VARIABLE,
                         lib.SOURCE_REPOSITORY_VARIABLE, lib.SHARD_PHASE_VARIABLE, lib.SHARD_MAXIMUM_VARIABLE, lib.SHARD_VARIABLE, lib.SHARD_PLAN_VARIABLE, lib.SHARD_OUTPUT_VARIABLE, lib.SHARD_HOST_VARIABLE,
                         lib.SHARD_OUTPUTS_VARIABLE, lib.SHARD_MISSING_VARIABLE, *shared_roots)   # 合并那一步派到第二台也读得到各片的清单（那份 TSV 在共享根下的分片根里）与缺了哪几片
        if lib.broken("shard-outputs-not-carried", scheduler.environment):   # 弄坏开关：照改前不带各片的清单（task_server_lib.py 文件头「分片」③）
            carried_names = tuple(name for name in carried_names if name != lib.SHARD_OUTPUTS_VARIABLE)
        pairs = [f"{name}={value}" for name, value in sorted(environment.items()) if name in task["environment"] or name in carried_names]
        # 本机的路径与编译目录，第二台用它自己的（编译目录由 multi-host-run.sh 在第二台本地盘上建）
        pairs = [pair for pair in pairs if not pair.startswith(("PATH=", "CARGO_HOME=", "RUSTUP_HOME=", "TMPDIR=", *(name + "=" for name in lib.TARGET_DIRECTORY_VARIABLES)))]
        if not lib.broken("host-specific-variables-sent-to-peer", scheduler.environment):
            # 按台取值的（预编库这类）不带；值是本机绝对路径的只带落在共享根下的（第二台照同一个绝对路径读得到：来源仓根、两个共享根、分片的计划与产物目录），别的第二台用它自己的
            pairs = [pair for pair in pairs if pair.split("=", 1)[0] not in PEER_HOST_SPECIFIC_VARIABLES
                     and (not pair.split("=", 1)[1].startswith("/") or pair.split("=", 1)[0] in (lib.SOURCE_REPOSITORY_VARIABLE, *shared_roots)
                          or scheduler.containing_shared_root(os.path.realpath(pair.split("=", 1)[1])))]
        # 判定库服务的地址不带过去：第二台照 multi-host-run.sh 带过去的 SINGLEFS_MULTI_HOST_CONFIG 读同一份多机配置里的 PRUNE_SERVER_TCP_ADDRESS；
        # 配置里没写，第二台上的判定库客户端退 3、点名缺的是哪一样（scripts/prune-server/prune_server_client.py 文件头「连哪一处」），服务不从别处补
        command = list(task["command"])
        copied = []
        if lib.broken("peer-tree-copied", scheduler.environment):   # 弄坏开关：照改前把工作目录拷进第二台的一棵树、在那份拷贝里起
            copied = ["--send", task["cwd"], ".task-server-tree-copy"]
            command = ["env", "-C", ".task-server-tree-copy", *command]
        # 服务给编译目录的（会编的）叫 multi-host-run.sh 跑完留着组里这一个；组里已有的（不是这一次新开的）经 --reuse-build-directory 点名接着用
        # （task_server_lib.py 文件头「编译目录」，两台同一段）。弄坏开关 peer-retry-build-directory-fresh 下不叫留着、不点名接着用
        kept = []
        if build is not None and not lib.broken("peer-retry-build-directory-fresh", scheduler.environment):
            kept = ["--keep-build-directory"] + (["--reuse-build-directory", os.path.basename(build.directory)] if build.reused else [])
        return (["bash", scheduler.peer_runner, "--peer", "--memory-cap", lib.mebibytes_text(placement.cap), "--label", label, *kept, *copied, "--"]
                + (["env", *pairs] if pairs else []) + capped + command)

    def build_directory(self, label, tree):
        # multi-host-run.sh 在这台的编译根（多机配置 BUILD_ROOT_BY_HOST 里它那一项）下 builds/<标签>/ 新建、带 --keep-build-directory 跑完留着；服务记下它在哪，
        # 组里删它的时候经 --remove-build-directory 删（remove_build_directory）
        resources = self.scheduler.resources
        root = resources.build_root_of_peer()
        return (os.path.join(root, "builds", label) if root else ""), False


class Resources:
    """两台的余量：平常经各台的 Machine.probe 现查（缓存 RESOURCE_CACHE_SECONDS 秒，尝试结束时与起之前复核时作废）；自证里读 TASK_SERVER_FAKE_RESOURCES 那份 JSON（每次现读）。
    两台同一套判法：每台照它自己的接口（Machine）连、量，盘余量照同一个下限判（disk_problem）。"""

    def __init__(self, environment, host_dispatch, build_directory, configuration_path):
        self.environment = environment
        self.host_dispatch = host_dispatch
        self.build_directory = build_directory   # 本机编译目录所在盘（TASK_SERVER_BUILD_DIRECTORY）：余量记进本机报告的 DISK_REPORT_KEY
        self.configuration_path = configuration_path   # 多机配置：两台在 HOSTS 里的地址照它认（addresses）
        self.addresses_read = None   # (配置文件的修改时刻与大小, 读的时刻, Addresses 或 None, 问题)
        self.fake_path = lib.selftest_override("TASK_SERVER_FAKE_RESOURCES", environment)
        self.slow_seconds = float(lib.selftest_override("TASK_SERVER_SELFTEST_SLOW_RESOURCES_SECONDS", environment) or 0)
        self.settings = host_dispatch.settings_from_environment()
        self.settings.say = lambda line: None
        # host-dispatch.py 经 peer-host-lib.sh 起的 ssh 接上复用层（task_server_lib.py 文件头「主机」）：PATH 前面接控制目录下的 bin/ssh
        self.settings.environment = lib.ssh_multiplexed_environment(self.settings.environment, environment)
        self.cached = None   # (读的时刻, Snapshot)：本机那一份的缓存
        self.peer = None
        self.peer_read_at = 0.0
        self.machines = {}   # {台: Machine}：Scheduler 建好之后放进来（两台各一份接口）
        # 第二台的余量（peer_probe_loop 读，snapshot 只取最近一份；文件头 ②「第二台的余量不在排队线程里读」）
        self.peer_lock = threading.Lock()
        self.peer_reading = None   # 最近一份 PeerReading
        self.peer_failed = None   # (失败的时刻 monotonic, 原因)：读不成的那一回之后 PEER_PROBE_FAILURE_HOLD_SECONDS 之内不再去连
        self.peer_refresh_seconds = FAST_PEER_PROBE_REFRESH_SECONDS if lib.fast(environment) else PEER_PROBE_REFRESH_SECONDS

    def invalidate(self):
        """作废本机的缓存（尝试结束、起之前复核时）；第二台的那一份归探测线程（peer_probe_loop），不跟着清。"""
        self.cached = None

    def build_root_of_peer(self):
        """第二台的编译根：真服务读多机配置里它那一项（peer_build_root）；自证读假余量 JSON 的 peer_build_root（自证不连真的第二台、self.peer 永远是 None，
        假 multi-host-run.sh 在那个目录下 builds/<标签>/ 建这一趟的编译目录）。读不到交空串。"""
        if self.fake_path:
            return (read_json_file(self.fake_path) or {}).get("peer_build_root") or ""
        return peer_build_root(self.peer.declarations) if self.peer is not None else ""

    def addresses(self):
        """(Addresses(this, peer) 或 None, 问题)：服务所在这一台与另一台在 HOSTS 里的地址。配置文件没改过、又没过 PEER_RECHECK_SECONDS 的照上一次的，不每轮都起判法。"""
        try:
            status = os.stat(self.configuration_path)
            stamp = (status.st_mtime_ns, status.st_size)
        except OSError:
            stamp = None
        cached = self.addresses_read
        if cached is not None and cached[0] == stamp and time.monotonic() - cached[1] < PEER_RECHECK_SECONDS:
            return cached[2], cached[3]
        addresses, problem = lib.host_addresses(self.configuration_path, self.environment)
        self.addresses_read = (stamp, time.monotonic(), addresses, problem)
        return addresses, problem

    def peer_cooling_down(self):
        """第二台上一回读不成、还在 PEER_PROBE_FAILURE_HOLD_SECONDS 之内（peer_failed）→ 那一句；不在交空串。收尾线程、核进程线程、空转线程这几处连第二台之前先问它，
        冷却里直接交「连不上」、不去连（不在第二台断着时每一处各等满自己的 ssh 时限）。弄坏开关 peer-cooldown-ignored-off-scheduler 下一律交空串。"""
        failed = self.peer_failed
        if failed is None or lib.broken("peer-cooldown-ignored-off-scheduler", self.environment) or time.monotonic() - failed[0] >= PEER_PROBE_FAILURE_HOLD_SECONDS:
            return ""
        return f"第二台上一回读不成（{failed[1]}），{PEER_PROBE_FAILURE_HOLD_SECONDS:.0f} 秒之内不再去连"

    def refresh_peer(self):
        """多机配置里的另一台（host-dispatch.py 的 Peer：判法里要连第二台，带 PEER_PROBE_TIMEOUT_SECONDS 的时限）至多每 PEER_RECHECK_SECONDS 重读一次。
        只在探测线程里调（peer_probe_round），排队线程不等它。"""
        if self.peer is None or time.monotonic() - self.peer_read_at > PEER_RECHECK_SECONDS:
            self.peer = self.host_dispatch.Peer(self.settings, timeout=PEER_PROBE_TIMEOUT_SECONDS)
            self.peer_read_at = time.monotonic()

    def read_peer(self, machine):
        """连第二台读一回（探测线程里，或还一份都没有时头一回）→ PeerReading：多机配置判得过才探测，探测到的照 disk_problem 判盘余量（同 snapshot 里本机那一路）。"""
        self.refresh_peer()
        reachable, why = machine.reachable(self)
        report, kind = None, ""
        if reachable:
            report, why = machine.probe(self)
        if report is not None:
            disk_problem = self.disk_problem(machine, report)
            if disk_problem:
                report, why, kind = None, disk_problem, DISK_FLOOR_KIND
            else:
                self.note_io_pressure(machine, report)
        return PeerReading(time.monotonic(), report, why, kind, bool(reachable))

    def peer_probe_round(self):
        """探测线程的一回（文件头 ②「第二台的余量不在排队线程里读」）：上一回读不成的 PEER_PROBE_FAILURE_HOLD_SECONDS 之内不再去连。
        读不成（不是盘余量不到下限）记进 peer_failed。"""
        machine = self.machines.get(PEER)
        if machine is None or self.fake_path:
            return
        if self.peer_failed is not None and time.monotonic() - self.peer_failed[0] < PEER_PROBE_FAILURE_HOLD_SECONDS:
            return
        reading = self.read_peer(machine)
        with self.peer_lock:
            self.peer_reading = reading
        failed = None
        if reading.report is None and reading.kind != DISK_FLOOR_KIND and reading.reachable:
            failed = (time.monotonic(), reading.why)
        self.peer_failed = failed

    def peer_probe_loop(self, stopping):
        """探测线程：每 peer_refresh_seconds 一回（peer_probe_round）；一回出事不停服务，打出来、下一回再读。"""
        while not stopping.wait(self.peer_refresh_seconds):
            try:
                self.peer_probe_round()
            except Exception:
                say("  ! 这一回读第二台的余量出了异常（下一回再读）：\n" + traceback.format_exc())

    def peer_snapshot_entry(self, machine):
        """snapshot 里第二台那一项 → (连不连得上, 报告的拷贝或 None, 原因, down_kind)：取探测线程最近一份（peer_probe_loop），不在调用方的线程里连第二台。
        一份都还没有（服务刚起）的这一回照 read_peer 读一次；上一回读不成、还在 PEER_PROBE_FAILURE_HOLD_SECONDS 之内的照那一句当此刻用不了；
        最近一份比 PEER_PROBE_STALE_SECONDS 旧（探测线程卡在第二台上）的同样当此刻用不了。弄坏开关 peer-probe-in-scheduler 下每次在调用方的线程里现连、读不成不记。"""
        if lib.broken("peer-probe-in-scheduler", self.environment):
            reading = self.read_peer(machine)
            return reading.reachable, (dict(reading.report) if reading.report is not None else None), reading.why, reading.kind
        with self.peer_lock:
            reading = self.peer_reading
        failed = self.peer_failed
        now = time.monotonic()
        if failed is not None and now - failed[0] < PEER_PROBE_FAILURE_HOLD_SECONDS:
            return False, None, f"{machine.label}上一回读不成（{failed[1]}），{PEER_PROBE_FAILURE_HOLD_SECONDS:.0f} 秒之内不再去连", ""
        if reading is None:
            reading = self.read_peer(machine)
            with self.peer_lock:
                self.peer_reading = reading
            if reading.report is None and reading.kind != DISK_FLOOR_KIND and reading.reachable:
                self.peer_failed = (time.monotonic(), reading.why)
        elif now - reading.at > PEER_PROBE_STALE_SECONDS:
            return False, None, f"{machine.label}的余量最近一份是 {now - reading.at:.0f} 秒以前读的（探测线程卡在{machine.label}上），这一轮不派", ""
        return reading.reachable, (dict(reading.report) if reading.report is not None else None), reading.why, reading.kind

    def fake_report(self, fake, machine):
        """自证：假余量 JSON 里这一台的报告（{台: 报告}；down 里写了这一台的演它此刻用不了）→ (报告或 None, 原因)。"""
        why = (fake.get("down") or {}).get(machine.name)
        if fake.get(machine.name) and not why:
            return fake[machine.name], ""
        return None, why or f"自证里没给{machine.label}"

    def snapshot(self):
        fake = None
        if self.fake_path:
            if self.slow_seconds:   # 自证：演余量读得慢（/health 不该跟着慢）
                time.sleep(self.slow_seconds)
            fake = read_json_file(self.fake_path) or {}
        else:
            cached = self.cached
            if cached is not None and time.monotonic() - cached[0] < RESOURCE_CACHE_SECONDS:
                return cached[1]
        reports, reachable, down, down_kind, expected, caps = {}, {}, {}, {}, {}, {}
        for host, machine in self.machines.items():
            judged = False   # 第二台那一项已在 read_peer 里判过盘余量
            if fake is not None:   # 假余量里写了 down（{台: 一句原因}）的演那台用不了：照真的那样要告警；没给那一台的格不当它用不了
                report, why = self.fake_report(fake, machine)
                reachable[host] = machine.always_reachable or report is not None
                expected[host] = host in fake or bool((fake.get("down") or {}).get(host))
                cap = (fake.get("memory_caps") or {}).get(host)
            elif host == PEER:   # 取探测线程最近一份（peer_snapshot_entry），不在这个线程里连第二台
                reachable[host], report, why, kind = self.peer_snapshot_entry(machine)
                if kind:
                    down_kind[host] = kind
                judged = True
                expected[host] = True
                cap = machine.configured_cap(self)
            else:
                reachable[host], why = machine.reachable(self)
                report = None
                if reachable[host]:
                    report, why = machine.probe(self)
                expected[host] = True
                cap = machine.configured_cap(self)
            if cap:
                caps[host] = int(cap)
            if report is not None and not judged:
                disk_problem = self.disk_problem(machine, report, fake)
                if disk_problem:   # 盘余量不到下限或查不成：这一轮不给这台的报告、不派它（同查不成余量的那一种，够了下一轮自己认回）
                    report, why, down_kind[host] = None, disk_problem, DISK_FLOOR_KIND
                else:
                    self.note_io_pressure(machine, report, fake)
            if report is None:
                down[host] = why
            else:
                reports[host] = report
        snapshot = Snapshot(reports, caps, reachable, down, bool((fake or {}).get("hold")), expected=expected, down_kind=down_kind)
        if fake is None:
            self.cached = (time.monotonic(), snapshot)
        return snapshot

    def disk_problem(self, machine, report, fake=None):
        """这台编译目录所在盘的余量够不够（task_server_lib.py 文件头「挑机器」的「盘余量」；两台同一个下限 DISK_FLOOR_BYTES）：→ 不到下限或查不成的那一句，够了交空串。
        report 里有 DISK_REPORT_KEY 照它；没有就照这台的接口现量（Machine.disk_free：本机读编译目录所在盘，第二台在它的编译根上起 df）。
        自证里（fake 是假余量那份 JSON）没有这一项读 fake 的 df 里这一台那一项（演 df 的输出），都没写的当盘够。弄坏开关 disk-floor-unchecked 下一律交空串。"""
        if lib.broken("disk-floor-unchecked", self.environment):
            return ""
        available = report.get(DISK_REPORT_KEY)
        if not isinstance(available, int) or isinstance(available, bool):
            if fake is not None:
                shown = (fake.get("df") or {}).get(machine.name)
                if shown is None:
                    return ""
                status, output, error = 0, str(shown), ""
            else:
                status, output, error = machine.disk_free(self)
            available = df_available_bytes(output) if status == 0 else None
            if available is None:
                return f"{machine.label}盘余量查不成（退 {status}：{(error or output).strip()[-200:]}）"
            report[DISK_REPORT_KEY] = available   # 记进报告：派活时照它判这一条的编译目录占盘放不放得下（文件头 ②「编译目录占盘」）
        if available < DISK_FLOOR_BYTES:
            return f"{machine.label}盘只剩 {gibibytes_text(available)}（下限 {gibibytes_text(DISK_FLOOR_BYTES)}）：编译会在中途写满，{machine.disk_advice()}"
        return ""

    def note_io_pressure(self, machine, report, fake=None):
        """这台此刻 /proc/pressure/io 里 full 那一行的 avg300（百分数）记进报告的 IO_PRESSURE_REPORT_KEY（文件头 ②「磁盘压力」；Scheduler.io_pressure_problem 读它）：
        多机配置没写阈值（lib.io_pressure_limit 交不出数：没写，或写坏了由 io_pressure_problem 报）的不读；读不到、认不出的不带这一项。两台同一套：本机读 /proc/pressure/io，
        第二台经同一条连接起 PEER_IO_PRESSURE_PROBE_TEXT（Machine.io_pressure）。自证里（fake 是假余量那份 JSON）读 fake 的 io_pressure 里这一台那一项（演 /proc/pressure/io 的内容），没写的不带。"""
        limit, _ = lib.io_pressure_limit(self.configuration_path, self.environment)
        if limit is None:
            return
        if fake is not None:
            shown = (fake.get("io_pressure") or {}).get(machine.name)
            status, output = (0, str(shown)) if shown is not None else (1, "")
        else:
            status, output, _ = machine.io_pressure(self)
        reading = io_pressure_full_avg300(output) if status == 0 else None
        if reading is not None:
            report[IO_PRESSURE_REPORT_KEY] = reading

    def recheck(self, host, base):
        """起之前现读一次那一台（文件头 ②「起之前复核」）：缓存作废，交一份新的 Snapshot，这台的余量是现读的，别的照 base；
        现读到的报告照 snapshot 那样判盘余量（disk_problem：不到下限或查不成的当这台这一轮读不到，down_kind 记 DISK_FLOOR_KIND；第二台的探测不带盘余量，
        这里现量的那一份记进报告，编译目录占盘照它判）。这台读不到的，交的那份里没有它的报告、down 写原因。
        自证里读假余量 JSON，里面写了 recheck 的（{台: 余量}，写 null 的演这台这一次读不到）这一次照它。弄坏开关 recheck-disk-unchecked 下不判盘余量。"""
        self.invalidate()
        machine = self.machines[host]
        fake = None
        if self.fake_path:
            fake = read_json_file(self.fake_path) or {}
            report = dict(fake, **(fake.get("recheck") or {})).get(host)
            why = "" if report is not None else f"自证里复核时没给{machine.label}"
        elif host == PEER and self.peer_failed is not None and time.monotonic() - self.peer_failed[0] < PEER_PROBE_FAILURE_HOLD_SECONDS:
            # 第二台上一回读不成、还在不再去连的那一段里（peer_snapshot_entry）：复核同样不连，照那一句当这一次读不到
            report, why = None, f"{machine.label}上一回读不成（{self.peer_failed[1]}），{PEER_PROBE_FAILURE_HOLD_SECONDS:.0f} 秒之内不再去连"
        else:
            if host == PEER and self.peer is None:
                self.refresh_peer()
            reachable, why = machine.reachable(self)
            report = None
            if reachable:
                report, why = machine.probe(self)   # 第二台的探测与 df 都带 PEER_PROBE_TIMEOUT_SECONDS 的时限
        down = {name: text for name, text in base.down.items() if name != host}
        down_kind = {name: kind for name, kind in base.down_kind.items() if name != host}
        if report is not None and not lib.broken("recheck-disk-unchecked", self.environment):
            disk_problem = self.disk_problem(machine, report, fake)
            if disk_problem:   # 同 snapshot：盘余量不到下限或查不成，这一轮不给这台的报告、不派它
                report, why, down_kind[host] = None, disk_problem, DISK_FLOOR_KIND
            else:
                self.note_io_pressure(machine, report, fake)
        reports = {name: value for name, value in base.reports.items() if name != host}
        if report is not None:
            reports[host] = report
        else:
            down[host] = why or f"起之前复核读不到{machine.label}"
        return dataclasses.replace(base, reports=reports, down=down, down_kind=down_kind)


class PassView:
    """一轮排队读的余量（文件头 ②「余量一轮只读一次」）：每台的余量（Resources.snapshot）、服务账上还没涨到量的那一截（server_pending）
    在这一轮第一次用到时读，这一轮里各条共用；这一轮起了一条，它的记账加进那台的待涨量（book），后面的照加过的判。
    弄坏开关 pending-read-per-task 下每次用到都现读，pass-booking-not-carried 下起了的不加。"""

    def __init__(self, scheduler, source):
        self.scheduler = scheduler
        self.source = source              # pass（排队那一轮）、health（/health 判排头）
        self.number = next(scheduler.view_numbers)
        self.latest_snapshot = None       # 这一轮读进来的那一份 Snapshot：待涨量照它里面第二台的报告对 scope（server_pending）
        self.pending = {}                 # {台: 内存字节}：读到的待涨量
        self.disk_pending = {}            # {台: 字节}：服务账上这台在跑的尝试编译目录还要涨的占盘，连同这一轮起了的那几条的记账
        self.carried = {}                 # {台: 内存字节}：这一轮起了的那几条的记账

    def read_each_time(self):
        return lib.broken("pending-read-per-task", self.scheduler.environment)

    def snapshot(self):
        if self.latest_snapshot is None or self.read_each_time():
            self.scheduler.selftest_read(f"read view={self.number} source={self.source} what=snapshot")
            self.latest_snapshot = self.scheduler.peer_view(self.scheduler.resources.snapshot())
        return self.latest_snapshot

    def read_pending(self, host, source):
        """现读这台的待涨量；读进来的已含库里此刻在跑的每一次尝试，这一轮起了的那几条不再另加。第二台的用量照这一轮最近读进来的那一份余量报告
        （latest_snapshot；这一轮还没读余量时没有报告，第二台在跑的记整份上限），不为它另读一次余量。"""
        self.scheduler.selftest_read(f"read view={self.number} source={source} what=pending host={host}")
        report = self.latest_snapshot.reports.get(host) if self.latest_snapshot is not None else None
        self.pending[host] = self.scheduler.server_pending(host, report)
        self.carried.pop(host, None)

    def pending_of(self, host):
        """内存字节：这台还没涨到量的那一截，连同这一轮起了的那几条的记账。"""
        if host not in self.pending or self.read_each_time():
            self.read_pending(host, self.source)
        return self.pending[host] + self.carried.get(host, 0)

    def book(self, host, memory, disk=0):
        """这一轮在这台起了一条：把它的记账加进这一轮那台的待涨量。"""
        if lib.broken("pass-booking-not-carried", self.scheduler.environment):
            return
        self.carried[host] = self.carried.get(host, 0) + int(memory or 0)
        self.disk_pending[host] = self.disk_pending_of(host) + int(disk or 0)

    def disk_pending_of(self, host):
        """这台在跑的尝试的编译目录还要涨的占盘（server_disk_pending，这一轮第一次用到时读），连同这一轮起了的。"""
        if host not in self.disk_pending:
            self.disk_pending[host] = self.scheduler.server_disk_pending(host)
        return self.disk_pending[host]

    def refresh(self, host, snapshot):
        """起之前复核现读过的：现读的余量换进这一轮，这台的待涨量也现读（复核那一读记 source=recheck）。"""
        self.latest_snapshot = snapshot
        self.read_pending(host, "recheck")


@dataclass
class PeerReading:
    """探测线程读第二台的一回（Resources.read_peer）。"""
    at: float          # 读完的时刻（time.monotonic）
    report: object     # 余量报告；读不成或盘余量不到下限是 None
    why: str           # 读不成的原因
    kind: str          # down_kind：盘余量不到下限是 DISK_FLOOR_KIND，别的是空串
    reachable: bool    # 多机配置判得过、第二台用得上


@dataclass
class Placement:
    kind: str                  # start、blocked、never、unmet
    host: str = ""
    cap: int = 0
    booked: int = 0
    disk_booked: int = 0       # 编译目录占盘的记账（字节；文件头 ②「编译目录占盘」）
    history: str = ""
    why: str = ""
    cpu_small_slot: str = ""   # 靠 CPU 闸要核数少的名额起的那一句（cpu_small_slot_note）：起的时候记进尝试，cpu_small_slots_used 照它数
    build_tree: str = ""       # blocked 且放得下的各台都只缺这棵树的编译目录名额时，那棵树的仓根（pool_only_tree）；别的原因空
    idle_short: str = ""       # blocked 且在每一台上就算服务在那台跑的任务全收尾、整机空闲也不够要的空闲时，各台各一句（idle_short_text）；别的原因空
    idle_short_settled: bool = False   # blocked 时 idle_short_text 真对每一台判过（有一台量不到余量、盘不够或连不上而没判成的是 False；是不是在每一台上都起不来，结论在 idle_short 里）；note_idle_short_judged 照它记


HEAD_STARTED = "started"                      # 派活这一轮判它能起，起了
HEAD_NOT_STARTED = "startable-not-started"    # 派活这一轮判它能起，却没起成（why 写为什么）：/health 只报连着几轮这样的
HEAD_BLOCKED = "not-startable"                # 派活这一轮判它不能起（why 写在等什么，与排队原因同一句）或判不了


@dataclass
class HeadVerdict:
    """派活这一轮给一条排着的任务的结论（Scheduler.place_queued 记，排头那一条换进 head_verdict；文件头「接口」的 /health）。"""
    task_id: int
    kind: str                  # HEAD_STARTED、HEAD_NOT_STARTED、HEAD_BLOCKED
    why: str = ""


@dataclass
class SuspectWindow:
    """判疑似卡住的一个 CPU 窗口（Scheduler.check_suspect）：时刻是 time.monotonic() 的钟。"""
    started_at: float
    started_cpu: int
    last_at: float
    last_cpu: int
    suspected: bool           # 这一段已报疑似卡住、还没结束


@dataclass
class IoWaitWindow:
    """判卡住的一个 CPU 窗口里每一轮读到的 D 状态进程数（Scheduler.note_io_wait；文件头 ①「卡在 IO 上」）：时刻是 time.monotonic() 的钟。"""
    window_started_at: float  # 这一个窗口开头的时刻（log_growth 记的那一个）：窗口换了从头记
    polls: int                # 这个窗口里读了几轮
    every_poll_blocked: bool  # 这个窗口里每一轮都读到了、都有进程在 D 状态
    blocked_now: object       # 最近一轮读到的 D 状态进程数（int；读不到是 None）
    blocked_since: object     # 从哪一刻起每一轮都有进程在 D 状态（float；跨窗口接着算，有一轮没有或读不到是 None；日志涨了整份删）


@dataclass
class ActivityWindow:
    """判卡住的一个窗口里这一次尝试的进程内存（Scheduler.note_activity；文件头 ①「卡住」的有进展）：窗口换了从头记。"""
    window_started_at: float  # 这一个窗口开头的时刻（log_growth 记的那一个）
    memory_first: object      # 这个窗口里头一回读到的进程内存（字节，Machine.memory_now 的此刻；读不到是 None）
    peak_first: object        # 同一回读到的这一次尝试的内存峰值（字节；读不到是 None）
    memory_last: object       # 最近一回读到的进程内存（读不到的那一回不盖掉）
    peak_last: object         # 最近一回读到的内存峰值（同上）


def signal_name(number):
    """信号号的名字（SIGTERM 这类）；认不出的写「信号 <号>」。"""
    try:
        return signal.Signals(number).name
    except ValueError:
        return f"信号 {number}"


def activity_window_after(previous, window_started_at, memory, peak):
    """ActivityWindow 记上这一轮读到的进程内存 memory 与内存峰值 peak（字节）：窗口换了（或头一回）从这一轮起记，
    读不到（None）的那一样不盖掉已记的。→ 新的那一份。"""
    if previous is None or previous.window_started_at != window_started_at:
        return ActivityWindow(window_started_at, memory, peak, memory, peak)
    return ActivityWindow(window_started_at,
                          previous.memory_first if previous.memory_first is not None else memory,
                          previous.peak_first if previous.peak_first is not None else peak,
                          memory if memory is not None else previous.memory_last, peak if peak is not None else previous.peak_last)


def activity_text(window, window_started_at):
    """这一窗口里有没有进展（文件头 ①「卡住」的有进展；judge_stall_stop 用）：进程内存或内存峰值涨了至少 ACTIVITY_MEMORY_BYTES，
    交给人看的那半句；都没有、窗口不是这一个或没记过交空串。"""
    if window is None or window.window_started_at != window_started_at:
        return ""
    for name, first, last in (("进程内存", window.memory_first, window.memory_last), ("内存峰值", window.peak_first, window.peak_last)):
        if first is not None and last is not None and last - first >= ACTIVITY_MEMORY_BYTES:
            return f"{name}这一窗口从 {first >> 20} MiB 涨到 {last >> 20} MiB"
    return ""


def descendant_processes(root_process_id, directory="/proc"):
    """root 的子孙进程号（照 <directory>/<号>/stat 的父进程号一层层往下，不按名字找；读不到的那几个跳过）。directory 只在自证里换（TASK_SERVER_SELFTEST_PROC_ROOT）。"""
    children = {}
    for name in os.listdir(directory):
        if not name.isdigit():
            continue
        try:
            with open(os.path.join(directory, name, "stat"), encoding="utf-8", errors="replace") as handle:
                parent = int(handle.read().rsplit(")", 1)[1].split()[1])
        except (OSError, IndexError, ValueError):
            continue
        children.setdefault(parent, []).append(int(name))
    found, frontier = set(), [root_process_id]
    while frontier:
        for child in children.get(frontier.pop(), []):
            if child not in found:
                found.add(child)
                frontier.append(child)
    return found


def process_is_zombie(process_id):
    """这个进程号已退出、只剩等父进程收的僵尸（/proc/<号>/stat 的状态是 Z）；读不到交 False（调用方另看 /proc/<号> 在不在）。"""
    try:
        with open(f"/proc/{process_id}/stat", encoding="utf-8", errors="replace") as handle:
            return handle.read().rsplit(")", 1)[1].split()[0] == "Z"
    except (OSError, IndexError):
        return False


def process_run_state(process_id, directory="/proc"):
    """<directory>/<号>/stat 的状态字段（R、S、D、Z 这类一个字母；进程名里有右括号也照最后一个右括号切）；读不到交空串。"""
    try:
        with open(os.path.join(directory, str(process_id), "stat"), encoding="utf-8", errors="replace") as handle:
            return handle.read().rsplit(")", 1)[1].split()[0]
    except (OSError, IndexError):
        return ""


def io_pressure_text(path=IO_PRESSURE_PATH):
    """本机此刻 /proc/pressure/io 的 some avg60（「some avg60=12.34」）；文件不在、读不了、认不出交「/proc/pressure/io 读不到」。"""
    try:
        with open(path, encoding="utf-8") as handle:
            for line in handle:
                found = re.match(r"^some .*\bavg60=([0-9.]+)", line)
                if found:
                    return f"some avg60={found.group(1)}"
    except OSError:
        pass
    return f"{path} 读不到"


def io_pressure_full_avg300(text):
    """/proc/pressure/io 的内容里「full」那一行的 avg300（百分数，float）；没有这一行、认不出交 None（文件头 ②「磁盘压力」）。"""
    found = re.search(r"^full .*\bavg300=([0-9]+(?:\.[0-9]+)?)", text, re.MULTILINE)
    return float(found.group(1)) if found else None


def shell_script_word_indexes(command):
    """命令里 shell 的 -c 脚本那一个词的下标（find_outside_shared_path 照 shell 词读它）：命令词名（basename）在 SHELL_COMMAND_NAMES 里的词后面，
    选项（- 或 + 起头）里有一个带 c 的短选项串（-c、-ec、-lc）的，选项（与收尾的 --）之后头一个词；SHELL_OPTIONS_WITH_VALUE 后面那个词是选项的值，跳过。
    套在包装里的（env、timeout、临时 prune-server 的 -- 后面）照认：每个命令词名对得上的词都往后看。"""
    indexes = set()
    for index, word in enumerate(command):
        if os.path.basename(word) not in SHELL_COMMAND_NAMES:
            continue
        cursor, script = index + 1, False
        while cursor < len(command) and command[cursor][:1] in ("-", "+") and command[cursor] != "--":
            option = command[cursor]
            if option in SHELL_OPTIONS_WITH_VALUE:
                cursor += 1
            elif not option.startswith("--") and "c" in option[1:]:
                script = True
            cursor += 1
        if cursor < len(command) and command[cursor] == "--":
            cursor += 1
        if script and cursor < len(command):
            indexes.add(cursor)
    return indexes


def shell_word_paths(script, pattern=SHELL_WORD_PATH):
    """shell 脚本（-c 那一个词）里的绝对路径（find_outside_shared_path）：照 shlex（posix）切成 shell 词，每个词里照 SHELL_WORD_PATH 取：以 / 起头的整个词、
    「名字=绝对路径」「--选项=绝对路径」的值，与挨着 ;|&<>( 写、没隔空白的下一个词（cmd;/x、>/x、(/x）。引号里的整段是一个词（"（接的是 /）" 里的 / 跟在空白后面，不算），
    词中间夹着的 / 不算；${X:?}/…、$(pwd)/… 这类展开之后的尾巴不算（弄坏开关 expansion-tail-as-path 下调用方给 pattern=SHELL_WORD_PATH_WITH_EXPANSION_TAILS，尾巴也取）。
    # 不当注释（注释里写的路径照认）。切不开的（引号不配对）照 QUOTED_WORD_PATH 按子串认：多拒不漏。"""
    try:
        lexer = shlex.shlex(script, posix=True)
        lexer.whitespace_split = True
        lexer.commenters = ""
        words = list(lexer)
    except ValueError:
        return QUOTED_WORD_PATH.findall(script)
    return [path for word in words for path in pattern.findall(word)]


# ── 单元 ──
def unit_states(unit, environment):
    """systemctl --user show 读的 (LoadState, ActiveState)（照输出判，不看退出码）；查不了交 None。
    只在自证里：TASK_SERVER_SELFTEST_WORLD 下有 UNIT_STATES_UNREADABLE_FILE 时交 None（unit-unreadable- 起头的那四格演一直读不到）。"""
    world = lib.selftest_override("TASK_SERVER_SELFTEST_WORLD", environment)
    if world and os.path.exists(os.path.join(world, UNIT_STATES_UNREADABLE_FILE)):
        return None
    try:
        completed = subprocess.run(["systemctl", "--user", "show", "-p", "LoadState", "-p", "ActiveState", unit], capture_output=True, text=True,
                                   env=environment, stdin=subprocess.DEVNULL, timeout=30)
    except (OSError, subprocess.TimeoutExpired):
        return None
    values = dict(line.split("=", 1) for line in completed.stdout.splitlines() if "=" in line)
    if "ActiveState" not in values:
        return None
    return values.get("LoadState", ""), values["ActiveState"]


def unit_processes(unit, environment):
    """单元 cgroup 里此刻的进程号（读 cgroup.procs 的内容）；单元已经没了交空表。"""
    try:
        completed = subprocess.run(["systemctl", "--user", "show", "-p", "ControlGroup", "--value", unit], capture_output=True, text=True,
                                   env=environment, stdin=subprocess.DEVNULL, timeout=30)
    except (OSError, subprocess.TimeoutExpired):
        return None
    if completed.returncode:
        states = unit_states(unit, environment)
        return [] if states is not None and states[0] == "not-found" else None
    group = completed.stdout.strip()
    if not group:
        return []
    return cgroup_processes("/sys/fs/cgroup" + group)


def recorded_text(path, what):
    """读尝试目录里记一行字的小文件（起跑标记 started、attempt_runner 写的 scope 文件；文件头 ①「读不到单元状态」）→ (去掉首尾空白的内容, "")；
    文件不在而尝试目录在交 ("", "")（还没写过），读不了、尝试目录也不在交 ("", 那一句)。"""
    try:
        with open(path, encoding="utf-8") as handle:
            return handle.read().strip(), ""
    except FileNotFoundError:
        if os.path.isdir(os.path.dirname(path)):
            return "", ""
        return "", f"{what} {path} 不在，尝试目录也不在，核不出"
    except OSError as error:
        return "", f"{what} {path} 读不了（{type(error).__name__}: {error}），核不出"


def cgroup_processes(group_path):
    """一个 cgroup 目录此刻的进程号（读 cgroup.procs）：目录已经没了交空表，读不了交 None。"""
    try:
        with open(os.path.join(group_path, "cgroup.procs"), encoding="utf-8") as handle:
            return handle.read().split()
    except FileNotFoundError:
        return None if os.path.isdir(group_path) else []
    except OSError:
        return None


def registered_threads(environment):
    """每台默认重活线程数（task_server_lib.py 文件头「线程」）；取不到抛 Unavailable（不退回按核数算）。"""
    heavy_threads = lib.load_module("heavy_threads", os.path.join(lib.HOSTS_DIRECTORY, "heavy_threads.py"))
    try:
        return int(heavy_threads.default_heavy_threads(environment))
    except (heavy_threads.HeavyThreadDefaultError, ValueError) as error:
        raise lib.Unavailable(f"取不到每台默认重活线程数（{error}）", "修好 .claude/heavy-thread-default（或 unset 写坏的 SINGLEFS_THREAD_CAP）再起服务") from None
