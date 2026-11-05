#!/usr/bin/env python3
# admission: always 每一格都起此刻编出的 prune-server 与此刻的三家脚本，上一次的结论不替这一次作保
"""prune_server_client.py 的自证（格、弄坏开关、三家原有自证对临时服务重跑）与临时服务的起法。与客户端分成两份文件：客户端被三家判定库 import，
它的模块级 main-guard 会随 import 进每一家判法的单元闭包，自证里起服务、起 cargo、载入 admission.py 的边不能挂在客户端上
（lib_function_units 的 main-guard 单元；lib_verdict_library 自证「判法单元指纹不含库自己」那一格就是这么红的）。
同一个缘故，三家与旁路的原有自证不 import 这一份、不起它：它们只调客户端的 selftest_service_problem() 核环境，临时服务由这一份的
--with-temporary-service 在外面起好。

  python3 scripts/prune-server/prune_server_client_selftest.py --selftest
  python3 scripts/prune-server/prune_server_client_selftest.py --selftest-families
  python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- <命令> [<参数>…]
三条整条都不带二进制，没给 --server-binary 就现编；自己编了、两台都读得到的二进制才在子命令后另加 --server-binary <路径>。
格的清单、弄坏开关、末行的写法都在 scripts/prune-server/prune_server_client.py 文件头「命令行」「自证的格」两段。
"""
import contextlib
import hashlib
import importlib.util
import json
import os
import random
import select
import shutil
import signal
import socket
import struct
import subprocess
import sys
import tempfile
import threading
import time

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.realpath(__file__)))
from prune_server_client import (  # noqa: E402  客户端的常量、类与函数：自证照原来写在同一份文件里时的名字用
    BINARY_VARIABLE, BREAK_VARIABLE, EXIT_OTHER, EXIT_UNAVAILABLE, EXIT_USAGE, FAMILIES, FAMILY_APPARATUS_ITEMS, FAMILY_UNIT_VERDICTS, MAXIMUM_FRAME_BODY_BYTES,
    MULTI_HOST_CONFIGURATION_VARIABLE, OPERATION_ANNOUNCE_STOP, PROCESS_CLIENT, RECONNECT_LIMIT_VARIABLE, REPOSITORY_ROOT, RESPONSE_TIMEOUT_VARIABLE,
    SEGMENT_BYTES, SELFTEST_VARIABLE, SOCKET_VARIABLE, STATUS_COMPLETED, TCP_ADDRESS_VARIABLE, TEMPORARY_SERVICE_VARIABLE,
    Client, LibraryStore, PruneServerFailure, break_is, encode_frame, encode_manifest, manifest_of, process_client, read_response, request_body,
    namespace_of, reset_process_client, segment_count_key, segment_key, segment_key_parts, segments_needed, selftest_service_problem,
)


CELLS = ("round-trip-unix", "prefix-scan", "tcp-round-trip", "busy-retried-after-delay",
         "stopped-exits-75", "unreachable-exits-75", "families-over-service", "unreachable-family-exits-75",
         "connect-bounded-by-reconnect-total", "large-value-round-trip", "segmented-read-incomplete-refused",
         "segmented-read-sees-a-rewrite", "segment-count-over-index-digits-refused", "scan-row-over-frame-refused",
         "segment-key-shapes-only-reserved", "oversize-frame-reported-at-once", "temporary-service-exits-with-parent", "service-problem-judges-given-environment",
         "connection-variables-refused-outside-temporary-service", "command-line-read", "namespace-only-under-temporary-service",
         "batched-read-answer-over-frame-limit-split", "binary-not-built-without-target-directory", "delete-prefix-by-namespace",
         "delete-by-item-name-picks-by-value")
BREAK_RED_CELLS = (("read-failure-reported-as-absent", "unreachable-exits-75"), ("reconnect-unbounded", "unreachable-exits-75"),
                   ("stop-notice-retried", "stopped-exits-75"), ("busy-retry-without-delay", "busy-retried-after-delay"),
                   ("family-prefixes-shared", "families-over-service"),
                   ("unreachable-treated-as-no-library", "unreachable-family-exits-75"),
                   ("connect-given-full-response-timeout", "connect-bounded-by-reconnect-total"),
                   ("large-value-sent-unsegmented", "large-value-round-trip"),
                   ("segment-count-shrinks-on-rewrite", "large-value-round-trip"),
                   ("segments-assembled-without-sha256", "segmented-read-incomplete-refused"),
                   ("segmented-read-not-reread", "segmented-read-sees-a-rewrite"),
                   ("segment-count-unbounded", "segment-count-over-index-digits-refused"),
                   ("scan-row-over-frame-split", "scan-row-over-frame-refused"),
                   ("nul-keys-reserved-whole", "segment-key-shapes-only-reserved"),
                   ("segment-key-shapes-accepted", "segment-key-shapes-only-reserved"),
                   ("other-segment-keys-scanned-as-entries", "segment-key-shapes-only-reserved"),
                   ("oversize-frame-sent-and-retried", "oversize-frame-reported-at-once"),
                   ("temporary-service-outlives-parent", "temporary-service-exits-with-parent"),
                   ("service-problem-reads-process-environment", "service-problem-judges-given-environment"),
                   ("connection-variables-honored-outside-temporary-service", "connection-variables-refused-outside-temporary-service"),
                   ("command-line-read-absent-as-empty", "command-line-read"),
                   ("namespace-by-temporary-directory", "namespace-only-under-temporary-service"),
                   ("oversized-answer-read-unsplit", "batched-read-answer-over-frame-limit-split"),
                   ("build-without-target-directory", "binary-not-built-without-target-directory"),
                   ("delete-prefix-without-namespace", "delete-prefix-by-namespace"),
                   ("item-name-ignores-kind", "delete-by-item-name-picks-by-value"),
                   ("item-name-dry-run-deletes", "delete-by-item-name-picks-by-value"))
CARGO_TARGET_DIRECTORY_VARIABLE = "CARGO_TARGET_DIR"   # 任务服务给的编译目录；没有它就不现编服务二进制（resolve_binary）
SERVER_READY_SECONDS = 120
CHILD_TIMEOUT_SECONDS = 60
STOP_EXIT_WITHIN_SECONDS = 10
SELFTEST_RECONNECT_SECONDS = 3
SELFTEST_BUSY_COMMIT_DELAY_MILLISECONDS = 200
SELFTEST_BUSY_RETRY_MILLISECONDS = 50
LISTENING_LINE_PREFIX = "PRUNE_SERVER_LISTENING"
MAXIMUM_SOCKET_PATH_BYTES = 107
CONNECT_CELL_RECONNECT_SECONDS = 2     # connect-bounded-by-reconnect-total：重连总时长上限
CONNECT_CELL_RESPONSE_SECONDS = 30     # 同一格：等回应的超时，远大于总时长上限（建连给满它就堵过限时）
CONNECT_CELL_GRACE_SECONDS = 3         # 同一格：总时长上限之外容的秒数（起 python 子进程、退避抖动）
CONNECT_CELL_FILL_ATTEMPTS = 64        # 同一格：塞连接队列至多试这么多条
CONNECT_CELL_PROBE_SECONDS = 0.5       # 同一格：一条 TCP 连接这么久还没握手完成，就算队列满了
LARGE_VALUE_BYTES = MAXIMUM_FRAME_BODY_BYTES + 3 * 1024 * 1024 + 17   # large-value-round-trip：大过帧长上限的值（分 5 段）
LARGE_VALUE_SEED = 19
REWRITTEN_TAIL_BYTES = 5               # 同一格：重写成 SEGMENT_BYTES 加这么多字节（要两段）
INCOMPLETE_TAIL_BYTES = 11             # segmented-read-incomplete-refused：值是两整段加这么多字节（分 3 段）
OVER_LIMIT_VALUE_COUNT = 5             # batched-read-answer-over-frame-limit-split：各正好一段长（不分段存）的值几把，合计超过帧长上限
SHAPE_CELL_PREFIX = b"\x1bshape-cell/"  # segment-key-shapes-only-reserved：键都以一个表标签那样的字节起头（崩溃判定库的表标签在 0x00 到 0x1B）
SHAPE_CELL_TAIL_BYTES = 13             # 同一格：带 NUL 的那把大值是一整段加这么多字节（分 2 段）
OVERSIZE_EXIT_WITHIN_SECONDS = SELFTEST_RECONNECT_SECONDS   # oversize-frame-reported-at-once：要在重连总时长之内就退出（不重连）
PARENT_DEATH_EXIT_WITHIN_SECONDS = 10  # temporary-service-exits-with-parent：起服务的进程被 SIGKILL 之后，服务要在这么多秒里不在
HOLD_SERVER_SECONDS = CHILD_TIMEOUT_SECONDS   # 同一格：起服务的子进程打出进程号之后至多等这么久（这一格在那之前就 SIGKILL 它）
CLIENT_PATH = os.path.join(os.path.dirname(os.path.realpath(__file__)), "prune_server_client.py")
# TestServer 起服务经的这一段（客户端文件头「命令行」临时服务跟着起它的进程走）：先设父进程死亡信号、核起它的进程还在，再换成服务二进制。
# 参数：起它的进程号、服务二进制、服务的参数…
PARENT_DEATH_LAUNCHER = """import ctypes, os, signal, sys
PARENT_DEATH_SIGNAL_OPTION = 1   # prctl 的 PR_SET_PDEATHSIG
parent = int(sys.argv[1])
if ctypes.CDLL(None, use_errno=True).prctl(PARENT_DEATH_SIGNAL_OPTION, int(signal.SIGKILL)) != 0:
    sys.exit(f"prctl(PR_SET_PDEATHSIG) 没成：errno {ctypes.get_errno()}")
if os.getppid() != parent:
    sys.exit("起这份临时服务的进程在设父进程死亡信号之前就没了")
os.execv(sys.argv[2], sys.argv[2:])
"""
ENVIRONMENT_CLEARED_FOR_CELLS = (SOCKET_VARIABLE, TCP_ADDRESS_VARIABLE, TEMPORARY_SERVICE_VARIABLE, RECONNECT_LIMIT_VARIABLE,
                                 RESPONSE_TIMEOUT_VARIABLE, MULTI_HOST_CONFIGURATION_VARIABLE)


def family_selftests():
    """三家与它们旁路的原有自证（只给 --selftest-families 用；每一份都只在临时服务上跑，客户端文件头「命令行」）。写成函数、不放模块级：
    模块级的程序边会随 import 进每一家判法的单元闭包，把 admission.py 这类入口拉进 verus-proof-check.py 等判法的键里（lib_function_units 按词认起脚本的命令）。"""
    return (
        ("scripts/crash/lib_verdict_library.py", ("python3", "scripts/crash/lib_verdict_library.py", "--selftest")),
        ("scripts/gate/admission.py", ("python3", "scripts/gate/admission.py", "--selftest")),
        ("scripts/gate/lib_reuse_checks.py", ("python3", "scripts/gate/lib_reuse_checks.py", "--selftest")),
        ("scripts/gate/verus-proof-check.py", ("python3", "scripts/gate/verus-proof-check.py", "--selftest")),
        ("scripts/mutation/mutate.sh", ("bash", "scripts/mutation/mutate.sh", "--selftest")),
        ("scripts/mutation/crates-mutation-rows.py", ("python3", "scripts/mutation/crates-mutation-rows.py", "--selftest")),
        # 经三家写服务的别的自证（开跑前同样调 selftest_service_problem，裸跑退 2）：门禁与入口那几份经 admission.py，e165 与 lkmm.sh 经 lib_verdict_library，
        #   回归单测经 lib_mutation_units
        ("scripts/crash/crash-point-replay.sh", ("bash", "scripts/crash/crash-point-replay.sh", "--selftest")),
        ("scripts/mutation/mutation-shard-run.sh", ("bash", "scripts/mutation/mutation-shard-run.sh", "--selftest")),
        ("scripts/experiment-apparatus/e165-pin-red-examples.py", ("python3", "scripts/experiment-apparatus/e165-pin-red-examples.py", "--selftest")),
        ("scripts/gate/lkmm-selftest.sh", ("bash", "scripts/gate/lkmm-selftest.sh", "--selftest")),
        ("scripts/gate/merge-and-verify.sh", ("bash", "scripts/gate/merge-and-verify.sh", "--selftest")),
        ("scripts/mutation/tests/test_audit_regressions.py", ("python3", "scripts/mutation/tests/test_audit_regressions.py", "--selftest")),
        # 族 broad-mutation 的驱动（自己就是那一族的读写方，开跑前同样调 selftest_service_problem）
        ("scripts/mutation/broad-mutation.py", ("python3", "scripts/mutation/broad-mutation.py", "--selftest")),
    )


def selftest_root():
    """临时目录：socket 全路径要放得下 107 字节，TMPDIR 太长时退到 /tmp。"""
    base = tempfile.gettempdir()
    if len(os.path.join(base, "prune-client-0123456789", "s", "server.sock")) > MAXIMUM_SOCKET_PATH_BYTES:
        base = "/tmp"
    return tempfile.mkdtemp(prefix="prune-client-", dir=base)


def build_server_binary():
    """没给二进制时编一份（重活，交 task-server）：照 cargo 的 JSON 消息取 prune-server 的可执行文件路径。"""
    manifest = os.path.join(REPOSITORY_ROOT, "prune-server", "Cargo.toml")
    completed = subprocess.run(["cargo", "build", "--release", "--offline", "--manifest-path", manifest, "--bin", "prune-server",
                                "--message-format=json-render-diagnostics"], capture_output=True, text=True)
    executable = ""
    for line in completed.stdout.splitlines():
        with contextlib.suppress(ValueError):
            message = json.loads(line)
            if message.get("reason") == "compiler-artifact" and message.get("target", {}).get("name") == "prune-server" and message.get("executable"):
                executable = message["executable"]
    if completed.returncode != 0 or not executable:
        return None, f"cargo build 退 {completed.returncode}：{completed.stderr.strip()[-600:]}"
    return executable, ""


class TestServer:
    """一格一份真 prune-server：socket、状态目录、库、密钥都在这一格的临时目录里；它自己一个进程组，收尾对整组发信号。
    经 PARENT_DEATH_LAUNCHER 起：起它的进程被硬杀、收尾不跑时内核停服务（客户端文件头「命令行」临时服务跟着起它的进程走）；
    父进程死亡信号认的是建它的那一条线程，只在活到收尾的线程里建。弄坏开关 temporary-service-outlives-parent 下直接起二进制（改前的样子）。"""

    def __init__(self, binary, directory, extra=()):
        os.makedirs(directory)
        self.directory = directory
        self.socket = os.path.join(directory, "server.sock")
        self.library = os.path.join(directory, "library")
        self.standard_error = open(os.path.join(directory, "server.stderr"), "w", encoding="utf-8")
        command = [binary, "serve", "--selftest-instance", "--socket", self.socket, "--state-directory", directory,
                   "--library", self.library, "--tcp", "127.0.0.1:0", "--worker-threads", "2", *extra]
        if not break_is("temporary-service-outlives-parent"):
            command = [sys.executable, "-I", "-c", PARENT_DEATH_LAUNCHER, str(os.getpid()), *command]
        self.process = subprocess.Popen(command,
                                        stdout=subprocess.PIPE, stderr=self.standard_error, text=True, start_new_session=True,
                                        env={name: value for name, value in os.environ.items() if not name.startswith("PRUNE_SERVER_BREAK")})
        self.tcp_address = ""
        line = self.wait_for_listening_line()
        for field in line.split():
            if field.startswith("tcp="):
                self.tcp_address = field[len("tcp="):]
        if not self.tcp_address:
            self.stop()
            raise RuntimeError(f"服务的就绪行没有 tcp=：{line!r}；标准错误：{self.error_text()}")

    def wait_for_listening_line(self):
        found = []

        def reader():
            # 读到就绪行之后接着读空标准输出（服务再打什么也不会因为管道满了堵住），流断了才退出
            for text in self.process.stdout:
                if text.startswith(LISTENING_LINE_PREFIX) and not found:
                    found.append(text.strip())

        thread = threading.Thread(target=reader, daemon=True)
        thread.start()
        deadline = time.monotonic() + SERVER_READY_SECONDS
        while not found and thread.is_alive() and time.monotonic() < deadline:
            time.sleep(0.02)
        if not found:
            self.stop()
            raise RuntimeError(f"服务 {SERVER_READY_SECONDS} 秒内没打就绪行；标准错误：{self.error_text()}")
        return found[0]

    def error_text(self):
        try:
            self.standard_error.flush()
        except ValueError:
            pass   # stop() 之后文件已关，落盘的内容照读
        try:
            with open(os.path.join(self.directory, "server.stderr"), encoding="utf-8", errors="replace") as handle:
                return handle.read()[-800:]
        except OSError:
            return "（读不了）"

    def kill(self):
        """kill -9 整个进程组（它是这一格自己起的进程，进程组号就是它的进程号），不重起。"""
        if self.process.poll() is None:
            with contextlib.suppress(ProcessLookupError):
                os.killpg(self.process.pid, signal.SIGKILL)
            self.process.wait(timeout=30)

    def stop(self):
        if self.process.poll() is None:
            with contextlib.suppress(ProcessLookupError):
                os.killpg(self.process.pid, signal.SIGTERM)
            try:
                self.process.wait(timeout=30)
            except subprocess.TimeoutExpired:
                self.kill()
        with contextlib.suppress(ProcessLookupError, PermissionError):
            os.killpg(self.process.pid, 0)
            os.killpg(self.process.pid, signal.SIGKILL)
        # 标准输出不在这里关：读它的线程读到流尾自己退出（进程组已经停了）
        self.standard_error.close()

    def announce_stop(self):
        """经 unix socket 发「宣布停服务」（只给自证，控制命令 stop 走的就是这一帧）。"""
        connection = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        connection.settimeout(30)
        connection.connect(self.socket)
        try:
            connection.sendall(encode_frame(request_body(1, OPERATION_ANNOUNCE_STOP, b"")))
            return read_response(connection)[1]
        finally:
            connection.close()


def with_temporary_service_mark(changes):
    """自证各格设连哪一处时照 --with-temporary-service 同样设临时服务的标记（客户端文件头「连哪一处」：改连接地址的变量只在临时服务底下认）：
    changes 里有 SOCKET_VARIABLE 或 TCP_ADDRESS_VARIABLE、又没写标记的，标记取它们的值；写了标记的（连空串）照原样，演平常跑的那一格靠它。"""
    if (SOCKET_VARIABLE in changes or TCP_ADDRESS_VARIABLE in changes) and TEMPORARY_SERVICE_VARIABLE not in changes:
        return dict(changes, **{TEMPORARY_SERVICE_VARIABLE: changes.get(SOCKET_VARIABLE) or changes[TCP_ADDRESS_VARIABLE]})
    return dict(changes)


@contextlib.contextmanager
def environment_for(changes):
    """进程内的格换一份环境：清掉连哪一处与开关那几个变量、设上 changes，收尾还原；连接一并丢掉。"""
    saved = {name: os.environ.get(name) for name in set(ENVIRONMENT_CLEARED_FOR_CELLS) | set(changes)}
    for name in ENVIRONMENT_CLEARED_FOR_CELLS:
        os.environ.pop(name, None)
    os.environ.update(with_temporary_service_mark(changes))
    reset_process_client()
    try:
        yield
    finally:
        reset_process_client()
        for name, value in saved.items():
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value


def unix_environment(server):
    return {SOCKET_VARIABLE: server.socket}


def tcp_environment(server):
    return {SOCKET_VARIABLE: os.path.join(server.directory, "no-such.sock"), TCP_ADDRESS_VARIABLE: server.tcp_address}


def cell_environment(changes):
    """格起的子进程的环境：清掉连哪一处与开关那几个变量、设上 changes（临时服务的标记照 with_temporary_service_mark），自证标记设 1。"""
    environment = {name: value for name, value in os.environ.items() if name not in ENVIRONMENT_CLEARED_FOR_CELLS}
    environment.update(with_temporary_service_mark(changes))
    environment[SELFTEST_VARIABLE] = "1"
    return environment


def run_child(arguments, changes, timeout=CHILD_TIMEOUT_SECONDS, program=None):
    """起本文件的 --selftest-child（program 给了起它：[解释器, 脚本…]；它自己一个进程组，超时对整组发 SIGKILL）：
    交 (退出码或 None（超时）, 用时, 标准输出, 标准错误)。"""
    environment = cell_environment(changes)
    started = time.monotonic()
    process = subprocess.Popen(program or [sys.executable, os.path.realpath(__file__), "--selftest-child", *arguments], stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, text=True, env=environment, start_new_session=True)
    try:
        output, error = process.communicate(timeout=timeout)
        return process.returncode, time.monotonic() - started, output, error
    except subprocess.TimeoutExpired:
        with contextlib.suppress(ProcessLookupError):
            os.killpg(process.pid, signal.SIGKILL)
        output, error = process.communicate()
        return None, time.monotonic() - started, output, error


def selftest_child(arguments):
    """自证起的子进程：read <键> 读一个键打「absent」或「present」；verdict-lookup <目录> 跑 lib_verdict_library 的 lookup、打它交的那一对；
    write-long-key <字节数> 写一条键长这么多字节的条目，写成了打「written」；hold-server <二进制> <目录> 起一份 TestServer、打「server_pid=<进程号>」，
    再睡 HOLD_SERVER_SECONDS 秒（temporary-service-exits-with-parent 在那之前 SIGKILL 它）。"""
    if arguments[:1] == ["hold-server"] and len(arguments) == 3:
        server = TestServer(arguments[1], arguments[2])
        print(f"server_pid={server.process.pid}", flush=True)
        time.sleep(HOLD_SERVER_SECONDS)
        server.stop()
        return 0
    if arguments[:1] == ["read"] and len(arguments) == 2:
        value = process_client().read_many([arguments[1].encode()])[0]
        print("absent" if value is None else "present")
        return 0
    if arguments[:1] == ["write-long-key"] and len(arguments) == 2 and arguments[1].isdigit():
        process_client().write_many([(b"k" * int(arguments[1]), b"v")])
        print("written")
        return 0
    if arguments[:1] == ["verdict-lookup"] and len(arguments) == 2:
        module = load_by_path("lib_verdict_library_for_prune_client_selftest", os.path.join(REPOSITORY_ROOT, "scripts", "crash", "lib_verdict_library.py"))
        library = module.VerdictLibrary(arguments[1], "kind", module.digest_text("判法"))
        print(library.lookup("一项", [("这一项", module.digest_text("甲"))]))
        return 0
    print(f"  ✗ --selftest-child 认不出 {arguments}", file=sys.stderr)
    print("     → 怎么办：--selftest-child 只由本脚本的自证自己起，不手起；要跑自证写 --selftest", file=sys.stderr)
    return EXIT_USAGE


def load_by_path(name, path):
    specification = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(specification)
    sys.modules[name] = module
    specification.loader.exec_module(module)
    return module


def files_under(directory):
    return sorted(os.path.relpath(os.path.join(walked, name), directory) for walked, _directories, names in os.walk(directory) for name in names) \
        if os.path.isdir(directory) else []


def cell_round_trip_unix(binary, work):
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        with environment_for(unix_environment(server)):
            client = process_client()
            client.write_many([(b"round/one", b"1"), (b"round/two", b"2"), (b"round/three", b"\x00\xff")])
            values = client.read_many([b"round/one", b"round/two", b"round/three", b"round/missing"])
            written, already = client.write_frame([(b"round/one", b"overwritten-if-reapplied")], request_number=1)
            after = client.read_many([b"round/one"])[0]
            route = client.route
        passed = values == [b"1", b"2", b"\x00\xff", None] and already and after == b"1" and route.startswith("unix:")
        return passed, f"读回 {values}，同一请求号再发：写了 {written} 条、早已生效 {already}，之后 round/one={after!r}，走的 {route}"
    finally:
        server.stop()


def cell_prefix_scan(binary, work):
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        with environment_for(unix_environment(server)):
            client = process_client()
            client.write_many([(b"scan/a/2", b"two"), (b"scan/a/1", b"one"), (b"scan/b/1", b"other"), (b"scan/a/3", b"three")])
            entries = client.scan(b"scan/a/")
        passed = entries == [(b"scan/a/1", b"one"), (b"scan/a/2", b"two"), (b"scan/a/3", b"three")]
        return passed, f"扫 scan/a/ 得 {entries}"
    finally:
        server.stop()


def cell_tcp_round_trip(binary, work):
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        with environment_for(tcp_environment(server)):
            client = process_client()
            client.write_many([(b"tcp/one", b"over-tcp")])
            value = client.read_many([b"tcp/one"])[0]
            route = client.route
        return value == b"over-tcp" and route.startswith("tcp:"), f"读回 {value!r}，走的 {route}"
    finally:
        server.stop()


def cell_busy_retried_after_delay(binary, work):
    server = TestServer(binary, os.path.join(work, "server"),
                        ("--maximum-pending-total", "1", "--busy-retry-milliseconds", str(SELFTEST_BUSY_RETRY_MILLISECONDS),
                         "--selftest-commit-delay-milliseconds", str(SELFTEST_BUSY_COMMIT_DELAY_MILLISECONDS)))
    try:
        with environment_for(unix_environment(server)):
            clients = [Client(), Client()]
            problems = []

            def writer(client, label):
                try:
                    for index in range(4):
                        client.write_many([(f"busy/{label}/{index}".encode(), label.encode())])
                except PruneServerFailure as error:
                    problems.append(str(error))

            threads = [threading.Thread(target=writer, args=(client, label)) for client, label in zip(clients, ("left", "right"))]
            for thread in threads:
                thread.start()
            for thread in threads:
                thread.join()
            landed = process_client().scan(b"busy/")
            for client in clients:
                client.close()
        waits = [wait for client in clients for wait in client.busy_waits]
        short = [(milliseconds, seconds) for milliseconds, seconds in waits if seconds < milliseconds / 1000.0]
        busy = sum(client.busy_responses for client in clients)
        passed = not problems and len(landed) == 8 and busy >= 1 and not short
        return passed, f"忙 {busy} 次，等得不够的 {short[:3]}，落库 {len(landed)} 条（要 8），出错 {problems[:2]}"
    finally:
        server.stop()


def cell_stopped_exits_75(binary, work):
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        status = server.announce_stop()
        code, seconds, output, error = run_child(["read", "anything"], dict(unix_environment(server), **{RECONNECT_LIMIT_VARIABLE: "30"}),
                                                 timeout=STOP_EXIT_WITHIN_SECONDS * 2)
        passed = status == STATUS_COMPLETED and code == EXIT_UNAVAILABLE and seconds < STOP_EXIT_WITHIN_SECONDS and "服务停了" in error
        return passed, (f"宣布停服务回状态 {status}；子进程退 {code}（{seconds:.1f} 秒，要 {STOP_EXIT_WITHIN_SECONDS} 秒内），"
                        f"标准输出「{output.strip()}」，标准错误「{error.strip()[-300:]}」")
    finally:
        server.stop()


def cell_unreachable_exits_75(binary, work):
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        with environment_for(unix_environment(server)):
            process_client().write_many([(b"unreachable/one", b"1")])
        server.kill()
        code, seconds, output, error = run_child(["read", "unreachable/one"],
                                                 dict(unix_environment(server), **{RECONNECT_LIMIT_VARIABLE: str(SELFTEST_RECONNECT_SECONDS)}),
                                                 timeout=SELFTEST_RECONNECT_SECONDS * 8)
        passed = code == EXIT_UNAVAILABLE and "连不上" in error and "absent" not in output
        return passed, f"子进程退 {code}（{seconds:.1f} 秒；None 是超时被停），标准输出「{output.strip()}」，标准错误「{error.strip()[-300:]}」"
    finally:
        server.stop()


def cell_families_over_service(binary, work):
    """五族（三家、广谱变异的驱动、装置的转交）各开一份库目录在临时目录底下的库，读写一遍；服务侧按各自前缀扫、本地目录一个文件都没有。
    装置的转交（apparatus-items）整条路由 device-code-key.py 的自证核，这里只经 LibraryStore 写一条、读回、看前缀。"""
    server = TestServer(binary, os.path.join(work, "server"))
    notes, passed = [], True
    try:
        verdict_directory = os.path.join(work, "verdict-root", "singlefs-sample-verdicts")
        mutation_directory = os.path.join(work, "mutation-root", "singlefs-sample-library")
        broad_directory = os.path.join(work, "broad-root", "broad-mutation-units")
        admission_tree = os.path.join(work, "admission-tree")
        os.makedirs(admission_tree)
        initialized = subprocess.run(["git", "init", "-q", admission_tree], capture_output=True, text=True)
        admission_common = os.path.join(admission_tree, ".git")
        before_admission = files_under(admission_common)
        with environment_for(unix_environment(server)):
            verdict_module = load_by_path("lib_verdict_library_for_prune_client_cells", os.path.join(REPOSITORY_ROOT, "scripts", "crash", "lib_verdict_library.py"))
            library = verdict_module.VerdictLibrary(verdict_directory, "kind", verdict_module.digest_text("判法"))
            components = [("这一项", verdict_module.digest_text("甲"))]
            first = library.lookup("一项", components)
            library.record("一项", components, "判绿", run="自证")
            second = library.lookup("一项", components)
            library.record_red("一项", components, "判红", run="自证")
            third = library.lookup("一项", components)
            verdict_ok = first == (False, verdict_module.NO_RECORD_REASON) and second[0] and third == (False, verdict_module.RED_RECORD_REASON)
            notes.append(f"verdict-library：头一次 {first}、记绿之后 {second}、记红之后 {third}")

            admission_module = load_by_path("admission_for_prune_client_cells", os.path.join(REPOSITORY_ROOT, "scripts", "gate", "admission.py"))
            unit_library = admission_module.VerdictLibrary(admission_tree, "sample-gate.sh:cell")
            problem = unit_library.record("请求原文", "一条", "a" * 64, "自证")
            items = unit_library.items()
            read_back = unit_library.read(unit_library.path_of_key("a" * 64))
            unit_library.record_red("请求原文", "一条", "a" * 64, "自证")
            red_back = unit_library.read_red(unit_library.path_of_key("a" * 64), "a" * 64)
            admission_ok = problem == "" and [name for name, _record in items] == ["一条"] and (read_back or {}).get("verdict") == "green" \
                and (red_back or {}).get("verdict") == "red"
            notes.append(f"unit-verdicts：写 {problem!r}、items {[name for name, _record in items]}、读回 {(read_back or {}).get('verdict')}、记红之后 {(red_back or {}).get('verdict')}")

            units_module = load_by_path("lib_mutation_units_for_prune_client_cells", os.path.join(REPOSITORY_ROOT, "scripts", "mutation", "lib_mutation_units.py"))
            mutation_library = units_module.Library(mutation_directory)
            mutation_library.write("records", "row-key", {"kind": "mutation", "verdict": "caught"})
            mutation_library.write_json("coverage", "coverage-key", {"units": ["abc"]})
            record = mutation_library.read("records", "row-key")
            coverage = mutation_library.read_json("coverage", "coverage-key")
            mutation_ok = (record or {}).get("verdict") == "caught" and coverage == {"units": ["abc"]}
            notes.append(f"mutation-units：记录 {record}、覆盖 {coverage}")

            broad_module = load_by_path("broad_mutation_for_prune_client_cells", os.path.join(REPOSITORY_ROOT, "scripts", "mutation", "broad-mutation.py"))
            broad_library = broad_module.Library(broad_directory)
            broad_library.record_verdict(["a" * 64, "1", "singlefs-core", "自证的一条", "caught", "one", "自证的测试", "b" * 64, "", "0.0", "0.0",
                                          broad_module.today_text(), "fresh", "", "", ""])
            broad_outcomes = [row["outcome"] for row in broad_library.read_verdicts().rows]
            broad_ok = broad_outcomes == ["caught"]
            notes.append(f"broad-mutation：读回的结局 {broad_outcomes}")

            apparatus_store = LibraryStore(FAMILY_APPARATUS_ITEMS, "fake-device", os.path.join(work, "apparatus-root"))
            apparatus_store.write("items/item-key.out", b"judged\n")
            apparatus_ok = apparatus_store.read("items/item-key.out") == b"judged\n"
            notes.append(f"apparatus-items：读回{'对' if apparatus_ok else '不对'}")

            client = process_client()
            prefixes = {family: LibraryStore(family, "", os.path.join(work, "x")).prefix.split(b"@")[0].rstrip(b"/") for family in FAMILIES}
            scanned = {family: [key for key, _value in client.scan(prefixes[family])] for family in FAMILIES}
        distinct = len(set(prefixes.values())) == len(FAMILIES)
        each_own = all(scanned[family] and all(key.startswith(prefixes[family] + b"@") for key in scanned[family]) for family in FAMILIES) \
            and sum(len(keys) for keys in scanned.values()) == len({key for keys in scanned.values() for key in keys})
        local_files = files_under(verdict_directory) + files_under(mutation_directory) + files_under(broad_directory) \
            + [path for path in files_under(admission_common) if path not in before_admission]
        passed = initialized.returncode == 0 and verdict_ok and admission_ok and mutation_ok and broad_ok and apparatus_ok and distinct and each_own and not local_files
        notes.append(f"五族前缀 {sorted(prefixes.values())}（互不相同 {distinct}），各自扫到 { {family: len(keys) for family, keys in scanned.items()} }、"
                     f"都只在自己前缀底下 {each_own}，本地新建的文件 {local_files[:5]}")
        return passed, "；".join(notes)
    finally:
        server.stop()


def cell_unreachable_family_exits_75(binary, work):
    """服务连不上（指到一个没人听的回环端口）：lib_verdict_library 的 lookup 子进程退 75、说连不上，不打「没给判定库」「无记录」那一对，库目录没建。"""
    library_directory = os.path.join(work, "unreachable-root", "singlefs-sample-verdicts")
    closed = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    closed.bind(("127.0.0.1", 0))
    address = "127.0.0.1:%d" % closed.getsockname()[1]
    closed.close()
    code, seconds, output, error = run_child(["verdict-lookup", library_directory],
                                             {SOCKET_VARIABLE: os.path.join(work, "no-such.sock"), TCP_ADDRESS_VARIABLE: address,
                                              RECONNECT_LIMIT_VARIABLE: str(SELFTEST_RECONNECT_SECONDS)}, timeout=SELFTEST_RECONNECT_SECONDS * 8)
    local = files_under(os.path.dirname(library_directory))
    passed = code == EXIT_UNAVAILABLE and not local and "连不上" in error and not output.strip()
    return passed, f"子进程退 {code}（{seconds:.1f} 秒），库目录底下 {local[:3]}，标准输出「{output.strip()}」，标准错误「{error.strip()[-300:]}」"


def filled_unix_listener(path):
    """只听不收的 unix socket，连接队列塞满（再来一条非阻塞连接当场回 EAGAIN 才停）：交 (监听的 socket, 塞进去的连接)。"""
    listener = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    listener.bind(path)
    listener.listen(1)
    fillers = []
    for _ in range(CONNECT_CELL_FILL_ATTEMPTS):
        filler = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        filler.setblocking(False)
        try:
            filler.connect(path)
        except BlockingIOError:
            filler.close()
            return listener, fillers
        fillers.append(filler)
    raise RuntimeError(f"unix socket 塞了 {CONNECT_CELL_FILL_ATTEMPTS} 条连接，队列还没满")


def filled_tcp_listener():
    """只听不收的回环 TCP，连接队列塞满（再来一条连接 CONNECT_CELL_PROBE_SECONDS 秒里握手不完成才停，SYN 被丢）：
    交 (监听的 socket, 塞进去的连接, 「地址:端口」)。"""
    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.bind(("127.0.0.1", 0))
    listener.listen(1)
    host, port = listener.getsockname()
    fillers = []
    for _ in range(CONNECT_CELL_FILL_ATTEMPTS):
        probe = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        probe.setblocking(False)
        probe.connect_ex((host, port))
        _readable, writable, _failed = select.select([], [probe], [], CONNECT_CELL_PROBE_SECONDS)
        if writable and probe.getsockopt(socket.SOL_SOCKET, socket.SO_ERROR) == 0:
            fillers.append(probe)
            continue
        probe.close()
        return listener, fillers, f"{host}:{port}"
    raise RuntimeError(f"回环 TCP 塞了 {CONNECT_CELL_FILL_ATTEMPTS} 条连接，每一条都握手完成了，队列还没满")


def cell_connect_bounded_by_reconnect_total(binary, work):
    """建连只用重连总时长剩下的时间（客户端文件头「容错」）：unix 连接队列满、TCP 握手不完成两种现场，子进程读一次都在总时长加
    CONNECT_CELL_GRACE_SECONDS 秒里退 75、说连不上。两种现场都是这一格自己听着不收的 socket，不起服务（binary 不用）。"""
    del binary
    changes = {RECONNECT_LIMIT_VARIABLE: str(CONNECT_CELL_RECONNECT_SECONDS), RESPONSE_TIMEOUT_VARIABLE: str(CONNECT_CELL_RESPONSE_SECONDS)}
    within = CONNECT_CELL_RECONNECT_SECONDS + CONNECT_CELL_GRACE_SECONDS
    opened, notes, passed = [], [], True
    try:
        unix_path = os.path.join(work, "full.sock")
        unix_listener, unix_fillers = filled_unix_listener(unix_path)
        opened += [unix_listener, *unix_fillers]
        tcp_listener, tcp_fillers, tcp_address = filled_tcp_listener()
        opened += [tcp_listener, *tcp_fillers]
        routes = (("unix 连接队列满", len(unix_fillers), {SOCKET_VARIABLE: unix_path}),
                  ("TCP 握手不完成", len(tcp_fillers), {SOCKET_VARIABLE: os.path.join(work, "no-such.sock"), TCP_ADDRESS_VARIABLE: tcp_address}))
        for what, filled, route in routes:
            code, seconds, output, error = run_child(["read", "connect/any"], dict(changes, **route), timeout=within)
            route_passed = code == EXIT_UNAVAILABLE and seconds < within and "连不上" in error and "absent" not in output
            passed = passed and route_passed
            notes.append(f"{what}（队列里塞了 {filled} 条）：子进程退 {code}（{seconds:.1f} 秒，要 {within} 秒内；None 是超时被停），"
                         f"标准错误「{error.strip()[-200:]}」")
        return passed, "；".join(notes)
    finally:
        for opened_socket in opened:
            with contextlib.suppress(OSError):
                opened_socket.close()


def segment_indices(raw_entries):
    """服务里原样扫回的 [(键, 值)] 里段键形状的段号，升序。"""
    return sorted(parts[1] for parts in (segment_key_parts(key) for key, _value in raw_entries) if parts is not None)


def cell_large_value_round_trip(binary, work):
    """大于帧长上限的值经分段写读往返、扫描只交原键；重写成两段长的值之后旧段号全被新清单点名（客户端文件头「键空间」大值分段）。"""
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        large = random.Random(LARGE_VALUE_SEED).randbytes(LARGE_VALUE_BYTES)
        rewritten = large[:SEGMENT_BYTES + REWRITTEN_TAIL_BYTES]
        with environment_for(unix_environment(server)):
            client = process_client()
            client.write_many([(b"large/one", large), (b"large/small", b"small")])
            values = client.read_many([b"large/one", b"large/small", b"large/missing"])
            round_trip_ok = values[0] == large and values[1:] == [b"small", None]
            scanned = client.scan(b"large/")
            scanned_keys = [key for key, _value in scanned]
            scan_ok = scanned_keys == [b"large/one", b"large/small"] and scanned[0][1] == large
            del values, scanned
            before = segment_indices(client.raw_scan_stream(b"large/one"))
            client.write_many([(b"large/one", rewritten)])
            after = client.read_many([b"large/one"])[0]
            raw_after = dict(client.raw_scan_stream(b"large/one"))
        manifest = manifest_of(b"large/one", raw_after.get(b"large/one"))
        listed = len(manifest["segments"]) if manifest else 0
        after_indices = segment_indices(raw_after.items())
        count_value = raw_after.get(segment_count_key(b"large/one"))
        before_ok = before == list(range(segments_needed(large)))
        rewrite_ok = after == rewritten and manifest is not None and after_indices == list(range(listed)) and count_value == b"%d" % listed
        passed = round_trip_ok and scan_ok and before_ok and rewrite_ok
        return passed, (f"写读 {len(large)} 字节的值逐字节相同 {round_trip_ok}（小值与不在的键一并对 {round_trip_ok}）；按前缀扫交 {scanned_keys}、大值拼回相同 {scan_ok}；"
                        f"服务里的段号 {before}（要 0 到 {segments_needed(large) - 1}）；重写成 {len(rewritten)} 字节之后读回相同 {after == rewritten}、"
                        f"新清单点名 {listed} 段、服务里的段号 {after_indices}、计数键 {count_value!r}（段号要与清单一一对上、计数键等于清单段数）")
    finally:
        server.stop()


def segmented_read_outcome(action):
    """跑一次读：交「退 3（segmented-read）」这类说法与它是不是分段读错。"""
    try:
        result = action()
    except PruneServerFailure as failure:
        return f"退 {failure.code}（{failure.kind}）", failure.kind == "segmented-read"
    if isinstance(result, list):
        result = result[0] if result else None
    return ("交回「不在」" if result is None else f"交回 {len(result) if isinstance(result, bytes) else result} 字节"), False


def cell_segmented_read_incomplete_refused(binary, work):
    """分段存的值拼不全（缺段）、一段短了、一段同长换了字节：点读都退 3（分段拼不回），按前缀扫换了字节的那一把同样退 3。
    协议没有删键，缺段那一把是另起一把键、只写清单与段号 0、2。"""
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        value = random.Random(LARGE_VALUE_SEED + 1).randbytes(SEGMENT_BYTES * 2 + INCOMPLETE_TAIL_BYTES)
        pieces = [value[:SEGMENT_BYTES], value[SEGMENT_BYTES:SEGMENT_BYTES * 2], value[SEGMENT_BYTES * 2:]]
        altered = bytes([pieces[1][0] ^ 0xFF]) + pieces[1][1:]
        with environment_for(unix_environment(server)):
            client = process_client()
            client.write_many([(b"broken/short", value), (b"broken/altered", value)])
            client.write_frame([(b"broken/gap", encode_manifest(value, [len(piece) for piece in pieces])),
                                (segment_key(b"broken/gap", 0), pieces[0]), (segment_key(b"broken/gap", 2), pieces[2])])
            client.write_frame([(segment_key(b"broken/short", 2), pieces[2][:-1])])
            client.write_frame([(segment_key(b"broken/altered", 1), altered)])
            outcomes = {name: segmented_read_outcome(lambda key=key: client.read_many([key]))
                        for name, key in (("缺段", b"broken/gap"), ("短段", b"broken/short"), ("换了字节", b"broken/altered"))}
            outcomes["按前缀扫换了字节的"] = segmented_read_outcome(lambda: [value for _key, value in client.scan(b"broken/altered")])
        passed = all(refused for _shown, refused in outcomes.values())
        return passed, "；".join(f"{name}：{shown}" for name, (shown, _refused) in outcomes.items()) + "（每一种都要退 3（segmented-read））"
    finally:
        server.stop()


REWRITE_MANIFEST_AFTER_SECONDS = 0.3


def cell_segmented_read_sees_a_rewrite(binary, work):
    """同一把键正被重写（客户端文件头「键空间」大值分段「读」）：段已换成新值、清单还是旧的，另一条线程 REWRITE_MANIFEST_AFTER_SECONDS 秒后写新清单；
    点读与按前缀扫都交回新值，不退 3。"""
    server = TestServer(binary, os.path.join(work, "server"))
    old = random.Random(LARGE_VALUE_SEED + 3).randbytes(SEGMENT_BYTES + INCOMPLETE_TAIL_BYTES)
    new = random.Random(LARGE_VALUE_SEED + 4).randbytes(SEGMENT_BYTES + INCOMPLETE_TAIL_BYTES)
    lengths = [SEGMENT_BYTES, INCOMPLETE_TAIL_BYTES]
    try:
        with environment_for(unix_environment(server)):
            client = process_client()
            outcomes = {}
            for name, key, action in (("点读", b"rewritten/read", lambda key: client.read_many([key])[0]),
                                      ("按前缀扫", b"rewritten/scan", lambda key: dict(client.scan(key)).get(key))):
                client.write_many([(key, old)])
                client.write_frame([(segment_key(key, 0), new[:SEGMENT_BYTES]), (segment_key(key, 1), new[SEGMENT_BYTES:])])
                writer = threading.Timer(REWRITE_MANIFEST_AFTER_SECONDS, lambda key=key: client.write_frame([(key, encode_manifest(new, lengths))]))
                writer.start()
                try:
                    result = action(key)
                    outcomes[name] = ("交回新值" if result == new else "交回旧值" if result == old else
                                      "交回「不在」" if result is None else f"交回 {len(result)} 字节的别的值")
                except PruneServerFailure as failure:
                    outcomes[name] = f"退 {failure.code}（{failure.kind}）"
                finally:
                    writer.join()
        passed = all(shown == "交回新值" for shown in outcomes.values())
        return passed, "；".join(f"{name}：{shown}" for name, shown in outcomes.items()) + "（都要交回新值）"
    finally:
        server.stop()


def cell_segment_count_over_index_digits_refused(binary, work):
    """段数段号装不下的值写之前拒（客户端文件头「键空间」大值分段「写」）：段长与段号位数调成 1 字节、1 位（至多 10 段），
    要 11 段的值退 3（too-many-segments）、原键、段键、计数键一条都没写；要 10 段的照写、读回相同。"""
    server = TestServer(binary, os.path.join(work, "server"))
    client_module = sys.modules[Client.__module__]
    saved = (client_module.SEGMENT_BYTES, client_module.SEGMENT_INDEX_DIGITS)
    try:
        with environment_for(unix_environment(server)):
            client = process_client()
            client_module.SEGMENT_BYTES, client_module.SEGMENT_INDEX_DIGITS = 1, 1
            try:
                refused = write_outcome(client, [(b"digits/eleven", b"0123456789A")])
                written = write_outcome(client, [(b"digits/ten", b"0123456789")])
                ten = client.read_many([b"digits/ten"])[0]
                left = client.read_many_raw([b"digits/eleven", client_module.segment_count_key(b"digits/eleven"),
                                             *[client_module.segment_key(b"digits/eleven", index) for index in range(11)]])
            finally:
                client_module.SEGMENT_BYTES, client_module.SEGMENT_INDEX_DIGITS = saved
        stored = [value for value in left if value is not None]
        passed = refused == "too-many-segments" and not stored and written == "written" and ten == b"0123456789"
        return passed, (f"11 段：{refused}、服务里留下 {len(stored)} 条（要 too-many-segments、0 条）；"
                        f"10 段：{written}、读回 {ten!r}（要照写、读回 b'0123456789'）")
    finally:
        client_module.SEGMENT_BYTES, client_module.SEGMENT_INDEX_DIGITS = saved
        server.stop()


SCAN_ROW_OVER_FRAME_CALL_LIMIT = 300


def cell_scan_row_over_frame_refused(_binary, _work):
    """按前缀扫一帧截断而一行都没回（客户端文件头「键空间」大值分段「扫」）：假的服务每一帧都回「截断、0 行」，问了一帧就退 3（scan-row-over-frame）。
    往下拆前缀的（改前的样子）问到 SCAN_ROW_OVER_FRAME_CALL_LIMIT 帧就停下，判红。"""
    calls = []

    class TruncatingEveryFrame(Client):
        def raw_scan(self, prefix):
            calls.append(prefix)
            if len(calls) > SCAN_ROW_OVER_FRAME_CALL_LIMIT:
                raise RuntimeError(f"问了 {len(calls)} 帧还在往下拆前缀")
            return [], True

        def read_many_raw(self, keys):
            return [None for _key in keys]

    try:
        list(TruncatingEveryFrame().raw_scan_stream(b"over-frame/"))
        shown = "扫完了、没退"
        kind = None
    except PruneServerFailure as failure:
        shown, kind = f"退 {failure.code}（{failure.kind}）", failure.kind
    except (RuntimeError, RecursionError) as stopped:
        shown, kind = f"没退：{stopped}", None
    passed = kind == "scan-row-over-frame" and len(calls) == 1
    return passed, f"{shown}，问了 {len(calls)} 帧（要退 3（scan-row-over-frame）、只问 1 帧）"


def write_outcome(client, entries):
    """write_many 一批：交「written」或失败的种类（reserved-key 这类）。"""
    try:
        client.write_many(entries)
    except PruneServerFailure as failure:
        return failure.kind
    return "written"


def cell_segment_key_shapes_only_reserved(binary, work):
    """认段键只认形状（客户端文件头「键空间」认段键只认形状）：带 NUL 而不是段键、计数键形状的原键照写照读照扫，
    一把排在大值原键与它的段键之间的原键不打乱拼回；段键、计数键形状的原键写时整批拒。"""
    server = TestServer(binary, os.path.join(work, "server"))
    big_key = SHAPE_CELL_PREFIX + b"big\x00\x07"
    between_key = big_key + b"\x00\x01"
    big = random.Random(LARGE_VALUE_SEED + 2).randbytes(SEGMENT_BYTES + SHAPE_CELL_TAIL_BYTES)
    stored = {SHAPE_CELL_PREFIX + b"\x00\x00digest\x00": b"label-and-nul", SHAPE_CELL_PREFIX + b"five\x0012345": b"five-digits",
              SHAPE_CELL_PREFIX + b"seven\x001234567": b"seven-digits", SHAPE_CELL_PREFIX + b"counted\x00countdown": b"count-then-more",
              big_key: big, between_key: b"between-owner-and-segments"}
    refused = {"段键形状": (SHAPE_CELL_PREFIX + b"refused\x00000003", SHAPE_CELL_PREFIX + b"alongside-segment-shape"),
               "计数键形状": (SHAPE_CELL_PREFIX + b"refused\x00count", SHAPE_CELL_PREFIX + b"alongside-count-shape")}
    try:
        with environment_for(unix_environment(server)):
            client = process_client()
            stored_outcome = write_outcome(client, list(stored.items()))
            read_ok = client.read_many(list(stored)) == list(stored.values())
            scanned = client.scan(SHAPE_CELL_PREFIX)
            scanned_keys = [key for key, _value in scanned]
            scan_ok = scanned == sorted(stored.items())
            del scanned
            raw_keys = [key for key, _value in client.raw_scan_stream(SHAPE_CELL_PREFIX)]
            refusals = {}
            for name, (reserved_key, alongside_key) in refused.items():
                kind = write_outcome(client, [(alongside_key, b"legal"), (reserved_key, b"shaped")])
                refusals[name] = (kind, client.read_many_raw([reserved_key, alongside_key]))
        big_segment_keys = [segment_key(big_key, index) for index in range(segments_needed(big))]
        stored_raw_ok = all(key in raw_keys for key in [*big_segment_keys, segment_count_key(big_key)])
        interleaved = (all(key in raw_keys for key in (big_key, between_key, big_segment_keys[0]))
                       and raw_keys.index(big_key) < raw_keys.index(between_key) < raw_keys.index(big_segment_keys[0]))
        refused_ok = all(kind == "reserved-key" and values == [None, None] for kind, values in refusals.values())
        passed = stored_outcome == "written" and read_ok and scan_ok and stored_raw_ok and interleaved and refused_ok
        return passed, (f"带 NUL 而不是那两种形状的 {len(stored)} 把原键：写 {stored_outcome}（要 written）、逐字节读回相同 {read_ok}、"
                        f"按前缀扫交 {scanned_keys}、与写进去的逐条相同 {scan_ok}；服务里原样有大值的段键与计数键 {stored_raw_ok}、"
                        f"排在原键与段键之间的那把确实夹在中间 {interleaved}；"
                        + "；".join(f"{name}：写 {kind}、服务里这把与同批合法那条 {['不在' if value is None else '在' for value in values]}"
                                   for name, (kind, values) in refusals.items())
                        + "（都要 reserved-key、两条都不在）")
    finally:
        server.stop()


def cell_oversize_frame_reported_at_once(binary, work):
    """单条超限当场报（客户端文件头「容错」）：子进程写键长超过帧长上限的一条，当场退 3、说帧长上限；进程内把本地上限放宽到两倍照发，
    服务回「帧长度不对」，当场退 3（frame-too-large）、不重连；之后服务照常可用。"""
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        oversize = MAXIMUM_FRAME_BODY_BYTES + 1
        changes = dict(unix_environment(server), **{RECONNECT_LIMIT_VARIABLE: str(SELFTEST_RECONNECT_SECONDS)})
        code, seconds, output, error = run_child(["write-long-key", str(oversize)], changes, timeout=SELFTEST_RECONNECT_SECONDS * 8)
        child_ok = code == EXIT_OTHER and seconds < OVERSIZE_EXIT_WITHIN_SECONDS and "帧长上限" in error and "连不上" not in error and "written" not in output
        with environment_for(changes):
            client = Client()
            client.frame_limit = MAXIMUM_FRAME_BODY_BYTES * 2
            started = time.monotonic()
            try:
                client.write_frame([(b"o" * oversize, b"v")])
                server_side, server_kind = "写成了", ""
            except PruneServerFailure as failure:
                server_side, server_kind = failure.message[-300:], failure.kind
            finally:
                client.close()
            server_seconds = time.monotonic() - started
            process_client().write_many([(b"oversize/after", b"1")])
            usable = process_client().read_many([b"oversize/after"])[0] == b"1"
        server_ok = server_kind == "frame-too-large" and "帧长度不对" in server_side and server_seconds < OVERSIZE_EXIT_WITHIN_SECONDS
        passed = child_ok and server_ok and usable
        return passed, (f"子进程写 {oversize} 字节的键：退 {code}（{seconds:.1f} 秒，要 {OVERSIZE_EXIT_WITHIN_SECONDS} 秒内退 3），标准输出「{output.strip()}」，"
                        f"标准错误「{error.strip()[-300:]}」；本地上限放宽之后照发：{server_kind or '没报错'}（{server_seconds:.1f} 秒）「{server_side}」"
                        f"（要 frame-too-large、说帧长度不对）；之后服务照常读写 {usable}")
    finally:
        server.stop()


def process_state_and_start(process_id):
    """/proc/<进程号>/stat 的状态与起始时刻（时钟滴答，字段 3 与 22）：进程不在交 None。"""
    try:
        with open(f"/proc/{process_id}/stat", encoding="utf-8", errors="replace") as handle:
            text = handle.read()
    except OSError:
        return None
    fields = text[text.rindex(")") + 2:].split()
    return fields[0], fields[19]


def still_running(process_id, start_ticks):
    """这个进程号还是起始时刻为 start_ticks 的那一个、又不是僵尸：换成别的进程、成了僵尸、不在都算不在。"""
    state = process_state_and_start(process_id)
    return state is not None and state[0] not in ("Z", "X") and state[1] == start_ticks


def cell_temporary_service_exits_with_parent(binary, work):
    """临时服务跟着起它的进程走（客户端文件头「命令行」）：子进程起一份 TestServer、打出服务的进程号之后被 SIGKILL（finally 不跑），
    服务要在 PARENT_DEATH_EXIT_WITHIN_SECONDS 秒内不在；还在就由这一格停掉它（只停这一份服务：它自己一个进程组）、判红。"""
    holder = subprocess.Popen([sys.executable, os.path.realpath(__file__), "--selftest-child", "hold-server", binary, os.path.join(work, "held")],
                              stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, env=cell_environment({}), start_new_session=True)
    server_process, start_ticks, survived, seconds = None, None, False, 0.0
    try:
        ready, _writable, _failed = select.select([holder.stdout], [], [], SERVER_READY_SECONDS)
        line = holder.stdout.readline() if ready else ""
        fields = dict(part.split("=", 1) for part in line.split() if "=" in part)
        if not fields.get("server_pid", "").isdigit():
            return False, f"起服务的子进程 {SERVER_READY_SECONDS} 秒内没打出服务的进程号（读到「{line.strip()}」）"
        server_process = int(fields["server_pid"])
        state = process_state_and_start(server_process)
        start_ticks = state[1] if state else None
        os.killpg(holder.pid, signal.SIGKILL)
        holder.wait(timeout=30)
        killed_at = time.monotonic()
        while start_ticks is not None and still_running(server_process, start_ticks) and time.monotonic() - killed_at < PARENT_DEATH_EXIT_WITHIN_SECONDS:
            time.sleep(0.05)
        seconds = time.monotonic() - killed_at
        survived = start_ticks is not None and still_running(server_process, start_ticks)
        passed = start_ticks is not None and not survived
        return passed, (f"起服务的子进程 {holder.pid} 被 SIGKILL 之后，服务 {server_process} "
                        + (f"过了 {seconds:.1f} 秒还在（要 {PARENT_DEATH_EXIT_WITHIN_SECONDS} 秒内不在；这一格停掉了它）" if survived
                           else f"{seconds:.1f} 秒内不在了" if start_ticks is not None else "打出进程号时已经不在（没起成）"))
    finally:
        if holder.poll() is None:
            with contextlib.suppress(ProcessLookupError):
                os.killpg(holder.pid, signal.SIGKILL)
            holder.wait(timeout=30)
        if server_process is not None and start_ticks is not None and still_running(server_process, start_ticks):
            with contextlib.suppress(ProcessLookupError):
                os.killpg(server_process, signal.SIGKILL)
        holder.stdout.close()
        holder.stderr.close()


def cell_service_problem_judges_given_environment(binary, work):
    """selftest_service_problem 给了环境只判那一份（客户端文件头「命令行」）：本进程没有临时服务、交进去的指着活的那一份 ⇒ 空串；
    本进程指着临时服务、交进去的是空的一份 ⇒ 一句为什么。"""
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        given = {SOCKET_VARIABLE: server.socket, TCP_ADDRESS_VARIABLE: server.tcp_address, TEMPORARY_SERVICE_VARIABLE: server.socket}
        with environment_for({}):
            judged_given = selftest_service_problem(given)
        with environment_for(given):
            judged_empty = selftest_service_problem({})
        passed = judged_given == "" and judged_empty != ""
        return passed, (f"本进程没有临时服务、交进去的指着活的那一份：交「{judged_given or '空串'}」（要空串）；"
                        f"本进程指着临时服务、交进去的是空的一份：交「{judged_empty or '空串'}」（要一句为什么）")
    finally:
        server.stop()


def cell_connection_variables_refused_outside_temporary_service(binary, work):
    """改连接地址与时限的变量只在临时服务底下认（客户端文件头「连哪一处」）：临时服务的标记设成空，只设 socket、设 TCP 地址（socket 指到不在的路径）、
    只设重连总时长、只设等回应时限各读一次，都退 3、点名那个变量、什么都没读；标记指回临时服务再读一次照常读到 absent。
    只设 TCP 不设 socket 那一种不造：开关下会落到本机真服务的 socket。只设时限的两趟把 XDG_RUNTIME_DIR 与多机配置指到不在的地方：
    开关下（照认时限）落到「两处都没有」那一句（同样退 3，但不说「只在临时服务底下认」），不连本机真服务。"""
    server = TestServer(binary, os.path.join(work, "server"))
    notes, passed = [], True
    try:
        reconnect = {RECONNECT_LIMIT_VARIABLE: str(SELFTEST_RECONNECT_SECONDS)}
        nowhere = {"XDG_RUNTIME_DIR": os.path.join(work, "no-runtime"), MULTI_HOST_CONFIGURATION_VARIABLE: os.path.join(work, "no-such.env"),
                   TEMPORARY_SERVICE_VARIABLE: ""}
        for what, changes, variable in (
                ("只设 socket", dict(reconnect, **{SOCKET_VARIABLE: server.socket, TEMPORARY_SERVICE_VARIABLE: ""}), SOCKET_VARIABLE),
                ("设 TCP 地址", dict(reconnect, **{SOCKET_VARIABLE: os.path.join(work, "no-such.sock"), TCP_ADDRESS_VARIABLE: server.tcp_address,
                                                   TEMPORARY_SERVICE_VARIABLE: ""}), TCP_ADDRESS_VARIABLE),
                ("只设重连总时长", dict(nowhere, **reconnect), RECONNECT_LIMIT_VARIABLE),
                ("只设等回应时限", dict(nowhere, **{RESPONSE_TIMEOUT_VARIABLE: "5"}), RESPONSE_TIMEOUT_VARIABLE)):
            code, seconds, output, error = run_child(["read", "override/any"], changes, timeout=SELFTEST_RECONNECT_SECONDS * 8)
            refused = code == EXIT_OTHER and variable in error and "只在临时服务底下认" in error and not output.strip()
            passed = passed and refused
            notes.append(f"{what}、不在临时服务底下：退 {code}（要 3），标准输出「{output.strip()}」（要空），标准错误「{error.strip()[-200:]}」")
        code, seconds, output, error = run_child(["read", "override/any"], dict(unix_environment(server), **reconnect))
        honored = code == 0 and output.strip() == "absent"
        passed = passed and honored
        notes.append(f"标记指回临时服务：退 {code}、标准输出「{output.strip()}」（要 0 与 absent）")
        return passed, "；".join(notes)
    finally:
        server.stop()


def cell_namespace_only_under_temporary_service(binary, work):
    """命名空间只在临时服务底下给（客户端文件头「键空间」命名空间）：标记设着时临时目录下两份库目录交两个不同的「@…」、仓里的库目录同样带；
    标记空着时临时目录下的库目录与仓里的都交空串，LibraryStore 的前缀不带「@」。只调客户端的函数，不起服务（binary 不用）。"""
    first, second = os.path.join(work, "first-root", "singlefs-sample-verdicts"), os.path.join(work, "second-root", "singlefs-sample-verdicts")
    in_repository = os.path.join(REPOSITORY_ROOT, ".git", "singlefs-sample-verdicts")
    marked = {TEMPORARY_SERVICE_VARIABLE: os.path.join(work, "server.sock")}
    unmarked = {}
    under_service = [namespace_of(path, marked) for path in (first, second, in_repository)]
    outside_service = [namespace_of(path, unmarked) for path in (first, in_repository)]
    with environment_for({TEMPORARY_SERVICE_VARIABLE: ""}):
        prefix = LibraryStore(FAMILY_UNIT_VERDICTS, "sample-kind", first).prefix
    passed = (all(value.startswith("@") for value in under_service) and len(set(under_service)) == 3
              and outside_service == ["", ""] and prefix == f"{FAMILY_UNIT_VERDICTS}/sample-kind/".encode())
    return passed, (f"临时服务底下三份库目录交 {under_service}（要三个不同的 @…）；不在临时服务底下交 {outside_service}（要两个空串）；"
                    f"不在临时服务底下临时目录里那份的前缀 {prefix!r}（要不带 @）")


def cell_command_line_read(binary, work):
    """命令行 read（客户端文件头「命令行」）：经 LibraryStore 写一条，起客户端 read 读回：在的原样打出、退 0；不在的退 1、标准输出空；认不得的族退 2。"""
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        directory = os.path.join(work, "read-root", "singlefs-sample-verdicts")
        stored = b'{"verdict": "green"}\n'
        with environment_for(unix_environment(server)):
            LibraryStore(FAMILY_UNIT_VERDICTS, "sample-kind", directory).write("items/present.json", stored)
        outcomes = {}
        for what, arguments in (("在的那一条", (FAMILY_UNIT_VERDICTS, "sample-kind", directory, "items/present.json")),
                                ("不在的那一条", (FAMILY_UNIT_VERDICTS, "sample-kind", directory, "items/absent.json")),
                                ("认不得的族", ("no-such-family", "sample-kind", directory, "items/present.json"))):
            outcomes[what] = run_child([], unix_environment(server), program=[sys.executable, CLIENT_PATH, "read", *arguments])
        present, absent, unknown = outcomes["在的那一条"], outcomes["不在的那一条"], outcomes["认不得的族"]
        passed = (present[0] == 0 and present[2] == stored.decode() and absent[0] == 1 and absent[2] == "" and unknown[0] == EXIT_USAGE)
        return passed, (f"在的那一条：退 {present[0]}、标准输出「{present[2].strip()}」（要 0 与写进去的值）；不在的那一条：退 {absent[0]}、标准输出「{absent[2].strip()}」"
                        f"（要 1 与空）；认不得的族：退 {unknown[0]}（要 2）；标准错误「{(present[3] + absent[3]).strip()[-200:]}」")
    finally:
        server.stop()


def cell_batched_read_answer_over_frame_limit_split(binary, work):
    """回应超过帧长上限就拆键（客户端文件头「容错」）：OVER_LIMIT_VALUE_COUNT 把各正好一段长（SEGMENT_BYTES，不分段存）的值合计超过帧长上限，
    中间夹一条小值与一把不在的键，一次 read_many 读全：逐字节对、次序与问的相同；那几把都没有计数键（真没分段，单键回应装得下一帧）。"""
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        generator = random.Random(LARGE_VALUE_SEED + 2)
        sized = [(b"split/sized-%d" % index, generator.randbytes(SEGMENT_BYTES)) for index in range(OVER_LIMIT_VALUE_COUNT)]
        stored = dict(sized)
        stored[b"split/small"] = b"small"
        total = sum(len(value) for _key, value in sized)
        asked = [key for key, _value in sized]
        asked[2:2] = [b"split/small"]
        asked[4:4] = [b"split/missing"]
        expected = [stored.get(key) for key in asked]
        with environment_for(unix_environment(server)):
            client = process_client()
            client.write_many(sized + [(b"split/small", b"small")])
            count_values = client.read_many_raw([segment_count_key(key) for key, _value in sized])
            try:
                values = client.read_many(asked)
                outcome = "读完"
            except PruneServerFailure as failure:
                values, outcome = None, f"退 {failure.code}（{failure.kind}）"
        unsegmented = count_values == [None] * len(sized)
        matched = values == expected
        shown = "-" if values is None else [len(value) if value is not None else None for value in values]
        passed = total > MAXIMUM_FRAME_BODY_BYTES and unsegmented and matched
        return passed, (f"{len(sized)} 把各 {SEGMENT_BYTES} 字节的值合计 {total} 字节（帧长上限 {MAXIMUM_FRAME_BODY_BYTES}，要超过）、计数键都不在 {unsegmented}；"
                        f"一次读 {len(asked)} 把键：{outcome}，各值长 {shown}，逐字节、次序与问的相同 {matched}（要读完且相同）")
    finally:
        server.stop()


def cell_binary_not_built_without_target_directory(_binary, _work):
    """没给服务二进制、环境里又没有任务服务给的编译目录时 resolve_binary 不起 cargo、交 (None, 为什么)，为什么点名 --server-binary；
    有编译目录时才走 build_server_binary。build_server_binary 在这一格里换成只记调用的替身，不真起 cargo。"""
    calls = []
    original = globals()["build_server_binary"]
    globals()["build_server_binary"] = lambda: (calls.append("built"), ("/built/prune-server", ""))[1]
    saved = {name: os.environ.pop(name, None) for name in (BINARY_VARIABLE, CARGO_TARGET_DIRECTORY_VARIABLE)}
    try:
        without_directory = resolve_binary([])
        calls_without_directory = list(calls)
        os.environ[CARGO_TARGET_DIRECTORY_VARIABLE] = "/service/given/target"
        with_directory = resolve_binary([])
        calls_with_directory = list(calls)
    finally:
        globals()["build_server_binary"] = original
        os.environ.pop(CARGO_TARGET_DIRECTORY_VARIABLE, None)
        for name, value in saved.items():
            if value is not None:
                os.environ[name] = value
    passed = (calls_without_directory == [] and without_directory[0] is None and "--server-binary" in without_directory[1]
              and calls_with_directory == ["built"] and with_directory == ("/built/prune-server", ""))
    return passed, (f"没有编译目录：起了 cargo {len(calls_without_directory)} 次（要 0）、交 {without_directory!r}（要 None 加点名 --server-binary 的说明）；"
                    f"有编译目录：起了 cargo {len(calls_with_directory)} 次（要 1）、交 {with_directory!r}")


def cell_delete_prefix_by_namespace(binary, work):
    """删前缀（客户端文件头「命令行」delete-prefix）在临时服务底下照命名空间删：两份库目录（命名空间不同）在同一个族同一个种类底下各写几条，
    对其中一份的相对前缀先只数不删（数到的条数对、库不动）、再删（删掉的条数相同）、再数是 0；这一份相对前缀之外的键与另一份的键一条不少。
    命令行 delete-prefix --dry-run 与不带它对另一份各起一次：打 PRUNE_SERVER_DELETE_PREFIX 行，数到与删掉的条数对上，删完那一份空。"""
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        first_directory = os.path.join(work, "first-root", "singlefs-sample-verdicts")
        second_directory = os.path.join(work, "second-root", "singlefs-sample-verdicts")
        with environment_for(unix_environment(server)):
            first = LibraryStore(FAMILY_UNIT_VERDICTS, "sample-kind", first_directory)
            second = LibraryStore(FAMILY_UNIT_VERDICTS, "sample-kind", second_directory)
            first.write_many([("items/a.json", b"1"), ("items/b.json", b"2"), ("items/c.json", b"3"), ("other/d.json", b"4")])
            second.write_many([("items/a.json", b"x"), ("items/b.json", b"y")])
            counted = first.delete_prefix("items/", count_only=True)
            after_counting = [relative for relative, _value in first.scan("")]
            deleted = first.delete_prefix("items/")
            counted_again = first.delete_prefix("items/", count_only=True)
            first_left = [relative for relative, _value in first.scan("")]
            second_left = [relative for relative, _value in second.scan("")]
        outcomes = {}
        for what, extra in (("数", ["--dry-run"]), ("删", [])):
            outcomes[what] = run_child([], unix_environment(server), program=[sys.executable, CLIENT_PATH, "delete-prefix", *extra,
                                                                              FAMILY_UNIT_VERDICTS, "sample-kind", second_directory, "items/"])
        counted_line, deleted_line = outcomes["数"][2], outcomes["删"][2]
        raw_keys = [b"\x1bhex-cell/a", b"\x1bhex-cell/b", b"\x1chex-cell/c", b"\x1dhex-cell/kept"]
        with environment_for(unix_environment(server)):
            second_left_after_command = [relative for relative, _value in second.scan("")]
            process_client().write_many([(key, b"raw") for key in raw_keys])
        for what, extra in (("原样数", ["--dry-run"]), ("原样删", [])):
            outcomes[what] = run_child([], unix_environment(server), program=[sys.executable, CLIENT_PATH, "delete-prefix", *extra, "--hex", "1b", "--hex", "1c"])
        with environment_for(unix_environment(server)):
            raw_left = [key for key, value in zip(raw_keys, process_client().read_many(raw_keys)) if value is not None]
        raw_counted_text, raw_deleted_text = outcomes["原样数"][2], outcomes["原样删"][2]
        raw_passed = (outcomes["原样数"][0] == 0 and outcomes["原样删"][0] == 0 and "PRUNE_SERVER_DELETE_PREFIX_TOTAL mode=dry-run prefixes=2 keys=3" in raw_counted_text
                      and "PRUNE_SERVER_DELETE_PREFIX_TOTAL mode=deleted prefixes=2 keys=3" in raw_deleted_text and raw_left == [raw_keys[3]])
        passed = (raw_passed and counted == 3 and after_counting == ["items/a.json", "items/b.json", "items/c.json", "other/d.json"] and deleted == 3 and counted_again == 0
                  and first_left == ["other/d.json"] and second_left == ["items/a.json", "items/b.json"]
                  and outcomes["数"][0] == 0 and outcomes["删"][0] == 0 and "mode=dry-run" in counted_line and "keys=2" in counted_line
                  and "mode=deleted" in deleted_line and "keys=2" in deleted_line and second_left_after_command == [])
        return passed, (f"这一份：数到 {counted}（要 3）、数完库里 {after_counting}、删掉 {deleted}（要 3）、再数 {counted_again}（要 0）、剩 {first_left}（要只剩 other/d.json）；"
                        f"另一份命名空间的键删之前还有 {second_left}（要两条都在）；命令行：数 退 {outcomes['数'][0]}「{counted_line.strip()}」、删 退 {outcomes['删'][0]}"
                        f"「{deleted_line.strip()}」（要都退 0、行里 keys=2）、删完另一份剩 {second_left_after_command}（要空）；原样键前缀 --hex 1b --hex 1c：数 退 {outcomes['原样数'][0]}、删 退 {outcomes['原样删'][0]}、"
                        f"总数行要是 prefixes=2 keys=3、删完剩 {raw_left}（要只剩 0x1d 起头那一把）；标准错误「{(outcomes['数'][3] + outcomes['删'][3]).strip()[-200:]}」")
    finally:
        server.stop()


def cell_delete_by_item_name_picks_by_value(binary, work):
    """按值挑键删（客户端文件头「命令行」delete-by-item-name）：临时服务底下两个种类各写几条判定记录（JSON，item 字段是项名；一条大到分段存、一条不是 JSON），
    名单点名其中一个种类的两个项名（一个项名有两条记录、一个是分段存的）与另一个种类的一个不存在的项名。命令行 --dry-run：退 0、逐条打键与项名、matched 对、库不动；
    不带 --dry-run：退 0、点名种类里项名命中的几条没了（含分段的），同种类里别的项、不是 JSON 的条目与另一个种类里同名的项一条不少；再 --dry-run 是 matched=0；
    名单里没找到记录的（种类、项名）逐条点名；用法错（缺名单文件）退 2。"""
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        directory = os.path.join(work, "root", "singlefs-sample-verdicts")
        named_kind, other_kind = "sample-stage.sh:replay", "other-stage.sh:replay"
        list_path = os.path.join(work, "names.tsv")
        with open(list_path, "w", encoding="utf-8") as handle:
            handle.write("# 种类\t项名\n" + f"{named_kind}\tsample-sparrow\n{named_kind}\tsample-heron\n{other_kind}\tsample-nobody\n")
        large_record = json.dumps({"verdict": "green", "item": "sample-heron", "padding": "x" * (SEGMENT_BYTES + 10)}).encode()

        def record(item):
            return json.dumps({"verdict": "green", "item": item}).encode()

        with environment_for(unix_environment(server)):
            named = LibraryStore(FAMILY_UNIT_VERDICTS, named_kind, directory)
            other = LibraryStore(FAMILY_UNIT_VERDICTS, other_kind, directory)
            named.write_many([("keys/aaa.json", record("sample-sparrow")), ("keys/bbb.json", record("sample-sparrow")), ("keys/ccc.json", record("real-lighthouse")),
                              ("keys/fff.json", large_record), ("items/xyz", b"not-json")])
            other.write_many([("keys/ddd.json", record("sample-sparrow")), ("keys/eee.json", record("real-harbor"))])
            before = (sorted(relative for relative, _value in named.scan("")), sorted(relative for relative, _value in other.scan("")))
        environment = unix_environment(server)
        program = [sys.executable, CLIENT_PATH, "delete-by-item-name"]
        counted = run_child([], environment, program=[*program, "--dry-run", FAMILY_UNIT_VERDICTS, directory, list_path])
        with environment_for(unix_environment(server)):
            after_counting = (sorted(relative for relative, _value in named.scan("")), sorted(relative for relative, _value in other.scan("")))
        deleted = run_child([], environment, program=[*program, FAMILY_UNIT_VERDICTS, directory, list_path])
        with environment_for(unix_environment(server)):
            named_left = sorted(relative for relative, _value in named.scan(""))
            other_left = sorted(relative for relative, _value in other.scan(""))
            segments_left = [key for key, _value in process_client().raw_scan_stream(named.key("keys/fff.json"))]
        counted_again = run_child([], environment, program=[*program, "--dry-run", FAMILY_UNIT_VERDICTS, directory, list_path])
        misuse = run_child([], environment, program=[*program, FAMILY_UNIT_VERDICTS, directory])
        counted_text, deleted_text, again_text = counted[2], deleted[2], counted_again[2]
        named_prefix = named.prefix.decode()
        expected_rows = [f"kind={named_kind} keys=1 key={named_prefix}keys/aaa.json item=sample-sparrow", f"kind={named_kind} keys=1 key={named_prefix}keys/bbb.json item=sample-sparrow"]
        passed = (counted[0] == 0 and deleted[0] == 0 and counted_again[0] == 0 and misuse[0] == EXIT_USAGE
                  and all(row in counted_text and row in deleted_text for row in expected_rows)
                  and f"kind={named_kind} keys=" in counted_text and f"key={named_prefix}keys/fff.json item=sample-heron" in counted_text
                  and "mode=dry-run kinds=2 matched=3 " in counted_text and "mode=deleted kinds=2 matched=3 " in deleted_text and "names_not_found=1" in counted_text
                  and f"PRUNE_SERVER_DELETE_BY_ITEM_NAME_NOT_FOUND kind={other_kind} item=sample-nobody" in counted_text
                  and "mode=dry-run kinds=2 matched=0 " in again_text
                  and after_counting == before
                  and named_left == ["items/xyz", "keys/ccc.json"] and other_left == ["keys/ddd.json", "keys/eee.json"] and segments_left == [])
        return passed, (f"命令行 --dry-run 退 {counted[0]}「{counted_text.strip()[-500:]}」、数完库里 {after_counting}（要和写入后的 {before} 一样）；"
                        f"删 退 {deleted[0]}「{deleted_text.strip()[-300:]}」、点名种类剩 {named_left}（要 items/xyz、keys/ccc.json）、另一个种类剩 {other_left}（要两条都在）、"
                        f"分段存的那条的原键与段键剩 {segments_left}（要空）；再数 退 {counted_again[0]}「{again_text.strip()[-200:]}」（要 matched=0）；"
                        f"缺名单文件退 {misuse[0]}（要 {EXIT_USAGE}）；标准错误「{(counted[3] + deleted[3]).strip()[-200:]}」")
    finally:
        server.stop()


CELL_FUNCTIONS = {"round-trip-unix": cell_round_trip_unix, "prefix-scan": cell_prefix_scan,
                  "tcp-round-trip": cell_tcp_round_trip,
                  "busy-retried-after-delay": cell_busy_retried_after_delay, "stopped-exits-75": cell_stopped_exits_75,
                  "unreachable-exits-75": cell_unreachable_exits_75, "families-over-service": cell_families_over_service,
                  "unreachable-family-exits-75": cell_unreachable_family_exits_75,
                  "connect-bounded-by-reconnect-total": cell_connect_bounded_by_reconnect_total,
                  "large-value-round-trip": cell_large_value_round_trip,
                  "segmented-read-incomplete-refused": cell_segmented_read_incomplete_refused,
                  "segmented-read-sees-a-rewrite": cell_segmented_read_sees_a_rewrite,
                  "segment-count-over-index-digits-refused": cell_segment_count_over_index_digits_refused,
                  "scan-row-over-frame-refused": cell_scan_row_over_frame_refused,
                  "segment-key-shapes-only-reserved": cell_segment_key_shapes_only_reserved,
                  "oversize-frame-reported-at-once": cell_oversize_frame_reported_at_once,
                  "temporary-service-exits-with-parent": cell_temporary_service_exits_with_parent,
                  "service-problem-judges-given-environment": cell_service_problem_judges_given_environment,
                  "connection-variables-refused-outside-temporary-service": cell_connection_variables_refused_outside_temporary_service,
                  "command-line-read": cell_command_line_read, "namespace-only-under-temporary-service": cell_namespace_only_under_temporary_service,
                  "batched-read-answer-over-frame-limit-split": cell_batched_read_answer_over_frame_limit_split,
                  "binary-not-built-without-target-directory": cell_binary_not_built_without_target_directory,
                  "delete-prefix-by-namespace": cell_delete_prefix_by_namespace,
                  "delete-by-item-name-picks-by-value": cell_delete_by_item_name_picks_by_value}


def resolve_binary(arguments):
    if "--server-binary" in arguments:
        index = arguments.index("--server-binary")
        if index + 1 >= len(arguments):
            return None, "--server-binary 后面缺路径"
        return os.path.abspath(arguments[index + 1]), ""
    if os.environ.get(BINARY_VARIABLE):
        return os.path.abspath(os.environ[BINARY_VARIABLE]), ""
    if not os.environ.get(CARGO_TARGET_DIRECTORY_VARIABLE) and not break_is("build-without-target-directory"):
        return None, (f"没给 --server-binary 与 {BINARY_VARIABLE}，环境里也没有任务服务给的编译目录（{CARGO_TARGET_DIRECTORY_VARIABLE}）：不现编，"
                      "现编会把二进制编进主仓的 prune-server/target")
    return build_server_binary()


def selftest(arguments):
    os.environ[SELFTEST_VARIABLE] = "1"
    # 三家照这个名字载入客户端：让它们用的就是这一进程里已经 import 的那一份客户端模块（同一个进程级连接，environment_for 换服务时一起丢掉）
    sys.modules.setdefault("singlefs_prune_server_client", sys.modules["prune_server_client"])
    binary, problem = resolve_binary(arguments)
    if binary is None or not os.access(binary, os.X_OK):
        print(f"  ✗ 自证起不了服务：{problem or binary + ' 不可执行'}")
        print(f"     → 怎么办：--server-binary <编好的 prune-server 二进制> 或 {BINARY_VARIABLE}=<路径>；只有任务服务给了 {CARGO_TARGET_DIRECTORY_VARIABLE} 才在这里现编")
        return EXIT_USAGE
    failed = []
    for name in CELLS:
        work = selftest_root()
        try:
            passed, detail = CELL_FUNCTIONS[name](binary, work)
        except (RuntimeError, OSError, subprocess.SubprocessError, PruneServerFailure) as error:
            passed, detail = False, f"这一格的现场出错：{type(error).__name__}：{error}"
        finally:
            shutil.rmtree(work, ignore_errors=True)
        if passed:
            print(f"  ✓ 自证 {name}：{detail}")
        else:
            failed.append(name)
            print(f"  ✗ 自证 {name}：{detail}")  # gate-lint:detail
    print(f"PRUNE_SERVER_CLIENT_SELFTEST cells={len(CELLS)} passed={len(CELLS) - len(failed)} failed={len(failed)} failed_cells={','.join(failed) or '-'}")
    if failed:
        return 1
    if os.environ.get(BREAK_VARIABLE):
        return 0
    matched, unmatched = 0, []
    for switch, red_cell in BREAK_RED_CELLS:
        completed = subprocess.run([sys.executable, os.path.realpath(__file__), "--selftest", "--server-binary", binary], capture_output=True, text=True,
                                   env=dict(os.environ, **{BREAK_VARIABLE: switch}))
        if completed.returncode != 0 and f"✗ 自证 {red_cell}：" in completed.stdout:
            matched += 1
        else:
            unmatched.append(f"{switch}（要红在 {red_cell}，退 {completed.returncode}）")
    print(f"PRUNE_SERVER_CLIENT_BREAK_SWEEP switches={len(BREAK_RED_CELLS)} matched={matched} unmatched={'；'.join(unmatched) or '-'}")
    if unmatched:
        print("  ✗ 弄坏开关打开之后没红在点名那一格")
        print("     → 怎么办：那几个开关管的那一段没有自证盯着：补一格会被它弄错的自证，或修开关")
        return 1
    return 0


@contextlib.contextmanager
def temporary_service(arguments):
    """起一份临时服务（二进制照 resolve_binary），连哪一处（socket 与 TCP 都指它，不落到多机配置里的真服务）与 TEMPORARY_SERVICE_VARIABLE
    设进本进程的环境，交 (服务, "")；起不了交 (None, 为什么)。收尾停服务、删临时目录、环境照原样还回去（客户端文件头「命令行」--with-temporary-service）。"""
    binary, problem = resolve_binary(arguments)
    if binary is None or not os.access(binary, os.X_OK):
        yield None, problem or f"{binary} 不可执行"
        return
    work = selftest_root()
    changed = (SOCKET_VARIABLE, TCP_ADDRESS_VARIABLE, TEMPORARY_SERVICE_VARIABLE, BINARY_VARIABLE)
    saved = {name: os.environ.get(name) for name in changed}
    server = None
    try:
        try:
            server = TestServer(binary, os.path.join(work, "server"))
        except (RuntimeError, OSError, subprocess.SubprocessError) as error:
            yield None, f"临时服务起不来：{type(error).__name__}：{error}"
            return
        os.environ.update({SOCKET_VARIABLE: server.socket, TCP_ADDRESS_VARIABLE: server.tcp_address,
                           TEMPORARY_SERVICE_VARIABLE: server.socket, BINARY_VARIABLE: binary})
        reset_process_client()
        yield server, ""
    finally:
        reset_process_client()
        for name, value in saved.items():
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value
        if server is not None:
            server.stop()
        shutil.rmtree(work, ignore_errors=True)


def with_temporary_service(arguments):
    """--with-temporary-service [--server-binary <…>] -- <命令…>：在一份临时服务底下起那条命令，交它的退出码（客户端文件头「命令行」）。"""
    if "--" not in arguments or arguments.index("--") + 1 >= len(arguments):
        print("  ✗ --with-temporary-service 要在 -- 后面写要起的命令", file=sys.stderr)
        print("     → 怎么办：--with-temporary-service [--server-binary <prune-server 二进制>] -- python3 scripts/crash/lib_verdict_library.py --selftest", file=sys.stderr)
        return EXIT_USAGE
    separator = arguments.index("--")
    options, command = arguments[:separator], arguments[separator + 1:]
    unknown = [word for position, word in enumerate(options) if word != "--server-binary" and (position == 0 or options[position - 1] != "--server-binary")]
    if unknown:
        print(f"  ✗ --with-temporary-service 认不出 {' '.join(unknown)}", file=sys.stderr)
        print("     → 怎么办：-- 之前只写 --server-binary <prune-server 二进制>，要起的命令写在 -- 后面", file=sys.stderr)
        return EXIT_USAGE
    with temporary_service(options) as (server, problem):
        if server is None:
            print(f"  ✗ 临时 prune-server 起不来：{problem}", file=sys.stderr)
            print(f"     → 怎么办：--server-binary <编好的 prune-server 二进制> 或 {BINARY_VARIABLE}=<路径>；只有任务服务给了 {CARGO_TARGET_DIRECTORY_VARIABLE} 才在这里现编", file=sys.stderr)
            return EXIT_USAGE
        completed = subprocess.run(command, cwd=os.getcwd())
        server_alive = server.process.poll() is None
    print(f"PRUNE_SERVER_TEMPORARY_SERVICE exit={completed.returncode} server_alive_at_end={int(server_alive)}")
    return completed.returncode


def selftest_families(arguments):
    """起一份临时服务，逐份对它跑三家与旁路的原有自证（文件头「命令行」）。"""
    failed = []
    families = family_selftests()
    with temporary_service(arguments) as (server, problem):
        if server is None:
            print(f"  ✗ 起不了服务：{problem}")
            print(f"     → 怎么办：--server-binary <prune-server 二进制> 或 {BINARY_VARIABLE}=<路径>")
            return EXIT_USAGE
        # 连哪一处与临时服务的标记照 temporary_service 设的带过去；别的几个连法与库根的变量不带（各家自证照自己的写法取）
        environment = {name: value for name, value in os.environ.items()
                       if name in (SOCKET_VARIABLE, TCP_ADDRESS_VARIABLE, TEMPORARY_SERVICE_VARIABLE) or name not in ENVIRONMENT_CLEARED_FOR_CELLS}
        for script, command in families:
            started = time.monotonic()
            completed = subprocess.run(list(command), cwd=REPOSITORY_ROOT, capture_output=True, text=True, env=environment)
            print(f"FAMILY_SELFTEST script={script} exit={completed.returncode} seconds={time.monotonic() - started:.0f}")
            if completed.returncode != 0:
                failed.append(script)
                lines = (completed.stdout + completed.stderr).strip().splitlines()
                print("    " + "\n    ".join([line for line in lines if "✗" in line][:20] or lines[-6:]))  # gate-lint:detail
        server_alive = server.process.poll() is None
    print(f"PRUNE_SERVER_FAMILY_SELFTESTS scripts={len(families)} passed={len(families) - len(failed)} failed={len(failed)} "
          f"failed_scripts={','.join(failed) or '-'} server_alive_at_end={int(server_alive)}")
    return 1 if failed else 0


def main(arguments):
    if arguments[:1] == ["--selftest"]:
        return selftest(arguments[1:])
    if arguments[:1] == ["--selftest-families"]:
        return selftest_families(arguments[1:])
    if arguments[:1] == ["--with-temporary-service"]:
        return with_temporary_service(arguments[1:])
    if arguments[:1] == ["--selftest-child"]:
        return selftest_child(arguments[1:])
    print(f"  ✗ 认不出的用法：{' '.join(arguments) or '（没给参数）'}", file=sys.stderr)
    print("     → 怎么办：--selftest [--server-binary <…>]、--selftest-families [--server-binary <…>] 或 "
          "--with-temporary-service [--server-binary <…>] -- <命令…>", file=sys.stderr)
    return EXIT_USAGE


if __name__ == "__main__":
    sys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "..", ".claude", "scripts"))
    from project_preflight import preflight
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
