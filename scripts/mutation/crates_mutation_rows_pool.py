"""crates-mutation-rows.py 拆出的一份。
先编一份依赖（先编组、拷依赖产物）、工作进程的初始化、基线与变异与覆盖三种任务、进程池 run_pool，连同这几样的自证格。
入口文件头写着这一份在的位置、弄坏开关表与用法；入口 crates-mutation-rows.py 从各份 import 全部名字再导出，别处照旧按入口取名字。
拆出的各份只向前 import（base → judging → support → static → units → baseline → targets → memory → runs → pool → plan → stage → commands → driver），没有环。
"""
import collections
import concurrent.futures
import contextlib
import fnmatch
import hashlib
import heapq
import io
import json
import multiprocessing
import os
import random
import re
import shutil
import signal
import stat
import subprocess
import sys
import tempfile
import threading
import time
import tomllib
import traceback
from concurrent.futures.process import BrokenProcessPool
from crates_mutation_rows_base import (BREAK_VARIABLES, CARGO_CONFIGURATION_DIRECTORY, COVERAGE_TARGET_SUFFIX, DISK_FREE_VARIABLE,
    DISK_RESERVE_VARIABLE, HeavyThreadDefaultError, IGNORED_WHEN_COPYING_INTO_EACH_SHARD, MODULE_PATH, PER_MUTATION_TEMPORARY_DIRECTORY_NAME, POOL_ERROR_CHILD_PROGRAM,
    PRUNE_SERVER_TEMPORARY_SERVICE_SCRIPT, RECORD_HEADER, RUN_BUILD_DIRECTORY_NAME, Row, SEED_COPY_TEMPORARY_SUFFIX, SEED_DROPPED_SWITCHES,
    SEED_LOCK_SUFFIX, SEED_SELECTOR_SWITCHES, SEED_SOURCE_SUFFIX, SEED_TARGET_SUFFIX, SEED_VALUE_DROPPED_SWITCHES, SEED_VALUE_KEPT_SWITCHES,
    SEED_VALUE_SELECTOR_SWITCHES, SHARD_ROOT_PREFIX, STOP_SIGNALS, TABLE, TARGET_DIRECTORY_CAP_VARIABLE, UNIT_ARTIFACT_NAME, UNIT_DIRECTORY_NAME,
    WORKER_PEAK_SAMPLE_SECONDS, WORKER_STATE, break_is, format_size, mutation_units, note, print_rejection, process_usage_bytes, read_table,
    split_at_double_dash)
from crates_mutation_rows_judging import (baseline_arguments, gpu_test_run, judge_batch, judge_row, output_tail, prepare_shard)
from crates_mutation_rows_support import (build_selftest_repository, per_row_how, plain_invocations, replace_in_file, selftest_new_library_copy,
    selftest_verdict_view, selftest_write, source_hash)
from crates_mutation_rows_static import (release_registered_targets)
from crates_mutation_rows_units import (read_timing, row_content_hash, run_records_directory, selftest_key_pool_closure_keys, selftest_write_timings,
    snapshot_identity, snapshot_of)
from crates_mutation_rows_baseline import (baseline_chunks, baseline_long_test_seconds, baseline_test_groups, batch_key,
    selftest_heavy_chunk_repository)
from crates_mutation_rows_targets import (claim_worker_slot, deferred_write_library, environment_records, hold_worker_target_lock,
    own_workers_stopped_on_stop, raise_stop_requested, remove_owned_build_root, run_build_target_directory, target_over_cap_directory,
    target_over_cap_reports, target_reclaim_lines, trim_target_directory, trim_worker_target, unwritten_library)
from crates_mutation_rows_memory import (CARGO_JOBS_VARIABLE, base_reading_line, cargo_jobs_for, cell_peak_packages, compile_thread_budget, grown_worker_count, memory_admits,
    new_memory_admission, peak_key_label, process_tree_usage, refresh_memory_admission, sample_worker_peak, unmeasured_packages_at,
    worker_peak_scope)
from crates_mutation_rows_runs import (coverage_build_settings, gpu_released_test_targets, judge_coverage, measure_and_remove_temporary_directory,
    mutation_run_prune_server, mutation_run_prune_server_binary, run_capturing_output, test_runs_multi_host_configuration, write_row_timing)


def init_worker(sequence_counter, shard_root, context):
    """工作进程启动时跑一次：领一个稳定的片号、建自己的源码副本与编译目录。片号从共用编译目录那一层的锁文件领（claim_worker_slot，跨调用不重），
    活是动态领的（crates-mutation-replay 文件头）。sequence_counter 只在拿不到锁文件、或弄坏开关 local-shard-numbering 下当本地序号用。"""
    # 父进程在 run_pool 里把 STOP_SIGNALS 接成「收尾再退」；工作进程照默认（收到就停），不把父进程的接法带进来
    #   （它起着的临时 prune-server 经 mutation_run_prune_server_binary 写的包装带父进程死亡信号，工作进程一死就跟着停）
    for signal_number in STOP_SIGNALS:
        signal.signal(signal_number, signal.SIG_DFL)
    # 每个工作进程的峰值（文件头「内存上限」的「每个工作进程的峰值」）：起采样线程量本进程整棵树，读数记在此刻在判的那一格点名的包下（WORKER_STATE["peak_scope"]，
    #   baseline_job、mutation_job、coverage_job 在判每一格的前后设与清），按编译并行度记进运行记录目录。
    #   被判的不是本仓（门禁样本根）不起（worker_peaks_recorded_for）；弄坏开关 worker-peaks-unrecorded：都不起
    if context.get("worker_peaks_recorded") and not break_is(context["environment"], "worker-peaks-unrecorded"):
        threading.Thread(target=sample_worker_peak, name="worker-peak-sampler", daemon=True,
                         args=(context["run_records_directory"], context["environment"]["CARGO_BUILD_JOBS"], context["environment"],
                               context.get("worker_peak_seen"))).start()
    worker_sequence, slot_lock = claim_worker_slot(context["shared_target_directory"], sequence_counter, context["environment"])
    WORKER_STATE["slot_lock"] = slot_lock   # 拿到进程退出：内核放，别的调用的工作进程才试得到这一片
    work, target_directory = prepare_shard(worker_sequence, shard_root, context)
    note(f"  … 工作进程 {os.getpid()} 领到片号 {worker_sequence}（编译目录 {target_directory}；"
         + (f"片号锁 {slot_lock.name}" if slot_lock is not None else "没有片号锁，按本地序号") + "）")
    environment = dict(context["environment"])
    environment["CARGO_TARGET_DIR"] = target_directory
    # 在用的标记（WORKER_TARGET_LOCK_NAME）：进程退出内核就放，别的一趟回收时拿不到排他锁就不删这份
    WORKER_STATE["target_lock"] = hold_worker_target_lock(target_directory)
    # 先编一份依赖（文件头「先编一份依赖」）：先编成的那一份里外部依赖的产物拷进自己的两份编译目录（普通、插桩），工作区的包照旧自己编
    copy_seed_dependencies(context, target_directory)
    # 这份副本的整树快照：只当基线记录的一样属性记下（弄坏开关 baseline-by-tree-snapshot 按它比），不当复用的依据
    snapshot = snapshot_identity(snapshot_of(work, context["copied"]))
    # 判完一格就原子写进判定库（Library.write：同目录临时文件、fsync、改名），整组被硬杀时判过的格留着，重跑同一条命令照键复用。弄坏开关 records-written-at-end：攒到进程池收尾才写
    library = mutation_units.Library(context["library_directory"], RECORD_HEADER)
    if break_is(context["environment"], "records-written-at-end"):
        library = deferred_write_library(library)
    # 缩过域的一趟（文件头「缩过域的一趟」）：判定、基线与覆盖一样都不写进判定库（库里的是全域下判的），读照读。弄坏开关 narrowed-verdicts-recorded：照写
    if context.get("narrowing") and not break_is(context["environment"], "narrowed-verdicts-recorded"):
        library = unwritten_library(library)
    WORKER_STATE.update(work=work, environment=environment, context=context, shard_root=shard_root, worker_sequence=worker_sequence,
                        per_mutation_temporary_root=os.path.join(shard_root, PER_MUTATION_TEMPORARY_DIRECTORY_NAME),
                        snapshot=snapshot, library=library, unit_keys={},   # 包目录 → 记覆盖用的单元键（record_coverage）
                        # 起进程那一下、找多机配置文件、取放开 --test-threads 1 的测试目标名单与起临时 prune-server 经这几个引用：起的程序（cargo、prune-server）各有自己的身份，
                        # 多机配置是交给测试的一份文件，名单是调度参数（不改判定），临时服务的起法在客户端自证那一份
                        # 放开名单照被判的根现取（gpu_released_test_targets：验证包登记的只经卡池取卡的测试文件读被判那棵树的那一份）
                        launch=run_capturing_output, multi_host_configuration=test_runs_multi_host_configuration,
                        gpu_released_targets=lambda: gpu_released_test_targets(context["root"], context["environment"]), temporary_service=mutation_run_prune_server,
                        # 不碰判定记录的记账：量 TMPDIR 占盘再删、写用时表，同样经引用调
                        remove_temporary=measure_and_remove_temporary_directory, write_timing=write_row_timing)


# ── 先编一份依赖（文件头「先编一份依赖」） ───────────────────────────────────────

def seed_group_of(before):
    """一行 cargo test 参数里 `--` 之前那一段 → (组的键, 先编这一组用的参数)。键只留选包、特性、档、目标三元组与别的开关（外部依赖编成什么只随它们变），
    选目标的开关（--test <名>、--lib……）、TESTNAME 过滤词与只管输出的开关（-q、-j……）不进键；先编用的参数是键里那几样加这一行自己选的目标。"""
    key, selectors, index = [], [], 0
    while index < len(before):
        word = before[index]
        name, equals, _value = word.partition("=")
        width = 1 if equals else 2
        if name in SEED_VALUE_SELECTOR_SWITCHES:
            selectors += before[index:index + width]
            index += width
        elif name in SEED_SELECTOR_SWITCHES:
            selectors.append(word)
            index += 1
        elif name in SEED_VALUE_DROPPED_SWITCHES:
            index += width
        elif name in SEED_DROPPED_SWITCHES or not word.startswith("-"):
            index += 1
        elif name in SEED_VALUE_KEPT_SWITCHES:
            key += before[index:index + width]
            index += width
        else:
            key.append(word)
            index += 1
    return tuple(key), key + selectors


def read_lock_file_packages(root):
    """根下 Cargo.lock 的 [[package]]（读不了、读不懂、没有交空表）。"""
    try:
        with open(os.path.join(root, "Cargo.lock"), "rb") as handle:
            packages = tomllib.load(handle).get("package", [])
    except (OSError, tomllib.TOMLDecodeError):
        return []
    return [package for package in packages if isinstance(package, dict) and isinstance(package.get("name"), str)]


def path_packages_reaching_external(root):
    """根下 Cargo.lock 里每个 path 包（没有 source 的）顺 dependencies 够不够得着带 source 的包：{包名: 真或假}。可选依赖照算（Cargo.lock 不分特性）。"""
    packages = read_lock_file_packages(root)
    by_name = collections.defaultdict(list)
    for package in packages:
        by_name[package["name"]].append(package)

    def resolved(reference):
        name, _space, rest = str(reference).partition(" ")
        version = rest.split(" ")[0] if rest else None
        return [package for package in by_name.get(name, []) if version is None or package.get("version") == version]
    reaching = {}
    for package in packages:
        if "source" in package:
            continue
        seen, stack, found = set(), [package], False
        while stack and not found:
            for dependency in (item for reference in stack.pop().get("dependencies", []) for item in resolved(reference)):
                identity = (dependency["name"], dependency.get("version"), dependency.get("source"))
                if identity in seen:
                    continue
                seen.add(identity)
                if "source" in dependency:
                    found = True
                    break
                stack.append(dependency)
        reaching[package["name"]] = reaching.get(package["name"], False) or found
    return reaching


def seed_groups(rows, reaching):
    """这一趟要先编的组：{组的键: 先编这一组用的参数}，按行的次序；选的包（-p，没写是整个工作区）都够不着外部依赖的组不编。"""
    groups = collections.OrderedDict()
    for row in rows:
        key, arguments = seed_group_of(split_at_double_dash(row.arguments)[0])
        if key in groups:
            continue
        packages = [key[index + 1] for index, word in enumerate(key[:-1]) if word in ("-p", "--package")]
        packages += [word.partition("=")[2] for word in key if word.startswith(("-p=", "--package="))]
        if any(reaching.get(package, False) for package in (packages or list(reaching))):
            groups[key] = arguments
    return groups


@contextlib.contextmanager
def seed_lock(shared, exclusive):
    """先编那一份的锁：建（同步源码、编）的时候排他，工作进程拷的时候共享；拿不到就等。"""
    import fcntl
    path = os.path.normpath(shared) + SEED_LOCK_SUFFIX
    os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
    with open(path, "a") as handle:
        fcntl.flock(handle, fcntl.LOCK_EX if exclusive else fcntl.LOCK_SH)
        try:
            yield
        finally:
            fcntl.flock(handle, fcntl.LOCK_UN)


def files_under(base, item):
    """base 底下 item（一个文件或一个目录）里的文件，相对 base；跳过 IGNORED_WHEN_COPYING_INTO_EACH_SHARD 的目录，不在交空表。"""
    top = os.path.join(base, item)
    if not os.path.isdir(top):
        return [item] if os.path.lexists(top) else []
    found = []
    for directory, subdirectories, file_names in os.walk(top):
        subdirectories[:] = sorted(name for name in subdirectories
                                   if not any(fnmatch.fnmatch(name, pattern) for pattern in IGNORED_WHEN_COPYING_INTO_EACH_SHARD))
        found += [os.path.relpath(os.path.join(directory, name), base) for name in sorted(file_names)]
    return found


def package_directories(source):
    """源码副本里的包目录（相对副本根）：crates/ 底下一层、有 Cargo.toml 的。"""
    crates = os.path.join(source, "crates")
    try:
        names = sorted(os.listdir(crates))
    except OSError:
        return set()
    return {f"crates/{name}" for name in names if os.path.isfile(os.path.join(crates, name, "Cargo.toml"))}


def sync_seed_source(root, copied, seed_source, environment):
    """先编那一份的源码副本按内容同步：内容没变的文件不动；变了、新加的写进去（修改时刻是此刻），副本里有、拷贝范围里没有的删掉；
    改动所在的包整包把修改时刻摸成此刻，不在哪个包底下的改动摸全部包。回 (摸过的包目录的集合, 改动的文件数)。
    弄坏开关 seedmtimekept：内容变了的文件写进去之后修改时刻改回原来的、一个包都不摸（摸过的包照样交回，给 stale_seed_artifacts 核）。"""
    items = [item for item in copied if not (item == CARGO_CONFIGURATION_DIRECTORY and break_is(environment, "cargoconfignotcopied"))]
    wanted = sorted({relative for item in items for relative in files_under(root, item)})
    os.makedirs(seed_source, exist_ok=True)
    changed = []
    for relative in wanted:
        source, target = os.path.join(root, relative), os.path.join(seed_source, relative)
        with open(source, "rb") as handle:
            data = handle.read()
        previous = None
        if os.path.isdir(target) and not os.path.islink(target):
            shutil.rmtree(target)
        elif os.path.lexists(target):
            with open(target, "rb") as handle:
                if handle.read() == data:
                    continue
            previous = os.stat(target)
        os.makedirs(os.path.dirname(target), exist_ok=True)
        temporary = target + SEED_COPY_TEMPORARY_SUFFIX
        with open(temporary, "wb") as handle:
            handle.write(data)
        shutil.copymode(source, temporary)
        os.replace(temporary, target)
        if previous is not None and break_is(environment, "seedmtimekept"):
            os.utime(target, ns=(previous.st_atime_ns, previous.st_mtime_ns))
        changed.append(relative)
    tops = {item.split("/")[0] for item in items}
    for name in sorted(os.listdir(seed_source)):
        if name not in tops:
            path = os.path.join(seed_source, name)
            changed += files_under(seed_source, name) or [name]
            shutil.rmtree(path) if os.path.isdir(path) and not os.path.islink(path) else os.unlink(path)
    for relative in sorted(set(relative for item in items for relative in files_under(seed_source, item)) - set(wanted)):
        os.unlink(os.path.join(seed_source, relative))
        changed.append(relative)
    packages = package_directories(seed_source)
    touched = set()
    for relative in changed:
        parts = relative.split("/")
        package = "/".join(parts[:2]) if len(parts) > 2 and parts[0] == "crates" else None
        touched |= {package} if package in packages else packages
    if not break_is(environment, "seedmtimekept"):
        now = time.time_ns()
        for package in sorted(touched):
            for relative in files_under(seed_source, package):
                os.utime(os.path.join(seed_source, relative), ns=(now, now), follow_symlinks=False)
    return touched, len(changed)


def seed_unit_identity(message):
    """一条 compiler-artifact 消息 → 编译单元的身份（JSON 文本）：package_id、manifest_path、目标的 kind、crate_types、name，档、特性、filenames、executable。
    filenames 里 deps/ 下的文件名带 cargo 按包、特性、档、目标三元组、rustc 参数与依赖算的单元哈希（-C metadata），同一份编译目录里两个单元不会同名；
    同包另一套特性或档编出来的单元哈希不同、身份也不同。其余几样在 filenames 为空（只检查）、或消息缺字段时垫底。"""
    target = message.get("target") or {}
    return json.dumps([message.get("package_id"), message.get("manifest_path"), target.get("kind"), target.get("crate_types"), target.get("name"),
                       message.get("profile"), sorted(message.get("features") or []), message.get("filenames"), message.get("executable")], sort_keys=True)


def stale_seed_artifacts(output, seed_source, touched, compiled=None):
    """先编一组的 cargo JSON 输出里，摸过的包（touched）编出来而 fresh 为真的产物：[「<包目录> 的 <目标名>」…]。
    compiled 是同一份编译目录（普通或插桩）里前面的组已经以非 fresh 编出过的单元身份（seed_unit_identity）的集合，这一组里非 fresh 的往里加；
    fresh 的产物身份在里面的是复用前面的组编出来的，不算坏；头一次出现就是 fresh 的才算。不给（None）是只看这一组自己（弄坏开关 seedfreshacrossgroups 的样子）。"""
    root = os.path.realpath(seed_source)
    compiled = set() if compiled is None else compiled
    stale = []
    for line in output.splitlines():
        if not line.startswith("{"):
            continue
        try:
            message = json.loads(line)
        except ValueError:
            continue
        if not isinstance(message, dict) or message.get("reason") != "compiler-artifact":
            continue
        identity = seed_unit_identity(message)
        if not message.get("fresh"):
            compiled.add(identity)
            continue
        if identity in compiled:
            continue
        package = os.path.relpath(os.path.dirname(os.path.realpath(message.get("manifest_path") or "/")), root)
        if package in touched:
            stale.append(f"{package} 的 {(message.get('target') or {}).get('name', '?')}")
    return stale


def seed_jobs(context):
    """先编那一份的 CARGO_BUILD_JOBS：GATE_MUTATION_CARGO_JOBS 设了用它，不然取每台默认重活线程数（scripts/hosts/heavy_threads.py 的 default_heavy_threads，
    经 compile_thread_budget；这时还没有工作进程在跑，整台的线程都给它，取不到抛 HeavyThreadDefaultError）。"""
    environment = context["environment"]
    return str(cargo_jobs_for(environment, 1, compile_thread_budget(environment)))


def selftest_seed_jobs_cell(selftest):
    """先编那一份的并行度（seed_jobs，文件头「先编一份依赖」），只调函数、不起 cargo：没给 GATE_MUTATION_CARGO_JOBS 取每台默认重活线程数（SINGLEFS_THREAD_CAP=7 取 7、=11 取 11，
    不看本机核数），给了照它（GATE_MUTATION_CARGO_JOBS=3），取不到又没给（SINGLEFS_THREAD_CAP=abc）抛 HeavyThreadDefaultError。
    再一格：工作进程拷依赖产物的线程数（seed_copy_threads）没给 CARGO_BUILD_JOBS 时取每台默认重活线程数 ÷ 工作进程数、至少 1（SINGLEFS_THREAD_CAP=12：4 个工作进程取 3、1 个取 12、16 个取 1），
    给了照它，取不到又没给（SINGLEFS_THREAD_CAP=abc）抛 HeavyThreadDefaultError。
    弄坏开关 workers-by-processor-half（整台线程数取本机核数）让第一格判错；seed-copy-threads-unregistered（读不出 CARGO_BUILD_JOBS 时固定 1 条线程）让再一格判错。"""
    environment = {key: value for key, value in selftest.base_environment.items() if key != CARGO_JOBS_VARIABLE}
    by_registry = [seed_jobs({"environment": dict(environment, SINGLEFS_THREAD_CAP=cap)}) for cap in ("7", "11")]
    given = seed_jobs({"environment": dict(environment, SINGLEFS_THREAD_CAP="7", GATE_MUTATION_CARGO_JOBS="3")})
    try:
        seed_jobs({"environment": dict(environment, SINGLEFS_THREAD_CAP="abc")})
        unreadable_refused = False
    except HeavyThreadDefaultError:
        unreadable_refused = True
    selftest.check("先编那一份的并行度：没给 GATE_MUTATION_CARGO_JOBS 取每台默认重活线程数（SINGLEFS_THREAD_CAP=7 取 7、=11 取 11，不看本机核数），给了照它，取不到又没给就拒",
                   by_registry == ["7", "11"] and given == "3" and unreadable_refused,
                   f"没给时 {by_registry}（要 ['7', '11']）；给了 3 时 {given}；取不到时{'拒了' if unreadable_refused else '没拒'}")
    # 工作进程拷依赖产物的线程数（seed_copy_threads）：没给 CARGO_BUILD_JOBS 取每台默认重活线程数 ÷ 工作进程数（SINGLEFS_THREAD_CAP=12），不看本机核数
    copy_environment = {key: value for key, value in environment.items() if key != "CARGO_BUILD_JOBS"}
    divided = [seed_copy_threads({"environment": dict(copy_environment, SINGLEFS_THREAD_CAP="12"), "worker_count": count}) for count in (4, 1, 16)]
    copy_given = seed_copy_threads({"environment": dict(copy_environment, SINGLEFS_THREAD_CAP="12", CARGO_BUILD_JOBS="5"), "worker_count": 4})
    try:
        seed_copy_threads({"environment": dict(copy_environment, SINGLEFS_THREAD_CAP="abc"), "worker_count": 4})
        copy_unreadable_refused = False
    except HeavyThreadDefaultError:
        copy_unreadable_refused = True
    selftest.check("工作进程拷依赖产物的线程数：没给 CARGO_BUILD_JOBS 取每台默认重活线程数 ÷ 工作进程数、至少 1（SINGLEFS_THREAD_CAP=12：4 个工作进程取 3、1 个取 12、16 个取 1，不看本机核数），"
                   "给了照它，取不到又没给就拒",
                   divided == [3, 12, 1] and copy_given == 5 and copy_unreadable_refused,
                   f"没给时 4、1、16 个工作进程取 {divided}（要 [3, 12, 1]）；给了 5 时 {copy_given}；取不到时{'拒了' if copy_unreadable_refused else '没拒'}")


def build_seed_group(context, command, seed_source, extra_environment, temporary_root, instrumented):
    """先编一组：subprocess 的结果。TMPDIR 是这一组自己的，插桩那一份的构建脚本与过程宏的剖析文件也落在里面，编完整个删。"""
    temporary = tempfile.mkdtemp(prefix="seed-", dir=temporary_root)
    try:
        environment = dict(context["environment"], TMPDIR=temporary, **extra_environment)
        if instrumented:
            environment["LLVM_PROFILE_FILE"] = os.path.join(temporary, "build-%p-%m.profraw")
        return run_capturing_output(command, cwd=seed_source, env=environment)
    finally:
        shutil.rmtree(temporary, ignore_errors=True)


def prepare_seed(context, batches, ordered_rows, coverage_planner, temporary_root):
    """派活之前先编一份依赖：交 (要报的话, 判红的那几句)。编成的记进 context["seed_targets"]（{"plain" 或 "coverage": 编译目录}），
    工作进程起来时照它拷（copy_seed_dependencies）；没起、没编成的不在里面。弄坏开关 compile-per-worker：不起。"""
    context["seed_targets"] = {}
    environment = context["environment"]
    if break_is(environment, "compile-per-worker"):
        return ["  … 先编一份依赖：没起（弄坏开关 compile-per-worker：每个工作进程自己编外部依赖）"], []
    if context["worker_count"] < 2:
        return [f"  … 先编一份依赖：没起（这一趟只开 {context['worker_count']} 个工作进程，没有别的工作进程共用）"], []
    reaching = path_packages_reaching_external(context["root"])
    plain = seed_groups([row for _key, batch_rows in batches for row in batch_rows] + list(ordered_rows), reaching)
    coverage = seed_groups(coverage_planner.predicted_rows(ordered_rows), reaching)
    if not plain and not coverage:
        return ["  … 先编一份依赖：没起（要现跑的行选的包顺 Cargo.lock 都够不着外部依赖" + ("" if reaching else "；树里没有读得懂的 Cargo.lock") + "）"], []
    shared = os.path.normpath(context["shared_target_directory"])
    seed_source = shared + SEED_SOURCE_SUFFIX
    jobs = seed_jobs(context)
    lines, problems = [], []
    try:
        with seed_lock(shared, exclusive=True):
            touched, changed_count = sync_seed_source(context["root"], context["copied"], seed_source, environment)
            lines.append(f"  … 先编一份依赖：源码副本 {seed_source} 同步了 {changed_count} 个文件（摸了整包的 {len(touched)} 个"
                         + (f"：{'、'.join(sorted(touched))}" if touched else "") + f"）；并行度 {jobs}")
            overrides, configuration = coverage_build_settings(environment)
            for kind, groups, target, extra, words in (("plain", plain, shared + SEED_TARGET_SUFFIX, {}, []),
                                                       ("coverage", coverage, shared + SEED_TARGET_SUFFIX + COVERAGE_TARGET_SUFFIX, overrides, configuration)):
                if not groups:
                    continue
                failed = []
                compiled_units = set()
                for arguments in groups.values():
                    command = ["cargo", *words, "test", "--offline", *arguments, "--no-run", "--message-format=json-render-diagnostics"]
                    run = build_seed_group(context, command, seed_source, dict(extra, CARGO_TARGET_DIR=target, CARGO_BUILD_JOBS=jobs), temporary_root,
                                           kind == "coverage")
                    stale = [] if break_is(environment, "seedfreshunchecked") else stale_seed_artifacts(
                        run.stdout, seed_source, touched, None if break_is(environment, "seedfreshacrossgroups") else compiled_units)
                    problems += [f"{' '.join(command)}：{item} 是 fresh" for item in stale]
                    if run.returncode != 0:
                        failed.append(f"{' '.join(arguments)} 退 {run.returncode}：{output_tail(run.stderr, 3)}")
                with contextlib.suppress(OSError):
                    trim_target_directory(target, context.get("target_directory_cap_bytes"), environment)
                if failed:
                    lines.append(f"  ! 先编一份依赖（{'插桩' if kind == 'coverage' else '普通'}，{target}）{len(groups)} 组里 {len(failed)} 组没编成，"
                                 f"这一趟各工作进程自己编依赖：{'；'.join(failed[:3])}")
                else:
                    context["seed_targets"][kind] = target
                    lines.append(f"  … 先编一份依赖（{'插桩' if kind == 'coverage' else '普通'}）：{target} 编了 {len(groups)} 组，工作进程起来时拷其中外部依赖的产物")
    except OSError as error:
        # 同步源码、编、清增量产物时读写出错：这一趟不用先编那一份（工作进程照旧自己编依赖），判定不受影响
        context["seed_targets"] = {}
        lines.append(f"  ! 先编一份依赖没做成（{type(error).__name__}：{error}）：这一趟各工作进程自己编依赖")
    return lines, problems


def dependency_artifact_entries(seed_target, seed_source, environment):
    """先编那一份编译目录里外部依赖的产物：相对 seed_target 的路径（文件或目录）。每一档（底下有 .fingerprint 的那一层：<目录>/<档>、<目录>/<三元组>/<档>）里
    .fingerprint/、build/ 下包名不是 Cargo.lock 里 path 包的目录；deps/ 下按单元哈希归堆，那一堆恰好一份 .d、里面列的源文件都是绝对路径、都不在源码副本底下的那一堆。
    incremental/ 与档那一层的文件（cargo 拷上来的工作区产物）不拷。弄坏开关 workspaceunitsshared：.fingerprint/、build/、deps/ 底下的全拷。"""
    path_names = {package["name"] for package in read_lock_file_packages(seed_source) if "source" not in package}
    root = os.path.realpath(seed_source)
    everything = break_is(environment, "workspaceunitsshared")
    levels = []
    for name in sorted(os.listdir(seed_target)) if os.path.isdir(seed_target) else []:
        level = os.path.join(seed_target, name)
        if os.path.isdir(os.path.join(level, ".fingerprint")):
            levels.append(name)
        elif os.path.isdir(level):
            levels += [os.path.join(name, deeper) for deeper in sorted(os.listdir(level)) if os.path.isdir(os.path.join(level, deeper, ".fingerprint"))]
    entries = []
    for level in levels:
        for kind in (".fingerprint", "build"):
            base = os.path.join(seed_target, level, kind)
            for name in sorted(os.listdir(base)) if os.path.isdir(base) else []:
                matched = UNIT_DIRECTORY_NAME.match(name)
                if matched and (everything or matched.group("name") not in path_names):
                    entries.append(os.path.join(level, kind, name))
        deps = os.path.join(seed_target, level, "deps")
        units = collections.defaultdict(list)
        for name in sorted(os.listdir(deps)) if os.path.isdir(deps) else []:
            matched = UNIT_ARTIFACT_NAME.match(name)
            if matched:
                units[matched.group("hash")].append(name)
        for names in units.values():
            dep_infos = [name for name in names if name.endswith(".d")]
            sources = dependency_info_sources(os.path.join(deps, dep_infos[0])) if len(dep_infos) == 1 else []
            if everything or (sources and all(os.path.isabs(source) and not os.path.realpath(source).startswith(root + os.sep) for source in sources)):
                entries += [os.path.join(level, "deps", name) for name in names]
    return entries


def dependency_info_sources(path):
    """rustc 写的 .d 头一条规则冒号后面的源文件（反斜杠转义的空格还原）；读不了交空表。"""
    try:
        with open(path, encoding="utf-8", errors="replace") as handle:
            text = handle.read().replace("\\\n", " ")
    except OSError:
        return []
    _target, _colon, rest = text.split("\n", 1)[0].partition(": ")
    return [word.replace("\\ ", " ") for word in re.split(r"(?<!\\) ", rest) if word]


def sync_dependency_artifacts(seed_target, destination, seed_source, environment, threads):
    """把先编那一份里外部依赖的产物（dependency_artifact_entries）拷进 destination：大小与修改时刻都对得上的不拷，拷的保留修改时刻、同目录临时文件写完改名。
    要拷的文件分到 threads 条线程上同时拷（弄坏开关 seed-copy-serial：在一条线程上逐个拷）；哪一个拷不成，OSError 照抛给调用方。回 (拷了几个文件, 几字节)。"""
    pending = dependency_copy_jobs(seed_target, destination, seed_source, environment)
    copy_dependency_files(pending, threads, environment)
    return len(pending), sum(size for _path, _target, size in pending)


def copy_one_dependency_file(job):
    path, target, _size = job
    os.makedirs(os.path.dirname(target), exist_ok=True)
    temporary = target + SEED_COPY_TEMPORARY_SUFFIX
    shutil.copy2(path, temporary)
    os.replace(temporary, target)


def copy_dependency_files(pending, threads, environment):
    """pending 里的 (源, 目标, 字节) 逐个拷：threads 大于 1 时分到那么多条线程上同时拷，出错的头一个原样抛（并行不许把失败吃掉）。"""
    count = 1 if break_is(environment, "seed-copy-serial") else max(1, min(int(threads or 1), len(pending)))
    if count == 1:
        for job in pending:
            copy_one_dependency_file(job)
        return
    with concurrent.futures.ThreadPoolExecutor(max_workers=count) as pool:
        for future in [pool.submit(copy_one_dependency_file, job) for job in pending]:
            future.result()


def seed_copy_threads(context):
    """一个工作进程拷依赖产物用几条线程：CARGO_BUILD_JOBS（run_stage 按每台默认重活线程数 ÷ 工作进程数设的），读不出取每台默认重活线程数 ÷ 工作进程数、至少 1
    （compile_thread_budget 与 cargo_jobs_for，同 run_stage 那一份；取不到抛 HeavyThreadDefaultError，不回落到核数）。
    弄坏开关 seed-copy-threads-unregistered：读不出 CARGO_BUILD_JOBS 时固定 1 条线程，不读登记位、取不到也不拒。"""
    environment = context["environment"]
    given = str(environment.get("CARGO_BUILD_JOBS", "")).strip()
    if given.isdigit() and int(given) > 0:
        return int(given)
    if break_is(environment, "seed-copy-threads-unregistered"):
        return 1
    return cargo_jobs_for(environment, int(context["worker_count"]), compile_thread_budget(environment))


def dependency_copy_jobs(seed_target, destination, seed_source, environment):
    """先编那一份里外部依赖的产物里要拷进 destination 的文件：[(源, 目标, 字节)]；大小与修改时刻都对得上的不在里面。"""
    pending = []
    for relative in dependency_artifact_entries(seed_target, seed_source, environment):
        source = os.path.join(seed_target, relative)
        paths = ([os.path.join(directory, name) for directory, _subdirectories, names in os.walk(source) for name in names]
                 if os.path.isdir(source) else [source])
        for path in paths:
            status = os.lstat(path)
            if not stat.S_ISREG(status.st_mode):
                continue
            target = os.path.join(destination, os.path.relpath(path, seed_target))
            with contextlib.suppress(FileNotFoundError):
                existing = os.lstat(target)
                if existing.st_size == status.st_size and existing.st_mtime_ns == status.st_mtime_ns:
                    continue
            pending.append((path, target, status.st_size))
    return pending


def copy_seed_dependencies(context, target_directory):
    """工作进程起来时（init_worker）：先编成的那一份（普通、插桩）里外部依赖的产物拷进自己的两份编译目录，拿着先编那一份的共享锁拷。拷不成只报，自己编依赖。"""
    seeds = context.get("seed_targets") or {}
    if not seeds:
        return
    shared = os.path.normpath(context["shared_target_directory"])
    environment = context["environment"]
    threads = 1 if break_is(environment, "seed-copy-serial") else seed_copy_threads(context)
    reports, pending = [], []
    try:
        with seed_lock(shared, exclusive=False):
            # 普通、插桩两份要拷的文件先列全，再一起分到 threads 条线程上拷（文件头「先编一份依赖」的「拷」）
            for kind, destination in (("plain", target_directory), ("coverage", target_directory + COVERAGE_TARGET_SUFFIX)):
                if kind in seeds:
                    jobs = dependency_copy_jobs(seeds[kind], destination, shared + SEED_SOURCE_SUFFIX, environment)
                    pending += jobs
                    reports.append(f"{'插桩' if kind == 'coverage' else '普通'} {len(jobs)} 个文件 {format_size(sum(size for _p, _t, size in jobs))}")
            copy_dependency_files(pending, threads, environment)
    except OSError as error:
        note(f"  ! 工作进程 {os.getpid()} 从先编那一份拷依赖产物没拷成（{type(error).__name__}：{error}）：这一片自己编依赖")
        return
    note(f"  … 工作进程 {os.getpid()} 从先编那一份拷来外部依赖的产物：{'；'.join(reports)}（{threads} 条线程同时拷；大小与修改时刻对得上的不拷）")


def write_baseline_timings(rows, judged, seconds):
    """一块基线判完记下点名测试的用时（文件头「基线」的「没有用时的与长的测试各占一块」）：这一块只是一条测试（共用过滤词的行，baseline_test_groups 只有一组）、
    它的行基线绿了、用时表里还没有那一行的，照变异那一行同一张用时表、同一种写法（write_row_timing）记 seconds（这一块判完的秒数）。
    多条测试合在一块的、有过滤词缺的整批、基线不绿的、已有用时的不记（用时是整块的，记给每条测试会把短的记成长的；已有的是变异那一行记的，含编译改坏的那一份，更长）。
    交写不进去的那几句。弄坏开关 baseline-timing-unwritten：不记。"""
    context = WORKER_STATE["context"]
    groups = baseline_test_groups(rows)
    if break_is(context["environment"], "baseline-timing-unwritten") or groups is None or len(groups) != 1:
        return []
    green_lines = {line_number for line_number, problem, _tail in judged if problem is None}
    problems = []
    for row in rows:
        if row.line_number in green_lines and read_timing(context["timings_directory"], row_content_hash(row)) is None:
            problems += WORKER_STATE["write_timing"](row, {"seconds_running": f"{seconds:.3f}"}, context)   # init_worker 交的 write_row_timing
    return problems


def baseline_job(batch):
    """进程池里的一格基线：判完清一次普通编译目录（trim_worker_target），一条测试一块的记下用时（write_baseline_timings）。峰值记在这一批各行点名的包下（worker_peak_scope）。"""
    with worker_peak_scope([row.arguments for row in batch[1]]):
        started = time.monotonic()
        result = judge_batch(batch)
        result[1].extend(write_baseline_timings(batch[1], result[0], time.monotonic() - started))
        trim_worker_target(False)
    return result


def mutation_job(row):
    """进程池里的一格变异：判完清一次普通编译目录。峰值记在这一行点名的包下。"""
    with worker_peak_scope([row.arguments]):
        result = judge_row(row)
        trim_worker_target(False)
    return result


def coverage_job(job):
    """进程池里的一格覆盖：判完清一次插桩编译目录。峰值记在这一组首行点名的包下。"""
    with worker_peak_scope([job["arguments"]]):
        result = judge_coverage(job)
        trim_worker_target(True)
    return result


def new_shard_root_path():
    """$TMPDIR 下 SHARD_ROOT_PREFIX 起头加八位随机字符的路径，只起名、不建。tempfile.mkdtemp 是在 mkdir 里把目录建出来、返回之后名字才到调用方手里：
    TERM 落在这两步之间，收尾的 rmtree 不知道名字，空目录就漏在 $TMPDIR。这里名字先交到调用方、记进 shard_root，再由调用方 os.mkdir，
    收尾删的就是这个名字（名字在、目录不在也无妨）。"""
    alphabet = "abcdefghijklmnopqrstuvwxyz0123456789_"
    chooser = random.SystemRandom()
    return os.path.join(tempfile.gettempdir(), SHARD_ROOT_PREFIX + "".join(chooser.choice(alphabet) for _ in range(8)))


def pool_cell_runs_on_gpu(kind, payload, environment):
    """进程池里这一格是不是上卡的格（文件头「上卡的格一格接一格」）：起的 cargo test 是 gpu_test_run 认的那一种（开了上卡的特性、真跑测试）。
    基线的批照这一批的参数（baseline_arguments），变异照这一行的参数，记覆盖照这一组首行的参数。"""
    if kind == "baseline":
        arguments = baseline_arguments(*payload)
    elif kind == "mutation":
        arguments = payload.arguments.split()
    else:
        arguments = payload["arguments"].split()
    return gpu_test_run(["cargo", "test", "--offline"] + list(arguments), environment)


def card_hiding_environment(environment):
    """不上卡的格起 cargo 时交给它的环境（文件头「上卡的格一格接一格」的「不上卡的格看不到卡」）：environment 的一份拷贝，Vulkan 驱动清单指到这一格自己 TMPDIR 底下一份不存在的文件
    （VK_DRIVER_FILES、VK_ICD_FILENAMES），去掉追加的清单 VK_ADD_DRIVER_FILES，CUDA_VISIBLE_DEVICES=-1。"""
    missing_driver_list = os.path.join(environment.get("TMPDIR") or tempfile.gettempdir(), "no-vulkan-driver.json")
    hidden = dict(environment, VK_DRIVER_FILES=missing_driver_list, VK_ICD_FILENAMES=missing_driver_list, CUDA_VISIBLE_DEVICES="-1")
    hidden.pop("VK_ADD_DRIVER_FILES", None)
    return hidden


class WorkerJobFailed(RuntimeError):
    """工作进程里一格活抛了异常，pool_cell_job 在工作进程里接住、换成这个交回：只带两个字符串（原异常的类名带模块名、整份回溯文本）。
    原异常的类可以定义在按路径载入的模块里（scripts/prune-server 的 HostAddressUnavailable 那一种），那样的模块只在工作进程里载入过，父进程反序列化它抛的异常时
    ModuleNotFoundError、进程池整个坏掉，报成「有工作进程中途死了」、原因丢了；换成这个（本模块的类，同 pool_cell_job 一样父进程里按名字找得到）父进程一定解得开。"""

    def __init__(self, original_type, traceback_text):
        super().__init__(original_type, traceback_text)
        self.original_type = original_type
        self.traceback_text = traceback_text


def pool_cell_job(task):
    """进程池里一格活（文件头「上卡的格一格接一格」）：task = (种类, 那一格, 是不是上卡的格)。不上卡的格，这个工作进程交给判法的起进程引用（WORKER_STATE["launch"]）
    在这一格期间换成照 card_hiding_environment 藏卡的那一个（弄坏开关 plain-cells-cards-visible：不藏）；上卡的格原样起，环境里的 GPU_CARDS_BY_PRIORITY 有就原样交。
    引用判完还原。这一格抛的异常在这里接住、换成 WorkerJobFailed（带原类名与整份回溯）交回：父进程解不开原异常的类时进程池整个坏掉，丢了原因；
    父进程在 run_pool_reporting_dead_worker 里照原类名与回溯报红。弄坏开关 worker-failure-unconverted：原异常照原样往外抛（进程池按它的类 pickle）。"""
    kind, payload, on_card = task
    environment = WORKER_STATE["context"]["environment"]
    launch = WORKER_STATE["launch"]

    def launch_hiding_cards(command, **keywords):
        return launch(command, **dict(keywords, env=card_hiding_environment(keywords["env"])))

    if not on_card and not break_is(environment, "plain-cells-cards-visible"):
        WORKER_STATE["launch"] = launch_hiding_cards
    try:
        return {"baseline": baseline_job, "mutation": mutation_job, "coverage": coverage_job}[kind](payload)
    except Exception as error:
        if break_is(environment, "worker-failure-unconverted"):
            raise
        raise WorkerJobFailed(f"{type(error).__module__}.{type(error).__qualname__}", traceback.format_exc()) from None
    finally:
        WORKER_STATE["launch"] = launch


def run_pool(context, worker_count, batches, ordered_rows, baseline_ok_lines, stop_after, coverage_planner, item_progress=None):
    """一个进程池里先派基线、再派变异、最后派覆盖：一批基线判完，它 ok 的行进变异的队；已有基线 ok 的行一开始就在队里；
    一条变异判成抓到、它那一组还要记覆盖的（coverage_planner.job_after），那一组的覆盖进队；开跑时就要记的（coverage_planner.initial_jobs）一开始就在队里。
    同时在跑的至多 worker_count 格，空出一格补一格（谁先跑完谁再领），基线优先、变异按 ordered_rows 的次序、覆盖最后。
    context["worker_growth"]（run_stage 设，文件头「内存上限」的「跑着加开」）不是 None 时，判完一格按这一趟各工作进程到此刻实测峰值的最大重算能开几个，
    能多开就把进程池换成更大的一个（grown_worker_count、grow_pool）。
    回 dict：mutation（变异的判定）、mutation_info（{行号: {snapshot, test_full_name}}）、baseline（{行号: (问题或 None, 尾巴)}）、coverage（覆盖那几格的结果）、
    submitted、received、stopped、left_behind、record_problems，加开过的另有 worker_growth（收尾一行要接的话）。item_progress 是 ItemProgress（按项机读行，文件头同名一段）：
    基线不绿与变异判完的行在这里报结局。"""
    # 这一趟各工作进程到此刻实测峰值的最大（字节）：工作进程的采样线程判格期间往里写（sample_worker_peak），跑着加开读它
    context["worker_peak_seen"] = multiprocessing.Value("q", 0)
    sequence_counter = multiprocessing.Value("i", 0)
    # 被停的信号也要删掉 shard_root（$TMPDIR 下的 singlefs-mutation-replay-*）：先接住再建，建完之后收到的都走 finally；
    # 接信号、起名、建目录都在 try 里：哪一步之后收到都走 finally（信号处理还原、按已记下的名字删目录），名字先记进 shard_root 再 os.mkdir（new_shard_root_path）。
    # 只在主线程接得了。弄坏开关 shardrootkeptonsignal 下不接：收到就停，shard_root 留在 $TMPDIR；
    # 弄坏开关 shardrootnamedlate 下用 tempfile.mkdtemp 建（名字要等它返回才记得下）
    previous_handlers = {}
    shard_root = None
    # 上卡的格（文件头「上卡的格一格接一格」）：card_futures 是派出去还没交回结果的上卡的格
    card_futures = set()
    try:
        if threading.current_thread() is threading.main_thread() and not break_is(context["environment"], "shardrootkeptonsignal"):
            for signal_number in STOP_SIGNALS:
                previous_handlers[signal_number] = signal.signal(signal_number, raise_stop_requested)
        if break_is(context["environment"], "shardrootnamedlate"):
            shard_root = tempfile.mkdtemp(prefix=SHARD_ROOT_PREFIX)
        else:
            shard_root = new_shard_root_path()
            try:
                os.mkdir(shard_root, 0o700)
            except FileExistsError:
                shard_root = None   # 别人的同名目录：收尾不许删它
                raise
        per_mutation_temporary_root = os.path.join(shard_root, PER_MUTATION_TEMPORARY_DIRECTORY_NAME)
        os.mkdir(per_mutation_temporary_root)
        os.mkdir(target_over_cap_directory(shard_root))   # 停派目录（文件头「编译目录上限」）：工作进程清完仍超上限往里写
        # 这一趟自己的编译目录（文件头「编译目录」）：调用方没给 GATE_MUTATION_TARGET_DIR 就建在 <shard_root>/build 里，收尾（finally）删
        if context["shared_target_directory"] is None:
            build_root = os.path.join(shard_root, RUN_BUILD_DIRECTORY_NAME)
            os.mkdir(build_root)
            context["shared_target_directory"] = run_build_target_directory(shard_root)
            for line in target_reclaim_lines(context["shared_target_directory"], worker_count, ([], [], []), owned=True):
                note(line)
        # 先编一份依赖（文件头「先编一份依赖」）：工作进程起来之前编好，编成的由 init_worker 拷进各自的编译目录；摸过的包没重编整道判红、一个工作进程都不起
        # 底座读数（文件头「内存上限」的「底座读数」）：父进程自己此刻的用量，与先编一份依赖那一步（工作进程还没起）父进程整棵树的峰值，口径同每个工作进程的读数
        base_parent_bytes = process_usage_bytes(os.getpid())
        seed_tree_peak, seed_sampling_done = [None], threading.Event()

        def sample_seed_step_tree():
            while True:
                usage = process_tree_usage(os.getpid(), context["environment"])
                if usage is not None:
                    seed_tree_peak[0] = max(seed_tree_peak[0] or 0, usage)
                if seed_sampling_done.wait(WORKER_PEAK_SAMPLE_SECONDS):
                    return
        seed_sampler = threading.Thread(target=sample_seed_step_tree, daemon=True)
        seed_sampler.start()
        try:
            seed_lines, seed_problems = prepare_seed(context, batches, ordered_rows, coverage_planner, shard_root)
        finally:
            seed_sampling_done.set()
            seed_sampler.join(timeout=5)
        for line in seed_lines:
            note(line)
        note(base_reading_line(base_parent_bytes, seed_tree_peak[0], bool(context["seed_targets"])))
        if seed_problems:
            return dict(mutation=[], mutation_info={}, baseline={}, coverage=[], submitted=0, received=0, stopped=False, left_behind=[], record_problems=[],
                        left_behind_contents={}, per_mutation_temporary_root=per_mutation_temporary_root, seed_problems=seed_problems)
        # 变异复跑各起的临时 prune-server 用哪一份二进制（文件头「变异复跑的崩溃判定」）：工作进程起来之前定下，交给它们；取不到一个工作进程都不起
        context["prune_server_binary"], prune_server_problem = mutation_run_prune_server_binary(context["environment"], shard_root)
        if prune_server_problem:
            return dict(mutation=[], mutation_info={}, baseline={}, coverage=[], submitted=0, received=0, stopped=False, left_behind=[], record_problems=[],
                        left_behind_contents={}, per_mutation_temporary_root=per_mutation_temporary_root, prune_server_problem=prune_server_problem)
        order_index = {row.line_number: index for index, row in enumerate(ordered_rows)}
        rows_by_line = {row.line_number: row for row in ordered_rows}
        # 上卡的格（文件头「上卡的格一格接一格」）进 card_ready，不上卡的进 ready：ready 派完、收完，进程池里一格都不在跑了，才从 card_ready 一次派一格。
        #   弄坏开关 gpu-cells-dispatched-in-parallel：上卡的格也进 ready，照空位并行派
        ready, card_ready, serial = [], [], 0
        gpu_cells_in_parallel = break_is(context["environment"], "gpu-cells-dispatched-in-parallel")

        def queue_entry(entry):
            on_card = not gpu_cells_in_parallel and pool_cell_runs_on_gpu(entry[3], entry[4], context["environment"])
            heapq.heappush(card_ready if on_card else ready, entry)
        for index, batch in enumerate(batches):
            queue_entry((0, index, serial, "baseline", batch))
            serial += 1
        for row in ordered_rows:
            if row.line_number in baseline_ok_lines:
                queue_entry((1, order_index[row.line_number], serial, "mutation", row))
                serial += 1
        for job in coverage_planner.initial_jobs():
            queue_entry((2, serial, serial, "coverage", job))
            serial += 1
        if card_ready:
            given_cards = context["environment"].get("GPU_CARDS_BY_PRIORITY", "")
            note(f"  … 上卡的格：{len(card_ready)} 格（基线的批、变异与覆盖合计）挪到最后，不上卡的都收回之后一次一格；不领卡，"
                 + (f"环境里的 GPU_CARDS_BY_PRIORITY={given_cards} 原样交" if given_cards else "环境里没有 GPU_CARDS_BY_PRIORITY、不设"))
        outcome = dict(mutation=[], mutation_info={}, baseline={}, coverage=[], submitted=0, received=0, stopped=False, left_behind=[], record_problems=[])
        in_flight, in_flight_packages = {}, {}
        baseline_one_at_a_time = break_is(context["environment"], "baseline-one-at-a-time")
        # 探针（文件头「内存上限」的「探针」）：在跑的格连同要派的这一格按实测峰值放得下才派，没有实测的包的格单独跑、量到之后放开
        admission = new_memory_admission(context)

        def collect(future):
            """收一格交回的结果：照种类记进 outcome，基线绿的行排进变异、抓到的那一组要记覆盖的排进覆盖。"""
            nonlocal serial
            kind = in_flight.pop(future)
            in_flight_packages.pop(future)
            card_futures.discard(future)
            result, record_problems = future.result()   # 工作进程里抛出来的异常（pool_cell_job 换成的 WorkerJobFailed）在这里炸出去、run_pool_reporting_dead_worker 报红：并行不许把失败吃掉
            outcome["received"] += 1
            outcome["record_problems"] += record_problems
            if kind == "baseline":
                for line_number, problem, tail in result:
                    outcome["baseline"][line_number] = (problem, tail)
                    if problem is not None and item_progress is not None:
                        item_progress.settle(line_number, "baseline")
                    if problem is None and line_number in order_index:
                        queue_entry((1, order_index[line_number], serial, "mutation", rows_by_line[line_number]))
                        serial += 1
            elif kind == "mutation":
                judgement, info = result
                outcome["mutation"].append(judgement)
                outcome["mutation_info"][judgement[0]] = info
                if item_progress is not None:
                    item_progress.settle(judgement[0], judgement[1], seconds=info.get("wall_seconds", 0.0))
                job = coverage_planner.job_after(rows_by_line[judgement[0]], judgement, info)
                if job is not None:
                    queue_entry((2, serial, serial, "coverage", job))
                    serial += 1
                if stop_after is not None and len(outcome["mutation"]) >= stop_after:
                    outcome["stopped"] = True
            else:
                outcome["coverage"].append(result)
            note(f"  … 已判 {outcome['received']} 格、派出 {outcome['submitted']} 格、还排着 {len(ready) + len(card_ready)} 格（基线的批、变异与覆盖合计；"
                 f"其中上卡的 {len(card_ready)} 格；基线不绿的行不派变异）")

        # 跑着加开（文件头「内存上限」的「跑着加开」）：growth 是 run_stage 交的上界与上限，pool_size 是此刻的进程池开几个，grow_to 是定了要换成几个（在跑的收完再换）
        growth, pool_size, grow_to, grown = context.get("worker_growth"), worker_count, None, False

        class PoolInUse:
            """own_workers_stopped_on_stop 停的那一个进程池：_processes 取此刻在用的那一个池的（跑着加开换过池之后是新的那一个，旧的已收工）。"""
            current = None

            @property
            def _processes(self):
                return getattr(self.current, "_processes", None)
        in_use = PoolInUse()

        def new_pool(count):
            in_use.current = pool_stack.enter_context(concurrent.futures.ProcessPoolExecutor(max_workers=count, initializer=init_worker,
                                                                                             initargs=(sequence_counter, shard_root, context)))
            return in_use.current

        def grow_pool(old_pool, count):
            """跑着加开：在跑的都收完了才换（旧池收工、它的管理线程收掉之后再 fork 新池的工作进程）。编译并行度照新的个数重取（cargo_jobs_for），不高过此刻那一档：
            量到的峰值是此刻那一档下的，编译并行度越低峰值不会越高（推的）。新池的工作进程领片号照旧从 1 往上领，前面那几片的编译目录接着用。
            探针改按这一趟到此刻的实测峰值收（memory_admits 的 peak_seen）。交 (新池, 换之前的编译并行度, 换之后的)。"""
            old_pool.shutdown(wait=True)
            environment = context["environment"]
            old_jobs = int(environment["CARGO_BUILD_JOBS"])
            jobs = min(old_jobs, cargo_jobs_for(environment, count, growth["thread_budget"]))
            environment["CARGO_BUILD_JOBS"] = str(jobs)
            context["worker_count"] = count
            if admission is not None:
                admission.update(jobs=jobs, peak_seen=context["worker_peak_seen"],
                                 unmeasured=set(unmeasured_packages_at(admission["peaks"], admission["packages"], jobs)))
            return new_pool(count), old_jobs, jobs

        with contextlib.ExitStack() as pool_stack, own_workers_stopped_on_stop(in_use, shard_root, context["environment"]):
            pool = new_pool(worker_count)
            while True:
                if grow_to is not None and not in_flight and not outcome["stopped"]:
                    pool, old_jobs, jobs = grow_pool(pool, grow_to)
                    outcome["worker_growth"] = (f"跑着加开：{growth_text}，判完 {outcome['received']} 格时从 {pool_size} 个工作进程加到 {grow_to} 个"
                                                f"（每个 cargo 编译并行度 {old_jobs} → {jobs}）")
                    note(f"  … {outcome['worker_growth']}")
                    pool_size, grow_to, grown = grow_to, None, True
                while len(in_flight) < pool_size and not outcome["stopped"] and grow_to is None:
                    queue = ready
                    if not ready:
                        # 上卡的格（文件头「上卡的格一格接一格」）：不上卡的都派完、收完才轮到上卡的，一次一格
                        if not card_ready or in_flight:
                            break
                        queue = card_ready
                    entry = heapq.heappop(queue)
                    if baseline_one_at_a_time and entry[3] == "baseline" and "baseline" in in_flight.values():
                        # 弄坏开关（只给证红用）：基线的批一次只派一批，在跑的那一批收完才派下一批
                        heapq.heappush(queue, entry)
                        break
                    packages = cell_peak_packages(entry[3], entry[4])
                    running = list(in_flight_packages.values())
                    if not memory_admits(admission, running, packages, context["environment"]):
                        # 放不下再多这一格：放回去，在跑的收完再派（排在前头的这一格不让给后面的）
                        heapq.heappush(queue, entry)
                        if admission["held"] != entry[2]:
                            admission["held"] = entry[2]
                            unmeasured = unmeasured_packages_at(admission["peaks"], tuple(sorted(set(packages).union(*running))), admission["jobs"])
                            note(f"  … 内存上限放不下再多 1 格（点名包 {'、'.join(map(peak_key_label, packages))}，在跑 {len(running)} 格）：在跑的收完再派"
                                 + (f"；包 {'、'.join(map(peak_key_label, unmeasured))} 在编译并行度 {admission['jobs']} 下还没有实测，单独跑一格量峰值（探针）" if unmeasured else ""))
                        break
                    future = pool.submit(pool_cell_job, (entry[3], entry[4], pool_cell_runs_on_gpu(entry[3], entry[4], context["environment"])))
                    if queue is card_ready:
                        card_futures.add(future)
                    in_flight[future] = entry[3]
                    in_flight_packages[future] = packages
                    outcome["submitted"] += 1
                    if entry[3] == "baseline":
                        for notes_name, kind_text in (("unknown_baseline_notes", "没有用时的测试"), ("long_baseline_notes", "长测试")):
                            if entry[4][1][0].line_number in context.get(notes_name, {}):
                                note(f"  … 基线{kind_text}单列一块：{context[notes_name][entry[4][1][0].line_number]}")
                if not in_flight:
                    break
                done, _pending = concurrent.futures.wait(list(in_flight), return_when=concurrent.futures.FIRST_COMPLETED)
                for future in done:
                    collect(future)
                # 探针：还有没量到的包时重读峰值记录，刚收完的那几格量到的读数在里面，量到了就放开（文件头「内存上限」的「探针」）
                for line in refresh_memory_admission(admission, context["environment"]):
                    note(line)
                # 停派（文件头「编译目录上限」）：有工作进程清完仍超上限，在跑的收完就不再派活
                over_cap = target_over_cap_reports(shard_root)
                if over_cap and not outcome.get("over_cap"):
                    outcome["stopped"], outcome["over_cap"] = True, over_cap
                    note(f"  ! 编译目录清完仍超上限，停派：在跑的 {len(in_flight)} 格收完就不再派活（还排着 {len(ready) + len(card_ready)} 格；收尾判红，出路在那里）")
                # 跑着加开（文件头「内存上限」的「跑着加开」）：判完一格、还有不上卡的格排着、没有上卡的格在跑时，按这一趟到此刻实测峰值的最大重算能开几个；
                #   多得出来就定下要换成几个，停派、在跑的收完再换池（上面 grow_pool）。只加一次：实测峰值的最大只涨不跌，之后算出来的不会更多
                if (growth is not None and grow_to is None and not grown and done and ready and not card_futures and not outcome["stopped"]):
                    count, growth_text = grown_worker_count(growth, pool_size, context["worker_peak_seen"].value)
                    if count is not None:
                        grow_to = count
                        note(f"  … 跑着加开：{growth_text}，定下从 {pool_size} 个工作进程加到 {count} 个；在跑的 {len(in_flight)} 格收完就换进程池")
        outcome["left_behind"] = sorted(os.listdir(per_mutation_temporary_root))
        outcome["left_behind_contents"] = {name: (sorted(os.listdir(os.path.join(per_mutation_temporary_root, name)))
                                                  if os.path.isdir(os.path.join(per_mutation_temporary_root, name)) else ["（不是目录）"])
                                           for name in outcome["left_behind"]}
        outcome["per_mutation_temporary_root"] = per_mutation_temporary_root
        return outcome
    finally:
        # 先删这一趟的编译目录（remove_owned_build_root：核过没有进程开着里面的文件），没删成就连同 shard_root 留着（run_stage 判红）；
        # 被停时 own_workers_stopped_on_stop 已经删过，这里看到不在就照删 shard_root。弄坏开关 runtargetkept：两样都不删
        if shard_root is not None and os.path.isdir(shard_root):
            if remove_owned_build_root(shard_root, context["environment"]):
                shutil.rmtree(shard_root, ignore_errors=True)
            else:
                note(f"  · 这一趟的临时目录 {shard_root} 留着（里面的编译目录没删）")
        for signal_number, handler in previous_handlers.items():
            signal.signal(signal_number, handler)


def broken_pool_error_type(environment):
    """「有工作进程中途死了」认的异常类：模块载入时绑定的 BrokenProcessPool。concurrent.futures 不自己载入 process 子模块，进程池还没起（先编一份依赖那一段）
    就被停时按 concurrent.futures.process 属性找它，判 except 的那一刻抛 AttributeError、盖掉原来的 StopRequested（被停退 143 成了回溯退 1）。
    弄坏开关 broken-pool-looked-up-late：在判 except 的时候按属性找。"""
    if break_is(environment, "broken-pool-looked-up-late"):
        return concurrent.futures.process.BrokenProcessPool
    return BrokenProcessPool


def run_pool_reporting_dead_worker(context, worker_count, batches, ordered_rows, baseline_ok_lines, stop_after, coverage_planner, item_progress=None, pool_runner=None):
    """run_pool 的结果；有工作进程被硬杀（段错误、被内核杀）时打「有工作进程中途死了」、有 __cause__ 的接着打它的文本，交 None；
    工作进程里一格活抛了异常（pool_cell_job 换成 WorkerJobFailed 交回的）时打原类名与整份回溯、交 None；别的异常（被停的 StopRequested 这类）原样往外抛。
    pool_runner：真正跑进程池的可调用对象，参数与 run_pool 一样；不给就在调用的那一刻取本模块的 run_pool（自证格交假的进去，不改模块全局）。
    弄坏开关 dead-worker-cause-unprinted：「有工作进程中途死了」后面不接 __cause__ 的文本。"""
    runner = run_pool if pool_runner is None else pool_runner
    try:
        return runner(context, worker_count, batches, ordered_rows, baseline_ok_lines, stop_after, coverage_planner, item_progress)
    except WorkerJobFailed as failure:
        print_rejection(f"工作进程里一格活抛了 {failure.original_type}，这一轮的变异没跑全（回溯是工作进程里的原样）", failure.traceback_text.rstrip().splitlines(),
                        ["照上面回溯最后一行的异常去修；判完的那几条有判定库记录，下一趟只跑剩下的。"])
        return None
    except broken_pool_error_type(context["environment"]) as dead:
        # 一个工作进程被硬杀时进程池当场抛这个，而不是安静地少产出几项；进程池有原因（__cause__：父进程读工作进程交回的结果时出的错）的接着打
        cause = None if break_is(context["environment"], "dead-worker-cause-unprinted") else dead.__cause__
        print_rejection("有工作进程中途死了，这一轮的变异没跑全",
                        [] if cause is None else ["进程池报的原因（__cause__）：", *str(cause).rstrip().splitlines()],
                        ["整条门禁撞了统一任务服务给的内存上限时由服务放大重排，不在这里调；别的死法（段错误）单跑 GATE_MUTATION_WORKERS=1 看是不是并发度的问题，",
                         "还死就去看那一条变异本身（判完的那几条有判定库记录，下一趟只跑剩下的）。"])
        return None


def selftest_stop_before_pool_cell(selftest, work):
    """进程池还没起（先编一份依赖那一段）就被 TERM 停：concurrent.futures 的 process 子模块此刻没载入，「有工作进程中途死了」那一句的 except 照样认出
    这是 StopRequested、原样往外抛，退 143，不在判 except 的时候抛 AttributeError（灰度那一趟的样子：被停成了回溯退 1）。
    弄坏开关 broken-pool-looked-up-late（判 except 的时候按 concurrent.futures.process 属性找）下这一格必须判错。"""
    child = subprocess.run([sys.executable, "-c", POOL_ERROR_CHILD_PROGRAM, MODULE_PATH], capture_output=True, text=True, errors="replace",
                           env=dict(selftest.base_environment), stdin=subprocess.DEVNULL)
    selftest.check("进程池还没起就被 TERM 停（process 子模块没载入）：StopRequested 原样往外抛、退 143，不抛 AttributeError",
                   child.returncode == 128 + signal.SIGTERM and "AttributeError" not in child.stderr,
                   f"退 {child.returncode}；stderr 尾：{child.stderr.strip()[-300:]}")


def selftest_worker_failure_report_cell(selftest, work):
    """工作进程里的失败父进程照原样报红、不丢原因（pool_cell_job、run_pool_reporting_dead_worker、Selftest.pool_process_variables）。三格：
    ① 调用方的 SINGLEFS_MULTI_HOST_CONFIG（这一格里指到一份不在的文件）进了工作进程的进程环境：变异格在工作进程里起临时 prune-server，TestServer 照它认这一台的地址，
       读不到抛 HostAddressUnavailable（定义在按路径载入、只在工作进程里载入过的模块里，父进程里没有这个模块）。一趟 run_stage 退 1，打原类名与整份回溯（回溯里点名那份不在的配置），
       不报「有工作进程中途死了」。弄坏开关 pool-worker-caller-configuration-dropped 下工作进程认不到它、退回本模块所在仓根那一份，判错；
       弄坏开关 worker-failure-unconverted 下原异常照原样往外抛，父进程解不开它的类、进程池整个坏掉，判错。
    ② 进程池坏了、有 __cause__（交给 run_pool_reporting_dead_worker 的 pool_runner 是抛 BrokenProcessPool from <原因> 的）：打「有工作进程中途死了」，接着打原因的文本。弄坏开关 dead-worker-cause-unprinted 下判错。
    ③ 进程池坏了、没有 __cause__（被信号杀的样子）：只打「有工作进程中途死了」，不打原因那一行。"""
    repository = os.path.join(work, "worker-failure")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    absent = os.path.join(work, "caller-multi-host", "absent.env")
    saved = selftest.caller_multi_host_configuration
    selftest.caller_multi_host_configuration = absent   # 调用方指到一份不在的多机配置：工作进程认这一台的地址就读不到
    try:
        exit_code, stdout, stderr, _invocations = selftest.run(repository)
    finally:
        selftest.caller_multi_host_configuration = saved
    text = stdout + stderr
    selftest.check("调用方的 SINGLEFS_MULTI_HOST_CONFIG 进了工作进程的环境：工作进程里起临时 prune-server 认地址读它，读不到时一趟退 1、打原类名（HostAddressUnavailable）与整份回溯"
                   "（回溯里点名那份不在的配置），不报「有工作进程中途死了」",
                   exit_code == 1 and "HostAddressUnavailable" in text and "Traceback (most recent call last)" in text and absent in text and "有工作进程中途死了" not in text,
                   f"退 {exit_code}；{text[-1200:]}")
    dead_text = "A process in the process pool was terminated abruptly while the future was running or pending."

    def dead_report(raiser):
        """把 raiser 当 pool_runner 交给 run_pool_reporting_dead_worker：(交回值, 打出来的字)。"""
        printed = io.StringIO()
        with contextlib.redirect_stdout(printed), contextlib.redirect_stderr(printed):
            returned = run_pool_reporting_dead_worker(dict(environment=dict(selftest.base_environment)), 1, [], [], set(), None, None, pool_runner=raiser)
        return returned, printed.getvalue()

    def dead_with_cause(*arguments, **keywords):
        raise BrokenProcessPool(dead_text) from RuntimeError("子进程交回的结果解不开：自证样本的原因")

    def dead_without_cause(*arguments, **keywords):
        raise BrokenProcessPool(dead_text)
    with_cause = dead_report(dead_with_cause)
    without_cause = dead_report(dead_without_cause)
    selftest.check("进程池坏了且有 __cause__：打「有工作进程中途死了」、接着打原因的文本，交 None",
                   with_cause[0] is None and "有工作进程中途死了" in with_cause[1] and "自证样本的原因" in with_cause[1], f"交回 {with_cause[0]}；{with_cause[1][-600:]}")
    selftest.check("进程池坏了、没有 __cause__（被信号杀）：只打「有工作进程中途死了」，不打原因那一行，交 None",
                   without_cause[0] is None and "有工作进程中途死了" in without_cause[1] and "进程池报的原因" not in without_cause[1], f"交回 {without_cause[0]}；{without_cause[1][-600:]}")


def build_seed_repository(directory):
    """先编一份依赖那几格的小仓：照 build_selftest_repository，另放一份 Cargo.lock，tiny 依赖一个带 source 的外部包 fakedep（假 cargo 照它「编」依赖）。"""
    build_selftest_repository(directory, SELFTEST_TABLE_ROWS)
    selftest_write(os.path.join(directory, "Cargo.lock"), SELFTEST_EXTERNAL_LOCK)


def with_break(selftest, switch_name):
    """在调用方给的弄坏开关之外再打开 switch_name（这一格自己造坏输入用）：交 GATE_MUTATION_BREAK 的新值。"""
    given = [item for variable in BREAK_VARIABLES for item in selftest.base_environment.get(variable, "").split(",") if item]
    return ",".join(given + [switch_name])


def selftest_seed_cells(selftest, work):
    """先编一份依赖（文件头「先编一份依赖」）：小仓的 tiny 依赖外部包 fakedep，假 cargo 在缺 fakedep 产物的编译目录里「编」一遍并记一行。
    4 个工作进程（假 cargo 的屏障逼两个工作进程同时在编）：外部依赖只在 <共用目录>-seed 与 -seed-coverage 里各编一次，工作进程的编译目录一次都没编、产物是拷来的；
    判定与 1 个工作进程（不先编）那一趟逐字相同。直接调 dependency_artifact_entries：只交外部依赖的产物，工作区的包、incremental、档那一层的文件不交；拷两遍，第二遍一个都不拷。
    源码副本按内容同步：改了 tiny 的 lib.rs 之后只同步那 1 个文件、摸 crates/tiny，先编那一趟 tiny 重编、整道照常；修改时刻被留住（seedmtimekept）时先编那一趟判红、一条都不跑。
    先编有两组（一行带 --release）：后一组复用前一组编出来的 tiny、假 cargo 报 fresh，整道照常（seedfreshacrossgroups 打开时判红）；修改时刻被留住时两组都是头一次出现就 fresh，照判红。
    直接调 stale_seed_artifacts：前一组非 fresh、后一组同一个单元 fresh 不算坏；同包另一套特性的单元（身份不同）fresh、头一次出现就 fresh 的，照算坏。
    弄坏开关 compile-per-worker、workspaceunitsshared、seedmtimekept、seedfreshunchecked、seedfreshacrossgroups 各让一格判错。"""
    repository = os.path.join(work, "seed")
    build_seed_repository(repository)
    target = os.path.join(work, "seed-target")
    dependency_log = os.path.join(work, "seed-dependencies.log")
    environment_log = os.path.join(work, "seed-environment.jsonl")
    seeded = selftest.run(repository, GATE_MUTATION_WORKERS="4", GATE_MUTATION_TARGET_DIR=target, FAKE_CARGO_DEPENDENCY_LOG=dependency_log,
                          FAKE_CARGO_ENVIRONMENT_LOG=environment_log, FAKE_CARGO_BARRIER_DIR=os.path.join(work, "seed-barrier"), FAKE_CARGO_BARRIER_PRISTINE=source_hash(repository))
    compiled = open(dependency_log, encoding="utf-8").read().splitlines() if os.path.exists(dependency_log) else []
    seeds = sorted([target + SEED_TARGET_SUFFIX, target + SEED_TARGET_SUFFIX + COVERAGE_TARGET_SUFFIX])
    worker_targets = {record.get("CARGO_TARGET_DIR") for record in environment_records(environment_log)} - set(seeds)
    plain_worker_targets = {path for path in worker_targets if not path.endswith(COVERAGE_TARGET_SUFFIX)}
    copied_into = [path for path in sorted(worker_targets) if os.path.isfile(os.path.join(path, "debug", "deps", "libfakedep-"
                   + hashlib.sha256(b"fakedep1.2.3").hexdigest()[:16] + ".rlib"))]
    selftest.check("4 个工作进程：外部依赖只在先编那一份里各编一次（普通、插桩各 1 次），起过 cargo 的工作进程编译目录里一次都没编、依赖产物都是拷来的",
                   seeded[0] == 0 and sorted(compiled) == seeds and len(plain_worker_targets) >= 2 and copied_into == sorted(worker_targets),
                   f"退 {seeded[0]}；编过依赖的编译目录 {compiled}（要 {seeds}）；起过 cargo 的工作进程编译目录 {sorted(worker_targets)}，有拷来的 {copied_into}；{seeded[2][-600:]}")
    reference = selftest.run(selftest_new_library_copy(work, repository, "seed-reference"), GATE_MUTATION_WORKERS="1",
                             GATE_MUTATION_TARGET_DIR=os.path.join(work, "seed-reference-target"))
    selftest.check("先编一份依赖不改判定：4 个工作进程先编那一趟与 1 个工作进程（不先编）那一趟判定逐字相同",
                   seeded[0] == 0 and reference[0] == 0 and selftest_verdict_view(seeded[1]) == selftest_verdict_view(reference[1])
                   and "先编一份依赖：没起（这一趟只开 1 个工作进程" in reference[2], f"{seeded[1]}\n——\n{reference[1]}\n{reference[2][-300:]}")
    # 直接调：一份造出来的先编编译目录，外部依赖、工作区的包（.d 里是副本底下的绝对路径与相对路径两种）、incremental、档那一层的文件、三元组那一层各一样
    source = os.path.join(work, "classify-source")
    selftest_write(os.path.join(source, "Cargo.lock"), SELFTEST_EXTERNAL_LOCK)
    seed_target = os.path.join(work, "classify-target")
    external, workspace, relative, build, triple = "0123456789abcdef", "fedcba9876543210", "1111111111111111", "00112233445566ff", "abcdefabcdefabcd"
    layout = {f"debug/deps/libfakedep-{external}.rlib": "rlib", f"debug/deps/fakedep-{external}.d": f"x: /registry/fakedep-1.2.3/src/lib.rs\n",
              f"debug/deps/libtiny-{workspace}.rlib": "rlib", f"debug/deps/tiny-{workspace}.d": f"x: {source}/crates/tiny/src/lib.rs\n",
              f"debug/deps/smoke-{relative}": "exe", f"debug/deps/smoke-{relative}.d": "x: crates/tiny/tests/smoke.rs\n",
              f"debug/.fingerprint/fakedep-{external}/lib-fakedep": "f", f"debug/.fingerprint/tiny-{workspace}/lib-tiny": "f",
              f"debug/build/fakedep-{build}/out/generated.rs": "g", f"debug/build/tiny-{build}/out/generated.rs": "g",
              "debug/incremental/tiny-xyz/s/dep-graph.bin": "i", "debug/libtiny.rlib": "uplifted", "debug/.cargo-lock": "",
              f"x86_64-unknown-linux-gnu/release/.fingerprint/fakedep-{triple}/lib-fakedep": "f"}
    for path, text in layout.items():
        selftest_write(os.path.join(seed_target, path), text)
    entries = sorted(dependency_artifact_entries(seed_target, source, dict(selftest.base_environment)))
    wanted_entries = sorted([f"debug/.fingerprint/fakedep-{external}", f"debug/build/fakedep-{build}", f"debug/deps/fakedep-{external}.d",
                             f"debug/deps/libfakedep-{external}.rlib", f"x86_64-unknown-linux-gnu/release/.fingerprint/fakedep-{triple}"])
    destination = os.path.join(work, "classify-destination")
    first_copy = sync_dependency_artifacts(seed_target, destination, source, dict(selftest.base_environment), 1)
    second_copy = sync_dependency_artifacts(seed_target, destination, source, dict(selftest.base_environment), 1)
    kept_time = os.stat(os.path.join(destination, f"debug/deps/libfakedep-{external}.rlib")).st_mtime_ns == os.stat(os.path.join(seed_target, f"debug/deps/libfakedep-{external}.rlib")).st_mtime_ns
    selftest.check("先编那一份只拷外部依赖的产物：.fingerprint、build、deps 里 fakedep 的 5 项（含三元组那一层），工作区的包、incremental、档那一层的文件不拷；头一遍拷 5 个文件、保留修改时刻，第二遍一个都不拷",
                   entries == wanted_entries and first_copy[0] == 5 and second_copy == (0, 0) and kept_time
                   and not os.path.exists(os.path.join(destination, f"debug/deps/libtiny-{workspace}.rlib")),
                   f"交的 {entries}（要 {wanted_entries}）；头一遍 {first_copy}、第二遍 {second_copy}、修改时刻保留 {kept_time}")
    selftest_seed_copy_threads_cell(selftest, work, seed_target, source)
    # 源码副本按内容同步：头一趟整份进来；改了 tiny 的 lib.rs 之后只同步 1 个文件、摸 crates/tiny，先编那一趟 tiny 重编
    fresh_repository = os.path.join(work, "seed-fresh")
    build_seed_repository(fresh_repository)
    fresh_target = os.path.join(work, "seed-fresh-target")
    first = selftest.run(fresh_repository, GATE_MUTATION_WORKERS="2", GATE_MUTATION_TARGET_DIR=fresh_target)
    replace_in_file(os.path.join(fresh_repository, "crates", "tiny", "src", "lib.rs"), "// 本模块自证的样本 crate", "// 本模块自证的样本 crate（改过一次）")
    # 只改了注释，单元键不变：同一棵树再跑一趟全复用、不开工作进程，所以改过的这一份换一棵树（库是新的一份）再跑
    second = selftest.run(selftest_new_library_copy(work, fresh_repository, "seed-fresh-edited-once"), GATE_MUTATION_WORKERS="2", GATE_MUTATION_TARGET_DIR=fresh_target)
    selftest.check("先编那一份的源码按内容同步：改了 tiny 的 lib.rs 之后只同步那 1 个文件、摸 crates/tiny，先编那一趟 tiny 重编、整道照常",
                   first[0] == 0 and second[0] == 0 and "同步了 1 个文件（摸了整包的 1 个：crates/tiny）" in second[2]
                   and "先编一份依赖（普通）：" in second[2], f"头一趟退 {first[0]}；改过之后退 {second[0]}：{(second[1] + second[2])[-900:]}")
    replace_in_file(os.path.join(fresh_repository, "crates", "tiny", "src", "lib.rs"), "（改过一次）", "（改过两次）")
    kept = selftest.run(selftest_new_library_copy(work, fresh_repository, "seed-fresh-edited-twice"), GATE_MUTATION_WORKERS="2", GATE_MUTATION_TARGET_DIR=fresh_target,
                        GATE_MUTATION_BREAK=with_break(selftest, "seedmtimekept"))
    selftest.check("先编那一份的修改时刻被留住（seedmtimekept）：先编那一趟判红、点名 crates/tiny 是 fresh，一条都不跑",
                   kept[0] == 1 and "crates/tiny 的 tiny 是 fresh" in kept[1] + kept[2] and plain_invocations(kept[3]) == [],
                   f"退 {kept[0]}；{(kept[1] + kept[2])[-900:]}；假 cargo {kept[3]}")
    selftest_seed_group_cells(selftest, work)


def selftest_seed_copy_threads_cell(selftest, work, seed_target, source):
    """工作进程拷依赖产物分到几条线程上同时拷（文件头「先编一份依赖」的「拷」）：直接调 sync_dependency_artifacts（threads=4），
    shutil.copy2 换成先到一道两方屏障（5 秒）再拷的：头两个文件要在两条线程上同时拷到屏障才过得去，fakedep 的 rlib 拷到了（拷几个由「只拷外部依赖的产物」那一格管）。弄坏开关 seed-copy-serial 让它判错（屏障等满 5 秒被打破）。"""
    barrier = threading.Barrier(2, timeout=5)
    seen_threads, broken, arrivals = set(), [], []
    arrival_lock = threading.Lock()
    original_copy = shutil.copy2

    def copy_after_barrier(path, target, **keywords):
        with arrival_lock:
            seen_threads.add(threading.get_ident())
            arrivals.append(path)
            gated = len(arrivals) <= 2   # 只有头两个文件过屏障
        if gated:
            try:
                barrier.wait()
            except threading.BrokenBarrierError:
                broken.append(path)
        return original_copy(path, target, **keywords)
    destination = os.path.join(work, "classify-destination-threads")
    shutil.copy2 = copy_after_barrier
    try:
        copied = sync_dependency_artifacts(seed_target, destination, source, dict(selftest.base_environment), threads=4)
    finally:
        shutil.copy2 = original_copy
    selftest.check("工作进程拷依赖产物分到几条线程上同时拷：4 条线程，头两个文件同时到两方屏障（没有一次等满 5 秒），fakedep 的产物拷到了",
                   copied[0] >= 2 and len(seen_threads) >= 2 and not broken and os.path.isfile(os.path.join(destination, "debug/deps/libfakedep-0123456789abcdef.rlib")),
                   f"拷了 {copied}；拷的线程 {len(seen_threads)} 条；等满 5 秒屏障被打破的 {len(broken)} 次")


def seed_artifact_message(package, root, unit_hash, fresh, features=()):
    """cargo --message-format=json 的一条 compiler-artifact（字段照真 cargo：package_id、manifest_path、target、profile、features、filenames、executable、fresh）。"""
    return json.dumps({"reason": "compiler-artifact", "package_id": f"path+file://{root}/crates/{package}#{package}@0.1.0",
                       "manifest_path": os.path.join(root, "crates", package, "Cargo.toml"),
                       "target": {"kind": ["lib"], "crate_types": ["lib"], "name": package, "src_path": os.path.join(root, "crates", package, "src", "lib.rs")},
                       "profile": {"opt_level": "0", "debuginfo": 0, "test": True}, "features": list(features),
                       "filenames": [f"/target/debug/deps/{package}-{unit_hash}"], "executable": f"/target/debug/deps/{package}-{unit_hash}", "fresh": fresh})


def selftest_seed_group_cells(selftest, work):
    """先编有两组时，后一组复用前一组编出来的产物（cargo 报 fresh）不算坏，头一次出现就是 fresh 的才算坏。"""
    rows = SELFTEST_TABLE_ROWS + ["double-release-becomes-triple\tcrates/tiny/src/lib.rs\t{ value * 2 }\t{ value * 3 }\t-p tiny --release --lib -- doubles\tdoubles"]
    repository = os.path.join(work, "seed-groups")
    build_selftest_repository(repository, rows)
    selftest_write(os.path.join(repository, "Cargo.lock"), SELFTEST_EXTERNAL_LOCK)
    target = os.path.join(work, "seed-groups-target")
    common = dict(GATE_MUTATION_WORKERS="2", GATE_MUTATION_TARGET_DIR=target)
    twice = selftest.run(repository, **common)
    selftest.check("先编那一份有两组（-p tiny 与 -p tiny --release）：后一组复用前一组编出来的 tiny、假 cargo 报 fresh，不算坏、整道照常",
                   twice[0] == 0 and "先编一份依赖（普通）：" in twice[2] and "编了 2 组" in twice[2] and "是 fresh" not in twice[1] + twice[2],
                   f"退 {twice[0]}：{(twice[1] + twice[2])[-900:]}")
    replace_in_file(os.path.join(repository, "crates", "tiny", "src", "lib.rs"), "// 本模块自证的样本 crate", "// 本模块自证的样本 crate（改过一次）")
    edited = selftest.run(selftest_new_library_copy(work, repository, "seed-groups-edited"), **common)   # 注释改动单元键不变：换一棵树才有行要现判
    selftest.check("先编那一份有两组、改了 tiny 的 lib.rs 之后：摸 crates/tiny、前一组重编、后一组复用，整道照常",
                   edited[0] == 0 and "摸了整包的 1 个：crates/tiny" in edited[2] and "编了 2 组" in edited[2] and "是 fresh" not in edited[1] + edited[2],
                   f"退 {edited[0]}：{(edited[1] + edited[2])[-900:]}")
    replace_in_file(os.path.join(repository, "crates", "tiny", "src", "lib.rs"), "（改过一次）", "（改过两次）")
    per_group = selftest.run(selftest_new_library_copy(work, repository, "seed-groups-per-group"), **common, GATE_MUTATION_BREAK=with_break(selftest, "seedfreshacrossgroups"))
    selftest.check("先编那一份有两组、每组各判各的（seedfreshacrossgroups）：后一组里 tiny 是 fresh 记成坏的，先编那一趟判红、一条都不跑",
                   per_group[0] == 1 and "crates/tiny 的 tiny 是 fresh" in per_group[1] + per_group[2] and "--release" in per_group[1] + per_group[2]
                   and plain_invocations(per_group[3]) == [], f"退 {per_group[0]}：{(per_group[1] + per_group[2])[-900:]}；假 cargo {per_group[3]}")
    replace_in_file(os.path.join(repository, "crates", "tiny", "src", "lib.rs"), "（改过两次）", "（改过三次）")
    kept = selftest.run(selftest_new_library_copy(work, repository, "seed-groups-kept"), **common, GATE_MUTATION_BREAK=with_break(selftest, "seedmtimekept"))
    selftest.check("先编那一份有两组、修改时刻被留住（seedmtimekept）：两组里 tiny 头一次出现就是 fresh，照判红、一条都不跑",
                   kept[0] == 1 and "crates/tiny 的 tiny 是 fresh" in kept[1] + kept[2] and plain_invocations(kept[3]) == [],
                   f"退 {kept[0]}：{(kept[1] + kept[2])[-900:]}；假 cargo {kept[3]}")
    # 直接调：两组的 cargo JSON 输出，同一份记录（compiled）串起来
    root = os.path.join(work, "identity-source")
    touched = {"crates/tiny"}
    first_group = seed_artifact_message("tiny", root, "aaaaaaaaaaaaaaaa", False)
    reused = seed_artifact_message("tiny", root, "aaaaaaaaaaaaaaaa", True)
    other_features = seed_artifact_message("tiny", root, "bbbbbbbbbbbbbbbb", True, features=("gpu",))
    compiled = set()
    carried = [stale_seed_artifacts(first_group, root, touched, compiled), stale_seed_artifacts(reused, root, touched, compiled)]
    selftest.check("先编两组的 cargo 输出：前一组 tiny 非 fresh、后一组同一个单元 fresh，不算坏",
                   carried == [[], []] and len(compiled) == 1, f"{carried}；记下的单元 {len(compiled)} 个")
    separate = [stale_seed_artifacts(first_group, root, touched), stale_seed_artifacts(reused, root, touched)]
    selftest.check("先编两组的 cargo 输出各判各的（不给记录，seedfreshacrossgroups 的样子）：后一组 tiny fresh 算坏", separate == [[], ["crates/tiny 的 tiny"]], f"{separate}")
    compiled = set()
    stale_seed_artifacts(first_group, root, touched, compiled)
    selftest.check("同包另一套特性编出来的单元（身份不同）在后一组头一次出现就是 fresh：照算坏，只按包名放行不行",
                   stale_seed_artifacts(other_features, root, touched, compiled) == ["crates/tiny 的 tiny"], f"{compiled}")
    selftest.check("头一组里头一次出现就是 fresh 的：照算坏", stale_seed_artifacts(reused, root, touched, set()) == ["crates/tiny 的 tiny"])
    selftest.check("没摸过的包 fresh 不算坏（照旧）", stale_seed_artifacts(seed_artifact_message("fakedep", root, "cccccccccccccccc", True), root, touched, set()) == [])


def selftest_pool_parallel_cells(selftest, work):
    """派活那一段（文件头「派活那一段」）：变异表 8 行、cargo 参数 `--` 之前各差一个 -j，基线分成 8 批；假 cargo 记每次起止的时刻（FAKE_CARGO_SPAN_LOG）。
    4 个工作进程那一趟：同时在跑的基线峰值不少于 4。
    磁盘余量压到 1 个那一趟（GATE_MUTATION_DISK_FREE=200G：一个工作进程两份编译目录各按上限 64G 算、加备用 30G，第二个连同先编那两份放不下）：
    头一行写开 1 个、卡在「磁盘余量」这一项，同时在跑的基线至多 1 批。
    弄坏开关 baseline-one-at-a-time 让头一格判错（同时在跑的基线只有 1 批）。"""
    def table_rows(count):
        return [f"pool-row-{index}\tcrates/tiny/src/lib.rs\t{{ left + right }}\t{{ left * (right + {index}) }}\t-p tiny --lib -j {index + 1} -- adds\tadds"
                for index in range(count)]

    def spans_of(path):
        """[(start 或 end, TMPDIR 的名字, 时刻)]，照文件里的次序（假 cargo 每次 O_APPEND 一次写一行，行序就是真实的先后）。"""
        events = []
        with contextlib.suppress(OSError):
            with open(path, encoding="utf-8") as handle:
                for line in handle.read().splitlines():
                    fields = line.split("\t")
                    with contextlib.suppress(ValueError, IndexError):
                        events.append((fields[0], fields[1], float(fields[2].replace(",", "."))))
        return events

    def baseline_peak(events):
        opened, peak = set(), 0
        for kind, stamp, _moment in events:
            if not stamp.startswith("baseline-"):
                continue
            if kind == "start":
                opened.add(stamp)
                peak = max(peak, len(opened))
            else:
                opened.discard(stamp)
        return peak

    for name in ("pool-parallel-spans.log", "pool-serial-spans.log"):
        with contextlib.suppress(FileNotFoundError):
            os.unlink(os.path.join(work, name))
    parallel_repository = os.path.join(work, "pool-parallel")
    build_selftest_repository(parallel_repository, table_rows(8))
    parallel_spans = os.path.join(work, "pool-parallel-spans.log")
    parallel = selftest.run(parallel_repository, GATE_MUTATION_WORKERS="4",
                            FAKE_CARGO_SPAN_LOG=parallel_spans, FAKE_CARGO_SPAN_SLEEP="2", SINGLEFS_THREAD_CAP="4")
    parallel_events = spans_of(parallel_spans)
    selftest.check("4 个工作进程、8 批基线：同时在跑的基线不少于工作进程数（4），8 行都判成抓到",
                   parallel[0] == 0 and "这一轮开 4 个工作进程" in parallel[2] and "现跑基线 8 批 8 行" in parallel[2]
                   and baseline_peak(parallel_events) >= 4 and len(per_row_how(parallel[1])) == 8,
                   f"退 {parallel[0]}；同时在跑的基线峰值 {baseline_peak(parallel_events)}（cargo 起止 {len(parallel_events)} 行）；抓到 {len(per_row_how(parallel[1]))} 行；"
                   f"{parallel[2][:700]}")
    serial_repository = os.path.join(work, "pool-serial")
    build_selftest_repository(serial_repository, table_rows(3))
    serial_spans = os.path.join(work, "pool-serial-spans.log")
    disk_squeezed = {DISK_FREE_VARIABLE: "200G", TARGET_DIRECTORY_CAP_VARIABLE: "64G", DISK_RESERVE_VARIABLE: "30G"}
    serial = selftest.run(serial_repository, GATE_MUTATION_WORKERS="16",
                          FAKE_CARGO_SPAN_LOG=serial_spans, FAKE_CARGO_SPAN_SLEEP="0.1", SINGLEFS_THREAD_CAP="31", **disk_squeezed)
    serial_events = spans_of(serial_spans)
    selftest.check("工作进程数被磁盘余量压到 1：头一行写开 1 个、卡在「磁盘余量」这一项，同时在跑的基线至多 1 批",
                   serial[0] == 0 and "这一轮开 1 个工作进程" in serial[2] and "卡在「磁盘余量」这一项" in serial[2]
                   and baseline_peak(serial_events) == 1 and len(per_row_how(serial[1])) == 3,
                   f"退 {serial[0]}；cargo 起止 {len(serial_events)} 行，同时在跑的基线峰值 {baseline_peak(serial_events)}；{serial[2][:700]}")


def selftest_mutation_run_service_cells(selftest, work):
    """变异复跑的崩溃判定、基线连调用方的判定库服务（文件头「变异复跑的崩溃判定」「基线连调用方的判定库服务」）：假 cargo 每次真跑测试照收到的连哪一处往服务里写一条
    （演引擎写崩溃判定，键是前缀接 TMPDIR 的名字），按名字认是哪一趟（baseline- 起头的是基线，别的是变异与记覆盖的插桩跑）。一趟跑下来两格：
    变异与插桩跑每一次连的都是它自己的一份临时服务（socket 与临时服务的标记相同、不是这一趟自证所在的那份临时服务、各次互不相同、跑完 socket 不在了），
    自证所在的那份服务（演真服务）按那个前缀没有它们写的一条，真跑测试的总次数等于假 cargo 真跑测试的次数；弄坏开关 mutation-run-on-real-service 下这一格判错。
    基线每一次连的都是这一趟自证所在的那一份（socket 与临时服务的标记都是外面给的、没起自己的服务），它写的每一条都落在那一份上；
    弄坏开关 baseline-on-temporary-service 下这一格判错。"""
    client = mutation_units.Library.client()
    repository = os.path.join(work, "mutation-run-service")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    service_log = os.path.join(work, "mutation-run-service.log")
    prefix = f"crates-mutation-rows-selftest/mutation-run-crash-verdicts/{os.getpid()}-{os.urandom(4).hex()}/"
    outer_socket = selftest.base_environment.get(client.SOCKET_VARIABLE, "")
    exit_code, _stdout, stderr, invocations = selftest.run(repository, FAKE_CARGO_SERVICE_LOG=service_log,
                                                           FAKE_CARGO_CRASH_VERDICT_PREFIX=prefix,
                                                           FAKE_CARGO_CLIENT_DIRECTORY=os.path.dirname(os.path.realpath(PRUNE_SERVER_TEMPORARY_SERVICE_SCRIPT)))
    seen = []
    if os.path.exists(service_log):
        with open(service_log, encoding="utf-8") as handle:
            seen = [line.split("\t") for line in handle.read().splitlines()]
    baseline_seen = [fields for fields in seen if len(fields) == 3 and fields[2].startswith("baseline-")]
    mutated_seen = [fields for fields in seen if fields not in baseline_seen]
    mutated_sockets = [fields[0] for fields in mutated_seen]
    test_runs = [line for line in invocations if "--no-run" not in line.split("\t")[0].split()]
    written_names = {key.decode()[len(prefix):] for key, _value in client.process_client().scan(prefix.encode())}
    baseline_names = {fields[2] for fields in baseline_seen}
    selftest.check("变异复跑各对自己的临时 prune-server 跑：每次真跑测试连的都是它自己那一份（socket 与临时服务的标记相同、不是自证所在的那一份、各次互不相同、跑完不在了），"
                   "次数等于真跑测试的次数，自证所在的那一份（演真服务）按这个前缀没有变异与插桩跑写的一条",
                   exit_code == 0 and mutated_seen and len(seen) == len(test_runs)
                   and all(len(fields) == 3 and fields[0] and fields[0] == fields[1] for fields in mutated_seen)
                   and outer_socket not in mutated_sockets and len(set(mutated_sockets)) == len(mutated_sockets)
                   and not any(os.path.exists(path) for path in mutated_sockets)
                   and not (written_names - baseline_names),
                   f"退 {exit_code}；真跑测试 {len(test_runs)} 次、写了 {len(seen)} 次（其中变异与插桩 {len(mutated_seen)} 次）；连的 {mutated_sockets[:4]}（自证所在的那一份 {outer_socket}）；"
                   f"自证所在的那一份按前缀多了变异与插桩跑写的 {sorted(written_names - baseline_names)[:4]}；{stderr[-300:]}")
    selftest.check("基线连调用方的那一份判定库服务：每次基线连的都是这一趟自证所在的那一份（socket 与临时服务的标记都是外面给的、没起自己的服务），它写的每一条都落在那一份上",
                   exit_code == 0 and baseline_seen and bool(outer_socket)
                   and all(fields[0] == outer_socket and fields[1] == selftest.base_environment.get(client.TEMPORARY_SERVICE_VARIABLE, "") for fields in baseline_seen)
                   and baseline_names <= written_names,
                   f"退 {exit_code}；基线 {len(baseline_seen)} 次；连的 {[fields[0] for fields in baseline_seen[:4]]}（自证所在的那一份 {outer_socket}）；"
                   f"基线写的 {len(baseline_names)} 条里落在自证所在那一份上的 {len(baseline_names & written_names)} 条；{stderr[-300:]}")


def selftest_output_verdict_cell(selftest, work):
    """判定相同的总核对（文件头「边跑边收、边跑边报」）：PATH 上放一份包着假 cargo 的 cargo，真跑测试的每一次先往 stderr 写 2000 行 GPU 阶段现场行，
    结果行也改写到 stderr、结果词被一行 GPU 阶段现场字样隔开（--nocapture 那种样子）。同一批行滤与不滤（lifecycle-lines-kept）各跑一趟，判定逐字相同、都判抓到。"""
    shim_directory = os.path.join(work, "lifecycle-cargo-shim")
    selftest_write(os.path.join(shim_directory, "cargo"),
                   "#!/usr/bin/env python3\n"
                   "import os, re, subprocess, sys\n"
                   f"REAL = {os.path.join(selftest.bin_directory, 'cargo')!r}\n"
                   "arguments = sys.argv[1:]\n"
                   "if 'test' not in arguments or '--no-run' in arguments:\n"
                   "    os.execv(REAL, [REAL] + arguments)\n"
                   "for _line in range(2000):\n"
                   "    sys.stderr.write('GPU_LIFECYCLE unix_microseconds=1 elapsed_microseconds=2 pid=3 thread=ThreadId(4) test=None phase=queue_submit_end\\n')\n"
                   "sys.stderr.flush()\n"
                   "run = subprocess.run([REAL] + arguments, stdout=subprocess.PIPE, text=True)\n"
                   "for line in run.stdout.splitlines(keepends=True):\n"
                   "    found = re.fullmatch(r'(test \\S+ \\.\\.\\. )(ok|FAILED)\\n', line)\n"
                   "    if found:\n"
                   "        sys.stderr.write(found.group(1) + 'GPU_LIFECYCLE unix_microseconds=9 phase=between_the_result_words\\n' + found.group(2) + '\\n')\n"
                   "    else:\n"
                   "        sys.stdout.write(line)\n"
                   "sys.exit(run.returncode)\n", executable=True)
    shimmed_path = shim_directory + os.pathsep + selftest.base_environment["PATH"]
    filtered_repository = os.path.join(work, "lifecycle-filtered")
    build_selftest_repository(filtered_repository, SELFTEST_TABLE_ROWS)
    kept_repository = selftest_new_library_copy(work, filtered_repository, "lifecycle-kept")
    filtered = selftest.run(filtered_repository, PATH=shimmed_path)
    kept = selftest.run(kept_repository, PATH=shimmed_path, GATE_MUTATION_BREAK=with_break(selftest, "lifecycle-lines-kept"))
    selftest.check("边跑边收对拍：同一批行滤与不滤（lifecycle-lines-kept），selftest_verdict_view 逐字相同、都判抓到",
                   filtered[0] == 0 and kept[0] == 0 and "计数：抓到 4 条" in filtered[1] and selftest_verdict_view(filtered[1]) == selftest_verdict_view(kept[1]),
                   f"滤 退 {filtered[0]}：{selftest_verdict_view(filtered[1])[-500:]}\n不滤 退 {kept[0]}：{selftest_verdict_view(kept[1])[-500:]}\n{filtered[2][-300:]}")


def selftest_key_pool_repository_cells(selftest, work):
    """真跑的小仓（10 来行、两份各 300 多行的 .rs，假 cargo 不起）：fork 之前那三样真做了，闭包的键与就地算、与三样各不做时逐字相同，词法单元取值不变。"""
    import gc
    count = mutation_units.PARALLEL_MINIMUM + 2
    padding = "".join(f"// 填充第 {line} 行：样本的词法单元落在 256 行之后，行号是堆上的数\n" for line in range(300))
    functions = "".join(f"pub fn step_{index}(value: u32) -> u32 {{ value + {index + 100} }}\n" for index in range(count))
    tests = "".join(f"    #[test]\n    fn steps_{index}() {{ assert_eq!(super::step_{index}(1), {index + 101}); }}\n" for index in range(count))
    rows = [f"step-{index}\tcrates/tiny/src/lib.rs\t{{ value + {index + 100} }}\t{{ value + {index + 200} }}\t-p tiny --lib -- steps_{index}\tsteps_{index}" for index in range(count)]
    repository = os.path.join(work, "key-pool")
    build_selftest_repository(repository, rows, source=padding + "pub mod extra;\n" + functions + "\n#[cfg(test)]\nmod tests {\n" + tests + "}\n")
    selftest_write(os.path.join(repository, "crates", "tiny", "src", "extra.rs"),
                   padding + "".join(f"pub fn extra_{index}(value: u32) -> u32 {{ value + 1 }}\n" for index in range(count)))
    environment = dict(selftest.base_environment)
    table_rows, _malformed = read_table(os.path.join(repository, TABLE), environment)
    keys, closure_keys, facts = selftest_key_pool_closure_keys(repository, table_rows, environment, 4)
    _serial_keys, serial_closure_keys, serial_facts = selftest_key_pool_closure_keys(repository, table_rows, environment, 1)
    selftest.check("算键的进程池：真跑的小仓上算键的键与就地算逐字相同，就地算那一趟 fork 之前那三样都不做",
                   len(closure_keys) == count and all(closure_keys) and closure_keys == serial_closure_keys
                   and (serial_facts["tokens_shared"], serial_facts["prewarmed"], serial_facts["frozen_objects"]) == (0, 0, 0), f"{closure_keys}；就地算 {serial_closure_keys}；{serial_facts}")
    selftest.check("算键的进程池：真跑的小仓上先共用了词法单元，池收完永久代是空的", facts["tokens_shared"] > 0 and gc.get_freeze_count() == 0, f"{facts}；永久代 {gc.get_freeze_count()}")
    selftest.check("算键的进程池：真跑的小仓上先算了一条", facts["prewarmed"] == 1, f"{facts}")
    selftest.check("算键的进程池：真跑的小仓上冻了对象", facts["frozen_objects"] > 0, f"{facts}")
    files = list(keys.workspace.files.values())
    numbers = {}
    interned = len(files) == 2 and all(sys.intern(text) is text for rust_file in files for text in rust_file.texts)
    shared_numbers = all(numbers.setdefault(line, line) is line for rust_file in files for line in rust_file.lines)
    selftest.check("算键的进程池：真跑的小仓上词串进了 sys.intern、两份文件里同一个行号是同一个对象", interned and shared_numbers and max(numbers) > 256, f"{len(files)} 份文件；驻留 {interned}；行号共用 {shared_numbers}")
    lfu = mutation_units.function_units()
    lexed_unchanged = True
    for path, rust_file in keys.workspace.files.items():
        with open(os.path.join(repository, path), encoding="utf-8", errors="surrogateescape") as handle:
            lexed = lfu.rust_lex(handle.read())
        lexed_unchanged = lexed_unchanged and rust_file.texts == lexed[1] and rust_file.lines == lexed[2] and rust_file.pairs == lfu.match_brackets(lexed[0], lexed[1])[0]
    selftest.check("算键的进程池：共用之后词串、行号与括号配对的取值与现切的逐字相同", lexed_unchanged, "有文件的取值变了")
    for switch_name in ("key-token-values-unshared", "key-parent-not-prewarmed", "key-pool-not-frozen"):
        _switched_keys, switched, _switched_facts = selftest_key_pool_closure_keys(repository, table_rows, dict(environment, GATE_MUTATION_BREAK=with_break(selftest, switch_name)), 4)
        selftest.check(f"算键的进程池：{switch_name} 下闭包的键与默认那一趟逐字相同（这几样只动内存，不动键）", switched == closure_keys, f"{switched}；默认 {closure_keys}")


def selftest_heavy_chunk_verdict_cell(selftest, work):
    """切前切后对拍（文件头「基线」的「重基线批切块」）：7 行、每块至多 3 个过滤词切成 3 块，与不切（heavy-batch-unsplit）各跑一趟，判定逐字相同。
    一行基线不绿（那一块退 101）不碍同一块与别块的行。两个模块各有一条 same_holds、必须红的都写短名，分在两块：照整批的输出判（baseline-batch-composition）时不切那一趟对上两个用例、切了对上一个，判定不同。"""
    cases = [("tests::amber_holds", 101, True), ("tests::birch_holds", 111, True), ("tests::cedar_holds", 121, False), ("tests::same_holds", 131, True),
             ("tests::fern_holds", 141, True), ("tests::grove_holds", 151, True), ("extra::same_holds", 161, True)]
    split_repository = os.path.join(work, "heavy-chunk-split")
    selftest_heavy_chunk_repository(split_repository, cases)
    whole_repository = selftest_new_library_copy(work, split_repository, "heavy-chunk-whole")
    split = selftest.run(split_repository, GATE_MUTATION_BASELINE_CHUNK_FILTERS="3")
    whole = selftest.run(whole_repository, GATE_MUTATION_BASELINE_CHUNK_FILTERS="3", GATE_MUTATION_BREAK=with_break(selftest, "heavy-batch-unsplit"))
    selftest.check("重基线批切前切后对拍：7 行切成 3 块与不切，selftest_verdict_view 逐字相同，一块里基线不绿的那一行不碍别的行（抓到 6 条、基线不绿 1 行）",
                   split[0] == 1 and "现跑基线 3 批 7 行" in split[2] and "现跑基线 1 批 7 行" in whole[2]
                   and "计数：抓到 6 条" in split[1] and "基线不绿 1 行" in split[1] and selftest_verdict_view(split[1]) == selftest_verdict_view(whole[1]),
                   f"切 退 {split[0]}：{selftest_verdict_view(split[1])[-600:]}\n不切 退 {whole[0]}：{selftest_verdict_view(whole[1])[-600:]}\n切的 stderr {split[2][:400]}\n不切的 stderr {whole[2][:400]}")
    bad_limit = selftest.run(split_repository, GATE_MUTATION_BASELINE_CHUNK_FILTERS="0")
    bad_interval = selftest.run(split_repository, GATE_MUTATION_PROGRESS_SECONDS="sixty")
    selftest.check("切块大小与进度间隔写成不是正整数：run_stage 拒、一条都不跑",
                   bad_limit[0] == 1 and bad_limit[3] == [] and "GATE_MUTATION_BASELINE_CHUNK_FILTERS 写成了「0」" in bad_limit[1] + bad_limit[2]
                   and bad_interval[0] == 1 and bad_interval[3] == [] and "GATE_MUTATION_PROGRESS_SECONDS 写成了「sixty」" in bad_interval[1] + bad_interval[2],
                   f"切块大小 退 {bad_limit[0]}，{bad_limit[1][-200:]}；进度间隔 退 {bad_interval[0]}，{bad_interval[1][-200:]}")


def selftest_long_baseline_chunk_cells(selftest):
    """没有用时的与长测试各占一块的算法（文件头「基线」的「没有用时的与长的测试各占一块」），只调函数、不起 cargo：一批里有用时短的、没有用时的、有用时长的各几条，
    没有用时的每条测试一块、长的每条测试一块、短的照个数切（重的批）或整批一块（轻的批），块的次序是短的、没有用时的（照表序）、长的（用时从短到长）三段，
    共用过滤词的行跟着同一条测试进同一块，每行恰好进一块、并起来等于原来的行；门槛取函数里写的、自证里换值只在临时 prune-server 底下认；
    全部有用时且都短的与不单列（弄坏开关 long-baseline-not-isolated）逐块相同；没有用时的照短的办（弄坏开关 unknown-timing-merged-as-short）下没有单列的块。"""
    environment = dict(selftest.base_environment, GATE_MUTATION_BASELINE_LONG_TEST_SECONDS="1000", GATE_MUTATION_BASELINE_CHUNK_FILTERS="6")
    registered = release_registered_targets()
    timings_directory = os.path.join(selftest.work, "chunk-timings-mixed")

    def row_of(line_number, arguments, seconds=None):
        fields = (f"selftest-long-{line_number}", "crates/tiny/src/lib.rs", "a", "b", arguments, "unused")
        row = Row(line_number, fields[0], fields[1], "a", "b", arguments, "unused", fields)
        if seconds is not None:
            selftest_write_timings(timings_directory, [row], seconds)
        return row
    seconds_of = {103: 2000, 111: 1500, 107: 999, 160: 10}
    unknown_numbers = (105, 106, 112)
    heavy_rows = [row_of(100 + index, f"-p tiny --release --features gpu --lib -- case_{index:02d}_holds",
                         None if 100 + index in unknown_numbers else seconds_of.get(100 + index, 10)) for index in range(20)]
    long_sibling = row_of(150, "-p tiny --release --features gpu --lib -- case_03_holds")   # 没有用时，与 103（2000 秒）共用过滤词：整条测试算长
    unknown_sibling = row_of(160, "-p tiny --release --features gpu --lib -- case_05_holds", 10)   # 有用时，与 105（没有用时）共用过滤词：整条测试算没有用时
    light_rows = [row_of(10, "-p tiny --lib -- adds", 5000), row_of(11, "-p tiny --lib -- doubles", 5), row_of(12, "-p tiny --lib -- negates")]
    rows = heavy_rows + [long_sibling, unknown_sibling] + light_rows

    def lines_of(chunks):
        return [[row.line_number for row in chunk] for _key, chunk in chunks]
    chunks, unknown_notes, long_notes = baseline_chunks(rows, registered, environment, timings_directory)
    short_heavy = [100 + index for index in range(20) if 100 + index not in (103, 111) + unknown_numbers]
    expected = ([[11]] + [short_heavy[:6], short_heavy[6:12], short_heavy[12:]]
                + [[12], [105, 160], [106], [112]]
                + [[111], [103, 150], [10]])
    selftest.check("没有用时的与长测试各占一块：三段次序——短的块在前（轻的批 11，重的批 15 行按 6 个过滤词切成 6、6、3），没有用时的各占一块居中（12、105 与它共用过滤词的 160、106、112，照表序），"
                   "长的各占一块在最后（用时从短到长：111 的 1500、103 与它共用过滤词的 150 的 2000、10 的 5000）；999 秒（门槛 1000）的 107 算短",
                   lines_of(chunks) == expected and set(unknown_notes) == {12, 105, 106, 112} and set(long_notes) == {111, 103, 10},
                   f"{lines_of(chunks)}；没有用时的 {sorted(unknown_notes)}；长测试 {sorted(long_notes)}")
    flat = [line for chunk in lines_of(chunks) for line in chunk]
    selftest.check("没有用时的与长测试各占一块：每行恰好进一块，块的并起来等于原来的行，批的键不动",
                   sorted(flat) == sorted(row.line_number for row in rows) and len(flat) == len(set(flat))
                   and all(key == batch_key(chunk[0].arguments) for key, chunk in chunks), f"{flat}")
    selftest.check("没有用时的与长测试各占一块：起跑那一行点名测试目标与过滤词、写依据（没有用时数据，或上一次用时）与批首行",
                   all(text in unknown_notes.get(105, "") for text in ("lib 的", "case_05_holds", "没有用时数据", "批首行是变异表第 105 行"))
                   and all(text in long_notes.get(103, "") for text in ("lib 的", "case_03_holds", "上一次用时 2000 秒", "批首行是变异表第 103 行")),
                   f"没有用时的 {unknown_notes}；长测试 {long_notes}")
    default_environment = dict(environment)
    default_environment.pop("GATE_MUTATION_BASELINE_LONG_TEST_SECONDS")
    default_notes = baseline_chunks(rows, registered, default_environment, timings_directory)[2]
    selftest.check("长测试的门槛：函数里写着 900 秒，换值只在临时 prune-server 底下认（自证里换成 1000、不在临时 prune-server 底下的环境照取 900）；门槛 900 时 999 秒那一条也算长",
                   baseline_long_test_seconds(default_environment) == 900 and baseline_long_test_seconds(environment) == 1000
                   and baseline_long_test_seconds({"GATE_MUTATION_BASELINE_LONG_TEST_SECONDS": "5"}) == 900 and set(default_notes) == {111, 103, 107, 10},
                   f"默认 {baseline_long_test_seconds(default_environment)}、换值 {baseline_long_test_seconds(environment)}；门槛 900 的长测试 {sorted(default_notes)}")
    merged_chunks, merged_unknown_notes, merged_long_notes = baseline_chunks(
        rows, registered, dict(environment, GATE_MUTATION_BREAK=with_break(selftest, "unknown-timing-merged-as-short")), timings_directory)
    merged_lines = lines_of(merged_chunks)
    selftest.check("没有用时的照短的办（弄坏开关 unknown-timing-merged-as-short）：没有用时的不单列（12 与 11 同块、106 与别的行同块），长的照旧单列在最后",
                   merged_unknown_notes == {} and [11, 12] in merged_lines and [106] not in merged_lines and [12] not in merged_lines
                   and merged_lines[-3:] == [[111], [103, 150], [10]] and set(merged_long_notes) == {111, 103, 10}, f"{merged_lines}；没有用时的 {merged_unknown_notes}")
    short_directory = os.path.join(selftest.work, "chunk-timings-short")
    short_rows = []
    for index in range(40):
        short_rows.append(Row(500 + index, f"selftest-short-{index}", "crates/tiny/src/lib.rs", "a", "b", f"-p tiny --release --features gpu --lib -- short_{index:02d}_holds", "unused",
                              (f"selftest-short-{index}", "crates/tiny/src/lib.rs", "a", "b", f"short_{index:02d}", "unused")))
    selftest_write_timings(short_directory, short_rows, 10)
    sixteen = dict(environment, GATE_MUTATION_BASELINE_CHUNK_FILTERS="16")
    all_short = baseline_chunks(short_rows, registered, sixteen, short_directory)
    not_isolated = baseline_chunks(short_rows, registered, dict(sixteen, GATE_MUTATION_BREAK=with_break(selftest, "long-baseline-not-isolated")), short_directory)
    selftest.check("没有用时的与长测试各占一块：全部有用时且都短于门槛的 40 行切成 16、16、8，与不单列（弄坏开关 long-baseline-not-isolated）逐块相同、没有单列的块",
                   [len(chunk) for _key, chunk in all_short[0]] == [16, 16, 8] and [row for _key, chunk in all_short[0] for row in chunk] == short_rows
                   and all_short[1] == {} and all_short[2] == {} and all_short[0] == not_isolated[0] and not_isolated[1:] == ({}, {}),
                   f"{lines_of(all_short[0])}；没有用时的 {all_short[1]}；长测试 {all_short[2]}")
    no_timing_directory = os.path.join(selftest.work, "chunk-timings-absent")   # 没有这个目录：每一行都没有用时
    unknown_chunks, unknown_only_notes, unknown_long_notes = baseline_chunks(short_rows[:10], registered, sixteen, no_timing_directory)
    selftest.check("没有用时的与长测试各占一块：10 行都没有用时，每条测试一块、照表序、没有长的",
                   lines_of(unknown_chunks) == [[500 + index] for index in range(10)] and set(unknown_only_notes) == {500 + index for index in range(10)} and unknown_long_notes == {},
                   f"{lines_of(unknown_chunks)}；{sorted(unknown_only_notes)}")


def selftest_long_baseline_chunk_run_cell(selftest, work):
    """没有用时的与长测试单列一块的跑法（文件头「基线」的「没有用时的与长的测试各占一块」）：7 行每块至多 3 个过滤词，用时表里 4 条短的（10 秒）、fern_holds 400 秒（门槛换成 100 秒）、
    grove_holds 与 kiln_holds 没有用时：切成 5 块（短的 4 条切 3、1 两块，没有用时的各一块，长的 fern_holds 一块排在最后），stderr 头一行报「其中长测试独占 1 批、没有用时独占 2 批」，
    没有用时的与长的各自起跑打一行点名，判定与不单列（long-baseline-not-isolated）逐字相同；没有用时的照短的办（unknown-timing-merged-as-short）那一趟没有用时的并进按个数切的块、
    头一行报「没有用时独占 0 批」、没有那两行起跑。"""
    cases = [("tests::amber_holds", 101, True), ("tests::birch_holds", 111, True), ("tests::cedar_holds", 121, True), ("tests::dune_holds", 131, True),
             ("tests::fern_holds", 141, True), ("tests::grove_holds", 151, True), ("extra::kiln_holds", 161, True)]
    isolated_repository = os.path.join(work, "long-baseline-isolated")
    selftest_heavy_chunk_repository(isolated_repository, cases)
    merged_repository = selftest_new_library_copy(work, isolated_repository, "long-baseline-merged")
    mixed_repository = selftest_new_library_copy(work, isolated_repository, "long-baseline-mixed")
    pristine = source_hash(isolated_repository)
    table_rows, _malformed = read_table(os.path.join(isolated_repository, TABLE))
    seconds_of = {"fern_holds": 400, "grove_holds": None, "kiln_holds": None}
    for repository in (isolated_repository, merged_repository, mixed_repository):
        timings_directory = os.path.join(run_records_directory(repository, dict(selftest.base_environment)), "timings")
        for row in table_rows:
            if seconds_of.get(row.expected, 10) is not None:
                selftest_write_timings(timings_directory, [row], seconds_of.get(row.expected, 10))
    settings = dict(GATE_MUTATION_BASELINE_CHUNK_FILTERS="3", GATE_MUTATION_BASELINE_LONG_TEST_SECONDS="100")
    selftest.seeds_short_timings = False   # 没有用时的行要保持没有用时
    try:
        isolated = selftest.run(isolated_repository, **settings)
        merged = selftest.run(merged_repository, GATE_MUTATION_BREAK=with_break(selftest, "unknown-timing-merged-as-short"), **settings)
        mixed = selftest.run(mixed_repository, GATE_MUTATION_BREAK=with_break(selftest, "long-baseline-not-isolated"), **settings)
    finally:
        selftest.seeds_short_timings = True

    def baselines_of(run):
        return [line for line in plain_invocations(run[3]) if line.endswith("\t" + pristine)]
    isolated_baselines, merged_baselines, mixed_baselines = baselines_of(isolated), baselines_of(merged), baselines_of(mixed)
    long_line = re.search(r"^  … 基线长测试单列一块：.*fern_holds.*上一次用时 400 秒；批首行是变异表第 \d+ 行）$", isolated[2], re.M)
    unknown_lines = re.findall(r"^  … 基线没有用时的测试单列一块：.*(?:grove_holds|kiln_holds).*（没有用时数据；批首行是变异表第 \d+ 行）$", isolated[2], re.M)
    selftest.check("没有用时的与长测试单列一块的跑法：7 行切成 5 块（短的 4 条切两块、没有用时的 grove_holds 与 kiln_holds 各一块、长的 fern_holds 一块排在最后），"
                   "stderr 头一行报「其中长测试独占 1 批、没有用时独占 2 批」、两种块起跑各打一行点名，判定与不单列逐字相同",
                   isolated[0] == 0 and merged[0] == 0 and mixed[0] == 0
                   and "现跑基线 5 批 7 行（其中长测试独占 1 批、没有用时独占 2 批）" in isolated[2] and long_line is not None and len(unknown_lines) == 2
                   and len(isolated_baselines) == 5 and "fern_holds" in isolated_baselines[-1]
                   and "grove_holds" in isolated_baselines[-3] and "kiln_holds" in isolated_baselines[-2]
                   and not any(word in line for line in isolated_baselines[:-3] for word in ("fern_holds", "grove_holds", "kiln_holds"))
                   and selftest_verdict_view(isolated[1]) == selftest_verdict_view(mixed[1]) == selftest_verdict_view(merged[1]),
                   f"独占 退 {isolated[0]}、没有用时照短的 退 {merged[0]}、不单列 退 {mixed[0]}；长的起跑那一行 {long_line and long_line.group(0)}；没有用时的起跑 {unknown_lines}；独占的基线 {isolated_baselines}\\n"
                   f"独占的 stderr {isolated[2][:600]}")
    selftest.check("没有用时的与长测试单列一块的跑法：没有用时的照短的办（弄坏开关 unknown-timing-merged-as-short）那一趟头一行报「长测试独占 1 批、没有用时独占 0 批」、没有没有用时的起跑那两行，"
                   "grove_holds、kiln_holds 与短的同块；不单列（弄坏开关 long-baseline-not-isolated）那一趟报「独占 0 批、没有用时独占 0 批」、没有两种起跑行、fern_holds 与别的测试同块",
                   "现跑基线 3 批 7 行（其中长测试独占 1 批、没有用时独占 0 批）" in merged[2] and "基线没有用时的测试单列一块" not in merged[2] and "fern_holds" in merged_baselines[-1]
                   and len(merged_baselines) == 3 and "grove_holds" in merged_baselines[1] and "kiln_holds" in merged_baselines[1]
                   and "现跑基线 3 批 7 行（其中长测试独占 0 批、没有用时独占 0 批）" in mixed[2] and "基线长测试单列一块" not in mixed[2] and "基线没有用时的测试单列一块" not in mixed[2]
                   and len(mixed_baselines) == 3 and "fern_holds" not in mixed_baselines[-1],
                   f"没有用时照短的 stderr {merged[2][:500]}\\n不单列 stderr {mixed[2][:500]}；基线 {merged_baselines}；{mixed_baselines}")
