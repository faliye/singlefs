#!/usr/bin/env python3
"""task-server 每一次尝试的单元里跑的那一层（task_server.py 写好说明、经 systemd-run --user 起它；人与 agent 不起它）。

  attempt_runner.py <说明 spec.json>

照说明（argv、cwd、environment、log、directory、host、gpu、fast、fake_scope_directory、nvidia_smi、exit_peak_file、memory_peak_file、build_finished_pattern、build_phase_file）做：
  ① 在尝试目录里写起跑标记 started（服务据它分「起单元之前没了」与「起了」）；
  ② 换目录、换环境，起 argv 那一条（内存包装、multi-host-run.sh 或清场包装打头），标准输出与错误都追加进日志；
  ③ 跑的时候每 SAMPLE_SECONDS 秒采一次（各 scope 的 CPU 合计每次写进尝试目录的 cpu_usec，服务跑着时读）：这一条的子孙进程所在的 scope（/proc/<进程号>/cgroup 末段是 singlefs-memory-cap-*.scope 的那几个；
     离这一层最近的那一个是内存包装给这一条开的，写进尝试目录的 scope 文件给服务算「还没涨到量」那一截）的 cpu.stat 的 usage_usec，
     与外层 scope 进程用的内存（memory.stat 的 anon 加 shmem，scripts/hosts/memory_peaks_lib.py 的 process_memory_bytes；页缓存与内核挂在 scope 上的不算，
     task-server/README.md「算这台还放得下多少」同一个口径），留见过的最大，涨了就写进说明里 memory_peak_file 那一份（尝试目录下；服务判顶在上限上卡住读它）；
     编译那一段的峰值：每一轮采之前先从上一轮读到的位置接着读日志的整行（一轮至多 LOG_SCAN_BYTES），找说明里 build_finished_pattern 认的那一行（cargo 编完打的
     「Finished … target(s) in …」，task_server_lib.py 的 CARGO_FINISHED_PATTERN），还没见到的这一轮采到的进程内存收进编译那一段的最大，见到之后（连同见到它的那一轮）不再收；
     两样变了就写进说明里 build_phase_file 那一份（尝试目录下；一行「<编译那一段见过的最大，没采到写 0> <见没见到那一行：0|1>」，人与自证核对读它）。第二台那条命令的编译那一段由
     multi-host-run.sh 照它每一轮的读数另算（那份文件头 ⑤「编译段峰值」），这一层在本机采不到它的进程内存。说明里 build_phase_past_finished 为真时见到那一行之后照收，
     只给弄坏开关 build-phase-past-finished 用；
     本机的 GPU 任务另每 GPU_SAMPLE_EVERY 次读一次 nvidia-smi 的计算进程，把这一条子孙进程的显存相加取最大；
     说明里给了编译目录（build_directory，服务给这一次的 CARGO_TARGET_DIR 所在）的，另一个线程每 DISK_SAMPLE_SECONDS 秒量一次它的占盘
     （各文件 st_blocks × 512 之和，不跟符号链接、不跨文件系统），此刻的数写进尝试目录的 build_bytes（服务算「还要涨」那一截），取最大；子进程退了再量一次；
  ④ 收到 TERM / INT / HUP 只转给自己起的那一个子进程（它是内存包装或清场包装，它们各自停自己那一支），记下收到的是哪个信号；
  ⑤ 子进程退了，编译目录不删（任务有了结局由服务删，task_server_lib.py 文件头「编译目录」）。
  ⑥ 写结果 result.json（先写临时文件再改名）：exit_code（被信号杀的记 128 + 信号号）、started_at、ended_at、seconds、memory_peak（③ 按轮采到的进程内存的最大值；
     按轮一次都没采到（命令比一个采样间隔还短）才取说明里 exit_peak_file 那一份（服务经 RUN_WITH_MEMORY_CAP_PEAK_OUTPUT 交给内存包装，包装在 scope 收尾之前读一次
     scope 的 memory.peak 写进去：进程退了之后读不到进程内存，含页缓存的峰值只给这种短命令兜底）；字节，两样都没有是 null，不记 0；
     说明里 memory_peak_with_page_cache 为真时照改前按轮读 memory.peak、与 exit_peak_file 那一份取大的，只给弄坏开关 memory-peak-with-page-cache 用）、
     cpu_usec（各 scope 的 usage_usec 最大值之和）、cores（cpu_usec ÷ 用时，平均在用核数）、gpu_memory_peak（MiB）、stopped_signal、scope、build_directory_peak（编译目录占盘的最大值，字节；没给编译目录或一次都没量到是 null）、
     build_phase_peak（③ 编译那一段的峰值，字节；子进程退了之后把日志读到底再找一次那一行，日志里没有那一行（编不过、不是 cargo）或那一行之前一轮都没采到是 null）；退出码照子进程的。
     写之前停单元里留下的进程（stop_unit_leftovers）出了没料到的异常的，往日志记一行告警、unit_leftovers 写 null，result.json 照写、退出码照旧
     （records/2026-11-03-task-server与prune-server深入对攻审核.md「TS-02」；说明里 leftovers_error_uncaught 为真时照改前不隔离，只给弄坏开关 leftovers-error-uncaught 用）。
说明读不了、工作目录进不去：退 2，不写结果（服务按单元不在、没留结果记 launcher-lost；编译目录由服务对账时删）。
自证换 scope：说明里 fake_scope_directory 非空时把它当外层 scope 读（假的 memory.stat、memory.peak、cpu.stat）。
"""
import os
import sys

sys.dont_write_bytecode = True

import importlib.util  # noqa: E402
import json  # noqa: E402
import re  # noqa: E402
import signal  # noqa: E402
import subprocess  # noqa: E402
import threading  # noqa: E402
import time  # noqa: E402

SAMPLE_SECONDS = 1.0
FAST_SAMPLE_SECONDS = 0.1
GPU_SAMPLE_EVERY = 5
DISK_SAMPLE_SECONDS = 30.0      # 量编译目录占盘的间隔：几百 G 的编译目录走一遍要几秒（推的），不跟 1 秒一次的 CPU 采样
FAST_DISK_SAMPLE_SECONDS = 0.2
CGROUP_ROOT = "/sys/fs/cgroup"
SCOPE_PREFIX = "singlefs-memory-cap-"
LOG_SCAN_BYTES = 4 * 1024 * 1024   # 找 cargo 编完那一行时一轮至多读这么多日志（文件头 ③「编译那一段的峰值」）：读不完的下一轮接着读


def load_memory_peaks_library():
    """scripts/hosts/memory_peaks_lib.py（进程内存的读法 process_memory_bytes 只在那一份，不另写）：按这一份所在的仓根取，import 不起子进程、不开文件。
    → (模块, 载不进的原因)；载不进（那一份正被改坏、不在）不挡命令：模块交 None，这一次按轮采不到进程内存，main 往日志记一行。"""
    path = os.path.join(os.path.dirname(os.path.dirname(os.path.realpath(__file__))), "scripts", "hosts", "memory_peaks_lib.py")
    try:
        specification = importlib.util.spec_from_file_location("memory_peaks_lib", path)
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
    except Exception as error:   # 载库出事只是少一个数，命令照起
        return None, f"{path} 载不进（{type(error).__name__}: {error}）"
    return module, ""


def wake_server(socket_path):
    """写完结果之后叫醒服务的主循环（POST /wake）；调不通不报错，最晚一轮主循环后对账照样记下（task_server.py 文件头「通知」）。"""
    import socket
    try:
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as connection:
            connection.settimeout(5)
            connection.connect(socket_path)
            connection.sendall(b"POST /wake HTTP/1.1\r\nHost: task-server\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
            connection.recv(1024)
    except OSError:
        pass


def write_json_atomically(path, value):
    temporary = f"{path}.{os.getpid()}.partial"
    descriptor = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
    with os.fdopen(descriptor, "w", encoding="utf-8") as handle:
        json.dump(value, handle, ensure_ascii=False)
    os.replace(temporary, path)


def write_text_atomically(path, text):
    temporary = f"{path}.{os.getpid()}.partial"
    descriptor = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
    with os.fdopen(descriptor, "w", encoding="utf-8") as handle:
        handle.write(text)
    os.replace(temporary, path)


def descendants_with_depth(root_process_id):
    """root 的子孙进程号 → 离 root 几层（照 /proc/<号>/stat 的父进程号一层层往下，不按名字找）。"""
    children = {}
    for name in os.listdir("/proc"):
        if not name.isdigit():
            continue
        try:
            with open(f"/proc/{name}/stat", encoding="utf-8") as handle:
                parent = int(handle.read().rsplit(")", 1)[1].split()[1])
        except (OSError, IndexError, ValueError):
            continue
        children.setdefault(parent, []).append(int(name))
    depth = {}
    frontier = [(root_process_id, 0)]
    while frontier:
        current, level = frontier.pop()
        for child in children.get(current, []):
            if child not in depth:
                depth[child] = level + 1
                frontier.append((child, level + 1))
    return depth


def cgroup_of(process_id):
    try:
        with open(f"/proc/{process_id}/cgroup", encoding="utf-8") as handle:
            for line in handle:
                if line.startswith("0::"):
                    return line[3:].strip()
    except OSError:
        return ""
    return ""


UNIT_LEFTOVER_STOP_SECONDS = 10   # 停单元里留下的进程之后至多等这么久回读 cgroup.procs


def stop_unit_leftovers(unit, unit_unchecked=False):
    """命令正常退出之后，这一次尝试的单元 cgroup 里除了自己还有进程（命令起的孙进程留下）：逐个 SIGKILL、至多等 UNIT_LEFTOVER_STOP_SECONDS 秒回读。
    自己退了 systemd 就收单元、当场杀掉它们，服务那时回读不到，所以在这里停、记下来交给服务写结局说明（task_server.py 的 stop_leftovers）。
    只在自己待在说明点名的那个单元（unit）里时停：没起成单元（弄坏开关 no-detach，或单元名没给）时自己的 cgroup 是起服务的那一方的，里面的进程不归这一次尝试，
    一个不停（不然连起服务的门禁一起杀掉：records/2026-10-31-task-server巡检.md 任务 5662）。unit_unchecked（说明里的 leftovers_unit_unchecked，
    弄坏开关 leftovers-outside-own-unit）下照改前不核单元。
    → {"processes": 留下的进程号, "remaining": 回读还在的, "verified": 回读成功}；读不了不能当空。"""
    group = cgroup_of(os.getpid())
    if not group:
        return {"processes": [], "remaining": None, "verified": False}
    if (not unit or os.path.basename(group) != unit) and not unit_unchecked:
        return {"processes": [], "remaining": [], "verified": True, "skipped": f"自己的 cgroup {group} 不是这一次尝试的单元 {unit or '（没给）'}，不停"}
    procs = os.path.join(CGROUP_ROOT + group, "cgroup.procs")

    def others():
        try:
            with open(procs, encoding="utf-8") as handle:
                return [line.strip() for line in handle if line.strip() and line.strip() != str(os.getpid())]
        except OSError:
            return None
    left = others()
    if left is None:
        return {"processes": [], "remaining": None, "verified": False}
    if not left:
        return {"processes": [], "remaining": [], "verified": True}
    for process in left:
        try:
            os.kill(int(process), signal.SIGKILL)
        except (OSError, ValueError):
            pass
    deadline = time.monotonic() + UNIT_LEFTOVER_STOP_SECONDS
    remaining = others()
    while remaining and time.monotonic() < deadline:
        time.sleep(0.2)
        remaining = others()
    return {"processes": left, "remaining": remaining, "verified": remaining is not None}


def read_integer(path):
    try:
        with open(path, encoding="utf-8") as handle:
            return int(handle.read().strip())
    except (OSError, ValueError):
        return None


def read_usage_usec(directory):
    try:
        with open(os.path.join(directory, "cpu.stat"), encoding="utf-8") as handle:
            for line in handle:
                name, _, value = line.partition(" ")
                if name == "usage_usec" and value.strip().isdigit():
                    return int(value)
    except OSError:
        return None
    return None


def directory_disk_bytes(path):
    """path 底下各文件实际占的盘（st_blocks × 512，lstat 不跟符号链接、不跨进别的文件系统）；目录不在交 None，读不到的那几项跳过。"""
    try:
        device = os.lstat(path).st_dev
    except OSError:
        return None
    total, pending = 0, [path]
    while pending:
        current = pending.pop()
        try:
            entries = list(os.scandir(current))
        except OSError:
            continue
        for entry in entries:
            try:
                status = entry.stat(follow_symlinks=False)
            except OSError:
                continue
            total += status.st_blocks * 512
            if entry.is_dir(follow_symlinks=False) and status.st_dev == device:
                pending.append(entry.path)
    return total


class BuildDiskSampler(threading.Thread):
    """编译目录占盘（文件头 ③）：每 interval 秒量一次，此刻的数写进尝试目录的 build_bytes，peak 取最大。"""

    def __init__(self, build_directory, attempt_directory, interval):
        super().__init__(name="build-disk-sampler", daemon=True)
        self.build_directory = build_directory
        self.output = os.path.join(attempt_directory, "build_bytes")
        self.interval = interval
        self.stopped = threading.Event()
        self.peak = None

    def measure(self):
        size = directory_disk_bytes(self.build_directory)
        if size is None:
            return
        self.peak = max(size, self.peak or 0)
        write_text_atomically(self.output, f"{size}\n")

    def run(self):
        while not self.stopped.is_set():
            try:
                self.measure()
            except Exception:   # 量盘出事不挡命令：少一个数，结局照记
                pass
            self.stopped.wait(self.interval)


class Sampler:
    def __init__(self, specification, root_process_id, peaks):
        self.specification = specification
        self.peaks = peaks   # memory_peaks_lib（load_memory_peaks_library；载不进是 None，按轮采不到进程内存）
        self.root_process_id = root_process_id
        self.fake_scope = specification.get("fake_scope_directory") or ""
        self.scope_file = os.path.join(specification["directory"], "scope")
        self.outer_scope = ""
        self.memory_peak = None
        self.with_page_cache = bool(specification.get("memory_peak_with_page_cache"))   # 弄坏开关 memory-peak-with-page-cache（文件头 ⑥）
        self.peak_file = specification.get("memory_peak_file") or ""   # 进程内存见过的最大写到哪（文件头 ③；task_server.py 的 scope_memory_peak 读它）
        self.usage = {}
        self.gpu_memory_peak = None
        self.rounds = 0
        # 编译那一段的峰值（文件头 ③）：日志读到哪、见没见到 cargo 编完那一行、那一行之前采到的最大
        pattern = specification.get("build_finished_pattern") or ""
        self.finished_line = re.compile(pattern.encode("utf-8")) if pattern else None
        self.log_path = specification.get("log") or ""
        self.log_offset = 0
        self.build_finished = False
        self.build_phase_peak = None
        self.build_phase_written = None
        self.build_phase_past_finished = bool(specification.get("build_phase_past_finished"))   # 弄坏开关 build-phase-past-finished（文件头 ③）
        self.build_phase_file = specification.get("build_phase_file") or ""   # 编译那一段的最大与见没见到那一行写到哪（文件头 ③）

    def scan_build_log(self):
        """从上一次读到的位置接着读日志的整行（至多 LOG_SCAN_BYTES；没有换行的半行留到下一回），见到 cargo 编完那一行就记下、不再读。→ 这一回读了几个字节。"""
        if self.finished_line is None or self.build_finished or not self.log_path:
            return 0
        try:
            with open(self.log_path, "rb") as handle:
                handle.seek(self.log_offset)
                chunk = handle.read(LOG_SCAN_BYTES)
        except OSError:
            return 0
        complete = chunk[:chunk.rfind(b"\n") + 1] if len(chunk) < LOG_SCAN_BYTES else chunk
        for line in complete.splitlines():
            if self.finished_line.search(line):
                self.build_finished = True
                break
        self.log_offset += len(complete)
        return len(complete)

    def note_build_phase(self, measured):
        """这一轮采到的进程内存收进编译那一段的最大（这一轮之前已见到 cargo 编完那一行的不收；弄坏开关下照收），两样变了写 build_phase。"""
        if measured and (not self.build_finished or self.build_phase_past_finished):
            self.build_phase_peak = max(measured, self.build_phase_peak or 0)
        state = (self.build_phase_peak or 0, int(self.build_finished))
        if state != self.build_phase_written and self.build_phase_file:
            write_text_atomically(self.build_phase_file, f"{state[0]} {state[1]}\n")
            self.build_phase_written = state

    def scopes(self):
        if self.fake_scope:   # 自证：一份或几份（os.pathsep 隔开，第一份是外层、后面的是里层包装各开的平级 scope）
            return {directory: level for level, directory in enumerate(self.fake_scope.split(os.pathsep))}
        found = {}
        own_group = cgroup_of(os.getpid())   # 和自己同一层的不是内存包装给这一次开的 scope（没起成单元时那是起服务的那一方的）
        for process_id, level in descendants_with_depth(self.root_process_id).items():
            group = cgroup_of(process_id)
            if group != own_group and os.path.basename(group).startswith(SCOPE_PREFIX) and group.endswith(".scope"):
                directory = CGROUP_ROOT + group
                found[directory] = min(level, found.get(directory, level))
        return found

    def sample(self):
        self.rounds += 1
        self.scan_build_log()   # 先找 cargo 编完那一行：见到它的这一轮采的不算编译那一段（文件头 ③）
        measured = None
        scopes = self.scopes()
        if scopes:
            outer = min(scopes, key=lambda directory: (scopes[directory], directory))
            if outer != self.outer_scope:
                self.outer_scope = outer
                write_text_atomically(self.scope_file, outer + "\n")
            # 进程内存（文件头 ③）；弄坏开关 memory-peak-with-page-cache 下照改前读 memory.peak（含页缓存）
            if self.with_page_cache:
                measured = read_integer(os.path.join(outer, "memory.peak"))
            else:
                measured = self.peaks.process_memory_bytes(outer) if self.peaks is not None else None
            if measured and measured > (self.memory_peak or 0):   # 0 是 scope 已收尾、None 是读不了：都不算采到
                self.memory_peak = measured
                if self.peak_file:
                    write_text_atomically(self.peak_file, f"{measured}\n")
            for directory in scopes:
                usage = read_usage_usec(directory)
                if usage is not None:
                    self.usage[directory] = max(usage, self.usage.get(directory, 0))
            if self.usage:   # 跑着时的 CPU 合计：服务判「在算」与报在用核数读它（与结局的 cpu_usec 同一个口径）
                write_text_atomically(os.path.join(self.specification["directory"], "cpu_usec"), f"{sum(self.usage.values())}\n")
        self.note_build_phase(measured)
        if self.specification.get("gpu") and self.specification.get("host") == "local" and self.rounds % GPU_SAMPLE_EVERY == 1:
            self.sample_gpu()

    def sample_gpu(self):
        try:
            completed = subprocess.run([self.specification.get("nvidia_smi") or "nvidia-smi", "--query-compute-apps=pid,used_memory", "--format=csv,noheader,nounits"],
                                       capture_output=True, text=True, stdin=subprocess.DEVNULL, timeout=30)
        except (OSError, subprocess.TimeoutExpired):
            return
        if completed.returncode != 0:
            return
        family = set(descendants_with_depth(self.root_process_id)) | {self.root_process_id}
        used = 0
        for line in completed.stdout.splitlines():
            fields = [piece.strip() for piece in line.split(",")]
            if len(fields) == 2 and fields[0].isdigit() and fields[1].isdigit() and int(fields[0]) in family:
                used += int(fields[1])
        if used:
            self.gpu_memory_peak = max(used, self.gpu_memory_peak or 0)


def main(arguments):
    if len(arguments) != 1:
        print("  ✗ attempt_runner.py 用法：attempt_runner.py <说明 spec.json>（只由 task-server 起）", file=sys.stderr)
        print("  → 怎么办：提交任务用 task-server/task submit -- <命令…>", file=sys.stderr)
        return 2
    try:
        with open(arguments[0], encoding="utf-8") as handle:
            specification = json.load(handle)
        os.chdir(specification["cwd"])
        directory = specification["directory"]
        log = open(specification["log"], "ab")
    except (OSError, ValueError, KeyError) as error:
        print(f"  ✗ attempt_runner.py：说明 {arguments[0]} 读不了或工作目录进不去（{type(error).__name__}: {error}），命令没起", file=sys.stderr)
        print("  → 怎么办：这一层只由 task-server 写的说明起；看服务的日志（journalctl --user -u singlefs-task-server.service）", file=sys.stderr)
        return 2
    write_text_atomically(os.path.join(directory, "started"), f"{os.getpid()}\n")
    started_at = time.time()
    stopped = {"signal": 0}
    child = subprocess.Popen(specification["argv"], cwd=specification["cwd"], env=specification["environment"], stdin=subprocess.DEVNULL, stdout=log, stderr=log)

    def forward(signal_number, _frame):
        stopped["signal"] = signal_number
        if child.poll() is None:
            child.send_signal(signal_number)   # 只发给自己起的那一个子进程
    for signal_number in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
        signal.signal(signal_number, forward)
    peaks, peaks_problem = load_memory_peaks_library()
    if peaks_problem:
        log.write(f"  ! attempt_runner.py：{peaks_problem}，这一次按轮采不到进程内存，峰值只剩内存包装退出时读的那一个（命令照跑）\n".encode("utf-8"))
    sampler = Sampler(specification, child.pid, peaks)
    interval = FAST_SAMPLE_SECONDS if specification.get("fast") else SAMPLE_SECONDS
    disk_sampler = None
    if specification.get("build_directory"):
        disk_sampler = BuildDiskSampler(specification["build_directory"], directory,
                                        FAST_DISK_SAMPLE_SECONDS if specification.get("fast") else DISK_SAMPLE_SECONDS)
        disk_sampler.start()
    while child.poll() is None:
        try:
            sampler.sample()
        except Exception as error:   # 采样出事不挡命令：记一行进日志，接着跑
            log.write(f"  ! attempt_runner.py：这一次采样出了事（{type(error).__name__}: {error}），接着跑\n".encode("utf-8"))
        time.sleep(interval)
    if sampler.memory_peak is None:
        try:   # 一次都没采到（命令比一个间隔还短）：scope 多半已经收尾，读得到就补一次
            sampler.sample()
        except Exception:   # 补采出事只是少一个数，不改结局
            pass
    exit_peak = read_integer(specification["exit_peak_file"]) if specification.get("exit_peak_file") else None
    # 内存包装退出时读的那一个（文件头 ⑥）：按轮一次都没采到才用它；弄坏开关 memory-peak-with-page-cache 下照改前与按轮采到的取大的
    if exit_peak is not None and exit_peak > 0 and (sampler.memory_peak is None or sampler.with_page_cache):
        sampler.memory_peak = max(exit_peak, sampler.memory_peak or 0)
    try:   # 编译那一段（文件头 ③）：子进程退了，把日志读到底再找一次 cargo 编完那一行（最后一轮之后才打出来的）
        while sampler.scan_build_log():
            pass
    except Exception:   # 读日志出事只是少一个数，不改结局
        pass
    if disk_sampler is not None:
        disk_sampler.stopped.set()
        disk_sampler.join()
        try:   # 子进程退了再量一次（编译目录留着，这时候的数不比跑着时小就是峰值）
            disk_sampler.measure()
        except Exception:   # 量盘出事只是少一个数，不改结局
            pass
    code = child.returncode
    exit_code = 128 - code if code < 0 else code
    ended_at = time.time()
    seconds = round(ended_at - started_at, 3)
    cpu_usec = sum(sampler.usage.values()) if sampler.usage else None
    unit_leftovers = None
    if not stopped["signal"] and not specification.get("leftovers_kept"):   # 正常结束（没被停）才停留下的；弄坏开关 leftover-processes-kept 下不停
        try:   # 停留下的进程出了没料到的事也照写 result.json（文件头 ⑥）：记一行告警、unit_leftovers 留 None，服务照「回读不成」那一支自己重查单元
            unit_leftovers = stop_unit_leftovers(specification.get("unit") or "", bool(specification.get("leftovers_unit_unchecked")))
        except Exception as error:
            if specification.get("leftovers_error_uncaught"):   # 弄坏开关 leftovers-error-uncaught：照改前不隔离，异常一直抛出去、result.json 写不出
                raise
            log.write(f"  ! attempt_runner.py：停单元里留下的进程时出了事（{type(error).__name__}: {error}），结果照写，留下的进程交服务重查\n".encode("utf-8"))
            unit_leftovers = None
    write_json_atomically(os.path.join(directory, "result.json"), {
        "runner_pid": os.getpid(),
        "unit_leftovers": unit_leftovers,
        "exit_code": exit_code, "started_at": round(started_at, 3), "ended_at": round(ended_at, 3), "seconds": seconds,
        "memory_peak": sampler.memory_peak, "cpu_usec": cpu_usec,
        "cores": round(cpu_usec / 1e6 / seconds, 3) if cpu_usec is not None and seconds > 0 else None,
        "gpu_memory_peak": sampler.gpu_memory_peak, "stopped_signal": stopped["signal"], "scope": sampler.outer_scope,
        "build_phase_peak": sampler.build_phase_peak if sampler.build_finished else None,
        "build_directory_peak": disk_sampler.peak if disk_sampler is not None else None})
    log.close()
    if specification.get("wake_socket") and not specification.get("wake_skipped"):
        wake_server(specification["wake_socket"])
    return exit_code


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
