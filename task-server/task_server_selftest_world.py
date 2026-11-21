"""task_server_selftest.py 拆出的一份：自证的世界（World：假的服务环境）、等待与读数的小工具，以及几个分组共用的辅助（总说明在 task_server_selftest.py 文件头）。"""
import os
import sys
import contextlib
import json
import re
import shlex
import shutil
import signal
import socket
import sqlite3
import subprocess
import tempfile
import threading
import time
import types
import task_server_lib as lib
from task_server_selftest_fakes import (
    CLIENT_GROUP_STOP_SECONDS, CONFIRM_BUDGET_SECONDS, CONFIRM_RETRIES, DEFAULT_INBOX_TOKEN, FAKE_CARGO, FAKE_GPU_SERVICE_CONTROL,
    FAKE_MEMORY_WRAPPER, FAKE_PEER_RUNNER, FAKE_PRUNE_CHECK, FAKE_SHARDED, FAKE_SSH, FAKE_SYSTEMCTL, FAKE_SYSTEMD_RUN_DELAYED,
    FAKE_SYSTEMD_RUN_REFUSING, FAKE_SYSTEMD_RUN_SELECTIVE, FAKE_SYSTEMD_RUN_SILENT, FAKE_TEMPORARY_SERVICE, GIBIBYTE, MEBIBYTE, PEAKS, POLL_SECONDS,
    PROGRESS_IGNORED_FILES, PROGRESS_SAMPLE_SECONDS, SEEDED_ENDED_AT, SELFTEST_LOCAL_ADDRESS, SELFTEST_MARGIN_GIBIBYTES, SELFTEST_MULTI_HOST_KEYS,
    SELFTEST_PEER_ADDRESS, SELFTEST_SLICE_CAP_TEXT, SERVER, SESSION_A, SOCKET_ROOT_VARIABLE, STATE_TIMEOUT_SECONDS, WAIT_CEILING_FACTOR,
    WAIT_SLICE_SECONDS, WORLD_SOCKET_PREFIX,
)


def process_memory_stat(anon_bytes, file_bytes=0, kernel_bytes=0):
    """假 scope 的 memory.stat（attempt_runner.py 文件头 ③：进程内存读的是 anon 加 shmem）：anon、file、kernel 照给，shmem 0，末尾带换行。"""
    return f"anon {anon_bytes}\nfile {file_bytes}\nkernel {kernel_bytes}\nshmem 0\n"


def by_host_text(local_text, peer_text):
    """SLICE_MEMORY_CAP_BY_HOST 的值：这一台（SELFTEST_LOCAL_ADDRESS）与另一台（SELFTEST_PEER_ADDRESS）各一项。"""
    return f"{SELFTEST_LOCAL_ADDRESS}={local_text} {SELFTEST_PEER_ADDRESS}={peer_text}"


class SelftestFailure(Exception):
    def __init__(self, detail, extra_lines=()):
        super().__init__(detail)
        self.extra_lines = list(extra_lines)


def write_executable(path, text):
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)
    os.chmod(path, 0o755)


# 各格假余量报告里缺省的 CPU（memory_report）：负载 0、核数够大，服务在一台上在跑的线程之和（每条 31）几百条都不到核数 × 2，CPU 闸只红点名的几格
SELFTEST_IDLE_CPU = {"load1": 0.0, "procs_running": 1, "cpus": 4096}


def memory_report(free_gibibytes, slice_gibibytes, slice_used_gibibytes=0, margin_gibibytes=SELFTEST_MARGIN_GIBIBYTES, scopes=None):
    """slice_used_gibibytes：slice 里此刻进程用的内存（memory_peaks_lib 的 headroom 交的 slice_used），slice 放得下的照总上限减它。
    margin_gibibytes：这台此刻的余量（memory_peaks_lib 的 MARGIN_REPORT_KEY，自证喂的定值）；None 演量不到（原因写进 MARGIN_PROBLEM_KEY）。
    scopes：{尝试的 label: 回收不掉的用量（字节）}，写成这台 slice 下各个在跑的 scope（memory_peaks_lib 的 SCOPES_REPORT_KEY，单元名是假的）；不给就不写这一项。"""
    free = int(free_gibibytes * GIBIBYTE)
    slice_total = int(slice_gibibytes * GIBIBYTE)
    slice_used = int(slice_used_gibibytes * GIBIBYTE)
    margin = None if margin_gibibytes is None else int(margin_gibibytes * GIBIBYTE)
    report = {"memory": {"mem_total": 512 * GIBIBYTE, "mem_available": free, "pending": 0, "running": 0, "slice_total": slice_total, "slice_used": slice_used,
                         "free": free, "slice_room": slice_total - slice_used, PEAKS.MARGIN_REPORT_KEY: margin, PEAKS.MARGIN_MEASURED_KEY: margin,
                         PEAKS.MARGIN_PROBLEM_KEY: "" if margin is not None else "自证：这台 slice 的 memory.current 读不了"}}
    report["memory"][PEAKS.CPU_REPORT_KEY] = dict(SELFTEST_IDLE_CPU)   # 读得到的 CPU（读不到的那一台这一轮不派，task_server.py 的 cpu_problem）；local_cpu 给了的格换掉它
    if scopes is not None:
        report["memory"][PEAKS.SCOPES_REPORT_KEY] = {f"singlefs-memory-cap-selftest-{position}.scope": {PEAKS.SCOPE_USED_KEY: int(used), PEAKS.SCOPE_LABEL_KEY: label}
                                                     for position, (label, used) in enumerate(sorted(scopes.items()))}
    return report


class Patience:
    """一次等待的耐心（World.keep_waiting 的包装）：patience = world.patience(秒数)，循环写 while patience.waiting():，出来之后 patience.elapsed() 是等了多久。
    循环里自己有进展信号的（例如服务进程的 CPU 在涨）把它交给 waiting(extra=…)：这个值变了算有进展。"""

    def __init__(self, world, limit):
        self.world = world
        self.limit = limit
        self.started = time.monotonic()
        self.extra = None
        self.extra_moved_at = 0.0

    def waiting(self, extra=None):
        if extra is not None and extra != self.extra:
            self.extra = extra
            self.extra_moved_at = time.monotonic()
        return self.world.keep_waiting(self.started, self.limit, self.extra_moved_at)

    def elapsed(self):
        return time.monotonic() - self.started


class World:
    """一格的假世界：临时目录里的多机配置（判法要的键写全，另写 TASK_SERVER_ 那几个键）、密钥、假包装、两台的假余量与一份真的 task-server。"""

    def __init__(self, cell, root, prepare=True):
        self.cell = cell
        self.root = root
        self.bin = os.path.join(root, "bin")
        self.world = os.path.join(root, "world")
        self.work = os.path.join(root, "work")
        self.home = os.path.join(root, "home")
        self.fake_cargo = os.path.join(root, "fake-cargo")
        # 格里不另给假 scope 时尝试读的那一份：空目录，峰值量不到、回收不掉的用量算 0（task_server_lib.py 文件头「自证」）
        self.empty_scope = os.path.join(root, "empty-scope")
        # 这一格的假 NFS 共享根（多机配置的 NFS_SHARED_ROOTS）就是这一格的临时目录：格里交的命令点到的路径都在它下面，提交时不因共享根外的路径被拒
        # （task_server.py 文件头 ②「提交时拒」）；假 etab 导出它、假 ssh 在本机读写它（task_server_lib.py 文件头「NFS 共享」）
        self.shared_root = root
        # 共享根外的两处放在这一格临时目录的旁边（整份自证的临时目录里，收尾一起删）：假第二台本地盘上的编译目录，与 *-refused 那几格点名的共享根外路径
        self.peer_local = root.rstrip(os.sep) + ".peer-local"
        self.outside_root = root.rstrip(os.sep) + ".outside"
        # 假仓根放在假共享根下（仓也在 NFS 共享根下，第二台在同一棵树里跑：task_server_lib.py 文件头「挑机器」）
        self.repo = os.path.join(self.shared_root, "repo")
        # 这一格服务的工作根与共享数据根（SINGLEFS_WORK_ROOT、SINGLEFS_DATA_ROOT）：放在假共享根下，提交时的核对（task_server.py 的 shared_roots_problem）过得去
        self.work_root = os.path.join(self.shared_root, "work-root")
        self.data_root = os.path.join(self.shared_root, "data-root")
        self.exports = os.path.join(root, "fake-etab")
        self.sessions = os.path.join(root, "sessions")   # 假的会话登记目录（服务照会话号找收件 socket 读它；FakeInbox 往里登）
        for directory in (self.bin, self.world, self.work, self.repo, self.home, self.fake_cargo, self.empty_scope, self.shared_root, self.work_root, self.data_root,
                          self.sessions):
            os.makedirs(directory, exist_ok=True)
        self.write_exports([self.shared_root])
        # 假仓根里的登记表 .claude/gate.d/verdict-library-scripts.tsv：只有表头（带判定库的一行都没有）；读不了的照真的那样判不了、提交时拒，
        # verdict-library-family-refused 那一格自己写行
        registry = os.path.join(self.repo, SERVER.VERDICT_LIBRARY_REGISTRY)
        os.makedirs(os.path.dirname(registry), exist_ok=True)
        with open(registry, "w", encoding="utf-8") as handle:
            handle.write("# 自证的登记表：路径<TAB>自证命令<TAB>判定库放哪<TAB>复用三问格名\n")
        self.state = os.path.join(root, "state")
        # socket 目录放在短的公共目录下（task_server_lib.py 文件头「自证」）：格的临时目录套几层之后接上 /run/server.sock 会超 socket 路径的上限
        self.short_root = tempfile.mkdtemp(prefix=WORLD_SOCKET_PREFIX, dir=lib.selftest_override(SOCKET_ROOT_VARIABLE) or root)
        self.runtime = os.path.join(self.short_root, "run")
        self.key = os.path.join(root, "config", "signing.key")
        self.builds = os.path.join(root, "builds")
        self.config = os.path.join(root, "multi-host.env")
        self.write_config(self.config, self.state, self.runtime, self.key, self.builds, self.shared_root)
        if prepare:
            lib.create_signing_key(self.key)
        fakes = {"fake-wrapper": FAKE_MEMORY_WRAPPER, "fake-peer-runner": FAKE_PEER_RUNNER,
                 "fake-systemd-run-refusing": FAKE_SYSTEMD_RUN_REFUSING, "fake-systemd-run-selective": FAKE_SYSTEMD_RUN_SELECTIVE,
                 "fake-systemctl": FAKE_SYSTEMCTL,
                 "fake-journalctl-earlyoom": FAKE_JOURNALCTL_EARLYOOM, "fake-ssh": FAKE_SSH, "fake-systemd-run-delayed": FAKE_SYSTEMD_RUN_DELAYED,
                 "fake-systemd-run-silent": FAKE_SYSTEMD_RUN_SILENT, "fake-prune-check": FAKE_PRUNE_CHECK, "fake-gpu-service-control": FAKE_GPU_SERVICE_CONTROL}
        for name, text in fakes.items():
            write_executable(os.path.join(self.bin, name), text)
        write_executable(os.path.join(self.fake_cargo, "cargo"), FAKE_CARGO)
        write_executable(os.path.join(self.repo, "fake-sharded.py"), FAKE_SHARDED)
        os.makedirs(os.path.join(self.repo, "scripts", "prune-server"), exist_ok=True)
        write_executable(os.path.join(self.repo, "scripts", "prune-server", lib.TEMPORARY_SERVICE_SCRIPT), FAKE_TEMPORARY_SERVICE)
        self.resources_path = os.path.join(self.world, "resources.json")
        self.resources(local=(30, 30))
        mark = os.environ.get(lib.SELFTEST_MARK_VARIABLE, "")
        common = {lib.MULTI_HOST_CONFIG_VARIABLE: self.config, lib.SELFTEST_MARK_VARIABLE: mark, "TASK_SERVER_SELFTEST_FAST": "1",
                  "TASK_SERVER_SELFTEST_WORLD": self.world, "TASK_SERVER_SELFTEST_ALLOW_TMPFS": "1",
                  "TASK_SERVER_NFS_EXPORTS": self.exports, "TASK_SERVER_PEER_SSH": os.path.join(self.bin, "fake-ssh")}
        if os.environ.get(lib.BREAK_VARIABLE):
            common[lib.BREAK_VARIABLE] = os.environ[lib.BREAK_VARIABLE]
        base = {name: os.environ[name] for name in ("PATH", "HOME", "USER", "LOGNAME", "LANG", "XDG_RUNTIME_DIR", "DBUS_SESSION_BUS_ADDRESS") if name in os.environ}
        self.server_environment = dict(base, **common, **{
            "TASK_SERVER_FAKE_RESOURCES": self.resources_path, "TASK_SERVER_MEMORY_WRAPPER": os.path.join(self.bin, "fake-wrapper"),
            "TASK_SERVER_PEER_RUNNER": os.path.join(self.bin, "fake-peer-runner"),
            "TASK_SERVER_SESSION_BOUNDARY_PID": str(os.getpid()), "TASK_SERVER_FAKE_REPOSITORY_ROOT": self.repo, "TASK_SERVER_FAKE_SCOPE_DIRECTORY": self.empty_scope,
            lib.WORK_ROOT_VARIABLE: self.work_root,   # 共享数据根不设 SINGLEFS_DATA_ROOT：服务照多机配置的 SHARED_DATA_ROOT 取（同真服务的单元环境）
            lib.CPU_OVERSUBSCRIPTION_VARIABLE: "2",   # CPU 闸的倍数，小世界里是 2（线 64）：各格照它搭的；真值由 cpu-gate-line-twice-cores 那一格钉
            lib.YOUNG_BOOKING_VARIABLE: "0",   # 起跑不满两分钟照整份上限记，小世界里不看：各格的任务几秒就跑完；那一条由 running-young-booked-at-cap 套在假调度器上判
            lib.SLICE_ROOM_FLOOR_VARIABLE: "0",   # slice 兜底的下限，小世界里是 0（不判）：各格的小 slice 不被 4G 的下限停派，slice-floor-holds-dispatch 那一格自己判
            lib.CONFIRM_RULE_SELFTEST_VARIABLE: "all",   # 交任务的二次确认只拦长任务（task_server.py 文件头 ⑦），小世界里不分长短一律待确认：各格演确认单、待确认的不进队；long-task-*、short-task-*、confirm-on-queued-task-accepted 几格起服务时写空串照真的判（LONG_TASK_JUDGED）
            lib.UNNAMED_RUN_SELFTEST_VARIABLE: "off",   # 提交时拒 ⑦（启动测试与验证没点名）小世界里不判：各格交的命令演别的规矩；unnamed-* 那几格起服务时写空串照真的判
            lib.MEMORY_FLOOR_VARIABLE: "0",   # 内存上限的下限，小世界里是 0：各格的小上限不被 1024M 的下限抬走，弄坏开关 cap-floor-skipped 与 recorded-cap-floored 只红点名的几格（task_server_lib.py 文件头「只在自证里认的变量」）；那几格写空串照真的
            "RUN_WITH_MEMORY_CAP_PEAKS": self.peak_table, "HOST_DISPATCH_STATE_DIR": os.path.join(root, "host-dispatch"),
            lib.SESSIONS_DIRECTORY_OVERRIDE: self.sessions})
        # 客户端与 control 的 XDG 目录换到这一格的临时目录：多机配置不在、照缺省路径接着判的那一种（弄坏开关 config-missing-ignored）也连不到本机在跑的那一份服务。
        # 客户端读不到收件地址就拒（task 的 submit）：缺省给一个不在的收件 socket 与一个令牌；会话登记目录里没登这一格的会话，服务照会话号找不到，提醒不发、结局发不出（写告警），
        # wait 照取；要收消息的格起 FakeInbox，它把会话登进会话登记目录
        self.client_environment = dict(common, PATH=os.environ.get("PATH", "/usr/bin:/bin"), HOME=self.home,
                                       **{name: os.path.join(root, "xdg", name.lower()) for name in ("XDG_RUNTIME_DIR", "XDG_STATE_HOME", "XDG_CONFIG_HOME", "XDG_DATA_HOME")},
                                       **{lib.INBOX_SOCKET_VARIABLE: os.path.join(self.short_root, "inbox-unregistered.sock"), lib.INBOX_TOKEN_VARIABLE: DEFAULT_INBOX_TOKEN})
        self.server = None
        self.server_logs = 0
        self.background = []
        self.inboxes = []   # 这一格起的假收件 socket（FakeInbox），close 关掉
        self.seeded = []   # 服务起来之前要记进它库里的尝试（seed_peak、seed_duration），起来之后写进去
        self.last_progress = time.monotonic()   # 世界上一次有进展的时刻（keep_waiting 采到记号变了就更新）
        self.progress_sampled_at = 0.0
        self.progress_marked = None
        self.database_digest = ()   # 任务与尝试的状态摘要；库一时读不了（服务正写）就沿用上一回的，不把读不到当成有变化

    @property
    def peak_table(self):
        return os.path.join(self.root, "memory-peaks.tsv")

    # ── 文件 ──
    @staticmethod
    def write_config(path, state, runtime, key, build, shared_root, data_root=True, slice_caps=None, io_pressure_limit=None):
        """整份多机配置：共用取法 scripts/hosts/multi_host_addresses.py 整份判过才给第二台的地址，判法要的键写全（SELFTEST_MULTI_HOST_KEYS），另写 task-server 读的
        TASK_SERVER_ 那几个键。shared_root 给空串就不写 NFS_SHARED_ROOTS 与 SHARED_DATA_ROOT（nfs-share-required 那一格）；data_root=False 不写 SHARED_DATA_ROOT
        （data-root-key-required）；slice_caps 是 SLICE_MEMORY_CAP_BY_HOST 那一行的值（不给是两台都 SELFTEST_SLICE_CAP_TEXT）；
        io_pressure_limit 给了就多写一行 TASK_SERVER_IO_PRESSURE_LIMIT=<值>（不给就不写这一行，服务不按磁盘压力排）。
        数据根是共享根下的 data-root（同 World.data_root）。注释单起一行：.claude/hooks/lib_work_root.py 与多机配置判法读值都不去行尾注释。"""
        hosts = f"{lib.HOSTS_KEY}={SELFTEST_LOCAL_ADDRESS} {SELFTEST_PEER_ADDRESS}\n"
        shared = ""
        if shared_root:
            shared = f"# 自证：假共享根与它下面的共享数据根\n{lib.NFS_ROOTS_KEY}={shared_root}\n"
            if data_root:
                shared += f"{lib.DATA_ROOT_KEY}={os.path.join(shared_root, 'data-root')}\n"
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(f"# task-server 自证这一格的多机配置：判法要的键写全，另写 task-server 读的那几个键\n{hosts}"
                         + SELFTEST_MULTI_HOST_KEYS.format(slice_caps=slice_caps or by_host_text(SELFTEST_SLICE_CAP_TEXT, SELFTEST_SLICE_CAP_TEXT)) + shared
                         + f"TASK_SERVER_STATE_DIRECTORY={state}\nTASK_SERVER_RUNTIME_DIRECTORY={runtime}\n"
                         f"TASK_SERVER_SIGNING_KEY_FILE={key}\nTASK_SERVER_BUILD_DIRECTORY={build}\n"
                         + (f"{lib.IO_PRESSURE_LIMIT_KEY}={io_pressure_limit}\n" if io_pressure_limit is not None else ""))

    def write_exports(self, roots):
        """这一格的假 /var/lib/nfs/etab（TASK_SERVER_NFS_EXPORTS）：每个根一行，第一列是根，写法照真的那样「<目录><TAB><主机>(<选项>)」。"""
        temporary = self.exports + ".partial"
        with open(temporary, "w", encoding="utf-8") as handle:
            handle.writelines(f"{root}\tselftest-peer(rw,sync,wdelay,hide,no_subtree_check)\n" for root in roots)
        os.replace(temporary, self.exports)

    def set_slice_cap(self, text):
        """这一格的多机配置里 SLICE_MEMORY_CAP_BY_HOST 那一行换成两台都写 <text>（钉值现读它）。"""
        self.set_slice_caps(by_host_text(text, text))

    def set_slice_caps(self, value):
        """这一格的多机配置里 SLICE_MEMORY_CAP_BY_HOST 那一行的整值换成 <value>（钉值现读它；判法要这个键、只许一行；少一台、多一项的那几格照原样写）。
        先写临时文件再改名（服务现读，读不到写了一半的）。"""
        temporary = self.config + ".partial"
        self.write_config(temporary, self.state, self.runtime, self.key, self.builds, self.shared_root, slice_caps=value)
        os.replace(temporary, self.config)

    def set_io_pressure_limit(self, text):
        """这一格的多机配置里 TASK_SERVER_IO_PRESSURE_LIMIT 那一行写成 <text>；None 是整行不写（服务不按磁盘压力排）。先写临时文件再改名（服务现读，读不到写了一半的）。"""
        temporary = self.config + ".partial"
        self.write_config(temporary, self.state, self.runtime, self.key, self.builds, self.shared_root, io_pressure_limit=text)
        os.replace(temporary, self.config)

    def resources(self, local=None, peer=None, peer_cap_gibibytes=None, hold=False, path=None, recheck_local=None, recheck_peer=None, local_slice_used=0,
                  peer_down="", peer_disk_bytes=None, peer_df=None, local_disk_bytes=None,
                  local_margin=SELFTEST_MARGIN_GIBIBYTES, peer_margin=SELFTEST_MARGIN_GIBIBYTES, peer_slice_used=0, peer_scopes=None, local_scopes=None, local_cpu=None, local_slice_unstarted=False,
                  local_io_pressure=None, peer_io_pressure=None):
        """写两台的假余量（服务每次现读）；path 给了就写到那里（假第二台跑完时换上的那一份），不给是这一格服务读的那一份。
        local_io_pressure、peer_io_pressure 给了，那一台的报告里带 /proc/pressure/io 的 full avg300（百分数，task_server.py 的 IO_PRESSURE_REPORT_KEY）；不给的报告里没有这一项。
        recheck_local 给了，起之前复核读本机时照它（排队那一轮照 local）；recheck_peer 给了，起之前复核读第二台时照它（不带盘余量，同真的第二台探测：服务照 peer_df 现量）；
        local_slice_used 是本机 slice 里回收不掉的用量（GiB）。
        peer_down 给了（一句原因），演第二台此刻用不了（照真的那样告警）。
        peer_disk_bytes、local_disk_bytes 给了，那一台的报告里带盘余量（task_server.py 的 DISK_REPORT_KEY）；peer_df 给了，演服务在第二台另起的 df 打出的那几行；都不给的服务当盘够。
        local_margin、peer_margin：两台此刻的余量（GiB，缺省 SELFTEST_MARGIN_GIBIBYTES；None 演那台量不到余量）。
        peer_slice_used：第二台 slice 里此刻进程用的内存（GiB）；peer_scopes、local_scopes：那一台报告里在跑的 scope（{尝试的 label: 回收不掉的用量字节}，memory_report 的 scopes）。"""
        value = {"hold": hold, "peer_build_root": self.peer_local}   # 第二台的编译根：假 multi-host-run.sh 在它下面 builds/<标签>/ 建这一趟的编译目录，服务照它记（Resources.build_root_of_peer）
        if peer_df is not None:
            value["df"] = {SERVER.PEER: peer_df}
        if peer_down:
            value["down"] = {SERVER.PEER: peer_down}
        if local:
            value["local"] = memory_report(local[0], local[1], local_slice_used, local_margin, local_scopes)
            if local_slice_unstarted:   # 演本机 slice 还没起：slice_total 是估的（memory_peaks_lib 的 SLICE_STARTED_KEY 为假）
                value["local"]["memory"][PEAKS.SLICE_STARTED_KEY] = False
            if local_cpu is not None:   # 本机报告里的 CPU 负载（memory_peaks_lib 的 CPU_REPORT_KEY；不给的照 SELFTEST_IDLE_CPU，memory_report）
                value["local"]["memory"][PEAKS.CPU_REPORT_KEY] = dict(local_cpu)
            if local_disk_bytes is not None:   # 本机编译目录所在盘的余量（task_server.py 的 DISK_REPORT_KEY）；不给的服务不判本机占盘
                value["local"][SERVER.DISK_REPORT_KEY] = int(local_disk_bytes)
            if local_io_pressure is not None:   # 本机此刻 /proc/pressure/io 的 full avg300（IO_PRESSURE_REPORT_KEY）；不给的服务读不到、不按压力判本机
                value["local"][SERVER.IO_PRESSURE_REPORT_KEY] = float(local_io_pressure)
        if recheck_local:
            value["recheck"] = {"local": memory_report(recheck_local[0], recheck_local[1], margin_gibibytes=local_margin)}
        if recheck_peer:
            value.setdefault("recheck", {})["peer"] = memory_report(recheck_peer[0], recheck_peer[1], margin_gibibytes=peer_margin)
        if peer:
            value["peer"] = memory_report(peer[0], peer[1], peer_slice_used, peer_margin, peer_scopes)
            if peer_disk_bytes is not None:
                value["peer"][SERVER.DISK_REPORT_KEY] = int(peer_disk_bytes)
            if peer_io_pressure is not None:
                value["peer"][SERVER.IO_PRESSURE_REPORT_KEY] = float(peer_io_pressure)
            value["memory_caps"] = {SERVER.PEER: int((peer_cap_gibibytes or peer[1]) * GIBIBYTE)}
        target = path or self.resources_path
        temporary = target + ".partial"
        with open(temporary, "w", encoding="utf-8") as handle:
            json.dump(value, handle)
        os.replace(temporary, target)

    def seed_peak(self, key, byte_count, shard_count=None):
        """往这一格服务自己的库里记一次跑完（succeeded）的尝试、量到 byte_count 的峰值：服务定额度只认自己库里的尝试（task_server_lib.py 文件头「资源记录」）。
        key 是命令键（RUN_WITH_MEMORY_CAP_KEY 的值，或 PEAKS.command_key 换算过的）。分片的片（键是 lib.shard_child_key 写的那种）要给 shard_count：
        定额度只认片数与这一趟相同的记录（task_server.py 的 record_fits_task），记的是那一趟共几片。"""
        fields = {"command_key": key, "outcome": "succeeded", "memory_peak": int(byte_count)}
        if shard_count is not None:
            fields["shard_count"] = shard_count
        self.seed_attempt(fields)

    def seed_duration(self, key, seconds, command):
        """往这一格服务自己的库里记一次跑完的尝试、用时 seconds 秒（峰值量不到，不占定额度的名额）：上次运行时长只认自己库里的尝试。"""
        self.seed_attempt({"command_key": key, "command": list(command), "outcome": "succeeded", "memory_peak": None, "seconds": float(seconds)})

    def seed_attempt(self, fields, metric=lib.PROCESS_MEMORY_PEAK_METRIC):
        """记一次结束了的尝试；metric 是峰值口径一栏（task_server_lib.py 文件头「资源记录」）：缺省照这一版服务记的写，给 None 演批 92 之前记的、没有这一栏的旧行。
        fields 里给了 task_id、number 的照给的记（control key-stats 那几格按任务折重试），没给的记任务 0、第 1 次。"""
        record = {"peak_metric": metric} if metric is not None else {}
        record.update(fields)
        ended_at = float(fields.get("ended_at") or SEEDED_ENDED_AT)   # 要排先后的格自己给 ended_at
        self.seeded.append(dict(record, task_id=fields.get("task_id", 0), number=fields.get("number", 1), host="local", state="ended", session="", ended_at=ended_at,
                                started_at=ended_at - float(fields.get("seconds") or 1)))
        self.flush_seeded()

    def flush_seeded(self):
        """服务在跑、库已建好时把记下的尝试写进库（用这一格的签名密钥签）；服务还没起的等 start_server 之后再写。"""
        if self.server is None or self.server.poll() is not None or not os.path.exists(os.path.join(self.state, lib.DATABASE_NAME)):
            return
        while self.seeded:
            self.insert_signed_row("attempts", self.seeded.pop(0))

    def write_fake_scope(self, peak_bytes):
        """尝试的假 scope（attempt_runner.py 文件头「自证换 scope」）：进程内存（memory.stat 的 anon）与 memory.peak 都写成 peak_bytes（没有页缓存），
        各先写临时文件再改名（采样读不到写了一半的）；交目录。"""
        scope = os.path.join(self.root, "fake-scope")
        os.makedirs(scope, exist_ok=True)
        for name, text in (("memory.stat", process_memory_stat(peak_bytes)), ("memory.peak", f"{peak_bytes}\n")):
            temporary = os.path.join(scope, name + ".partial")
            with open(temporary, "w", encoding="utf-8") as handle:
                handle.write(text)
            os.replace(temporary, os.path.join(scope, name))
        return scope


    def write_spec(self, name, value):
        with open(os.path.join(self.repo, name), "w", encoding="utf-8") as handle:
            json.dump(value, handle)
        return ["python3", "fake-sharded.py", "run", name]

    def sharded_calls(self, kind, **fields):
        """调用记录里假分片入口记的那几行（kind 是 plan、shard、merge、whole），按栏过滤，交 [{栏: 值}]。"""
        rows = []
        for line in self.calls():
            if not line.startswith(f"sharded-{kind} "):
                continue
            row = dict(item.split("=", 1) for item in line.split()[1:] if "=" in item)
            if all(row.get(name) == str(value) for name, value in fields.items()):
                rows.append(row)
        return rows

    def calls(self):
        try:
            with open(os.path.join(self.world, "calls"), encoding="utf-8") as handle:
                return handle.read().splitlines()
        except FileNotFoundError:
            return []

    def calls_of(self, task_id, kind="wrapper"):
        return [line for line in self.calls() if line.startswith(f"{kind} task={task_id} ")]

    def scheduler_reads(self):
        """服务记的排队读数（task_server.py 的 selftest_read）：每行「pass view=…」「read view=… source=… what=…」「predict task=…」「recheck task=…」。"""
        try:
            with open(os.path.join(self.world, "scheduler-reads"), encoding="utf-8") as handle:
                return handle.read().splitlines()
        except FileNotFoundError:
            return []

    def full_passes(self, queued_count, passes):
        """等到排队线程连着排了 passes + 1 轮、每轮都看到 queued_count 条在排；交前 passes 轮的编号（第 passes + 1 轮开了头，前面那几轮的读数都已记完）。"""
        patience = self.patience(STATE_TIMEOUT_SECONDS)
        numbers = []
        while patience.waiting():
            numbers = [int(found.group(1)) for found in (re.match(r"^pass view=(\d+) queued=(\d+)$", line) for line in self.scheduler_reads())
                       if found and int(found.group(2)) == queued_count]
            if len(numbers) > passes:
                return numbers[:passes]
            time.sleep(POLL_SECONDS)
        raise SelftestFailure(f"等了 {STATE_TIMEOUT_SECONDS:.0f} 秒，排队线程只排了 {len(numbers)} 轮看到 {queued_count} 条在排（要 {passes + 1} 轮）", self.server_log_lines())

    def next_pass(self):
        """等排队线程开下一轮：此刻在排的那一轮的读数都已记完。"""
        started = len([line for line in self.scheduler_reads() if line.startswith("pass ")])
        patience = self.patience(STATE_TIMEOUT_SECONDS)
        while patience.waiting():
            if len([line for line in self.scheduler_reads() if line.startswith("pass ")]) > started:
                return
            time.sleep(POLL_SECONDS)
        raise SelftestFailure(f"等了 {STATE_TIMEOUT_SECONDS:.0f} 秒，排队线程没再开一轮", self.server_log_lines())

    def alerts(self):
        try:
            with open(os.path.join(self.state, "alerts.jsonl"), encoding="utf-8") as handle:
                return [json.loads(line) for line in handle if line.strip()]
        except FileNotFoundError:
            return []

    def operations(self):
        try:
            with open(os.path.join(self.state, "operations.jsonl"), encoding="utf-8") as handle:
                return [json.loads(line) for line in handle if line.strip()]
        except FileNotFoundError:
            return []

    # ── 等待（上面常量处「等待的判法」）──
    def progress_mark(self):
        """这一格的世界此刻的进展记号：假世界目录里每份文件的大小（假脚本的调用记录等；每一轮排队都涨的 PROGRESS_IGNORED_FILES 不算：那是服务活着，不是有事发生）、
        尝试的日志与说明、告警与操作日志、各次服务的输出，加任务与尝试的状态摘要。只含有事发生才变的东西，服务空转它不变。"""
        files = []
        for directory in (self.world, os.path.join(self.state, "logs"), os.path.join(self.state, "attempts")):
            for current, _, names in os.walk(directory):
                files.extend(os.path.join(current, name) for name in names if name not in PROGRESS_IGNORED_FILES)
        files.extend(os.path.join(self.state, name) for name in ("alerts.jsonl", "operations.jsonl"))
        files.extend(os.path.join(self.root, f"server-{number}.log") for number in range(1, self.server_logs + 1))
        sizes = []
        for path in sorted(files):
            with contextlib.suppress(OSError):
                sizes.append((path, os.stat(path).st_size))
        database = os.path.join(self.state, lib.DATABASE_NAME)
        if os.path.exists(database):
            with contextlib.suppress(sqlite3.Error):
                with contextlib.closing(sqlite3.connect(f"file:{database}?mode=ro", uri=True, timeout=1)) as connection:
                    self.database_digest = tuple(connection.execute(
                        "SELECT 'task', id, state FROM tasks UNION ALL SELECT 'attempt', id, state FROM attempts").fetchall())
        return tuple(sizes), self.database_digest

    def keep_waiting(self, started, limit, also_moved=0.0):
        """还要不要接着等（各格的等待都问这一处，不逐格写时限）：从 started 起，世界（progress_mark）上一次有进展、或 also_moved 那个时刻之后安静不满 limit 秒，
        且总共没到 limit × WAIT_CEILING_FACTOR 秒。弄坏开关 wait-limit-fixed 下退回写死的时限：从 started 起满 limit 秒就判，不看进展、没有上限的说法（改前的样子）。"""
        now = time.monotonic()
        if lib.broken("wait-limit-fixed"):
            return now - started < limit
        if now - self.progress_sampled_at >= PROGRESS_SAMPLE_SECONDS:
            self.progress_sampled_at = now
            marked = self.progress_mark()
            if marked != self.progress_marked:
                self.progress_marked = marked
                self.last_progress = now
        return now - max(started, self.last_progress, also_moved) < limit and now - started < limit * WAIT_CEILING_FACTOR

    def patience(self, limit):
        return Patience(self, limit)

    def communicate(self, process, limit):
        """等 process 退出、收它的输出（process.communicate 的耐心版，limit 是安静时限）；等不到照 subprocess.TimeoutExpired 抛，调用处原有的 except 照用。"""
        started = time.monotonic()
        while True:
            try:
                return process.communicate(timeout=WAIT_SLICE_SECONDS)
            except subprocess.TimeoutExpired:
                if not self.keep_waiting(started, limit):
                    raise

    def wait_process(self, process, limit):
        """等 process 退出、交退出码（process.wait 的耐心版）；等不到照 subprocess.TimeoutExpired 抛。"""
        started = time.monotonic()
        while True:
            try:
                return process.wait(timeout=WAIT_SLICE_SECONDS)
            except subprocess.TimeoutExpired:
                if not self.keep_waiting(started, limit):
                    raise

    def server_cpu_ticks(self):
        """这一格起的服务进程自己用过的 CPU 时钟数（/proc/<进程号>/stat 的 utime 加 stime）；读不到交 None。服务起来的路上它在涨就是还在忙。"""
        if self.server is None:
            return None
        try:
            with open(f"/proc/{self.server.pid}/stat", encoding="utf-8") as handle:
                text = handle.read()
        except OSError:
            return None
        fields = text[text.rindex(")") + 2:].split()
        return int(fields[11]) + int(fields[12])

    # ── 服务 ──
    def server_log_lines(self):
        lines = []
        for number in range(1, self.server_logs + 1):
            with contextlib.suppress(OSError):
                with open(os.path.join(self.root, f"server-{number}.log"), encoding="utf-8", errors="replace") as handle:
                    lines += handle.read().splitlines()
        return lines[-20:]

    def start_server(self, extra=None, runtime=None):
        """起服务、等它的 /health 回 ok；runtime 是这一回服务的 socket 目录（给了别的多机配置的那一格各自传它的，不给是这一格的）。"""
        self.server_logs += 1
        log = open(os.path.join(self.root, f"server-{self.server_logs}.log"), "ab")
        environment = dict(self.server_environment, **(extra or {}))
        self.server = subprocess.Popen([sys.executable, "-B", lib.SERVER_SCRIPT, "serve"], env=environment, stdin=subprocess.DEVNULL, stdout=log, stderr=log,
                                       start_new_session=True)
        log.close()
        socket_path = os.path.join(runtime or self.runtime, lib.SOCKET_NAME)
        patience = self.patience(STATE_TIMEOUT_SECONDS)
        while patience.waiting(extra=self.server_cpu_ticks()):   # 起来的路上服务进程的 CPU 在涨就是还在忙（机器忙时导入与建库慢）
            if self.server.poll() is not None:
                raise SelftestFailure(f"task-server 起来就退了（退 {self.server.returncode}）", self.server_log_lines())
            with contextlib.suppress(lib.ServerUnreachable):
                status, health = lib.call_server(socket_path, "GET", "/health", timeout=5)
                if status == 200 and health.get("ok"):
                    self.flush_seeded()
                    return
            time.sleep(POLL_SECONDS)
        raise SelftestFailure(f"task-server 起了 {patience.elapsed():.0f} 秒（安静 {STATE_TIMEOUT_SECONDS:.0f} 秒没有进展）/health 还不回 ok", self.server_log_lines())

    def stop_server(self):
        if self.server is None or self.server.poll() is not None:
            return
        self.server.send_signal(signal.SIGTERM)   # 只对自己起的那一个进程号发
        try:
            self.server.wait(timeout=30)
        except subprocess.TimeoutExpired:
            self.server.kill()
            self.server.wait(timeout=10)

    def serve_until_exit(self, config, timeout=STATE_TIMEOUT_SECONDS):
        """拿 config 这一份多机配置起一份服务、等它自己退出（起不来的那几种）：→ (退出码, 它的输出)；它起来了（这一格的 socket 上 /health 回话）
        当场停掉（只对自己起的那一个进程号发）、交 (None, 它的输出)。"""
        self.server_logs += 1
        log_path = os.path.join(self.root, f"server-{self.server_logs}.log")
        with open(log_path, "ab") as log:
            server = subprocess.Popen([sys.executable, "-B", lib.SERVER_SCRIPT, "serve"], env=dict(self.server_environment, **{lib.MULTI_HOST_CONFIG_VARIABLE: config}),
                                      stdin=subprocess.DEVNULL, stdout=log, stderr=log, start_new_session=True)
        code = None
        patience = self.patience(timeout)
        while patience.waiting():
            code = server.poll()
            if code is not None:
                break
            with contextlib.suppress(lib.ServerUnreachable):
                status, _ = lib.call_server(os.path.join(self.runtime, lib.SOCKET_NAME), "GET", "/health", timeout=5)
                if status == 200:
                    break
            time.sleep(POLL_SECONDS)
        if code is None:
            server.send_signal(signal.SIGTERM)
            try:
                server.wait(timeout=30)
            except subprocess.TimeoutExpired:
                server.kill()
                server.wait(timeout=10)
        with open(log_path, encoding="utf-8", errors="replace") as handle:
            return code, handle.read()

    def wait_server_exit(self, timeout=30):
        try:
            return self.wait_process(self.server, timeout)
        except subprocess.TimeoutExpired:
            raise SelftestFailure(f"task-server {timeout} 秒没有照自证造的崩溃退出", self.server_log_lines()) from None

    def api(self, method, path, body=None):
        status, reply = lib.call_server(os.path.join(self.runtime, lib.SOCKET_NAME), method, path, body, timeout=30)
        return status, reply

    def summary(self, task_id):
        status, reply = self.api("GET", f"/tasks/{task_id}")
        if status != 200:
            raise SelftestFailure(f"读任务 {task_id} 回 HTTP {status}：{reply}")
        return reply

    def wait_for(self, task_id, predicate, what, timeout=STATE_TIMEOUT_SECONDS):
        """等任务 task_id 的摘要让 predicate 为真；timeout 是安静时限（上面常量处「等待的判法」）：世界还在变就接着等，安静满 timeout 秒或到绝对上限才判红。"""
        patience = self.patience(timeout)
        summary = None
        while patience.waiting():
            with contextlib.suppress(lib.ServerUnreachable):
                summary = self.summary(task_id)
                if predicate(summary):
                    return summary
            time.sleep(POLL_SECONDS)
        raise SelftestFailure(f"任务 {task_id} 等了 {patience.elapsed():.0f} 秒没{what}（安静时限 {timeout:.0f} 秒），"
                              f"最后一次读到：{json.dumps(summary, ensure_ascii=False)[:600] if summary else '读不到'}", self.server_log_lines())

    def finished(self, task_id, timeout=STATE_TIMEOUT_SECONDS):
        return self.wait_for(task_id, lambda summary: summary["task"].get("state") == "finished", "收尾", timeout)

    def running(self, task_id, count=1):
        return self.wait_for(task_id, lambda summary: len([attempt for attempt in summary["attempts"] if attempt.get("state") == "running"]) >= count
                             and summary["task"].get("state") == "running", "跑起来")

    # ── 客户端与 control ──
    def client(self, arguments, session=SESSION_A, extra=None, forged=None, timeout=90, background=False, script=None, cwd=None):
        """经一个「会话 shell」起客户端（或 control，control 是唯一传 script 的）：那个 bash 的环境里是 session，客户端自己的环境里换成 forged（给了的话）。
        每一条自成一个进程组（start_new_session）：bash、它起的客户端与客户端的子进程同组，超时与收尾经 stop_client 对整组发信号，只停 bash 的话客户端拿着管道写端、communicate 等不到 EOF。"""
        environment = dict(self.client_environment, **(extra or {}))
        environment.pop(lib.SESSION_VARIABLE, None)
        if session:
            environment[lib.SESSION_VARIABLE] = session
        if forged is None:
            inner, shell_name = '"$@"; exit $?', "session-shell"
        else:
            inner, shell_name = 'CLAUDE_CODE_SESSION_ID="$0" "$@"; exit $?', forged
        if script is None:
            target = lib.CLIENT_SCRIPT
        else:
            target = lib.CONTROL_SCRIPT
        argv = ["bash", "-c", inner, shell_name, sys.executable, "-B", target, *arguments]
        if background:
            process = subprocess.Popen(argv, env=environment, cwd=cwd or self.work, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                       text=True, start_new_session=True)
            self.background.append(process)
            return process
        process = subprocess.Popen(argv, env=environment, cwd=cwd or self.work, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
                                   start_new_session=True)
        try:
            stdout, stderr = self.communicate(process, timeout)   # timeout 是安静时限（上面常量处「等待的判法」）
        except subprocess.TimeoutExpired:
            stdout, stderr = self.stop_client(process)
            raise SelftestFailure(f"{' '.join(arguments)[:200]} 跑了 {timeout} 秒没退（安静时限；世界一直在变的话，绝对上限 {timeout * WAIT_CEILING_FACTOR} 秒）", [str(stdout or ""), str(stderr or "")]) from None
        return subprocess.CompletedProcess(argv, process.returncode, stdout, stderr)

    def stop_client(self, process):
        """停 client 起的那一条（对它自己的进程组发 SIGKILL，进程组是那一次 Popen 建的）、收它的输出 → (stdout, stderr)。组外还有进程拿着管道写端的，至多等 CLIENT_GROUP_STOP_SECONDS 秒，交已经读到的。"""
        with contextlib.suppress(ProcessLookupError):
            os.killpg(process.pid, signal.SIGKILL)
        try:
            return process.communicate(timeout=CLIENT_GROUP_STOP_SECONDS)
        except subprocess.TimeoutExpired as error:
            process.poll()
            return tuple(piece.decode("utf-8", "replace") if isinstance(piece, bytes) else (piece or "") for piece in (error.stdout, error.stderr))

    def control(self, arguments, **options):
        return self.client(arguments, script=lib.CONTROL_SCRIPT, **options)

    def submit(self, command, session=SESSION_A, extra=None, forged=None, cwd=None):
        """交一条、照确认单一起确认（confirm_submitted；task_server.py 文件头 ⑦），交任务号。"""
        full_gate = (extra or {}).get("SINGLEFS_RUN_SCOPE") == SCOPE_COMMIT and lib.commit_gate_entry(list(command), cwd or self.work)   # 演提交时那一趟：带全量标识
        completed = self.client(["submit", "--cwd", cwd or self.work, *([] if session else ["--manual"]), *(["--full-gate"] if full_gate else []), "--", *command],
                                session=session, extra=extra, forged=forged)
        found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", completed.stdout)
        if completed.returncode != 0 or not found:
            raise SelftestFailure(f"提交 {command} 没拿到任务号（退 {completed.returncode}）", [completed.stdout, completed.stderr] + self.server_log_lines())
        self.confirm_submitted(completed, session=session, extra=extra, forged=forged)
        return int(found.group(1))

    def confirm_submitted(self, completed, session=SESSION_A, extra=None, forged=None, required=True):
        """照提交回复（或确认被拒时）打的 CONFIRM_NEEDED_MARK 那一行确认（自证世界的帮手：自证造的任务照样要确认，不另设跳过确认的开关）：
        数对不上的（这一会儿同会话别的任务起了或收了）照服务打回的那一行再确认；回复里没有那一行的（弄坏开关 confirm-skipped 下交了就进队）不确认。
        服务核数是在会话锁里算了就比（task_server.py 的 confirm、counts_mismatch），每一回对不上都是同会话又有一条在排在跑的收了（本帮手不在确认期间再交）：
        头一回对不上时服务回的条数就是还能收掉的最多几条，所以重试的回数在 CONFIRM_RETRIES 之外再放宽这么多回（有界；机器忙、几十条一齐收尾时每一回客户端往返比收一条还慢，三回追不上）；
        回数用完了服务打回的还是「数对不上」（打回的回话里还带着新的确认单）的，再追到 CONFIRM_BUDGET_SECONDS 秒为止：对不上的次数跟着同会话别的任务起、收的次数走，机器忙时次数多，不能拿固定回数判。
        弄坏开关 confirm-retries-fixed 下只重试 CONFIRM_RETRIES 回。
        → 最后一次 confirm 的 CompletedProcess（不用确认交 None）；required 为真时确认不成判红。"""
        reply = completed
        allowed, widened = CONFIRM_RETRIES, False
        tries, deadline = 0, time.monotonic() + getattr(self, "confirm_budget_seconds", CONFIRM_BUDGET_SECONDS)
        while tries < allowed or (not lib.broken("confirm-retries-fixed") and reply is not completed and reply.returncode != 0 and time.monotonic() < deadline):
            tries += 1
            needed = lib.confirm_needed_fields(reply.stdout)
            if needed is None:
                return None if reply is completed else reply
            task_id, items, tasks = needed
            if not widened and not lib.broken("confirm-retries-fixed"):
                allowed, widened = allowed + tasks, True
            reply = self.client(["confirm", str(task_id), "--items", str(items), "--tasks", str(tasks)], session=session, extra=extra, forged=forged)
            if reply.returncode == 0:
                return reply
        if required:
            raise SelftestFailure(f"照确认单确认了 {tries} 回都没成（最后退 {reply.returncode}）", [completed.stdout, reply.stdout, reply.stderr] + self.server_log_lines())
        return reply

    def register_session(self, session, socket_path, name, pid=None, started_milliseconds=None):
        """往假的会话登记目录写一份 <name>.json（同 Claude Code 每个进程写的那一份：pid、sessionId、messagingSocketPath、startedAt）；pid 缺省是这一格的进程（活着）。"""
        record = {"pid": pid or os.getpid(), "sessionId": session, "messagingSocketPath": socket_path,
                  "startedAt": started_milliseconds if started_milliseconds is not None else int(time.time() * 1000)}
        temporary = os.path.join(self.sessions, name + ".json.partial")
        with open(temporary, "w", encoding="utf-8") as handle:
            json.dump(record, handle)
        os.replace(temporary, os.path.join(self.sessions, name + ".json"))

    def unit_active(self, unit):
        if not unit:
            return False
        completed = subprocess.run(["systemctl", "--user", "show", "-p", "ActiveState", "--value", unit], capture_output=True, text=True, stdin=subprocess.DEVNULL,
                                   timeout=30)
        return completed.stdout.strip() in lib.RUNNING_UNIT_STATES

    def database_units(self):
        path = os.path.join(self.state, lib.DATABASE_NAME)
        if not os.path.exists(path):
            return []
        with contextlib.closing(sqlite3.connect(path, timeout=30)) as connection:
            rows = connection.execute("SELECT record FROM attempts").fetchall()
        units = []
        for (text,) in rows:
            with contextlib.suppress(ValueError, AttributeError):
                if json.loads(text).get("unit"):
                    units.append(json.loads(text)["unit"])
        return units

    def rewrite_signed_task(self, task_id, change):
        """服务停着时用这一格的签名密钥改写库里一条任务（演「旧规则下交进来、还在排」）：change 就地改 record。"""
        self.rewrite_signed_row("tasks", task_id, change)

    def rewrite_signed_row(self, table, row_id, change):
        """用这一格的签名密钥改写库里一行（tasks、attempts、history）：change 就地改 record；record 里的 state、session、command_key 照写进几列、重签。"""
        key = lib.read_signing_key(self.key)
        with contextlib.closing(sqlite3.connect(os.path.join(self.state, lib.DATABASE_NAME), timeout=30, isolation_level=None)) as connection:
            connection.execute("BEGIN IMMEDIATE")
            record = json.loads(connection.execute(f"SELECT record FROM {table} WHERE id = ?", (row_id,)).fetchone()[0])
            change(record)
            self.write_signed(connection, key, table, row_id, record)
            connection.execute("COMMIT")

    def insert_signed_row(self, table, record):
        """用这一格的签名密钥往库里写一行（演「改键之前记下的行」、同一次号再插一行）；交它的行号。"""
        key = lib.read_signing_key(self.key)
        with contextlib.closing(sqlite3.connect(os.path.join(self.state, lib.DATABASE_NAME), timeout=30, isolation_level=None)) as connection:
            connection.execute("BEGIN IMMEDIATE")
            cursor = connection.execute(f"INSERT INTO {table}(task_id, state, session, command_key, record, signature) VALUES(?, '', '', '', '{{}}', '')",
                                        (record.get("task_id", 0),))
            self.write_signed(connection, key, table, cursor.lastrowid, dict(record, id=cursor.lastrowid))
            connection.execute("COMMIT")
        return cursor.lastrowid

    @staticmethod
    def write_signed(connection, key, table, row_id, record):
        text = lib.canonical_json(record)
        state, session, command_key = record.get("state", ""), record.get("session", ""), record.get("command_key", "")
        connection.execute(f"UPDATE {table} SET state = ?, session = ?, command_key = ?, record = ?, signature = ? WHERE id = ?",
                           (state, session, command_key, text, lib.row_signature(key, table, row_id, state, session, command_key, text), row_id))

    def signed_rows(self, table):
        """[(record, 校验过没有)]：照服务读回的规则核（校验列、正文里的号与几列对得上）。"""
        key = lib.read_signing_key(self.key)
        with contextlib.closing(sqlite3.connect(os.path.join(self.state, lib.DATABASE_NAME), timeout=30)) as connection:
            rows = connection.execute(f"SELECT id, state, session, command_key, record, signature FROM {table} ORDER BY id").fetchall()
        checked = []
        for row_id, state, session, command_key, text, signature in rows:
            record = json.loads(text)
            valid = (signature == lib.row_signature(key, table, row_id, state, session, command_key, text) and record.get("id") == row_id
                     and record.get("state", "") == state and record.get("session", "") == session and record.get("command_key", "") == command_key)
            checked.append((record, valid))
        return checked

    def insert_bypass_row(self, record):
        """绕过服务直接往库里写一行任务（校验列是乱填的）；交它的行号。"""
        with contextlib.closing(sqlite3.connect(os.path.join(self.state, lib.DATABASE_NAME), timeout=30, isolation_level=None)) as connection:
            connection.execute("BEGIN IMMEDIATE")
            cursor = connection.execute("INSERT INTO tasks(task_id, state, session, command_key, record, signature) VALUES(0, 'queued', ?, 'selftest-bypass', '{}', '')",
                                        (record["session"],))
            record["id"] = cursor.lastrowid
            connection.execute("UPDATE tasks SET record = ?, signature = ? WHERE id = ?", (lib.canonical_json(record), "0" * 64, record["id"]))
            connection.execute("COMMIT")
        return record["id"]

    def close(self):
        for inbox in self.inboxes:
            inbox.close()
        for process in self.background:
            if process.poll() is None:
                self.stop_client(process)
        self.stop_server()
        with contextlib.suppress(OSError, sqlite3.Error, subprocess.SubprocessError):
            for unit in self.database_units():
                if self.unit_active(unit):
                    subprocess.run(["systemctl", "--user", "stop", unit], capture_output=True, stdin=subprocess.DEVNULL, timeout=200)
        shutil.rmtree(self.short_root, ignore_errors=True)
        shutil.rmtree(self.root, ignore_errors=True)


def server_modules():
    """服务拆成的各份模块：入口 SERVER（按路径载入的 task_server.py）加同目录里已经 import 进来、名字以 task_server_ 起头的各份（库与自证本身的各份不算）。"""
    own = os.path.realpath(lib.SCRIPT_DIRECTORY)
    others = [module for name, module in sorted(sys.modules.items())
              if name.startswith("task_server_") and not name.startswith(("task_server_lib", "task_server_selftest"))
              and str(getattr(module, "__file__", "") or "").endswith(".py") and os.path.dirname(os.path.realpath(module.__file__)) == own]
    return [SERVER] + others


def describe(summary):
    task = summary["task"]
    return (f"结局 {task.get('outcome')}、退 {task.get('exit_code')}、尝试 "
            + "、".join(f"{attempt.get('host')} {attempt.get('memory_cap_text')} {attempt.get('outcome')}" for attempt in summary["attempts"]))


# ── 发消息给会话（task_server.py 文件头 ⑥）──
INBOX_TOKEN_SAMPLE = "selftest-inbox-token-5f0c2a9e"   # 假会话令牌：只该出现在收件 socket 收到的认证行里，不该出现在 status、日志与服务的输出里
INBOX_WAIT_SECONDS = 20.0     # 等一条消息的安静时限（发消息那一路自证里每 FAST_INBOX_ROUND_SECONDS 0.5 秒一轮）
INBOX_QUIET_SECONDS = 2.0     # 收到一条之后再看这么久（四轮），同一样不该再多收一条


class FakeInbox:
    """假的会话收件 socket（task_server.py 文件头 ⑥）：这一格 socket 目录里的一个 unix socket，一个线程收连接；一个连接是一条消息，收到的整行逐行记下，
    同时追加进假世界目录里的 inbox-<名>（世界有进展的记号）。令牌是 INBOX_TOKEN_SAMPLE 接名字。World.close 关它。"""

    def __init__(self, world, name, session=SESSION_A, token=None, register=True):
        self.path = os.path.join(world.short_root, f"inbox-{name}.sock")
        self.record = os.path.join(world.world, f"inbox-{name}")
        self.token = token or f"{INBOX_TOKEN_SAMPLE}-{name}"
        self.session = session
        self.connections = []
        self.lock = threading.Lock()
        self.closing = threading.Event()
        self.listener = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.listener.bind(self.path)
        self.listener.listen(16)
        self.listener.settimeout(0.2)
        self.thread = threading.Thread(target=self.serve, name=f"fake-inbox-{name}", daemon=True)
        self.thread.start()
        world.inboxes.append(self)
        if register:   # 把这个会话登进假的会话登记目录：服务照会话号找到这个 socket（task_server.py 文件头 ⑥「收件地址」）
            world.register_session(session, self.path, f"inbox-{name}")

    def environment(self):
        """交给 task submit 的两个变量（同 Claude Code 导出给 Bash 命令的）。"""
        return {lib.INBOX_SOCKET_VARIABLE: self.path, lib.INBOX_TOKEN_VARIABLE: self.token}

    def serve(self):
        while not self.closing.is_set():
            try:
                connection, _ = self.listener.accept()
            except socket.timeout:
                continue
            except OSError:
                return
            data = b""
            with connection:
                connection.settimeout(5)
                with contextlib.suppress(OSError):
                    while True:
                        chunk = connection.recv(65536)
                        if not chunk:
                            break
                        data += chunk
            lines = data.decode("utf-8", "replace").splitlines()
            with self.lock:
                self.connections.append(lines)
            with open(self.record, "a", encoding="utf-8") as handle:
                handle.write(json.dumps(lines, ensure_ascii=False) + "\n")

    def messages(self):
        """[{auth, text, raw}]：每个连接一条；auth 是恰好两行、头一行是认证行且令牌对得上，text 是 user 那一行的正文（读不出交空串），raw 是收到的原行。"""
        with self.lock:
            connections = list(self.connections)
        found = []
        for lines in connections:
            parsed = []
            for line in lines:
                try:
                    parsed.append(json.loads(line))
                except ValueError:
                    parsed.append(None)
            auth = len(parsed) == 2 and parsed[0] == {"type": "auth", "token": self.token}
            user = parsed[-1] if parsed and isinstance(parsed[-1], dict) and parsed[-1].get("type") == "user" else {}
            message = user.get("message") if isinstance(user.get("message"), dict) else {}
            text = message.get("content", "") if message.get("role") == "user" and isinstance(message.get("content"), str) else ""
            found.append({"auth": auth, "text": text, "raw": lines})
        return found

    def headed(self, head):
        """正文第一行以 head 起头的那几条。"""
        return [message for message in self.messages() if message["text"].split("\n", 1)[0].startswith(head)]

    def wait_for(self, world, head, count=1, limit=INBOX_WAIT_SECONDS):
        """等到正文第一行以 head 起头的收到 count 条（limit 是安静时限）；交此刻收到的那几条（不够数也交，调用处判）。"""
        patience = world.patience(limit)
        while patience.waiting():
            if len(self.headed(head)) >= count:
                break
            time.sleep(POLL_SECONDS)
        return self.headed(head)

    def close(self):
        self.closing.set()
        with contextlib.suppress(OSError):
            self.listener.close()
        self.thread.join(timeout=5)


def inbox_problems(messages, head, what, count=1):
    """收件 socket 的共同判据：第一行以 head 起头的恰好 count 条，每一条都先发了认证行、令牌对得上。→ 问题表。"""
    problems = []
    if len(messages) != count:
        problems.append(f"{what}：收件 socket 应当恰好收到 {count} 条第一行是「{head}…」的消息，实际 {len(messages)} 条")
    for message in messages:
        if not message["auth"]:
            problems.append(f"{what}：每条消息应当先发一行认证行、令牌对得上，再发一行 user，实际收到 {[line[:80] for line in message['raw']]}")
    return problems


def start_inbox_server(world, extra=None):
    """发消息那几格起服务：服务盯自己的单元那一样用假 systemctl、照假世界的 systemctl-state 打 active（不随这一台的真单元变）。"""
    with open(os.path.join(world.world, "systemctl-state"), "w", encoding="utf-8") as handle:
        handle.write("active\n")
    world.start_server(extra=dict({"TASK_SERVER_CONTROL_SYSTEMCTL": os.path.join(world.bin, "fake-systemctl")}, **(extra or {})))


def outcome_head(task_id, outcome):
    """推给会话的结局那一条首行到「（<种类>」为止的一段（task_server.py 的 outcome_headline）：类别「结局」、任务已结束、任务号、中文名。"""
    return f"{SERVER.INBOX_HEADLINE}结局（任务已结束）任务 {task_id} {SERVER.kind_label(outcome)}（{outcome}"


def reminder_head(task_id, kind, state="任务还在跑"):
    """推给会话的要处置的提醒首行到「（<种类>」为止的一段（task_server.py 的 reminder_headline）：类别「提醒」、任务此刻的状态（任务还在排 / 任务还在跑，不是结局）、任务号、中文名。"""
    return f"{SERVER.INBOX_HEADLINE}提醒（{state}，不是结局）任务 {task_id} {SERVER.kind_label(kind)}（{kind}"


def reminders_after(messages, seconds):
    """messages 里正文写着 reminded_after_seconds=<seconds> 的那几条（时长判的提醒照末行机器读的那一行分得出这一回跨过的是哪道提醒门槛）。"""
    return [message for message in messages if f" reminded_after_seconds={seconds} " in message["text"]]


KEEP_TEMPORARY_EXTRA = {"SINGLEFS_KEEP_TEST_TEMPORARY_FILES": "1"}   # 提交时设了它：现场留到 task clean（task_server_lib.py 文件头「现场」）

HEALTH_READ_SECONDS = 5   # health_until 每一次读 /health 的时限：到点没回不当服务没回，服务进程还活着就照耐心再读（read_health_patiently）


def read_health_patiently(world, socket_path):
    """读一次 /health（health_until 用）：一次读满 HEALTH_READ_SECONDS 没回（机器忙，任务 9728 第二台 load average 约 60 时这样超时）不当服务没回——
    服务进程还活着就照 world.patience 再读，它的 CPU 在涨算有进展（同起服务那一段等 /health）；服务进程退了、或安静满 STATE_TIMEOUT_SECONDS 还没回，判红。"""
    patience = world.patience(STATE_TIMEOUT_SECONDS)
    while True:
        try:
            return lib.call_server(socket_path, "GET", "/health", timeout=HEALTH_READ_SECONDS)[1]
        except lib.ServerUnreachable as error:
            if world.server is not None and world.server.poll() is not None:
                raise SelftestFailure(f"读 /health 时服务已经退了（退 {world.server.returncode}）：{error}", world.server_log_lines()) from None
            if not patience.waiting(extra=world.server_cpu_ticks()):
                raise SelftestFailure(f"读 /health 读了 {patience.elapsed():.0f} 秒都没回（安静 {STATE_TIMEOUT_SECONDS:.0f} 秒没有进展）：{error}", world.server_log_lines()) from None


def health_until(world, wanted, seconds):
    """在 seconds 秒里每隔 POLL_SECONDS 读一次 /health，wanted(health) 为真就交它；到点交最后一次读到的。每一次读照 read_health_patiently（机器忙时一次读慢了不判红）。"""
    socket_path = os.path.join(world.runtime, lib.SOCKET_NAME)
    patience = world.patience(seconds)
    while True:
        health = read_health_patiently(world, socket_path)
        if wanted(health) or not patience.waiting():
            return health
        time.sleep(POLL_SECONDS)


STARVED_HEALTH_TEXT = "派活判它此刻放得下"   # task_server.py 的 starved_problem 报「判能起却没起成」那一句里的几个字（看进展那一句没有它）


def attempt_has_run_for(summary, seconds):
    """summary 里在跑的那一次尝试，照服务记的起跑时刻（started_at，同一台机器的时钟）已跑满 seconds 秒。"""
    return any(attempt.get("state") == "running" and time.time() - float(attempt.get("started_at") or time.time()) >= seconds for attempt in summary["attempts"])


def first_attempt(summary):
    return (summary["attempts"] or [{}])[0]


def start_burning_scope(world, name, files, usec_per_tick=200000):
    """一份假 scope（attempt_runner.py 文件头「自证换 scope」）：files 里的几份（{文件名: 数或正文}，memory.current、memory.peak、memory.stat 这类）照写、末尾补一个换行，
    cpu.stat 的 usage_usec 照挂钟涨：起了 t 秒就是 t ÷ 0.2 × usec_per_tick 微秒（缺省每 0.2 秒涨 0.2 秒：一个核在算），每 0.2 秒左右写一回，先写临时文件再改名（采样读不到写了一半的）。
    照挂钟算、不照写了几回累加：机器忙时这条线程排不上、写得稀，读到的平均核数照样是设定的那个数（只差最后一回写到读之间那一截）。交 (目录, 停的 Event)：格收尾时设上它。"""
    scope = os.path.join(world.root, name)
    os.makedirs(scope)
    for file_name, value in files.items():
        with open(os.path.join(scope, file_name), "w", encoding="utf-8") as handle:
            handle.write(f"{value}\n")
    stat = os.path.join(scope, "cpu.stat")
    with open(stat, "w", encoding="utf-8") as handle:
        handle.write("usage_usec 0\n")
    stop = threading.Event()

    started = time.monotonic()

    def keep_burning():
        while not stop.is_set():
            used = int((time.monotonic() - started) / 0.2 * usec_per_tick)
            with open(stat + ".partial", "w", encoding="utf-8") as handle:
                handle.write(f"usage_usec {used}\n")
            os.replace(stat + ".partial", stat)
            time.sleep(0.2)

    threading.Thread(target=keep_burning, daemon=True).start()
    return scope, stop


def running_shards(summary):
    """分片父任务的各片里此刻有尝试在跑的：{片号: 那一次的台}。"""
    return {index: attempt.get("host") for index, child in shard_children(summary).items()
            for attempt in child["attempts"] if attempt.get("state") == "running"}


def child_roles(summary):
    return sorted(child["task"].get("role") for child in summary.get("children", []))


def shard_children(summary):
    """分片父任务的各片：{片号: 子任务的 summary}。"""
    return {child["task"].get("shard_index"): child for child in summary.get("children", []) if child["task"].get("role") == "shard"}


def wait_last_line(world, task_id):
    """客户端 wait 的退出码与末行。"""
    waited = world.client(["wait", str(task_id)])
    return waited.returncode, (waited.stdout.strip().splitlines()[-1] if waited.stdout.strip() else "")


def systemctl_start_count(world):
    """假 systemctl 收到过几回 start singlefs-task-server.service。"""
    try:
        with open(os.path.join(world.world, "systemctl-calls"), encoding="utf-8") as handle:
            return sum(1 for line in handle if line.split()[1:3] == ["start", lib.UNIT_NAME])
    except FileNotFoundError:
        return 0


def shown_local_pin(completed):
    """control pin-cap 打的两台此刻的钉值（lib.pinned_caps_text 的「本机 <数>M、第二台 <数>M」）里本机那一项的 MiB 数；没打出来交 None。"""
    found = re.search(r"本机 (\d+)M", completed.stdout)
    return int(found.group(1)) if found else None


def process_gone(process_id):
    """进程号不在了（/proc 里没有它，或只剩僵尸）。"""
    try:
        with open(f"/proc/{process_id}/stat", encoding="utf-8", errors="replace") as handle:
            return handle.read().rsplit(")", 1)[1].split()[0] == "Z"
    except (OSError, IndexError):
        return True


def wait_path(world, path, what, timeout=STATE_TIMEOUT_SECONDS):
    """等 path 出现（服务在自证的停顿点建的 reached-<点> 这类）；等不到判红。"""
    patience = world.patience(timeout)
    while patience.waiting():
        if os.path.exists(path):
            return
        time.sleep(POLL_SECONDS)
    raise SelftestFailure(f"等了 {patience.elapsed():.0f} 秒没{what}（{path} 不在；安静时限 {timeout:.0f} 秒）", world.server_log_lines())


def blocking_command(release, counter=None):
    """跑到 release 在了才退 0 的命令；给了 counter 的先往它追加一个 x（数命令跑了几次）。"""
    head = 'printf x >> "$2"; ' if counter else ""
    return ["bash", "-c", head + 'for round in $(seq 1 2000); do [[ -e "$1" ]] && exit 0; sleep 0.1; done; exit 9', "block", release] + ([counter] if counter else [])


def read_text(path):
    with contextlib.suppress(OSError):
        with open(path, encoding="utf-8") as handle:
            return handle.read()
    return ""


def raw_column_update(world, table, column_task_id, row_id):
    """绕过服务只改一行的 task_id 列（正文与签名不动；TS-05 的那一种篡改）。"""
    with contextlib.closing(sqlite3.connect(os.path.join(world.state, lib.DATABASE_NAME), timeout=30, isolation_level=None)) as connection:
        connection.execute(f"UPDATE {table} SET task_id = ? WHERE id = ?", (column_task_id, row_id))


SCENE_COMMAND_EXIT = 3   # failure-scene-kept-until-collected：假包装演命令退非 0、留现场时退的码（表外的码，记 command-failed、不重排）


def scene_of(world, summary):
    """这一格那一次尝试的现场目录（尝试记录的 scene_directory）与照服务的规矩该在的那个路径（工作根下 SCENES_ROOT_NAME/<尝试标签>）。"""
    attempt = first_attempt(summary)
    return attempt.get("scene_directory") or "", os.path.join(world.work_root, lib.SCENES_ROOT_NAME, attempt.get("label") or "?")


QUEUED_SHARD_KEY = "selftest-shard-third-queued"   # 三片分片的命令键：第 0、1 片记过 2G、第 2 片记过 30G（start_three_shards）


def start_three_shards(world, name, shards, extra=None):
    """三片的一件，第 0、1 片同时在本机跑、第 2 片排着：本机 slice 20G、第二台用不了，第 2 片记过的峰值 30G 只有第二台放得下，排着不判永远放不下
    （同 down-host-waits-not-never）。shards 是规格里三片的写法，extra 是这一格服务的设置；等到第 0、1 片在跑，再排完两轮、核第 2 片一直排着，
    交父任务号与 {片号: 子任务号}。第 2 片放得下（记录读不到时按缺省上限）就与第 0、1 片在同一轮里起，所以不等「第 2 片排着」那一眼：
    起跑中途碰巧看到它还排着的不放过，这几格在记录读不到时都红在下面那一句。"""
    for index, peak in ((0, 2), (1, 2), (2, 30)):
        world.seed_peak(lib.shard_child_key(QUEUED_SHARD_KEY, "shard", index, 3), peak * GIBIBYTE, shard_count=3)
    world.resources(local=(50, 20), peer_down="自证：第二台此刻用不了")
    world.start_server(extra=extra)
    task_id = world.submit(world.write_spec(f"spec-{name}.json", {"shards": shards}), cwd=world.repo, extra={lib.KEY_VARIABLE: QUEUED_SHARD_KEY})
    world.wait_for(task_id, lambda summary: {0, 1} <= set(running_shards(summary)), "第 0、1 片跑起来")
    world.next_pass()   # 在排的那一轮的起跑都已做完：第 2 片放得下就已起；两轮合计不到一秒（排队线程每 0.1 秒一轮），不耗四小时硬上限那一格压到 15 秒的界
    world.next_pass()
    summary = world.summary(task_id)
    if sorted(running_shards(summary)) != [0, 1] or queued_shards(summary) != [2]:
        raise SelftestFailure(f"第 2 片记过的峰值 30G、本机 slice 20G、第二台用不了：排两轮之后应当还是第 0、1 片在跑、第 2 片排着，"
                              f"实际在跑的片 {running_shards(summary)}、排着的片 {queued_shards(summary)}", world.server_log_lines())
    return task_id, {index: child["task"]["id"] for index, child in shard_children(summary).items()}


def queued_shards(summary):
    """分片父任务的各片里还排着、一次没起的片号。"""
    return sorted(index for index, child in shard_children(summary).items() if child["task"].get("state") == "queued" and not child["attempts"])


def shard_outcomes(summary):
    """分片父任务的各片：{片号: 结局（还没结局是空串）}。"""
    return {index: child["task"].get("outcome") or "" for index, child in shard_children(summary).items()}


ATTENTION_WAIT_SECONDS = 60          # 各提醒格里 wait 至多等这么久（弄坏开关下它一直等，等满就判红）
ATTENTION_QUIET_SECONDS = "3"        # attention-quiet-busy-log：日志长期不涨的提醒门槛（秒；TASK_SERVER_SELFTEST_QUIET_ATTENTION_SECONDS），比卡住阈值 2 秒长


def attention_lines(completed):
    return [line for line in completed.stdout.splitlines() if line.startswith(lib.ATTENTION_MARK + " ")]


def attention_fields(line):
    """TASK_SERVER_ATTENTION 那一行拆成 {名: 值}（值照 shlex 引，拆得回）。"""
    return dict(word.split("=", 1) for word in shlex.split(line)[1:] if "=" in word)


def attention_problems(completed, task_id, kind, expected_exit):
    """wait 报要处置的提醒的共同判据：恰好一行 TASK_SERVER_ATTENTION，task=<号> type=needs-attention kind=<kind>，依据、原因、出路都不空，退 expected_exit；
    退的是 ATTENTION_EXIT 的不该打 TASK_SERVER_RESULT。→ (问题表, 那一行拆出的栏)"""
    lines = attention_lines(completed)
    fields = attention_fields(lines[0]) if lines else {}
    problems = []
    if completed.returncode != expected_exit or len(lines) != 1:
        problems.append(f"wait 应当打一行 {lib.ATTENTION_MARK}、退 {expected_exit}，实际退 {completed.returncode}、打了 {len(lines)} 行，输出「{completed.stdout.strip()[-300:]}」")
    elif (fields.get("task"), fields.get("type"), fields.get("kind")) != (str(task_id), lib.ATTENTION_TYPE, kind) or not all(fields.get(name) for name in ("basis", "why", "next")):
        problems.append(f"那一行应当 task={task_id} type={lib.ATTENTION_TYPE} kind={kind} 且依据、原因、出路都不空，实际「{lines[0][:400]}」")
    elif not fields["next"].startswith(table_advice(kind, fields.get("part") or task_id)):   # 出路的头一段是异常表这一种那一行的修改意见（task_server_lib.py 文件头「异常表」）
        problems.append(f"那一行的 next= 应当以异常表里 {kind} 那一行的修改意见「{table_advice(kind, fields.get('part') or task_id)[:120]}…」起头，实际「{fields['next'][:300]}」")
    if expected_exit == lib.ATTENTION_EXIT and lib.RESULT_MARK in completed.stdout:
        problems.append(f"任务还没结局，wait 不该打 {lib.RESULT_MARK}，实际输出「{completed.stdout.strip()[-300:]}」")
    return problems, fields


RUN_LIMIT_SAMPLE_SECONDS = "3"   # 跑满两小时那几格：界压到 3 秒（TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS），不真等两小时

LONG_TICKS_SCRIPT = 'for round in $(seq 1 30); do echo "tick $round"; sleep 0.2; done'   # 跑 6 秒、每 0.2 秒打一行，比 3 秒的界长

SCOPE_DAILY = "种类=items; 跑什么=自证造的一条; 范围=1 条; 为什么=自证：日常开发跑一阵子"
SCOPE_COMMIT = "种类=whole; 跑什么=自证造的门禁; 范围=全部; 为什么=自证：提交前跑整轮"
LONG_RUN_WHY_SAMPLE = "自证：这一条点的就是要跑的全部范围，确实要跑这么久"
LONG_RUN_KIND = "long-run-unconfirmed"

FAKE_COMMIT_GATE = ".claude/gate.d/lkmm.sh"   # 假仓里的一道提交时门禁（lib.COMMIT_GATE_ENTRIES 之一）：跑 LONG_TICKS_SCRIPT
FAKE_MERGE_ENTRY = "scripts/gate/merge-and-verify.sh"


def write_fake_entry(world, relative):
    """假仓根下 relative 那份脚本：正文是 LONG_TICKS_SCRIPT（参数不看）。"""
    path = os.path.join(world.repo, relative)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    write_executable(path, "#!/usr/bin/env bash\n" + LONG_TICKS_SCRIPT + "\n")


def read_text(path):
    with contextlib.suppress(OSError):
        with open(path, encoding="utf-8", errors="replace") as handle:
            return handle.read()
    return ""


def seed_ended_attempt(world, key, ended_at, outcome, memory_peak=None, memory_cap=None):
    """往这一格服务自己的库里记一次结束了的尝试，结束时刻照给的（World.seed_attempt 的结束时刻固定，排不出先后）。
    """
    record = {"command_key": key, "outcome": outcome, "memory_peak": memory_peak, "memory_cap": memory_cap, "task_id": 0, "number": 1,
              "host": "local", "state": "ended", "session": "", "ended_at": ended_at, "started_at": ended_at - 1,
              "peak_metric": lib.PROCESS_MEMORY_PEAK_METRIC}
    world.seeded.append(record)
    world.flush_seeded()


def attention_lines(completed):
    return [line for line in completed.stdout.splitlines() if line.startswith(f"{lib.ATTENTION_MARK} ")]


def restart_surviving_scheduler(world, database_name):
    """换服务不丢的那几份（task_server.py 的 RESTART_SURVIVING_STATES）那几格共用的假调度器：这一格临时库上的 Store（同一份库上再建一个就是服务停了再起），
    Scheduler.__init__ 里那几张表与锁照建，照它末尾那样读回库里那几份（restore_restart_surviving_state）；告警记进 alerts。不起服务。"""
    scheduler = object.__new__(SERVER.Scheduler)
    scheduler.environment = dict(os.environ)
    scheduler.store = SERVER.Store(os.path.join(world.root, database_name), bytes(32), scheduler.environment)
    scheduler.attention_lock = threading.Lock()
    scheduler.runaway_notified, scheduler.advance_failures, scheduler.advance_episodes = {}, {}, {}
    scheduler.suspect_reported, scheduler.suspect_lock, scheduler.health_notified = {}, threading.Lock(), {}
    scheduler.booking_drift_notified, scheduler.booking_drift_lock = {}, threading.Lock()
    scheduler.restart_surviving_lock, scheduler.restart_surviving_written = threading.Lock(), {}
    scheduler.gpu_cards_failed_alerted, scheduler.gpu_cards_failed_lock, scheduler.gpu_cards_failed_known = set(), threading.Lock(), False
    scheduler.gpu_hosts_read = set()
    scheduler.alerts = []
    scheduler.alert = lambda kind, task_id, line, howto: scheduler.alerts.append((kind, task_id, line))
    SERVER.Scheduler.restore_restart_surviving_state(scheduler)
    return scheduler


def long_run_split_scheduler(world, exempt_lines=()):
    """long-run-split-reminded-at-outcome 的假调度器：仓根是这一格自己的目录（豁免表 .claude/long-run-split-exempt 照 exempt_lines 写），任务行与尝试行由调用方给；
    attention_of_record 里出了结局的任务走到的几样（改主工作区的核、预算提醒表）照建。→ (假调度器, 它的仓根)。"""
    root = os.path.join(world.root, f"split-root-{len(os.listdir(world.root))}")
    os.makedirs(os.path.join(root, ".claude"))
    with open(os.path.join(root, ".claude", "long-run-split-exempt"), "w", encoding="utf-8") as handle:
        handle.write("# 跑过两小时也不拆的项，只登记用户点名的\n" + "".join(f"{line}\n" for line in exempt_lines))
    fake = object.__new__(SERVER.Scheduler)
    fake.environment, fake.peer_root = dict(os.environ), root
    fake.budget_notices, fake.idle_short_notices, fake.workspace_write_checks = {}, {}, {}
    return fake, root


FILLS_CAP_SEEDED_CAP_BYTES = 1000 * MEBIBYTE   # fills-any-cap-reported 记下的三次尝试各自的上限
FILLS_CAP_SEEDED_PEAK_BYTES = 950 * MEBIBYTE   # 那三次的峰值：是上限的 95%，不低于 90%

REPUSH_SAMPLE_SECONDS = "2"        # anomaly-repushed-with-history：同一道门槛报过之后再推的门槛（秒；TASK_SERVER_SELFTEST_ANOMALY_REPUSH_SECONDS）


def table_advice(kind, task_id):
    """异常表（lib.ANOMALY_TABLE_TEXT）里这一种那一行的修改意见，{task} 换成任务号：照表现读（lib.anomaly_rows），不经 lib.anomaly_advice（弄坏开关 attention-next-generic 弄坏的是那一侧）。"""
    rows = {row.kind: row for row in lib.anomaly_rows(lib.ANOMALY_TABLE_TEXT)}
    return rows[kind].advice.replace(lib.ANOMALY_TASK_PLACEHOLDER, str(task_id))


def verifying_store(world, environment=None):
    """照服务读回的规则（task_server.py 的 Store.verified：校验列、正文与几列、关联列）读这一格的库，不建表、不写：交一个只读用的 Store。
    environment 缺省是空的（弄坏开关不进判法，判的是真的规则）。"""
    store = object.__new__(SERVER.Store)
    store.path, store.key, store.local = os.path.join(world.state, lib.DATABASE_NAME), lib.read_signing_key(world.key), threading.local()
    store.environment = {} if environment is None else environment
    return store


def verified_row(world, table, row_id):
    """(正文, 照服务的规则核不核得过, 关联列 task_id)；库里没有交 (None, False, None)。"""
    store = verifying_store(world)
    found = store.select(table, "id = ?", (row_id,))
    if not found:
        return None, False, None
    (column,) = store.connection().execute(f"SELECT task_id FROM {table} WHERE id = ?", (row_id,)).fetchone()
    return found[0][0], found[0][1], column


def raise_cell_problems(world, problems, what):
    if problems:
        raise SelftestFailure(f"{what}：" + "；".join(problems), world.server_log_lines())


def loaded_host_dispatch():
    """假调度器要的 host-dispatch.py（Prediction、Verdict）：这一格的进程里载一份。"""
    return lib.load_module("host_dispatch_for_scheduling_cells", os.path.join(lib.HOSTS_DIRECTORY, "host-dispatch.py"))


RUNNING_BOOKING_LABEL = "task-selftest-running-booking-1"


def running_booking_scheduler(peak_bytes, scope_used_bytes, slice_used_gibibytes=0, cap_bytes=20 * GIBIBYTE):
    """running-booked-at-measured-usage 与 running-booked-unscoped-at-cap 的假调度器与一份本机报告：slice 总上限 30G、余量 1G，一条在跑、上限 cap_bytes，
    它这一次到此刻的峰值 peak_bytes（Machine.memory_now；在跑记账不读它），报告里它的 scope 此刻回收不掉 scope_used_bytes（None 是报告里没有它的 scope）。
    → (假调度器, Snapshot, 那份报告)。"""
    attempt = {"id": 1, "task_id": 1, "host": SERVER.LOCAL, "label": RUNNING_BOOKING_LABEL, "memory_cap": cap_bytes}
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.peaks = PEAKS
    fake.host_dispatch = loaded_host_dispatch()
    fake.machines = {SERVER.LOCAL: types.SimpleNamespace(memory_now=lambda found: (scope_used_bytes, peak_bytes))}
    fake.store = types.SimpleNamespace(running_attempts=lambda: [dict(attempt)])
    fake.host_margin = lambda host, snapshot: (GIBIBYTE, "")
    fake.slice_room_floor = SERVER.SLICE_ROOM_FLOOR_BYTES
    fake.unit_stopped = set()   # booking_attempts 不数的、单元已确认停了的尝试号（Scheduler.__init__ 建的）
    report = memory_report(30, 30, slice_used_gibibytes=slice_used_gibibytes,
                           scopes=None if scope_used_bytes is None else {RUNNING_BOOKING_LABEL: scope_used_bytes})
    return fake, SERVER.Snapshot({SERVER.LOCAL: report}, {}, {SERVER.LOCAL: True}, {}), report


CPU_GATE_PROBE_KEY = "selftest-cpu-gate-recorded"   # cpu-gate-new-task-by-recorded-cores：库里记过平均在用核数的那把命令键
CPU_GATE_HEAVY_KEY = "selftest-cpu-gate-heavy"      # 库里另记过 heavy_cores 的那把命令键（cpu-gate-running-started-by-recorded-cores）


def seed_peak_readings(fake, attempts, usage_at, now):
    """假调度器的峰值段读数表（Scheduler.note_peak_segment 读写的 running_peak_samples）照服务一路在看记两回：起跑那一刻（累计 0）与 now（usage_at(尝试)）。
    起了满 CPU_MEASURED_AFTER_SECONDS 的，下一回 running_threads 读到的那一段就是起跑到 now 这一段（各格的假本机一路同一个核数，峰值段就等于它）；不满的照服务还没量满一段办。"""
    for attempt in attempts:
        fake.running_peak_samples[attempt["id"]] = {"first_seen": attempt["started_at"], "samples": [(attempt["started_at"], 0.0), (now, usage_at(attempt))]}


def cpu_gate_scheduler(running_count, cpu, ages=None, cores=1.0, recorded_cores=(), keys=None, recorded_rows=None, heavy_cores=()):
    """cpu-gate-counts-running-threads 的假调度器与 Snapshot：本机在跑 running_count 条、各登记 31 线程，起了多少秒照 ages（缺省各 100 秒，不到 CPU_MEASURED_AFTER_SECONDS），
    一路在用核数都是 cores（假本机的 cpu_usec 照它与建这个假世界那一刻之前的已跑秒数算；峰值段读数照 seed_peak_readings 记起跑与建世界两回）；报告里的 CPU 是 cpu（None 是读不到）。
    recorded_cores 是 CPU_GATE_PROBE_KEY 这把命令键库里跑成的那几次记的核数，heavy_cores 是 CPU_GATE_HEAVY_KEY 那几次的（峰值段核数、CPU 闸线程数与平均在用核数三栏同值；
    admission_threads、running_threads 读；空的是没有记录）；keys 是各条在跑的尝试的命令键（缺省都不带命令键）；cores 给 None 是假本机量不到 CPU（cpu_usec 交 None）。
    recorded_rows 给了就整个换掉 recorded_cores 造的那几行（每行是尝试记录的 outcome、exit_code、cores，峰值段核数与 CPU 闸线程数照 cores 补，命令键补成 CPU_GATE_PROBE_KEY）。
    线程读数（note_threads 的 running_peak_runnable、thread_overruns）是空的，假本机不报线程读数（各格要的自己换 thread_reading）。"""
    now = time.time()
    peak, gate = SERVER.PEAK_SEGMENT_CORES_COLUMN, SERVER.CPU_GATE_THREADS_COLUMN
    running = [dict({"id": position + 1, "host": SERVER.LOCAL, "threads": 31, "started_at": now - age}, **({"command_key": keys[position]} if keys and keys[position] else {}))
               for position, age in enumerate(ages or [100] * running_count)]
    recorded = [({"outcome": "succeeded", "cores": value, peak: value, gate: value, "command_key": CPU_GATE_PROBE_KEY}, True) for value in recorded_cores]
    if recorded_rows is not None:
        recorded = [(dict({peak: row.get("cores"), gate: row.get("cores")}, **row, command_key=CPU_GATE_PROBE_KEY), True) for row in recorded_rows]
    by_key = {(CPU_GATE_PROBE_KEY,): recorded,
              (CPU_GATE_HEAVY_KEY,): [({"outcome": "succeeded", "cores": value, peak: value, gate: value, "command_key": CPU_GATE_HEAVY_KEY}, True) for value in heavy_cores]}
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.peaks = PEAKS
    fake.threads = 31
    fake.admission_threads_cache = {}
    fake.running_threads_cache = {}
    fake.running_usage_lock = threading.Lock()   # CPU 闸读数表的锁（Scheduler.__init__ 建的，假调度器照建）
    fake.unit_stopped = set()   # booking_attempts 不数的、单元已确认停了的尝试号（Scheduler.__init__ 建的）
    fake.running_peak_samples, fake.running_peak_cores = {}, {}   # note_peak_segment 的读数表与各段最大（Scheduler.__init__ 建的）
    fake.running_latest_cores, fake.running_latest_runnable = {}, {}   # 最近一段的在用核数与最近一回的可跑线程数（latest_running_cores 读）
    fake.running_peak_runnable, fake.thread_overruns = {}, {}   # note_threads 记的可跑线程数与线程远超登记数（Scheduler.__init__ 建的）
    fake.machines = {SERVER.LOCAL: types.SimpleNamespace(cpu_usec=lambda attempt: None if cores is None else cores * 1e6 * (now - attempt["started_at"]),
                                                         thread_reading=lambda attempt: None, label="本机")}
    if cores is not None:
        seed_peak_readings(fake, running, fake.machines[SERVER.LOCAL].cpu_usec, now)
    fake.store = types.SimpleNamespace(running_attempts=lambda: [dict(attempt) for attempt in running],
                                       select=lambda table, where="1", parameters=(), order="id": list(by_key.get(tuple(parameters), [])))
    report = memory_report(30, 30)
    if cpu is None:
        report["memory"].pop(PEAKS.CPU_REPORT_KEY, None)
    else:
        report["memory"][PEAKS.CPU_REPORT_KEY] = dict(cpu)
    return fake, SERVER.Snapshot({SERVER.LOCAL: report}, {}, {SERVER.LOCAL: True}, {})


NO_BORROW_ENTRY_SCRIPT = "scripts/gate/selftest-no-borrow-entry.sh"   # shard-start-borrows-nothing：被分片的入口脚本（父任务的 sharded_entry）

FAKE_JOURNALCTL_EARLYOOM = r'''#!/usr/bin/env bash
# task-server 自证的假 journalctl（TASK_SERVER_JOURNALCTL；服务读 earlyoom 与内核日志认被外部信号杀掉的尝试，task_server.py 的 kill_log_lines）：
# 参数里有 -t earlyoom 时，世界目录里的 earlyoom-pid 在、里面有进程号，就打一行 earlyoom 杀进程的日志点名这个号（没有就一行不打）；有 -k 的一行不打；别的参数退 9
case " $* " in
  *" -t earlyoom "*)
    if [[ -s "$TASK_SERVER_SELFTEST_WORLD/earlyoom-pid" ]]; then
      echo "sending SIGTERM to process $(cat -- "$TASK_SERVER_SELFTEST_WORLD/earlyoom-pid") uid 1000 \"rustc\": badness 1414, VmRSS 9708 MiB"
    fi
    exit 0 ;;
  *" -k "*) exit 0 ;;
  *) exit 9 ;;
esac
'''


def peer_log_scheduler(world, name, lines=()):
    """第二台日志那几格的假调度器：第二台的一次尝试（id 7），日志是这一格临时目录里的一份（先写进 lines），经真的 log_content 记表。→ (调度器, 尝试)。"""
    log = os.path.join(world.root, f"peer-log-{name}.log")
    with open(log, "w", encoding="utf-8") as handle:
        handle.write("".join(f"{line}\n" for line in lines))
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    for field in ("log_offsets", "peer_memory", "peer_memory_stat", "peer_io_wait", "peer_threads",
                  "peer_external_kill", "peer_io_rounds", "peer_thread_rounds", "gpu_not_run_marks"):
        setattr(fake, field, {})
    fake.machines = {SERVER.PEER: SERVER.PeerMachine(fake)}
    return fake, {"id": 7, "task_id": 77, "number": 1, "host": SERVER.PEER, "label": f"task-selftest-peer-{name}", "log": log}


def append_lines(path, lines):
    with open(path, "a", encoding="utf-8") as handle:
        handle.write("".join(f"{line}\n" for line in lines))


# ── 编译目录按树与机器给（task_server_lib.py 文件头「编译目录」）：组的几格，与编译目录、task clean、取消跟着它改的几格 ──
def fake_cargo_extra(world):
    """交假 cargo 的任务环境：PATH 最前面是这一格的假 cargo（FAKE_CARGO）。"""
    return {"PATH": f"{world.fake_cargo}:{os.environ.get('PATH', '/usr/bin:/bin')}"}


def cargo_targets(world):
    """这一格假 cargo 每一次拿到的 CARGO_TARGET_DIR（照调用记录的先后）。"""
    return [line.split(" target=", 1)[1].split(" args=", 1)[0] for line in world.calls() if line.startswith("cargo dir=")]


def build_slot_tree(world, name):
    """这一格自己建的一棵树（task_server_lib.build_tree_root 认的那一层：带 .git 的目录），放在假共享根下：→ 它的绝对路径。"""
    tree = os.path.join(world.shared_root, name)
    os.makedirs(os.path.join(tree, ".git"), exist_ok=True)
    return tree
