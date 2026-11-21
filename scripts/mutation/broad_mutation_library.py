"""broad-mutation.py 拆出的一份。
判定库（Library）、读表、取库目录与源码树、导出范围、清单的生成与缓存、规矩库与复用三问库的载入、变异补丁的施加与切分。
入口文件头写着这一份在的位置、弄坏开关表与用法；入口 broad-mutation.py 从各份 import 全部名字再导出，别处照旧按入口取名字。
拆出的各份只向前 import（base → judging → library → runner → gpu → run → selftest_a → selftest_b），没有环。
"""
import fcntl
import importlib.util
import json
import os
import secrets
import shutil
import subprocess
import sys
import tempfile
import time
from typing import NamedTuple
from broad_mutation_base import (BASELINES_TABLE, BASELINE_COLUMNS, COMPOSITIONS_TABLE, DEFAULT_HOME_ENTRY, EXIT_INCONSISTENT, EXIT_LOCKED,
    HUNK_HEADER, IGNORED_DIRECTORY_NAMES, LIBRARY_KIND, LISTS_TABLE, LOCK_POLL_SECONDS, MUTATED_PACKAGES, MUTATION_MARKER, NAME_LOCATION_PREFIX,
    ORDER_STATE, PRUNE_SERVER_CLIENT_MODULE, PRUNE_SERVER_CLIENT_PATH, REPOSITORY, SCRIPT_DIRECTORY, SELFTEST_WORK_VARIABLE, SELF_PATH,
    TEST_ONLY_VARIABLES, VERDICTS_TABLE, VERDICT_COLUMNS, VOLATILE_FILESYSTEMS, VOLATILE_PREFIXES, run_build_directory)
from broad_mutation_judging import (break_is, file_digest, note, raw_break, refuse, replace_span, say, test_mode, test_variable, today_text,
    write_atomically)


class Stopped(BaseException):
    """驱动收到 TERM / INT：在途那一条不落行，拷回原件再退。"""


def print_refusal(refusal):
    print(f"  ✗ {refusal.message}", file=sys.stderr)
    print(f"  → 怎么办：{refusal.howto}", file=sys.stderr)


def path_is_under(path, directory):
    return path == directory or path.startswith(directory.rstrip("/") + "/")


def refuse_outside_selftest(present):
    """只供自证的变量设着而这个进程不是 --selftest 自己起的子进程：退 2，点名是哪几个变量只在自证里认。"""
    unset_command = "env " + " ".join(f"-u {name}" for name in present) + f" python3 {SELF_PATH} …"
    refuse(f"环境里设了只供 --selftest 用的变量（{'、'.join(present)}），而这个进程不是 --selftest 自己起的子进程：假的一步、假清单与弄坏开关不许进真跑",
           f"清掉再跑：{unset_command}；要看这些变量怎么用，读文件头「只供测试」一段")


def check_test_only_variables_before_parsing(argv):
    """只供自证的变量平常跑设了就拒，排在一切参数解析之前（认不得的选项也先看这个）；命令行带 --selftest 的是自证本身，放过。
    弄坏开关 break-guard-after-parsing：不在这里拒，退回参数解析之后的 check_test_only_variables 去拒。"""
    present = [name for name in TEST_ONLY_VARIABLES if os.environ.get(name)]
    if present and "--selftest" not in argv and not raw_break("break-guard-after-parsing") and not test_mode():
        refuse_outside_selftest(present)


def check_test_only_variables(home):
    """只供自证的变量出现在自证之外、或指着自证工作目录之外的库：拒，一步都不做。"""
    present = [name for name in TEST_ONLY_VARIABLES if os.environ.get(name)]
    if not present:
        return
    unset_command = "env " + " ".join(f"-u {name}" for name in present) + f" python3 {SELF_PATH} …"
    if not test_mode():
        refuse_outside_selftest(present)
    work = os.environ.get(SELFTEST_WORK_VARIABLE)
    if raw_break("test-variables-anywhere"):
        return
    if not work or not path_is_under(home, os.path.realpath(work)):
        refuse(f"环境里设了只供 --selftest 用的变量（{'、'.join(present)}），而库目录 {home} 不在这一趟自证的工作目录（{work or '没给'}）里：只供自证的变量不许碰别的库",
               f"自证里的库都建在它自己的工作目录下；真库上跑就清掉这些变量：{unset_command}")


def clean_field(value):
    return str(value).replace("\t", " ").replace("\r", " ").replace("\n", " ")


# ── 仓里的登记表（排除表）：本地文件 ──────────────────────────────────────────────────────────────────────

class TableRead(NamedTuple):
    rows: list              # 成形的行（段数对），按文件里的次序
    malformed_lines: list   # 写完了（有换行结尾）而段数不对的行的行号
    partial_tail: str       # 末行没写完（没有换行结尾）时它的文字，不算数；没有是 ""


def read_table(path, column_count):
    """读只追加的表。# 起头的行与空行不算；末行没有换行结尾的是没写完的半行，不算数也不算不成形。"""
    rows, malformed_lines, partial_tail = [], [], ""
    if not os.path.exists(path):
        return TableRead(rows, malformed_lines, partial_tail)
    with open(path, "rb") as handle:
        data = handle.read()
    lines = data.decode("utf-8", errors="replace").split("\n")
    if lines[-1] != "":
        partial_tail = lines[-1]
    for number, line in enumerate(lines[:-1], start=1):
        if not line or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) != column_count:
            malformed_lines.append(number)
            continue
        rows.append(fields)
    return TableRead(rows, malformed_lines, partial_tail)


# ── 判定库：prune-server 里那一份（文件头「判定库」） ─────────────────────────────────────────────────────────

def prune_server_client():
    """scripts/prune-server/prune_server_client.py（判定库服务的客户端），照 PRUNE_SERVER_CLIENT_MODULE 那个名字只载入一次；载入不连服务。"""
    if PRUNE_SERVER_CLIENT_MODULE not in sys.modules:
        specification = importlib.util.spec_from_file_location(PRUNE_SERVER_CLIENT_MODULE, PRUNE_SERVER_CLIENT_PATH)
        module = importlib.util.module_from_spec(specification)
        sys.modules[PRUNE_SERVER_CLIENT_MODULE] = module
        specification.loader.exec_module(module)
    return sys.modules[PRUNE_SERVER_CLIENT_MODULE]


def order_stamp():
    """只追加的表里一行的条目名（文件头「判定库」<次序>）：写入时刻的纳秒（这个进程里只增）、进程号、这个进程里的序号、随机；按名字升序就是写入的次序。"""
    now = max(time.time_ns(), ORDER_STATE["last"] + 1)
    ORDER_STATE["last"] = now
    ORDER_STATE["count"] += 1
    return f"{now:020d}-{os.getpid():010d}-{ORDER_STATE['count']:08d}-{secrets.token_hex(4)}"


class LibraryRows(NamedTuple):
    rows: list              # 段数对的行（{列名: 值}），按写入的次序
    malformed: list         # 段数不对的条目（<表>/<次序>）：不算判定

    def malformed_text(self):
        return f"{len(self.malformed)} 行（例：条目 {self.malformed[0]}）"


# ── 库目录、锁 ─────────────────────────────────────────────────────────────────────────────────────────────

def filesystem_type_of(path):
    best, best_type = "", ""
    try:
        with open("/proc/mounts", encoding="utf-8", errors="replace") as handle:
            for line in handle:
                parts = line.split()
                if len(parts) < 3:
                    continue
                mount_point = parts[1].replace("\\040", " ")
                if (path == mount_point or path.startswith(mount_point.rstrip("/") + "/")) and len(mount_point) > len(best):
                    best, best_type = mount_point, parts[2]
    except OSError:
        return ""
    return best_type


def acquire_lock(home, wait_seconds):
    """库目录的排他锁。描述符可继承：每一步命令都带着它，驱动被 KILL 之后那些进程还活着时锁还占着，下一趟看得见。"""
    os.makedirs(home, exist_ok=True)
    descriptor = os.open(os.path.join(home, "lock"), os.O_RDWR | os.O_CREAT, 0o644)
    os.set_inheritable(descriptor, True)
    deadline = time.monotonic() + wait_seconds
    announced = False
    while True:
        try:
            fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
            return descriptor
        except OSError:
            if time.monotonic() >= deadline:
                os.close(descriptor)
                refuse(f"库目录 {home} 的锁被占着（等了 {wait_seconds} 秒）：另一趟驱动在跑，或上一趟被 KILL 之后它起的那一步（它起的 cargo 与测试进程）还没退",
                       "`ps -o pid,ppid,etimes,args -u \"$(id -u)\"` 找出还在跑的那一趟（cargo、cargo-nextest、测试二进制），"
                       "等它退出再重跑同一条命令；确是上一趟留下的，逐个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>` 停掉再跑；"
                       "要多等就加 --lock-wait <秒>", EXIT_LOCKED)
            if not announced:
                note(f"  · 库目录的锁被占着，最多等 {wait_seconds} 秒（上一趟留下的进程退出就放开）")
                announced = True
            time.sleep(LOCK_POLL_SECONDS)


def export_paths():
    """导出哪些路径：门禁 crates-mutation-replay 登记的拷贝范围（去掉 research/ 下的：那是那道门禁自己的脚本）加 .cargo。
    拷贝范围已登记了 .cargo 就不再加。"""
    specification = importlib.util.spec_from_file_location("crates_mutation_rows_for_broad_mutation",
                                                           os.path.join(SCRIPT_DIRECTORY, "crates-mutation-rows.py"))
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    try:
        copied = module.copied_paths(REPOSITORY)
        copied = module.copied_with_path_dependencies(REPOSITORY, copied, os.environ)
    except Exception as error:  # 登记表读不了、没有那一行
        refuse(f"读不出导出范围（scripts/mutation/crates-mutation-rows.py 的 copied_paths）：{error}",
               "看 .claude/gate.d/stage-inputs.tsv 里 crates-mutation-replay.sh 那一行还在不在、成不成形")
    paths = [path for path in copied if not path.startswith("research/")]
    if ".cargo" not in paths:
        paths.append(".cargo")
    if "crates" not in paths:
        refuse(f"导出范围里没有 crates（读到的是 {copied}）", "看 .claude/gate.d/stage-inputs.tsv 里 crates-mutation-replay.sh 那一行登记的路径")
    return paths


def remove_tree(path):
    if os.path.lexists(path):
        shutil.rmtree(path)


# ── 库目录 ─────────────────────────────────────────────────────────────────────────────────────────────────

class Library:
    """库目录里的各个位置（文件头「库目录」：运行目录、日志、锁）与判定库（文件头「判定库」：prune-server 里那一份，库目录只当它的命名空间的名字，本地不建）。"""

    def __init__(self, home):
        self.home = home
        self.runs = os.path.join(home, "runs")
        self.run_directory = None
        self.run_directory_from_service = False   # 运行目录是服务给的编译目录底下那一份（docstring「库目录」runs/）：收尾不删
        # 专用树、编译目录、每一步的 TMPDIR 与在途记录都在这一趟的运行目录里（文件头「库目录」runs/），begin_run 之前没有。
        # 专用树先是空串：只读的用法不开跑，Judgement 里 os.path.isdir 判它不在，弄坏开关下才读的构建环境与 rustc -Vv 就在 --source 上算
        self.tree = ""
        self.target = self.target_coverage = self.scratch = None
        self.inflight_record = self.inflight_original = self.inflight_restore = None
        self.tree_synced = os.path.join(home, "tree.synced")
        self.logs = os.path.join(home, "logs")
        self.manifest = None
        self.lock_descriptor = None
        self.deferred_rows = []
        client = prune_server_client()
        self.store = client.library_store(client.FAMILY_BROAD_MUTATION, LIBRARY_KIND, home)

    def location(self):
        """判定库在服务里的前缀（报错与给人看的那一行用；不是本地路径）。"""
        return self.store.location()

    def scan(self, relative_prefix):
        """按前缀扫判定库：[(相对路径, 值)]，按键升序。服务不可用客户端退 75（弄坏开关 unreachable-as-empty-library 下交空表）。"""
        try:
            return self.store.scan(relative_prefix)
        except prune_server_client().PruneServerFailure:
            if break_is("unreachable-as-empty-library"):
                return []
            raise

    def read(self, relative):
        """判定库里一个条目的值；不在交 None。服务不可用客户端退 75（弄坏开关 unreachable-as-empty-library 下交 None）。"""
        try:
            return self.store.read(relative)
        except prune_server_client().PruneServerFailure:
            if break_is("unreachable-as-empty-library"):
                return None
            raise

    def read_rows(self, table, columns):
        """一张只追加的表（文件头「判定库」）：按条目名（写入的次序）扫回来；段数对的行交 {列名: 值}，不对的条目列进 malformed。"""
        rows, malformed = [], []
        for relative, value in self.scan(f"{table}/"):
            fields = value.decode("utf-8", "surrogateescape").split("\t")
            if len(fields) != len(columns):
                malformed.append(relative)
                continue
            rows.append(dict(zip(columns, fields)))
        return LibraryRows(rows, malformed)

    def append_rows(self, table, rows):
        """往一张只追加的表里写几行（一帧一个写批）：一行一个条目、条目名是写入的次序（order_stamp），服务回确认才返回。"""
        if rows:
            self.store.write_many([(f"{table}/{order_stamp()}", "\t".join(clean_field(field) for field in fields)) for fields in rows])

    def read_verdicts(self):
        table = self.read_rows(VERDICTS_TABLE, VERDICT_COLUMNS)
        deferred = [dict(zip(VERDICT_COLUMNS, [clean_field(field) for field in row])) for row in self.deferred_rows]
        return LibraryRows(table.rows + deferred, table.malformed)

    def record_verdict(self, fields):
        """落一行判定（十六段）：写进判定库、等服务回确认（文件头「一条一条跑」）。"""
        if break_is("no-row-flush"):
            self.deferred_rows.append(fields)
            return
        self.append_rows(VERDICTS_TABLE, [fields])

    def flush_deferred_rows(self):
        if self.deferred_rows:
            self.append_rows(VERDICTS_TABLE, self.deferred_rows)
        self.deferred_rows = []

    def write_composition(self, key, lines):
        """一把键的组成清单（文件头「判定库」compositions/）：同一把键只写一次。"""
        if not key:
            return
        relative = f"{COMPOSITIONS_TABLE}/{key}"
        if self.read(relative) is not None:
            return
        self.store.write(relative, "".join(f"{line}\n" for line in lines))

    def read_composition(self, key):
        data = self.read(f"{COMPOSITIONS_TABLE}/{key}") if key else None
        return None if data is None else data.decode("utf-8", "surrogateescape").splitlines()

    def read_list_cache(self):
        """清单按单元存的那张表（文件头「判定库」lists/）：{清单的单元键: [这个单元里的变异]}。"""
        cache = {}
        for relative, value in self.scan(f"{LISTS_TABLE}/"):
            try:
                cache[relative[len(LISTS_TABLE) + 1:]] = json.loads(value.decode("utf-8", "surrogateescape"))
            except ValueError:
                continue
        return cache

    def write_list_cache(self, rows):
        """清单按单元记：[(清单的单元键, 这个单元里的变异的 JSON)]，同一个单元键再写就盖掉。"""
        self.store.write_many([(f"{LISTS_TABLE}/{unit_key}", mutations) for unit_key, mutations in rows])

    def baseline_rows(self):
        """基线表里段数对的行（{列名: 值}），按写入的次序。"""
        return self.read_rows(BASELINES_TABLE, BASELINE_COLUMNS).rows

    def place_run_directory(self, directory):
        self.run_directory = directory
        self.tree = os.path.join(directory, "tree")
        self.target = os.path.join(directory, "target")
        self.target_coverage = os.path.join(directory, "target-coverage")
        self.scratch = os.path.join(directory, "scratch")
        self.inflight_record = os.path.join(directory, "inflight.json")
        self.inflight_original = os.path.join(directory, "inflight.original")
        self.inflight_restore = os.path.join(directory, "inflight.restore")

    def begin_run(self):
        """这一趟的运行目录（文件头「库目录」runs/）：拿到锁之后先删上一趟被 KILL 留下的运行目录（树、编译目录、在途记录；删之前核没有进程开着里面的文件），
        再新建这一趟的 <库目录>/runs/run-<进程号>-<随机>/。交上一趟被打断在哪一条的在途记录（没有交 None）。
        弄坏开关 interrupted-run-directory-kept：不删上一趟留下的、不认它的在途记录；build-directory-in-home：把树、编译目录与在途记录放在库目录里、跨趟留着。"""
        if break_is("build-directory-in-home"):
            os.makedirs(self.home, exist_ok=True)
            self.place_run_directory(self.home)
            self.run_directory = None
            return None
        interrupted = None
        os.makedirs(self.runs, exist_ok=True)
        if not break_is("interrupted-run-directory-kept"):
            for name in sorted(os.listdir(self.runs)):
                leftover = os.path.join(self.runs, name)
                record_path = os.path.join(leftover, "inflight.json")
                if os.path.isfile(record_path):
                    try:
                        with open(record_path, encoding="utf-8") as handle:
                            interrupted = json.load(handle)
                    except (OSError, ValueError):
                        interrupted = None
                    if interrupted is not None:
                        say(f"  · 上一趟被打断在第 {interrupted['sequence']} 条（改的是 {interrupted['file']}）：它那一趟的运行目录删掉，这一趟在新换的树上重判它")
                note(f"  · 上一趟留下的运行目录 {leftover}（那一趟被 KILL，或删的时候有进程开着里面的文件）：")
                for line in run_build_directory.remove_run_build_directory(leftover)[1]:
                    note(line)
        given = service_build_directory()
        if given is not None:
            return self.begin_run_in_service_build_directory(given, interrupted)
        self.place_run_directory(tempfile.mkdtemp(prefix=f"run-{os.getpid()}-", dir=self.runs))
        return interrupted

    def begin_run_in_service_build_directory(self, given, interrupted):
        """服务给了编译目录（docstring「库目录」runs/ 那一段）：运行目录用 <它>/broad-mutation-run/，没有就建、有就接着用（撞上限重来的那一次照用上一次编出的产物）。
        上一次尝试被停在半路留下的在途记录：报被打断在哪一条、删掉在途记录（改过的那份文件由接着的换树照源码拷回）。交被打断在哪一条的在途记录（没有交 interrupted 原样）。"""
        directory = os.path.join(given, "broad-mutation-run")
        os.makedirs(directory, exist_ok=True)
        self.place_run_directory(directory)
        self.run_directory_from_service = True
        if os.path.isfile(self.inflight_record):
            try:
                with open(self.inflight_record, encoding="utf-8") as handle:
                    interrupted = json.load(handle)
            except (OSError, ValueError):
                pass
            if interrupted is not None:
                say(f"  · 上一次尝试被打断在第 {interrupted['sequence']} 条（改的是 {interrupted['file']}）：删掉它的在途记录，换树把那份文件照源码拷回，这一趟重判它")
            for path in (self.inflight_record, self.inflight_original, self.inflight_restore):
                if os.path.lexists(path):
                    os.remove(path)
        say(f"  · 运行目录用服务给的编译目录 {directory}（撞上限重来时接着用上一次编出的产物；收尾不删，服务在任务有了结局时删）")
        return interrupted

    def end_run(self):
        """删这一趟的运行目录（正常结束、拒绝、被 TERM / INT 停都在 main 的 finally 里走到）：删之前核本机没有进程开着里面的文件，
        有就不删、逐个报进程（scripts/hosts/run_build_directory.py）。交删成了没有；没建过（build-directory-in-home）交 True。
        弄坏开关 run-directory-kept：不删；run-directory-open-files-unchecked：不核开着的文件就删。"""
        directory, self.run_directory = self.run_directory, None
        if directory is None:
            return True
        if self.run_directory_from_service:
            return True    # 服务给的编译目录底下那一份（begin_run_in_service_build_directory）：不删，服务在任务有了结局时删
        if break_is("run-directory-kept"):
            return True
        removed, lines = run_build_directory.remove_run_build_directory(directory, check_open=not break_is("run-directory-open-files-unchecked"))
        for line in lines:
            note(line)
        return removed


def service_build_directory():
    """环境里给的编译目录（docstring「库目录」runs/ 那一段；会编的入口由统一任务服务给这一次尝试一份）：CARGO_TARGET_DIR 设了交它的绝对路径，没设交 None。
    变量名写在函数里、不做模块级常量。弄坏开关 service-build-directory-ignored：一律交 None（同没给时一样在 runs/ 下新建、跑完删）。"""
    given = os.environ.get("CARGO_TARGET_DIR", "").strip()
    if not given or break_is("service-build-directory-ignored"):
        return None
    return os.path.realpath(given)


def default_home():
    """库目录的缺省：数据根下的 broad-mutation-units（.claude/hooks/lib_work_root.py 的 shared_data_path，由多机配置定）；那份库读不到、取不到就拒，不经 ~ 取。"""
    try:
        specification = importlib.util.spec_from_file_location("lib_work_root_for_broad_mutation", os.path.join(REPOSITORY, ".claude", "hooks", "lib_work_root.py"))
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        return module.shared_data_path(DEFAULT_HOME_ENTRY)
    except (OSError, ImportError, AttributeError, ValueError) as error:
        refuse(f"取不到库目录的缺省（数据根下的 {DEFAULT_HOME_ENTRY}，.claude/hooks/lib_work_root.py：{error}）",
               "修好那份库、多机配置里写上 SHARED_DATA_ROOT，或给 --home <目录>")


def resolve_home(given, read_only=False):
    """库目录：先核只供自证的变量；要往库里写的用法另拒落在开机会清空的目录里的库。只读的用法（--list-items、--summary）
    不写库，落在哪都照读：共享门禁的 stage-selftest.sh 把样本拷进 mktemp -d（/tmp）再判，样本的库就在那里。"""
    home = os.path.realpath(given or default_home())
    # 只供自证的变量先判：它们带着假的一步与弄坏开关，出现在自证之外比库目录落在哪更根本；放在后面，临时目录那一拒会先把它盖掉
    check_test_only_variables(home)
    if test_variable("BROAD_MUTATION_ALLOW_VOLATILE_HOME") == "1":
        return home
    if read_only and not break_is("read-only-volatile-refused"):
        return home
    existing = home
    while not os.path.exists(existing):
        existing = os.path.dirname(existing)
    under_volatile = any(home == prefix or home.startswith(prefix + "/") for prefix in VOLATILE_PREFIXES)
    if under_volatile or filesystem_type_of(existing) in VOLATILE_FILESYSTEMS:
        refuse(f"库目录 {home} 落在开机会清空的目录里（/tmp、/run、/dev/shm 或 tmpfs）：判定会写进只给自证与样本用的那一份命名空间，进不了两台共用的判定库，运行目录一重启也没了",
               f"换一个落在盘上的目录：--home <目录>；默认是数据根下的 {DEFAULT_HOME_ENTRY}")
    return home


def resolve_source(given, home, read_only=False):
    """被判的那棵工作树（--source，默认本仓根）。要换树的用法里它与库目录不许一个套着另一个；只读的用法（不换树）不拦。"""
    source = os.path.realpath(os.path.expanduser(given or REPOSITORY))
    if not os.path.isdir(source):
        refuse(f"--source 给的 {source} 不是目录", "给被判的那棵工作树（主工作区或一份副本），不给就是本仓根")
    if not read_only and (path_is_under(source, home) or path_is_under(home, source)):
        refuse(f"--source {source} 与库目录 {home} 一个套着另一个：换树会把库目录自己（专用树、编译目录）当源码拷进去",
               "库目录与被判的工作树各在各处：库目录默认在数据根下（.claude/hooks/lib_work_root.py 的 shared_data_path）")
    return source


# ── 换树：把被判的工作树逐文件换进专用树 ───────────────────────────────────────────────────────────────────

def files_in_tree(tree):
    """树里每个文件的相对路径（符号链接也算一个）；叫 target / mutants.out 的目录与 .git 不算。"""
    found = set()
    for directory, directory_names, file_names in os.walk(tree):
        directory_names[:] = sorted(name for name in directory_names if name not in IGNORED_DIRECTORY_NAMES and name != ".git")
        for file_name in file_names:
            found.add(os.path.relpath(os.path.join(directory, file_name), tree))
    return found


def exported_files(source, paths):
    """被判那棵树在导出范围里的文件：{相对路径: 源文件的绝对路径}（跟着符号链接取内容）。"""
    found = {}
    for path in paths:
        full = os.path.normpath(os.path.join(source, path))
        if os.path.isfile(full):
            found[os.path.relpath(full, source)] = full
            continue
        if not os.path.isdir(full):
            continue
        for relative in files_in_tree(full):
            candidate = os.path.join(full, relative)
            if os.path.isfile(candidate):
                found[os.path.relpath(candidate, source)] = candidate
    return found


def sync_tree(library, source):
    """换树（文件头「库目录」runs/ 的 tree/）：把导出范围拷进这一趟新建的专用树（内容不同的才写、mtime 取此刻，源码里没有的删掉）。交 (改了几个, 删了几个)。"""
    paths = ["."] if test_variable("BROAD_MUTATION_EXPORT_EVERYTHING") == "1" else export_paths()
    wanted = exported_files(source, paths)
    if break_is("exclude-re-dropped"):
        # 弄坏开关：.cargo/mutants.toml 不拷进专用树，cargo-mutants 读不到里面的 exclude_re（等于忽略它），清单按单元存的键也不随它变
        wanted.pop(os.path.join(".cargo", "mutants.toml"), None)
    if not any(path.endswith(".rs") for path in wanted):
        refuse(f"被判的那棵树 {source} 在导出范围（{' '.join(paths)}）里一份 .rs 都没有", "--source 给被判的仓根（主工作区或一份副本）")
    os.makedirs(library.tree, exist_ok=True)
    manifest, changed, removed = {}, 0, 0
    for relative, origin in sorted(wanted.items()):
        digest = file_digest(origin)
        manifest[relative] = digest
        target = os.path.join(library.tree, relative)
        if os.path.isfile(target) and not os.path.islink(target) and file_digest(target) == digest:
            continue
        os.makedirs(os.path.dirname(target), exist_ok=True)
        if os.path.lexists(target):
            os.remove(target)
        shutil.copyfile(origin, target)
        shutil.copymode(origin, target)
        changed += 1
    for relative in sorted(files_in_tree(library.tree) - set(wanted)):
        os.remove(os.path.join(library.tree, relative))
        removed += 1
    library.manifest = manifest
    write_atomically(library.tree_synced, (json.dumps({"synced_on": today_text(), "source": source, "files": len(manifest), "changed": changed,
                                                       "removed": removed}, ensure_ascii=False) + "\n").encode())
    say(f"  · 换树：{source} 导出范围里 {len(manifest)} 个文件，{changed} 个内容不同、照写进专用树，删了 {removed} 个源码里没有的")
    return changed, removed


# ── 清单：由 cargo-mutants 生成，逐条核对它给的 diff，按单元存 ───────────────────────────────────────────────

def canonical_items(items):
    """清单的次序与序号（文件头「选哪些条」）：先 core 后 checker，同包里按文件、起止、替换文、变异名排；同名的另带「（第 N 个同名）」。"""
    rank = {package: index for index, package in enumerate(MUTATED_PACKAGES)}
    items.sort(key=lambda item: (rank.get(item["package"], len(rank)), item["file"], tuple(item["start"]), tuple(item["end"]), item["replacement"], item["name"]))
    seen = {}
    for index, item in enumerate(items, start=1):
        base = item["name"]
        seen[base] = seen.get(base, 0) + 1
        if seen[base] > 1:
            item["name"] = f"{base}（第 {seen[base]} 个同名）"
        item["sequence"] = index
    return items


def generate_list(library):
    """此刻专用树上的清单：[{sequence, package, name, file, start, end, replacement}]。每条都拿 cargo-mutants 给的 diff 核一遍
    「照区间换掉」与它自己的改法逐字相同：区间的读法错了会让每一条都改错地方，那样整份结果是假的。"""
    items, original_texts = [], {}
    span_howto = "看 replace_span 对区间的读法（1 起、列按字符、end 不含）与这一份 cargo-mutants 是不是还一致；对不上就不生成清单"
    for package in MUTATED_PACKAGES:
        for mutant in raw_mutants_of_package(library, package):
            file_path, span, replacement = mutant.get("file"), mutant.get("span"), mutant.get("replacement")
            if not isinstance(file_path, str) or not isinstance(span, dict) or not isinstance(replacement, str) or "diff" not in mutant:
                refuse(f"cargo-mutants 给的清单里有一项缺 file / span / replacement / diff：{json.dumps(mutant, ensure_ascii=False)[:300]}",
                       "这一份 cargo-mutants 的 --list --json --diff 输出换了形状：对着 https://mutants.rs/ 改 generate_list 的读法，不照猜的往下跑")
            start = (span["start"]["line"], span["start"]["column"])
            end = (span["end"]["line"], span["end"]["column"])
            if file_path not in original_texts:
                with open(os.path.join(library.tree, file_path), encoding="utf-8") as handle:
                    original_texts[file_path] = handle.read()
            original = original_texts[file_path]
            try:
                mutated = replace_span(original, start, end, replacement + MUTATION_MARKER)
                by_diff = apply_unified_diff(original, mutant["diff"])
            except ValueError as error:
                refuse(f"清单里「{mutant['name']}」这一条套不上原文：{error}", span_howto)
            if mutated == original or mutated.rstrip("\n") != by_diff.rstrip("\n"):
                refuse(f"清单里「{mutant['name']}」照区间换掉之后与 cargo-mutants 自己给的 diff 改出来的不一样（或根本没改动）", span_howto)
            items.append({"package": mutant.get("package") or package, "name": clean_field(mutant["name"]), "file": file_path,
                          "start": list(start), "end": list(end), "replacement": replacement})
    if not items:
        refuse("cargo-mutants 生成的清单是空的", "看树里的 .cargo/mutants.toml 的 examine_globs 与 --package 的包名还对不对得上")
    return canonical_items(items)


def item_identity(item):
    """清单里一条的身份（复原核对用）：包、文件、起止、替换文与变异名去掉位置的那一段（位置那一段由复原照此刻的起点重写）。"""
    return (item["package"], item["file"], tuple(item["start"]), tuple(item["end"]), item["replacement"], NAME_LOCATION_PREFIX.sub("", item["name"], count=1))


def record_list_cache(library, judgement, items):
    """清单按单元记进判定库 lists/（文件头「判定库」）：被变异包里每一个单元一行（没有变异的记空表）；之后照这张表在此刻的源码上复原一遍，
    与 cargo-mutants 给的逐条比（变异名、区间、替换文），复原不回去就拒（退 1）。交 (新写了几行, 单元数)。"""
    rules, tree = judgement.rules, judgement.tree
    by_unit = {}
    for item in items:
        unit = tree.container(item["file"], tuple(item["start"]), tuple(item["end"]))
        if unit is None:
            refuse(f"清单里第 {item['sequence']} 条（{item['name']}）落不进任何单元：{item['file']}:{item['start'][0]} 一带切不开",
                   "函数外的项各是单元，切不开是 scripts/gate/lib_function_units.py 的切分缺陷：把那一段与这句话交给改它的人修；不退回按整个文件算",
                   EXIT_INCONSISTENT)
        by_unit.setdefault(id(unit), []).append(rules.list_entry(tree, unit, item))
    cache = library.read_list_cache()
    rows, count = [], 0
    for path in judgement.package_files(MUTATED_PACKAGES):
        for unit in tree.code_units(path):
            count += 1
            unit_key = rules.list_unit_key(tree, unit, judgement.mutants_version, judgement.mutants_configuration)
            entries = by_unit.get(id(unit), [])
            if cache.get(unit_key) != entries:
                rows.append([unit_key, json.dumps(entries, ensure_ascii=False, sort_keys=True)])
                cache[unit_key] = entries
    if rows:
        library.write_list_cache(rows)
    restored, unlisted, unrestorable = restore_list(judgement, cache)
    if [item_identity(item) for item in restored] != [item_identity(item) for item in items] and not break_is("list-cache-unchecked"):
        missing = sorted(set(map(item_identity, items)) - set(map(item_identity, restored)))
        extra = sorted(set(map(item_identity, restored)) - set(map(item_identity, items)))
        refuse(f"清单按单元存之后复原不回去：cargo-mutants 给了 {len(items)} 条，照 lists/ 复原出 {len(restored)} 条（少 {len(missing)} 条，例 {missing[:1]}；"
               f"多 {len(extra)} 条，例 {extra[:1]}；没记清单的单元 {len(unlisted)} 个；复原不回去的 {len(unrestorable)} 处）",
               "看 scripts/mutation/mutation-prune.py 的 list_entry、restore_item 与 list_unit_key：相对位置、原文与变异名去掉位置的那一段要复原得回 cargo-mutants 给的那一条",
               EXIT_INCONSISTENT)
    return len(rows), count


def restore_list(judgement, cache):
    """照清单按单元存的那张表，在此刻的源码上复原清单：交 (清单, 没记清单的单元, 复原不回去的 [(单元, 那一条)])。"""
    rules, tree = judgement.rules, judgement.tree
    items, unlisted, unrestorable = [], [], []
    for package in MUTATED_PACKAGES:
        for path in judgement.package_files([package]):
            for unit in tree.code_units(path):
                unit_key = rules.list_unit_key(tree, unit, judgement.mutants_version, judgement.mutants_configuration)
                if unit_key not in cache:
                    unlisted.append(unit)
                    continue
                for entry in cache[unit_key]:
                    item = rules.restore_item(tree, unit, entry)
                    if item is None:
                        unrestorable.append((unit, entry))
                        continue
                    item["package"] = package
                    items.append(item)
    return canonical_items(items), unlisted, unrestorable


# 复用三问与复用哨兵全仓一份（scripts/gate/lib_reuse_checks.py 文件头「一」「二」）：第一次用到（复用哨兵、自证）时才载入，不放在模块级
REUSE_CHECKS_MODULE = None


def reuse_checks_library():
    global REUSE_CHECKS_MODULE
    if REUSE_CHECKS_MODULE is None:
        specification = importlib.util.spec_from_file_location("lib_reuse_checks_for_broad_mutation", os.path.join(SCRIPT_DIRECTORY, "..", "gate", "lib_reuse_checks.py"))
        REUSE_CHECKS_MODULE = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(REUSE_CHECKS_MODULE)
    return REUSE_CHECKS_MODULE


def apply_unified_diff(original_text, diff_text):
    """照一份 unified diff 改 original_text，交回改后的文本；上下文或删掉的行与原文对不上抛 ValueError。"""
    original, result, position = original_text.split("\n"), [], 0
    lines = diff_text.split("\n")
    index = 0
    while index < len(lines) and not lines[index].startswith("@@"):
        index += 1
    while index < len(lines):
        header = HUNK_HEADER.match(lines[index])
        if header is None:
            raise ValueError(f"认不出的 hunk 头：{lines[index]!r}")
        old_start, old_count = int(header.group(1)), int(header.group(2) if header.group(2) is not None else "1")
        hunk_start = old_start - 1 if old_count > 0 else old_start
        if hunk_start < position:
            raise ValueError("hunk 次序倒了")
        result.extend(original[position:hunk_start])
        position = hunk_start
        index += 1
        while index < len(lines) and not lines[index].startswith("@@"):
            line = lines[index]
            index += 1
            if line.startswith("\\"):
                continue
            if line == "" and index == len(lines):
                break
            tag, body = (line[:1], line[1:]) if line else (" ", "")
            if tag in (" ", "-"):
                if position >= len(original) or original[position] != body:
                    raise ValueError(f"diff 的第 {position + 1} 行与原文对不上")
                if tag == " ":
                    result.append(body)
                position += 1
            elif tag == "+":
                result.append(body)
            else:
                raise ValueError(f"认不出的 diff 行：{line!r}")
    result.extend(original[position:])
    return "\n".join(result)


def raw_mutants_of_package(library, package):
    """一个包的原始清单（cargo mutants --list --json --diff 的那个数组，每项补上 name）。"""
    environment = dict(os.environ, CARGO_TARGET_DIR=library.target)
    fake_driver = test_variable("BROAD_MUTATION_LIST_DRIVER")
    if fake_driver:
        listed = subprocess.run([fake_driver, package], capture_output=True, text=True, cwd=library.tree, stdin=subprocess.DEVNULL)
        if listed.returncode != 0:
            refuse(f"假的清单驱动退 {listed.returncode}：{listed.stderr.strip()}", "看 BROAD_MUTATION_LIST_DRIVER 指的脚本")
        return json.loads(listed.stdout)
    if shutil.which("cargo-mutants") is None:
        refuse("PATH 上没有 cargo-mutants，生成不了清单", "cargo install cargo-mutants 之后重跑同一条命令")
    base = ["cargo", "mutants", "--list", "--package", package]
    as_json = subprocess.run(base + ["--json", "--diff"], capture_output=True, text=True, cwd=library.tree, env=environment, stdin=subprocess.DEVNULL)
    if as_json.returncode != 0:
        refuse(f"cargo mutants --list --json --diff --package {package} 退 {as_json.returncode}：{as_json.stderr.strip()[-600:]}",
               f"在 {library.tree} 里手跑这一条看报错；树里的 .cargo/mutants.toml 是换进来的那一份")
    mutants = json.loads(as_json.stdout)
    if all(isinstance(mutant.get("name"), str) for mutant in mutants):
        return mutants
    as_text = subprocess.run(base, capture_output=True, text=True, cwd=library.tree, env=environment, stdin=subprocess.DEVNULL)
    names = [line for line in as_text.stdout.split("\n") if line.strip()]
    if as_text.returncode != 0 or len(names) != len(mutants):
        refuse(f"cargo mutants --list --package {package} 的文本清单（{len(names)} 行，退 {as_text.returncode}）与 JSON 清单（{len(mutants)} 项）条数对不上，变异名配不上",
               "看这一版 cargo-mutants 的 --list 输出有没有多出不是变异名的行；对不上就不生成清单，不拿猜的名字往下跑")
    for mutant, name in zip(mutants, names):
        mutant["name"] = name
    return mutants


def in_run_scope(item, arguments):
    """这一趟选条参数罩得住的条（--item、--from、--package；不带就是全部）：复用哨兵只从这里面抽。"""
    if arguments.item:
        return str(item["sequence"]) in arguments.item or item["name"] in arguments.item
    if arguments.start_from is not None and item["sequence"] < arguments.start_from:
        return False
    if arguments.package and item["package"] != arguments.package:
        return False
    return True
