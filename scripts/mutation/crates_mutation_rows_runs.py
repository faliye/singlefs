"""crates-mutation-rows.py 拆出的一份。
起 cargo 与收输出：边跑边收、边跑边报、卡池目标、临时 prune-server、记覆盖、写用时，连同这几样与起 cargo 那几样（在 judging）的自证格。
入口文件头写着这一份在的位置、弄坏开关表与用法；入口 crates-mutation-rows.py 从各份 import 全部名字再导出，别处照旧按入口取名字。
拆出的各份只向前 import（base → judging → support → static → units → baseline → targets → memory → runs → pool → plan → stage → commands → driver），没有环。
"""
import ast
import collections
import contextlib
import importlib.util
import json
import os
import re
import shutil
import signal
import subprocess
import sys
import tempfile
import threading
import time
from crates_mutation_rows_base import (BUILD_PROFILES_DIRECTORY_NAME, BYTES_PER_STAT_BLOCK, CARGO_CONFIGURATION_DIRECTORY, COVERAGE_FLAG,
    COVERAGE_TARGET_SUFFIX, GPU_TEST_THREADS_RELEASED_TARGETS, MODULE_PATH, MODULE_SOURCE_PATHS, PRUNE_SERVER_BINARY_VARIABLE,
    PRUNE_SERVER_BUILD_DIRECTORY_NAME, PRUNE_SERVER_PARENT_DEATH_WRAPPER, PRUNE_SERVER_TEMPORARY_SERVICE_SCRIPT, PRUNE_SERVER_WRAPPER_NAME,
    RECORD_HEADER, Row, STAGE_REPOSITORY, STOP_GRACE_SECONDS, STOP_SIGNALS, TABLE, TARGET_DIRECTORY_NAME, WORKER_STATE,
    _PRUNE_SERVER_TEMPORARY_SERVICE_MODULE, break_is, mutation_units, note, output_progress_seconds, read_table, write_atomically)
from crates_mutation_rows_judging import (how_text, output_tail, results_by_test, run_cargo_command, verdict_of_run)
from crates_mutation_rows_support import (build_selftest_repository, selftest_write)
from crates_mutation_rows_static import (manifest_directory_of, unit_keys_for_directory)
from crates_mutation_rows_units import (row_content_hash)
from crates_mutation_rows_memory import (process_kind, process_tree_anon_bytes_by_kind)


def measure_and_remove_temporary_directory(directory, context):
    """数下一条的 TMPDIR 里有几个文件、占盘与表观各多少字节，再整个删掉。回 (文件数, 占盘字节, 表观字节, 删完之后它还在不在)。"""
    file_count = allocated_bytes = apparent_bytes = 0
    for current_directory, _subdirectory_names, file_names in os.walk(directory):
        for file_name in file_names:
            try:
                file_status = os.lstat(os.path.join(current_directory, file_name))
            except OSError:
                continue   # 数的时候已经没了（还没退干净的进程在收尾），不算
            file_count += 1
            allocated_bytes += file_status.st_blocks * BYTES_PER_STAT_BLOCK
            apparent_bytes += file_status.st_size
    if not break_is(context["environment"], "keepmutationtmp"):
        shutil.rmtree(directory, ignore_errors=True)
    return file_count, allocated_bytes, apparent_bytes, os.path.lexists(directory)


def gpu_lifecycle_prefix(environment):
    """GPU 阶段现场行的行首（crate singlefs-verification 的 gpu_lifecycle_line 打的）。弄坏开关 lifecycle-prefix-renamed：换一个 crates 那侧不打的。"""
    return "GPU_LIFECYCLE unix_us=" if break_is(environment, "lifecycle-prefix-renamed") else "GPU_LIFECYCLE unix_microseconds="


def new_output_collection(environment):
    """一条命令收输出的账（文件头「边跑边收、边跑边报」）：两根管道各自留下的行与字节、滤掉的条数与字节、最后 32 行、读线程出的错，连同这一趟怎么认 GPU 阶段现场行。
    弄坏开关 lifecycle-lines-kept：一行都不认；lifecycle-prefix-loose：行里任何位置有这几个字就认；lifecycle-filter-any-stream：stdout 也滤。"""
    prefix = gpu_lifecycle_prefix(environment)
    if break_is(environment, "lifecycle-lines-kept"):
        lifecycle_line = lambda line: False
    elif break_is(environment, "lifecycle-prefix-loose"):
        lifecycle_line = lambda line: prefix in line
    else:
        lifecycle_line = lambda line: line.startswith(prefix)
    return dict(stdout=[], stderr=[], stdout_bytes=0, stderr_bytes=0, dropped_lines=0, dropped_bytes=0, tail=collections.deque(maxlen=32), tail_lock=threading.Lock(),
                failures=[], lifecycle_line=lifecycle_line, filter_every_stream=break_is(environment, "lifecycle-filter-any-stream"))


def collect_output_lines(stream, stream_name, collection):
    """读线程的活：按行读 stream 到头。GPU 阶段现场行（只认 stderr 的）只计数、进最后 32 行，别的行留着。这里抛的（读不了、内存不够）记下来交主线程，不吃掉。"""
    kept, lifecycle_line = collection[stream_name], collection["lifecycle_line"]
    filtered = stream_name == "stderr" or collection["filter_every_stream"]
    try:
        for line in stream:
            size = len(line.encode("utf-8", "replace"))
            collection[stream_name + "_bytes"] += size
            if filtered and lifecycle_line(line):
                collection["dropped_lines"] += 1
                collection["dropped_bytes"] += size
                with collection["tail_lock"]:
                    collection["tail"].append(line)
            else:
                kept.append(line)
    except BaseException as error:   # 读线程里没人接得住：记下来，主线程照原样抛
        collection["failures"].append(error)


def output_progress_line(command, seconds, collection):
    """进度那一行（文件头「边跑边收、边跑边报」的写法）。"""
    anon_bytes = ",".join(f"{kind}:{size}" for kind, size in process_tree_anon_bytes_by_kind(os.getpid()).items())
    return (f"MUTATION_WORKER_PROGRESS process={os.getpid()} elapsed_seconds={seconds:.0f} stdout_bytes={collection['stdout_bytes']} stderr_bytes={collection['stderr_bytes']} "
            f"lifecycle_lines_dropped={collection['dropped_lines']} lifecycle_bytes_dropped={collection['dropped_bytes']} anon_bytes={anon_bytes} command={' '.join(command)[:200]}")


def computing_line_seconds(environment):
    """在算行的门槛（文件头「边跑边收、边跑边报」的「在算」）：GATE_MUTATION_COMPUTING_LINE_SECONDS，没设 300 秒。写法由 run_stage 开头核过。"""
    text = environment.get("GATE_MUTATION_COMPUTING_LINE_SECONDS", "").strip()
    return int(text) if text else 300


def computing_cell_label(environment):
    """在算行里这一格是什么：起命令的 TMPDIR 名字照 judge_row、run_baseline_batch、judge_coverage 建它时的前缀认，
    line-<N>- 是 mutation-row:<N>（变异表第 N 行），baseline-<N>- 是 baseline-batch:<N>（批首行是第 N 行的那一批基线），coverage-<N>- 是 coverage-group:<N>；认不出写 -。
    前缀表写在函数里（不做模块级常量）；judge_row、run_baseline_batch 建目录的前缀不能改成从这里取，
    自证「在算行：judge_row、run_baseline_batch、judge_coverage 建临时目录的前缀…」那一格读这三个函数的源码，核它们建的目录名都认得出。
    弄坏开关 computing-cell-baseline-unrecognized：不认 baseline- 前缀；computing-cell-coverage-unrecognized：不认 coverage- 前缀；computing-cell-prefix-renamed：变异行的前缀认成 row-。"""
    stems = {"line": "mutation-row", "baseline": "baseline-batch", "coverage": "coverage-group"}
    if break_is(environment, "computing-cell-baseline-unrecognized"):
        del stems["baseline"]
    if break_is(environment, "computing-cell-coverage-unrecognized"):
        del stems["coverage"]
    if break_is(environment, "computing-cell-prefix-renamed"):
        stems = {("row" if stem == "line" else stem): cell for stem, cell in stems.items()}
    found = re.match("(" + "|".join(stems) + r")-([0-9]+)-", os.path.basename(os.path.normpath(environment.get("TMPDIR", "") or "-")))
    if found is None:
        return "-"
    return stems[found.group(1)] + ":" + found.group(2)


def computing_tests_of(command):
    """在算行里点名的测试：(测试目标，逗号分隔；过滤词，逗号分隔)。从命令里头一个 test 词往后取：`--` 之前的 --test <名>、--test=<名> 写 test:<名>、--lib 写 lib；
    `--` 之后不以 - 起头的词是过滤词（--test-threads、--skip、--format、--color、--logfile、--shuffle-seed 分开写时后面那个值不算）。哪一样都没有写 -。"""
    words = list(command)
    if "test" not in words:
        return "-", "-"
    words = words[words.index("test") + 1:]
    separator = words.index("--") if "--" in words else len(words)
    before, after = words[:separator], words[separator + 1:]
    targets = []
    for index, word in enumerate(before):
        if word == "--test" and index + 1 < len(before):
            targets.append("test:" + before[index + 1])
        elif word.startswith("--test="):
            targets.append("test:" + word[len("--test="):])
        elif word == "--lib":
            targets.append("lib")
    filters, value_follows = [], False
    for word in after:
        if value_follows:
            value_follows = False
        elif word in ("--test-threads", "--skip", "--format", "--color", "--logfile", "--shuffle-seed"):
            value_follows = True
        elif not word.startswith("-"):
            filters.append(word)
    return ",".join(targets)[:300] or "-", ",".join(filters)[:300] or "-"


def output_computing_line(command, environment, seconds, window_seconds, collection, window):
    """在算那一行（文件头「边跑边收、边跑边报」的「在算」）：不带心跳前缀，统一任务服务把它算作日志在涨。window 是上一个在算点的读数（输出字节、滤掉的条数与字节）。"""
    targets, filters = computing_tests_of(command)
    output_bytes = collection["stdout_bytes"] + collection["stderr_bytes"] - window["output_bytes"]
    return (f"MUTATION_WORKER_COMPUTING cell={computing_cell_label(environment)} targets={targets} tests={filters} elapsed_seconds={seconds:.0f} window_seconds={window_seconds:.0f} "
            f"lifecycle_lines_dropped_in_window={collection['dropped_lines'] - window['dropped_lines']} "
            f"lifecycle_bytes_dropped_in_window={collection['dropped_bytes'] - window['dropped_bytes']} output_bytes_in_window={output_bytes}")


def wait_for_output_readers(readers, command, collection, environment):
    """等读线程都读到头，照固定间隔打进度：间隔点是起跑之后第 1、2、… 个间隔，每跨过一个打一行，子进程有没有新输出都打（文件头「边跑边收、边跑边报」）。
    线程等待接得住信号：这一进程接住的停的信号在这里抛。
    每一行进度以 TASK_SERVER_HEARTBEAT 加一个空格起头（统一任务服务的心跳行，文件头「边跑边收、边跑边报」）。
    另照门槛 computing_line_seconds 打在算行（output_computing_line，不带心跳前缀）：门槛点是起跑之后第 1、2、… 个门槛，晚醒跨过几个点只打一行；
    从上一个门槛点到这一个子进程有新输出（滤掉的 GPU 阶段现场行也算）才打，一个字都没出的不打（真卡死只剩心跳行，服务照样判得出）。
    弄坏开关 progress-line-only-on-output：这一段子进程没有新输出就不打；progress-line-unwritten：一行都不打；heartbeat-prefix-dropped：不带心跳前缀；
    computing-line-unwritten：在算行一行都不打；computing-line-without-output：子进程没有新输出也打在算行。"""
    interval = output_progress_seconds(environment)
    started = time.monotonic()
    next_look, reported_bytes = started + interval, 0
    only_on_output, unwritten = break_is(environment, "progress-line-only-on-output"), break_is(environment, "progress-line-unwritten")
    no_prefix = break_is(environment, "heartbeat-prefix-dropped")
    computing_interval = computing_line_seconds(environment)
    next_computing = started + computing_interval
    window = dict(started=started, output_bytes=0, dropped_lines=0, dropped_bytes=0)   # 上一个门槛点的读数
    computing_unwritten, computing_without_output = break_is(environment, "computing-line-unwritten"), break_is(environment, "computing-line-without-output")
    while True:
        alive = [reader for reader in readers if reader.is_alive()]
        if not alive:
            return
        alive[0].join(timeout=max(0.05, min(next_look, next_computing) - time.monotonic()))
        now = time.monotonic()
        if now >= next_computing:   # 晚醒跨过了几个门槛点的只打一行，读数算从上一个门槛点到此刻
            while next_computing <= now:
                next_computing += computing_interval
            total_bytes = collection["stdout_bytes"] + collection["stderr_bytes"]
            if not computing_unwritten and (total_bytes > window["output_bytes"] or computing_without_output):
                note(output_computing_line(command, environment, now - started, now - window["started"], collection, window))
            window = dict(started=now, output_bytes=total_bytes, dropped_lines=collection["dropped_lines"], dropped_bytes=collection["dropped_bytes"])
        while now >= next_look:   # 晚醒跨过了几个间隔点的，每个点一行
            next_look += interval
            total_bytes = collection["stdout_bytes"] + collection["stderr_bytes"]
            if unwritten or (only_on_output and total_bytes == reported_bytes):
                continue
            reported_bytes = total_bytes
            # 心跳前缀只在这里写、不做模块级常量（文件头「边跑边收、边跑边报」）：统一任务服务认这个前缀的行不算日志在涨
            note(("" if no_prefix else "TASK_SERVER_HEARTBEAT ") + output_progress_line(command, now - started, collection))


def report_lifecycle_tail(collection, environment):
    """被停时把留下的 GPU 阶段现场行打进任务日志。弄坏开关 lifecycle-tail-dropped-on-stop：不打。"""
    with collection["tail_lock"]:
        lines = list(collection["tail"])
    if lines and not break_is(environment, "lifecycle-tail-dropped-on-stop"):
        note(f"  ! 这一格被停：滤掉的 GPU 阶段现场行共 {collection['dropped_lines']} 行、{collection['dropped_bytes']} 字节，最后 {len(lines)} 行：\n" + "".join(lines).rstrip("\n"))


def print_lifecycle_tail_on_default_stop(collection, environment):
    """停的信号是默认处理（工作进程，init_worker）时，这一格跑着的这一段时间里收到信号先打留下的行、再照默认处理停：退出码照旧是被信号停。交要还原的 {信号: 原来的处理}。
    只有主线程设得了信号处理；不是主线程、或这一进程接住了信号（父进程在 run_pool 里）的不设，那边走 run_capturing_output 的 except。"""
    if threading.current_thread() is not threading.main_thread():
        return {}

    def lifecycle_tail_then_default_stop(signal_number, _frame):
        signal.signal(signal_number, signal.SIG_DFL)
        try:
            with contextlib.suppress(RuntimeError, OSError):   # 主线程正在打进度行时不可重入；打不出来也要照默认处理停
                report_lifecycle_tail(collection, environment)
        finally:
            os.kill(os.getpid(), signal_number)
    previous = {}
    for signal_number in STOP_SIGNALS:
        if signal.getsignal(signal_number) == signal.SIG_DFL:
            previous[signal_number] = signal.signal(signal_number, lifecycle_tail_then_default_stop)
    return previous


def collected_stderr_text(collection):
    """交回的 stderr：留下的行，滤掉过行的末尾另起一行汇总（文件头「边跑边收、边跑边报」）。"""
    text = "".join(collection["stderr"])
    if not collection["dropped_lines"]:
        return text
    separator = "" if not text or text.endswith("\n") else "\n"
    return f"{text}{separator}GPU_LIFECYCLE_DROPPED lines={collection['dropped_lines']} bytes={collection['dropped_bytes']}\n"


def run_capturing_output(command, **popen_keywords):
    """跑一条命令、边跑边收它的 stdout 与 stderr，交回 subprocess.CompletedProcess（文件头「边跑边收、边跑边报」：除 GPU 阶段现场行与它的汇总行之外，
    与 capture_output=True, text=True, errors="replace" 收的逐字相同）。
    命令起在它自己的进程组里；这一进程里被停（run_pool 接住的 TERM、HUP，先编一份依赖那几趟在父进程里跑）时对整组发 TERM，过 STOP_GRACE_SECONDS 还在再发 KILL，
    等它退了打留下的行、再往外抛。工作进程里信号是默认处理：打完留下的行照默认处理停（print_lifecycle_tail_on_default_stop）。
    不限时：卡住的由统一任务服务按日志与 CPU 都不涨停整条任务（task-server/）。"""
    environment = popen_keywords.get("env") or os.environ
    process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, errors="replace", start_new_session=True, **popen_keywords)
    collection = new_output_collection(environment)
    readers = [threading.Thread(target=collect_output_lines, args=(stream, stream_name, collection), daemon=True)
               for stream, stream_name in ((process.stdout, "stdout"), (process.stderr, "stderr"))]
    previous_handlers = print_lifecycle_tail_on_default_stop(collection, environment)
    try:
        for reader in readers:
            reader.start()
        wait_for_output_readers(readers, command, collection, environment)
        if collection["failures"]:
            raise collection["failures"][0]
        process.wait()
    except BaseException:
        with contextlib.suppress(ProcessLookupError):
            os.killpg(process.pid, signal.SIGTERM)
        try:
            process.wait(timeout=STOP_GRACE_SECONDS)
        except subprocess.TimeoutExpired:
            with contextlib.suppress(ProcessLookupError):
                os.killpg(process.pid, signal.SIGKILL)
            process.wait()
        for reader in readers:
            if reader.ident is not None:
                reader.join(timeout=5)
        report_lifecycle_tail(collection, environment)
        raise
    finally:
        for signal_number, handler in previous_handlers.items():
            signal.signal(signal_number, handler)
    process.stdout.close()
    process.stderr.close()
    return subprocess.CompletedProcess(command, process.returncode, "".join(collection["stdout"]), collected_stderr_text(collection))


def gpu_released_test_targets(root, environment):
    """放开 --test-threads 1 的测试目标（文件头「带 gpu 特性的格测试参数收尾带 --test-threads」）：本模块的 GPU_TEST_THREADS_RELEASED_TARGETS，
    加 root 下验证包登记的只经卡池取卡的测试目标（card_pool_only_test_targets）。with_gpu_test_threads 靠 init_worker 交的取名单函数照被判的根调它。
    登记认不出时打一行说明、这一次只放开 GPU_TEST_THREADS_RELEASED_TARGETS。弄坏开关 gpu-test-threads-all-one：一个都不放开（带 gpu 特性的格一律带 --test-threads 1）。"""
    if break_is(environment, "gpu-test-threads-all-one"):
        return ()
    pool_only, problem = card_pool_only_test_targets(root)
    if problem is not None:
        note(f"  ! {problem}：只经卡池取卡的测试目标这一次照旧带 --test-threads 1，只放开 {'、'.join(GPU_TEST_THREADS_RELEASED_TARGETS)}")
    return tuple(GPU_TEST_THREADS_RELEASED_TARGETS) + tuple(name for name in pool_only if name not in GPU_TEST_THREADS_RELEASED_TARGETS)


def card_pool_only_test_targets(root):
    """验证包登记的只经卡池取卡的测试目标（文件头「带 gpu 特性的格测试参数收尾带 --test-threads」）：root 下
    crates/singlefs-verification/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs 里 FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL
    那张表每一项的「"tests/<名>.rs"」取 <名>（集成测试目标名就是文件名去掉 .rs：验证包的 Cargo.toml 不另写 [[test]]）。
    交 (目标名的元组, 认不出时的那一句或 None)：文件不在交 ((), None)（这棵树没有验证包，点名不到它的目标）；读不了、认不出那张表或表里一项都没有交 ((), 那一句)。
    路径与表名写在函数里，不做模块级常量。"""
    path = os.path.join(root, "crates", "singlefs-verification", "tests", "gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs")
    try:
        with open(path, encoding="utf-8") as handle:
            text = handle.read()
    except FileNotFoundError:
        return (), None
    except (OSError, UnicodeDecodeError) as error:
        return (), f"{path} 读不了（{type(error).__name__}：{error}）"
    table = re.search(r"^const FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL\b[^=]*=\s*\[(.*?)^\];", text, re.M | re.S)
    names = tuple(re.findall(r'"tests/([A-Za-z0-9_]+)\.rs"', table.group(1))) if table else ()
    if not names:
        return (), f"{path} 里认不出 FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL 那张表（没有这个名字，或表里一项「\"tests/<名>.rs\"」都没有）"
    return names, None


def test_runs_multi_host_configuration(environment):
    """真跑测试的格交给测试的多机配置（文件头「真跑测试的格把多机配置交给测试、带 gpu 特性的格跑完核真用了卡」）：(绝对路径, 文件在不在)。找法在 scripts/hosts/task_memory.py 的
    test_runs_multi_host_configuration：调用方设了 SINGLEFS_MULTI_HOST_CONFIG 照它；没设取 GATE_MUTATION_MULTI_HOST_CONFIG（自证用），再没设照 admission.py 的 multi_host_configuration_file
    取本模块所在仓的那一份（git common-dir 的上一层下的 multi-host.env）。相对路径按本进程的当前目录取成绝对路径：测试在源码副本里起，相对路径到那里就指错了。"""
    import task_memory
    return task_memory.test_runs_multi_host_configuration(environment)


def prune_server_temporary_service_module():
    """scripts/prune-server/prune_server_client_selftest.py（临时服务的起法：TestServer 与放它的临时目录 selftest_root），第一次起临时服务时才读进来。"""
    if not _PRUNE_SERVER_TEMPORARY_SERVICE_MODULE:
        specification = importlib.util.spec_from_file_location("prune_server_temporary_service_for_crates_mutation_rows", PRUNE_SERVER_TEMPORARY_SERVICE_SCRIPT)
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        _PRUNE_SERVER_TEMPORARY_SERVICE_MODULE.append(module)
    return _PRUNE_SERVER_TEMPORARY_SERVICE_MODULE[0]


@contextlib.contextmanager
def mutation_run_prune_server(binary, command, temporary_directory, environment, on_real_service=False):
    """起 cargo test 那一次自己的临时 prune-server（文件头「变异复跑的崩溃判定」）：交要加进那一次环境的三个变量（连哪一处与临时服务的标记，都指它），
    收尾停服务、删它的目录。服务的目录放这一次的 temporary_directory 里（那一格收尾的删与留下核也管它）；socket 全路径放不下 107 字节时照客户端自证的
    selftest_root 另取（TMPDIR 太长退到 /tmp）。只编不跑的（--no-run）不起、交空；on_real_service（基线那一趟，文件头「基线连调用方的判定库服务」）同样不起、交空：
    环境里连哪一处照调用方的，判过的块按键从那一份读、判了写进去。
    弄坏开关 mutation-run-on-real-service：不起、交空（照调用方的环境连那一份服务）；baseline-on-temporary-service：基线那一趟也起。
    服务起不来抛 RuntimeError，取不到这一台的地址抛 HostAddressUnavailable（工作进程里抛的经 pool_cell_job 换成 WorkerJobFailed 交回，run_pool_reporting_dead_worker 报红，不吃掉）。"""
    if ("--no-run" in command or break_is(environment, "mutation-run-on-real-service")
            or (on_real_service and not break_is(environment, "baseline-on-temporary-service"))):
        yield {}
        return
    module = prune_server_temporary_service_module()
    directory = tempfile.mkdtemp(prefix="prune-server-", dir=temporary_directory)
    for fallback in (module.selftest_root, lambda: tempfile.mkdtemp(prefix="prune-server-", dir="/tmp")):
        if len(os.path.join(directory, "service", "server.sock").encode()) <= module.MAXIMUM_SOCKET_PATH_BYTES:
            break
        os.rmdir(directory)
        directory = fallback()
    server = None
    try:
        server = module.TestServer(binary, os.path.join(directory, "service"))
        yield {module.SOCKET_VARIABLE: server.socket, module.TCP_ADDRESS_VARIABLE: server.tcp_address, module.TEMPORARY_SERVICE_VARIABLE: server.socket}
    finally:
        if server is not None:
            server.stop()
        shutil.rmtree(directory, ignore_errors=True)


def mutation_run_prune_server_binary(environment, shard_root):
    """变异复跑起临时 prune-server 用的可执行文件（文件头「变异复跑的崩溃判定」）：(路径, "") 或 (None, 为什么)。真二进制取 PRUNE_SERVER_BINARY（绝对路径、可执行）；
    没设照本模块所在仓的 prune-server/ 编一份进 <shard_root>/prune-server-build（这一趟的编译目录，收尾随 shard_root 删）。
    交出去的是 <shard_root> 里写的一份包装（PRUNE_SERVER_PARENT_DEATH_WRAPPER）：服务各在自己的会话里，起它的工作进程被停或被硬杀时 finally 不跑，
    包装先设父进程死亡信号再换成真二进制，工作进程一死服务就跟着停。"""
    configured = environment.get(PRUNE_SERVER_BINARY_VARIABLE, "").strip()
    if configured:
        if not (os.path.isabs(configured) and os.path.isfile(configured) and os.access(configured, os.X_OK)):
            return None, f"{PRUNE_SERVER_BINARY_VARIABLE}={configured} 不是可执行文件的绝对路径"
        binary = configured
    else:
        target = os.path.join(shard_root, PRUNE_SERVER_BUILD_DIRECTORY_NAME)
        manifest = os.path.join(STAGE_REPOSITORY, "prune-server", "Cargo.toml")
        completed = subprocess.run(["cargo", "build", "--release", "--offline", "--manifest-path", manifest, "--bin", "prune-server"],
                                   env=dict(environment, CARGO_TARGET_DIR=target), capture_output=True, text=True, check=False)
        binary = os.path.join(target, "release", "prune-server")
        if completed.returncode != 0 or not os.access(binary, os.X_OK):
            return None, f"cargo build --release --offline --manifest-path {manifest} --bin prune-server 退 {completed.returncode}：{completed.stderr.strip()[-400:]}"
    wrapper = os.path.join(shard_root, PRUNE_SERVER_WRAPPER_NAME)
    with open(wrapper, "x", encoding="utf-8") as handle:
        handle.write(PRUNE_SERVER_PARENT_DEATH_WRAPPER.replace("REAL_BINARY_PATH", repr(binary)))
    os.chmod(wrapper, 0o755)
    return wrapper, ""


def write_row_timing(row, seconds, context):
    """变异一行判完写用时表（运行记录，文件头「判定库」末段）：timings/<六段原样的 sha256>，只排派活次序。交写不进去的那几句（写成交空表）。
    经 init_worker 交进 WORKER_STATE["write_timing"] 的引用调。"""
    timing_path = os.path.join(context["timings_directory"], row_content_hash(row))
    try:
        write_atomically(timing_path, mutation_units.record_text(RECORD_HEADER, dict(
            row_content_hash=row_content_hash(row), seconds_running=seconds["seconds_running"], judged_date=mutation_units.today_text())))
    except OSError as error:
        return [f"{timing_path}：{error}"]
    return []


# ── 覆盖（judge_coverage 与它用的几个函数） ────────────────────────────────────

def coverage_build_settings(environment):
    """插桩编译的开关：(只加在那几次的环境, 接在 cargo 后面的参数)。环境里有 CARGO_ENCODED_RUSTFLAGS / RUSTFLAGS 就接在后面
    （它们顶掉配置文件里的 build.rustflags，接在后面才不丢），没有就经 --config build.rustflags（与配置文件里的合并）。"""
    encoded = environment.get("CARGO_ENCODED_RUSTFLAGS", "")
    if encoded:
        return {"CARGO_ENCODED_RUSTFLAGS": encoded + "\x1f" + COVERAGE_FLAG}, []
    flags = environment.get("RUSTFLAGS", "").strip()
    if flags:
        return {"RUSTFLAGS": flags + " " + COVERAGE_FLAG}, []
    return {}, ["--config", f'build.rustflags=["{COVERAGE_FLAG}"]']


def relative_inside(path, work):
    """cargo 消息里的路径 → 相对源码副本根的路径；落在副本之外（注册表、OUT_DIR）交 None。"""
    full = os.path.normpath(path if os.path.isabs(path) else os.path.join(work, path))
    root = os.path.normpath(work)
    return os.path.relpath(full, root) if full.startswith(root + os.sep) else None


def executables_of_build(messages, work):
    """--no-run 那一次的 compiler-artifact 消息 → 本仓编出来的可执行文件（测试与 bin，构建脚本不算）。"""
    executables = []
    for message in messages:
        if message.get("reason") != "compiler-artifact" or relative_inside(os.path.dirname(message.get("manifest_path", "")), work) is None:
            continue
        executable = message.get("executable")
        if executable and "custom-build" not in message.get("target", {}).get("kind", []) and executable not in executables:
            executables.append(executable)
    return executables


def judge_coverage(job):
    """进程池里的一格活：一条点名测试在没改坏的副本上记一份覆盖（文件头「判定库」覆盖记录 ①–④）。
    回 ({coverage（覆盖号或 None）, problem（没记成的那一句或 None）, line}, 写不进去的记录)。"""
    state, context = WORKER_STATE, WORKER_STATE["context"]
    result = dict(coverage=None, problem=None, line=job["line"], identity=job["coverage_id"])
    if break_is(context["environment"], "coveragenotrecorded"):
        return result, []
    if state["library"].read_json("coverage", job["coverage_id"]) is not None and not job.get("stale"):
        result.update(coverage=job["coverage_id"], existing=True)
        return result, []
    overrides, configuration = coverage_build_settings(state["environment"])
    overrides["CARGO_TARGET_DIR"] = state["environment"]["CARGO_TARGET_DIR"] + COVERAGE_TARGET_SUFFIX
    temporary_directory = tempfile.mkdtemp(prefix=f"coverage-{job['line']}-", dir=state["per_mutation_temporary_root"])
    profiles = os.path.join(temporary_directory, "profiles")
    os.mkdir(profiles)
    try:
        result["problem"] = record_coverage(job, overrides, configuration, temporary_directory, profiles)
        if result["problem"] is None:
            result["coverage"] = job["coverage_id"]
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        result["problem"] = f"记覆盖时出错（{type(error).__name__}：{error}）"
    finally:
        measure_and_remove_temporary_directory(temporary_directory, context)
    return result, []


def record_coverage(job, overrides, configuration, temporary_directory, profiles):
    """judge_coverage 的 ①–④：记成交 None，哪一步不成交那一句。"""
    state, context = WORKER_STATE, WORKER_STATE["context"]
    # 带 -Cinstrument-coverage 的编译：cargo 没带 --target 时 RUSTFLAGS 也罩构建脚本与过程宏，它们在各自的源码目录（~/.cargo/registry/src/ 下依赖包的目录）里
    # 起，没设 LLVM_PROFILE_FILE 就在那里写 default_<哈希>_<进程号>.profraw；编译这一步同样把剖析文件指进这一格的 TMPDIR（不并进 profiles：那里只放测试那一趟的）
    build_profiles = os.path.join(temporary_directory, BUILD_PROFILES_DIRECTORY_NAME)
    os.mkdir(build_profiles)
    build_overrides = overrides if break_is(context["environment"], "buildprofilesunset") else dict(
        overrides, LLVM_PROFILE_FILE=os.path.join(build_profiles, "build-%p-%m.profraw"))
    label = f"覆盖（组首行是变异表第 {job['line']} 行）"
    build, _seconds = run_cargo_command(["cargo", *configuration, "test", "--offline", *job["before"], "--no-run", "--message-format=json-render-diagnostics"],
                                        temporary_directory, label, build_overrides)
    if build.returncode != 0:
        return f"插桩编译没成（退出码 {build.returncode}）：{output_tail(build.stderr, 3)}"
    messages = []
    for line in build.stdout.splitlines():
        if line.startswith("{"):
            with contextlib.suppress(ValueError):
                messages.append(json.loads(line))
    executables = executables_of_build(messages, state["work"])
    if not executables:
        return "插桩编译没交出一个本仓的可执行文件"
    # 点名用例前自己补 --exact：行的 libtest 参数里已经带了的不再重复补（libtest 报 Option 'exact' given more than once、退 101，覆盖就记不成）；
    # 弄坏开关 exactduplicated：原样接在后面
    switches = list(job["switches"]) if break_is(context["environment"], "exactduplicated") else [word for word in job["switches"] if word != "--exact"]
    run, _seconds = run_cargo_command(["cargo", *configuration, "test", "--offline", *job["before"], "--", *switches, "--exact", job["test"]],
                                      temporary_directory, label, dict(overrides, LLVM_PROFILE_FILE=os.path.join(profiles, "cov-%p-%m.profraw")))
    output = run.stdout + run.stderr
    results = results_by_test(job["test"], output)
    if run.returncode != 0 or len(results) != 1 or any(result != "ok" for result in next(iter(results.values()))):
        return (f"插桩跑 {job['test']} 没恰好一个 ok（退出码 {run.returncode}；"
                f"对上 {len(results)} 个用例）：{output_tail(output, 3)}")
    raw_profiles = sorted(os.path.join(profiles, name) for name in os.listdir(profiles) if name.endswith(".profraw"))
    if not raw_profiles:
        return "插桩跑完一份 .profraw 都没留下（LLVM_PROFILE_FILE 没传到测试进程）"
    merged = os.path.join(temporary_directory, "merged.profdata")
    tools = context["llvm_tools"]
    merging = subprocess.run([os.path.join(tools, "llvm-profdata"), "merge", "-sparse", *raw_profiles, "-o", merged], capture_output=True, text=True, errors="replace")
    if merging.returncode != 0:
        return f"llvm-profdata merge 退 {merging.returncode}：{output_tail(merging.stderr, 3)}"
    objects = [executables[0]] + [word for executable in executables[1:] for word in ("--object", executable)]
    exporting = subprocess.run([os.path.join(tools, "llvm-cov"), "export", "-format=text", "-skip-expansions", "-instr-profile", merged, *objects],
                               capture_output=True, text=True, errors="replace")
    if exporting.returncode != 0:
        return f"llvm-cov export 退 {exporting.returncode}：{output_tail(exporting.stderr, 3)}"
    export = json.loads(exporting.stdout)
    # ④ 执行过的单元：计数 > 0 的函数落进的 fn 单元，连同点名测试闭包里不是 fn 的单元（常量、类型、静态项、宏、数据文件），都在这份副本上切；
    #   单元键按这一行的包目录取（--manifest-path 指的包用它自己那一份，文件头「--manifest-path」），同一个包目录在这个工作进程里只建一份
    directory, why = manifest_directory_of(state["work"], job["arguments"], context["environment"])
    if why is not None:
        return f"副本里认不出 cargo 参数的 --manifest-path（{why}）"
    if directory not in state["unit_keys"]:
        state["unit_keys"][directory] = unit_keys_for_directory(state["work"], directory)
    keys = state["unit_keys"][directory]
    # 覆盖记的是副本上执行过的单元，没有被改坏的那一行：一律取产品行为视图（这里的 UnitKeys 没问过 mutated_units）
    test, refusal = keys.resolve_test(job["arguments"], job["expected"], view=mutation_units.function_units().PRODUCT_BEHAVIOR)
    if refusal is not None:
        return f"副本里认不出点名的测试（{refusal}）"
    closure = keys.closure(test.unit, test.target, job["arguments"])
    executed, unplaced = keys.executed_units(export, state["work"])
    members = {(reference.kind, reference.file, reference.name, reference.fingerprint): reference
               for reference in executed + [member for member in closure.members if member.kind != "fn"]}
    state["library"].write_json("coverage", job["coverage_id"], dict(
        test=list(mutation_units.reference_of(test.unit)), executed=sorted(list(reference) for reference in members.values()),
        executed_functions=len(executed), unplaced_functions=unplaced, complete=unplaced == 0 and bool(executed),
        executables=len(executables), judged_date=mutation_units.today_text(), how=how_text(context, "覆盖")))
    return None


def selftest_gpu_test_threads_pool_targets_cell(selftest, work):
    """只经卡池取卡的测试目标放开 --test-threads 1（gpu_released_test_targets、card_pool_only_test_targets，文件头「带 gpu 特性的格测试参数收尾带 --test-threads」）：
    样本仓里验证包那份名单把 pool_only_sample 登记在 FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL、still_bypassing_sample 只登记在还没挪上卡池的那一张，两行都带 gpu 特性：
    跑一趟（SINGLEFS_THREAD_CAP=7 当登记值），点名 pool_only_sample 的行基线、变异、插桩跑真跑测试的每一次都收尾带 `--test-threads 7`，点名 still_bypassing_sample 的每一次带 `--test-threads 1`；
    名单读法另判：样本名单读出的恰好是 pool_only_sample；去掉那张表的名单报一句、一个都不读出；这棵树（本模块所在的仓）的名单读得出、每一项的 tests/<名>.rs 都在（这棵树没带 crates 记本次未跑）。
    弄坏开关 gpu-test-threads-all-one（一律带 1）下头一格判错。"""
    root = os.path.join(work, "gpu-test-threads-pool-targets")
    build_selftest_repository(root, SELFTEST_GPU_POOL_TARGETS_ROWS, extra_source=SELFTEST_GPU_POOL_TARGETS_SOURCE)
    selftest_write(os.path.join(root, "crates", "tiny", "tests", "pool_only_sample.rs"), "mod tests {\n    #[test]\n    fn pool_agrees() { assert_eq!(tiny::pooled(), 10); }\n}\n")
    selftest_write(os.path.join(root, "crates", "tiny", "tests", "still_bypassing_sample.rs"),
                   "mod tests {\n    #[test]\n    fn bypass_agrees() { assert_eq!(tiny::bypassing(), 14); }\n}\n")
    selftest_write(os.path.join(root, CARGO_CONFIGURATION_DIRECTORY, "config.toml"), SELFTEST_CARGO_CONFIGURATION)
    list_path = os.path.join(root, "crates", "singlefs-verification", "tests", "gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs")
    selftest_write(list_path, SELFTEST_GPU_POOL_LIST)
    exit_code, stdout, stderr, invocations = selftest.run(root, SINGLEFS_THREAD_CAP="7")

    def runs_of(test):
        """起了点名这条测试、真跑测试的每一次（--no-run 的先编不算）的 libtest 参数。"""
        tails = [line.split("\t")[0].split() for line in invocations if "--no-run" not in line.split("\t")[0].split()]
        tails = [words[words.index("--") + 1:] if "--" in words else [] for words in tails]
        return [tail for tail in tails if any(word in (test, "tests::" + test) for word in tail)]
    pool_runs, bypass_runs = runs_of("pool_agrees"), runs_of("bypass_agrees")
    selftest.check("只经卡池取卡的测试目标放开：名单 FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL 里的目标基线、变异、插桩跑每一次都收尾带登记值，"
                   "只登记在还没挪上卡池那一张里的目标每一次带 --test-threads 1",
                   exit_code == 0 and len(pool_runs) >= 3 and all(tail[-2:] == ["--test-threads", "7"] for tail in pool_runs)
                   and len(bypass_runs) >= 3 and all(tail[-2:] == ["--test-threads", "1"] for tail in bypass_runs),
                   f"退 {exit_code}；pool_only_sample {pool_runs}；still_bypassing_sample {bypass_runs}；{stdout[-300:]}{stderr[-300:]}")
    sample_names, sample_problem = card_pool_only_test_targets(root)
    selftest_write(list_path, SELFTEST_GPU_POOL_LIST_WITHOUT_TABLE)
    changed_names, changed_problem = card_pool_only_test_targets(root)
    absent_names, absent_problem = card_pool_only_test_targets(os.path.join(work, "gpu-test-threads-pool-targets-absent"))
    selftest.check("只经卡池取卡的名单读法：样本名单读出恰好 pool_only_sample；去掉那张表的报一句、一个都不读出；名单文件不在的不读出、不报",
                   sample_names == ("pool_only_sample",) and sample_problem is None and changed_names == () and changed_problem is not None
                   and "FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL" in changed_problem and absent_names == () and absent_problem is None,
                   f"样本 {sample_names}、{sample_problem}；去掉表 {changed_names}、{changed_problem}；不在 {absent_names}、{absent_problem}")
    label = "只经卡池取卡的名单读法：这棵树验证包那份名单读得出，每一项的 tests/<名>.rs 都在"
    if not os.path.isdir(os.path.join(STAGE_REPOSITORY, "crates", "singlefs-verification", "tests")):
        selftest.skip(label, f"{STAGE_REPOSITORY} 没带 crates/singlefs-verification/tests")
    else:
        names, problem = card_pool_only_test_targets(STAGE_REPOSITORY)
        missing = [name for name in names if not os.path.isfile(os.path.join(STAGE_REPOSITORY, "crates", "singlefs-verification", "tests", name + ".rs"))]
        selftest.check(label, bool(names) and problem is None and not missing, f"读出 {names}；{problem}；文件不在的 {missing}")


def selftest_gpu_cell_dispatch_cells(selftest, work):
    """上卡的格一格接一格、不领卡（文件头「上卡的格一格接一格」）。PATH 前面套一层 cargo，每一次 cargo test 记起止时刻、这一格的名字（TMPDIR 的名字带行号）、参数、
    收到的 GPU_CARDS_BY_PRIORITY 与藏卡的那几个变量（FAKE_CARGO_SPAN_LOG 只借它让假 cargo 每次睡一会儿、把先后拉开）；环境里给 GPU_CARDS_BY_PRIORITY=selftest-card-0
    与 VK_ADD_DRIVER_FILES。两趟：
    混合那一趟：变异表两行不上卡（adds、halves）、两行上卡（--features gpu 的 doubles、negates），2 个工作进程；
    全上卡那一趟：三行都上卡（doubles、negates、halves），3 个工作进程。
    上卡的格一格接一格：全上卡那一趟三格变异起的测试彼此不重叠、三行都抓到；两趟里上卡的格（基线的批、变异、记覆盖：记覆盖的插桩跑起的 cargo 在 test 前带 --config，
      PATH 上那层 cargo 跳过 --config 的两个词再认 test）起的每条 cargo test 收到的卡串就是环境里给的那一串，基线的批与变异都没带 --no-run（不另编）；
      记覆盖那一格只有插桩编译那一次带 --no-run（judge_coverage 本来就先 --no-run 编、再按编出的可执行文件跑，不上卡的记覆盖同样两次，不是另编）；混合那一趟四行都抓到。
      判错时失败说明先列不对的是哪几样。
    不上卡的格看不到卡（混合那一趟）：不上卡的格起的每条 cargo 都带藏卡的变量（驱动清单指到这一格自己 TMPDIR 底下不存在的文件、CUDA_VISIBLE_DEVICES=-1、
      没有 VK_ADD_DRIVER_FILES），上卡的格一个都不带。
    弄坏开关 gpu-cells-dispatched-in-parallel 下「上卡的格一格接一格」判错（三格变异照空位同时派，测试重叠；推的：靠假 cargo 睡的那一会儿把先后拉开），
    plain-cells-cards-visible 下「不上卡的格看不到卡」判错。"""
    gpu_negate_row = "gpu-negate-becomes-identity\tcrates/tiny/src/lib.rs\t{ 0 - value }\t{ value }\t-p tiny --features gpu --lib -- negates\tnegates"
    gpu_halve_row = "gpu-halve-becomes-third\tcrates/tiny/src/lib.rs\t{ value / 2 }\t{ value / 3 }\t-p tiny --features gpu --lib -- halves\thalves"
    wrapper_directory = os.path.join(work, "gpu-cell-dispatch-bin")
    real_cargo = os.path.join(selftest.bin_directory, "cargo")
    hiding_names = ("VK_DRIVER_FILES", "VK_ICD_FILENAMES", "CUDA_VISIBLE_DEVICES", "VK_ADD_DRIVER_FILES")
    extra_driver_list = "/gpu-cell-dispatch-selftest-extra-driver-list.json"
    given_cards = "selftest-card-0"
    selftest_write(os.path.join(wrapper_directory, "cargo"),
                   "#!/usr/bin/env python3\nimport json, os, subprocess, sys, time\n"
                   "started = time.time()\n"
                   f"code = subprocess.run([{real_cargo!r}] + sys.argv[1:]).returncode\n"
                   "words = sys.argv[1:]\n"
                   "while words[:1] == ['--config']:\n"
                   "    words = words[2:]\n"
                   "if os.environ.get('GPU_CELL_DISPATCH_CARGO_LOG') and words[:1] == ['test']:\n"
                   "    record = {'start': started, 'end': time.time(), 'cell': os.path.basename(os.environ.get('TMPDIR', '')), 'arguments': ' '.join(sys.argv[1:]),\n"
                   "              'cards': os.environ.get('GPU_CARDS_BY_PRIORITY'),\n"
                   f"              'hiding': {{name: os.environ.get(name) for name in {hiding_names!r}}}}}\n"
                   "    with open(os.environ['GPU_CELL_DISPATCH_CARGO_LOG'], 'a', encoding='utf-8') as log:\n"
                   "        log.write(json.dumps(record) + '\\n')\n"
                   "sys.exit(code)\n", executable=True)

    def run_mode(mode, mode_rows, workers):
        repository = os.path.join(work, f"gpu-cell-dispatch-{mode}")
        build_selftest_repository(repository, mode_rows)
        selftest_write(os.path.join(repository, CARGO_CONFIGURATION_DIRECTORY, "config.toml"), SELFTEST_CARGO_CONFIGURATION)
        cargo_log, span_log = (os.path.join(work, f"gpu-cell-dispatch-{mode}-{name}.log") for name in ("cargo", "spans"))
        result = selftest.run(repository, GATE_MUTATION_WORKERS=workers, PATH=wrapper_directory + os.pathsep + selftest.base_environment["PATH"],
                              FAKE_CARGO_SPAN_LOG=span_log, GPU_CELL_DISPATCH_CARGO_LOG=cargo_log, GPU_CARDS_BY_PRIORITY=given_cards,
                              VK_ADD_DRIVER_FILES=extra_driver_list)
        table_rows = read_table(os.path.join(repository, TABLE), selftest.base_environment)[0]
        gpu_lines = {row.line_number for row in table_rows if "--features gpu" in row.arguments}
        cargo_records = []
        if os.path.exists(cargo_log):
            with open(cargo_log, encoding="utf-8") as handle:
                for line in handle.read().splitlines():
                    record = json.loads(line)
                    number = re.match(r"(?:baseline|line|coverage)-(\d+)-", record["cell"])
                    if number:
                        record["card_cell"] = int(number.group(1)) in gpu_lines
                        cargo_records.append(record)
        return result, sorted(cargo_records, key=lambda record: record["start"])

    def hidden_properly(record):
        """这一条 cargo 的环境藏了卡：两个驱动清单变量指同一个文件，在这一格自己的 TMPDIR（名字就是格名）底下、叫 no-vulkan-driver.json、不存在；CUDA_VISIBLE_DEVICES=-1；追加清单去掉了。"""
        hiding = record["hiding"]
        path = hiding["VK_DRIVER_FILES"] or ""
        return (bool(path) and hiding["VK_ICD_FILENAMES"] == path and os.path.basename(path) == "no-vulkan-driver.json"
                and os.path.basename(os.path.dirname(path)) == record["cell"] and not os.path.exists(path)
                and hiding["CUDA_VISIBLE_DEVICES"] == "-1" and hiding["VK_ADD_DRIVER_FILES"] is None)

    def serial_within(runs):
        """几条一条接一条、不重叠。"""
        ordered = sorted(runs, key=lambda record: record["start"])
        return all(earlier["end"] <= later["start"] for earlier, later in zip(ordered, ordered[1:]))

    def coverage_compile(record):
        """记覆盖那一格的插桩编译（judge_coverage 的第一步：--no-run --message-format=json-render-diagnostics）。"""
        return record["cell"].startswith("coverage-") and "--message-format=json-render-diagnostics" in record["arguments"].split()

    def card_runs_untouched(card_records):
        """上卡的格起的每条 cargo test：收到的卡串就是环境里给的那一串；带 --no-run 的只许是记覆盖的插桩编译（基线的批与变异不另编）。"""
        return bool(card_records) and all(record["cards"] == given_cards and ("--no-run" not in record["arguments"].split() or coverage_compile(record))
                                          for record in card_records)

    mixed_rows = [SELFTEST_TABLE_ROWS[0], SELFTEST_TABLE_ROWS[3], SELFTEST_GPU_FEATURE_ROW, gpu_negate_row]
    (mixed_exit, mixed_stdout, mixed_stderr, _invocations), mixed_records = run_mode("mixed", mixed_rows, "2")
    mixed_card_records = [record for record in mixed_records if record["card_cell"]]
    plain_records = [record for record in mixed_records if not record["card_cell"]]
    three_rows = [SELFTEST_GPU_FEATURE_ROW, gpu_negate_row, gpu_halve_row]
    (three_exit, three_stdout, three_stderr, _invocations), three_records = run_mode("three", three_rows, "3")
    mutation_runs = [record for record in three_records if record["card_cell"] and record["cell"].startswith("line-")]
    serial_parts = (("全上卡那一趟退 0、抓到 3 条", three_exit == 0 and "计数：抓到 3 条" in three_stdout),
                    ("全上卡那一趟三格变异的测试恰好三条、彼此不重叠", len(mutation_runs) == 3 and serial_within(mutation_runs)),
                    ("全上卡那一趟上卡的格不另编、卡串原样", card_runs_untouched([record for record in three_records if record["card_cell"]])),
                    ("混合那一趟退 0、抓到 4 条", mixed_exit == 0 and "计数：抓到 4 条" in mixed_stdout),
                    ("混合那一趟上卡的格不另编、卡串原样", card_runs_untouched(mixed_card_records)))
    selftest.check("上卡的格一格接一格：全上卡那一趟三格变异起的测试彼此不重叠、三行都抓到；两趟里上卡的格起的每条 cargo test 收到的卡串就是环境里给的那一串，"
                   "基线的批与变异都没带 --no-run（记覆盖只有插桩编译那一次带）；混合那一趟四行都抓到",
                   all(ok for _what, ok in serial_parts),
                   f"不对的：{'、'.join(what for what, ok in serial_parts if not ok)}；全上卡那一趟退 {three_exit}，三格变异 {[(record['cell'], record['start'], record['end'], record['cards']) for record in mutation_runs]}；"
                   f"{three_stdout[-300:]}{three_stderr[-300:]}；混合那一趟退 {mixed_exit}，上卡的格 {[(record['cell'], record['arguments'], record['cards']) for record in mixed_card_records]}；"
                   f"{mixed_stdout[-300:]}{mixed_stderr[-300:]}")
    untouched = {"VK_DRIVER_FILES": None, "VK_ICD_FILENAMES": None, "CUDA_VISIBLE_DEVICES": None, "VK_ADD_DRIVER_FILES": extra_driver_list}
    selftest.check("不上卡的格看不到卡：不上卡的格起的每条 cargo 都带藏卡的变量（驱动清单指到这一格自己 TMPDIR 底下不存在的文件、CUDA_VISIBLE_DEVICES=-1、没有 VK_ADD_DRIVER_FILES），"
                   "上卡的格一个都不带",
                   mixed_exit == 0 and plain_records and mixed_card_records
                   and all(hidden_properly(record) for record in plain_records) and all(record["hiding"] == untouched for record in mixed_card_records),
                   f"退 {mixed_exit}；不上卡的格 {[(record['cell'], record['hiding']) for record in plain_records]}；上卡的格 {[(record['cell'], record['hiding']) for record in mixed_card_records]}")


def selftest_service_build_directory_cell(selftest, work):
    """服务给的编译目录（文件头「编译目录」）：不设 GATE_MUTATION_TARGET_DIR、环境里给 CARGO_TARGET_DIR，起的 cargo 收到的 CARGO_TARGET_DIR 都在
    <它>/singlefs-crates-mutation-target（第 1 片）下，跑完它还在（不删）。弄坏开关 service-build-directory-ignored 下判错。"""
    repository = os.path.join(work, "service-build-directory")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS[:1])
    service = os.path.join(work, "service-build")
    environment_log = os.path.join(work, "service-build-environment.jsonl")
    exit_code, stdout, stderr, _invocations = selftest.run(repository, GATE_MUTATION_TARGET_DIR="", CARGO_TARGET_DIR=service,
                                                           FAKE_CARGO_ENVIRONMENT_LOG=environment_log)
    targets = []
    if os.path.exists(environment_log):
        with open(environment_log, encoding="utf-8") as handle:
            targets = [json.loads(line).get("CARGO_TARGET_DIR", "") for line in handle if line.strip()]
    expected = os.path.join(os.path.realpath(service), TARGET_DIRECTORY_NAME)
    selftest.check("服务给的编译目录：起的 cargo 的编译目录都在服务给的那一个底下、跑完不删",
                   exit_code == 0 and targets and all(target == expected or target.startswith(expected + "-") for target in targets) and os.path.isdir(expected),
                   f"退 {exit_code}；起的 cargo 收到的 CARGO_TARGET_DIR {sorted(set(targets))}（应在 {expected} 下）；跑完它{'还在' if os.path.isdir(expected) else '不在了'}；"
                   f"{stdout[-300:]}{stderr[-400:]}")


def selftest_capture_child_program():
    """子进程的正文：按路径载入本模块（argv[1]），调 run_capturing_output 跑 argv[2]（JSON 的命令），标准输出交一份 JSON：那条命令的退出码、VmHWM 涨幅、stdout 与 stderr（太长只交尾巴）。
    argv[3] 是 own-handler 时先把 TERM 接成 raise_stop_requested（父进程在 run_pool 里的接法），别的值照默认处理（工作进程的接法）。"""
    return ("import importlib.util, json, os, signal, sys\n"
            "specification = importlib.util.spec_from_file_location('crates_mutation_rows_capture_child', sys.argv[1])\n"
            "module = importlib.util.module_from_spec(specification)\n"
            "sys.modules[specification.name] = module\n"
            "specification.loader.exec_module(module)\n"
            "if sys.argv[3] == 'own-handler':\n"
            "    signal.signal(signal.SIGTERM, module.raise_stop_requested)\n"
            "def peak_bytes():\n"
            "    with open('/proc/self/status', encoding='utf-8') as handle:\n"
            "        return next(int(line.split()[1]) * 1024 for line in handle if line.startswith('VmHWM:'))\n"
            "before = peak_bytes()\n"
            "run = module.run_capturing_output(json.loads(sys.argv[2]), env=dict(os.environ))\n"
            "limit = 1 << 20\n"
            "json.dump(dict(returncode=run.returncode, peak_growth=peak_bytes() - before, stdout=run.stdout[-limit:], stderr=run.stderr[-limit:]), sys.stdout)\n")


def selftest_capture_run(selftest, producer, **changes):
    """在子进程里经 run_capturing_output 跑一段 python（producer）：(子进程退出码, 它交的那份 JSON，交不出是 None, 它自己的 stderr：进度行与被停时打的行都在这里)。"""
    child = subprocess.run([sys.executable, "-c", selftest_capture_child_program(), MODULE_PATH, json.dumps([sys.executable, "-c", producer]), "default-handler"],
                           env=dict(selftest.base_environment, **changes), capture_output=True, text=True, errors="replace", timeout=600, check=False)
    try:
        return child.returncode, json.loads(child.stdout), child.stderr
    except ValueError:
        return child.returncode, None, child.stderr


def selftest_output_collection_cells(selftest, work):
    """边跑边收（文件头「边跑边收、边跑边报」）：内存有界、别的行逐字相同、stdout 不滤、只认行首、前缀与 crates 那侧打的同一个。"""
    filler = 'GPU_LIFECYCLE unix_microseconds=1793843585212043 elapsed_microseconds=12792699 pid=67478 thread=ThreadId(7) test=Some("tests::doubles") phase=queue_submit_end '
    lifecycle_line = filler + "d" * (349 - len(filler)) + "\n"   # 一行 350 字节，3000 行一块、200 块：210000000 字节（200 MiB）
    flood = (f"import sys\nline = {lifecycle_line!r}\nblock = (line * 3000).encode()\n"
             "for _block in range(200):\n    sys.stderr.buffer.write(block)\nsys.stderr.buffer.flush()\n"
             "sys.stderr.write('ordinary stderr line\\n')\nsys.stderr.flush()\nprint('test tests::doubles ... ok')\n")
    exit_code, flooded, stderr = selftest_capture_run(selftest, flood)
    selftest.check("边跑边收：假命令往 stderr 写 200 MiB 的 GPU 阶段现场行：工作进程 VmHWM 涨幅不超过 32 MiB，结果里一行不留、别的行逐字留着，末尾一行汇总条数与字节",
                   exit_code == 0 and flooded is not None and flooded["returncode"] == 0 and flooded["peak_growth"] <= 32 << 20
                   and flooded["stdout"] == "test tests::doubles ... ok\n"
                   and flooded["stderr"] == "ordinary stderr line\nGPU_LIFECYCLE_DROPPED lines=600000 bytes=210000000\n",
                   f"退 {exit_code}；VmHWM 涨幅 {flooded and flooded['peak_growth']} 字节；stdout 尾 {flooded and flooded['stdout'][-120:]!r}；stderr 尾 {flooded and flooded['stderr'][-200:]!r}；"
                   f"子进程 stderr 尾 {stderr[-300:]!r}")
    awkward = ("import sys\nsys.stdout.buffer.write(b'caf\\xc3\\xa9 line\\r\\nbare cr\\rbad \\xff byte\\nno newline at the end')\nsys.stdout.buffer.flush()\n"
               "sys.stderr.buffer.write(b'stderr \\xfe\\r\\nlast without newline')\nsys.stderr.buffer.flush()\nsys.exit(3)\n")
    environment = dict(selftest.base_environment)
    reference = subprocess.run([sys.executable, "-c", awkward], capture_output=True, text=True, errors="replace", env=environment, check=False)
    _exit_code, collected, stderr = selftest_capture_run(selftest, awkward)
    selftest.check('边跑边收：没有 GPU 阶段现场行时 stdout、stderr 与 capture_output=True, text=True, errors="replace" 收的逐字相同（CRLF、单独的 CR、坏 UTF-8、末尾没换行）',
                   collected is not None and collected["returncode"] == reference.returncode == 3
                   and collected["stdout"] == reference.stdout and collected["stderr"] == reference.stderr,
                   f"收的 {collected and (collected['stdout'], collected['stderr'])!r}；标准库 {(reference.stdout, reference.stderr)!r}；子进程 stderr 尾 {stderr[-300:]!r}")
    on_stdout = "GPU_LIFECYCLE unix_microseconds=1 phase=on_stdout\ntest tests::doubles ... ok\n"
    _exit_code, kept, stderr = selftest_capture_run(selftest, f"import sys\nsys.stdout.write({on_stdout!r})\n")
    selftest.check("边跑边收：stdout 里同样前缀的行不滤，逐字留着、不进汇总",
                   kept is not None and kept["stdout"] == on_stdout and kept["stderr"] == "",
                   f"{kept}；子进程 stderr 尾 {stderr[-300:]!r}")
    interleaved = "test tests::doubles ... GPU_LIFECYCLE unix_microseconds=1 phase=between_the_result_words\nFAILED\n"
    _exit_code, loose, stderr = selftest_capture_run(selftest, f"import sys\nsys.stderr.write({interleaved!r})\nsys.exit(101)\n")
    row = Row(1, "selftest-interleaved", "crates/tiny/src/lib.rs", "a", "b", "-p tiny --lib -- doubles", "doubles", ())
    verdict = verdict_of_run(row, loose["returncode"], loose["stdout"] + loose["stderr"], environment)[0] if loose is not None else None
    selftest.check("边跑边收：行中间夹着 GPU 阶段现场字样的不滤：结果词被它隔开的 FAILED 照判抓到",
                   loose is not None and loose["stderr"] == interleaved and verdict == "caught",
                   f"判成 {verdict}；{loose}；子进程 stderr 尾 {stderr[-300:]!r}")
    source = os.path.join(STAGE_REPOSITORY, "crates", "singlefs-verification", "src", "gpu_unit_checks.rs")
    label = "边跑边收：滤的前缀与 crate singlefs-verification 的 gpu_lifecycle_line 打的前缀是同一个"
    if not os.path.isfile(source):
        selftest.skip(label, f"{source} 不在（这棵树没带 crates）")
    else:
        with open(source, encoding="utf-8") as handle:
            source_text = handle.read()
        selftest.check(label, f'"{gpu_lifecycle_prefix(environment)}{{unix_microseconds}} ' in source_text,
                       f"{source} 里没有 {gpu_lifecycle_prefix(environment)!r} 起头的那一行格式：照 gpu_lifecycle_line 改 gpu_lifecycle_prefix")


def selftest_output_stop_run(selftest, work, handler_kind):
    """一条写 100 行 GPU 阶段现场行之后挂住的命令经 run_capturing_output 跑，等进度行报出 100 行都收了，对自己起的子进程发 TERM：(退出码, 它的 stderr, 进度行等到没有)。"""
    producer = ("import os, sys, time\n"
                "for number in range(1, 101):\n"
                "    sys.stderr.write(f'GPU_LIFECYCLE unix_microseconds={number} phase=numbered\\n')\n"
                "sys.stderr.flush()\n"
                "parent = os.getppid()\n"
                "deadline = time.monotonic() + 120\n"
                "while os.getppid() == parent and time.monotonic() < deadline:\n"
                "    time.sleep(0.05)\n")
    stderr_path = os.path.join(work, f"output-stop-{handler_kind}-stderr.txt")
    with open(stderr_path, "w", encoding="utf-8") as stderr_handle:
        child = subprocess.Popen([sys.executable, "-c", selftest_capture_child_program(), MODULE_PATH, json.dumps([sys.executable, "-c", producer]), handler_kind],
                                 env=dict(selftest.base_environment, GATE_MUTATION_PROGRESS_SECONDS="1"), stdout=subprocess.DEVNULL, stderr=stderr_handle)

    def stderr_text():
        with open(stderr_path, encoding="utf-8", errors="replace") as handle:
            return handle.read()
    deadline = time.monotonic() + 120
    while "lifecycle_lines_dropped=100 " not in stderr_text() and child.poll() is None and time.monotonic() < deadline:
        time.sleep(0.05)
    progress_seen = "lifecycle_lines_dropped=100 " in stderr_text()
    if child.poll() is None:
        child.send_signal(signal.SIGTERM)
    try:
        child.wait(timeout=120)
    except subprocess.TimeoutExpired:
        child.kill()
        child.wait()
    return child.returncode, stderr_text(), progress_seen


def selftest_output_stop_cells(selftest, work):
    """被停时留下的行进任务日志（文件头「边跑边收、边跑边报」）：工作进程那种默认处理、父进程那种接住的信号，两种都打最后 32 行、照各自的退法退。"""
    def tail_is_exact(text):
        return (all(f"GPU_LIFECYCLE unix_microseconds={number} phase=numbered\n" in text for number in range(69, 101))
                and "GPU_LIFECYCLE unix_microseconds=68 phase=numbered\n" not in text)
    code, text, progress_seen = selftest_output_stop_run(selftest, work, "default-handler")
    selftest.check("被停（工作进程的默认处理）：留下的最后 32 行打进日志，照默认处理退 -15",
                   progress_seen and code == -signal.SIGTERM and tail_is_exact(text), f"进度行等到 {progress_seen}，退 {code}；stderr 尾 {text[-500:]!r}")
    code, text, progress_seen = selftest_output_stop_run(selftest, work, "own-handler")
    selftest.check("被停（父进程接住的信号）：留下的最后 32 行打进日志，对整组发 TERM、等它退，退 143",
                   progress_seen and code == 128 + signal.SIGTERM and tail_is_exact(text), f"进度行等到 {progress_seen}，退 {code}；stderr 尾 {text[-500:]!r}")


def selftest_output_progress_cells(selftest, work):
    """边跑边报（文件头「边跑边收、边跑边报」）：有新输出按间隔打，字段写全；不出输出照样每个间隔打一行、不丢；进程按类别归得对；
    在算行：变异行、基线批、覆盖组三种 TMPDIR 前缀各点名自己的格，建目录的前缀（judge_row、run_baseline_batch、judge_coverage 的 mkdtemp）computing_cell_label 都认得出。"""
    writing = ("import sys, time\nsys.stderr.write('GPU_LIFECYCLE unix_microseconds=1 phase=single\\n')\nsys.stderr.flush()\n"
               "deadline = time.monotonic() + 4\n"
               "while time.monotonic() < deadline:\n    print('still writing', flush=True)\n    time.sleep(0.2)\n")
    _exit_code, _payload, stderr = selftest_capture_run(selftest, writing, GATE_MUTATION_PROGRESS_SECONDS="1")
    progress_form = re.compile(r"^TASK_SERVER_HEARTBEAT MUTATION_WORKER_PROGRESS process=\d+ elapsed_seconds=\d+ stdout_bytes=[1-9]\d* stderr_bytes=[1-9]\d* lifecycle_lines_dropped=1 lifecycle_bytes_dropped=[1-9]\d* "
                               r"anon_bytes=python:[1-9]\d*,test-binary:\d+,rustc:\d+,prune-server:\d+,other:\d+ command=\S", re.M)
    selftest.check("边跑边报：子进程一直有新输出时按间隔打 TASK_SERVER_HEARTBEAT 起头的 MUTATION_WORKER_PROGRESS，带用时、字节、滤掉的条数与按类别的 RssAnon",
                   len(progress_form.findall(stderr)) >= 2, f"对上写法的 {len(progress_form.findall(stderr))} 行；子进程 stderr 尾 {stderr[-600:]!r}")
    # 假测试二进制：开头打一行之后 10 个间隔（间隔 1 秒，按比例代替 60 秒间隔下的 10 分钟）一个字都不出。每个间隔点一行：至少 9 行，
    #   头一行在 2 秒以内、末一行不早于 9 秒，相邻两行的 elapsed_seconds 至多差 2（中间没有跨过一个间隔点不打的）
    silent = "import time\nprint('one line', flush=True)\ntime.sleep(10)\n"
    _exit_code, payload, stderr = selftest_capture_run(selftest, silent, GATE_MUTATION_PROGRESS_SECONDS="1")
    silent_elapsed = [int(found) for found in re.findall(r"^TASK_SERVER_HEARTBEAT MUTATION_WORKER_PROGRESS process=\d+ elapsed_seconds=(\d+) ", stderr, re.M)]
    selftest.check("边跑边报：子进程 10 个间隔不出输出时照样每个间隔打一行进度，不丢",
                   payload is not None and payload["stdout"] == "one line\n" and len(silent_elapsed) >= 9 and silent_elapsed[0] <= 2 and silent_elapsed[-1] >= 9
                   and all(later - earlier <= 2 for earlier, later in zip(silent_elapsed, silent_elapsed[1:])),
                   f"这 10 秒里进度行 {len(silent_elapsed)} 行（间隔 1 秒，只有开头一行输出），elapsed_seconds 依次 {silent_elapsed}；{payload}；子进程 stderr 尾 {stderr[-300:]!r}")
    kinds = {("python3", "/usr/bin/python3.12"): "python", ("rustc", "/toolchain/bin/rustc"): "rustc", ("prune-server", "/build/release/prune-server"): "prune-server",
             ("crash_judge_gpu", "/build/release/deps/crash_judge_gpu-0123456789abcdef"): "test-binary", ("cargo", "/toolchain/bin/cargo"): "other", ("", ""): "other"}
    answers = {pair: process_kind(*pair) for pair in kinds}
    selftest.check("边跑边报：进程按类别归：python、rustc、prune-server、deps/ 底下的测试二进制、别的", answers == kinds, f"{answers}")
    # 在算行（output_computing_line）：门槛调成 2 秒（按比例代替 300 秒），TMPDIR 照变异那一格的前缀起名，命令后面接 cargo test 那样的目标与过滤词（python -c 不读它们）。
    #   假测试二进制 6.5 秒里每 0.1 秒往 stderr 写一行 GPU 阶段现场行（全被滤掉、结果里一行不留）：至少 2 行在算行，不带心跳前缀，点名这一格与测试，滤掉的条数大于 0；
    #   另一个 5 秒里一个字都不出：一行在算行都不打（心跳照打）
    computing = ("import sys, time\ndeadline = time.monotonic() + 6.5\nnumber = 0\n"
                 "while time.monotonic() < deadline:\n    number += 1\n"
                 "    sys.stderr.write(f'GPU_LIFECYCLE unix_microseconds={number} phase=computing\\n')\n    sys.stderr.flush()\n    time.sleep(0.1)\n")

    def computing_run(producer, directory_name="line-42-sample"):
        """在子进程里经 run_capturing_output 跑 producer（命令后面接 test --test sample_pool_target -- sample_filter --exact），TMPDIR 是 work 下的 directory_name：
        (子进程退出码, 它的 stderr)。"""
        cell_directory = os.path.join(work, "computing-line", directory_name)
        os.makedirs(cell_directory, exist_ok=True)
        command = [sys.executable, "-c", producer, "test", "--test", "sample_pool_target", "--", "sample_filter", "--exact"]
        child = subprocess.run([sys.executable, "-c", selftest_capture_child_program(), MODULE_PATH, json.dumps(command), "default-handler"],
                               env=dict(selftest.base_environment, GATE_MUTATION_PROGRESS_SECONDS="1", GATE_MUTATION_COMPUTING_LINE_SECONDS="2", TMPDIR=cell_directory),
                               capture_output=True, text=True, errors="replace", timeout=600, check=False)
        return child.returncode, child.stderr

    def computing_form_of(cell):
        """点名 cell 这一格的在算行的写法（头一组是 elapsed_seconds）。"""
        return re.compile(rf"^MUTATION_WORKER_COMPUTING cell={cell} targets=test:sample_pool_target tests=sample_filter elapsed_seconds=([0-9]+) window_seconds=[0-9]+ "
                          r"lifecycle_lines_dropped_in_window=[1-9][0-9]* lifecycle_bytes_dropped_in_window=[1-9][0-9]* output_bytes_in_window=[1-9][0-9]*$", re.M)
    code, stderr = computing_run(computing)
    computing_elapsed = [int(found) for found in computing_form_of("mutation-row:42").findall(stderr)]
    selftest.check("在算行：子进程 6.5 秒里只出被滤掉的 GPU 阶段现场行、门槛 2 秒时，至少 2 行不带心跳前缀的 MUTATION_WORKER_COMPUTING，点名这一格（变异表第 42 行）与测试，滤掉的条数大于 0",
                   code == 0 and len(computing_elapsed) >= 2 and computing_elapsed[0] >= 2 and "TASK_SERVER_HEARTBEAT MUTATION_WORKER_COMPUTING" not in stderr,
                   f"退 {code}；对上写法的在算行 elapsed_seconds 依次 {computing_elapsed}；子进程 stderr 尾 {stderr[-600:]!r}")
    code, stderr = computing_run("import time\ntime.sleep(5)\n")
    heartbeats = re.findall(r"^TASK_SERVER_HEARTBEAT MUTATION_WORKER_PROGRESS ", stderr, re.M)
    selftest.check("在算行：子进程 5 秒里一个字都不出、门槛 2 秒时，一行在算行都不打（心跳照打：真卡死服务照样判得出）",
                   code == 0 and "MUTATION_WORKER_COMPUTING" not in stderr and len(heartbeats) >= 3,
                   f"退 {code}；心跳 {len(heartbeats)} 行；子进程 stderr 尾 {stderr[-600:]!r}")
    labels = {"line-7-abc": "mutation-row:7", "baseline-12-x": "baseline-batch:12", "coverage-3-y": "coverage-group:3", "shard-1-z": "-", "": "-"}
    label_answers = {name: computing_cell_label({"TMPDIR": os.path.join(work, name) if name else ""}) for name in labels}
    bad_threshold_root = os.path.join(work, "computing-line-bad-threshold")
    build_selftest_repository(bad_threshold_root, SELFTEST_TABLE_ROWS)
    bad_threshold = selftest.run(bad_threshold_root, GATE_MUTATION_COMPUTING_LINE_SECONDS="0")
    selftest.check("在算行：TMPDIR 的前缀认成变异行、基线批、覆盖组，认不出写 -；门槛写成不是正整数 run_stage 拒、一条都不跑",
                   label_answers == labels and bad_threshold[0] == 1 and bad_threshold[3] == []
                   and "GATE_MUTATION_COMPUTING_LINE_SECONDS 写成了「0」" in bad_threshold[1] + bad_threshold[2],
                   f"{label_answers}；门槛写 0 退 {bad_threshold[0]}，{bad_threshold[1][-200:]}{bad_threshold[2][-200:]}")
    # 基线批与覆盖组的 TMPDIR 前缀：同一段子进程（3.5 秒，门槛 2 秒）各跑一趟，在算行点名的格是 baseline-batch:42、coverage-group:42
    shorter = computing.replace("+ 6.5\n", "+ 3.5\n")
    assert shorter != computing
    code, stderr = computing_run(shorter, "baseline-42-sample")
    baseline_elapsed = [int(found) for found in computing_form_of("baseline-batch:42").findall(stderr)]
    selftest.check("在算行：TMPDIR 是基线批的前缀（baseline-42-…）时，在算行点名的格是 baseline-batch:42",
                   code == 0 and len(baseline_elapsed) >= 1 and baseline_elapsed[0] >= 2,
                   f"退 {code}；对上写法的在算行 elapsed_seconds 依次 {baseline_elapsed}；子进程 stderr 尾 {stderr[-600:]!r}")
    code, stderr = computing_run(shorter, "coverage-42-sample")
    coverage_elapsed = [int(found) for found in computing_form_of("coverage-group:42").findall(stderr)]
    selftest.check("在算行：TMPDIR 是覆盖组的前缀（coverage-42-…）时，在算行点名的格是 coverage-group:42",
                   code == 0 and len(coverage_elapsed) >= 1 and coverage_elapsed[0] >= 2,
                   f"退 {code}；对上写法的在算行 elapsed_seconds 依次 {coverage_elapsed}；子进程 stderr 尾 {stderr[-600:]!r}")
    # 建目录的前缀与认格名的是同一处写法：读 judge_row、run_baseline_batch、judge_coverage 里 tempfile.mkdtemp 的 prefix，造一个这样的目录名交 computing_cell_label 认
    creators = {"judge_row": "mutation-row", "run_baseline_batch": "baseline-batch", "judge_coverage": "coverage-group"}
    templates = selftest_temporary_directory_templates(set(creators))
    recognized = {name: computing_cell_label(dict(selftest.base_environment, TMPDIR=os.path.join(work, templates[name][0].replace("<N>", "57") + "sample")))
                  for name in creators if len(templates.get(name, ())) == 1 and templates[name][0].count("<N>") == 1}
    selftest.check("在算行：judge_row、run_baseline_batch、judge_coverage 建临时目录的前缀各只有一处写法，computing_cell_label 都认得出（变异行、基线批、覆盖组）",
                   set(recognized) == set(creators) and all(recognized[name] == f"{cell}:57" for name, cell in creators.items()),
                   f"三个函数里 mkdtemp 的 prefix 写法 {templates}；认出的格 {recognized}")


def selftest_temporary_directory_templates(function_names):
    """入口与拆出的各份模块（MODULE_SOURCE_PATHS）里 function_names 各函数里 tempfile.mkdtemp 的 prefix 写法：{函数名: [模板]}，f 串里的每一处代入写成 <N>（judge_row 的 f"line-{row.line_number}-" 得 "line-<N>-"）。
    prefix 不是 f 串的不收（读不出模板，调用方核不过）。"""
    templates = {}
    for path in MODULE_SOURCE_PATHS:
        with open(path, encoding="utf-8") as handle:
            tree = ast.parse(handle.read())
        for function in ast.walk(tree):
            if not isinstance(function, ast.FunctionDef) or function.name not in function_names:
                continue
            for call in ast.walk(function):
                if isinstance(call, ast.Call) and isinstance(call.func, ast.Attribute) and call.func.attr == "mkdtemp":
                    for keyword in call.keywords:
                        if keyword.arg == "prefix" and isinstance(keyword.value, ast.JoinedStr):
                            templates.setdefault(function.name, []).append(
                                "".join(part.value if isinstance(part, ast.Constant) else "<N>" for part in keyword.value.values))
    return templates
