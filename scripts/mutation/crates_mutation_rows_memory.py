"""crates-mutation-rows.py 拆出的一份。
内存：任务内存上限、进程树用量、每个工作进程的峰值（量、记、读、除）、内存准入、工作进程数与跑着加开，连同这几样的自证格。
入口文件头写着这一份在的位置、弄坏开关表与用法；入口 crates-mutation-rows.py 从各份 import 全部名字再导出，别处照旧按入口取名字。
拆出的各份只向前 import（base → judging → support → static → units → baseline → targets → memory → runs → pool → plan → stage → commands → driver），没有环。
"""
import contextlib
import os
import re
import subprocess
import sys
import time
from crates_mutation_rows_base import (CGROUP_LISTING, CGROUP_ROOT, DISK_FREE_VARIABLE, DISK_RESERVE_VARIABLE, HeavyThreadDefaultError, Row,
    SIZE_TEXT, SIZE_UNIT_BYTES, SLICE_MEMORY_CAP_KEY, STAGE_REPOSITORY, TARGET_DIRECTORY_CAP_VARIABLE, TASK_MEMORY_CAP_VARIABLE, WORKER_PEAKS_RECENT,
    WORKER_PEAK_PACKAGE_NAME, WORKER_PEAK_RECORD_NAME, WORKER_PEAK_SAMPLE_SECONDS, WORKER_PEAK_WORKSPACE_SCOPE, WORKER_PEAK_WRITE_STEP_BYTES,
    WORKER_STATE, break_is, bytes_of_size, default_heavy_threads, format_size, mutation_units, note, task_memory_cap_bytes,
    under_temporary_prune_server, write_atomically)
from crates_mutation_rows_support import (build_selftest_repository, per_row_how, plain_invocations, selftest_new_library_copy, selftest_write)
from crates_mutation_rows_units import (row_content_hash, run_records_directory)
from crates_mutation_rows_baseline import (dispatch_order)
from crates_mutation_rows_targets import (disk_bound_for, selftest_write_random)


# 给了顶掉每台默认重活线程数的两个变量：工作进程数（worker_count_for）、每个 cargo 的编译并行度（cargo_jobs_for）；
# 读它们、没给取 default_heavy_threads 的地方都在本模块（compile_thread_budget、worker_count_for），别的模块只经这一对名字点名它们
WORKER_COUNT_VARIABLE = "GATE_MUTATION_WORKERS"
CARGO_JOBS_VARIABLE = "GATE_MUTATION_CARGO_JOBS"
THREAD_SETTING_VARIABLES = (WORKER_COUNT_VARIABLE, CARGO_JOBS_VARIABLE)


def compile_thread_budget(environment):
    """cargo_jobs_for 里被工作进程数除的整台线程数：每台默认重活线程数（scripts/hosts/heavy_threads.py 的 default_heavy_threads：SINGLEFS_THREAD_CAP 或
    .claude/heavy-thread-default；取不到抛 HeavyThreadDefaultError）。GATE_MUTATION_CARGO_JOBS 给了，cargo_jobs_for 不用这个数，这时不读登记位、交 1 占位。
    run_stage 与分片的内存账（mutation-shard-run.sh）同用这一份，两边算的编译并行度对得上。
    弄坏开关 workers-by-processor-half：整台线程数取本机核数，不读登记位、取不到也不拒。"""
    if break_is(environment, "workers-by-processor-half"):
        return os.cpu_count() or 4
    return 1 if environment.get("GATE_MUTATION_CARGO_JOBS", "").strip() else int(default_heavy_threads(environment))


def cargo_jobs_for(environment, worker_count, thread_budget):
    """每个 cargo 的编译并行度（文件头「编译并行度」）：GATE_MUTATION_CARGO_JOBS 给了照它（run_stage 先核过是正整数），不然 max(1, 整台线程数 ÷ 工作进程数)，
    整台线程数 thread_budget 取 compile_thread_budget 交的每台默认重活线程数。
    memory_bound_for 试每个进程数、run_stage 设 CARGO_BUILD_JOBS 用的都是这一份，两边算的编译并行度对得上。"""
    given = environment.get("GATE_MUTATION_CARGO_JOBS", "").strip()
    return int(given) if given else max(1, thread_budget // max(1, worker_count))


def worker_peaks_directory(records_directory):
    """每个工作进程的峰值记录（文件头「内存上限」的「每个工作进程的峰值」）：运行记录目录下 worker-peaks/<包>/<编译并行度>/<记录名>。"""
    return os.path.join(records_directory, "worker-peaks")


def worker_peak_packages(arguments_texts):
    """一格（或这一趟要判的几行）的 cargo 参数点名的包，排好序、去重（文件头「内存上限」的「记在哪」）：每份参数的 -p / --package 各一个（name@版本、name:版本只取包名）；
    一份参数没点名包、或点名的写法不是包名（通配这一类），算 WORKER_PEAK_WORKSPACE_SCOPE。带不带特性（--features gpu 这一类）不分：同一个包的各种特性共用一份记录，取里面最大的。"""
    found = set()
    for text in arguments_texts:
        named = {re.split(r"[@:]", spec, maxsplit=1)[0] for spec in mutation_units.parse_cargo_arguments(text).packages}
        valid = {name for name in named if WORKER_PEAK_PACKAGE_NAME.fullmatch(name)}
        found |= valid
        if not valid or valid != named:
            found.add(WORKER_PEAK_WORKSPACE_SCOPE)
    return tuple(sorted(found))


def worker_peak_target_key(arguments_text):
    """一份 cargo 参数的测试目标那一级的记录名（文件头「内存上限」的「记在哪」）：恰好点名一个包（名字是包名）、又恰好点名一个目标（--lib、--test <名>、--bin <名>、--example <名>、--bench <名>）时，
    <包>--lib 或 <包>--<目标种类>--<目标名>；别的（没点名包、点名几个包、没点名目标、点名几个目标、--bins 与 --tests 这类带通配的、名字不是包名那种写法）交 None。
    这个名字与包名同样是 worker-peaks/ 下一级的目录名（measured_worker_peaks 读回来照包名判，不认得特别的写法）；包名里带 -- 的不出这一级，免得拆回包名时拆错。"""
    parsed = mutation_units.parse_cargo_arguments(arguments_text)
    if len(parsed.packages) != 1 or len(parsed.targets) != 1:
        return None
    package = re.split(r"[@:]", parsed.packages[0], maxsplit=1)[0]
    kind, name = parsed.targets[0]
    if "--" in package or kind not in ("lib", "test", "bin", "example", "bench"):
        return None
    key = f"{package}--lib" if kind == "lib" else f"{package}--{kind}--{name}"
    return key if WORKER_PEAK_PACKAGE_NAME.fullmatch(key) and WORKER_PEAK_PACKAGE_NAME.fullmatch(package) else None


def worker_peak_keys(arguments_texts):
    """这一趟要判的几行（或一格）估每个工作进程的峰值取哪几个记录名，排好序、去重：每份 cargo 参数有测试目标那一级的记录名（worker_peak_target_key）就取它，没有取包名（worker_peak_packages 那一份）。
    一个重测试目标只定它自己那几行的估值，不定整个包。弄坏开关 worker-peaks-unkeyed-by-target：一律取包名（worker_peak_packages）。"""
    if break_is(os.environ, "worker-peaks-unkeyed-by-target"):
        return worker_peak_packages(arguments_texts)
    found = set()
    for text in arguments_texts:
        key = worker_peak_target_key(text)
        found |= {key} if key is not None else set(worker_peak_packages([text]))
    return tuple(sorted(found))


def peak_key_label(key):
    """给人看的记录名：<包>--lib 写包名（点名 --lib 的行与包名同一句话），别的照原样。"""
    return key[:-len("--lib")] if key.endswith("--lib") else key


def peak_record_names(key):
    """估一个记录名的峰值先后看哪几份记录：测试目标那一级的名字（<包>--…）先看它自己、再看它的包；包名只看自己。"""
    return (key, key.split("--", 1)[0]) if "--" in key else (key,)


@contextlib.contextmanager
def worker_peak_scope(arguments_texts):
    """工作进程里判一格期间，采样线程（sample_worker_peak）把读数记在这些 cargo 参数点名的包下，也记在它的测试目标那一级下（worker_peak_target_key）；出来清掉：格与格之间、起来时拷副本的读数不记在任何一个包下。"""
    targets = () if break_is(os.environ, "worker-peaks-unkeyed-by-target") else tuple(
        sorted({key for key in map(worker_peak_target_key, arguments_texts) if key is not None}))
    WORKER_STATE["peak_scope"] = worker_peak_packages(arguments_texts) + targets
    try:
        yield
    finally:
        WORKER_STATE["peak_scope"] = ()


def new_worker_peak_record_name():
    """一个工作进程这一趟那份记录的名字（scripts/hosts/task_memory.py 的 new_worker_peak_record_name）：起采样的时刻纳秒（20 位，名字排序就是先后）、进程号、8 位随机字符。"""
    import task_memory
    return task_memory.new_worker_peak_record_name()


def record_worker_peak(records_directory, package, jobs, name, peak_bytes):
    """把一个工作进程判 package 这个包的格时到此刻的最大写进 worker-peaks/<package>/<jobs>/<name>（同目录临时文件、改名换上；写不进抛 OSError）。"""
    write_atomically(os.path.join(worker_peaks_directory(records_directory), package, str(jobs), name), f"{peak_bytes}\n")


def worker_peaks_recorded_for(root, environment):
    """这一趟的工作进程写不写峰值记录（文件头「内存上限」的「记在哪」）：被判的是本模块所在的仓、或 --staged 的临时树（GATE_STAGED_FROM 设着）、
    或在临时 prune-server 底下（自证：运行记录目录按库所在的目录分开）才写；别处（门禁样本根这一类）不写：样本 crate 的峰值不代表本仓，
    不在临时服务底下时它的运行记录目录就是本仓那一份，写进去会把小读数掺进去。弄坏开关 sample-root-peaks-recorded：都写。"""
    return (break_is(environment, "sample-root-peaks-recorded") or os.path.realpath(root) == os.path.realpath(STAGE_REPOSITORY)
            or bool(environment.get("GATE_STAGED_FROM", "").strip()) or under_temporary_prune_server(environment))


def process_kind(command_name, executable):
    """进度行按哪一类记这个进程（文件头「边跑边收、边跑边报」）：/proc/<pid>/comm 的 command_name 与 /proc/<pid>/exe 指到的 executable（读不到是空串）。
    cargo 编出来的测试二进制在 deps/ 底下，名字各不相同，只能看路径。"""
    if command_name.startswith("python"):
        return "python"
    if command_name == "rustc":
        return "rustc"
    if command_name == "prune-server" or os.path.basename(executable) == "prune-server":
        return "prune-server"
    return "test-binary" if "/deps/" in executable else "other"


def process_tree_anon_bytes_by_kind(root_pid):
    """root_pid 连同它整棵进程树此刻按类别（process_kind）加总的 RssAnon：{类别: 字节}，五类都在。读不到的进程（刚退出）不算。"""
    import task_memory
    totals = dict.fromkeys(("python", "test-binary", "rustc", "prune-server", "other"), 0)
    pending, seen = [root_pid], set()
    while pending:
        pid = pending.pop()
        if pid in seen:
            continue
        seen.add(pid)
        anon_bytes = task_memory.process_status_bytes(pid, ("RssAnon",))
        if anon_bytes is None:
            continue
        try:
            with open(f"/proc/{pid}/comm", encoding="utf-8", errors="replace") as handle:
                command_name = handle.read().strip()
            executable = os.readlink(f"/proc/{pid}/exe")
        except OSError:
            command_name, executable = "", ""
        totals[process_kind(command_name, executable)] += anon_bytes
        pending += task_memory.process_children(pid)[0]
    return totals


def base_reading_line(parent_bytes, seed_tree_peak_bytes, seed_built):
    """底座读数那一行（文件头「内存上限」的「底座读数」）：机读的 MUTATION_BASE_READING 三个字段，后面接一句人读的。"""
    parent = "none" if parent_bytes is None else str(parent_bytes)
    seed = "none" if seed_tree_peak_bytes is None else str(seed_tree_peak_bytes)
    parent_text = "读不到" if parent_bytes is None else format_size(parent_bytes)
    seed_text = "读不到" if seed_tree_peak_bytes is None else format_size(seed_tree_peak_bytes)
    return (f"MUTATION_BASE_READING parent_bytes={parent} seed_step_tree_peak_bytes={seed} seed_step={'built' if seed_built else 'not-built'}"
            f"  （底座读数：派活之前起这一道的父进程自己 {parent_text}；先编一份依赖那一步（工作进程起来之前）父进程整棵进程树的峰值 {seed_text}"
            f"{'' if seed_built else '，这一趟没先编，只量到父进程自己'}；口径同每个工作进程的读数：RssAnon + RssShmem + VmPTE 加总）")


def process_tree_usage(root_pid, environment):
    """root_pid 连同它整棵进程树此刻的用量（文件头「内存上限」的「量什么」）：字节；量法在 scripts/hosts/task_memory.py 的 process_tree_usage。
    root_pid 自己读不到、或它主线程的 children 文件不在（这台内核没有这一项）交 None。弄坏开关 worker-tree-self-only：只算 root_pid 自己。"""
    import task_memory
    return task_memory.process_tree_usage(root_pid, self_only=break_is(environment, "worker-tree-self-only"))


def sample_worker_peak(records_directory, jobs, environment, seen):
    """工作进程里的采样线程（init_worker 起的守护线程，随工作进程一起没）：每 WORKER_PEAK_SAMPLE_SECONDS 秒量一次本进程整棵树（process_tree_usage），
    读数记在此刻在判的那一格点名的包下（WORKER_STATE["peak_scope"]，worker_peak_scope 设；点名几个包的，每个包都记）：每个包一份记录（名字同一个工作进程用同一个），
    这个包头一次量到就写，之后这个包的最大涨了 WORKER_PEAK_WRITE_STEP_BYTES 以上再重写（record_worker_peak）。不在判任何一格时（起来时拷副本、格与格之间）的读数不记。
    判格期间的读数同时往 seen（run_pool 建的 multiprocessing.Value，这一趟各工作进程到此刻实测峰值的最大，跑着加开读它）里取大；seen 是 None 不写。
    环境里 GATE_MUTATION_SELFTEST_WORKER_BYTES 设了（只给自证造「实测峰值」的现场，生产不设），量得到时读数换成它。
    量不了、写不进各报一行就收手：这一格的判定不受影响，下一趟少这一份读数。"""
    name, peaks, written = new_worker_peak_record_name(), {}, {}
    selftest_usage_text = environment.get("GATE_MUTATION_SELFTEST_WORKER_BYTES", "").strip()
    while True:
        usage = process_tree_usage(os.getpid(), environment)
        if usage is None:
            note(f"  ! 工作进程 {os.getpid()} 量不了自己的进程树（/proc/{os.getpid()}/task/{os.getpid()}/children 不在：这台内核没有这一项），这一趟不记峰值；"
                 "下一趟这个编译并行度照没有实测取保守值")
            return
        if selftest_usage_text.isdigit() and int(selftest_usage_text) > 0:
            usage = int(selftest_usage_text)
        if seen is not None and WORKER_STATE.get("peak_scope"):
            with seen.get_lock():
                seen.value = max(seen.value, usage)
        for package in WORKER_STATE.get("peak_scope", ()):
            peaks[package] = max(peaks.get(package, 0), usage)
            if package not in written or peaks[package] >= written[package] + WORKER_PEAK_WRITE_STEP_BYTES:
                try:
                    record_worker_peak(records_directory, package, jobs, name, peaks[package])
                except OSError as error:
                    note(f"  ! 工作进程 {os.getpid()} 的峰值记录没写进 {worker_peaks_directory(records_directory)}（{type(error).__name__}：{error}）；下一趟少这一份读数，不碍这一格的判定")
                    return
                written[package] = peaks[package]
        time.sleep(WORKER_PEAK_SAMPLE_SECONDS)


def measured_worker_peaks(records_directory, environment):
    """读回每个工作进程的峰值记录（文件头「内存上限」的「怎么读」）：({包: {编译并行度: (最新 WORKER_PEAKS_RECENT 份里最大的字节, 份数)}} 或 None, 说明)。
    worker-peaks/ 还没有交 {}（从没量过）；有一样不是这一版的写法或读不了交 None（整份读不了），说明里点名哪一份、为什么与出路。
    读的时候删掉每个包、每个编译并行度里比最新那几份更旧的。弄坏开关 worker-peaks-unkeyed-by-package：每个包都读成所有包合起来的（同一个编译并行度下取各包里最大的）；
    worker-peaks-unkeyed-by-jobs：每个包里各编译并行度都取全部记录里最大的；worker-peaks-window-unbounded：取全部记录的最大、不删更旧的。"""
    base = worker_peaks_directory(records_directory)
    howto = f"删掉那一份（或整个 {base}）再跑：下一趟照没有实测取保守值、重新量"
    try:
        package_names = sorted(os.listdir(base))
    except FileNotFoundError:
        return {}, f"峰值记录 {base} 还没有（从没量过）"
    except OSError as error:
        return None, f"峰值记录 {base} 读不了（{type(error).__name__}：{error}）；{howto}"
    unbounded = break_is(environment, "worker-peaks-window-unbounded")
    readings = {}
    for package_name in package_names:
        package_directory = os.path.join(base, package_name)
        if package_name != WORKER_PEAK_WORKSPACE_SCOPE and not WORKER_PEAK_PACKAGE_NAME.fullmatch(package_name):
            return None, f"峰值记录 {base} 读不了：{package_directory} 的名字不是包名；{howto}"
        try:
            jobs_names = sorted(os.listdir(package_directory))
        except OSError as error:
            return None, f"峰值记录 {base} 读不了：{package_directory}（{type(error).__name__}：{error}）；{howto}"
        for jobs_name in jobs_names:
            directory = os.path.join(package_directory, jobs_name)
            if not re.fullmatch(r"[1-9][0-9]*", jobs_name):
                return None, f"峰值记录 {base} 读不了：{directory} 的名字不是编译并行度（正整数）；{howto}"
            try:
                names = sorted(name for name in os.listdir(directory) if not name.startswith(".partial-"))
            except OSError as error:
                return None, f"峰值记录 {base} 读不了：{directory}（{type(error).__name__}：{error}）；{howto}"
            records = []
            for name in names:
                path = os.path.join(directory, name)
                if not WORKER_PEAK_RECORD_NAME.fullmatch(name):
                    return None, f"峰值记录 {base} 读不了：{path} 的名字不是「<时刻纳秒 20 位>-<进程号>-<8 位随机字符>」；{howto}"
                try:
                    with open(path, encoding="utf-8", errors="replace") as handle:
                        text = handle.read()
                except FileNotFoundError:
                    continue   # 别的一趟读的时候刚删掉的更旧那几份
                except OSError as error:
                    return None, f"峰值记录 {base} 读不了：{path}（{type(error).__name__}：{error}）；{howto}"
                if not re.fullmatch(r"[1-9][0-9]*\n", text):
                    return None, f"峰值记录 {base} 读不了：{path} 写着「{text.strip()[:40]}」，不是一个正整数字节数；{howto}"
                records.append((name, int(text)))
            window = records if unbounded else records[-WORKER_PEAKS_RECENT:]
            if not unbounded:
                for name, _value in records[:-WORKER_PEAKS_RECENT]:
                    with contextlib.suppress(OSError):
                        os.unlink(os.path.join(directory, name))
            if window:
                readings.setdefault(package_name, {})[int(jobs_name)] = (max(value for _name, value in window), len(window))
    if break_is(environment, "worker-peaks-unkeyed-by-package") and readings:
        merged = {}
        for by_jobs in readings.values():
            for jobs, (value, count) in by_jobs.items():
                merged[jobs] = (max(value, merged.get(jobs, (0, 0))[0]), count + merged.get(jobs, (0, 0))[1])
        readings = {package: dict(merged) for package in readings}
    if break_is(environment, "worker-peaks-unkeyed-by-jobs") and readings:
        readings = {package: {jobs: (max(value for value, _count in by_jobs.values()), sum(count for _value, count in by_jobs.values())) for jobs in by_jobs}
                    for package, by_jobs in readings.items()}
    listed = "、".join(f"包 {package} 编译并行度 {jobs} 最新 {count} 份里最大 {format_size(value)}"
                      for package, by_jobs in sorted(readings.items()) for jobs, (value, count) in sorted(by_jobs.items()))
    return readings, f"峰值记录 {base}：" + (listed or "一份都没有（从没量过）")


def worker_peak_for_jobs(peaks, packages, jobs, cap, environment):
    """编译并行度 jobs 下、这一趟要判的 packages 这几个包里每个工作进程的峰值（文件头「内存上限」的「怎么除」）：(字节, 取法)，取各包里最大的。
    peaks 是 measured_worker_peaks 交的（None 是整份读不了）。每个包：① jobs 有实测取它；② 没有而更高的编译并行度有，借比它大的里最小那一个的；
    测试目标那一级的名字自己在 jobs 与更高的都没有实测、包有记录的，取同包别的测试目标那一级的中位数或起步值（unmeasured_target_peak），不借包；
    ③ 都没有（含这个包从没量过）、或整份读不了，取保守值 = 内存上限 cap 本身（只开 1 个）。
    弄坏开关 unmeasured-worker-peak-guessed（③ 里没有实测那一种）、unreadable-worker-peaks-guessed（读不了那一种）：写死的默认 3G；
    unmeasured-target-borrows-package-peak：没有实测的测试目标借它的包。"""
    if peaks is None:
        if break_is(environment, "unreadable-worker-peaks-guessed"):
            return bytes_of_size("3G"), "峰值记录读不了，照写死的默认 3G（弄坏开关 unreadable-worker-peaks-guessed）"
        return cap, "峰值记录读不了，取保守值：按一个工作进程占满整份内存上限算"
    per_package = []
    for key in packages:
        # 测试目标那一级的名字（<包>--…）自己在 jobs 或更高的编译并行度没有实测：取同包别的测试目标那一级实测的中位数，一份都没有取起步值（unmeasured_target_peak）；
        #   包从没量过的照下面走（③ 或探针）。弄坏开关 unmeasured-target-borrows-package-peak：借它的包（整个包最新几份里最大的）
        names = peak_record_names(key)
        if (len(names) > 1 and not any(measured >= jobs for measured in peaks.get(key, {}))
                and not break_is(environment, "unmeasured-target-borrows-package-peak")):
            estimate = unmeasured_target_peak(peaks, key, jobs)
            if estimate is not None:
                per_package.append(estimate)
                continue
        package = next((name for name in names if any(measured >= jobs for measured in peaks.get(name, {}))), names[-1])
        by_jobs = peaks.get(package, {})
        label = peak_key_label(package)
        if jobs in by_jobs:
            value, count = by_jobs[jobs]
            per_package.append((value, f"包 {label} 最新 {count} 份实测里最大的"))
            continue
        higher = [measured for measured in by_jobs if measured > jobs]
        if higher:
            value, count = by_jobs[min(higher)]
            per_package.append((value, f"包 {label} 没有实测，借编译并行度 {min(higher)} 最新 {count} 份实测里最大的（推的：编译并行度越高，每个工作进程的峰值不会越低）"))
            continue
        if break_is(environment, "unmeasured-worker-peak-guessed"):
            per_package.append((bytes_of_size("3G"), f"包 {label} 没有实测，照写死的默认 3G（弄坏开关 unmeasured-worker-peak-guessed）"))
            continue
        per_package.append((cap, f"包 {label} " + ("从没量过" if not by_jobs else "只有更低编译并行度的实测、不借") + "，取保守值：按一个工作进程占满整份内存上限算"))
    value, basis = max(per_package, key=lambda entry: entry[0])
    return value, (f"这一趟涉及 {len(packages)} 个包，取其中最大的：{basis}" if len(packages) > 1 else basis)


def peak_at_jobs(by_jobs, jobs):
    """一个记录名的 {编译并行度: (字节, 份数)} 在编译并行度 jobs 下取哪一档（worker_peak_for_jobs 的 ①②）：(字节, 份数, 取的那一档) 或 None。
    jobs 有实测取它；没有而更高的有，借比它大的里最小的那一档（推的：编译并行度越高，每个工作进程的峰值不会越低）；只有更低的交 None。"""
    usable = [measured for measured in by_jobs if measured >= jobs]
    if not usable:
        return None
    chosen = min(usable)
    return by_jobs[chosen] + (chosen,)


def worker_peak_starting_bytes():
    """测试目标那一级、包里一份目标那一级的实测都没有时的起步值（文件头「内存上限」的「怎么除」）：一个 rustc 进程量到的峰值。
    口径与出处：records/2026-11-14-门禁静态审核.md 变异表-8 那一行「rustc 最高只 6.7 GiB」（量的：共享根运行记录里验证包每个工作进程 26.5 GiB 那一份是测试二进制，
    编译那一段 rustc 最高 6.7 GiB）。峰值记录不分只编不跑那一类，取不到包自己只编不跑的实测，所以取这个数。
    写在函数体里、不做模块级常量。估小了撞上限由统一任务服务乘 1.5 重排，这里不另兜。"""
    return 67 * (1 << 30) // 10


def unmeasured_target_peak(peaks, key, jobs):
    """测试目标那一级的记录名 key（<包>--…）自己在 jobs 与更高的编译并行度都没有实测时，每个工作进程的峰值（文件头「内存上限」的「怎么除」）：(字节, 取法) 或 None。
    · 同包别的测试目标那一级（worker-peaks/ 下 <包>--… 那几份）在 jobs 下取得到的（peak_at_jobs），取它们的中位数（个数是偶数时取中间两个里大的）；
    · 一份都取不到而包自己有记录（哪个编译并行度都算）：取起步值 worker_peak_starting_bytes，包在 jobs 下取得到、比起步值小时取包的（整个包的最大也比起步值小）；
    · 包从没量过（记录里这个包一份都没有）交 None：照包算（worker_peak_for_jobs 的 ③，或这一趟写峰值记录时由探针收住）。
    不借整个包最新几份里最大的：那是包里最重的一个测试目标整目标跑出来的，罩住包里全部目标，借它会让没量过的轻目标也只开 1 个。"""
    package = key.split("--", 1)[0]
    siblings = []
    for name in sorted(peaks):
        if name != key and name.startswith(package + "--"):
            found = peak_at_jobs(peaks[name], jobs)
            if found is not None:
                siblings.append((found[0], name))
    if siblings:
        values = sorted(value for value, _name in siblings)
        median = values[len(values) // 2]
        return median, (f"测试目标 {key} 在编译并行度 {jobs} 与更高的都没有实测，取包 {package} 里 {len(values)} 个测试目标那一级实测的中位数"
                        f"（{'、'.join(f'{name} {format_size(value)}' for value, name in sorted(siblings))}）")
    if not peaks.get(package):
        return None
    starting = worker_peak_starting_bytes()
    found = peak_at_jobs(peaks[package], jobs)
    if found is not None and found[0] < starting:
        return found[0], (f"测试目标 {key} 没有实测、包 {package} 里一份测试目标那一级的实测都没有，起步值 {format_size(starting)} 比包 {package} 编译并行度 {found[2]} "
                          f"最新 {found[1]} 份实测里最大的 {format_size(found[0])} 大，取包的")
    return starting, (f"测试目标 {key} 没有实测、包 {package} 里一份测试目标那一级的实测都没有，取起步值 {format_size(starting)}"
                      f"（一个 rustc 进程量到的峰值，worker_peak_starting_bytes）")


def new_packages_of(peaks, packages):
    """packages 里从没量过的包（文件头「内存上限」「怎么除」的 ④）：峰值记录读得了、这个包下一份记录都没有（哪个编译并行度都没有）。
    peaks 是 None（整份读不了）交空：读不了不算从没量过，照 ③ 取保守值。"""
    if peaks is None:
        return ()
    return tuple(package for package in packages if not any(peaks.get(name) for name in peak_record_names(package)))


def unmeasured_packages_at(peaks, packages, jobs):
    """packages 里在编译并行度 jobs 与更高的都没有实测的包（worker_peak_for_jobs 落到 ③ 的那几个；文件头「内存上限」的「探针」）。peaks 是 None（整份读不了）全算。
    测试目标那一级的名字（<包>--…）自己与它的包都没有才算（peak_record_names）。"""
    if peaks is None:
        return tuple(packages)
    return tuple(package for package in packages
                 if not any(measured >= jobs for name in peak_record_names(package) for measured in peaks.get(name, {})))


def memory_bound_for(environment, ceiling, thread_budget, records_directory, packages, cgroup_listing=CGROUP_LISTING, cgroup_root=CGROUP_ROOT, configuration_path=None):
    """按内存上限能开几个工作进程（文件头「内存上限」）：(个数或 None, 头一行要写的话)。这一趟写峰值记录、从没量过的包靠探针收住时的那一种，
    取法在 memory_bound_probing（probing 是真，按这一条拿到的上限除）。"""
    return memory_bound_probing(environment, ceiling, thread_budget, records_directory, packages, True, cgroup_listing, cgroup_root, configuration_path, None)[:2]


def host_memory_capacity(environment):
    """这一台能给的内存（文件头「内存上限」）：(字节或 None, 说明)；转调 scripts/hosts/task_memory.py 的 host_slice_cap_bytes（函数体里 import，同 task_memory_cap_bytes）。"""
    import task_memory
    return task_memory.host_slice_cap_bytes(environment)


def memory_bound_probing(environment, ceiling, thread_budget, records_directory, packages, probing, cgroup_listing, cgroup_root, configuration_path, capacity):
    """按内存能开几个工作进程（文件头「内存上限」）：(个数或 None, 头一行要写的话, 这一条拿到的上限的字节或 None, 开 n 个要多少内存的函数或 None)。
    capacity 是这一台能给的内存 (字节或 None, 说明)（run_stage 交 host_memory_capacity 的）：字节不是 None 就按它除，这一条拿到的上限（task_memory_cap_bytes）另交回当硬顶；
    capacity 是 None 或里面的字节是 None，按这一条拿到的上限除。第三项是探针与撞内存上限比的那个数：这一条拿到的上限，取不到时是除的那个数。
    第四项 requirement(n) 交 (n 个工作进程在那个编译并行度下要的字节, 怎么算的)，峰值是保守值（没有实测）时交 (None, 原因)；要判的包都从没量过时第四项是 None。
    ceiling 是另外三项的小者，thread_budget 是整台线程数（compile_thread_budget 交的，cargo_jobs_for 用），
    records_directory 是运行记录目录（峰值记录在它的 worker-peaks/ 下，measured_worker_peaks 读），packages 是这一趟要判的行点名的包（worker_peak_packages 交的）。
    probing 是这一趟写不写峰值记录（worker_peaks_recorded_for）：写的时候从没量过的包（new_packages_of）不进这一项，由派活时的探针收住（文件头「探针」）；
    要判的包都从没量过交 ceiling。不写的时候探针量不到东西，从没量过的包照保守值进这一项。
    n 从 1 试到 ceiling：编译并行度 j = cargo_jobs_for(n)，每个工作进程的峰值 P = worker_peak_for_jobs(有记录的包, j)（各包里最大的），满足 n × P ≤ 上限的最大 n 是这一项；
    到了 ceiling 还满足报 ⌊上限 ÷ P⌋（编译并行度照 ceiling 算，不与另外三项并列成「卡在」）；一个都不满足交 0（调用方按 1 个开）。
    没有内存上限（memory.max 是 max、读不到）交 None 与原因，这一项不收、照线程数开。
    弄坏开关 memory-cap-ignored：不看内存上限；jobs-feedback-ignored：每个 n 的编译并行度都照 ceiling 算；
    new-package-run-single-worker：从没量过的包照保守值进这一项（整趟只开 1 个）。"""
    if break_is(environment, "memory-cap-ignored"):
        return None, "弄坏开关 memory-cap-ignored：不看内存上限", None, None
    hard_cap, hard_source = task_memory_cap_bytes(environment, cgroup_listing, cgroup_root, configuration_path)
    if capacity is not None and capacity[0] is not None:
        cap, cap_source = capacity
        cap_label = limit_word = "这一台能给的内存"
        hard_text = (f"；这一条拿到的上限 {format_size(hard_cap)}（{hard_source}）当硬顶，开的工作进程 × 峰值装不下时往下减到装得下的个数（至多减到 1 个），1 个也装不下照开 1 个起跑、真撞了由统一任务服务判" if hard_cap is not None
                     else f"；这一条拿到的上限没有（{hard_source}），不另比")
    else:
        cap, cap_source = hard_cap, hard_source
        cap_label, limit_word = "内存上限", "上限"
        hard_text = (f"；这一台能给的内存取不到（{capacity[1]}），按这一条拿到的上限除" if capacity is not None else "")
    if cap is None:
        return None, cap_source + hard_text, None, None
    admission_cap = hard_cap if hard_cap is not None else cap
    peaks, peaks_source = measured_worker_peaks(records_directory, environment)
    ceiling = max(1, ceiling)
    new_packages = new_packages_of(peaks, packages) if probing and not break_is(environment, "new-package-run-single-worker") else ()
    recorded_packages = tuple(package for package in packages if package not in new_packages)
    probe_text = (f"；包 {'、'.join(map(peak_key_label, new_packages))} 从没量过：不进这一项，派活时它的格单独跑（探针：同时只判这一格），量到之后同时在跑的格数按 ⌊上限 ÷ 量到的峰值⌋ 放开"
                  if new_packages else "")
    if not recorded_packages:
        jobs = cargo_jobs_for(environment, ceiling, thread_budget)
        return ceiling, (f"{cap_label} {format_size(cap)}（{cap_source}）：这一趟要判的包都从没量过，这一项交天花板 {ceiling} 个工作进程（编译并行度 {jobs}）"
                         f"{probe_text}{hard_text}；{peaks_source}"), admission_cap, None

    def planned(count):
        jobs = cargo_jobs_for(environment, ceiling if break_is(environment, "jobs-feedback-ignored") else count, thread_budget)
        return (jobs,) + worker_peak_for_jobs(peaks, recorded_packages, jobs, cap, environment)

    def requirement(count):
        jobs, peak, basis = planned(count)
        if "取保守值" in basis:   # 没有实测、取的是除的那个数本身（worker_peak_for_jobs 的 ③）：不是量到的峰值，不拿它比上限
            return None, f"编译并行度 {jobs} 下没有实测（{basis}）"
        return count * peak, f"{count} × 每个工作进程的峰值 {format_size(peak)}（编译并行度 {jobs} 下：{basis}）= {format_size(count * peak)}"

    best = max((count for count in range(1, ceiling + 1) if count * planned(count)[1] <= cap), default=0)
    jobs, peak, basis = planned(max(1, best))
    count = best if best < ceiling else cap // peak
    text = (f"{cap_label} {format_size(cap)}（{cap_source}）÷ 每个工作进程的峰值 {format_size(peak)}（编译并行度 {jobs} 下：{basis}）= 同时开 {count} 个工作进程"
            + (f"（{best} × {format_size(peak)} = {format_size(best * peak)}，不超过{limit_word}）" if best else f"（1 个就要 {format_size(peak)}，超了{limit_word}）"))
    if 1 <= best < ceiling:
        next_jobs, next_peak, next_basis = planned(best + 1)
        text += (f"；再多 1 个：编译并行度 {next_jobs} 下每个 {format_size(next_peak)}（{next_basis}），"
                 f"{best + 1} × {format_size(next_peak)} = {format_size((best + 1) * next_peak)}，超了{limit_word}")
    return count, f"{text}{probe_text}{hard_text}；{peaks_source}", admission_cap, requirement


def cell_peak_packages(kind, payload):
    """进程池里一格活点名的包（文件头「内存上限」的「探针」）：与这一格判的时候 worker_peak_scope 设给采样线程的是同一份（baseline_job、mutation_job、coverage_job）。"""
    if kind == "baseline":
        return worker_peak_keys([row.arguments for row in payload[1]])
    if kind == "mutation":
        return worker_peak_keys([payload.arguments])
    return worker_peak_keys([payload["arguments"]])


def new_memory_admission(context):
    """run_pool 派活时的探针状态（文件头「内存上限」的「探针」）：context["memory_admission"]（run_stage 设：cap、jobs、packages）是 None 交 None（这一条不收）；
    不然读一次峰值记录，交 dict(cap, jobs, packages, records_directory, peaks, unmeasured, held)：unmeasured 是 packages 里在 jobs 下还没有实测的包，held 是上一次压住的那一格。"""
    settings = context.get("memory_admission")
    if not settings:
        return None
    peaks, _source = measured_worker_peaks(context["run_records_directory"], context["environment"])
    return dict(settings, records_directory=context["run_records_directory"], peaks=peaks,
                unmeasured=set(unmeasured_packages_at(peaks, settings["packages"], settings["jobs"])), held=None)


def memory_admits(admission, running, packages, environment):
    """再派一格点名 packages 的活放不放得下（文件头「内存上限」的「探针」）：running 是在跑的各格点名的包（每格一个元组）。
    admission 是 None、或一格都没在跑，照派；不然在跑的连同这一格点名的包在 admission["jobs"] 下的峰值 P 照 worker_peak_for_jobs 取（没有实测的取上限本身），
    (在跑的格数 + 1) × P ≤ 上限才派。弄坏开关 unmeasured-package-not-probed-alone：没有实测的包不算进 P，一个包都不剩就照派。
    跑着加开过的（admission 里有 peak_seen，run_pool 的 grow_pool 放的，文件头「内存上限」的「跑着加开」）：涉及的包在 admission["jobs"] 下都有实测时，
    P 取这一趟各工作进程到此刻实测峰值的最大（peak_seen 此刻的值），与加开时的算法同一个口径；有没实测的包照上面取。"""
    if admission is None or not running:
        return True
    involved = tuple(sorted(set(packages).union(*running)))
    if break_is(environment, "unmeasured-package-not-probed-alone"):
        unmeasured = unmeasured_packages_at(admission["peaks"], involved, admission["jobs"])
        involved = tuple(package for package in involved if package not in unmeasured)
        if not involved:
            return True
    seen = admission.get("peak_seen")
    if seen is not None and seen.value > 0 and not unmeasured_packages_at(admission["peaks"], involved, admission["jobs"]):
        return (len(running) + 1) * seen.value <= admission["cap"]
    peak, _basis = worker_peak_for_jobs(admission["peaks"], involved, admission["jobs"], admission["cap"], environment)
    return (len(running) + 1) * peak <= admission["cap"]


def refresh_memory_admission(admission, environment):
    """收完一格之后（文件头「内存上限」的「探针」）：这一趟还有没有实测的包时重读峰值记录，换上新读到的；新量到的包各交一行说明（stderr 要打的）。都量到了不读、交空。"""
    if admission is None or not admission["unmeasured"]:
        return []
    peaks, _source = measured_worker_peaks(admission["records_directory"], environment)
    still = set(unmeasured_packages_at(peaks, admission["packages"], admission["jobs"]))
    lines = []
    for package in sorted(admission["unmeasured"] - still):
        peak, basis = worker_peak_for_jobs(peaks, (package,), admission["jobs"], admission["cap"], environment)
        lines.append(f"  … 包 {peak_key_label(package)} 量到了（探针）：编译并行度 {admission['jobs']} 下每个工作进程 {format_size(peak)}（{basis}），"
                     f"只判它的格同时至多 {admission['cap'] // max(1, peak)} 格（内存上限 {format_size(admission['cap'])} ÷ 峰值）")
    admission.update(peaks=peaks, unmeasured=still)
    return lines


def worker_threads_bound(environment):
    """线程数那一项（worker_count_for 与分片的内存账 mutation-shard-run.sh 同用）：(项名, 个数, 头一行要写的话)。
    GATE_MUTATION_WORKERS 给了照它（写的不是整数时 int 抛 ValueError）；没给取每台默认重活线程数
    （scripts/hosts/heavy_threads.py 的 default_heavy_threads：SINGLEFS_THREAD_CAP 或 .claude/heavy-thread-default；取不到抛 HeavyThreadDefaultError）。"""
    registered = None if (given := environment.get("GATE_MUTATION_WORKERS", "").strip()) else int(default_heavy_threads(environment))
    if given:
        return "GATE_MUTATION_WORKERS", int(given), f"GATE_MUTATION_WORKERS 给 {int(given)} 个"
    return "每台默认重活线程数", registered, f"每台默认重活线程数 {registered}（scripts/hosts/heavy_threads.py：SINGLEFS_THREAD_CAP 或 .claude/heavy-thread-default）"


def worker_count_for(row_count, environment, disk=None, memory=None):
    """(开几个工作进程, 头一行要报的话)。四项取最小、至少 1：每台默认重活线程数
    （GATE_MUTATION_WORKERS 顶掉它；取不到抛 HeavyThreadDefaultError）、这一趟要现跑的条数、磁盘余量放得下几个工作进程的编译目录（disk 给了才有这一项）、
    内存上限 ÷ 每个工作进程的峰值（memory 给了才有这一项）。
    disk 与 memory 都是函数：前面几项取最小的那个数当天花板（ceiling）传给它，它交回 (放得下几个或 None, 头一行要写的话)，None 是没有这一项，头一行写明原因
    （disk_bound_for，文件头「磁盘余量」；memory_bound_for，文件头「内存上限」）。内存那一项排在最后：它按最后开的进程数算编译并行度，天花板要含磁盘那一项。
    一条变异的挂钟 = 重编译那几个 crate + 跑点名的那个测试，测试那一段是单线程的，所以开得比核数 ÷ 4 多，再用 GATE_MUTATION_CARGO_JOBS
    把每个 cargo 的编译并行度收住。"""
    bounds = []
    if break_is(environment, "workers-by-processor-half") and not environment.get("GATE_MUTATION_WORKERS", "").strip():
        processor_count = os.cpu_count() or 4
        by_processor = max(1, min(16, processor_count // 2))
        bounds.append(("核数", by_processor, f"核数 {processor_count} 的一半、至多 16，给 {by_processor} 个（弄坏开关 workers-by-processor-half）"))
    else:
        bounds.append(worker_threads_bound(environment))   # 取不到抛 HeavyThreadDefaultError，run_stage 判红退 2
    # 要判的单位（文件头「工作进程数」）：工作进程数不超过它，不照拿到的上限开满；弄坏开关 workers-beyond-rows 下不收这一项
    if not break_is(environment, "workers-beyond-rows"):
        bounds.append(("这一趟要现跑的条数", row_count, f"这一趟要现跑 {row_count} 条（没有作数的抓到记录的；基线的批与覆盖跟着这几行走，不另算工作进程）"))
    disk_unread = None
    if disk is not None:
        disk_count, disk_text = disk(max(1, min(count for _name, count, _text in bounds)))
        if disk_count is None:
            disk_unread = disk_text
        else:
            bounds.append(("磁盘余量", disk_count, disk_text))
    memory_unlimited = None
    if memory is not None:
        memory_count, memory_text = memory(max(1, min(count for _name, count, _text in bounds)))
        if memory_count is None:
            memory_unlimited = memory_text
        else:
            bounds.append(("内存上限", memory_count, memory_text))
    smallest = min(count for _name, count, _text in bounds)
    binding = "、".join(f"「{name}」" for name, count, _text in bounds if count == smallest)
    floor_note = "（那一项不足 1，按 1 个开）" if smallest < 1 else ""
    worker_count = max(1, smallest)
    head_line = (f"  … 这一轮开 {worker_count} 个工作进程（各项取最小、至少 1）：{'；'.join(text for _name, _count, text in bounds)}。"
                 f"卡在{binding}这一项{floor_note}")
    if memory_unlimited is not None:
        head_line += f"；没有内存上限这一项（{memory_unlimited}），工作进程数不按内存收"
    if disk_unread is not None:
        head_line += f"；磁盘余量没读到（{disk_unread}），工作进程数不按磁盘收"
    return worker_count, head_line


def grown_worker_count(growth, current, seen_bytes):
    """跑着加开（文件头「内存上限」的「跑着加开」）：此刻开着 current 个工作进程，这一趟各工作进程到此刻实测峰值的最大是 seen_bytes，能加到几个：(个数或 None, 说明)。
    growth 是 run_stage 交的 dict(cap=这一条拿到的上限, parent_bytes=起这一道的父进程自己, ceiling=线程数、条数、磁盘余量三项的小者, thread_budget=整台线程数)。
    取满足「n × seen_bytes + 父进程自己 ≤ 上限」、不超过 ceiling 的最大 n；还没量到（seen_bytes 不是正数）或算出来不比 current 多交 None。"""
    if seen_bytes <= 0:
        return None, "这一趟还没量到工作进程的峰值"
    room = growth["cap"] - growth["parent_bytes"]
    count = min(growth["ceiling"], room // seen_bytes) if room > 0 else 0
    text = (f"这一趟各工作进程到此刻实测峰值至多 {format_size(seen_bytes)}，{count} × {format_size(seen_bytes)} + 起这一道的父进程自己 {format_size(growth['parent_bytes'])}"
            f" = {format_size(count * seen_bytes + growth['parent_bytes'])}，不超过这一条拿到的上限 {format_size(growth['cap'])}"
            f"（上界取线程数、条数、磁盘余量三项的小者 {growth['ceiling']} 个）")
    return (count, text) if count > current else (None, text)


def selftest_worker_count_cells(selftest, work):
    """工作进程数（文件头「工作进程数」）：GATE_MUTATION_WORKERS（顶掉每台默认重活线程数那一项）与要现跑的条数取最小，这几格直接调 worker_count_for 不给内存项（内存那几格在 selftest_memory_cells），头一行一个字都不提内存；
    没给 GATE_MUTATION_WORKERS 时取每台默认重活线程数（SINGLEFS_THREAD_CAP=7 取 7），取不到（SINGLEFS_THREAD_CAP=abc）抛 HeavyThreadDefaultError、整道退 2。
    编译并行度的整台线程数（compile_thread_budget）那一格接在后面。
    弄坏开关 workers-by-processor-half 让「取每台默认重活线程数」「取不到每台默认重活线程数」与编译并行度的整台线程数三格判错（取核数一半至多 16、不认登记值，也不拒；整台线程数取核数）。"""
    sixteen = dict(selftest.base_environment, GATE_MUTATION_WORKERS="16")
    many, many_head = worker_count_for(20, sixteen)
    few, few_head = worker_count_for(5, sixteen)
    selftest.check("工作进程数取 GATE_MUTATION_WORKERS 与要现跑的条数的小者，没给内存项时头一行不提内存（20 条开 16 个、5 条开 5 个）",
                   many == 16 and "卡在「GATE_MUTATION_WORKERS」这一项" in many_head and few == 5 and "卡在「这一趟要现跑的条数」这一项" in few_head
                   and "内存" not in many_head + few_head, f"{many}：{many_head}；{few}：{few_head}")
    registered_environment = {key: value for key, value in selftest.base_environment.items() if key != "GATE_MUTATION_WORKERS"}
    registered_environment["SINGLEFS_THREAD_CAP"] = "7"
    by_registry, registry_head = worker_count_for(20, registered_environment)
    selftest.check("没给 GATE_MUTATION_WORKERS：工作进程数取每台默认重活线程数（SINGLEFS_THREAD_CAP=7、20 条开 7 个，卡在「每台默认重活线程数」这一项）",
                   by_registry == 7 and "卡在「每台默认重活线程数」这一项" in registry_head, f"{by_registry}：{registry_head}")
    repository = os.path.join(work, "workers-unregistered")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS[:1])
    refused = selftest.run(repository, GATE_MUTATION_WORKERS="", GATE_MUTATION_KEY_WORKERS="2", SINGLEFS_THREAD_CAP="abc")
    selftest.check("没给 GATE_MUTATION_WORKERS、取不到每台默认重活线程数（SINGLEFS_THREAD_CAP=abc）：整道退 2、报「取不到每台默认重活线程数」，一次 cargo test 都不起",
                   refused[0] == 2 and "取不到每台默认重活线程数" in refused[1] + refused[2] and not plain_invocations(refused[3]),
                   f"退 {refused[0]}；cargo {len(refused[3])} 次；{(refused[1] + refused[2])[-600:]}")
    # 编译并行度的整台线程数：没给 GATE_MUTATION_CARGO_JOBS 取每台默认重活线程数（SINGLEFS_THREAD_CAP=7、两个工作进程各 3 条），给了不读登记位，取不到又没给就抛
    budget = compile_thread_budget(registered_environment)
    unreadable_environment = dict(registered_environment, SINGLEFS_THREAD_CAP="abc")
    pinned_budget = compile_thread_budget(dict(unreadable_environment, GATE_MUTATION_CARGO_JOBS="5"))
    try:
        compile_thread_budget(unreadable_environment)
        unreadable_refused = False
    except HeavyThreadDefaultError:
        unreadable_refused = True
    selftest.check("编译并行度的整台线程数取每台默认重活线程数：没给 GATE_MUTATION_CARGO_JOBS 时 SINGLEFS_THREAD_CAP=7 取 7（两个工作进程各 3 条），给了不读登记位，取不到又没给就拒",
                   budget == 7 and cargo_jobs_for(registered_environment, 2, budget) == 3 and pinned_budget == 1 and unreadable_refused,
                   f"整台线程数 {budget}、两个工作进程各 {cargo_jobs_for(registered_environment, 2, budget)} 条；给了 GATE_MUTATION_CARGO_JOBS 时 {pinned_budget}；取不到时{'拒了' if unreadable_refused else '没拒'}")


def selftest_memory_cells(selftest, work):
    """开几个工作进程按内存上限收（文件头「内存上限」）：内存上限的取法（服务交的变量、多机配置、自己 scope 的 memory.max 与它当硬顶）与除出来的个数，
    同线程数（GATE_MUTATION_WORKERS 顶掉登记值）、要现跑的条数取最小、至少 1。每个工作进程的峰值在这几格固定：编译并行度照 GATE_MUTATION_CARGO_JOBS=4 固定，
    临时运行记录目录里包 tiny 在编译并行度 4 下一份 4 GiB 的实测，要判的行点名的包就是 tiny，个数就是 ⌊上限 ÷ 4 GiB⌋；量、记、读、除的那几格在 selftest_worker_peak_cells。
    直接调 worker_count_for 与 memory_bound_for，/proc/self/cgroup 与 /sys/fs/cgroup 换成临时目录里造的假文件：
    ① 服务交的 TASK_SERVER_MEMORY_CAP_BYTES=20 GiB（假多机配置的 SLICE_MEMORY_CAP_BY_HOST 这一台那一项写 50G、假 memory.max 写 64 GiB 摆在旁边）、线程数 16、20 条：开 5 个，
      头一行写明上限、每个工作进程的峰值、个数与合计、卡在「内存上限」（假 memory.max 写 64 GiB 不比它小，不收窄：服务交的变量先于多机配置）；
    多机配置两格：无服务变量、假多机配置的 SLICE_MEMORY_CAP_BY_HOST 这一台那一项写 24G、另一台那一项写 48G（整值带引号）、假 memory.max 64 GiB 在旁边：取这一台那一项，开 6 个；
    这一台那一项写 lots：判法整份判不过、认不出这一台，头一行点名、改读 memory.max（8 GiB）开 2 个；
    多机配置照判法写全键（判法认这一台要整份判得过）：主机数组写 127.0.0.1（每一台的网卡上都有，是这一台）与文档用的地址段 192.0.2.70；
    ② 上限 100 GiB 放得下 25 个，线程数 16 更小：开 16 个，卡在 GATE_MUTATION_WORKERS；③ 上限 1 GiB 一个都放不下：按 1 个开，头一行写「那一项不足 1」；
    ⑤ 没有服务交的变量、假 memory.max 写 8589934592：开 2 个，头一行写 memory.max 的路径；写 max：不收，照线程数开 16 个，头一行写「是 max」；假 /proc/self/cgroup 读不到：同样不收；
    ⑥ TASK_SERVER_MEMORY_CAP_BYTES 写成 abc：改读 memory.max（8 GiB 开 2 个），头一行点名写法不对。再起一趟 run_stage：线程数 16，上限 8 GiB、每个 4 GiB，开 2 个、卡在「内存上限」，
    收尾那一行也写个数。⑧ 再起一趟 run_stage：上限 2 GiB、每个 4 GiB，一个都放不下：照开 1 个起跑、退 0，头一行写估值与上限、「照开 1 个起跑、不自己退 250」，
    stdout 没有「按撞内存上限退出」，假 cargo 起了。
    run_stage 那几趟的峰值记录同时放在包 tiny 与测试目标那一级 tiny--lib 下（这几行是 -p tiny --lib：估值先看 tiny--lib，弄坏开关 worker-peaks-unkeyed-by-target 下看 tiny）。
    按这一台能给的内存除（host_slice_cap_bytes 交的那一种）：多机配置这一台那一项 50G、服务交的 20 GiB、假 memory.max 64 GiB：开 12 个，交回的硬顶是 20 GiB、开 12 个要 48 GiB。
    弄坏开关 memory-cap-ignored（不看内存上限，照线程数开满）下除常量核对那一格之外的二十一格全部判错（含 ⑨ 的四格，见下）；memory-cap-short-self-exit（一个都装不下就自己退 250）下 ⑧ 与 ⑨ 的「1 个也不够」两格判错。
    memory.max 是硬顶（scope 真上限 20736M，多机配置写 50G）：⑦ 多机配置 50G（无服务变量）、memory.max 20 GiB：按 20 GiB 开 5 个，头一行写各档读到的值与取 memory.max 的原因；
    服务交的变量 50 GiB、memory.max 20 GiB：同样按 20 GiB 开 5 个；服务交的变量 50 GiB、memory.max 写 max：照 50 GiB 开 12 个（没有硬顶可比）；服务交的变量 50 GiB、/proc/self/cgroup 读不到：照 50 GiB 开 12 个；
    再起一趟 run_stage：服务交的变量 50 GiB、经 GATE_MUTATION_CGROUP_LISTING / GATE_MUTATION_CGROUP_ROOT 指到 memory.max 8 GiB：开 2 个。
    弄坏开关 memory-max-not-hard-cap（取到变量或多机配置的值就不看 memory.max）下 ⑦ 的三格（多机配置、服务交的变量、run_stage）判错，写 max 与读不到那两格不红。
    ⑨ 照这一次服务给的上限开工作进程（四趟 run_stage，多机配置这一台那一项 48G、每个工作进程 10 GiB、4 行，按这一台能给的内存要开 4 个）：服务交的上限 48 GiB 照开满 4 个；
    15 GiB 只装得下 1 个：减到 1 个开、照跑、退 0；35 GiB 装得下 3 个：减到 3 个开、照跑、退 0；
    8 GiB 1 个也装不下：照开 1 个起跑、退 0，头一行写减到 1 个也装不下与「照开 1 个起跑」，假 cargo 起了。
    弄坏开关 memory-cap-exits-before-fewer-workers（不往下找装得下的个数，照一个都装不下办）下「上限只够 1 个工作进程」「上限够 3 个工作进程」两格判错；
    memory-cap-ignored 下 ⑨ 的四格都判错：除「够开满」之外的三格判错在没有减个数的那一句；「够开满」开的个数不变，判错在头一行没有「这一台能给的内存」那一项。"""
    import task_memory
    skew = "/skewed" if break_is(selftest.base_environment, "task-memory-constants-skewed") else ""
    selftest.check("内存上限的常量与大小写法：本文件留着的模块级赋值与 scripts/hosts/task_memory.py 里用的同一个值",
                   (TASK_MEMORY_CAP_VARIABLE, SLICE_MEMORY_CAP_KEY, CGROUP_LISTING, CGROUP_ROOT + skew, SIZE_TEXT.pattern, SIZE_UNIT_BYTES)
                   == (task_memory.TASK_MEMORY_CAP_VARIABLE, task_memory.SLICE_MEMORY_CAP_KEY, task_memory.CGROUP_LISTING, task_memory.CGROUP_ROOT,
                       task_memory.SIZE_TEXT.pattern, task_memory.SIZE_UNIT_BYTES),
                   "两边的值对不上：改 scripts/hosts/task_memory.py 的同名常量跟着本文件的值")
    gibibyte = 1 << 30
    scope_directory = os.path.join(work, "memory-cgroup", "fake.slice", "job.scope")
    os.makedirs(scope_directory, exist_ok=True)
    listing = os.path.join(work, "memory-cgroup-listing")
    with open(listing, "w", encoding="utf-8") as handle:
        handle.write("12:memory:/legacy\n0::/fake.slice/job.scope\n")
    root = os.path.join(work, "memory-cgroup")
    memory_max = os.path.join(scope_directory, "memory.max")

    def write_memory_max(text):
        with open(memory_max, "w", encoding="utf-8") as handle:
            handle.write(text)

    configuration = os.path.join(work, "memory-config", "multi-host.env")
    os.makedirs(os.path.dirname(configuration), exist_ok=True)

    def write_configuration(text):
        with open(configuration, "w", encoding="utf-8") as handle:
            handle.write(text)

    def write_slice_configuration(slice_items):
        # 判法（scripts/crash/crash-point-replay-shard-configuration-check.sh）整份判得过的多机配置，SLICE_MEMORY_CAP_BY_HOST 照给的写
        write_configuration("HOSTS=127.0.0.1 192.0.2.70\nCARGO_BIN_DIRECTORY_BY_HOST=127.0.0.1=/selftest/this-cargo 192.0.2.70=/selftest/other-cargo\n"
                            "BUILD_ROOT_BY_HOST=127.0.0.1=/selftest/this-builds 192.0.2.70=/selftest/other-builds\n"
                            f"SLICE_MEMORY_CAP_BY_HOST={slice_items}\nQUIESCE_STOP_COMMAND=\nQUIESCE_STOPPED_CHECK_COMMAND=\nQUIESCE_START_COMMAND=\n"
                            f"QUIESCE_STARTED_CHECK_COMMAND=\nENABLE_ACROSS_MACHINES=0\nENABLE_GPU=0\nNFS_SHARED_ROOTS={os.path.dirname(configuration)}\n")

    # 每个工作进程的峰值固定成包 tiny 在编译并行度 4 下一份 4 GiB 的实测（直接调 memory_bound_for 的那几格读这一份，要判的包就是 tiny）
    records = os.path.join(work, "memory-run-records")
    record_worker_peak(records, "tiny", 4, f"{1:020d}-1-selftest", 4 * gibibyte)

    def counted(row_count, environment):
        return worker_count_for(row_count, environment, memory=lambda ceiling: memory_bound_for(environment, ceiling, 16, records, ("tiny",), listing, root, configuration))

    sixteen = {key: value for key, value in selftest.base_environment.items() if key != TASK_MEMORY_CAP_VARIABLE}
    sixteen.update(GATE_MUTATION_WORKERS="16", GATE_MUTATION_CARGO_JOBS="4")
    peak_text = "每个工作进程的峰值 4.0 GiB（编译并行度 4 下：包 tiny 最新 1 份实测里最大的）"
    write_memory_max(str(64 * gibibyte) + "\n")
    write_slice_configuration("127.0.0.1=50G 192.0.2.70=8G")
    served = dict(sixteen, **{TASK_MEMORY_CAP_VARIABLE: str(20 * gibibyte)})
    count, head = counted(20, served)
    selftest.check("服务交的内存上限 20 GiB、每个工作进程的峰值 4 GiB（编译并行度 4 下的实测）、线程数 16、20 条：同时开 5 个工作进程，头一行写上限、每个工作进程的峰值、个数与合计，"
                   "卡在「内存上限」（服务交的变量先于多机配置的 50G；假 memory.max 的 64 GiB 不比它小、不收窄，头一行写各档读到的值）",
                   count == 5 and "卡在「内存上限」这一项" in head and "内存上限 20.0 GiB（服务交的 TASK_SERVER_MEMORY_CAP_BYTES=" in head
                   and peak_text in head and "= 同时开 5 个工作进程（5 × 4.0 GiB = 20.0 GiB，不超过上限）" in head
                   and f"scope 的 memory.max={64 * gibibyte}（64.0 GiB）" in head and "多机配置 SLICE_MEMORY_CAP_BY_HOST 里这一台 127.0.0.1 那一项=50G（50.0 GiB）" in head
                   and "不收窄" in head, f"{count}：{head}")
    # 按这一台能给的内存除（run_stage 交 host_memory_capacity 的那一种）：这一台那一项 50G、服务交的 20 GiB、假 memory.max 64 GiB
    capacity = task_memory.host_slice_cap_bytes(served, configuration)
    count, head, hard, requirement = memory_bound_probing(served, 16, 16, records, ("tiny",), True, listing, root, configuration, capacity)
    needed = requirement(12)[0] if requirement is not None else None
    selftest.check("按这一台能给的内存除：多机配置这一台那一项 50G ÷ 4 GiB 开 12 个（不按服务交的 20 GiB 收），交回的硬顶是服务交的 20 GiB，开 12 个要 48 GiB（run_stage 照它往下减个数）",
                   count == 12 and hard == 20 * gibibyte and needed == 48 * gibibyte and "这一台能给的内存 50.0 GiB（多机配置" in head
                   and "这一条拿到的上限 20.0 GiB（服务交的 TASK_SERVER_MEMORY_CAP_BYTES=" in head, f"{count}、硬顶 {hard}、开 12 个要 {needed}：{head}")
    sliced = dict(sixteen)
    write_slice_configuration("'127.0.0.1=24G 192.0.2.70=48G'")
    count, head = counted(20, sliced)
    selftest.check("没有服务交的变量、多机配置 SLICE_MEMORY_CAP_BY_HOST 这一台那一项 24G、另一台那一项 48G（整值带引号）、假 memory.max 64 GiB 摆在旁边（不比它小、不收窄）："
                   "取这一台那一项，24 GiB ÷ 4 GiB 开 6 个，头一行写多机配置的路径与这一台那一项",
                   count == 6 and f"多机配置 {configuration} 的 SLICE_MEMORY_CAP_BY_HOST 里这一台 127.0.0.1 那一项=24G" in head and "卡在「内存上限」这一项" in head, f"{count}：{head}")
    write_slice_configuration("127.0.0.1=lots 192.0.2.70=8G")
    write_memory_max(str(8 * gibibyte) + "\n")
    count, head = counted(20, sliced)
    selftest.check("多机配置 SLICE_MEMORY_CAP_BY_HOST 这一台那一项写成 lots：判法整份判不过，头一行点名取不到这一台那一项与「lots」、改读 memory.max（8 GiB 开 2 个）",
                   count == 2 and "SLICE_MEMORY_CAP_BY_HOST 取不到这一台那一项：认不出这一台" in head and "「lots」" in head, f"{count}：{head}")
    write_configuration("ENABLE_GPU=1\n")
    write_memory_max(str(200 * gibibyte) + "\n")   # 比下面三格的服务交的变量都大：memory.max 不收窄它们
    roomy = dict(sixteen, **{TASK_MEMORY_CAP_VARIABLE: str(100 * gibibyte)})
    count, head = counted(20, roomy)
    selftest.check("内存上限 100 GiB 放得下 25 个、线程数 16 更小：开 16 个，卡在 GATE_MUTATION_WORKERS，头一行仍写内存那一项",
                   count == 16 and "卡在「GATE_MUTATION_WORKERS」这一项" in head and "= 同时开 25 个工作进程" in head, f"{count}：{head}")
    tiny = dict(sixteen, **{TASK_MEMORY_CAP_VARIABLE: str(gibibyte)})
    count, head = counted(20, tiny)
    selftest.check("内存上限 1 GiB 一个都放不下（每个工作进程的峰值 4 GiB）：按 1 个开，头一行写「那一项不足 1」",
                   count == 1 and "那一项不足 1，按 1 个开" in head and "= 同时开 0 个工作进程（1 个就要 4.0 GiB，超了上限）" in head, f"{count}：{head}")
    scope_only = dict(sixteen)
    write_memory_max(str(8 * gibibyte) + "\n")
    count, head = counted(20, scope_only)
    selftest.check("没有服务交的变量、自己 scope 的 memory.max 是 8 GiB：开 2 个，头一行写 memory.max 的路径（假 /proc/self/cgroup 里 cgroup v2 的那一行接假 /sys/fs/cgroup）",
                   count == 2 and "自己 scope 的 " + memory_max in head and "卡在「内存上限」这一项" in head, f"{count}：{head}")
    write_memory_max("max\n")
    count, head = counted(20, scope_only)
    selftest.check("自己 scope 的 memory.max 是 max（不限）：这一项不收，照线程数开 16 个，头一行写「是 max」",
                   count == 16 and "没有内存上限这一项" in head and "是 max（不限）" in head and "卡在「GATE_MUTATION_WORKERS」这一项" in head, f"{count}：{head}")
    unreadable = memory_bound_for(scope_only, 16, 16, records, ("tiny",), os.path.join(work, "memory-cgroup-listing-absent"), root)
    selftest.check("读不到 /proc/self/cgroup：没有内存上限这一项（交 None 与原因），不当成 0 个也不当成放开之后再算",
                   unreadable[0] is None and "读不到" in unreadable[1], f"{unreadable}")
    write_memory_max(str(8 * gibibyte) + "\n")
    garbled = dict(sixteen, **{TASK_MEMORY_CAP_VARIABLE: "abc"})
    count, head = counted(20, garbled)
    selftest.check("TASK_SERVER_MEMORY_CAP_BYTES 写成 abc：头一行点名写法不对、改读 memory.max（8 GiB 开 2 个）",
                   count == 2 and "TASK_SERVER_MEMORY_CAP_BYTES 写成「abc」" in head, f"{count}：{head}")
    # 自己 scope 的 memory.max 是硬顶（scope 真上限只有 20736M，按多机配置的 50G 会开 12 个工作进程，顶满被杀）
    write_memory_max(str(20 * gibibyte) + "\n")
    write_slice_configuration("127.0.0.1=50G 192.0.2.70=8G")
    count, head = counted(20, sliced)
    selftest.check("多机配置 SLICE_MEMORY_CAP_BY_HOST 这一台那一项 50G（没有服务交的变量）、自己 scope 的 memory.max 20 GiB：memory.max 是硬顶，取两者小的 20 GiB，开 5 个（不是 12 个），"
                   "头一行写各档读到的值、取 memory.max 与多机配置的 50G 不取",
                   count == 5 and "内存上限 20.0 GiB（" in head and f"自己 scope 的 {memory_max}（硬顶：超了整组被杀）" in head
                   and f"比它大的 多机配置 {configuration} 的 SLICE_MEMORY_CAP_BY_HOST 里这一台 127.0.0.1 那一项=50G（50.0 GiB）不取" in head
                   and (f"各档读到：{TASK_MEMORY_CAP_VARIABLE} 没给、多机配置 SLICE_MEMORY_CAP_BY_HOST 里这一台 127.0.0.1 那一项=50G（50.0 GiB）、"
                        f"scope 的 memory.max={20 * gibibyte}（20.0 GiB）") in head
                   and "= 同时开 5 个工作进程" in head, f"{count}：{head}")
    served_over = dict(sixteen, **{TASK_MEMORY_CAP_VARIABLE: str(50 * gibibyte)})
    count, head = counted(20, served_over)
    selftest.check("服务交的变量 50 GiB、自己 scope 的 memory.max 20 GiB：取两者小的 20 GiB，开 5 个，头一行写服务交的 50 GiB 不取",
                   count == 5 and "内存上限 20.0 GiB（" in head and f"比它大的 服务交的 {TASK_MEMORY_CAP_VARIABLE}={50 * gibibyte}（50.0 GiB）不取" in head
                   and "= 同时开 5 个工作进程" in head, f"{count}：{head}")
    write_memory_max("max\n")
    count, head = counted(20, served_over)
    selftest.check("服务交的变量 50 GiB、自己 scope 的 memory.max 是 max（不限）：没有硬顶可比，照服务交的 50 GiB 开 12 个，头一行写 memory.max 没有硬顶与原因",
                   count == 12 and "内存上限 50.0 GiB（服务交的 TASK_SERVER_MEMORY_CAP_BYTES=" in head and "scope 的 memory.max 没有硬顶（" in head
                   and f"{memory_max} 是 max（不限）" in head and "= 同时开 12 个工作进程" in head, f"{count}：{head}")
    unreadable_served = memory_bound_for(served_over, 16, 16, records, ("tiny",), os.path.join(work, "memory-cgroup-listing-absent"), root)
    selftest.check("服务交的变量 50 GiB、/proc/self/cgroup 读不到：没有硬顶可比，照服务交的 50 GiB 放得下 12 个，头一行写读不到",
                   unreadable_served[0] == 12 and "内存上限 50.0 GiB（" in unreadable_served[1] and "scope 的 memory.max 没有硬顶（读不到" in unreadable_served[1], f"{unreadable_served}")
    write_memory_max(str(8 * gibibyte) + "\n")
    repository = os.path.join(work, "memory-workers")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    target = os.path.join(work, "memory-workers-target")
    # run_stage 读的运行记录目录是共享根底下按这个小仓的命名空间分的那一层（run_records_directory），峰值记录照它放
    stage_changes = dict(GATE_MUTATION_RUN_RECORDS_DIRECTORY=os.path.join(work, "memory-workers-run-records"), GATE_MUTATION_CARGO_JOBS="4")
    selftest_record_tiny_peak(run_records_directory(repository, dict(selftest.base_environment, **stage_changes)), 4, 1, 4 * gibibyte)
    exit_code, _stdout, stderr, _ = selftest.run(
        repository, GATE_MUTATION_WORKERS="16", GATE_MUTATION_TARGET_DIR=target, **stage_changes,
        **{TASK_MEMORY_CAP_VARIABLE: str(8 * gibibyte)})
    selftest.check("内存上限交给 run_stage：开的工作进程数按内存收（8 GiB ÷ 4 GiB 开 2 个，不是条数 4），头一行与收尾一行都写上限、每个工作进程的峰值、个数，卡在「内存上限」",
                   exit_code == 0 and "这一轮开 2 个工作进程" in stderr and "卡在「内存上限」这一项" in stderr and stderr.count("= 同时开 2 个工作进程") >= 2
                   and "内存上限 8.0 GiB" in stderr and peak_text in stderr, f"退 {exit_code}；{stderr[:1200]}")
    # 第二趟换一棵树（库是新的一份，要现跑才开工作进程）：运行记录目录按库的命名空间分，峰值记录也放一份给它
    second_repository = selftest_new_library_copy(work, repository, "memory-workers-second")
    selftest_record_tiny_peak(run_records_directory(second_repository, dict(selftest.base_environment, **stage_changes)), 4, 1, 4 * gibibyte)
    exit_code, _stdout, stderr, _ = selftest.run(
        second_repository, GATE_MUTATION_WORKERS="16", GATE_MUTATION_TARGET_DIR=target, **stage_changes,
        GATE_MUTATION_CGROUP_LISTING=listing, GATE_MUTATION_CGROUP_ROOT=root, **{TASK_MEMORY_CAP_VARIABLE: str(50 * gibibyte)})
    selftest.check("run_stage 整趟：服务交的变量 50 GiB、自己 scope 的 memory.max（经 GATE_MUTATION_CGROUP_LISTING / GATE_MUTATION_CGROUP_ROOT 指到假的）8 GiB："
                   "开的工作进程数按 8 GiB 收（2 个，不是按 50 GiB 的 12 与条数 4），开跑行写 memory.max 是硬顶",
                   exit_code == 0 and "这一轮开 2 个工作进程" in stderr and "内存上限 8.0 GiB（" in stderr and "（硬顶：超了整组被杀）" in stderr
                   and "卡在「内存上限」这一项" in stderr, f"退 {exit_code}；{stderr[:1500]}")
    # ⑧ 撞内存上限：上限 2 GiB 装不下一个 4 GiB 的工作进程：照开 1 个起跑、不自己退 250，真撞了由统一任务服务判（错误类型表里 memory-cap-hit 那一行）
    short_repository = selftest_new_library_copy(work, repository, "memory-workers-short")
    selftest_record_tiny_peak(run_records_directory(short_repository, dict(selftest.base_environment, **stage_changes)), 4, 1, 4 * gibibyte)
    exit_code, stdout, stderr, invocations = selftest.run(
        short_repository, GATE_MUTATION_WORKERS="16", GATE_MUTATION_TARGET_DIR=target, **stage_changes, **{TASK_MEMORY_CAP_VARIABLE: str(2 * gibibyte)})
    selftest.check("内存上限装不下一个工作进程的估值：run_stage 照开 1 个起跑、不自己退 250（上限 2 GiB、每个工作进程的峰值 4 GiB），退 0，"
                   "头一行写估值与这一条拿到的上限、「照开 1 个起跑、不自己退 250」，收尾一行带同一句，stdout 没有「按撞内存上限退出」，假 cargo 起了",
                   exit_code == 0 and "这一轮开 1 个工作进程（内存上限装不下一个工作进程的估值：这一条拿到的上限 2.0 GiB" in stderr
                   and "照开 1 个起跑、不自己退 250" in stderr and stderr.count("照开 1 个起跑、不自己退 250") >= 2 and peak_text in stderr
                   and "按撞内存上限退出" not in stdout and plain_invocations(invocations),
                   f"退 {exit_code}；假 cargo 起了 {len(plain_invocations(invocations))} 次基线与变异；stdout「{stdout[-800:]}」；stderr「{stderr[-1500:]}」")
    # ⑨ 照这一次服务给的上限开工作进程（文件头「内存上限」的「撞内存上限」）：多机配置这一台那一项 48G（这一台能给的内存，减去起这一道的父进程自己）、
    #   包 tiny 在编译并行度 4 下一份 10 GiB 的实测、4 行：按这一台能给的内存要开 4 个（4 × 10 GiB = 40 GiB）。服务交的上限四档（父进程自己按不到 5 GiB 算）：
    #   48 GiB 装得下 4 个照开满；15 GiB 只装得下 1 个：减到 1 个、照跑、退 0；35 GiB 装得下 3 个：减到 3 个、照跑、退 0；
    #   8 GiB 1 个也装不下：照开 1 个起跑、退 0
    top_configuration = os.path.join(work, "memory-top-config", "multi-host.env")
    os.makedirs(os.path.dirname(top_configuration), exist_ok=True)
    with open(top_configuration, "w", encoding="utf-8") as handle:
        handle.write("HOSTS=127.0.0.1 192.0.2.70\nCARGO_BIN_DIRECTORY_BY_HOST=127.0.0.1=/selftest/this-cargo 192.0.2.70=/selftest/other-cargo\n"
                     "BUILD_ROOT_BY_HOST=127.0.0.1=/selftest/this-builds 192.0.2.70=/selftest/other-builds\n"
                     "SLICE_MEMORY_CAP_BY_HOST=127.0.0.1=48G 192.0.2.70=8G\nQUIESCE_STOP_COMMAND=\nQUIESCE_STOPPED_CHECK_COMMAND=\nQUIESCE_START_COMMAND=\n"
                     f"QUIESCE_STARTED_CHECK_COMMAND=\nENABLE_ACROSS_MACHINES=0\nENABLE_GPU=0\nNFS_SHARED_ROOTS={os.path.dirname(top_configuration)}\n")
    top_runs = {}
    for tier, served_gibibytes in (("full", 48), ("one", 15), ("three", 35), ("none", 8)):
        tier_repository = selftest_new_library_copy(work, repository, f"memory-workers-tier-{tier}")
        selftest_record_tiny_peak(run_records_directory(tier_repository, dict(selftest.base_environment, **stage_changes)), 4, 1, 10 * gibibyte)
        top_runs[tier] = selftest.run(tier_repository, GATE_MUTATION_WORKERS="16", GATE_MUTATION_TARGET_DIR=target, GATE_MUTATION_MULTI_HOST_CONFIG=top_configuration,
                                      **stage_changes, **{TASK_MEMORY_CAP_VARIABLE: str(served_gibibytes * gibibyte)})
    exit_code, _stdout, stderr, _invocations = top_runs["full"]
    selftest.check("照这一次服务给的上限开工作进程（够开满）：这一台能给的内存 48G、每个工作进程 10 GiB、4 行，服务交的上限 48 GiB 装得下 4 个：照开满 4 个、退 0、不往下减",
                   exit_code == 0 and "这一轮开 4 个工作进程" in stderr and "因为内存从" not in stderr and "这一台能给的内存 " in stderr, f"退 {exit_code}；{stderr[:1500]}")
    for tier, kept in (("one", 1), ("three", 3)):
        exit_code, stdout, stderr, invocations = top_runs[tier]
        served = {"one": 15, "three": 35}[tier]
        selftest.check(f"照这一次服务给的上限开工作进程（上限{'只' if kept == 1 else ''}够 {kept} 个工作进程）：服务交的上限 {served} GiB 装不下要开的 4 个、装得下 {kept} 个："
                       f"减到 {kept} 个开、照跑、退 0，头一行写因为内存从 4 个减到了 {kept} 个，收尾一行写 {kept} 个工作进程，假 cargo 起了",
                       exit_code == 0 and f"这一轮开 {kept} 个工作进程（因为内存从 4 个减到了 {kept} 个：这一条拿到的上限 {served}.0 GiB 装不下 4 个" in stderr
                       and f"这一轮 {kept} 个工作进程" in stderr and "按撞内存上限退出" not in stdout
                       and plain_invocations(invocations),
                       f"退 {exit_code}；假 cargo 起了 {len(plain_invocations(invocations))} 次基线与变异；stdout「{stdout[-600:]}」；stderr「{stderr[-1500:]}」")
    exit_code, stdout, stderr, invocations = top_runs["none"]
    selftest.check("照这一次服务给的上限开工作进程（1 个也不够）：服务交的上限 8 GiB 装不下 1 个（10 GiB 加父进程自己）：照开 1 个起跑、退 0，"
                   "头一行写从 4 个减到 1 个也装不下与「照开 1 个起跑、不自己退 250」，stdout 没有「按撞内存上限退出」，假 cargo 起了",
                   exit_code == 0 and "这一轮开 1 个工作进程（内存从 4 个减到 1 个也装不下：这一条拿到的上限 8.0 GiB 装不下 4 个" in stderr
                   and "照开 1 个起跑、不自己退 250" in stderr and "按撞内存上限退出" not in stdout and plain_invocations(invocations),
                   f"退 {exit_code}；假 cargo 起了 {len(plain_invocations(invocations))} 次基线与变异；stdout「{stdout[-800:]}」；stderr「{stderr[-1500:]}」")


def selftest_worker_peak_cells(selftest, work):
    """每个工作进程的峰值：量、记、读、除（文件头「内存上限」的「每个工作进程的峰值」）。
    ① 量一棵进程树：本进程起一个 sh、sh 再起一个占着 256 MiB 的 python（孙进程），process_tree_usage(本进程) 比起之前至少多 240 MiB（弄坏开关 worker-tree-self-only 下判错）；
    ② 记下的峰值按包、按编译并行度分开、读得回来：包 tiny 在编译并行度 4 记 5 GiB、6 GiB，8 记 12 GiB，读回 {tiny: {4: (6 GiB, 2 份), 8: (12 GiB, 1 份)}}（worker-peaks-unkeyed-by-jobs 下判错）；
    ③ 每个包、每个编译并行度只看最新的 WORKER_PEAKS_RECENT 份：包 tiny 在编译并行度 3 记多两份，最旧的两份 20 GiB、其余 1 GiB，读回 1 GiB，读完只剩那么多份（worker-peaks-window-unbounded 下判错）；
    ④ 固定编译并行度（GATE_MUTATION_CARGO_JOBS=4）：上限 20 GiB ÷ 编译并行度 4 下的 6 GiB = 3 个，3 × 6 GiB 不超过上限（worker-peaks-unkeyed-by-jobs 下判错）；
    ⑤ 编译并行度跟着进程数走：核数 16、天花板 8，编译并行度 2、4、8、16 各实测 4、7、9、14 GiB，上限 20 GiB：开 2 个（编译并行度 8、每个 9 GiB、合计 18 GiB），
      3 个时编译并行度 5 借 8 的 9 GiB、合计 27 GiB 超了（jobs-feedback-ignored、worker-peaks-unkeyed-by-jobs 下判错）；
    ⑥ 从没量过：上限 20 GiB，不照保守值收成 1 个，这一项交天花板 16 个，头一行写包从没量过、派活时单独跑（探针）与峰值记录在哪（new-package-run-single-worker 下判错）；
      同样的记录交给不写峰值记录的一趟（memory_bound_probing 的 probing 是假，门禁样本根）：照保守值开 1 个（unmeasured-worker-peak-guessed 下判错）；
    ⑦ 只有更低编译并行度的实测：GATE_MUTATION_CARGO_JOBS=4、只有编译并行度 2 的 1 GiB，不往低处借，取保守值开 1 个（unmeasured-worker-peak-guessed 下判错）；
    ⑧ 峰值记录里有一份内容不是正整数、⑨ worker-peaks 是一份文件不是目录：整份读不了，不退回写死的默认值，取保守值开 1 个，头一行写「读不了」与那一份（unreadable-worker-peaks-guessed 下判错）；
    ⑩ 交给 run_stage：GATE_MUTATION_CARGO_JOBS=3、运行记录里编译并行度 3 一份 3 GiB，上限 6 GiB、4 条：开 2 个工作进程，头一行与收尾一行都写个数
      （worker-peaks-unkeyed-by-package 下判错：包 tiny 与 tiny--lib 各记一份读数，合起来读成最新 2 份）；
    ⑪ 交给 run_stage 的工作进程各写一份峰值记录（⑩ 那一趟的假 cargo 每格睡 0.8 秒，采样线程每格里至少量到一次）：那一趟之后包 tiny 在编译并行度 3 下多出至少 2 份（一个工作进程一份），每份是不小于 1 MiB 的正整数，测试目标那一级 tiny--lib 同样多出至少 2 份、记录名只有 tiny 与 tiny--lib（worker-peaks-unrecorded 下判错）；
    ⑫ 样本根上的工作进程不写峰值记录：不在临时服务底下时别处的根不写、本模块所在的仓与设了 GATE_STAGED_FROM 的写，在临时服务底下写（sample-root-peaks-recorded 下判错）；
    ⑬ 只判小包的行：记录里大包 30 GiB、小包 1 GiB，上限 20 GiB（小于大包、大于小包），只判小包：能开工作进程（20 ÷ 1 个，不是 0 个），取法写小包（worker-peaks-unkeyed-by-package 下判错）；
    ⑭ 同样的记录同时判大包与小包的行：取大包的 30 GiB，头一行写涉及 2 个包与大包，上限装不下一个（交 0）；
    ⑮ 要判的包里有一个从没量过：小包有实测、新包没有，新包不进这一项，照小包的 1 GiB 开 20 个，头一行点名新包从没量过与探针（new-package-run-single-worker、worker-peaks-unkeyed-by-package 下判错）；
      探针派活（memory_admits）：tiny 实测 4 GiB、新包没有，上限 20 GiB：新包的格只在一格都没在跑时派，新包在跑时 tiny 的格也不派，只有 tiny 时至多 5 格
      （unmeasured-worker-peak-guessed、unmeasured-package-not-probed-alone 下判错）；新包记一份 2 GiB 之后 refresh_memory_admission 重读、交一行「量到了（探针）」写新包自己的 2 GiB、放开到至多 5 格，都量到之后不再读
      （worker-peaks-unkeyed-by-package 下判错：新包读成 tiny 的 4 GiB）；
    ⑯ 交给 run_stage 只判小包的行：运行记录里另一个包 30 GiB、小包（tiny）1 GiB，上限 6 GiB、4 条：开 4 个工作进程、不退 250，头一行写包 tiny（worker-peaks-unkeyed-by-package 下 tiny 读成 30 GiB、只开 1 个起跑）；
    ⑰ 一格点名的包怎么取（worker_peak_packages）：-p 的包名排序去重，name@版本只取包名，没点名包的与写法不是包名的算 @workspace，特性不分；
      worker_peak_scope 在判一格期间设着采样线程要读的范围、出来清掉（采样线程不设范围时 ⑪ 那一格没有记录而判错）；
    ⑱ 交给 run_stage、这几行点名的包（tiny）从没量过：4 行各差一个 -j（4 批基线），上限 16 GiB：开 4 个工作进程，头一批基线单独跑（假 cargo 的起止记录里它起止之间没有别的 cargo），
      量到之后放开（同时在跑的 cargo 至少 2 个），stderr 写「包 tiny 量到了（探针）」（new-package-run-single-worker、unmeasured-package-not-probed-alone、
      unmeasured-worker-peak-guessed、worker-peaks-unrecorded、memory-cap-ignored 下判错）。
    弄坏开关 memory-cap-ignored 下经 memory_bound_for、memory_bound_probing 的 ④–⑩ 八格（⑥ 两格）与 ⑬–⑯、⑱ 五格判错。"""
    gibibyte = 1 << 30
    environment = {key: value for key, value in selftest.base_environment.items() if key not in (TASK_MEMORY_CAP_VARIABLE, "GATE_MUTATION_CARGO_JOBS")}

    def record(records, package, jobs, index, value):
        record_worker_peak(records, package, jobs, f"{index:020d}-1-selftest", value)

    holder_code = "import sys\nblock = b'\\x01' * (256 << 20)\nsys.stdout.write('ready\\n')\nsys.stdout.flush()\nsys.stdin.read()\n"
    before = process_tree_usage(os.getpid(), environment)
    # sh 后面还有一条命令就不 exec，python 是本进程的孙进程；读到标准输入结束就退
    holder = subprocess.Popen(["sh", "-c", '"$0" -c "$1"; exit 0', sys.executable, holder_code], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
    try:
        ready = holder.stdout.readline().strip()
        during = process_tree_usage(os.getpid(), environment)
    finally:
        holder.stdin.close()
        try:
            holder.wait(timeout=60)
        except subprocess.TimeoutExpired:
            holder.kill()   # 只对自己起的这一个发信号；孙进程读到标准输入结束自己退
            holder.wait()
        holder.stdout.close()
    grown = (during or 0) - (before or 0)
    selftest.check("量一棵进程树：本进程起的 sh 再起一个占着 256 MiB 的 python（孙进程），process_tree_usage(本进程) 比起之前多出至少 240 MiB",
                   ready == "ready" and before is not None and during is not None and grown >= 240 << 20, f"ready={ready!r}，之前 {before}、之后 {during}")

    keyed = os.path.join(work, "worker-peaks-keyed")
    record(keyed, "tiny", 4, 1, 5 * gibibyte)
    record(keyed, "tiny", 4, 2, 6 * gibibyte)
    record(keyed, "tiny", 8, 3, 12 * gibibyte)
    peaks, source = measured_worker_peaks(keyed, environment)
    selftest.check("记下的峰值按包、按编译并行度分开、读得回来：包 tiny 在编译并行度 4 记了 5 GiB 与 6 GiB、8 记了 12 GiB，读回编译并行度 4 最新 2 份里最大 6 GiB、8 最新 1 份里最大 12 GiB",
                   peaks == {"tiny": {4: (6 * gibibyte, 2), 8: (12 * gibibyte, 1)}} and "包 tiny 编译并行度 4 最新 2 份里最大 6.0 GiB" in source, f"{peaks}：{source}")
    windowed = os.path.join(work, "worker-peaks-window")
    for index in range(1, WORKER_PEAKS_RECENT + 3):
        record(windowed, "tiny", 3, index, 20 * gibibyte if index <= 2 else gibibyte)
    peaks, source = measured_worker_peaks(windowed, environment)
    left = len(os.listdir(os.path.join(worker_peaks_directory(windowed), "tiny", "3")))
    selftest.check(f"每个包、每个编译并行度只看最新的 {WORKER_PEAKS_RECENT} 份：包 tiny 在编译并行度 3 记了 {WORKER_PEAKS_RECENT + 2} 份，最旧的两份 20 GiB、其余 1 GiB，"
                   f"读回 1 GiB（{WORKER_PEAKS_RECENT} 份），读完只剩 {WORKER_PEAKS_RECENT} 份",
                   peaks == {"tiny": {3: (gibibyte, WORKER_PEAKS_RECENT)}} and left == WORKER_PEAKS_RECENT, f"{peaks}；剩 {left} 份")

    roof = dict(environment, **{TASK_MEMORY_CAP_VARIABLE: str(20 * gibibyte)})
    fixed = dict(roof, GATE_MUTATION_CARGO_JOBS="4")
    count, text = memory_bound_for(fixed, 16, 16, keyed, ("tiny",))
    selftest.check("固定编译并行度：GATE_MUTATION_CARGO_JOBS=4、编译并行度 4 最大 6 GiB（8 那一份 12 GiB 不算），上限 20 GiB，天花板 16：开 ⌊20 ÷ 6⌋ = 3 个，3 × 6 GiB = 18 GiB 不超过上限",
                   count == 3 and count * 6 * gibibyte <= 20 * gibibyte and "每个工作进程的峰值 6.0 GiB（编译并行度 4 下：包 tiny 最新 2 份实测里最大的）" in text
                   and "= 同时开 3 个工作进程（3 × 6.0 GiB = 18.0 GiB，不超过上限）" in text, f"{count}：{text}")
    following = os.path.join(work, "worker-peaks-following")
    for index, (jobs, size) in enumerate(((2, 4), (4, 7), (8, 9), (16, 14)), 1):
        record(following, "tiny", jobs, index, size * gibibyte)
    count, text = memory_bound_for(roof, 8, 16, following, ("tiny",))
    selftest.check("编译并行度跟着进程数走：核数 16、天花板 8，编译并行度 2、4、8、16 各实测 4、7、9、14 GiB，上限 20 GiB：开 2 个（编译并行度 8、每个 9 GiB，2 × 9 GiB = 18 GiB 不超过上限），"
                   "3 个时编译并行度 5 借 8 的 9 GiB、合计 27 GiB 超了；不照天花板 8 个时的编译并行度 2 去除（那样开 5 个，编译并行度 3 借 4 的 7 GiB、合计 35 GiB）",
                   count == 2 and count * 9 * gibibyte <= 20 * gibibyte and "每个工作进程的峰值 9.0 GiB（编译并行度 8 下：包 tiny 最新 1 份实测里最大的）" in text
                   and "= 同时开 2 个工作进程（2 × 9.0 GiB = 18.0 GiB，不超过上限）" in text
                   and "再多 1 个：编译并行度 5 下每个 9.0 GiB（包 tiny 没有实测，借编译并行度 8 最新 1 份实测里最大的" in text and "3 × 9.0 GiB = 27.0 GiB，超了上限" in text,
                   f"{count}：{text}")
    never = os.path.join(work, "worker-peaks-never")
    count, text = memory_bound_for(roof, 16, 16, never, ("tiny",))
    selftest.check("从没量过：峰值记录还没有，上限 20 GiB：不照保守值把整趟收成 1 个，这一项交天花板 16 个（编译并行度 1），头一行写包 tiny 从没量过、派活时单独跑（探针）与峰值记录在哪",
                   count == 16 and "这一趟要判的包都从没量过，这一项交天花板 16 个工作进程（编译并行度 1）" in text
                   and "包 tiny 从没量过：不进这一项，派活时它的格单独跑（探针" in text
                   and f"峰值记录 {worker_peaks_directory(never)} 还没有（从没量过）" in text, f"{count}：{text}")
    count, text, cap = memory_bound_probing(roof, 16, 16, never, ("tiny",), False, CGROUP_LISTING, CGROUP_ROOT, None, None)[:3]
    selftest.check("不写峰值记录的一趟（门禁样本根）：探针量不到东西，从没量过的包照保守值（上限本身 20 GiB）进这一项，开 1 个，头一行写从没量过与保守值",
                   count == 1 and cap == 20 * gibibyte and "包 tiny 从没量过，取保守值" in text and "= 同时开 1 个工作进程（1 × 20.0 GiB = 20.0 GiB，不超过上限）" in text
                   and "探针" not in text, f"{count}、上限 {cap}：{text}")
    lower = os.path.join(work, "worker-peaks-lower")
    record(lower, "tiny", 2, 1, gibibyte)
    count, text = memory_bound_for(fixed, 16, 16, lower, ("tiny",))
    selftest.check("只有更低编译并行度的实测：GATE_MUTATION_CARGO_JOBS=4、只有编译并行度 2 的 1 GiB，不往低处借，取保守值开 1 个",
                   count == 1 and "只有更低编译并行度的实测、不借，取保守值" in text, f"{count}：{text}")
    garbled = os.path.join(work, "worker-peaks-garbled")
    selftest_write(os.path.join(worker_peaks_directory(garbled), "tiny", "4", f"{1:020d}-1-selftest"), "lots\n")
    count, text = memory_bound_for(fixed, 16, 16, garbled, ("tiny",))
    selftest.check("峰值记录里有一份内容不是正整数：整份读不了，不退回写死的默认值，取保守值开 1 个，头一行写「读不了」、那一份的路径与「lots」",
                   count == 1 and "峰值记录读不了，取保守值" in text and "写着「lots」，不是一个正整数字节数" in text and f"{1:020d}-1-selftest" in text, f"{count}：{text}")
    flat = os.path.join(work, "worker-peaks-flat")
    selftest_write(worker_peaks_directory(flat), "不是目录\n")
    count, text = memory_bound_for(fixed, 16, 16, flat, ("tiny",))
    selftest.check("峰值记录的 worker-peaks 是一份文件、不是目录：整份读不了，取保守值开 1 个，头一行写「读不了」与 NotADirectoryError",
                   count == 1 and "峰值记录读不了，取保守值" in text and "NotADirectoryError" in text, f"{count}：{text}")

    # ⑬⑭⑮ 峰值按这一趟要判的行点名的包取：大包 30 GiB、小包 1 GiB（都在编译并行度 4 下），上限 20 GiB
    packaged = os.path.join(work, "worker-peaks-packages")
    record(packaged, "big-package", 4, 1, 30 * gibibyte)
    record(packaged, "small-package", 4, 2, gibibyte)
    count, text = memory_bound_for(fixed, 16, 16, packaged, ("small-package",))
    selftest.check("只判小包的行：记录里大包 30 GiB、小包 1 GiB，上限 20 GiB（小于大包的峰值、大于小包的）：每个工作进程的峰值取小包的 1 GiB，天花板 16 个全开得出（一个都放不下才是 0），"
                   "头一行写小包、不写大包",
                   count == 20 and "每个工作进程的峰值 1.0 GiB（编译并行度 4 下：包 small-package 最新 1 份实测里最大的）" in text and "big-package 最新" not in text,
                   f"{count}：{text}")
    count, text = memory_bound_for(fixed, 16, 16, packaged, ("big-package", "small-package"))
    selftest.check("同样的记录同时判大包与小包的行：取大包的 30 GiB，头一行写涉及 2 个包与大包；上限 20 GiB 装不下一个，交 0（run_stage 据此照开 1 个起跑）",
                   count == 0 and "每个工作进程的峰值 30.0 GiB（编译并行度 4 下：这一趟涉及 2 个包，取其中最大的：包 big-package 最新 1 份实测里最大的）" in text
                   and "1 个就要 30.0 GiB，超了上限" in text, f"{count}：{text}")
    count, text = memory_bound_for(fixed, 16, 16, packaged, ("small-package", "fresh-package"))
    selftest.check("要判的包里有一个从没量过：小包有实测、新包没有，新包不进这一项（派活时探针收住），每个工作进程的峰值取小包的 1 GiB：开 20 个、不收成 1 个，头一行点名新包从没量过与探针",
                   count == 20 and "每个工作进程的峰值 1.0 GiB（编译并行度 4 下：包 small-package 最新 1 份实测里最大的）" in text
                   and "包 fresh-package 从没量过：不进这一项，派活时它的格单独跑（探针" in text and "取保守值" not in text, f"{count}：{text}")

    # 探针派活（memory_admits、refresh_memory_admission）：包 tiny 在编译并行度 4 下实测 4 GiB，包 fresh-package 从没量过，上限 20 GiB
    probed = os.path.join(work, "worker-peaks-probe")
    record(probed, "tiny", 4, 1, 4 * gibibyte)
    admission = new_memory_admission(dict(memory_admission=dict(cap=20 * gibibyte, jobs=4, packages=("fresh-package", "tiny")),
                                          run_records_directory=probed, environment=fixed))
    admitted = (memory_admits(admission, [], ("fresh-package",), fixed),
                memory_admits(admission, [("tiny",)], ("fresh-package",), fixed),
                memory_admits(admission, [("fresh-package",)], ("tiny",), fixed),
                memory_admits(admission, [("tiny",)] * 4, ("tiny",), fixed),
                memory_admits(admission, [("tiny",)] * 5, ("tiny",), fixed))
    selftest.check("探针派活：包 tiny 在编译并行度 4 下实测 4 GiB、fresh-package 从没量过，上限 20 GiB：fresh-package 的格一格都没在跑时照派、tiny 在跑时不派，"
                   "fresh-package 在跑时 tiny 的格也不派；只有 tiny 时在跑 4 格还派得下第 5 格（5 × 4 GiB = 20 GiB）、在跑 5 格不派第 6 格；还没量到的包是 fresh-package",
                   admission is not None and admission["unmeasured"] == {"fresh-package"} and admitted == (True, False, False, True, False),
                   f"派不派（没在跑派 fresh、tiny 在跑派 fresh、fresh 在跑派 tiny、tiny 在跑 4 格派 tiny、tiny 在跑 5 格派 tiny）= {admitted}；"
                   f"还没量到 {admission and admission['unmeasured']}")
    record(probed, "fresh-package", 4, 2, 2 * gibibyte)
    lines = refresh_memory_admission(admission, fixed)
    released = (memory_admits(admission, [("tiny",)], ("fresh-package",), fixed),
                memory_admits(admission, [("tiny",)] * 4, ("fresh-package",), fixed),
                memory_admits(admission, [("fresh-package",)] * 5, ("tiny",), fixed))
    again = refresh_memory_admission(admission, fixed)
    selftest.check("探针量到之后放开：fresh-package 在编译并行度 4 下记了一份 2 GiB，重读之后它不再算没量到，交一行「包 fresh-package 量到了（探针）」；"
                   "tiny 在跑 1 格、4 格时都派得下 fresh-package 的格（峰值取两个包里大的 4 GiB，至多 5 格），在跑 5 格时不派；都量到之后再重读交空",
                   not admission["unmeasured"] and len(lines) == 1 and "包 fresh-package 量到了（探针）：编译并行度 4 下每个工作进程 2.0 GiB" in lines[0]
                   and released == (True, True, False) and again == [],
                   f"还没量到 {admission['unmeasured']}；说明 {lines}；派不派（tiny 在跑 1 格、4 格派 fresh，fresh 在跑 5 格派 tiny）= {released}；再重读 {again}")

    repository = os.path.join(work, "worker-peaks-run-stage")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    # run_stage 读写的运行记录目录是共享根底下按这个小仓的命名空间分的那一层（run_records_directory），预先放的那一份照它放
    stage_changes = dict(GATE_MUTATION_RUN_RECORDS_DIRECTORY=os.path.join(work, "worker-peaks-run-stage-records"), GATE_MUTATION_CARGO_JOBS="3")
    stage_records = run_records_directory(repository, dict(selftest.base_environment, **stage_changes))
    selftest_record_tiny_peak(stage_records, 3, 1, 3 * gibibyte)
    exit_code, _stdout, stderr, _ = selftest.run(
        repository, GATE_MUTATION_WORKERS="16", GATE_MUTATION_TARGET_DIR=os.path.join(work, "worker-peaks-run-stage-target"),
        FAKE_CARGO_MUTATION_SLEEP="0.8", **stage_changes, **{TASK_MEMORY_CAP_VARIABLE: str(6 * gibibyte)})   # 假 cargo 每格睡 0.8 秒：采样线程（隔 0.2 秒量一次）在每一格里至少量到一次
    selftest.check("交给 run_stage：GATE_MUTATION_CARGO_JOBS=3、运行记录里编译并行度 3 一份 3 GiB，上限 6 GiB、4 条：开 2 个工作进程，头一行与收尾一行都写个数，卡在「内存上限」",
                   exit_code == 0 and "这一轮开 2 个工作进程" in stderr and stderr.count("= 同时开 2 个工作进程") >= 2 and "卡在「内存上限」这一项" in stderr
                   and "每个工作进程的峰值 3.0 GiB（编译并行度 3 下：包 tiny 最新 1 份实测里最大的）" in stderr, f"退 {exit_code}；{stderr[:1500]}")
    recorded = os.path.join(worker_peaks_directory(stage_records), "tiny", "3")
    written = sorted(name for name in os.listdir(recorded) if name != f"{1:020d}-1-selftest" and not name.startswith(".partial-")) if os.path.isdir(recorded) else []
    values = []
    for name in written:
        with open(os.path.join(recorded, name), encoding="utf-8") as handle:
            text = handle.read()
        values.append(int(text) if re.fullmatch(r"[1-9][0-9]*\n", text) else text)
    packages_recorded = sorted(os.listdir(worker_peaks_directory(stage_records)))
    # 这几行都是 -p tiny --lib：采样线程把每个读数记在包 tiny 下，也记在测试目标那一级 tiny--lib 下（worker_peak_scope），两处各一个工作进程一份
    target_recorded = os.path.join(worker_peaks_directory(stage_records), "tiny--lib", "3")
    target_written = (sorted(name for name in os.listdir(target_recorded) if name != f"{1:020d}-1-selftest" and not name.startswith(".partial-"))
                      if os.path.isdir(target_recorded) else [])
    target_values = []
    for name in target_written:
        with open(os.path.join(target_recorded, name), encoding="utf-8") as handle:
            text = handle.read()
        target_values.append(int(text) if re.fullmatch(r"[1-9][0-9]*\n", text) else text)
    selftest.check("交给 run_stage 的工作进程各写一份峰值记录：那一趟之后包 tiny 在编译并行度 3 下多出至少 2 份（一个工作进程一份），每份是不小于 1 MiB 的正整数字节数，"
                   "记录只在这几行点名的包 tiny 与它的测试目标那一级 tiny--lib 下（没有别的包、没有 @workspace），tiny--lib 在编译并行度 3 下同样至少 2 份、每份是不小于 1 MiB 的正整数字节数",
                   len(written) >= 2 and all(isinstance(value, int) and value >= 1 << 20 for value in values) and packages_recorded == ["tiny", "tiny--lib"]
                   and len(target_written) >= 2 and all(isinstance(value, int) and value >= 1 << 20 for value in target_values),
                   f"多出 {len(written)} 份：{values}；tiny--lib 下 {len(target_written)} 份：{target_values}；worker-peaks 下的记录名：{packages_recorded}")
    # ⑯ 同一棵表、另起一份新库（要现跑才开工作进程）：运行记录里另一个包 30 GiB、这几行点名的包 tiny 1 GiB，上限 6 GiB
    packaged_repository = selftest_new_library_copy(work, repository, "worker-peaks-packages-run-stage")
    packaged_records = run_records_directory(packaged_repository, dict(selftest.base_environment, **stage_changes))
    record(packaged_records, "other-big-package", 3, 1, 30 * gibibyte)
    selftest_record_tiny_peak(packaged_records, 3, 2, gibibyte)
    exit_code, _stdout, stderr, _ = selftest.run(
        packaged_repository, GATE_MUTATION_WORKERS="16", GATE_MUTATION_TARGET_DIR=os.path.join(work, "worker-peaks-packages-run-stage-target"),
        **stage_changes, **{TASK_MEMORY_CAP_VARIABLE: str(6 * gibibyte)})
    selftest.check("交给 run_stage 只判小包（tiny）的行：运行记录里另一个包 30 GiB、tiny 1 GiB，上限 6 GiB（小于另一个包、大于 tiny）、4 条：开 4 个工作进程、不退 250，"
                   "头一行写包 tiny 的 1 GiB、不写另一个包",
                   exit_code == 0 and "这一轮开 4 个工作进程" in stderr and "每个工作进程的峰值 1.0 GiB（编译并行度 3 下：包 tiny 最新 1 份实测里最大的）" in stderr
                   and "内存上限装不下一个工作进程的峰值" not in stderr, f"退 {exit_code}；{stderr[:1500]}")
    # ⑱ 交给 run_stage、包 tiny 从没量过：4 行各差一个 -j（基线分成 4 批，同时派得出），上限 16 GiB，运行记录里一份都没有；假 cargo 每次睡 2 秒（FAKE_CARGO_SPAN_LOG 记起止）
    probe_repository = os.path.join(work, "worker-peaks-probe-run-stage")
    build_selftest_repository(probe_repository, [f"probe-row-{index}\tcrates/tiny/src/lib.rs\t{{ left + right }}\t{{ left * (right + {index}) }}\t-p tiny --lib -j {index + 1} -- adds\tadds"
                                                 for index in range(4)])
    probe_changes = dict(GATE_MUTATION_RUN_RECORDS_DIRECTORY=os.path.join(work, "worker-peaks-probe-run-stage-records"), GATE_MUTATION_CARGO_JOBS="3")
    probe_records = run_records_directory(probe_repository, dict(selftest.base_environment, **probe_changes))
    probe_spans = os.path.join(work, "worker-peaks-probe-spans.log")
    with contextlib.suppress(FileNotFoundError):
        os.unlink(probe_spans)
    exit_code, stdout, stderr, _ = selftest.run(
        probe_repository, GATE_MUTATION_WORKERS="16", GATE_MUTATION_TARGET_DIR=os.path.join(work, "worker-peaks-probe-run-stage-target"),
        FAKE_CARGO_SPAN_LOG=probe_spans, FAKE_CARGO_SPAN_SLEEP="2", **probe_changes, **{TASK_MEMORY_CAP_VARIABLE: str(16 * gibibyte)})
    span_events = []
    with contextlib.suppress(OSError):
        with open(probe_spans, encoding="utf-8") as handle:
            for line in handle.read().splitlines():
                fields = line.split("\t")
                if len(fields) == 3:
                    span_events.append((fields[0], fields[1]))
    opened, most, probe_alone, probe_name = set(), 0, None, None
    for kind, name in span_events:
        if kind == "start":
            if probe_name is None and name.startswith("baseline-"):
                probe_name, probe_alone = name, not opened
            elif probe_name is not None and probe_name in opened:
                probe_alone = False
            opened.add(name)
            most = max(most, len(opened))
        else:
            opened.discard(name)
    probe_recorded = os.path.isdir(os.path.join(worker_peaks_directory(probe_records), "tiny", "3"))
    selftest.check("交给 run_stage、这几行点名的包从没量过：上限 16 GiB、4 条（4 批基线）、运行记录里一份都没有：不收成 1 个，开 4 个工作进程，头一行写包 tiny 从没量过；"
                   "头一批基线单独跑（它起止之间没有别的 cargo 起来），它量到峰值之后放开（同时在跑的 cargo 至少 2 个），stderr 写「包 tiny 量到了（探针）」，4 行都判成抓到",
                   exit_code == 0 and "这一轮开 4 个工作进程" in stderr and "这一趟要判的包都从没量过，这一项交天花板 4 个工作进程（编译并行度 3）" in stderr
                   and "包 tiny 从没量过：不进这一项" in stderr and probe_alone is True and most >= 2 and "包 tiny 量到了（探针）：编译并行度 3 下" in stderr
                   and probe_recorded and len(per_row_how(stdout)) == 4,
                   f"退 {exit_code}；头一批基线 {probe_name} 单独跑 {probe_alone}；同时在跑的 cargo 最多 {most}（起止 {len(span_events)} 行）；"
                   f"峰值记录 worker-peaks/tiny/3 在 {probe_recorded}；抓到 {len(per_row_how(stdout))} 行；{stderr[:1500]}")
    selftest.check("一格点名的包怎么取：-p 的包名排序去重；name@版本只取包名；没点名包、写法不是包名的算 @workspace；特性不分；worker_peak_scope 在判一格期间设范围、出来清掉",
                   worker_peak_packages(["-p tiny --lib -- adds"]) == ("tiny",)
                   and worker_peak_packages(["-p singlefs-unit-graph --features gpu --test x", "-p alpha"]) == ("alpha", "singlefs-unit-graph")
                   and worker_peak_packages(["-p name@1.2.3 --lib"]) == ("name",) and worker_peak_packages(["--workspace"]) == (WORKER_PEAK_WORKSPACE_SCOPE,)
                   and worker_peak_packages(["-p a*"]) == (WORKER_PEAK_WORKSPACE_SCOPE,) and worker_peak_packages(["-p alpha", "--lib"]) == (WORKER_PEAK_WORKSPACE_SCOPE, "alpha"),
                   f"{worker_peak_packages(['-p tiny --lib -- adds'])}、{worker_peak_packages(['-p name@1.2.3 --lib'])}、{worker_peak_packages(['--workspace'])}、"
                   f"{worker_peak_packages(['-p a*'])}、{worker_peak_packages(['-p alpha', '--lib'])}")
    with worker_peak_scope(["-p inside"]):
        inside = WORKER_STATE.get("peak_scope")
    selftest.check("worker_peak_scope：判一格期间 WORKER_STATE[\"peak_scope\"] 是这一格点名的包，出来之后清成空", inside == ("inside",) and WORKER_STATE.get("peak_scope") == (),
                   f"之内 {inside}、之后 {WORKER_STATE.get('peak_scope')}")
    sample_root = os.path.join(work, "worker-peaks-sample-root")
    os.makedirs(sample_root, exist_ok=True)
    outside = {key: value for key, value in environment.items() if key not in (mutation_units.Library.client().TEMPORARY_SERVICE_VARIABLE, "GATE_STAGED_FROM")}
    decided = (worker_peaks_recorded_for(sample_root, outside), worker_peaks_recorded_for(STAGE_REPOSITORY, outside),
               worker_peaks_recorded_for(sample_root, dict(outside, GATE_STAGED_FROM=STAGE_REPOSITORY)), worker_peaks_recorded_for(sample_root, environment))
    selftest.check("样本根上的工作进程不写峰值记录：不在临时 prune-server 底下时，被判的是别处的根不写，是本模块所在的仓、或设了 GATE_STAGED_FROM 写；在临时服务底下（自证）写",
                   decided == (False, True, True, True), f"（别处的根、本仓、GATE_STAGED_FROM、临时服务底下）= {decided}")


def selftest_dispatch_order_cell(selftest, work):
    """派变异的次序（dispatch_order、rows_grouped_by_configuration，文件头「并发」）：六行按表序交进去，三个编译配置（-p a --lib 三行、-p b --test t1 两行、-p a --release --lib 一行）、
    两个 crate（a 四行、b 两行）、改的文件交错：先按编译配置归堆（行多的在前），堆内按 crate、改的文件、行号，得到 3、1、5、2、4、6，同一组行一行不少、不重；
    上一次用时不短于门槛的行先派不受影响（用时表里记第 4 行 900 秒：它排最前，别的照归堆）。
    弄坏开关 dispatch-by-crate-only（只按 crate 归堆，得到 1、3、5、6、2、4）下这一格判错。"""
    timings = os.path.join(work, "dispatch-timings")
    os.makedirs(timings)

    def row_of(number, path, arguments):
        return Row(number, f"m{number}", path, "a", "b", arguments, "t", (f"m{number}", path, "a", "b", arguments, "t"))
    rows = [row_of(1, "crates/a/src/f1.rs", "-p a --lib -- x"), row_of(2, "crates/b/src/g.rs", "-p b --test t1 -- y"),
            row_of(3, "crates/a/src/f0.rs", "-p a --lib -- x"), row_of(4, "crates/b/src/g.rs", "-p b --test t1 -- y"),
            row_of(5, "crates/a/src/f1.rs", "-p a --lib -- x"), row_of(6, "crates/a/src/f0.rs", "-p a --release --lib -- x")]
    ordered, moved = dispatch_order(rows, timings, 300, dict(selftest.base_environment))
    lines = [row.line_number for row in ordered]
    selftest.check("派变异的次序：六行（三个编译配置、两个 crate、改的文件交错）先按编译配置归堆、堆内按 crate 与改的文件与行号：3、1、5、2、4、6，一行不少不重，没有前移",
                   lines == [3, 1, 5, 2, 4, 6] and moved == 0, f"次序 {lines}，前移 {moved}")
    selftest_write(os.path.join(timings, row_content_hash(rows[3])), "seconds_running=900.0\n")
    ordered, moved = dispatch_order(rows, timings, 300, dict(selftest.base_environment))
    lines = [row.line_number for row in ordered]
    # 第 4 行挪走之后：-p a --lib 三行（3、1、5）最多排最前，另两个编译配置各一行，按配置的词比，-p a --release --lib（6）在 -p b --test t1（2）之前；
    # 只按 crate 归堆的次序是 1、3、5、6、2（同一个 crate 里按表序），与这一个不同
    selftest.check("上一次用时不短于门槛的行先派：第 4 行记 900 秒排最前（前移 1 条），别的行照归堆的次序 3、1、5、6、2（行 4 挪走后 -p a --lib 三行多的在前，另两个配置各一行按词比）",
                   lines == [4, 3, 1, 5, 6, 2] and moved == 1, f"次序 {lines}，前移 {moved}")


def selftest_peak_target_cells(selftest, work):
    """每个工作进程的峰值按测试目标估（文件头「内存上限」的「记在哪」「怎么除」）：包 pk 记了整个包的 28 GiB、测试目标 light 的 5 GiB、--lib 的 3 GiB（都在编译并行度 4 下）。
    ① 一格点名的测试目标那一级的记录名：-p pk --test light → pk--test--light，--lib → pk--lib，--bin x → pk--bin--x；点名几个包、几个目标、带通配的、没点名包的交 None；
    ② 这一趟要判的行估峰值取哪几个记录名（worker_peak_keys）：同一个目标的几行一个名字，没有目标那一级的取包名；
    ③ 估值：light 的行 5 GiB（不是包的 28 GiB），--lib 的行 3 GiB 且话里写包名，包都没量过的行取保守值（上限本身）；
    ④ 判一格期间采样线程要记的范围（worker_peak_scope）：包名加测试目标那一级的名字，出来清掉；
    ⑤ 从没量过（new_packages_of）、编译并行度 8 下没有实测（unmeasured_packages_at）都看目标那一级与它的包两处；
    ⑥ 没有实测的测试目标（另一份记录：包 pk 28 GiB，--lib 3、light 5、mid 4 GiB，low 只在编译并行度 1 记 100 GiB）：heavy 在编译并行度 4 与 2 下都取 3、4、5 的中位数 4 GiB；
    ⑦ 包里一份测试目标那一级的实测都没有：solo（包 28 GiB）与 lone（包只在编译并行度 1 有实测）取起步值 6.7 GiB，small（包 2 GiB）取包的，zz（从没量过）取保守值。
    弄坏开关 worker-peaks-unkeyed-by-target（一律取包名、不记目标那一级）下 ② ③ ④ 判错；unmeasured-target-borrows-package-peak（没有实测的测试目标借它的包）下 ⑥ ⑦ 判错。"""
    gibibyte = 1 << 30
    records = os.path.join(work, "peaks-by-target")
    environment = dict(selftest.base_environment)
    for index, (name, value) in enumerate((("pk", 28 * gibibyte), ("pk--test--light", 5 * gibibyte), ("pk--lib", 3 * gibibyte)), 1):
        record_worker_peak(records, name, 4, f"{index:020d}-1-selftest", value)
    peaks, source = selftest_peaks_read(records, environment)
    selftest.check("一格点名的测试目标那一级的记录名：-p pk --test light → pk--test--light，--lib → pk--lib，--bin x → pk--bin--x；点名几个包、几个目标、带通配、没点名包的、包名带 -- 的都是 None",
                   worker_peak_target_key("-p pk --test light -- a") == "pk--test--light" and worker_peak_target_key("-p pk --lib") == "pk--lib"
                   and worker_peak_target_key("-p pk --bin x --release") == "pk--bin--x"
                   and all(worker_peak_target_key(text) is None for text in ("-p a -p b --lib", "-p pk --lib --test t", "-p pk --tests", "--lib", "-p pk", "-p a--b --lib")),
                   f"{[worker_peak_target_key(text) for text in ('-p pk --test light -- a', '-p pk --lib', '-p pk --bin x --release', '-p a -p b --lib', '-p pk --tests', '-p a--b --lib')]}")
    keys = worker_peak_keys(["-p pk --test light -- a", "-p pk --test light -- b", "-p pk --test heavy -- c", "-p other"])
    selftest.check("这一趟要判的行估峰值取哪几个记录名：同一个目标的几行一个名字，没有目标那一级的取包名，排好序、去重",
                   keys == ("other", "pk--test--heavy", "pk--test--light"), f"{keys}")
    light, light_basis = worker_peak_for_jobs(peaks, worker_peak_keys(["-p pk --test light -- a"]), 4, 64 * gibibyte, environment)
    library, library_basis = worker_peak_for_jobs(peaks, worker_peak_keys(["-p pk --lib -- a"]), 4, 64 * gibibyte, environment)
    selftest.check("估值：light 的行取它自己那一级的 5 GiB（不是包的 28 GiB）；--lib 的行 3 GiB、话里写包名 pk",
                   light == 5 * gibibyte and "包 pk--test--light 最新 1 份" in light_basis and library == 3 * gibibyte and "包 pk 最新 1 份" in library_basis,
                   f"{light}：{light_basis}；{library}：{library_basis}")
    # ⑥ 没有实测的测试目标（unmeasured_target_peak）：另一份记录，包 pk 整个包 28 GiB，测试目标那一级 --lib 3 GiB、light 5 GiB、mid 4 GiB（编译并行度 4），
    #   low 只在编译并行度 1 记了 100 GiB（比 4 低，不借）。heavy 在 4 与 2 下都取 3、4、5 的中位数 4 GiB，不取包的 28 GiB、不取最大的 5 GiB、不把 low 算进去
    median_records = os.path.join(work, "peaks-by-target-median")
    for index, (name, jobs, value) in enumerate((("pk", 4, 28 * gibibyte), ("pk--lib", 4, 3 * gibibyte), ("pk--test--light", 4, 5 * gibibyte),
                                                  ("pk--test--mid", 4, 4 * gibibyte), ("pk--test--low", 1, 100 * gibibyte)), 1):
        record_worker_peak(median_records, name, jobs, f"{index:020d}-1-selftest", value)
    median_peaks, median_source = selftest_peaks_read(median_records, environment)
    heavy, heavy_basis = worker_peak_for_jobs(median_peaks, ("pk--test--heavy",), 4, 64 * gibibyte, environment)
    borrowed, borrowed_basis = worker_peak_for_jobs(median_peaks, ("pk--test--heavy",), 2, 64 * gibibyte, environment)
    selftest.check("没有实测的测试目标取同包测试目标那一级实测的中位数：包 pk 28 GiB，--lib 3 GiB、light 5 GiB、mid 4 GiB（编译并行度 4），low 只在编译并行度 1 记了 100 GiB；"
                   "没量过的 heavy 在编译并行度 4 与 2 下都取 4 GiB（3 个的中位数），不借包的 28 GiB、不取最大的 5 GiB、不把只有更低编译并行度的 low 算进去",
                   heavy == 4 * gibibyte and borrowed == 4 * gibibyte and "取包 pk 里 3 个测试目标那一级实测的中位数" in heavy_basis and "pk--test--low" not in heavy_basis,
                   f"编译并行度 4：{heavy}：{heavy_basis}；编译并行度 2：{borrowed}：{borrowed_basis}；{median_source}")
    # ⑦ 包里一份测试目标那一级的实测都没有：solo 整个包 28 GiB → 起步值（worker_peak_starting_bytes）；small 整个包 2 GiB 比起步值小 → 取包的；
    #   lone 整个包只在编译并行度 1 记了 1 GiB（4 下取不到包的）→ 起步值；zz 从没量过 → 照包算（保守值，上限本身）
    starting_records = os.path.join(work, "peaks-by-target-starting")
    for index, (name, jobs, value) in enumerate((("solo", 4, 28 * gibibyte), ("small", 4, 2 * gibibyte), ("lone", 1, gibibyte)), 1):
        record_worker_peak(starting_records, name, jobs, f"{index:020d}-1-selftest", value)
    starting_peaks, starting_source = selftest_peaks_read(starting_records, environment)
    estimates = {key: worker_peak_for_jobs(starting_peaks, (key,), 4, 64 * gibibyte, environment) for key in ("solo--test--x", "small--lib", "lone--test--y", "zz--test--z")}
    starting = worker_peak_starting_bytes()
    selftest.check("包里一份测试目标那一级的实测都没有：没量过的测试目标取起步值（solo 整个包 28 GiB → 起步值，不借包的；lone 只有编译并行度 1 的 1 GiB → 起步值）；"
                   "包的实测比起步值小（small 2 GiB）取包的；包从没量过（zz）照包算、取保守值（上限本身 64 GiB）",
                   starting == 67 * gibibyte // 10 and estimates["solo--test--x"][0] == starting and "取起步值" in estimates["solo--test--x"][1]
                   and estimates["lone--test--y"][0] == starting and "取起步值" in estimates["lone--test--y"][1]
                   and estimates["small--lib"][0] == 2 * gibibyte and "取包的" in estimates["small--lib"][1]
                   and estimates["zz--test--z"][0] == 64 * gibibyte and "从没量过" in estimates["zz--test--z"][1],
                   f"起步值 {starting}；{estimates}；{starting_source}")
    nothing, nothing_basis = worker_peak_for_jobs(peaks, worker_peak_keys(["-p unknown --test x -- a"]), 4, 64 * gibibyte, environment)
    selftest.check("包都没量过的行取保守值（上限本身 64 GiB），话里写从没量过与包名", nothing == 64 * gibibyte and "包 unknown--test--x 从没量过" not in nothing_basis and "从没量过" in nothing_basis,
                   f"{nothing}：{nothing_basis}")
    with worker_peak_scope(["-p pk --test light -- a", "-p pk --test light -- b"]):
        inside = WORKER_STATE.get("peak_scope")
    selftest.check("判一格期间采样线程要记的范围：包名加测试目标那一级的名字（pk、pk--test--light），出来清掉",
                   inside == ("pk", "pk--test--light") and WORKER_STATE.get("peak_scope") == (), f"{inside}、{WORKER_STATE.get('peak_scope')}")
    selftest.check("从没量过与编译并行度 8 下没有实测都看目标那一级与它的包两处：pk--test--never 的包 pk 有记录、不算从没量过，zz--test--x 两处都没有才算；pk--test--light 与 pk 只有编译并行度 4，在 8 下没有实测",
                   new_packages_of(peaks, ("pk--test--never", "zz--test--x")) == ("zz--test--x",)
                   and unmeasured_packages_at(peaks, ("pk--test--light", "pk--lib"), 8) == ("pk--test--light", "pk--lib")
                   and unmeasured_packages_at(peaks, ("pk--test--light", "pk--lib"), 4) == (), f"{new_packages_of(peaks, ('pk--test--never', 'zz--test--x'))}")
    selftest.check("读回来的峰值记录里有包 pk、pk--test--light、pk--lib 三份（目标那一级的名字照包名的写法读得回来）", sorted(peaks) == ["pk", "pk--lib", "pk--test--light"], f"{source}")


def selftest_record_tiny_peak(records, jobs, index, value):
    """交给 run_stage 的那几格预先放的峰值记录：包 tiny 与它的测试目标那一级 tiny--lib 各一份同样的读数（样本表的行都是 -p tiny --lib：
    估值先看 tiny--lib 自己，弄坏开关 worker-peaks-unkeyed-by-target 下只看 tiny；两处都放，那几格量的是开几个、不受取哪一级影响）。"""
    for name in ("tiny", "tiny--lib"):
        record_worker_peak(records, name, jobs, f"{index:020d}-1-selftest", value)


def selftest_peaks_read(records, environment):
    """selftest_peak_target_cells 读峰值记录用：measured_worker_peaks 的结果，读不了就让格判错而不是抛。"""
    peaks, source = measured_worker_peaks(records, environment)
    return ({} if peaks is None else peaks), source


def selftest_jobs_after_memory_cut_cell(selftest, work):
    """内存把工作进程从 4 个减到 1 个时的编译并行度（run_stage，文件头「内存上限」的「撞内存上限」）：包 tiny 在 4 个工作进程的编译并行度下记 10 GiB、在 1 个工作进程的编译并行度
    （整台线程数）下记 3 GiB，服务交的上限 15 GiB（这一台能给的内存 48G）、4 条：4 个装不下，减到 1 个；1 个那一档有实测、装得下，编译并行度照 1 个重取成整台线程数，不留在 4 个那一档；头一行写重取，
    收尾一行写每个 cargo 编译并行度是整台线程数。整台线程数与 4 个那一档相同（只有 1 个线程）时记本次未跑。
    弄坏开关 worker-jobs-held-after-memory-cut（一律留在减之前那一档）下这一格判错。"""
    gibibyte = 1 << 30
    environment = dict(selftest.base_environment)
    budget = compile_thread_budget(environment)
    wanted_jobs, rederived_jobs = cargo_jobs_for(environment, 4, budget), cargo_jobs_for(environment, 1, budget)
    if wanted_jobs == rederived_jobs:
        selftest.skip("内存把工作进程从 4 个减到 1 个时编译并行度照 1 个重取", f"整台线程数 {budget} 下 4 个与 1 个工作进程的编译并行度都是 {wanted_jobs}，造不出差别")
        return
    repository = os.path.join(work, "jobs-after-cut")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    stage_changes = dict(GATE_MUTATION_RUN_RECORDS_DIRECTORY=os.path.join(work, "jobs-after-cut-run-records"))
    records = run_records_directory(repository, dict(selftest.base_environment, **stage_changes))
    selftest_record_tiny_peak(records, wanted_jobs, 1, 10 * gibibyte)
    selftest_record_tiny_peak(records, rederived_jobs, 2, 3 * gibibyte)
    # 这一台能给的内存 48G（多机配置 SLICE_MEMORY_CAP_BY_HOST 这一台那一项）：开 4 个照它除，服务交的上限 15 GiB 是硬顶，4 个装不下往下减
    configuration = os.path.join(work, "jobs-after-cut-config", "multi-host.env")
    selftest_write(configuration, "HOSTS=127.0.0.1 192.0.2.70\nCARGO_BIN_DIRECTORY_BY_HOST=127.0.0.1=/selftest/this-cargo 192.0.2.70=/selftest/other-cargo\n"
                                  "BUILD_ROOT_BY_HOST=127.0.0.1=/selftest/this-builds 192.0.2.70=/selftest/other-builds\n"
                                  "SLICE_MEMORY_CAP_BY_HOST=127.0.0.1=48G 192.0.2.70=8G\nQUIESCE_STOP_COMMAND=\nQUIESCE_STOPPED_CHECK_COMMAND=\nQUIESCE_START_COMMAND=\n"
                                  f"QUIESCE_STARTED_CHECK_COMMAND=\nENABLE_ACROSS_MACHINES=0\nENABLE_GPU=0\nNFS_SHARED_ROOTS={os.path.dirname(configuration)}\n")
    exit_code, stdout, stderr, _invocations = selftest.run(
        repository, GATE_MUTATION_WORKERS="4", GATE_MUTATION_TARGET_DIR=os.path.join(work, "jobs-after-cut-target"), GATE_MUTATION_MULTI_HOST_CONFIG=configuration,
        **stage_changes, **{TASK_MEMORY_CAP_VARIABLE: str(15 * gibibyte)})
    selftest.check(f"内存把工作进程从 4 个减到 1 个：编译并行度照 1 个重取成 {rederived_jobs}（那一档有实测 3 GiB、装得下），不留在 4 个那一档的 {wanted_jobs}；头一行写重取，收尾一行写编译并行度 {rederived_jobs}",
                   exit_code == 0 and "因为内存从 4 个减到了 1 个" in stderr and f"编译并行度照 1 个重取成 {rederived_jobs}" in stderr
                   and f"每个 cargo 编译并行度 {rederived_jobs}" in stderr, f"退 {exit_code}；stdout「{stdout[-300:]}」；stderr「{stderr[:1500]}」")


def selftest_worker_growth_cell(selftest, work):
    """跑着加开（文件头「内存上限」的「跑着加开」）：样本表 4 行（-p tiny --lib，合成一批基线），测试目标那一级 tiny--lib 在 1 个工作进程的编译并行度（整台线程数）下记 28 GiB，
    服务交的上限 30 GiB：开跑时只开 1 个（30 ÷ 28）。采样线程的读数换成 2 GiB（GATE_MUTATION_SELFTEST_WORKER_BYTES：造「头一格实测 2 GiB」），假 cargo 每次睡 2 秒
    （FAKE_CARGO_SPAN_LOG 记起止）：头一格（那一批基线）判完，n × 2 GiB + 父进程自己不超过 30 GiB、上界是条数 4，定下加到 4 个、换进程池，
    编译并行度照 4 个重取（不高过开跑时那一档）；后面派出的格同时在跑的 cargo 至少 2 个，4 行都判成抓到，收尾一行带「跑着加开」。
    弄坏开关 workers-not-grown-after-measuring（不加开）下这一格判错（同时在跑的至多 1 个、没有「跑着加开」）。"""
    gibibyte = 1 << 30
    environment = dict(selftest.base_environment)
    budget = compile_thread_budget(environment)
    start_jobs = cargo_jobs_for(environment, 1, budget)
    grown_jobs = min(start_jobs, cargo_jobs_for(environment, 4, budget))
    repository = os.path.join(work, "worker-growth")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    stage_changes = dict(GATE_MUTATION_RUN_RECORDS_DIRECTORY=os.path.join(work, "worker-growth-run-records"))
    record_worker_peak(run_records_directory(repository, dict(environment, **stage_changes)), "tiny--lib", start_jobs, f"{1:020d}-1-selftest", 28 * gibibyte)
    spans = os.path.join(work, "worker-growth-spans.log")
    with contextlib.suppress(FileNotFoundError):
        os.unlink(spans)
    exit_code, stdout, stderr, _invocations = selftest.run(
        repository, GATE_MUTATION_WORKERS="16", GATE_MUTATION_TARGET_DIR=os.path.join(work, "worker-growth-target"),
        GATE_MUTATION_SELFTEST_WORKER_BYTES=str(2 * gibibyte), FAKE_CARGO_SPAN_LOG=spans, FAKE_CARGO_SPAN_SLEEP="2",
        **stage_changes, **{TASK_MEMORY_CAP_VARIABLE: str(30 * gibibyte)})
    opened, most = set(), 0
    with contextlib.suppress(OSError):
        with open(spans, encoding="utf-8") as handle:
            for line in handle.read().splitlines():
                fields = line.split("\t")
                if len(fields) != 3:
                    continue
                if fields[0] == "start":
                    opened.add(fields[1])
                    most = max(most, len(opened))
                else:
                    opened.discard(fields[1])
    selftest.check("跑着加开：起步 1 个（测试目标 tiny--lib 记 28 GiB、上限 30 GiB），头一格实测 2 GiB：判完头一格从 1 个工作进程加到 4 个、编译并行度照 4 个重取，"
                   "后面派出的格同时在跑的 cargo 至少 2 个，4 行都判成抓到，收尾一行带「跑着加开」",
                   exit_code == 0 and "这一轮开 1 个工作进程" in stderr and "从 1 个工作进程加到 4 个" in stderr
                   and f"（每个 cargo 编译并行度 {start_jobs} → {grown_jobs}）" in stderr and stderr.count("跑着加开：") >= 3
                   and most >= 2 and len(per_row_how(stdout)) == 4,
                   f"退 {exit_code}；同时在跑的 cargo 最多 {most}；抓到 {len(per_row_how(stdout))} 行；stderr「{stderr[-2500:]}」")


def selftest_disk_cells(selftest, work):
    """磁盘余量（文件头「磁盘余量」）：每个工作进程最多占两份编译目录（普通、插桩）、各涨到上限，派活前按目标盘余量定工作进程数。
    先直接调 worker_count_for 与 disk_bound_for：GATE_MUTATION_WORKERS 本来给 16 个，假的余量 100G、上限 10G、备用 5G，每个工作进程要 20G、开 2 个以上另加先编一份依赖的 20G，
    放得下 3 个（共 80G），头一行写明卡在磁盘余量；已有的编译目录只算还能涨的（片号 1 的普通编译目录已占 8M，上限 10M、余量 100M、备用 1M：放得下 4 个，没有已有目录是 3 个）。
    再起一趟 run_stage：假的余量 70G、备用 10G，开的工作进程数按余量收成 2 个（条数 4 放得下更多），头一行写明；余量写法不认的整道拒；
    余量设成空（外层没给）时读编译目录所在的盘，头一行写明来源。弄坏开关 disknotchecked（不看磁盘）下 run_stage 那一格与直接调的两格判错。"""
    environment = dict(selftest.base_environment)
    sixteen = dict(environment, GATE_MUTATION_WORKERS="16")
    sized = dict(sixteen, **{DISK_FREE_VARIABLE: "100G", TARGET_DIRECTORY_CAP_VARIABLE: "10G", DISK_RESERVE_VARIABLE: "5G"})
    shared = os.path.join(work, "disk-target")
    count, head = worker_count_for(20, sized, disk=lambda ceiling: disk_bound_for(shared, sized, ceiling, None))
    selftest.check("磁盘余量放不下 16 份：GATE_MUTATION_WORKERS 本来给 16 个，余量 100G（上限 10G、备用 5G、每个工作进程两份共 20G、开 2 个以上另加先编一份依赖的两份 20G）按余量收成 3 个，头一行写明卡在磁盘余量与放得下几个",
                   count == 3 and "卡在「磁盘余量」这一项" in head and "磁盘余量 100.0 GiB" in head and "3 个共还要 80.0 GiB，再多 1 个要 100.0 GiB、放不下" in head,
                   f"{count}：{head}")
    tiny = dict(sixteen, **{DISK_FREE_VARIABLE: "100M", TARGET_DIRECTORY_CAP_VARIABLE: "10M", DISK_RESERVE_VARIABLE: "1M"})
    absent = os.path.join(work, "disk-target-absent")
    present = os.path.join(work, "disk-target-present")
    selftest_write_random(os.path.join(present, "debug", "deps", "blob.rlib"), 8 << 20)
    absent_count, _head = disk_bound_for(absent, tiny, 9, None)
    present_count, present_head = disk_bound_for(present, tiny, 9, None)
    selftest.check("已有的编译目录只算还能涨的：片号 1 的普通编译目录已占 8M（上限 10M），余量 100M、备用 1M 放得下 4 个；没有已有目录是 3 个（开 2 个以上另加先编一份依赖的两份）",
                   absent_count == 3 and present_count == 4, f"没有已有目录 {absent_count}；片号 1 已占 8M {present_count}：{present_head}")
    repository = os.path.join(work, "disk-workers")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    target = os.path.join(work, "disk-workers-target")
    exit_code, _stdout, stderr, _ = selftest.run(
        repository, GATE_MUTATION_WORKERS="16", GATE_MUTATION_TARGET_DIR=target,
        **{DISK_FREE_VARIABLE: "70G", TARGET_DIRECTORY_CAP_VARIABLE: "10G", DISK_RESERVE_VARIABLE: "10G"})
    selftest.check("磁盘余量交给 run_stage：开的工作进程数按余量收（余量 70G、备用 10G、每个工作进程 20G、先编一份依赖的两份 20G 开 2 个，不是条数 4），头一行写明卡在磁盘余量",
                   exit_code == 0 and "这一轮开 2 个工作进程" in stderr and "卡在「磁盘余量」这一项" in stderr, f"退 {exit_code}；{stderr[:900]}")
    exit_code, stdout, stderr, _ = selftest.run(selftest_new_library_copy(work, repository, "disk-workers-unwritten"), GATE_MUTATION_TARGET_DIR=target,
                                                **{DISK_FREE_VARIABLE: "lots"})
    selftest.check("磁盘余量的写法不认：整道拒、带出路，一条变异都没跑", exit_code == 1 and f"{DISK_FREE_VARIABLE} 写成了「lots」" in stdout + stderr,
                   f"退 {exit_code}；{(stdout + stderr)[:500]}")
    exit_code, _stdout, stderr, _ = selftest.run(selftest_new_library_copy(work, repository, "disk-workers-unset"), GATE_MUTATION_TARGET_DIR=target,
                                                 **{DISK_FREE_VARIABLE: ""})
    selftest.check("外层没给磁盘余量时读编译目录所在的盘：头一行写来源，照常跑完", exit_code == 0 and "编译目录所在的盘此刻可用" in stderr, f"退 {exit_code}；{stderr[:700]}")
