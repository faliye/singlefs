"""prune_server_client_selftest.py 的一块：自证起的临时服务（编二进制与找二进制 resolve_binary、认这一台在多机配置里的地址、TestServer、各格与子进程的环境、起子进程的 run_child、selftest_child）与 --with-temporary-service、--selftest-families 的起法，以及测二进制怎么来的那一格 binary-not-built-without-target-directory（它换掉本模块里的 build_server_binary，所以和被测的两个函数放在一起）。入口 prune_server_client_selftest.py 导入并再导出这里的名字。"""
import contextlib
import importlib.util
import json
import os
import random
import shutil
import signal
import socket
import subprocess
import sys
import tempfile
import threading
import time
from prune_server_client import (
    BINARY_VARIABLE, Client, EXIT_USAGE, MULTI_HOST_CONFIGURATION_VARIABLE, OPERATION_ANNOUNCE_STOP, RECONNECT_LIMIT_VARIABLE,
    REPOSITORY_ROOT, RESPONSE_TIMEOUT_VARIABLE, SEGMENT_MANIFEST_MAGIC, SELFTEST_VARIABLE, SOCKET_VARIABLE, TCP_ADDRESS_VARIABLE,
    TEMPORARY_SERVICE_VARIABLE, break_is, encode_frame, process_client, read_response, request_body, reset_process_client
)


ENTRY_SCRIPT_PATH = os.path.join(os.path.dirname(os.path.realpath(__file__)), "prune_server_client_selftest.py")   # 自证起子进程要跑的入口；拆出来的模块里 __file__ 不是入口


CARGO_TARGET_DIRECTORY_VARIABLE = "CARGO_TARGET_DIR"   # 任务服务给的编译目录；没有它就不现编服务二进制（resolve_binary）
SERVER_READY_SECONDS = 120
CHILD_TIMEOUT_SECONDS = 60
STOP_EXIT_WITHIN_SECONDS = 10

LISTENING_LINE_PREFIX = "PRUNE_SERVER_LISTENING"
MAXIMUM_SOCKET_PATH_BYTES = 107

HOLD_SERVER_SECONDS = CHILD_TIMEOUT_SECONDS   # 同一格：起服务的子进程打出进程号之后至多等这么久（这一格在那之前就 SIGKILL 它）

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


RUN_CONDITION_EXIT = 78   # 运行条件不满足：多机配置 HOSTS 读不到、认不出这一台
MULTI_HOST_ADDRESSES_SCRIPT = os.path.join(REPOSITORY_ROOT, "scripts", "hosts", "multi_host_addresses.py")
THIS_HOST_ADDRESS_CACHE = []   # this_host_address 在本进程里取到的地址（取一次，一个进程里的几份临时服务不各起一遍多机配置判法）


class HostAddressUnavailable(RuntimeError):
    """多机配置 HOSTS 读不到，或认不出哪一项是这一台（临时服务的 TCP 要绑这一台的地址，不补回环）。"""


def this_host_address():
    """这一台在多机配置 HOSTS 里的地址：照 scripts/hosts/multi_host_addresses.py 的 resolve（整份配置判过、这一台照网卡上的地址认）取，
    取不到抛 HostAddressUnavailable（写明原因与出路）。不另设变量、不补缺省。"""
    if THIS_HOST_ADDRESS_CACHE:
        return THIS_HOST_ADDRESS_CACHE[0]
    specification = importlib.util.spec_from_file_location("multi_host_addresses_for_the_temporary_service", MULTI_HOST_ADDRESSES_SCRIPT)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    try:
        address = module.resolve().this
    except module.AddressesUnavailable as error:
        raise HostAddressUnavailable(f"{error}（{module.HOWTO}）") from error
    THIS_HOST_ADDRESS_CACHE.append(address)
    return address


def tcp_bind_address_of_this_host():
    """临时服务 --tcp 绑的地址 <这一台在 HOSTS 里的地址>:0（端口由内核挑；IPv6 地址带方括号）：两台的 GPU 批处理服务都连得到，不绑回环；取不到抛 HostAddressUnavailable。
    弄坏开关 temporary-service-binds-loopback 下不取、一律 127.0.0.1:0；temporary-service-loopback-when-hosts-unreadable 下取不到时退回 127.0.0.1:0。"""
    if break_is("temporary-service-binds-loopback"):
        return "127.0.0.1:0"
    try:
        address = this_host_address()
    except HostAddressUnavailable:
        if break_is("temporary-service-loopback-when-hosts-unreadable"):
            return "127.0.0.1:0"
        raise
    return f"[{address}]:0" if ":" in address else f"{address}:0"


def hosts_refusal(stream):
    """起临时服务之前先核 HOSTS 读得到：读得到交 None；读不到往 stream 打 ✗ 与出路、交退出码 78（运行条件不满足），一条命令都还没起。"""
    try:
        tcp_bind_address_of_this_host()
    except HostAddressUnavailable as error:
        print(f"  ✗ 临时 prune-server 的 TCP 要绑多机配置 HOSTS 里这一台的地址，读不到：{error}", file=stream)
        print("     → 怎么办：照仓根 multi-host.env.example 写好 HOSTS（含这一台网卡上的地址），或设 SINGLEFS_MULTI_HOST_CONFIG 指到它；不退回回环", file=stream)
        return RUN_CONDITION_EXIT
    return None


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
    """一格一份真 prune-server：socket、状态目录、库、密钥都在这一格的临时目录里；TCP 绑多机配置 HOSTS 里这一台的地址、端口由内核挑
    （tcp_bind_address_of_this_host，两台的 GPU 批处理服务都连得到；取不到抛 HostAddressUnavailable，不退回回环）；它自己一个进程组，收尾对整组发信号。
    经 PARENT_DEATH_LAUNCHER 起：起它的进程被硬杀、收尾不跑时内核停服务（客户端文件头「命令行」临时服务跟着起它的进程走）；
    父进程死亡信号认的是建它的那一条线程，只在活到收尾的线程里建。弄坏开关 temporary-service-outlives-parent 下直接起二进制（改前的样子）。
    对它那一组发信号只经 signal_group（在 __init__ 里闭包着刚起的这个子进程，进程组号就是它的进程号）：发信号的地方不去碰别处来的进程号。"""

    def __init__(self, binary, directory, extra=()):
        tcp_bind_address = tcp_bind_address_of_this_host()   # 先取：取不到抛 HostAddressUnavailable，这时目录与文件都还没建
        os.makedirs(directory)
        self.directory = directory
        self.socket = os.path.join(directory, "server.sock")
        self.library = os.path.join(directory, "library")
        self.standard_error = open(os.path.join(directory, "server.stderr"), "w", encoding="utf-8")
        command = [binary, "serve", "--selftest-instance", "--socket", self.socket, "--state-directory", directory,
                   "--library", self.library, "--tcp", tcp_bind_address, "--worker-threads", "2", *extra]
        if not break_is("temporary-service-outlives-parent"):
            command = [sys.executable, "-I", "-c", PARENT_DEATH_LAUNCHER, str(os.getpid()), *command]
        process = subprocess.Popen(command,
                                   stdout=subprocess.PIPE, stderr=self.standard_error, text=True, start_new_session=True,
                                   env={name: value for name, value in os.environ.items() if not name.startswith("PRUNE_SERVER_BREAK")})
        self.process = process

        def signal_group(signal_number):
            """给这一份服务自己的进程组发信号（signal_number 为 0 只探活）。"""
            os.killpg(process.pid, signal_number)

        self.signal_group = signal_group
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
                self.signal_group(signal.SIGKILL)
            self.process.wait(timeout=30)

    def stop(self):
        if self.process.poll() is None:
            with contextlib.suppress(ProcessLookupError):
                self.signal_group(signal.SIGTERM)
            try:
                self.process.wait(timeout=30)
            except subprocess.TimeoutExpired:
                self.kill()
        with contextlib.suppress(ProcessLookupError, PermissionError):
            self.signal_group(0)
            self.signal_group(signal.SIGKILL)
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
    process = subprocess.Popen(program or [sys.executable, ENTRY_SCRIPT_PATH, "--selftest-child", *arguments], stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, text=True, env=environment, start_new_session=True)
    try:
        output, error = process.communicate(timeout=timeout)
        return process.returncode, time.monotonic() - started, output, error
    except subprocess.TimeoutExpired:
        with contextlib.suppress(ProcessLookupError):
            os.killpg(process.pid, signal.SIGKILL)
        output, error = process.communicate(timeout=STOP_EXIT_WITHIN_SECONDS)
        return None, time.monotonic() - started, output, error


def selftest_child(arguments):
    """自证起的子进程：read <键> 读一个键打「absent」或「present」；verdict-lookup <目录> 跑 lib_verdict_library 的 lookup、打它交的那一对；
    write-long-key <字节数> 写一条键长这么多字节的条目，写成了打「written」；write-dies-before-manifest <键> <种子> <字节数> 照 write_many 写这个种子生成的值，
    写到清单那一帧之前 SIGKILL 自己（走完了才打「written」）；hold-server <二进制> <目录> 起一份 TestServer、打「server_pid=<进程号>」，
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
    if arguments[:1] == ["write-dies-before-manifest"] and len(arguments) == 4 and arguments[2].isdigit() and arguments[3].isdigit():
        value = random.Random(int(arguments[2])).randbytes(int(arguments[3]))

        class DiesBeforeManifest(Client):
            """写到清单那一帧之前 SIGKILL 自己（segmented-rewrite-survives-writer-death 演写方死在写完段、写清单之前）。"""

            def write_batches(self, entries):
                if any(stored.startswith(SEGMENT_MANIFEST_MAGIC) for _key, stored in entries):
                    os.kill(os.getpid(), signal.SIGKILL)
                super().write_batches(entries)

        DiesBeforeManifest().write_many([(arguments[1].encode(), value)])
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


def cell_binary_not_built_without_target_directory(_binary, _work):
    """没给服务二进制、环境里又没有任务服务给的编译目录时 resolve_binary 不起 cargo、交 (None, 为什么)，为什么点名 --server-binary；
    有编译目录时才走 build_server_binary。build_server_binary 在这一格里换成只记调用的替身，不真起 cargo。"""
    global build_server_binary
    calls = []
    original = build_server_binary
    build_server_binary = lambda: (calls.append("built"), ("/built/prune-server", ""))[1]
    saved = {name: os.environ.pop(name, None) for name in (BINARY_VARIABLE, CARGO_TARGET_DIRECTORY_VARIABLE)}
    try:
        without_directory = resolve_binary([])
        calls_without_directory = list(calls)
        os.environ[CARGO_TARGET_DIRECTORY_VARIABLE] = "/service/given/target"
        with_directory = resolve_binary([])
        calls_with_directory = list(calls)
    finally:
        build_server_binary = original
        os.environ.pop(CARGO_TARGET_DIRECTORY_VARIABLE, None)
        for name, value in saved.items():
            if value is not None:
                os.environ[name] = value
    passed = (calls_without_directory == [] and without_directory[0] is None and "--server-binary" in without_directory[1]
              and calls_with_directory == ["built"] and with_directory == ("/built/prune-server", ""))
    return passed, (f"没有编译目录：起了 cargo {len(calls_without_directory)} 次（要 0）、交 {without_directory!r}（要 None 加点名 --server-binary 的说明）；"
                    f"有编译目录：起了 cargo {len(calls_with_directory)} 次（要 1）、交 {with_directory!r}")


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
    hosts_exit_code = hosts_refusal(sys.stderr)
    if hosts_exit_code is not None:
        return hosts_exit_code
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
    hosts_exit_code = hosts_refusal(sys.stdout)
    if hosts_exit_code is not None:
        return hosts_exit_code
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
