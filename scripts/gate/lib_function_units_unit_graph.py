"""产品行为视图编出的 unit-graph 程序：放哪（缓存目录）、输入摘要、编译与运行取 dump。
（函数单元算键库 scripts/gate/lib_function_units.py 的一部分：用法、判法与弄坏开关表在那一份文件头，不单独起。）"""
import errno
import fcntl
import hashlib
import os
import shutil
import subprocess
import tempfile
import time
from lib_function_units_lexer import active_breaks, cross_run_cache_directory
from lib_function_units_base import LIBRARY_ROOT, PRODUCT_BEHAVIOR, SKIPPED_DIRECTORIES, UNIT_GRAPH_DIRECTORY_VARIABLE, UNIT_GRAPH_KEPT_PROGRAMS


UNIT_GRAPH_PRODUCT_VIEW_CACHE_DIRECTORY_NAME = "singlefs-unit-graph-product-view"


def unit_graph_product_view_cache_directory():
    """产品行为视图编出的 unit-graph 程序放哪（文件头「判定种类」里「unit-graph 从哪来」）：照 cross_run_cache_directory 放 singlefs-unit-graph-product-view。"""
    return cross_run_cache_directory(UNIT_GRAPH_PRODUCT_VIEW_CACHE_DIRECTORY_NAME)


UNIT_GRAPH_TEXT_BUSY_ATTEMPTS = 50        # 起 unit-graph 撞上 ETXTBSY 最多起几次
UNIT_GRAPH_TEXT_BUSY_PAUSE_SECONDS = 0.2  # 每两次之间等多久


def run_program_waiting_out_text_busy(command):
    """起 command、收它的输出；起的那一刻程序文件还有人开着写（ETXTBSY：同一台上别的进程 fork 时带走了写它的句柄，句柄一关就好）隔一会儿再起，
    最多 UNIT_GRAPH_TEXT_BUSY_ATTEMPTS 次，还不成照原样抛。别的 OSError 当场抛。弄坏开关 unit-graph-text-busy-not-waited 下撞上就抛。"""
    for attempt in range(1, UNIT_GRAPH_TEXT_BUSY_ATTEMPTS + 1):
        try:
            return subprocess.run(command, capture_output=True, text=True, errors="surrogateescape", stdin=subprocess.DEVNULL)
        except OSError as error:
            if error.errno != errno.ETXTBSY or attempt == UNIT_GRAPH_TEXT_BUSY_ATTEMPTS or "unit-graph-text-busy-not-waited" in active_breaks():
                raise
            time.sleep(UNIT_GRAPH_TEXT_BUSY_PAUSE_SECONDS)
    raise AssertionError("走不到：循环里末一次不是交回就是抛")


class ProductViewUnavailable(Exception):
    """拿不到 Rust 那一份产品行为视图（找不到或编不出 unit-graph、它退非 0）：有 verus! 块的文件记拒算 product-view-unavailable。"""


def product_view_dump(root):
    """unit-graph dump <根> --judgment-kind product-behavior 的输出行（文件头「判定种类」）；拿不到抛 ProductViewUnavailable。"""
    program = os.path.join(unit_graph_directory(), "unit-graph")
    try:
        completed = run_program_waiting_out_text_busy([program, "dump", os.path.abspath(root), "--judgment-kind", PRODUCT_BEHAVIOR])
    except OSError as error:
        raise ProductViewUnavailable(f"起不了 unit-graph：{error}") from error
    if completed.returncode != 0:
        tail = " | ".join(line for line in completed.stderr.strip().split("\n")[-2:] if line.strip())
        raise ProductViewUnavailable(f"unit-graph dump --judgment-kind {PRODUCT_BEHAVIOR} 退 {completed.returncode}：{tail[:200]}")
    return completed.stdout.splitlines()


def unit_graph_directory():
    """产品行为视图用的 unit-graph 所在的目录：设了 FUNCTION_UNITS_UNIT_GRAPH_DIRECTORY 取它，没设取这份库所在仓那一份编出来的（built_unit_graph）。"""
    return os.environ.get(UNIT_GRAPH_DIRECTORY_VARIABLE) or built_unit_graph(LIBRARY_ROOT)


def built_unit_graph(repository):
    """repository 里 crates/singlefs-unit-graph 编出来的 unit-graph 所在的目录（文件头「判定种类」里「unit-graph 从哪来」）；编不出抛 ProductViewUnavailable。"""
    source = os.path.join(repository, "crates", "singlefs-unit-graph")
    if not os.path.isfile(os.path.join(source, "Cargo.toml")):
        raise ProductViewUnavailable(f"{repository} 里没有 crates/singlefs-unit-graph")
    thread_registry = os.path.join(repository, ".claude", "heavy-thread-default")
    lock_file = os.path.join(repository, "Cargo.lock")
    cache = unit_graph_product_view_cache_directory()
    directory = os.path.join(cache, unit_graph_inputs_digest(repository, source, thread_registry, lock_file))
    program = os.path.join(directory, "unit-graph")
    if unit_graph_program_present(program):
        return directory
    lock_path = os.path.join(cache, ".lock")
    try:
        unit_graph_step("os.makedirs", cache, shared_directory, cache)
        with unit_graph_step("open", lock_path, open, lock_path, "a", encoding="utf-8") as lock:
            unit_graph_step("fcntl.flock", lock_path, fcntl.flock, lock, fcntl.LOCK_EX)
            # 拿到锁之后再判一次：等锁的时候别的进程（同一份共享盘上的另一台）可能已经换上
            if not unit_graph_program_present(program):
                build_unit_graph(repository, source, thread_registry, lock_file, cache, directory, program)
    except OSError as error:
        raise ProductViewUnavailable(f"编不了 unit-graph（{cache}）：{error}") from error
    return directory


def unit_graph_program_present(program):
    """缓存目录里的程序能直接用：有执行位，或文件在而本机看不到执行位（共享盘上别的机器刚换上、本机的属性缓存还是旧的）。
    换上只经 build_unit_graph 的 os.replace 整份换，文件在就是编完的那一份。弄坏开关 unit-graph-present-program-rebuilt 下只认执行位（看不到就重编）。"""
    if os.access(program, os.X_OK):
        return True
    return "unit-graph-present-program-rebuilt" not in active_breaks() and os.path.isfile(program)


def shared_directory(path):
    """建 path（连同上级）：别的进程已建好的照用；mkdir 说已在、本机却还看不出它是目录的（共享盘上别的机器刚建好，本机的目录项缓存还记着「没有」），
    列一次上级目录让本机重读那一层再看，还不是目录照原样抛。弄坏开关 unit-graph-directories-not-shared 下不带 exist_ok 建（已在就抛 EEXIST）。"""
    if "unit-graph-directories-not-shared" in active_breaks():
        os.makedirs(path)
        return
    try:
        os.makedirs(path, exist_ok=True)
    except FileExistsError:
        os.listdir(os.path.dirname(path) or ".")
        if not os.path.isdir(path):
            raise


def unit_graph_step(step, path, action, *arguments, **keywords):
    """编 unit-graph 的一步：起 action(*arguments, **keywords)、交它的返回值；抛 OSError 的改抛 ProductViewUnavailable，带上是哪一步（step：调的是哪个函数）、
    落在哪个路径（path）与原来的错误，几个调用都能抛同一个 errno 时分得出是哪一个。弄坏开关 unit-graph-failure-step-unnamed 下照原样抛（外层只报缓存目录与 errno）。"""
    try:
        return action(*arguments, **keywords)
    except OSError as error:
        if "unit-graph-failure-step-unnamed" in active_breaks():
            raise
        raise ProductViewUnavailable(f"编不了 unit-graph：{step}（{path}）抛 {type(error).__name__}：{error}") from error


def unit_graph_inputs_digest(repository, source, thread_registry, lock_file):
    """编 unit-graph 读的每一样（build_unit_graph 拷进工作区的登记文件与 Cargo.lock、source 整棵源码目录）的路径与字节摘要 → 缓存目录名：
    三样里改了一个字节就换目录、重编，旧程序不复用。弄坏开关 unit-graph-digest-skips-lock-file：摘要里不算 Cargo.lock。"""
    inputs = [thread_registry, lock_file]
    if "unit-graph-digest-skips-lock-file" in active_breaks():
        inputs.remove(lock_file)
    for directory, subdirectories, names in os.walk(source):
        subdirectories[:] = sorted(name for name in subdirectories if name not in SKIPPED_DIRECTORIES)
        inputs += [os.path.join(directory, name) for name in sorted(names)]
    digest = hashlib.sha256()
    for path in inputs:
        digest.update(os.path.relpath(path, repository).encode("utf-8", "surrogateescape") + b"\0")
        try:
            with open(path, "rb") as handle:
                digest.update(handle.read())
        except OSError:
            digest.update(b"<missing>")
        digest.update(b"\0")
    return digest.hexdigest()


def build_unit_graph(repository, source, thread_registry, lock_file, cache, directory, program):
    """在 cache 下的临时工作区里编 unit-graph、换上 program；编完删工作区与编译目录，cache 里只留最近 UNIT_GRAPH_KEPT_PROGRAMS 份程序。"""
    build = unit_graph_step("tempfile.mkdtemp", cache, tempfile.mkdtemp, prefix="build-", dir=cache)
    try:
        workspace = os.path.join(build, "workspace")
        member = os.path.join(workspace, "crates", "singlefs-unit-graph")
        unit_graph_step("shutil.copytree", member, shutil.copytree, source, member, ignore=shutil.ignore_patterns("target"))
        settings = os.path.join(workspace, ".claude")
        unit_graph_step("os.makedirs", settings, shared_directory, settings)
        copied_registry = os.path.join(settings, "heavy-thread-default")
        unit_graph_step("shutil.copy2", copied_registry, shutil.copy2, thread_registry, copied_registry)
        if os.path.isfile(lock_file):
            copied_lock = os.path.join(workspace, "Cargo.lock")
            unit_graph_step("shutil.copy2", copied_lock, shutil.copy2, lock_file, copied_lock)
        manifest = os.path.join(workspace, "Cargo.toml")
        with unit_graph_step("open", manifest, open, manifest, "w", encoding="utf-8") as handle:
            handle.write('[workspace]\nmembers = ["crates/singlefs-unit-graph"]\nresolver = "2"\n\n[profile.release]\nopt-level = 3\n'
                         'overflow-checks = true\ndebug-assertions = true\n')
        # 直接起 cargo：线程是服务给这一条套的 capped.sh 设好的 CARGO_BUILD_JOBS（capped.sh 只认服务套的那一层）
        completed = subprocess.run(["cargo", "build", "--release", "--offline",
                                    "--manifest-path", os.path.join(workspace, "Cargo.toml"), "-p", "singlefs-unit-graph", "--bin", "unit-graph"],
                                   capture_output=True, text=True, errors="surrogateescape", stdin=subprocess.DEVNULL,
                                   env=dict(os.environ, CARGO_TARGET_DIR=os.path.join(build, "target")))
        if completed.returncode != 0:
            tail = " | ".join(line for line in completed.stderr.strip().split("\n")[-3:] if line.strip())
            raise ProductViewUnavailable(f"cargo build --release -p singlefs-unit-graph 退 {completed.returncode}：{tail[:300]}")
        unit_graph_step("os.makedirs", directory, shared_directory, directory)
        # 临时名各取各的：两个进程同时换上同一份（锁在这个文件系统上不互斥时）不写同一个文件，改名换上的那一份没有人还开着写
        handle, partial = unit_graph_step("tempfile.mkstemp", directory, tempfile.mkstemp, prefix=".unit-graph-", dir=directory)
        os.close(handle)
        try:
            built = os.path.join(build, "target", "release", "unit-graph")
            unit_graph_step("shutil.copy2", built, shutil.copy2, built, partial)
            unit_graph_step("os.replace", program, os.replace, partial, program)
        finally:
            if os.path.lexists(partial):
                os.unlink(partial)
    finally:
        shutil.rmtree(build, ignore_errors=True)
    kept = sorted((name for name in os.listdir(cache) if not name.startswith((".", "build-"))),
                  key=lambda name: os.path.getmtime(os.path.join(cache, name)), reverse=True)
    for name in kept[UNIT_GRAPH_KEPT_PROGRAMS:]:
        shutil.rmtree(os.path.join(cache, name), ignore_errors=True)
