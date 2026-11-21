"""crates-mutation-rows.py 拆出的一份。
工作进程的停法、片号与编译目录（共用目录、片号锁、占盘、磁盘余量、上限清理），连同这几样的自证格。
入口文件头写着这一份在的位置、弄坏开关表与用法；入口 crates-mutation-rows.py 从各份 import 全部名字再导出，别处照旧按入口取名字。
拆出的各份只向前 import（base → judging → support → static → units → baseline → targets → memory → runs → pool → plan → stage → commands → driver），没有环。
"""
import contextlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
from crates_mutation_rows_base import (COVERAGE_TARGET_SUFFIX, DEFAULT_DISK_RESERVE, DEFAULT_TARGET_DIRECTORY_CAP, DISK_FREE_VARIABLE,
    DISK_RESERVE_VARIABLE, INCREMENTAL_DIRECTORY_NAME, MODULE_PATH, RUN_BUILD_DIRECTORY_NAME, RUN_BUILD_REMOVAL_FAILURES, SEED_TARGET_SUFFIX,
    SHARD_ROOT_PREFIX, STAGE_CHILD_PROGRAM, TARGET_DIRECTORY_CAP_VARIABLE, TARGET_DIRECTORY_NAME, WORKER_SLOT_LOCK_SUFFIX, WORKER_SLOT_SEARCH_LIMIT,
    WORKER_STATE, WORKER_TARGET_LOCK_NAME, admission_module, break_is, bytes_of_size, format_size, note, write_atomically)
from crates_mutation_rows_support import (build_selftest_repository, selftest_new_library_copy, selftest_write)
from crates_mutation_rows_units import (library_of, run_records_directory)


class StopRequested(SystemExit):
    """run_pool 收到 STOP_SIGNALS 里的信号：照 SystemExit 往外抛（退出码 128 + 信号号），一路上的 finally 都跑。"""


def raise_stop_requested(signal_number, _frame):
    raise StopRequested(128 + signal_number)


@contextlib.contextmanager
def own_workers_stopped_on_stop(pool, shard_root, environment):
    """收到停的信号时，在进程池等工作进程收工之前先停自己起的那几个工作进程（只对它们的进程号发 TERM）、等它们退出（每个至多 60 秒），
    再删这一趟自己建的编译目录（remove_owned_build_root：核过没有进程开着里面的文件）与 shard_root，再往外抛。编译目录没删成就连同 shard_root 留着。"""
    try:
        yield
    except StopRequested:
        processes = list((getattr(pool, "_processes", None) or {}).values())
        for process in processes:
            with contextlib.suppress(OSError, ValueError):
                process.terminate()
        for process in processes:
            with contextlib.suppress(OSError, ValueError, AssertionError):
                process.join(60)
        if remove_owned_build_root(shard_root, environment):
            shutil.rmtree(shard_root, ignore_errors=True)
        else:
            note(f"  · 这一趟的临时目录 {shard_root} 留着（里面的编译目录没删）")
        raise


def remove_owned_build_root(shard_root, environment):
    """删这一趟自己建的 <shard_root>/build（文件头「编译目录」；调用方给了 GATE_MUTATION_TARGET_DIR 时没有它）：交回删了或本来就没有。
    没删成（有进程开着里面的文件、删不成）记进 RUN_BUILD_REMOVAL_FAILURES（run_stage 判红）。弄坏开关 runtargetkept：不删、交回 False。"""
    build_root = os.path.join(shard_root, RUN_BUILD_DIRECTORY_NAME)
    if not os.path.lexists(build_root):
        return True
    if break_is(environment, "runtargetkept"):
        note(f"  ! 弄坏开关 GATE_MUTATION_BREAK=runtargetkept：这一趟的编译目录 {build_root} 不删")
        return False
    removed, removal_lines = remove_run_build_root(build_root, environment)
    for line in removal_lines:
        note(line)
    if not removed:
        RUN_BUILD_REMOVAL_FAILURES.append(build_root)
    return removed



def deferred_write_library(library):
    """弄坏开关 records-written-at-end 用：把真库的 write、write_json 换成先攒在这个工作进程的内存里，进程池正常收尾（工作进程正常退出时 multiprocessing 的退出终结器）才一并写进真库；
    读与别的方法转给真库。工作进程被硬杀时攒着的全丢（被怀疑过的样子：整组被杀、判过的格不留下）。
    类写在函数体里、不做模块级的类。"""
    import multiprocessing.util as exit_hooks

    pending = []
    write_now, write_json_now = library.write, library.write_json

    def flush():
        for write, kind, name, value in pending:
            write(kind, name, value)
        pending.clear()

    library.write = lambda kind, name, fields: pending.append((write_now, kind, name, fields))
    library.write_json = lambda kind, name, value: pending.append((write_json_now, kind, name, value))
    exit_hooks.Finalize(None, flush, exitpriority=10)
    return library


def crash_domain_narrowing(environment):
    """这一趟缩没缩崩溃域（文件头「缩过域的一趟」）：环境里设了的缩域变量逐个写成「名=值」、顿号连起来；没缩域交空串。
    变量名单与「设了就算、空值也算」的认法同 scripts/gate/admission.py 的 crash_case_narrowing_settings（名单 CRASH_CASE_NARROWING_VARIABLES，
    门禁 crash-point-replay 收尾写缩过域那一判同一份），这里不另列。门禁这一道（run_stage、row-plan）与 scripts/mutation/prove-red.sh 都经这一个函数判。"""
    names = admission_module().CRASH_CASE_NARROWING_VARIABLES
    return "、".join(f"{name}={environment[name]}" for name in names if name in environment)


def narrowed_outcome_text(text, narrowing):
    """缩过域的一趟里现判出的一行结局（文件头「缩过域的一趟」）：头一行末尾接「；缩过域（<变量=值>）、不算门禁通过、没入库」，后面的尾巴不动。"""
    first, separator, rest = text.partition("\n")
    return f"{first}；缩过域（{narrowing}）、不算门禁通过、没入库{separator}{rest}"


def narrowed_sentinel_off_reason(narrowing):
    """缩过域的一趟不抽复用哨兵的原因（文件头「缩过域的一趟」）：门禁这一道与 prove-red.sh 打同一句。"""
    return f"缩过域（{narrowing}）：强制现判出的是缩域下的结论，拿它比库里全域下判的抓到判不出复用对不对"


def narrowed_sharding_refusal(narrowing):
    """缩过域的一趟进了服务分片（文件头「缩过域的一趟」）：(红句, 出路)。row-plan 与分片的各片、合并那一步的 run_stage 同一句。"""
    return (f"缩过域（{narrowing}）的变异不分片：缩域判出的不入库，合并那一步按库核各片判的行核不到，一行都没列、一条都没跑",
            f"点名几行缩域证红用 prove-red.sh：task-server/task submit -- env {' '.join(narrowing.split('、'))} bash scripts/mutation/prove-red.sh --copy <仓副本> <crate> <变异名…>"
            "（不分片，成功句写缩过域、不算门禁通过、现判的不入库）；要门禁结论去掉缩域变量、范围声明的为什么一段写「全域理由：…」，"
            "再交 bash .claude/gate.d/crates-mutation-replay.sh --item <行号或变异名>（服务分片判）")


def unwritten_library(library):
    """缩过域的一趟用（文件头「缩过域的一趟」）：把这个工作进程那一份库的 write、write_json 换成一样都不写（判定、基线、覆盖），读与别的方法照真库。
    写在函数体里、不做模块级的类。"""
    library.write = lambda kind, name, fields: None
    library.write_json = lambda kind, name, value: None
    return library
    # 源码副本建在父进程的 shard_root 底下、由父进程收尾时整个删；编译目录留着跨轮复用。
    # 不在这里挂 atexit：进程池的工作进程走 os._exit 退出，atexit 在这里一次都不跑。


def slot_lock_path(shared, sequence):
    """片号 sequence 的锁文件：第 1 片 <共用目录>.slot，其余片 <共用目录>-w<片号>.slot，都在共用编译目录那一层里。"""
    base = os.path.normpath(shared)
    return base + WORKER_SLOT_LOCK_SUFFIX if sequence == 1 else f"{base}-w{sequence}{WORKER_SLOT_LOCK_SUFFIX}"


def claim_worker_slot(shared, local_counter, environment):
    """领片号：从 1 往上，对每一片的锁文件试拿排他 flock（不阻塞），拿到第一个空闲的就是这个工作进程的片号。回 (片号, 锁文件句柄或 None)。
    锁文件拿在句柄上，工作进程退出内核放；同一台机器上并发的几次调用因此拿到不相交的片号集合，缓存目录（-w<片号>）照旧跨轮复用。
    锁文件建不了、或 WORKER_SLOT_SEARCH_LIMIT 片都被占：报一行，退回本地序号（本调用内不重，跨调用可能重）。
    弄坏开关 local-shard-numbering：不看锁文件，从本调用的共享计数器现领序号。"""
    import fcntl
    if not break_is(environment, "local-shard-numbering"):
        reason = None
        try:
            os.makedirs(os.path.dirname(os.path.normpath(shared)) or ".", exist_ok=True)
            for sequence in range(1, WORKER_SLOT_SEARCH_LIMIT + 1):
                handle = open(slot_lock_path(shared, sequence), "a")
                try:
                    fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
                except OSError:
                    handle.close()
                    continue
                return sequence, handle
            reason = f"前 {WORKER_SLOT_SEARCH_LIMIT} 片的锁都被占着"
        except OSError as error:
            reason = f"锁文件拿不了（{error}）"
        note(f"  ! {reason}：片号退回本调用内的本地序号，同一台机器上并发的别的调用可能撞同一个 -w<片号>、被 cargo 的目录锁互相排队")
    with local_counter.get_lock():
        local_counter.value += 1
        return local_counter.value, None


def default_shared_target_directory(environment):
    """调用方给的第 1 片编译目录（其余片加 -w<片号>，记覆盖的再加 -coverage）：GATE_MUTATION_TARGET_DIR 设了交它（跑完不删）；没设而环境里给了
    CARGO_TARGET_DIR（统一任务服务给这一次尝试的那一份，或调用方给的）交 <它>/singlefs-crates-mutation-target（同样跑完不删）；都没设交 None，
    run_pool 在这一趟的 <shard_root>/build 里建（run_build_target_directory）、收尾删。弄坏开关 targetdefaulttmp：没设时交
    ${TMPDIR:-/tmp}/singlefs-crates-mutation-target（跨轮共用、不删）。"""
    explicit = environment.get("GATE_MUTATION_TARGET_DIR", "").strip()
    if explicit:
        return explicit
    # 环境里给的 CARGO_TARGET_DIR（文件头「编译目录」；统一任务服务给这一次尝试的那一份）：照调用方给的那样用、跑完不删。变量名写在函数里、不做模块级常量。
    #   弄坏开关 service-build-directory-ignored：不认它
    service = environment.get("CARGO_TARGET_DIR", "").strip()
    if service and not break_is(environment, "service-build-directory-ignored"):
        return os.path.join(os.path.realpath(service), TARGET_DIRECTORY_NAME)
    if break_is(environment, "targetdefaulttmp"):
        return os.path.join(environment.get("TMPDIR") or "/tmp", TARGET_DIRECTORY_NAME)
    return None


def run_build_target_directory(shard_root):
    """这一趟自己建的第 1 片编译目录：<shard_root>/build/singlefs-crates-mutation-target（别的片与先编那一份都是它在 build 里的兄弟）。"""
    return os.path.join(shard_root, RUN_BUILD_DIRECTORY_NAME, TARGET_DIRECTORY_NAME)


def planned_run_build_target_directory():
    """run_pool 还没建 shard_root 时，磁盘余量按它将来所在的盘算（$TMPDIR 底下；路径此刻不在，disk_free_bytes 往上找到 $TMPDIR）。"""
    return os.path.join(tempfile.gettempdir(), SHARD_ROOT_PREFIX + "planned", RUN_BUILD_DIRECTORY_NAME, TARGET_DIRECTORY_NAME)


def remove_run_build_root(build_root, environment):
    """删这一趟自己建的 <shard_root>/build（文件头「编译目录」）：删之前核本机没有进程开着里面的文件。回 (删成了没有, [要打的行])。
    弄坏开关 runtargetopenfilesunchecked：不核开着的文件就删。"""
    import run_build_directory   # scripts/hosts/run_build_directory.py（模块开头已把 scripts/hosts 放进 sys.path）
    return run_build_directory.remove_run_build_directory(build_root, check_open=not break_is(environment, "runtargetopenfilesunchecked"))


def hold_worker_target_lock(target_directory):
    """工作进程在自己的编译目录里拿共享锁、拿着到进程退出；建不了只报不停（回收那一侧还有按进程认的那几样）。"""
    import fcntl
    try:
        os.makedirs(target_directory, exist_ok=True)
        handle = open(os.path.join(target_directory, WORKER_TARGET_LOCK_NAME), "a")
        fcntl.flock(handle, fcntl.LOCK_SH)
        return handle
    except OSError as error:
        note(f"  ! 编译目录 {target_directory} 里拿不到在用的标记 {WORKER_TARGET_LOCK_NAME}（{error}）：别的一趟回收时只按进程认它在不在用")
        return None


def path_mentioned_in(text, path):
    """text 里出现 path 本身或它底下的路径（后面接 / 或结尾，或不是路径名里的字符）：-w3 不算 -w30、也不算 -w3-coverage。"""
    return re.search(re.escape(path) + r"(?![A-Za-z0-9_.-])", text) is not None


def processes_using_directories(directories):
    """{目录: [「进程 <pid>（<命令>）的<哪一样>」…]}：工作目录在它底下、命令行里有它或它底下的路径、环境里 CARGO_TARGET_DIR 指到它底下的进程。
    读不了的进程（刚退出、别的用户的）跳过；本进程不算。"""
    forms = {directory: {os.path.normpath(directory), os.path.realpath(directory)} for directory in directories}
    users = {directory: [] for directory in directories}
    for entry in os.listdir("/proc"):
        if not entry.isdigit() or int(entry) == os.getpid():
            continue
        base = os.path.join("/proc", entry)
        try:
            with open(os.path.join(base, "cmdline"), "rb") as handle:
                arguments = [word.decode(errors="replace") for word in handle.read().split(b"\0") if word]
        except OSError:
            continue
        try:
            working_directory = os.readlink(os.path.join(base, "cwd"))
        except OSError:
            working_directory = None
        target_variable = None
        try:
            with open(os.path.join(base, "environ"), "rb") as handle:
                for item in handle.read().split(b"\0"):
                    if item.startswith(b"CARGO_TARGET_DIR="):
                        target_variable = item[len(b"CARGO_TARGET_DIR="):].decode(errors="replace")
        except OSError:
            pass
        if target_variable and not os.path.isabs(target_variable) and working_directory:
            target_variable = os.path.join(working_directory, target_variable)
        shown = " ".join(arguments)[:120]
        for directory, paths in forms.items():
            reasons = []
            if working_directory and any(working_directory == path or working_directory.startswith(path + os.sep) for path in paths):
                reasons.append("工作目录")
            if any(path_mentioned_in(argument, path) for argument in arguments for path in paths):
                reasons.append("命令行")
            if target_variable and any(os.path.normpath(target_variable) == path or os.path.normpath(target_variable).startswith(path + os.sep)
                                       for path in paths):
                reasons.append("CARGO_TARGET_DIR")
            if reasons:
                users[directory].append(f"进程 {entry}（{shown}）的{'、'.join(reasons)}")
    return users


def worker_lock_is_held(target_directory):
    """那份编译目录里的在用标记被别的进程拿着：(True, None)；没被拿着：(False, 拿着排他锁的文件或 None)，调用方删完再关。"""
    return file_lock_is_held(os.path.join(target_directory, WORKER_TARGET_LOCK_NAME))


def file_lock_is_held(path):
    """path 这把锁被别的进程拿着：(True, None)；没被拿着（或文件不在、打不开）：(False, 拿着排他锁的文件或 None)，调用方删完再关。不建文件。"""
    import fcntl
    if not os.path.exists(path):
        return False, None
    try:
        handle = open(path, "a")
    except OSError:
        return False, None
    try:
        fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except OSError:
        handle.close()
        return True, None
    return False, handle


def allocated_bytes_of_tree(directory):
    total = 0
    for current_directory, subdirectory_names, file_names in os.walk(directory):
        for name in [*file_names, *subdirectory_names]:
            with contextlib.suppress(OSError):
                total += os.lstat(os.path.join(current_directory, name)).st_blocks * 512
    return total


def reclaim_extra_target_directories(shared, worker_count, environment):
    """开跑时回收片号比这一趟的工作进程数大的 -w<片号> 编译目录（连同它的 -coverage）：编译目录数按这一趟的工作进程数复用，多出来的删掉。
    删之前核没人在用：有进程的工作目录、命令行或 CARGO_TARGET_DIR 指到它，或它（记覆盖的看它那一片）的在用标记被别的进程拿着，就留着、写明谁在用。
    回 (删掉的 [(路径, 占盘字节)], 留着的 [(路径, 谁在用)], 删不成的 [(路径, 原因)])。
    弄坏开关 extratargetskept：一份都不回收；inusetargetremoved：不核在不在用，多出来的照删。"""
    removed, kept, failed = [], [], []
    if break_is(environment, "extratargetskept"):
        return removed, kept, failed
    parent, base = os.path.split(os.path.normpath(shared))
    form = re.compile(re.escape(base) + r"-w([1-9][0-9]*)(" + re.escape(COVERAGE_TARGET_SUFFIX) + r")?$")
    try:
        names = os.listdir(parent or ".")
    except OSError:
        return removed, kept, failed
    extra = sorted(((int(match.group(1)), name) for name in names
                    for match in [form.match(name)] if match and int(match.group(1)) > worker_count and os.path.isdir(os.path.join(parent, name))))
    if not extra:
        return removed, kept, failed
    paths = [os.path.join(parent, name) for _sequence, name in extra]
    check_users = not break_is(environment, "inusetargetremoved")
    users = processes_using_directories(paths) if check_users else {path: [] for path in paths}
    for (sequence, _name), path in zip(extra, paths):
        reasons = list(users[path])
        lock_handles = []
        if check_users:
            held, lock_handle = worker_lock_is_held(os.path.join(parent, f"{base}-w{sequence}"))
            lock_handles.append(lock_handle)
            if held:
                reasons.append(f"-w{sequence} 的在用标记 {WORKER_TARGET_LOCK_NAME} 被别的进程拿着（有一趟的工作进程在用）")
            # 片号锁（claim_worker_slot）：别的调用的工作进程刚领到这一片、还没来得及拿在用标记时也留着；拿到的话删的时候一直拿着，删完再放
            held, lock_handle = file_lock_is_held(slot_lock_path(shared, sequence))
            lock_handles.append(lock_handle)
            if held:
                reasons.append(f"-w{sequence} 的片号锁 {os.path.basename(slot_lock_path(shared, sequence))} 被别的进程拿着（有一趟的工作进程领了这一片）")
        try:
            if reasons:
                kept.append((path, "；".join(reasons)))
                continue
            size = allocated_bytes_of_tree(path)
            shutil.rmtree(path)
            removed.append((path, size))
        except OSError as error:
            failed.append((path, str(error)))
        finally:
            for lock_handle in lock_handles:
                if lock_handle is not None:
                    lock_handle.close()
    return removed, kept, failed


def target_reclaim_lines(shared, worker_count, outcome, owned):
    removed, kept, failed = outcome
    lines = [f"  … 编译目录：{shared}（第 1 片；其余片加 -w<片号>，记覆盖的再加 {COVERAGE_TARGET_SUFFIX}；"
             + ("这一趟自己建的，跑完连同 build 整个删" if owned else "调用方给的 GATE_MUTATION_TARGET_DIR，跑完不删") + "）；"
             + (f"回收了片号大于这一趟 {worker_count} 个工作进程的 {len(removed)} 份，共 {format_size(sum(size for _path, size in removed))}"
                f"（{'、'.join(os.path.basename(path) for path, _size in removed)}）" if removed
                else f"片号大于这一趟 {worker_count} 个工作进程、要回收的 0 份")
             + (f"；有进程在用、没删 {len(kept)} 份" if kept else "") + (f"；删不成 {len(failed)} 份" if failed else "")]
    lines += [f"    没删 {path}：{reasons}" for path, reasons in kept]
    lines += [f"    删不成 {path}：{reason}（下一趟开跑时再试；急着要空间就核没人在用之后手删）" for path, reason in failed]
    return lines


def size_setting_problems(environment):
    """GATE_MUTATION_TARGET_DIR_CAP、GATE_MUTATION_DISK_FREE、GATE_MUTATION_DISK_RESERVE 里写法不认的：[(卡在哪, 下一步)]。"""
    problems = []
    for variable in (TARGET_DIRECTORY_CAP_VARIABLE, DISK_FREE_VARIABLE, DISK_RESERVE_VARIABLE):
        text = environment.get(variable, "").strip()
        if text and bytes_of_size(text) is None:
            problems.append((f"{variable} 写成了「{text}」，不是 systemd 的写法",
                             f"写成正整数加 K / M / G / T，单位必写，例 64G（{variable} 的意思见 scripts/mutation/crates-mutation-rows.py 文件头「磁盘余量」「编译目录上限」），或者不设它。"))
    return problems


def target_directory_cap_bytes(environment):
    """每份编译目录的上限字节（GATE_MUTATION_TARGET_DIR_CAP，默认 DEFAULT_TARGET_DIRECTORY_CAP）。"""
    return bytes_of_size(environment.get(TARGET_DIRECTORY_CAP_VARIABLE, "").strip() or DEFAULT_TARGET_DIRECTORY_CAP)


def worker_target_directories(shared, sequence):
    """片号 sequence 的 (普通编译目录, 插桩编译目录)：第 1 片用共用的，其余片加 -w<片号>，插桩的再加 -coverage（与 prepare_shard、judge_coverage 同一套）。"""
    plain = shared if sequence == 1 else f"{shared}-w{sequence}"
    return plain, plain + COVERAGE_TARGET_SUFFIX


def disk_free_bytes(shared, environment):
    """目标盘（编译目录所在那块）此刻的余量：(字节或 None, 说明或没读到的原因)。外层给了 GATE_MUTATION_DISK_FREE 用它，不然读编译目录往上第一层存在的目录所在的盘。"""
    given = environment.get(DISK_FREE_VARIABLE, "").strip()
    if given:
        value = bytes_of_size(given)
        if value is None:
            return None, f"{DISK_FREE_VARIABLE} 写成「{given}」，不是正整数加 K / M / G / T"
        return value, f"外层给的 {DISK_FREE_VARIABLE}={given}"
    probe = os.path.abspath(shared)
    while not os.path.exists(probe) and os.path.dirname(probe) != probe:
        probe = os.path.dirname(probe)
    try:
        status = os.statvfs(probe)
    except OSError as error:
        return None, f"外层没给 {DISK_FREE_VARIABLE}，读编译目录所在的盘也不成（{type(error).__name__}：{error}）"
    return status.f_bavail * status.f_frsize, "编译目录所在的盘此刻可用"


def disk_bound_for(shared, environment, ceiling, peaks):
    """磁盘余量放得下几个工作进程（文件头「磁盘余量」）：(个数或 None, 头一行要写的话)。
    每个工作进程最多占两份编译目录（普通、插桩），每份涨到「每份的涨幅」为止：片号 1…n 各份还能涨的量（涨幅减去此刻已占的，目录还没建的是整个涨幅）
    加起来；n ≥ 2 时先编一份依赖的 -seed、-seed-coverage 两份也按同样的算法加上（每一趟新编）；再加备用 DISK_RESERVE，不超过目标盘余量的最大 n；
    n 从 1 数到 ceiling（前几项取最小的那个数，再多开也用不上）。一个都放不下交 0（调用方按 1 个开）。
    每份的涨幅：peaks（measured_target_peaks 交的 {"普通": 字节或 None, "插桩": 字节或 None}，运行记录目录里近几次每格判完量到的占盘峰值）里有那一种的读数就用它，
    没有读数、peaks 是 None 的按 TARGET_DIRECTORY_CAP 算（上限只当清理线，没实测时才拿它顶）。
    余量没读到、写法不认交 None 与原因，头一行写明。弄坏开关 disknotchecked：不看磁盘；diskbudgetbycap：不看实测峰值、一律按上限算。"""
    if break_is(environment, "disknotchecked"):
        return None, "弄坏开关 disknotchecked：不看磁盘余量"
    free, source = disk_free_bytes(shared, environment)
    if free is None:
        return None, source
    cap = target_directory_cap_bytes(environment)
    reserve = bytes_of_size(environment.get(DISK_RESERVE_VARIABLE, "").strip() or DEFAULT_DISK_RESERVE)
    if cap is None or reserve is None:
        return None, f"{TARGET_DIRECTORY_CAP_VARIABLE} 或 {DISK_RESERVE_VARIABLE} 的写法不认"
    cap_text = environment.get(TARGET_DIRECTORY_CAP_VARIABLE, "").strip() or DEFAULT_TARGET_DIRECTORY_CAP
    by_cap = peaks is None or break_is(environment, "diskbudgetbycap")
    growth = {kind: cap if by_cap or peaks.get(kind) is None else peaks[kind] for kind in ("普通", "插桩")}
    growth_text = "、".join(f"{kind}那份涨到 {cap_text}（没有实测读数，按上限）" if by_cap or peaks.get(kind) is None
                            else f"{kind}那份涨到 {format_size(growth[kind])}（近几次每格判完量到的占盘峰值）" for kind in ("普通", "插桩"))

    def room_of(directories):
        return sum(max(0, growth[kind] - allocated_bytes_of_tree(directory)) if os.path.isdir(directory) else growth[kind]
                   for kind, directory in zip(("普通", "插桩"), directories))
    seed_base = os.path.normpath(shared) + SEED_TARGET_SUFFIX
    seed_room = room_of([seed_base, seed_base + COVERAGE_TARGET_SUFFIX])
    count, total, next_total = 0, 0, None
    for sequence in range(1, max(1, ceiling) + 2):   # 多数一个：放得下天花板之外的一个，才不与前几项里最小的那一项并列成「卡在」
        room = room_of(worker_target_directories(shared, sequence)) + (seed_room if sequence == 2 else 0)
        if reserve + total + room > free:
            next_total = total + room
            break
        total += room
        count = sequence
    text = (f"磁盘余量 {format_size(free)}（{source}），留备用 {format_size(reserve)}，每个工作进程最多占两份编译目录（{growth_text}），"
            f"开 2 个以上时先编一份依赖的两份也照这两种涨幅算（已有的目录只算还能涨的）："
            f"{count} 个共还要 {format_size(total)}"
            + (f"，再多 1 个要 {format_size(next_total)}、放不下，所以给 {count} 个" if next_total is not None else f"，前几项的天花板 {ceiling} 个（连同再多 1 个）都放得下，这一项不卡"))
    return count, text


def incremental_directories_of(target_directory):
    """编译目录底下的增量产物目录：<目录>/<档>/incremental 与 <目录>/<目标三元组>/<档>/incremental。"""
    found = []
    try:
        levels = [entry.path for entry in os.scandir(target_directory) if entry.is_dir(follow_symlinks=False)]
    except OSError:
        return found
    for level in sorted(levels):
        candidate = os.path.join(level, INCREMENTAL_DIRECTORY_NAME)
        if os.path.isdir(candidate):
            found.append(candidate)
        try:
            deeper = sorted(entry.path for entry in os.scandir(level) if entry.is_dir(follow_symlinks=False))
        except OSError:
            continue
        found += [os.path.join(path, INCREMENTAL_DIRECTORY_NAME) for path in deeper if os.path.isdir(os.path.join(path, INCREMENTAL_DIRECTORY_NAME))]
    return found


def trim_target_directory(directory, cap_bytes, environment):
    """一份编译目录占盘超过 cap_bytes 时清掉它底下的增量产物（incremental，rustc 每编一次改坏的副本都往里加、只增不减），回 (清前占盘, 清后占盘, [(清掉的目录或文件, 字节)])。
    清完增量仍超线的，再清 deps 里同名产物的旧版本（stale_dependency_artifacts：每个产物只留最新那一版）。
    没超线、目录不在、cap_bytes 是 None 都不动，清前清后交同一个数。清完回读再量一遍，不拿清前减清掉的估。只在两格之间调（run_pool 里每格判完），那时没有 cargo 在这份目录上跑。
    每个产物的最新一版不动，清完仍超线的由调用方报并停派（trim_worker_target）。弄坏开关 targetcapignored：不清；depsnotpruned：只清增量、不清 deps 里的旧版本。"""
    if break_is(environment, "targetcapignored") or cap_bytes is None or not os.path.isdir(directory):
        return 0, 0, []
    before = allocated_bytes_of_tree(directory)
    if before <= cap_bytes:
        return before, before, []
    removed = []
    for path in incremental_directories_of(directory):
        size = allocated_bytes_of_tree(path)
        shutil.rmtree(path)
        removed.append((path, size))
    after = allocated_bytes_of_tree(directory)
    if after > cap_bytes and not break_is(environment, "depsnotpruned"):
        # 清完增量仍超线：deps 里同名产物只留最新那一版（stale_dependency_artifacts），清完再量（文件头「编译目录上限」）
        for path in stale_dependency_artifacts(directory):
            size = allocated_bytes_of_tree(path)
            os.unlink(path)
            removed.append((path, size))
        after = allocated_bytes_of_tree(directory)
    return before, after, removed


def stale_dependency_artifacts(target_directory):
    """<目录>/<档>/deps 与 <目录>/<目标三元组>/<档>/deps 里同名产物的旧版本：文件名照 cargo 写成「<名字>-<16 位单元哈希>[.<后缀>]」，
    按（目录、名字、后缀）归堆，每堆按修改时刻只留最新的那一份，其余的交出来（路径的列表，按路径排）。名字里不带单元哈希的（cargo 拷到档那一层的工作区产物这一类）不动。
    同一个包换了特性、档或参数编出来的另一版也算同名：留新的、删旧的，之后的格要用旧那一版时 cargo 照「产物不在」重编（慢，判定不变）。"""
    stale, deps_directories = [], []
    try:
        levels = [entry.path for entry in os.scandir(target_directory) if entry.is_dir(follow_symlinks=False)]
    except OSError:
        levels = []
    for level in sorted(levels):
        deps_directories.append(os.path.join(level, "deps"))
        with contextlib.suppress(OSError):
            deps_directories += [os.path.join(entry.path, "deps") for entry in os.scandir(level) if entry.is_dir(follow_symlinks=False)]
    for deps in sorted(set(path for path in deps_directories if os.path.isdir(path))):
        piles = {}
        with contextlib.suppress(OSError):
            for entry in os.scandir(deps):
                found = re.fullmatch(r"(.+)-[0-9a-f]{16}((?:\.[^.]+)*)", entry.name)
                if found is None or not entry.is_file(follow_symlinks=False):
                    continue
                piles.setdefault((found.group(1), found.group(2)), []).append((entry.stat(follow_symlinks=False).st_mtime_ns, entry.path))
        for pile in piles.values():
            stale += [path for _mtime, path in sorted(pile)[:-1]]
    return sorted(stale)


def trim_worker_target(coverage):
    """这个工作进程刚判完一格：它自己的那份编译目录（coverage 为真是插桩那份）占盘超过上限就清增量产物，stderr 报清了什么、清完还剩多少。
    清不成只报，不让这一格的判定因此丢（并行不许把失败吃掉：这里报出来，判定照交）。"""
    context = WORKER_STATE["context"]
    directory = WORKER_STATE["environment"]["CARGO_TARGET_DIR"] + (COVERAGE_TARGET_SUFFIX if coverage else "")
    cap = context.get("target_directory_cap_bytes")
    # 量占盘（整份编译目录 os.walk 加 lstat）不是每格都量：上一次量到的离上限还远、按这个工作进程见过的每格最大涨幅（每个工作进程各记各的，只在本进程里）两倍估也涨不到上限时，
    # 隔几格（至多 4 格，没设上限的固定隔 3 格）才量一次；头三次（还没有涨幅可估）每格都量；量到的离上限没有两倍涨幅那么远时每格都量。弄坏开关 target-measured-every-cell：每格都量。
    # 常量写在函数里、不做模块级赋值
    skipped_cells_most = 4
    cadence = WORKER_STATE.setdefault("trim_cadence", {}).setdefault(directory, {"after": None, "cells": 0, "growth": 0.0, "measured": 0, "skip": 0})
    cadence["cells"] += 1
    if cadence["skip"] > 0 and not break_is(context["environment"], "target-measured-every-cell"):
        cadence["skip"] -= 1
        return
    try:
        before, after, removed = trim_target_directory(directory, cap, context["environment"])
    except OSError as error:
        note(f"  ! 编译目录 {directory} 超过上限时清增量产物没清成（{type(error).__name__}：{error}）；下一格判完再试")
        return
    # 每次量记一次占盘（清之前量的那个数，文件头「磁盘余量」：下一趟的磁盘预算按近几次的峰值算）；trim 没量的（目录不在、弄坏开关 targetcapignored）这里量
    occupied = before or (allocated_bytes_of_tree(directory) if os.path.isdir(directory) else 0)
    if cadence["after"] is not None and occupied > cadence["after"]:
        cadence["growth"] = max(cadence["growth"], (occupied - cadence["after"]) / cadence["cells"])
    cadence["measured"] += 1
    cadence["after"], cadence["cells"] = (after or occupied), 0
    if cap is None:
        cadence["skip"] = 3
    elif cadence["measured"] >= 3 and cadence["growth"] > 0:
        cadence["skip"] = max(0, min(skipped_cells_most, int((cap - cadence["after"]) / (2 * cadence["growth"]))))
    if occupied and context.get("run_records_directory"):
        problem = record_target_occupancy(context["run_records_directory"], coverage, occupied)
        if problem is not None:
            note(f"  ! 编译目录占盘没记进 {target_occupancy_file(context['run_records_directory'])}（{problem}）；磁盘预算少一个读数，不碍这一格的判定")
    if before > (cap or before):
        incremental_removed = [size for path, size in removed if os.path.basename(path) == INCREMENTAL_DIRECTORY_NAME]
        deps_removed = [size for path, size in removed if os.path.basename(path) != INCREMENTAL_DIRECTORY_NAME]
        still_over = after > cap
        note(f"  … 编译目录 {directory} 占盘 {format_size(before)} 超过上限 {format_size(cap)}：清掉增量产物 {len(incremental_removed)} 处共 {format_size(sum(incremental_removed))}"
             + (f"、deps 里同名产物的旧版本 {len(deps_removed)} 份共 {format_size(sum(deps_removed))}" if deps_removed else "")
             + f"，清后 {format_size(after)}"
             + ("；清后仍超上限（剩下的是每个依赖与测试二进制的最新一版）：停派（文件头「编译目录上限」）" if still_over else ""))
        if still_over and not break_is(context["environment"], "overcapdispatched"):
            # 停派（文件头「编译目录上限」）：往这一趟 shard_root 里的停派目录写一份，父进程每收一格看一眼，有就不再派活。弄坏开关 overcapdispatched：只报不停
            report = (f"第 {WORKER_STATE.get('worker_sequence', '?')} 片的{'插桩' if coverage else '普通'}编译目录 {directory}："
                      f"清前 {format_size(before)}、清后 {format_size(after)}，上限 {format_size(cap)}")
            try:
                write_atomically(os.path.join(target_over_cap_directory(WORKER_STATE["shard_root"]), f"{os.getpid()}-{'coverage' if coverage else 'plain'}"), report + "\n")
            except OSError as error:
                note(f"  ! 停派的标记没写成（{type(error).__name__}：{error}）：父进程照旧派活")


def target_over_cap_directory(shard_root):
    """这一趟的停派目录（文件头「编译目录上限」）：工作进程清完仍超上限往里写一份，run_pool 每收一格看一眼。"""
    return os.path.join(shard_root, "target-over-cap")


def target_over_cap_reports(shard_root):
    """停派目录里每一份写的那一句（按文件名排）；目录不在或读不了的交 []。"""
    reports = []
    with contextlib.suppress(OSError):
        for name in sorted(os.listdir(target_over_cap_directory(shard_root))):
            with contextlib.suppress(OSError):
                with open(os.path.join(target_over_cap_directory(shard_root), name), encoding="utf-8", errors="replace") as handle:
                    reports.append(handle.read().strip())
    return reports


def target_occupancy_file(records_directory):
    """每格判完量到的编译目录占盘记在运行记录目录（run_records_directory，文件头「判定库」末段）下这一份（文件头「磁盘余量」）：
    一行「<时刻秒><TAB>普通|插桩<TAB><字节>」，只追加。是运行记录，不进判定库。"""
    return os.path.join(records_directory, "target-occupancy.tsv")


def record_target_occupancy(records_directory, coverage, size_bytes):
    """追加一行（O_APPEND，一行远小于 PIPE_BUF，几个工作进程同时追加不交错）；写进回 None，写不进回原因。"""
    line = f"{int(time.time())}\t{'插桩' if coverage else '普通'}\t{size_bytes}\n".encode()
    try:
        descriptor = os.open(target_occupancy_file(records_directory), os.O_WRONLY | os.O_APPEND | os.O_CREAT, 0o644)
        try:
            os.write(descriptor, line)
        finally:
            os.close(descriptor)
    except OSError as error:
        return f"{type(error).__name__}：{error}"
    return None


def measured_target_peaks(records_directory, recent):
    """{"普通": 字节或 None, "插桩": 字节或 None}：两种编译目录各取最近 recent 次记录里最大的（文件头「磁盘余量」）；文件不在、读不了的都是 None，写坏的行不算。
    记录多过 recent 的 50 倍时只留每种最近 recent 次，同目录临时文件改名换上（几趟同时用一份运行记录时可能丢几个读数，只是预算少几个样本）。"""
    kept = {"普通": [], "插桩": []}
    path = target_occupancy_file(records_directory)
    try:
        with open(path, encoding="utf-8", errors="replace") as handle:
            lines = handle.read().splitlines()
    except OSError:
        lines = []
    for line in lines:
        fields = line.split("\t")
        if len(fields) == 3 and fields[1] in kept and fields[2].isdigit():
            kept[fields[1]].append((line, int(fields[2])))
    if len(lines) > 50 * recent:
        survivors = sorted((entry for values in kept.values() for entry in values[-recent:]), key=lambda entry: entry[0].split("\t")[0])
        with contextlib.suppress(OSError):
            write_atomically(path, "".join(entry[0] + "\n" for entry in survivors))
    return {kind: max(size for _line, size in values[-recent:]) if values else None for kind, values in kept.items()}


def selftest_target_directory_cells(selftest, work):
    """编译目录（文件头「编译目录」）：片号大于这一趟工作进程数的 -w<片号> 连同 -coverage 开跑时回收、有进程在用的不删、两个变量都没设时不落 /tmp。"""
    repository = os.path.join(work, "target-reclaim")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)

    def seed(directory, names):
        for name in names:
            selftest_write(os.path.join(directory, name, "debug", "deps", "libold.rlib"), "x" * 8192)

    # 一：2 个工作进程。-w3、-w3-coverage、-w5 回收；第 1 片、它的 -coverage、-w2、-w2-coverage 留着；只是名字像的不碰
    targets = os.path.join(work, "reclaim-targets")
    shared = os.path.join(targets, TARGET_DIRECTORY_NAME)
    kept_names = [TARGET_DIRECTORY_NAME, TARGET_DIRECTORY_NAME + COVERAGE_TARGET_SUFFIX, TARGET_DIRECTORY_NAME + "-w2",
                  TARGET_DIRECTORY_NAME + "-w2" + COVERAGE_TARGET_SUFFIX, TARGET_DIRECTORY_NAME + "-w2x", "another-target-w9"]
    seed(targets, kept_names + [TARGET_DIRECTORY_NAME + "-w3", TARGET_DIRECTORY_NAME + "-w3" + COVERAGE_TARGET_SUFFIX, TARGET_DIRECTORY_NAME + "-w5"])
    exit_code, _stdout, stderr, _ = selftest.run(repository, GATE_MUTATION_TARGET_DIR=shared, GATE_MUTATION_WORKERS="2")
    present = sorted(name for name in os.listdir(targets) if not name.endswith(WORKER_SLOT_LOCK_SUFFIX))   # 片号锁文件（.slot）是领片号留下的，不是编译目录
    reclaim_lines = [line for line in stderr.splitlines() if "编译目录：" in line]
    selftest.check("片号大于这一趟工作进程数的 -w<片号> 连同 -coverage 开跑时回收、不大于的留着、打一行删了几份多大",
                   exit_code == 0 and present == sorted(kept_names) and any("回收了片号大于这一趟 2 个工作进程的 3 份，共 " in line for line in reclaim_lines),
                   f"退 {exit_code}；剩下 {present}（要 {sorted(kept_names)}）；{reclaim_lines}")
    # 二：多出来的四份各有人在用（工作目录、命令行、CARGO_TARGET_DIR、在用标记的共享锁），一份没人用：只删没人用的那份
    targets = os.path.join(work, "reclaim-in-use")
    shared = os.path.join(targets, TARGET_DIRECTORY_NAME)
    used = {kind: f"{TARGET_DIRECTORY_NAME}-w{sequence}" for kind, sequence in (("cwd", 4), ("cmdline", 6), ("variable", 7), ("lock", 8))}
    seed(targets, [TARGET_DIRECTORY_NAME, *used.values(), TARGET_DIRECTORY_NAME + "-w9"])
    lock_ready = os.path.join(work, "reclaim-lock-ready")
    sleeper = "import time; time.sleep(120)"
    lock_holder = ("import fcntl, os, time\nhandle = open(os.environ['SELFTEST_LOCK_PATH'], 'a')\nfcntl.flock(handle, fcntl.LOCK_SH)\n"
                   "open(os.environ['SELFTEST_LOCK_READY'], 'w').close()\ntime.sleep(120)\n")
    in_use_repository = selftest_new_library_copy(work, repository, "target-reclaim-in-use")   # 换一棵树：库是新的一份，这一趟要现跑才开工作进程、才回收
    plain_environment = {key: value for key, value in os.environ.items() if key != "CARGO_TARGET_DIR"}
    holders = []
    try:
        holders.append(subprocess.Popen([sys.executable, "-c", sleeper], cwd=os.path.join(targets, used["cwd"]), env=plain_environment))
        holders.append(subprocess.Popen([sys.executable, "-c", sleeper, os.path.join(targets, used["cmdline"], "debug")], cwd=work, env=plain_environment))
        holders.append(subprocess.Popen([sys.executable, "-c", sleeper], cwd=work,
                                        env=dict(plain_environment, CARGO_TARGET_DIR=os.path.join(targets, used["variable"]))))
        holders.append(subprocess.Popen([sys.executable, "-c", lock_holder], cwd=work,
                                        env=dict(plain_environment, SELFTEST_LOCK_PATH=os.path.join(targets, used["lock"], WORKER_TARGET_LOCK_NAME),
                                                 SELFTEST_LOCK_READY=lock_ready)))
        deadline = time.time() + 30
        while not os.path.exists(lock_ready) and time.time() < deadline:
            time.sleep(0.05)
        exit_code, _stdout, stderr, _ = selftest.run(in_use_repository, GATE_MUTATION_TARGET_DIR=shared, GATE_MUTATION_WORKERS="2")
    finally:
        for holder in holders:
            holder.kill()
            holder.wait()
    present = set(os.listdir(targets))
    reclaim_lines = [line for line in stderr.splitlines() if "编译目录：" in line or line.startswith("    没删 ")]
    selftest.check("多出来的 -w<片号> 有进程在用的不删（工作目录、命令行、CARGO_TARGET_DIR、在用标记各一份），没人用的照删",
                   exit_code == 0 and set(used.values()) <= present and TARGET_DIRECTORY_NAME + "-w9" not in present
                   and any("有进程在用、没删 4 份" in line for line in reclaim_lines),
                   f"退 {exit_code}；剩下 {sorted(present)}；在用的四份 {sorted(used.values())}；{reclaim_lines}")
    # 三：没设 GATE_MUTATION_TARGET_DIR（文件头「编译目录」）：编译目录建在这一趟自己的运行目录（$TMPDIR 下的 singlefs-mutation-replay-*）的 build 里，
    #   正常退 0、没跑全退 1（GATE_MUTATION_STOP_AFTER_ROWS=1）两种结局跑完都删、$TMPDIR 下一样不剩、跨轮缓存家目录一样没建；被 TERM 停那一种在 selftest_signal_and_pipe_cells 里核。
    #   HOME 与 TMPDIR 都换成临时目录里的一份（弄坏开关退回 $TMPDIR 下那一份时也不往真的 /tmp 里建）；rustc -V 要的 rustup 家目录照原样指回去
    home, temporary = os.path.join(work, "home"), os.path.join(work, "plain-tmp")
    os.makedirs(temporary, exist_ok=True)
    unset = dict(GATE_MUTATION_TARGET_DIR="", HOME=home, TMPDIR=temporary, GATE_MUTATION_WORKERS="1",
                 RUSTUP_HOME=os.environ.get("RUSTUP_HOME") or os.path.expanduser("~/.rustup"),
                 CARGO_HOME=os.environ.get("CARGO_HOME") or os.path.expanduser("~/.cargo"))
    problems = []
    saved_tempdir = tempfile.tempdir   # run_stage 在本进程里跑：shard_root 照 tempfile.gettempdir() 建，这里指进 temporary，跑完还原
    for name, extra, wanted_exit in (("正常", {}, 0), ("没跑全", {"GATE_MUTATION_STOP_AFTER_ROWS": "1"}, 1)):
        unset_repository = selftest_new_library_copy(work, repository, f"target-reclaim-unset-{wanted_exit}")   # 每一趟换一棵树：库是新的一份，各自要现跑
        tempfile.tempdir = temporary
        try:
            exit_code, _stdout, stderr, _ = selftest.run(unset_repository, **dict(unset, **extra))
        finally:
            tempfile.tempdir = saved_tempdir
        chosen = selftest_run_target_directories(stderr)
        left = sorted(os.listdir(temporary))
        if not (exit_code == wanted_exit and len(chosen) == 1 and chosen[0].startswith(os.path.join(temporary, SHARD_ROOT_PREFIX))
                and chosen[0].endswith(os.sep + os.path.join(RUN_BUILD_DIRECTORY_NAME, TARGET_DIRECTORY_NAME)) and not os.path.exists(chosen[0]) and left == []
                and f"删了这一趟的编译目录 {os.path.dirname(chosen[0])}" in stderr and not os.path.exists(os.path.join(home, ".local", "share", "singlefs", "cross-run-cache"))):
            problems.append(f"{name}那一趟退 {exit_code}（要 {wanted_exit}），编译目录 {chosen}，$TMPDIR 下剩 {left}；"
                            f"{[line for line in stderr.splitlines() if '编译目录' in line][:3]}")
    selftest.check("跑完运行目录里不剩它建的编译目录（没设 GATE_MUTATION_TARGET_DIR：建在 $TMPDIR 下这一趟的 singlefs-mutation-replay-* 的 build 里，正常退 0、没跑全退 1 都删，$TMPDIR 下一样不剩）",
                   not problems, "；".join(problems))
    # 四：这一趟的编译目录里还有进程开着文件（自证起一个 sleep 开着 build 里的一份文件）：remove_run_build_root 不删、报出那个进程；只停自证自己起的那一个进程
    build_root = os.path.join(work, "held-run", RUN_BUILD_DIRECTORY_NAME)
    held = os.path.join(build_root, TARGET_DIRECTORY_NAME, "debug", "held")
    os.makedirs(os.path.dirname(held))
    with open(held, "w", encoding="utf-8") as held_handle:
        holder = subprocess.Popen(["sleep", "60"], stdout=held_handle, stdin=subprocess.DEVNULL)
    try:
        deadline = time.time() + 10
        while time.time() < deadline:
            with contextlib.suppress(OSError):
                if os.readlink(f"/proc/{holder.pid}/fd/1") == held:
                    break
            time.sleep(0.05)
        removed, lines = remove_run_build_root(build_root, selftest.base_environment)
    finally:
        holder.kill()
        holder.wait()
    selftest.check("这一趟的编译目录里还有进程开着文件：不删、报出那个进程",
                   not removed and os.path.isdir(build_root) and any(f"进程 {holder.pid}（sleep 60）" in line for line in lines),
                   f"删成了 {removed}，build 还在 {os.path.isdir(build_root)}；{lines}")


def selftest_run_target_directories(stderr):
    """stderr 里「… 编译目录：<路径>（第 1 片…」那几行的路径（这一趟用的第 1 片编译目录）。"""
    return [line.split("编译目录：", 1)[1].split("（", 1)[0] for line in stderr.splitlines() if line.startswith("  … 编译目录：")]


def selftest_slot_overlap_problem(selftest, earlier_process, earlier_slots, shared):
    """后起的那次领到片号的那一刻，两次调用重叠了没有：先起的那次的子进程还活着、它领到的两片的片号锁都被占，重叠回 None，没重叠回原因。
    弄坏开关 overlapunchecked：不核，一律回 None，两次没重叠也照判对错。"""
    if break_is(selftest.base_environment, "overlapunchecked"):
        return None
    if earlier_process.poll() is not None:
        return f"后起的那次领到片号时先起的那次已经退了（退 {earlier_process.poll()}）"
    if len(earlier_slots) != 2:
        return f"先起的那次只报了片号 {earlier_slots}"
    free = []
    for slot in earlier_slots:
        held, handle = file_lock_is_held(slot_lock_path(shared, slot))
        if handle is not None:
            handle.close()
        if not held:
            free.append(slot)
    return f"后起的那次领到片号时先起的那次的片号锁 {free} 没被占" if free else None


def selftest_concurrent_slot_pair(selftest, work, tag, earlier_exits_first):
    """在一个共用编译目录上起两次调用（各 2 个工作进程），回 dict：not_ready、earlier_slots、later_slots、held_while_running、free_afterwards、overlap_problem。
    earlier_exits_first 为真是样本：先起的那次不挂假 cargo、等它退了才起后起的那次。"""
    shared = os.path.join(work, f"slot-targets-{tag}", TARGET_DIRECTORY_NAME)
    released = os.path.join(work, f"slot-cargo-released-{tag}")
    invocations = {}
    # 等的是条件（两个工作进程都报了片号；放掉假 cargo 之后两次调用都退出），不是定长：机器忙时头一步可以慢到分钟，照「被 TERM 停」那一格带超时地轮询；
    # 超时不再往下赌，记进 not_ready，这一格判错并写明没等到什么
    ready_seconds, finish_seconds = 300, 180
    not_ready = []
    for name in ("earlier", "later"):
        repository = os.path.join(work, f"slots-{tag}-{name}")
        build_selftest_repository(repository, [SELFTEST_TABLE_ROWS[0], SELFTEST_OTHER_BATCH_ROW])
        temporary = os.path.join(work, f"slots-{tag}-{name}-tmp")
        os.makedirs(temporary)
        environment = dict(selftest.base_environment, GATE_MUTATION_TARGET_DIR=shared, GATE_MUTATION_WORKERS="2", TMPDIR=temporary,
                           FAKE_CARGO_LOG=os.path.join(work, f"slots-{tag}-{name}-cargo.log"), FAKE_CARGO_HANG_FILE=os.path.join(work, f"slots-{tag}-{name}-hanging"),
                           FAKE_CARGO_RELEASE_FILE=released, FAKE_CARGO_DONE_FILE=os.path.join(work, f"slots-{tag}-{name}-done"))
        if earlier_exits_first and name == "earlier":
            del environment["FAKE_CARGO_HANG_FILE"]   # 样本：先起的那次不挂，跑完就退
        invocations[name] = dict(repository=repository, environment=environment, stderr_path=os.path.join(work, f"slots-{tag}-{name}-stderr.txt"))

    def slots_reported(name):
        with open(invocations[name]["stderr_path"], encoding="utf-8", errors="replace") as handle:
            return sorted(int(number) for number in re.findall(r"领到片号 (\d+)（", handle.read()))

    def start(name):
        with open(invocations[name]["stderr_path"], "w", encoding="utf-8") as stderr_handle:
            invocations[name]["process"] = subprocess.Popen([sys.executable, "-c", STAGE_CHILD_PROGRAM, MODULE_PATH, invocations[name]["repository"]],
                                                            env=selftest.pool_process_environment(invocations[name]["environment"]), stdout=subprocess.DEVNULL, stderr=stderr_handle)
        deadline = time.monotonic() + ready_seconds
        while len(slots_reported(name)) < 2 and invocations[name]["process"].poll() is None and time.monotonic() < deadline:
            time.sleep(0.05)
        reported = slots_reported(name)
        if len(reported) < 2:
            not_ready.append(f"{name} 那次 {ready_seconds} 秒内只报了片号 {reported}（子进程已退 {invocations[name]['process'].poll()}）")
        return reported

    def wait_for_own_selftest_child(process):
        """等自证自己起的那一个子进程退出；放掉假 cargo 之后 finish_seconds 秒还没退就停掉它（只停这一个，按进程对象，不按模式找），并记进 not_ready。"""
        try:
            process.wait(timeout=finish_seconds)
        except subprocess.TimeoutExpired:
            not_ready.append(f"放掉假 cargo 之后 {finish_seconds} 秒还没退，停掉了它（进程 {process.pid}）")
            process.kill()
            process.wait()

    earlier_slots = start("earlier")
    if earlier_exits_first:
        wait_for_own_selftest_child(invocations["earlier"]["process"])
    later_slots = start("later")
    overlap_problem = selftest_slot_overlap_problem(selftest, invocations["earlier"]["process"], earlier_slots, shared)
    locks_while_running = {slot: file_lock_is_held(slot_lock_path(shared, slot)) for slot in sorted({*earlier_slots, *later_slots})}
    held_while_running = sorted(slot for slot, (held, handle) in locks_while_running.items() if held)
    for _held, handle in locks_while_running.values():
        if handle is not None:
            handle.close()
    open(released, "w").close()
    for name in invocations:
        wait_for_own_selftest_child(invocations[name]["process"])
    free_afterwards = []
    for slot in sorted({*earlier_slots, *later_slots}):
        held, handle = file_lock_is_held(slot_lock_path(shared, slot))
        free_afterwards.append(not held)
        if handle is not None:
            handle.close()
    return dict(not_ready=not_ready, earlier_slots=earlier_slots, later_slots=later_slots, held_while_running=held_while_running,
                free_afterwards=free_afterwards, overlap_problem=overlap_problem)


def selftest_worker_slot_cells(selftest, work):
    """片号跨调用经锁领（文件头「编译目录」）：同一个共用编译目录上并发的两次调用各起 2 个工作进程（假 cargo 挂着不放，两次的工作进程都活着），
    两次领到的片号集合不相交，四把片号锁都被占，放掉之后锁都空了；片号锁被占着的多出来的 -w<片号> 回收时留着。
    两次调用各是一个子进程（stderr 落文件，片号从工作进程报的「领到片号 <n>」那一行读），基线的批分成两批、两个工作进程才都起得来。
    并发那一格先核两次调用确实重叠（selftest_slot_overlap_problem：后起的那次领到片号时先起的那次还活着、它的两把片号锁都被占），没重叠就记这一格本次未跑、不判对错；
    样本：先起的那次不挂假 cargo、等它退了才起后起的那次，那一格要认出没重叠（弄坏开关 overlapunchecked：不核重叠，样本那一格判错）。"""
    pair = selftest_concurrent_slot_pair(selftest, work, "concurrent", False)
    label = "同一个共用编译目录上并发的两次调用各起 2 个工作进程：领到的片号集合不相交、四把片号锁都被占、两次退出之后锁都空了"
    if pair["overlap_problem"] is not None:
        selftest.skip(label, f"两次调用没重叠，这一格的前提没造出来（{pair['overlap_problem']}）；先起的那次领到 {pair['earlier_slots']}、后起的那次领到 {pair['later_slots']}")
    else:
        earlier_slots, later_slots, held_while_running = pair["earlier_slots"], pair["later_slots"], pair["held_while_running"]
        selftest.check(label,
                       not pair["not_ready"] and len(earlier_slots) == 2 and len(later_slots) == 2 and not set(earlier_slots) & set(later_slots)
                       and held_while_running == sorted({*earlier_slots, *later_slots}) and len(held_while_running) == 4 and all(pair["free_afterwards"]),
                       f"{'；'.join(pair['not_ready'])}{'；' if pair['not_ready'] else ''}先起的那次领到 {earlier_slots}、后起的那次领到 {later_slots}；"
                       f"跑的时候被占的锁 {held_while_running}；退出之后锁空了 {pair['free_afterwards']}")
    sample = selftest_concurrent_slot_pair(selftest, work, "earlier-exits", True)
    selftest.check("先起的那次在后起的那次开始之前就退了（样本）：并发领片号那一格认出两次调用没重叠、记本次未跑，不判对错",
                   sample["overlap_problem"] is not None and "先起的那次已经退了" in sample["overlap_problem"],
                   f"认出的没重叠：{sample['overlap_problem']}；先起的那次领到 {sample['earlier_slots']}、后起的那次领到 {sample['later_slots']}；{'；'.join(sample['not_ready'])}")
    # 回收：多出来的 -w<片号> 的片号锁被别的调用的工作进程拿着（它刚领到这一片、还没来得及拿在用标记）时留着，没人拿的照删
    targets = os.path.join(work, "slot-reclaim")
    reclaim_shared = os.path.join(targets, TARGET_DIRECTORY_NAME)
    for name in (TARGET_DIRECTORY_NAME, TARGET_DIRECTORY_NAME + "-w3", TARGET_DIRECTORY_NAME + "-w4"):
        selftest_write(os.path.join(targets, name, "debug", "deps", "libold.rlib"), "x" * 4096)
    import fcntl
    holder = open(slot_lock_path(reclaim_shared, 3), "a")
    try:
        fcntl.flock(holder, fcntl.LOCK_EX | fcntl.LOCK_NB)
        removed, kept, failed = reclaim_extra_target_directories(reclaim_shared, 2, selftest.base_environment)
    finally:
        holder.close()
    selftest.check("多出来的 -w<片号> 的片号锁被别的进程拿着的不删（没有进程的工作目录、命令行、环境指到它），没人拿的照删",
                   [os.path.basename(path) for path, _size in removed] == [TARGET_DIRECTORY_NAME + "-w4"]
                   and [os.path.basename(path) for path, _reason in kept] == [TARGET_DIRECTORY_NAME + "-w3"] and "片号锁" in kept[0][1] and not failed
                   and os.path.isdir(os.path.join(targets, TARGET_DIRECTORY_NAME + "-w3")),
                   f"删了 {removed}；留着 {kept}；删不成 {failed}")



def environment_records(path):
    """假 cargo 的 FAKE_CARGO_ENVIRONMENT_LOG：每起一次 test 一行 JSON（假 cargo 头上那一段）；文件不在交空表。"""
    if not os.path.exists(path):
        return []
    with open(path, encoding="utf-8") as handle:
        return [json.loads(line) for line in handle.read().splitlines() if line.strip()]


def selftest_write_random(path, size):
    """size 字节的随机内容（压不下去、占盘就是 size 往上），自证里造占盘的编译目录用。"""
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "wb") as handle:
        handle.write(os.urandom(size))


def selftest_target_measure_cadence_cell(selftest, work):
    """每格判完量一次编译目录占盘（trim_worker_target，文件头「编译目录上限」）不是每格都量：上限 20 MiB、每格涨 1 MiB（不可压缩的随机内容）的目录，判 20 格：
    头三次每格都量（还没有涨幅可估），之后按「离上限的余量 ÷ 两倍的每格最大涨幅」隔几格（至多 4 格）量一次，量的格不超过 10 次；没量的格目录都没超过上限（量的次数少、上限照样守得住）。
    弄坏开关 target-measured-every-cell（每格都量，量 20 次）下这一格判错。"""
    directory = os.path.join(work, "cadence-target")
    os.makedirs(directory)
    cap = 20 << 20
    original = allocated_bytes_of_tree
    calls = []

    def counting(path):
        calls.append(path)
        return original(path)
    saved = dict(WORKER_STATE)
    WORKER_STATE.clear()
    WORKER_STATE.update(context=dict(target_directory_cap_bytes=cap, run_records_directory=None, environment=dict(os.environ)), environment={"CARGO_TARGET_DIR": directory})
    sizes, measured_cells = [], []
    try:
        globals()["allocated_bytes_of_tree"] = counting
        for cell in range(1, 21):
            with open(os.path.join(directory, f"grow-{cell}"), "wb") as handle:
                handle.write(os.urandom(1 << 20))
            sizes.append(original(directory))
            before_calls = len(calls)
            trim_worker_target(False)
            if len(calls) > before_calls:
                measured_cells.append(cell)
    finally:
        globals()["allocated_bytes_of_tree"] = original
        WORKER_STATE.clear()
        WORKER_STATE.update(saved)
    skipped_over = [cell for cell, size in enumerate(sizes, 1) if cell not in measured_cells and size > cap]
    selftest.check("编译目录占盘不是每格都量：头三格都量，20 格里量的不超过 10 次，没量的格目录都没超过上限 20 MiB",
                   measured_cells[:3] == [1, 2, 3] and len(measured_cells) <= 10 and not skipped_over,
                   f"量了 {len(measured_cells)} 次（第 {measured_cells} 格）；没量而超了上限的格 {skipped_over}")


def selftest_target_cap_cells(selftest, work):
    """编译目录上限（文件头「编译目录上限」）：一份编译目录占盘超过上限，在两格之间清掉它底下的增量产物（<目录>/<档>/incremental、<目录>/<三元组>/<档>/incremental），
    依赖与测试二进制不动；没超线不动；清完仍超线的照实报。先直接调 trim_target_directory：上限 2 MiB，增量 3+1 MiB、依赖 1 MiB，清掉两处增量、依赖还在、清后回读不到 2 MiB；
    没超线不动；清完仍超线（依赖 3 MiB）只清增量、清后仍超。再起两趟 run_stage：假 cargo 每次编（基线与变异直接跑测试的那一趟、覆盖的插桩编译）往增量产物里写 64 KiB，上限 1T 时跑完两份编译目录（普通、插桩）的增量产物都留着（对照），
    上限 1K 时跑完两份底下都没有增量产物。弄坏开关 targetcapignored（不清）下三格判错。"""
    environment = dict(selftest.base_environment)
    directory = os.path.join(work, "cap-target")
    selftest_write_random(os.path.join(directory, "debug", "incremental", "session-a", "dep-graph.bin"), 3 << 20)
    selftest_write_random(os.path.join(directory, "x86_64-unknown-linux-gnu", "debug", "incremental", "session-b", "query-cache.bin"), 1 << 20)
    selftest_write_random(os.path.join(directory, "debug", "deps", "libkept.rlib"), 1 << 20)
    before, after, removed = trim_target_directory(directory, 2 << 20, environment)
    selftest.check("编译目录超过上限：清掉两处增量产物，依赖还在，清后回读不到上限",
                   before >= 5 << 20 and len(removed) == 2 and after < 2 << 20 and os.path.isfile(os.path.join(directory, "debug", "deps", "libkept.rlib"))
                   and not os.path.exists(os.path.join(directory, "debug", "incremental")) and not os.path.exists(os.path.join(directory, "x86_64-unknown-linux-gnu", "debug", "incremental")),
                   f"清前 {before}、清后 {after}、清掉 {removed}")
    again = trim_target_directory(directory, 10 << 20, environment)
    selftest.check("没超线不动", again[2] == [] and again[0] == again[1] == after, f"{again}")
    heavy = os.path.join(work, "cap-target-heavy")
    selftest_write_random(os.path.join(heavy, "debug", "incremental", "session-c", "dep-graph.bin"), 1 << 20)
    selftest_write_random(os.path.join(heavy, "debug", "deps", "libbig.rlib"), 3 << 20)
    before, after, removed = trim_target_directory(heavy, 2 << 20, environment)
    selftest.check("清完仍超线：只清增量产物、不动依赖，清后仍大于上限（由调用方报）",
                   len(removed) == 1 and after > 2 << 20 and os.path.isfile(os.path.join(heavy, "debug", "deps", "libbig.rlib")), f"清前 {before}、清后 {after}、清掉 {removed}")
    repository = os.path.join(work, "cap-run")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    target = os.path.join(work, "cap-run-target")
    incremental_paths = (os.path.join(target, "debug", INCREMENTAL_DIRECTORY_NAME), os.path.join(target + COVERAGE_TARGET_SUFFIX, "debug", INCREMENTAL_DIRECTORY_NAME))
    # 对照：上限宽裕（1T）时假 cargo 写下的增量产物两份都留着（证明假 cargo 真写了；工作进程里的 stderr 在自证里收不到，所以不靠它报的话判）
    roomy_exit, _stdout, _stderr, _ = selftest.run(repository, GATE_MUTATION_TARGET_DIR=target, FAKE_CARGO_INCREMENTAL_BYTES="65536",
                                                   **{TARGET_DIRECTORY_CAP_VARIABLE: "1T"})
    roomy_left = [path for path in incremental_paths if os.path.exists(path)]
    shutil.rmtree(target, ignore_errors=True)
    shutil.rmtree(target + COVERAGE_TARGET_SUFFIX, ignore_errors=True)
    # 清的那一趟另起一棵仓（判定库在仓的 git 目录里，沿用上一趟的覆盖记录就不会再编插桩那一份）
    trimmed_repository = os.path.join(work, "cap-run-trimmed")
    build_selftest_repository(trimmed_repository, SELFTEST_TABLE_ROWS)
    # 上限 40K：假 cargo 每次编写进增量产物的 64K 一来就超线、清掉增量之后剩下的（目录与 deps 里一两份小文件，十几 K）在线下，所以照常跑完、不停派
    exit_code, _stdout, stderr, _ = selftest.run(trimmed_repository, GATE_MUTATION_TARGET_DIR=target, FAKE_CARGO_INCREMENTAL_BYTES="65536",
                                                 **{TARGET_DIRECTORY_CAP_VARIABLE: "40K"})
    leftovers = [path for path in incremental_paths if os.path.exists(path)]
    selftest.check("编译目录上限交给 run_stage：上限宽裕（1T）时增量产物两份都留着；上限 40K 时每格判完清掉超线的那份编译目录的增量产物（普通、插桩两份），跑完两份底下都没有增量产物",
                   roomy_exit == 0 and len(roomy_left) == 2 and exit_code == 0 and not leftovers and os.path.isdir(os.path.join(target + COVERAGE_TARGET_SUFFIX, "debug", "deps")),
                   f"宽裕时退 {roomy_exit}、留着 {len(roomy_left)} 份；40K 时退 {exit_code}、还剩 {leftovers}；{stderr[-500:]}")


def selftest_target_prune_cells(selftest, work):
    """编译目录上限的后两步（文件头「编译目录上限」）：清完增量仍超线时 deps 里同名产物只留最新一版、清到线下；还超线就停派。
    先直接调 trim_target_directory：上限 5 MiB，增量 1 MiB、libfoo 新旧两版各 2 MiB、foo_check 新旧两版各 1 MiB、不带单元哈希的 libplain 256 KiB，
    清掉增量与两份旧版、新版与 libplain 还在、清后回读在线下（弄坏开关 depsnotpruned 下清后仍超线，这一格判错）。
    再起一趟 run_stage：上限 1K（清什么都清不到线下），头一格（基线的批）判完就停派、整道判红、现判的变异 0 条、点名停派的那份编译目录（弄坏开关 overcapdispatched 下照派、退 0，这一格判错）。"""
    environment = dict(selftest.base_environment)
    directory = os.path.join(work, "prune-target")
    deps = os.path.join(directory, "debug", "deps")
    old_hash, new_hash = "a" * 16, "b" * 16
    selftest_write_random(os.path.join(directory, "debug", "incremental", "session-d", "dep-graph.bin"), 1 << 20)
    for name, size in ((f"libfoo-{old_hash}.rlib", 2 << 20), (f"libfoo-{new_hash}.rlib", 2 << 20), (f"foo_check-{old_hash}", 1 << 20), (f"foo_check-{new_hash}", 1 << 20),
                       ("libplain.rlib", 256 << 10)):
        selftest_write_random(os.path.join(deps, name), size)
        stamp = 1000 if old_hash in name else 2000
        os.utime(os.path.join(deps, name), (stamp, stamp))
    before, after, removed = trim_target_directory(directory, 5 << 20, environment)
    left = sorted(os.listdir(deps))
    selftest.check("编译目录清完增量仍超上限：deps 里同名产物只留最新一版（libfoo、foo_check 的旧版删掉），不带单元哈希的不动，清后回读在线下",
                   before > 5 << 20 and after <= 5 << 20 and left == sorted([f"libfoo-{new_hash}.rlib", f"foo_check-{new_hash}", "libplain.rlib"])
                   and sorted(os.path.basename(path) for path, _size in removed) == sorted([INCREMENTAL_DIRECTORY_NAME, f"libfoo-{old_hash}.rlib", f"foo_check-{old_hash}"]),
                   f"清前 {before}、清后 {after}、deps 里剩 {left}、清掉 {[path for path, _size in removed]}")
    repository = os.path.join(work, "prune-run")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    target = os.path.join(work, "prune-run-target")
    exit_code, stdout, stderr, _ = selftest.run(repository, GATE_MUTATION_TARGET_DIR=target, **{TARGET_DIRECTORY_CAP_VARIABLE: "1K"})
    selftest.check("编译目录清完仍超上限（1K）：头一格判完就停派、整道判红，现判的变异 0 条，拒绝说明点名停派的那份编译目录",
                   exit_code == 1 and "仍超上限 GATE_MUTATION_TARGET_DIR_CAP，停派" in stdout and "现判的变异判完 0 条" in stdout and f"编译目录 {target}：" in stdout,
                   f"退 {exit_code}；{(stdout + stderr)[-900:]}")


def selftest_disk_peak_cells(selftest, work):
    """磁盘预算按实测峰值算（文件头「磁盘余量」）：每格判完记一次占盘，下一趟每份编译目录的涨幅取近几次的峰值，上限只当清理线。
    先直接调：记下普通 9G、1G、2G、3G 与插桩 4G，取近 3 次是普通 3G（9G 那次出了窗口）、插桩 4G；余量 40G、上限 10G、备用 5G，
    峰值普通 2G、插桩 3G 时每个工作进程 5G、放得下 6 个，插桩没有读数的那一种按上限 10G、只放得下 1 个（弄坏开关 diskbudgetbycap 下一律按上限、头一格也是 1 个，那一格判错）。
    再起一趟 run_stage：跑完运行记录目录（run_records_directory，不是判定库）下记了普通编译目录的占盘。"""
    records_directory = os.path.join(work, "peak-run-records")
    os.makedirs(records_directory)
    for coverage, size in ((False, 9 << 30), (False, 1 << 30), (True, 4 << 30), (False, 2 << 30), (False, 3 << 30)):
        record_target_occupancy(records_directory, coverage, size)
    peaks = measured_target_peaks(records_directory, 3)
    selftest.check("编译目录占盘的记录：普通、插桩两种各取最近 3 次里最大的（窗口之外的 9G 不算）", peaks == {"普通": 3 << 30, "插桩": 4 << 30}, f"{peaks}")
    environment = dict(selftest.base_environment, **{DISK_FREE_VARIABLE: "40G", TARGET_DIRECTORY_CAP_VARIABLE: "10G", DISK_RESERVE_VARIABLE: "5G"})
    measured_count, measured_head = disk_bound_for(os.path.join(work, "peak-target"), environment, 9, {"普通": 2 << 30, "插桩": 3 << 30})
    mixed_count, mixed_head = disk_bound_for(os.path.join(work, "peak-target"), environment, 9, {"普通": 2 << 30, "插桩": None})
    selftest.check("磁盘预算按实测峰值算：峰值普通 2G、插桩 3G 时余量 40G（备用 5G）放得下 6 个，插桩没有读数时那一种按上限 10G、放得下 1 个，头一行写明涨幅从哪来",
                   measured_count == 6 and "普通那份涨到 2.0 GiB（近几次每格判完量到的占盘峰值）" in measured_head
                   and mixed_count == 1 and "插桩那份涨到 10G（没有实测读数，按上限）" in mixed_head,
                   f"按峰值 {measured_count}：{measured_head}；插桩没读数 {mixed_count}：{mixed_head}")
    repository = os.path.join(work, "peak-run")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    exit_code, _stdout, stderr, _ = selftest.run(repository, GATE_MUTATION_TARGET_DIR=os.path.join(work, "peak-run-target"))
    run_records = run_records_directory(repository, dict(selftest.base_environment))
    recorded = measured_target_peaks(run_records, 20)
    library_files = [name for _walked, _directories, names in os.walk(library_of(repository)) for name in names]
    selftest.check("run_stage 每格判完记一次编译目录占盘：跑完运行记录目录下有普通编译目录的读数，判定库目录本地一个文件都没有",
                   exit_code == 0 and recorded["普通"] is not None and not library_files,
                   f"退 {exit_code}；{run_records} 的读数 {recorded}；库目录本地的文件 {library_files[:3]}；{stderr[-400:]}")
