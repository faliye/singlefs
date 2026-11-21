"""把实现员交回的补丁目录打进主工作区（入口 apply-writer-patch.py）的一块：快照撞车、副本锁、补丁目录与三方合并、走不到的理由、打过的补丁数、变异锚点静态核与临时树。
入口 apply-writer-patch.py 把这一块的名字再导出；换上与被杀、自证与命令行入口在入口文件里。
"""

import collections
import fcntl
import glob
import importlib.util
import os
import re
import secrets
import shutil
import subprocess
import sys
import tempfile
import threading
import time
from lib_rust_test_module_files import CRATE_SOURCE_PATH, INNER_TEST_ATTRIBUTE, TEST_ATTRIBUTE, attribute_end, blank_out  # noqa: E402  去注释与字面量、认测试属性（整份文件的认法也在这一份）
from lib_rust_test_module_files import whole_test_files as whole_test_files_in_tree  # noqa: E402

SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))


BROKEN = os.environ.get("APPLY_WRITER_PATCH_BREAK", "")


CHECKER_SECTION = "受影响的流与崩溃枚举用例"
UNREACHABLE_SECTION = "走不到的理由"   # result.md 里逐处列新加的走不到的断言与未定条款的错误成员的那一节的标题（文件头「走不到的理由」）
UNREACHABLE_ROW = re.compile(r"^\s*(?:[-*]\s+)?`?(crates/\S+?\.rs):(\d+)`?\s+(\S.*?)\s*$")   # 那一节的一行：- <文件>:<行> <理由>
UNREACHABLE_MINIMUM_REASON_CHARACTERS = 8
CRATE_ROLE_TABLE = ".claude/gate.d/crate-roles.tsv"   # 包的角色登记表（四列制表符分隔：包名、角色、目录、理由；规矩与判法在 scripts/gate/crate-roles-check.py 文件头）
CRATE_ROLE_TABLE_COLUMNS = 4
PRODUCT_ROLE = "产品"   # 只有这个角色的包算产品代码，走不到的理由那一核只判它（文件头「走不到的理由」）；没登记角色的包照它判
PRODUCT_SOURCE_PATH = CRATE_SOURCE_PATH   # 第一组是包目录，与角色表第三列比
UNREACHABLE_MACRO = re.compile(r"(?<![A-Za-z0-9_])(todo|unreachable|assert)!")   # debug_assert! 的 assert 前是下划线，不命中
UNREACHABLE_MACRO_LOOSE = re.compile(r"(?<![A-Za-z0-9])(todo|unreachable|assert)!")   # 弄坏开关 debug-assert-judged 下用：debug_assert! 也命中
UNDECIDED_SUFFIXES = ("Undecided", "Unsupported", "OutsideTheFirstVersion")   # 条款没定、第一版不支持的错误成员，名字以这几个词之一收尾（.claude/agents/implementation-writer.md「做什么」第 6 步）
UNDECIDED_MEMBER = re.compile(r"^(?:#\[[^\]]*\]\s*)*([A-Z][A-Za-z0-9]*(?:" + "|".join(UNDECIDED_SUFFIXES) + r"))\s*(?:[,({=]|$)")   # 枚举成员的定义行
HUNK_HEADER = re.compile(r"^@@ -\d+(?:,(\d+))? \+(\d+)(?:,(\d+))? @@")
LOG_NAME = "singlefs-applied-writer-patches.tsv"
STATIC_CHECK_SCRIPT = os.path.join(REPOSITORY_ROOT, "scripts", "mutation", "crates-mutation-rows.py")   # 变异锚点静态核（文件头「变异锚点静态核」），取这份脚本所在仓里那一份
STATIC_CHECK_TREE_PATHS = ("crates", "litmus", "Cargo.toml", "Cargo.lock", ".cargo") + (".claude/gate.d/stage-inputs.tsv",)   # 弄坏开关 static-check-path-dependencies-skipped 下临时树里只放这几样；登记表那一份是 static-check 读被判那棵树的拷贝范围（没有就退 2），不能缺
STATIC_CHECK_DECLARED = (("litmus", "变异行可以锚在 litmus/ 下的文件上，static-check 要在临时树里读到它们"), (".claude/gate.d/stage-inputs.tsv", "static-check 读被判那棵树里的这张登记表，临时树里没有就退 2"))   # 拷贝范围之外另声明的（文件头「变异锚点静态核」）
REPOSITORY_COPY_LIBRARY = os.path.join(REPOSITORY_ROOT, "scripts", "hosts", "lib_repository_copy.py")   # 拷贝范围的共用库，取这份脚本所在仓里那一份
STATIC_CHECK_ALLOWED_KINDS = ("COPIED", "LITERAL", "CHECKED", "IDENTIFIERS_CHECKED")   # 输出里不算红的行首
STATIC_CHECK_HINT_KINDS = ("REACHABILITY_CHECKED", "REACHABILITY_NOT_JUDGED", "REACHABILITY_EXEMPT", "REACHABILITY_UNJUDGED")   # 可达性那一道的提示行，不算红（文件头「变异锚点静态核」）
STATIC_CHECK_ROW_NUMBER_KINDS = ("MALFORMED", "ANCHOR", "NAME", "ROW", "STRAY", "UNDEFINED", "UNREACHABLE", "RELEASE")   # 第二段是变异表行号的几种红：比基线时去掉行号
STATIC_CHECK_TIMEOUT_SECONDS = 900   # 每趟 static-check 的限时；量的：真仓上一趟补丁前后两次 static-check 约 400–740 秒（挂钟受同机别的任务影响）、内存峰值约 15 GiB（合入后验证的变异锚点核同此），限时留余量
CRASH_REPLAY_LOGS_NAME = "singlefs-crash-point-replay-logs"   # git common-dir 里崩溃点重放日志目录的名字（每趟一个子目录）
ALL_GREEN_RUN_GLOB = os.path.join(CRASH_REPLAY_LOGS_NAME, "*", "all-green")   # 不指定范围判绿那一趟日志目录里留的那一份（不是复用的标记，只记这一趟绿过）
COPY_LOCK_NAME = ".prove-red.lock"   # 副本锁（文件头「副本锁」）：放在副本根
COPY_OWNER_NAME = ".prove-red.owner"   # 副本锁的持有者记录（文件头「副本锁」的跨机一段）：放在副本根，排他新建、持有期间按心跳刷新修改时间
COPY_OWNER_HEARTBEAT_SECONDS = 15   # 持有者刷新记录修改时间的间隔
COPY_OWNER_STALE_SECONDS = 300   # 别的机器上的持有者记录这么久没刷新就当持有者没了（推的：NFS 客户端属性缓存最长 60 秒，加两个心跳间隔留足余量）
MACHINE_IDENTITY_PATH = "/proc/sys/kernel/random/boot_id"   # 这台机器这一次开机的标识：两台、两次开机不同，不含主机名
SERVICE_TASK_VARIABLE = "TASK_SERVER_TASK"   # 统一任务服务给每次尝试设的任务号（task-server/task_server.py 文件头 ③「环境」，两台都给）：写进持有者记录
SERVICE_ATTEMPT_VARIABLE = "TASK_SERVER_ATTEMPT"   # 同上，这是第几次尝试
NOT_FROM_SERVICE = "-"   # 持有者记录里任务号、尝试号那两段：不经服务起的写这个
COPY_OWNER_FIELDS = 5   # 持有者记录一行几段：机器标识 进程号 随机串 任务号 尝试号
COPY_OWNERS = {}   # 副本锁描述符 → (持有者记录路径, 记录内容, 停心跳的事件, 心跳线程)
BASELINE_DIRECTORY = "baseline"   # 补丁目录里放基线原样的子目录（文件头「三方合并」）
RECOGNIZED_PATCH_FILES = ("crates.patch", "mutations-append.tsv", "mutations-replacements.tsv", "mutations-delete.txt")   # 补丁目录里它认得的补丁（文件头「补丁目录里的文件」）；一份都没有就拒
THREE_WAY_UNSUPPORTED_HEADERS = ("rename from ", "copy from ", "old mode ", "similarity index ", "dissimilarity index ", "GIT binary patch", "Binary files ")   # 三方合并不认的段的文件头行
THREE_WAY_LABELS = ("主工作区此刻", "补丁的基线", "补丁改后")   # git merge-file 的三份名字，次序同它的参数
SNAPSHOT_TREE_SCRIPT = os.path.join(REPOSITORY_ROOT, "scripts", "agents", "snapshot-tree.py")   # 还原 Git C 风格引号路径的 unquoted，取这份脚本所在仓里那一份
OUTCOME_NAME = "apply-outcome.tsv"   # 补丁目录里的结局文件（文件头「结局文件」）
PENDING_DIRECTORY_NAME = "singlefs-apply-writer-patch-pending"   # <git-dir> 下的暂存目录（文件头「换上与被杀」）
LANDING_LOCK_NAME = "singlefs-apply-writer-patch.lock"   # <git-dir> 下换上时拿的锁
JOURNAL_NAME = "journal.json"
ABSENT = "-"   # sha256 栏里「文件不在」的写法
STATE_LANDED, STATE_NOT_LANDED, STATE_TOUCHED = "已换上", "没换上", "被别的改动动过"
ACTION_CHANGE, ACTION_CREATE, ACTION_DELETE = "改", "增", "删"
COMMAND_NAME = "python3 scripts/gate/apply-writer-patch.py"   # 出路里写的命令


SNAPSHOT_CONFLICTS_LIBRARY = os.path.join(REPOSITORY_ROOT, ".claude", "hooks", "lib_snapshot_conflicts.py")
_SNAPSHOT_CONFLICTS = []


def snapshot_conflicts_library():
    """开工快照的判法（文件头「开工快照」）：用到时才按路径载入这份脚本所在仓的 .claude/hooks/lib_snapshot_conflicts.py（派发闸 ⑥ 调同一份）；
    prove-red.sh 按路径载入这份脚本只用副本锁，不碰它。载入不了抛 OSError / ImportError / SyntaxError，调用方拒绝、一个字不改。"""
    if not _SNAPSHOT_CONFLICTS:
        specification = importlib.util.spec_from_file_location("lib_snapshot_conflicts_for_apply_writer_patch", SNAPSHOT_CONFLICTS_LIBRARY)
        if specification is None or not os.path.isfile(SNAPSHOT_CONFLICTS_LIBRARY):
            raise ImportError(f"{SNAPSHOT_CONFLICTS_LIBRARY} 不在")
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        _SNAPSHOT_CONFLICTS.append(module)
    return _SNAPSHOT_CONFLICTS[0]


MUTATION_TABLE = "crates/mutations.tsv"
MUTATION_FILES = ("mutations-append.tsv", "mutations-replacements.tsv", "mutations-delete.txt")
MUTATION_ROW_FIELDS = 6   # crates/mutations.tsv 一行几段；mutations-append.tsv 一行、mutations-delete.txt 一行（前像）都是这么多段
REPLACEMENT_FIELDS = 2 * MUTATION_ROW_FIELDS   # mutations-replacements.tsv 一行：新行六段接写补丁时表里那一行的六段（前像，文件头「补丁目录里的文件」）
PREIMAGE_WAY_OUT = ("以被打那棵树此刻的行为基线重出这几行：前像照此刻那一行重取，新行在此刻那一行上重做这一批的改法（合补丁那一支由 merge-and-verify.sh 照快照重算），"
                    "再跑一次")   # 前像对不上与段数不对的出路（文件头「补丁目录里的文件」）


def patch_file_paths(patch_path):
    """一份补丁改到的文件（仓根相对路径，新旧两侧、含 Git 引号与重命名，lib_rebuilt_packages.py 的 patch_paths 读的）→ 去重的集合。"""
    specification = importlib.util.spec_from_file_location(
        "rebuilt_packages_for_writer_paths", os.path.join(SCRIPT_DIRECTORY, "lib_rebuilt_packages.py"))
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    with open(patch_path, encoding="utf-8", errors="surrogateescape") as handle:
        return {os.path.normpath(path) for path in module.patch_paths(handle.read())}


def touched_paths(patch_directory, include_mutation_files=True):
    """补丁改到的文件（仓根相对路径）：crates.patch 新旧两侧（含 Git 引号与重命名），三份变异表文件有一份写了行就另算 crates/mutations.tsv。"""
    touched = set()
    patch_path = os.path.join(patch_directory, "crates.patch")
    if os.path.isfile(patch_path):
        touched.update(patch_file_paths(patch_path))
    for name in MUTATION_FILES if include_mutation_files else ():
        path = os.path.join(patch_directory, name)
        if os.path.isfile(path) and any(line.strip() and not line.startswith("#") for line in open(path, encoding="utf-8", errors="replace")):
            touched.add(MUTATION_TABLE)
    return touched


SESSION_VARIABLES = ("WORK_DIRECTORY_OWNER_SESSION", "CLAUDE_CODE_SESSION_ID", "CLAUDE_SESSION_ID")   # 检出记录的会话号依次取这几个（同 scripts/hosts/work-directory-ownership.sh）


def snapshot_library_break(name):
    """交给 lib_snapshot_conflicts.py 的弄坏开关：只转 APPLY_WRITER_PATCH_BREAK=snapshot-unreadable-unmarked 这一项（文件头「弄坏开关」）。"""
    return name == "snapshot-unreadable-unmarked" and BROKEN == "snapshot-unreadable-unmarked"


def snapshot_touched_paths(patch_directory):
    """开工快照判的「补丁改到的文件」（文件头「开工快照」）：crates.patch 两侧的路径；mutations-replacements.tsv 或 mutations-delete.txt 写了行另算 crates/mutations.tsv。
    只有 mutations-append.tsv 写了行（只追加）不算改到它（与派发闸 ⑤ 同义）；弄坏开关 append-only-mutations-conflict 下三份都算。"""
    if BROKEN == "append-only-mutations-conflict":
        return touched_paths(patch_directory)
    touched = touched_paths(patch_directory, include_mutation_files=False)
    for name in MUTATION_FILES:
        path = os.path.join(patch_directory, name)
        if name != "mutations-append.tsv" and os.path.isfile(path) \
                and any(line.strip() and not line.startswith("#") for line in open(path, encoding="utf-8", errors="replace")):
            touched.add(MUTATION_TABLE)
    return touched


def snapshot_refusal_remedy(conflicts):
    """拒绝说明里「→ 怎么办」那一行：出路只有等判决写完、用户已叫停的轮登记叫停表两条（文件头「开工快照」；没有单次放行的口子）。"""
    round_names = "、".join(round_name for round_name, _, _ in conflicts)
    return (f"→ 怎么办：等 {round_names} 的判决写完再打，或把这几处改动并进判决；用户已叫停、不会写判决的轮，在 research/prompts/abandoned-rounds.tsv 登记一行收掉它"
            "（轮名、日期、为什么撂下、去向与出处，写法照 .claude/skills/three-way/SKILL.md「用户叫停、不写判决的轮」，"
            "与派发闸 ⑥、门禁 doc-process-records 的 abandoned-rounds 格同一份判法），之后这一轮的快照不再挡；没有单次放过的口子")


def snapshot_refusal(patch_directory, root):
    """开工快照（文件头「开工快照」）：补丁改到的文件落在开着的三方轮快照里，交拒绝说明的几行；没撞交 []（提醒照样交回，前缀 !）。
    读不了的快照清单照「放行但必记检出」：提醒里逐份点名，没撞时往检出记录追加一行（文件头「开工快照」末段）。
    拒绝说明的列表取共用库的 refusal_text，「→ 怎么办」那一行换成这里的两条出路（共用库那一行另带派发闸 ⑥ 的放行行，这一侧不认）。"""
    notes = []
    try:
        library = snapshot_conflicts_library()
    except (OSError, ImportError, SyntaxError) as error:
        return [f"✗ 载入不了开工快照的判法 {SNAPSHOT_CONFLICTS_LIBRARY}：{error}",
                "→ 怎么办：那份库随仓走（派发闸 ⑥ 调同一份），从 git 找回再打；查不了快照就不往主工作区打，一个字都没改"], []
    unreadable = []
    conflicts = library.conflicts_of_paths(root, snapshot_touched_paths(patch_directory), set(), notes, broken=snapshot_library_break, unreadable=unreadable)
    notes += library.unreadable_lines(root, unreadable, who="apply-writer-patch.py", this_time="打补丁")
    if not conflicts:
        session = next((os.environ[name] for name in SESSION_VARIABLES if os.environ.get(name)), None)
        recorded = library.record_unreadable(root, unreadable, f"apply-writer-patch.py {os.path.abspath(patch_directory)}", session_id=session,
                                             agent_type="主 agent（apply-writer-patch.py）")
        return [], notes + ([recorded] if recorded else [])
    lines = library.refusal_text(root, "补丁要改的文件", conflicts, notes, "再打", "", this_time="打补丁").splitlines()
    remedy = snapshot_refusal_remedy(conflicts)
    replaced = [remedy if line.startswith("→ 怎么办：") else line for line in lines]
    if remedy not in replaced:
        replaced.append(remedy)
    return replaced + ["→ 一个字都没改"], []


def hold_copy_lock(root):
    """在 <root>/.prove-red.lock 上试拿排他锁、不等：拿到交回打开的描述符（里面写持有者的进程号），被占着交回 None；建不了锁文件抛 OSError。
    拿到之后核一次路径还指着同一份文件：上一个持有者收尾时删了锁文件，拿到的就是删掉的那一份，重开再拿。"""
    path = os.path.join(root, COPY_LOCK_NAME)
    for _attempt in range(3):
        descriptor = os.open(path, os.O_RDWR | os.O_CREAT | os.O_CLOEXEC, 0o644)
        try:
            fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            os.close(descriptor)
            return None
        try:
            same_file = os.fstat(descriptor).st_ino == os.stat(path).st_ino
        except FileNotFoundError:
            same_file = False
        if same_file:
            if BROKEN != "copy-owner-ignored" and not claim_copy_owner(root, descriptor):
                os.close(descriptor)   # 别的机器上有证红占着（flock 跨机不一定互斥，文件头「副本锁」的跨机一段）：只关描述符，锁文件留给那一趟
                return None
            os.ftruncate(descriptor, 0)
            os.write(descriptor, f"{os.getpid()}\n".encode())
            return descriptor
        os.close(descriptor)
    return None


def machine_identity():
    """这台机器这一次开机的标识（MACHINE_IDENTITY_PATH）；读不到写「不明」，「不明」不与任何持有者记录算同一台。"""
    try:
        with open(MACHINE_IDENTITY_PATH, encoding="utf-8") as handle:
            return handle.read().strip() or "不明"
    except OSError:
        return "不明"


def service_attempt_of_this_run():
    """这一趟是统一任务服务的哪一次尝试：→ (任务号, 尝试号)，取 SERVICE_TASK_VARIABLE、SERVICE_ATTEMPT_VARIABLE；两样里有一样没设或不是正整数的，两样都写 NOT_FROM_SERVICE。"""
    task, attempt = os.environ.get(SERVICE_TASK_VARIABLE, ""), os.environ.get(SERVICE_ATTEMPT_VARIABLE, "")
    if task.isdigit() and attempt.isdigit() and int(task) > 0 and int(attempt) > 0:
        return task, attempt
    return NOT_FROM_SERVICE, NOT_FROM_SERVICE


def earlier_attempt_of_this_task(written):
    """持有者记录（切好的几段）是这一趟所在任务更早的一次尝试写的吗（文件头「副本锁」的同一任务更早的尝试）：这一趟经服务起、记录有 COPY_OWNER_FIELDS 段、
    任务号相同、记录的尝试号比这一趟小才写 True。服务保证第 n+1 次只在第 n 次收尾确认之后才起，所以那一次的证红已经没了。
    弄坏开关 earlier-attempt-waits 下一律 False（同一任务的旧尝试也等记录过期）。"""
    if BROKEN == "earlier-attempt-waits" or len(written) != COPY_OWNER_FIELDS:
        return False
    task, attempt = service_attempt_of_this_run()
    if task == NOT_FROM_SERVICE or not written[4].isdigit():
        return False
    return written[3] == task and int(written[4]) < int(attempt)


def copy_owner_alive_elsewhere(path):
    """持有者记录 path 指的持有者此刻还占着副本吗（flock 已经拿到了才问这一句）：记录读不了、不在写 False；
    记录里的机器标识是这一台的，flock 拿到了就说明原持有者没了，写 False；别的机器上的看记录的修改时间，COPY_OWNER_STALE_SECONDS 内刷新过的写 True，
    记录是这一趟所在任务更早的尝试写的（earlier_attempt_of_this_task）照 False，当场接手。"""
    try:
        written = open(path, encoding="utf-8").read().split()
        age = time.time() - os.stat(path).st_mtime
    except FileNotFoundError:
        return False
    except OSError:
        return True   # 记录在而读不了：不当它没人占
    identity = machine_identity()
    if written and written[0] == identity and identity != "不明":
        return False
    if age >= COPY_OWNER_STALE_SECONDS:
        return False
    return not earlier_attempt_of_this_task(written)


def keep_copy_owner_fresh(path, content, stop):
    """持有期间每 COPY_OWNER_HEARTBEAT_SECONDS 秒刷新一次持有者记录的修改时间，记录已被别人换掉就不再刷。"""
    while not stop.wait(COPY_OWNER_HEARTBEAT_SECONDS):
        try:
            if open(path, encoding="utf-8").read() == content:
                os.utime(path)
        except OSError:
            continue


def claim_copy_owner(root, descriptor):
    """在 <root>/.prove-red.owner 上排他新建持有者记录（内容：机器标识 进程号 随机串 任务号 尝试号，后两样取 service_attempt_of_this_run），
    成了交 True，登记到 COPY_OWNERS[descriptor] 并起心跳线程；
    记录已在：持有者还占着（copy_owner_alive_elsewhere）交 False；没人占了（含同一任务更早的尝试写的）就把旧记录改名摘走、删掉再建。
    摘走时核摘到的还是刚才读的那一份：不是（摘走的瞬间别人刚建了新的）就原样放回、交 False。"""
    path = os.path.join(root, COPY_OWNER_NAME)
    task, attempt = service_attempt_of_this_run()
    content = f"{machine_identity()} {os.getpid()} {secrets.token_hex(8)} {task} {attempt}\n"
    for _attempt in range(3):
        try:
            owner_descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC, 0o644)
        except FileExistsError:
            if copy_owner_alive_elsewhere(path):
                return False
            try:
                stale_content = open(path, encoding="utf-8").read()
            except FileNotFoundError:
                continue
            removed = f"{path}.removed-{secrets.token_hex(4)}"
            try:
                os.rename(path, removed)
            except FileNotFoundError:
                continue
            if open(removed, encoding="utf-8").read() != stale_content:
                os.link(removed, path)   # 摘走的是别人刚建的新记录：放回
                os.unlink(removed)
                return False
            os.unlink(removed)
            continue
        with os.fdopen(owner_descriptor, "w", encoding="utf-8") as handle:
            handle.write(content)
            handle.flush()
            os.fsync(handle.fileno())
        stop = threading.Event()
        heartbeat = threading.Thread(target=keep_copy_owner_fresh, args=(path, content, stop), daemon=True)
        heartbeat.start()
        COPY_OWNERS[descriptor] = (path, content, stop, heartbeat)
        return True
    return False


def release_copy_owner(descriptor):
    """停心跳、删自己的持有者记录（记录已被换成别人的就不删）。"""
    owner = COPY_OWNERS.pop(descriptor, None)
    if owner is None:
        return
    path, content, stop, heartbeat = owner
    stop.set()
    heartbeat.join(timeout=5)
    try:
        if open(path, encoding="utf-8").read() == content:
            os.unlink(path)
    except OSError:
        pass


def release_copy_lock(root, descriptor):
    """放掉 hold_copy_lock 拿到的锁：先删自己的持有者记录，锁文件还是自己拿着的那一份就删掉，再关描述符（关了内核就放锁）。"""
    release_copy_owner(descriptor)
    path = os.path.join(root, COPY_LOCK_NAME)
    try:
        if os.fstat(descriptor).st_ino == os.stat(path).st_ino:
            os.unlink(path)
    except FileNotFoundError:
        pass
    os.close(descriptor)


def copy_lock_holder(root):
    """持有者记录里写的进程号、机器标识前八位（别的机器上的进程号在这一台上查不了，所以带上机器标识）与任务号、尝试号；
    没有记录时取锁文件里的进程号，都读不到写「不明」，只给拒绝那一句用。"""
    try:
        written = open(os.path.join(root, COPY_OWNER_NAME), encoding="utf-8").read().split()
        if len(written) >= 2:
            started = (f"任务 {written[3]} 第 {written[4]} 次尝试" if len(written) == COPY_OWNER_FIELDS and written[3] != NOT_FROM_SERVICE
                       else "不经统一任务服务起的")
            return (f"{written[1]}（机器标识 {written[0][:8]}、{started}；是别的机器上的证红时：经服务起的同一任务后面的尝试当场接手，"
                    f"别的它被杀了的话最多等 {COPY_OWNER_STALE_SECONDS} 秒记录过期自动让出）")
    except OSError:
        pass
    try:
        return open(os.path.join(root, COPY_LOCK_NAME), encoding="utf-8").read().strip() or "不明"
    except OSError:
        return "不明"


def git(root, *arguments, check=False):
    return subprocess.run(["git", "-C", root, *arguments], capture_output=True, text=True, check=check)


def table_rows(path):
    return [line.rstrip("\n").split("\t") for line in open(path, encoding="utf-8") if line.strip() and not line.startswith("#")] if os.path.isfile(path) else []


_SNAPSHOT_TREE_MODULE = []


def snapshot_tree_module():
    """scripts/agents/snapshot-tree.py（还原 Git C 风格引号路径的 unquoted），第一次用到时才读进来。"""
    if not _SNAPSHOT_TREE_MODULE:
        specification = importlib.util.spec_from_file_location("snapshot_tree_for_apply_writer_patch", SNAPSHOT_TREE_SCRIPT)
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        _SNAPSHOT_TREE_MODULE.append(module)
    return _SNAPSHOT_TREE_MODULE[0]


PatchFile = collections.namedtuple("PatchFile", "path old_blob created deleted hunks unsupported")
ThreeWay = collections.namedtuple("ThreeWay", "problems patch lines remedy work")
THREE_WAY_CONFLICT_REMEDY = ("  → 三方合并那几处怎么办：冲突的段是别的会话与补丁改了同一处或紧挨着的两处：退给实现员在主工作区此刻的样子上重做这几段（新建副本），"
                             "或主 agent 照上面点名的段手工并好、重出补丁再打；没点名的文件合得上，不用动")
THREE_WAY_BASELINE_REMEDY = (f"  → 三方合并那几处怎么办：把补丁改的文件改前的原样放进补丁目录 {BASELINE_DIRECTORY}/<相对仓根的路径>（实现员副本还在的，在副本里 git show :<路径> 取，"
                             "blob 哈希要以补丁头 index 行左边那个号起头），再跑一次；副本已经删了的，退给实现员在主工作区此刻的样子上重建副本、重出补丁")
THREE_WAY_REBUILD_REMEDY = "  → 三方合并那几处怎么办：三方合并只认文本文件的改、增、删；退给实现员在主工作区此刻的样子上重建副本、重出补丁"


def patch_file_sections(text):
    """crates.patch 按 `diff --git` 切成每个文件一段（文件头「三方合并」）→ [PatchFile]。文件头只读一段里第一行 @@ 之前那几行（段里以 --- 起头的删除行不当文件头）；
    hunks 是那一段的 @@ 行（只留 @@ 之间那一截）；unsupported 是三方合并不认的写法（改名、复制、改权限、二进制、没有 ---/+++ 文件头、两侧路径不同、路径出了仓根），认的段是 None。"""
    unquoted = snapshot_tree_module().unquoted
    sections = []
    for line in text.splitlines():
        if line.startswith("diff --git "):
            sections.append({"header": [], "hunks": [], "in_header": True})
        elif not sections:
            continue
        elif line.startswith("@@ "):
            sections[-1]["in_header"] = False
            match = re.match(r"@@ -\S+ \+\S+ @@", line)
            sections[-1]["hunks"].append(match.group(0) if match else line)
        elif sections[-1]["in_header"]:
            sections[-1]["header"].append(line)
    files = []
    for section in sections:
        sides, old_blob, created, deleted, unsupported = {}, None, False, False, None
        for line in section["header"]:
            if line.startswith(THREE_WAY_UNSUPPORTED_HEADERS):
                unsupported = unsupported or line.strip()
            elif line.startswith("index "):
                old_blob = line.split()[1].partition("..")[0]
            elif line.startswith("new file mode"):
                created = True
            elif line.startswith("deleted file mode"):
                deleted = True
            elif line.startswith(("--- ", "+++ ")):
                value = unquoted(line[4:].split("\t", 1)[0])
                sides[line[:3]] = "" if value == "/dev/null" else (value[2:] if value.startswith(("a/", "b/")) else value)
        old_path, new_path = sides.get("---"), sides.get("+++")
        created, deleted = created or old_path == "", deleted or new_path == ""
        path = new_path or old_path or ""
        if not unsupported and (old_path is None or new_path is None):
            unsupported = "没有 ---/+++ 文件头"
        elif not unsupported and old_path and new_path and old_path != new_path:
            unsupported = f"--- 与 +++ 两侧路径不同（{old_path} → {new_path}）"
        elif not unsupported and (os.path.isabs(path) or os.path.normpath(path).startswith("..")):
            unsupported = "路径出了仓根"
        files.append(PatchFile(path, old_blob, created, deleted, section["hunks"], unsupported))
    return files


def blob_hash(root, data):
    """data 当 blob 的哈希（git hash-object，不写对象库、不走过滤）；算不出交空串。"""
    done = subprocess.run(["git", "-C", root, "hash-object", "--no-filters", "--stdin"], input=data, capture_output=True)
    return done.stdout.decode(errors="replace").strip() if done.returncode == 0 else ""


def baseline_of(root, patch_directory, entry):
    """补丁里一个文件的基线（文件头「三方合并」）→ (字节, 取自哪里) 或 (None, 取不到的原因)。只读 root 的对象库，不写。"""
    if entry.created:
        return b"", "新文件，基线是空的"
    if entry.old_blob and entry.old_blob.strip("0"):
        resolved = git(root, "rev-parse", "--verify", "--quiet", f"{entry.old_blob}^{{blob}}")
        if resolved.returncode == 0 and resolved.stdout.strip():
            blob = subprocess.run(["git", "-C", root, "cat-file", "blob", resolved.stdout.strip()], capture_output=True)
            if blob.returncode == 0:
                return blob.stdout, f"对象库 {entry.old_blob}"
    stored = os.path.join(patch_directory, BASELINE_DIRECTORY, entry.path)
    if os.path.isfile(stored):
        with open(stored, "rb") as handle:
            data = handle.read()
        digest = blob_hash(root, data)
        if BROKEN == "baseline-directory-unverified" or (entry.old_blob and digest.startswith(entry.old_blob)):
            return data, f"补丁目录 {BASELINE_DIRECTORY}/{entry.path}"
        return None, (f"{entry.path}：补丁目录 {BASELINE_DIRECTORY}/ 里那一份的 blob 哈希 {digest[:12] or '（算不出）'} "
                      f"对不上补丁头 index 行左边的 {entry.old_blob or '（没有 index 行）'}")
    if BROKEN == "baseline-missing-as-empty":
        return b"", "（弄坏开关 baseline-missing-as-empty：当空文件）"
    return None, (f"{entry.path}：补丁头 index 行左边的基线 blob {entry.old_blob or '（没有 index 行）'} 在 {root} 的对象库里没有（或这个前缀对得上不止一个），"
                  f"补丁目录里也没有 {BASELINE_DIRECTORY}/{entry.path}")


def merge_one_file(work, entry, ours, base, theirs):
    """临时目录里 git merge-file 合一个文件（文件头「三方合并」）→ (合后的字节, 冲突数)；git 报错（例：认成二进制）抛 RuntimeError。"""
    directory = os.path.join(work, "merge", entry.path.replace("/", "%"))
    os.makedirs(directory, exist_ok=True)
    paths = []
    for name, data in (("ours", ours), ("base", base), ("theirs", theirs)):
        paths.append(os.path.join(directory, name))
        with open(paths[-1], "wb") as handle:
            handle.write(data)
    labels = [argument for label in THREE_WAY_LABELS for argument in ("-L", label)]
    done = subprocess.run(["git", "merge-file", "-p", "--diff3", *labels, *paths], capture_output=True)
    if not 0 <= done.returncode <= 127:
        raise RuntimeError(f"git merge-file 合 {entry.path} 报错（退 {done.returncode}）：{done.stderr.decode(errors='replace').strip()[:300]}")
    return done.stdout, done.returncode


def zero_context_hunks(work, name, before, after):
    """before → after 不带上下文的改动段（git diff --no-index -U0 的 @@ 行）→ [(基线起始行, 基线行数, 改后起始行, 改后行数)]，行号照 diff 的写法（行数 0 时起始行是「在它之后」）。"""
    directory = os.path.join(work, "hunks", name.replace("/", "%"))
    os.makedirs(directory, exist_ok=True)
    paths = [os.path.join(directory, "before"), os.path.join(directory, "after")]
    for path, data in zip(paths, (before, after)):
        with open(path, "wb") as handle:
            handle.write(data)
    done = subprocess.run(["git", "diff", "--no-index", "--no-color", "--no-ext-diff", "-U0", *paths], capture_output=True)
    if done.returncode not in (0, 1):
        raise RuntimeError(f"git diff --no-index 比 {name} 报错（退 {done.returncode}）：{done.stderr.decode(errors='replace').strip()[:300]}")
    hunks = []
    for match in re.finditer(rb"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@", done.stdout, re.M):
        base_start, base_count, side_start, side_count = match.groups()
        hunks.append((int(base_start), 1 if base_count is None else int(base_count), int(side_start), 1 if side_count is None else int(side_count)))
    return hunks


def line_span(start, count):
    return f"{start}" if count == 1 else f"{start}–{start + count - 1}"


def base_span(hunk):
    """一段改动在基线上占的区间（0 起、右开），判两段撞没撞用：行数 0 的插入占「它之后」那个空位。"""
    start = hunk[0] if hunk[1] == 0 else hunk[0] - 1
    return start, start + hunk[1]


def describe_base_hunk(hunk):
    return f"基线第 {line_span(hunk[0], hunk[1])} 行" if hunk[1] else f"基线第 {hunk[0]} 行之后"


def colliding_hunks(ours_hunks, theirs_hunks):
    """基线上重叠或紧挨着的（git merge-file 判冲突的口径：一段的终点不在另一段起点之前）→ [(主工作区此刻那一段, 补丁那一段)]。"""
    pairs = []
    for ours in ours_hunks:
        for theirs in theirs_hunks:
            (ours_start, ours_end), (theirs_start, theirs_end) = base_span(ours), base_span(theirs)
            if not (ours_end < theirs_start or theirs_end < ours_start):
                pairs.append((ours, theirs))
    return pairs


def describe_collision(path, ours, theirs):
    ours_now = f"现在是主工作区此刻第 {line_span(ours[2], ours[3])} 行" if ours[3] else "主工作区此刻删了这几行"
    patch_now = f"改成 {theirs[3]} 行" if theirs[3] else "补丁删了这几行"
    return f"冲突 {path}：主工作区此刻改的是{describe_base_hunk(ours)}（{ours_now}），补丁改的是{describe_base_hunk(theirs)}（{patch_now}）"


def rebased_patch(work, root, results):
    """results：{路径: (主工作区此刻的字节或 None, 合后的字节或 None)} → 临时仓里照「主工作区此刻 → 合后」重出的补丁路径；合后与此刻一样交 None。
    临时仓不提交（不起提交钩子）：此刻的样子进索引、合后的写进工作区，新文件 add -N，git diff 出补丁（--full-index、不认改名、a/ b/ 前缀写死）。"""
    tree = os.path.join(work, "rebased-tree")
    subprocess.run(["git", "init", "-q", tree], capture_output=True, text=True, check=True)
    present = [path for path, (ours, _) in results.items() if ours is not None]
    for path in present:
        os.makedirs(os.path.dirname(os.path.join(tree, path)), exist_ok=True)
        with open(os.path.join(tree, path), "wb") as handle:
            handle.write(results[path][0])
    if present:
        git(tree, "add", "-f", "--", *present, check=True)
    fresh = []
    for path, (ours, merged) in results.items():
        target = os.path.join(tree, path)
        if merged is None:
            if os.path.exists(target):
                os.unlink(target)
            continue
        os.makedirs(os.path.dirname(target), exist_ok=True)
        with open(target, "wb") as handle:
            handle.write(merged)
        if ours is None:
            fresh.append(path)
    if fresh:
        git(tree, "add", "-f", "-N", "--", *fresh, check=True)
    diff = subprocess.run(["git", "-C", tree, "-c", "core.autocrlf=false", "diff", "--no-renames", "--no-color", "--no-ext-diff", "--full-index",
                           "--src-prefix=a/", "--dst-prefix=b/"], capture_output=True, check=True)
    if not diff.stdout.strip():
        return None
    path = os.path.join(work, "rebased.patch")
    with open(path, "wb") as handle:
        handle.write(diff.stdout)
    return path


def three_way_merge(patch, patch_directory, root, check_error):
    """git apply --check 不过之后的三方合并（文件头「三方合并」）→ ThreeWay(问题清单, 重出的补丁或 None, 报的几行, 出路那一行或 None, 临时目录)。
    全程只读 root；临时目录由调用方删。"""
    work = tempfile.mkdtemp(prefix="apply-writer-patch-three-way-")
    head = f"git apply --check 不过（{(check_error.strip().splitlines() or ['没有输出'])[0][:200]}）"
    try:
        with open(patch, encoding="utf-8", errors="surrogateescape") as handle:
            entries = patch_file_sections(handle.read())
        unsupported = [entry for entry in entries if entry.unsupported]
        if not entries or (unsupported and BROKEN != "three-way-kinds-unchecked"):
            named = "；".join(f"{entry.path or '（路径读不出）'}（{entry.unsupported}）" for entry in unsupported) or "补丁里没有 diff --git 段"
            return ThreeWay([f"{head}，三方合并只认文本文件的改、增、删，这几段不认：{named}"], None, [], THREE_WAY_REBUILD_REMEDY, work)
        bases, missing = {}, []
        for entry in entries:
            data, source = baseline_of(root, patch_directory, entry)
            if data is None:
                missing.append(source)
            else:
                bases[entry.path] = (data, source)
        if missing:
            return ThreeWay([f"{head}，三方合并的基线取不到：{'；'.join(missing)}"], None, [], THREE_WAY_BASELINE_REMEDY, work)
        baseline_tree = os.path.join(work, "baseline-tree")
        subprocess.run(["git", "init", "-q", baseline_tree], capture_output=True, text=True, check=True)
        for entry in entries:
            if not entry.created:
                target = os.path.join(baseline_tree, entry.path)
                os.makedirs(os.path.dirname(target), exist_ok=True)
                with open(target, "wb") as handle:
                    handle.write(bases[entry.path][0])
        applied = git(baseline_tree, "apply", os.path.abspath(patch))
        if applied.returncode != 0 and BROKEN != "baseline-patch-failure-ignored":
            return ThreeWay([f"{head}，三方合并的基线取不到：补丁头 index 行点到的基线拼成的树上原补丁套不上（{applied.stderr.strip()[:300]}）"],
                            None, [], THREE_WAY_BASELINE_REMEDY, work)
        results, conflicts, conflicted, reports = {}, [], [], []
        for entry in entries:
            base = bases[entry.path][0]
            theirs_path, ours_path = os.path.join(baseline_tree, entry.path), os.path.join(root, entry.path)
            theirs = None if entry.deleted else (open(theirs_path, "rb").read() if os.path.isfile(theirs_path) else b"")
            ours = open(ours_path, "rb").read() if os.path.isfile(ours_path) else None
            merged, count = merge_one_file(work, entry, ours or b"", base, theirs or b"")
            if count and BROKEN != "three-way-conflict-ignored":
                pairs = colliding_hunks(zero_context_hunks(work, f"{entry.path}.ours", base, ours or b""),
                                        zero_context_hunks(work, f"{entry.path}.theirs", base, theirs or b""))
                conflicted.append(entry.path)
                conflicts.append(f"冲突 {entry.path}：git merge-file 报 {count} 处冲突，撞上的段 {len(pairs)} 对（下面逐对列）")
                conflicts += [describe_collision(entry.path, ours_hunk, theirs_hunk) for ours_hunk, theirs_hunk in pairs]
                continue
            results[entry.path] = (ours, None if entry.deleted and merged == b"" else merged)
            drifted = [describe_base_hunk(hunk) + f"（现在第 {line_span(hunk[2], hunk[3])} 行）" if hunk[3] else describe_base_hunk(hunk) + "（删了）"
                       for hunk in zero_context_hunks(work, f"{entry.path}.drift", base, ours or b"")] if ours != base else []
            reports.append((entry, bases[entry.path][1], drifted))
        if conflicts:
            return ThreeWay([f"{head}，三方合并有冲突，合不上：{'、'.join(conflicted)}"] + conflicts, None, [], THREE_WAY_CONFLICT_REMEDY, work)
        rebased = rebased_patch(work, root, results)
        if rebased:
            checked = git(root, "apply", "--check", rebased)
            if checked.returncode != 0:
                return ThreeWay([f"{head}，三方合并之后重出的补丁仍然套不上（{checked.stderr.strip()[:300]}）"], None, [], THREE_WAY_REBUILD_REMEDY, work)
            with open(rebased, encoding="utf-8", errors="surrogateescape") as handle:
                landed = {entry.path: entry.hunks for entry in patch_file_sections(handle.read())}
        else:
            landed = {}
        lines = [f"三方合并：{head}，照补丁的基线、补丁改后、主工作区此刻三份合上了 {len(reports)} 个文件"
                 f"（{sum(1 for _, _, drifted in reports if drifted)} 个主工作区此刻与基线不同）{'' if rebased else '；合后与主工作区此刻一样，没有要打的'}"]
        for entry, source, drifted in reports:
            if drifted:
                lines.append(f"  合了 {entry.path}（基线取自{source}）：补丁原来的段 {'、'.join(entry.hunks) or '无'}；"
                             f"主工作区此刻比基线多改的段 {'、'.join(drifted)}；打进去的段 {'、'.join(landed.get(entry.path, [])) or '无'}")
            else:
                lines.append(f"  照原样 {entry.path}（主工作区此刻与基线相同，基线取自{source}）：补丁的段 {'、'.join(entry.hunks) or '无'}")
        return ThreeWay([], rebased, lines, None, work)
    except (OSError, RuntimeError, UnicodeError, subprocess.CalledProcessError) as error:
        return ThreeWay([f"{head}，三方合并没做成：{error}"], None, [], THREE_WAY_REBUILD_REMEDY, work)


def added_lines_of_patch(text):
    """crates.patch 里每个文件新加的行 → {路径: [(改后文件里的行号, 行文)]}；删掉的文件不收。按 @@ 头里的行数配平每一段，段里以 +++ 起头的新加行不会被当成文件头。"""
    unquoted = snapshot_tree_module().unquoted
    added, path, lines, index = {}, None, text.splitlines(), 0
    while index < len(lines):
        line = lines[index]
        index += 1
        if line.startswith("+++ "):
            value = unquoted(line[4:].split("\t", 1)[0])
            path = None if value == "/dev/null" else (value[2:] if value.startswith("b/") else value)
            continue
        header = HUNK_HEADER.match(line)
        if header is None or path is None:
            continue
        old_remaining = int(header.group(1)) if header.group(1) is not None else 1
        new_line = int(header.group(2))
        new_remaining = int(header.group(3)) if header.group(3) is not None else 1
        while (old_remaining > 0 or new_remaining > 0) and index < len(lines):
            body = lines[index]
            index += 1
            tag = body[:1]
            if tag == "\\":
                continue
            if tag == "+":
                added.setdefault(path, []).append((new_line, body[1:]))
                new_line += 1
                new_remaining -= 1
            elif tag == "-":
                old_remaining -= 1
            else:
                new_line += 1
                old_remaining -= 1
                new_remaining -= 1
    return added


def test_code_lines(cleaned):
    """去掉注释与字面量之后的源码里，落在测试代码里的行号（1 起）：#[cfg(test)]、#[test]、#[<路径>::test] 标的条目的大括号里；#![cfg(test)] 整份文件都算。"""
    if INNER_TEST_ATTRIBUTE.search(cleaned):
        return set(range(1, cleaned.count("\n") + 2))
    inside, regions = set(), []
    depth, line, nesting, awaiting = 0, 1, 0, False
    for position, character in enumerate(cleaned):
        if regions:
            inside.add(line)
        if character == "#" and cleaned.startswith("#[", position):
            if TEST_ATTRIBUTE.match(cleaned[position:attribute_end(cleaned, position)]):
                awaiting, nesting = True, 0
        elif character == "\n":
            line += 1
        elif character in "([":
            nesting += 1
        elif character in ")]":
            nesting = max(nesting - 1, 0)
        elif character == "{":
            depth += 1
            if awaiting and nesting == 0:
                regions.append(depth)
                awaiting = False
                inside.add(line)
        elif character == "}":
            if regions and regions[-1] == depth:
                regions.pop()
            depth -= 1
        elif character == ";" and awaiting and nesting == 0:
            awaiting = False
    return inside


def whole_test_files(root, patch, texts, candidates):
    """candidates（crates/<包>/src/ 下补丁新加了可疑行的 .rs）里整份算测试代码的 → 集合（文件头「走不到的理由」；认法在 scripts/gate/lib_rust_test_module_files.py 文件头）：
    父模块文件里的 mod 声明带 #[cfg(test)]、祖先文件整份算测试、或文件自己带 #![cfg(test)]；声明在被判那棵树里找（texts 里套上补丁之后的，补丁删掉的没有，没改到的读 root）。
    弄坏开关 cfg-test-module-file-judged 下一份都不认。"""
    if BROKEN == "cfg-test-module-file-judged":
        return set()
    replaced = {path: texts.get(path) for path in patch_file_paths(patch) if path.endswith(".rs")}
    return whole_test_files_in_tree(root, candidates, replaced)


def patched_texts(root, patch, paths):
    """→ {路径: 套上补丁之后的全文}：补丁前原样的文件拷进临时目录，git apply 套补丁（同 compute_images 的做法）；套不上交 None。"""
    work = tempfile.mkdtemp(prefix="apply-writer-patch-sites-")
    try:
        for path in sorted(patch_file_paths(patch)):
            source = os.path.join(root, path)
            if os.path.isfile(source) and unsafe_path_reason(path) is None:
                copy = os.path.join(work, path)
                os.makedirs(os.path.dirname(copy), exist_ok=True)
                shutil.copyfile(source, copy)
        subprocess.run(["git", "init", "-q", work], capture_output=True, text=True, check=True)
        applied = subprocess.run(["git", "-C", work, "-c", "core.autocrlf=false", "apply", os.path.abspath(patch)], capture_output=True, text=True)
        if applied.returncode != 0:
            return None
        texts = {}
        for path in paths:
            after = os.path.join(work, path)
            if os.path.isfile(after):
                with open(after, encoding="utf-8", errors="replace") as handle:
                    texts[path] = handle.read()
        return texts
    except (OSError, subprocess.CalledProcessError):
        return None
    finally:
        shutil.rmtree(work, ignore_errors=True)


def crate_role_table_text(root, patch):
    """被判那棵树（补丁打上之后的那一棵）里角色表 CRATE_ROLE_TABLE 的全文；读不到（不在、补丁删了它、补丁套不上）交 None。
    补丁改了这张表的读改后那一份（patched_texts 在临时目录里套补丁），没改的读 root 里那一份。弄坏开关 role-table-before-patch 下一律读 root 里补丁前那一份。"""
    if BROKEN != "role-table-before-patch" and CRATE_ROLE_TABLE in patch_file_paths(patch):
        return (patched_texts(root, patch, [CRATE_ROLE_TABLE]) or {}).get(CRATE_ROLE_TABLE)
    try:
        with open(os.path.join(root, CRATE_ROLE_TABLE), encoding="utf-8", errors="replace") as handle:
            return handle.read()
    except OSError:
        return None


def crate_roles_by_directory(text):
    """角色表全文 → {包目录（仓根相对，不带末尾的 /）: 角色}：空行与 # 起头的行跳过；不是 CRATE_ROLE_TABLE_COLUMNS 列的行不认，那个包照没登记算
    （行写得对不对归门禁 code-tooling 的 crate-roles 格判，这里不另判）。"""
    roles = {}
    for line in text.split("\n"):
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        fields = [field.strip() for field in line.split("\t")]
        if len(fields) == CRATE_ROLE_TABLE_COLUMNS and fields[1] and fields[2]:
            roles[fields[2].rstrip("/")] = fields[1]
    return roles


def unreachable_sites(root, patch):
    """补丁在产品代码 crates/*/src/ 里新加的、要在 result.md「走不到的理由」一节逐处列的地方 → ([(路径, 行号, 种类)], 套不上补丁时的问题或 None, 没登记角色的包目录, 读没读到角色表)：
    todo!、unreachable!、assert!（不含 debug_assert!），与条款没定、第一版不支持的错误成员（枚举成员，名字以 UNDECIDED_SUFFIXES 之一收尾）的定义行；
    测试代码里的（#[cfg(test)] 模块、#[test] 函数、#![cfg(test)] 文件、父模块文件里 #[cfg(test)] mod 声明引进来的整份文件，whole_test_files）不算；包的角色照补丁打上之后那棵树的角色表（crate_role_table_text）判，角色不是 PRODUCT_ROLE 的不算，
    没登记的照产品判、目录记进第三项。弄坏开关见文件头。"""
    with open(patch, encoding="utf-8", errors="surrogateescape") as handle:
        added = added_lines_of_patch(handle.read())
    role_table = None
    if any(PRODUCT_SOURCE_PATH.match(path) for path in added):
        role_table = crate_role_table_text(root, patch)
    if role_table is None and BROKEN == "unreadable-role-table-as-non-product":
        return [], None, set(), False
    roles = crate_roles_by_directory(role_table) if role_table is not None else {}
    candidates, unregistered = {}, {}
    for path, rows in added.items():
        matched = PRODUCT_SOURCE_PATH.match(path)
        if matched is None:
            continue
        role = roles.get(matched.group(1))
        if role is not None and role != PRODUCT_ROLE and BROKEN != "non-product-crates-judged":
            continue
        hits = [number for number, text in rows if UNREACHABLE_MACRO_LOOSE.search(text) or any(suffix in text for suffix in UNDECIDED_SUFFIXES)]
        if hits:
            candidates[path] = hits
            if role is None:
                unregistered[path] = matched.group(1)
    if not candidates:
        return [], None, set(), role_table is not None
    texts = patched_texts(root, patch, sorted(set(candidates) | {path for path in patch_file_paths(patch) if path.endswith(".rs")}))
    if texts is None:
        return [], "补丁套不上（git apply 在临时目录里失败），算不出新加的断言落在哪几行", set(), role_table is not None
    test_files = whole_test_files(root, patch, texts, sorted(candidates))
    macro = UNREACHABLE_MACRO_LOOSE if BROKEN == "debug-assert-judged" else UNREACHABLE_MACRO
    sites = []
    for path, hits in sorted(candidates.items()):
        if path not in texts or path in test_files:
            continue
        cleaned = blank_out(texts[path])
        code_lines = cleaned.splitlines()
        in_test = set() if BROKEN == "test-module-judged" else test_code_lines(cleaned)
        for number in hits:
            if number in in_test or number > len(code_lines):
                continue
            code = code_lines[number - 1]
            found = macro.search(code)
            member = UNDECIDED_MEMBER.match(code.strip()) if BROKEN != "error-member-ignored" else None
            if found:
                sites.append((path, number, f"{found.group(1)}!"))
            elif member:
                sites.append((path, number, f"错误成员 {member.group(1)}"))
    return sites, None, {unregistered[path] for path, _number, _kind in sites if path in unregistered}, role_table is not None


def unreachable_report_rows(report):
    """result.md 里标题恰好是 UNREACHABLE_SECTION 的那一节 → ({(路径, 行号): 理由}, 有没有这一节)；节到下一个标题为止。"""
    rows, inside, found = {}, False, False
    for line in report.splitlines():
        heading = re.match(r"^#{1,6}\s*(.*?)\s*#*\s*$", line)
        if heading and line.lstrip().startswith("#"):
            inside = heading.group(1) == UNREACHABLE_SECTION
            found = found or inside
        elif inside:
            row = UNREACHABLE_ROW.match(line)
            if row:
                rows[(row.group(1), int(row.group(2)))] = row.group(3)
    return rows, found


def unreachable_problems(root, patch, report):
    """补丁新加的走不到的断言、未定条款的错误成员，result.md「走不到的理由」一节每处要有一行「- <文件>:<行> <理由>」（文件头「走不到的理由」）→ 问题清单。
    弄坏开关 unreachable-section-ignored 下不核。"""
    if BROKEN == "unreachable-section-ignored":
        return []
    sites, trouble, unregistered, role_table_found = unreachable_sites(root, patch)
    if trouble:
        return [trouble]
    if not sites:
        return []
    rows, found = unreachable_report_rows(report)
    missing = [site for site in sites if (site[0], site[1]) not in rows]
    short = [site for site in sites if (site[0], site[1]) in rows and len(rows[(site[0], site[1])]) < UNREACHABLE_MINIMUM_REASON_CHARACTERS and BROKEN != "short-reason-accepted"]
    problems = []
    remedy = (f"出路：result.md 加一节「## {UNREACHABLE_SECTION}」，新加的每一处一行「- <文件>:<行> <理由>」（行号是打补丁之后那份文件里的行）；"
              "断言写哪条构造保证走不到、哪几个调用点，错误成员写哪条条款没定、为什么在任何落盘动作之前返回；理由至少 "
              f"{UNREACHABLE_MINIMUM_REASON_CHARACTERS} 个字")
    if unregistered:
        why = (f"补丁打上之后那棵树里的 {CRATE_ROLE_TABLE} 没有它们的行" if role_table_found
               else f"补丁打上之后那棵树里读不到 {CRATE_ROLE_TABLE}（不在，或补丁删了它）")
        remedy += (f"。这几个包没登记角色、照产品判（{why}）：{'、'.join(sorted(unregistered))}；"
                   f"不是产品代码的包在 {CRATE_ROLE_TABLE} 登一行角色（验证、测试支撑或实验装置，.claude/rules/directory-layout.md「crates/：各包放什么」），"
                   "登在同一份补丁里也认")
    if missing:
        listed = "；".join(f"{path}:{number} {kind}" for path, number, kind in missing)
        where = f"result.md「{UNREACHABLE_SECTION}」一节少列了 {len(missing)} 处" if found else f"result.md 里没有「{UNREACHABLE_SECTION}」一节，{len(missing)} 处都没列"
        problems.append(f"补丁在产品代码里新加了 {len(sites)} 处走不到的断言或未定条款的错误成员，{where}：{listed}。{remedy}")
    if short:
        listed = "；".join(f"{path}:{number}" for path, number, _kind in short)
        problems.append(f"result.md「{UNREACHABLE_SECTION}」一节里这几处的理由不足 {UNREACHABLE_MINIMUM_REASON_CHARACTERS} 个字：{listed}。{remedy}")
    return problems


def plan(patch_directory, root):
    """→ (问题清单, 新的变异表全文或 None, 补丁路径或 None)：只要核的结论的调用方用；三方合并的临时目录随即删掉，交回的补丁路径是补丁目录里那一份。"""
    problems, merged, _, three_way = plan_with_three_way(patch_directory, root)
    if three_way is not None:
        shutil.rmtree(three_way.work, ignore_errors=True)
    patch_path = os.path.join(patch_directory, "crates.patch")
    return problems, merged, (patch_path if os.path.isfile(patch_path) and os.path.getsize(patch_path) > 0 else None)


def unrecognized_refusal(patch_directory):
    """补丁目录里一样它认得的补丁（RECOGNIZED_PATCH_FILES）都没有时的拒绝理由（出路写在里面），有的交 None（文件头「补丁目录里的文件」）：
    目录里有别的 *.patch 的点名那几份、写明一个字没打；一样补丁都没有的写空目录、给错了目录或实现员没交补丁。
    弄坏开关 unrecognized-patch-accepted 下有别的 *.patch 的也不拒，empty-directory-accepted 下一样补丁都没有的也不拒。"""
    if any(os.path.isfile(os.path.join(patch_directory, name)) for name in RECOGNIZED_PATCH_FILES):
        return None
    recognized = "、".join(RECOGNIZED_PATCH_FILES)
    others = sorted(name for name in os.listdir(patch_directory) if name.endswith(".patch") and os.path.isfile(os.path.join(patch_directory, name)))
    if others:
        if BROKEN == "unrecognized-patch-accepted":
            return None
        return (f"补丁目录 {patch_directory} 里没有它认得的补丁（认 {recognized}），只有别的补丁文件：{'、'.join(others)}，这几份它没打；"
                "出路：改 crates 之外的文件的补丁（例：task-server、scripts）用 git apply <补丁文件> 直接打、打完 git diff 回读核过；"
                "是实现员改 crates、litmus 的补丁就改名成 crates.patch、变异表的改动拆进那三份 mutations-*（文件头「补丁目录里的文件」）")
    if BROKEN == "empty-directory-accepted":
        return None
    return (f"补丁目录 {patch_directory} 里没有它认得的补丁（认 {recognized}），也没有别的 *.patch：目录是空的、给错了目录，或交回的人没交补丁；"
            "出路：核对给的是不是交回报告里点名的补丁目录；确实没有补丁就不用打")


def mutation_edits(replaced, deleted_rows):
    """mutations-replacements.tsv 与 mutations-delete.txt 的行（table_rows 读成的段）→ (问题清单, 替换 [(新行六段, 前像六段或 None)], 删除 [(名字, 前像六段或 None)])
    （文件头「补丁目录里的文件」）。替换一行十二段：前六段是新行、后六段是前像，两半第一段的名字相同；删除一行就是前像六段。
    段数不对的行（含没带前像的：替换只有六段、删除只写名字）不逐行放过：一份文件里有一行就整份不认，一句问题数出几行、点名，出路照 PREIMAGE_WAY_OUT。
    弄坏开关 replacement-preimage-optional / deletion-preimage-optional 下没带前像的那种行照名字换、删（前像记 None，不比对）。"""
    problems, replacements, deletions = [], [], []
    for row in replaced:
        if len(row) == REPLACEMENT_FIELDS:
            if row[0] != row[MUTATION_ROW_FIELDS]:
                problems.append(f"替换的「{row[0]}」两半的名字不同（前像那一半是「{row[MUTATION_ROW_FIELDS]}」）：一行只换同名的那一行，改名写成删掉旧名的行加追加新名的行")
            replacements.append((row[:MUTATION_ROW_FIELDS], row[MUTATION_ROW_FIELDS:]))
        elif len(row) == MUTATION_ROW_FIELDS and BROKEN == "replacement-preimage-optional":
            replacements.append((row, None))
    for row in deleted_rows:
        if len(row) == MUTATION_ROW_FIELDS:
            deletions.append((row[0], row))
        elif len(row) == 1 and BROKEN == "deletion-preimage-optional":
            deletions.append((row[0], None))
    for file_name, rows, width, optional_width, switch, shape in (
            ("mutations-replacements.tsv", replaced, REPLACEMENT_FIELDS, MUTATION_ROW_FIELDS, "replacement-preimage-optional", "新行六段接写补丁时表里那一行的六段（前像）"),
            ("mutations-delete.txt", deleted_rows, MUTATION_ROW_FIELDS, 1, "deletion-preimage-optional", "写补丁时表里那一行的原样（前像，六段）")):
        wrong = [f"{row[0]}（{len(row)} 段）" for row in rows if len(row) != width and not (BROKEN == switch and len(row) == optional_width)]
        if wrong:
            problems.append(f"{file_name} 有 {len(wrong)} 行不是 {width} 段（{'、'.join(wrong[:10])}{'…' if len(wrong) > 10 else ''}）：整份不认，一行要写成{shape}；"
                            f"出路：{PREIMAGE_WAY_OUT}")
    return problems, replacements, deletions


def tabs_shown(text):
    """拒绝说明里写一行变异行的原文：段与段之间的制表符写成 ⇥，两侧逐段对得出来。"""
    return text.replace("\t", " ⇥ ")


def preimage_problems(lines, names, replacements, deletions, root):
    """替换与删除的前像逐行比对被打那棵树此刻表里同名的那一行（文件头「补丁目录里的文件」）：→ 问题清单，每行一句写行名与两侧原文，末尾一句出路；全对得上交空。
    表里不是恰好一行的名字不在这里判（调用方已报）。弄坏开关：replacement-preimage-unchecked / deletion-preimage-unchecked 不比对替换 / 删除；
    replacement-preimage-against-new-row 拿替换的新行去比；deletion-preimage-newline-kept 删除那一侧拿带行尾换行符的原行去比。"""
    current = {}
    for line in lines:
        if line.strip() and not line.startswith("#"):
            current.setdefault(line.split("\t", 1)[0], line)
    checks = [("替换", new_row[0], "\t".join(new_row if BROKEN == "replacement-preimage-against-new-row" else preimage), False)
              for new_row, preimage in replacements if preimage is not None and BROKEN != "replacement-preimage-unchecked"]
    checks += [("删除", name, "\t".join(preimage), BROKEN == "deletion-preimage-newline-kept")
               for name, preimage in deletions if preimage is not None and BROKEN != "deletion-preimage-unchecked"]
    mismatched = []
    for kind, name, expected, newline_kept in checks:
        if names.count(name) != 1:
            continue
        now = current[name] if newline_kept else current[name].rstrip("\n")
        if now != expected:
            mismatched.append(f"{kind}的「{name}」前像与被打那棵树此刻表里那一行对不上：写补丁时「{tabs_shown(expected)}」，此刻「{tabs_shown(now.rstrip(chr(10)))}」")
    if not mismatched:
        return []
    return mismatched + [f"{len(mismatched)} 行变异行的前像对不上 {root} 此刻的 {MUTATION_TABLE}（上面逐行列了行名与两侧原文；写补丁之后别的改动动过这几行，照名字换、删会把那些改动盖掉），"
                         f"整份不打；出路：{PREIMAGE_WAY_OUT}"]


def plan_with_three_way(patch_directory, root):
    """→ (问题清单, 新的变异表全文或 None, 要打的补丁路径或 None, ThreeWay 或 None)。一样认得的补丁都没有的先拒（unrecognized_refusal）；
    git apply --check 不过时做三方合并（文件头「三方合并」），合上了要打的是重出的那一份（在 ThreeWay.work 里，调用方用完删）。"""
    refusal = unrecognized_refusal(patch_directory)
    if refusal is not None:
        return [refusal], None, None, None
    problems = []
    three_way = None
    patch_path = os.path.join(patch_directory, "crates.patch")
    patch = patch_path if os.path.isfile(patch_path) and os.path.getsize(patch_path) > 0 else None
    if patch:
        touched = touched_paths(patch_directory, include_mutation_files=False)
        if "crates/mutations.tsv" in touched:
            problems.append("crates.patch 里动了 crates/mutations.tsv：变异表的改动写进那三个文件，按名字合并")
        report_path = os.path.join(patch_directory, "result.md")
        report = open(report_path, encoding="utf-8", errors="replace").read() if os.path.isfile(report_path) else ""
        if any(path.startswith("crates/singlefs-checker/src/") for path in touched) and CHECKER_SECTION not in report and BROKEN != "checker-section-ignored":
            problems.append(f"补丁动了 crates/singlefs-checker/src/，result.md 里却没有「{CHECKER_SECTION}」一节")
        checked = git(root, "apply", "--check", patch)
        applies = checked.returncode == 0
        if checked.returncode != 0 and BROKEN == "three-way-skipped":
            problems.append(f"git apply --check 不过：{checked.stderr.strip()[:300]}")
        elif checked.returncode != 0 and BROKEN != "apply-unchecked":
            three_way = three_way_merge(patch, patch_directory, root, checked.stderr)
            problems += three_way.problems
            patch = three_way.patch if not three_way.problems else patch
            applies = not three_way.problems
        # 三方合并合后与主工作区此刻一样时 patch 是 None（文件头「三方合并」）：没有要打的、没有新加的行，不核走不到的理由；
        # 弄坏开关 unchanged-merge-judged 下照核，None 交给 unreachable_sites 读
        if applies and (patch is not None or BROKEN == "unchanged-merge-judged"):
            problems += unreachable_problems(root, patch, report)
    table_path = os.path.join(root, "crates", "mutations.tsv")
    if not os.path.isfile(table_path):
        return problems + ["主工作区没有 crates/mutations.tsv"], None, patch, three_way
    with open(table_path, encoding="utf-8") as handle:
        lines = handle.read().splitlines(keepends=True)
    names = [line.split("\t", 1)[0] for line in lines if line.strip() and not line.startswith("#")]
    appended = table_rows(os.path.join(patch_directory, "mutations-append.tsv"))
    replaced = table_rows(os.path.join(patch_directory, "mutations-replacements.tsv"))
    deletion_path = os.path.join(patch_directory, "mutations-delete.txt")
    edit_problems, replacements, deletions = mutation_edits(replaced, table_rows(deletion_path))
    problems += edit_problems
    deleted = [name for name, _preimage in deletions]
    for row in appended:
        if len(row) != MUTATION_ROW_FIELDS:
            problems.append(f"追加的「{row[0]}」不是六段（{len(row)} 段）")
    for row in appended:
        if names.count(row[0]) != 0 and BROKEN != "append-duplicates":
            problems.append(f"追加的「{row[0]}」表里已经有了")
    appended_names = [row[0] for row in appended]
    if BROKEN != "append-internal-duplicates":
        for name in dict.fromkeys(appended_names):
            if appended_names.count(name) > 1:
                problems.append(f"mutations-append.tsv 自己里「{name}」出现了 {appended_names.count(name)} 行：追加的变异名在追加表里也不许重名（改名，或合成一行）")
    for name in [new_row[0] for new_row, _preimage in replacements] + deleted:
        if names.count(name) != 1:
            problems.append(f"「{name}」在表里有 {names.count(name)} 行，要恰好 1 行")
    problems += preimage_problems(lines, names, replacements, deletions, root)
    if problems:
        return problems, None, patch, three_way
    replacement_by_name = {new_row[0]: "\t".join(new_row) + "\n" for new_row, _preimage in replacements}
    merged = []
    for line in lines:
        name = line.split("\t", 1)[0] if line.strip() and not line.startswith("#") else None
        if name in deleted:
            continue
        merged.append(replacement_by_name.get(name, line))
    if merged and not merged[-1].endswith("\n"):
        merged[-1] += "\n"
    merged += ["\t".join(row) + "\n" for row in appended]
    return [], "".join(merged), patch, three_way


def patches_since_last_crash_verification(root, log_path):
    common = git(root, "rev-parse", "--git-common-dir").stdout.strip()
    common = common if os.path.isabs(common) else os.path.join(root, common)
    green_runs = [] if BROKEN == "all-green-ignored" else glob.glob(os.path.join(common, ALL_GREEN_RUN_GLOB))
    since = max((os.path.getmtime(path) for path in green_runs), default=0.0)
    rows = [line.split("\t") for line in open(log_path, encoding="utf-8")] if os.path.isfile(log_path) else []
    return sum(1 for row in rows if row and float(row[0]) > since)


def static_check_command(tree):
    """变异锚点静态核的命令（文件头「变异锚点静态核」）：这份脚本所在仓里那一份 crates-mutation-rows.py static-check <树>。"""
    return [sys.executable or "python3", STATIC_CHECK_SCRIPT, "static-check", tree]


def run_static_check(command):
    """→ (退出码或 None, 输出的行)。起不了或超时交 None 和一行原因。"""
    try:
        done = subprocess.run(command, capture_output=True, text=True, timeout=STATIC_CHECK_TIMEOUT_SECONDS, env=copy_environment())
    except (OSError, subprocess.TimeoutExpired) as error:
        return None, [f"起不了或超过 {STATIC_CHECK_TIMEOUT_SECONDS} 秒：{error}"]
    return done.returncode, done.stdout.splitlines() + [f"stderr：{line}" for line in done.stderr.splitlines()[-5:]]


def static_check_red(lines):
    """输出里的红行 → [(比对用的键, 原行)]；键去掉变异表行号（追加删除会让行号漂）。
    可达性那一道的提示行（STATIC_CHECK_HINT_KINDS）不算红；弄坏开关 static-check-hints-red 下算红。
    弄坏开关 static-check-unreachable-row-number-kept 下 UNREACHABLE 连行号比（补丁删一行，补丁前就有的那一行跟着漂、被当成补丁带来的）；
    static-check-release-row-number-kept 下 RELEASE 同样连行号比。"""
    red = []
    not_red = STATIC_CHECK_ALLOWED_KINDS + (() if BROKEN == "static-check-hints-red" else STATIC_CHECK_HINT_KINDS)
    row_number_kinds = tuple(kind for kind in STATIC_CHECK_ROW_NUMBER_KINDS
                             if not (kind == "UNREACHABLE" and BROKEN == "static-check-unreachable-row-number-kept")
                             and not (kind == "RELEASE" and BROKEN == "static-check-release-row-number-kept"))
    for line in lines:
        fields = line.split("\t")
        if not line.strip() or fields[0] in not_red or not fields[0].isupper():
            continue
        key = tuple(fields[:1] + fields[2:]) if fields[0] in row_number_kinds and len(fields) > 2 else tuple(fields)
        red.append((key, line))
    return red


_REPOSITORY_COPY_MODULE = []


def repository_copy_module():
    """scripts/hosts/lib_repository_copy.py（拷贝范围的 copy_closure、copy_entries 与副本里的环境），第一次用到时才读进来。"""
    if not _REPOSITORY_COPY_MODULE:
        specification = importlib.util.spec_from_file_location("lib_repository_copy_for_apply_writer_patch", REPOSITORY_COPY_LIBRARY)
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        _REPOSITORY_COPY_MODULE.append(module)
    return _REPOSITORY_COPY_MODULE[0]


def static_check_tree_items(root):
    """临时树里放的几处（文件头「变异锚点静态核」）：root 带测试代码扫的拷贝范围（lib_repository_copy.py 的 copy_closure）再声明 litmus/。
    闭包算不成抛 ValueError（ClosureRefused 是它的子类）或 OSError。弄坏开关 static-check-path-dependencies-skipped：只放 STATIC_CHECK_TREE_PATHS。"""
    if BROKEN == "static-check-path-dependencies-skipped":
        return list(STATIC_CHECK_TREE_PATHS)
    return list(repository_copy_module().copy_closure(os.path.realpath(root), STATIC_CHECK_DECLARED, test_inputs=True).entries)


def copy_environment():
    """临时树上起 git apply 与 static-check 用的环境（lib_repository_copy.py 的 environment_for_commands_in_the_copy，多机配置取这份脚本所在的主工作区那一份）。"""
    return repository_copy_module().environment_for_commands_in_the_copy(REPOSITORY_ROOT)


def static_check_copied_items(root, patch):
    """两棵静态核树里放的几处（文件头「变异锚点静态核」）：static_check_tree_items 那几处，再加补丁改到、此刻在 root 里的每份文件（拷贝范围之外的也算：
    拒算表、角色表这一类跟着补丁走的登记表，补丁套在树上要读到它们的原样；补丁新建的不在 root 里、不拷）。
    弄坏开关 static-check-outside-files-not-copied：只放 static_check_tree_items 那几处，补丁改拷贝范围之外的文件时在树上套不上。"""
    items = static_check_tree_items(root)
    if not patch or BROKEN == "static-check-outside-files-not-copied":
        return items
    extra = sorted(path for path in patch_file_paths(patch)
                   if unsafe_path_reason(path) is None and os.path.lexists(os.path.join(root, path))
                   and not any(path == item or path.startswith(item + "/") for item in items))
    return items + extra


def build_static_check_trees(root, merged, patch):
    """补丁之前、补丁之后两棵静态核用的树，从 root 同一刻拷出（文件头「变异锚点静态核」）：先把 static_check_copied_items 那几处拷成补丁之前那一棵，
    再把它整棵拷成补丁之后那一棵、写进合并后的变异表、套上补丁。→ (补丁之前的树根, 补丁之后的树根)；套不上抛 RuntimeError，出错时已建的临时目录自己删，
    成功的两棵由调用方删（root 本身不删）。弄坏开关 static-check-baseline-on-live-root：补丁之前那一棵交 root 本身（两趟 static-check 之间主工作区被别的会话改了，基线跟着变）。"""
    before = tempfile.mkdtemp(prefix="apply-writer-patch-static-check-before-")
    after = None
    try:
        repository_copy_module().copy_entries(os.path.realpath(root), before, static_check_copied_items(root, patch))
        os.makedirs(os.path.join(before, "crates"), exist_ok=True)
        after = tempfile.mkdtemp(prefix="apply-writer-patch-static-check-after-")
        shutil.copytree(before, after, symlinks=True, dirs_exist_ok=True)
        with open(os.path.join(after, "crates", "mutations.tsv"), "w", encoding="utf-8") as handle:
            handle.write(merged)
        if patch:
            environment = copy_environment()
            subprocess.run(["git", "init", "-q", after], capture_output=True, text=True, check=True, env=environment)   # 临时目录自己是一个仓的根，git apply 不受外层仓的路径限制
            applied = subprocess.run(["git", "-C", after, "apply", os.path.abspath(patch)], capture_output=True, text=True, env=environment)
            if applied.returncode != 0:
                raise RuntimeError(f"补丁在临时树上套不上：{applied.stderr.strip()[:300]}")
    except BaseException:
        for tree in (before, after):
            if tree is not None:
                shutil.rmtree(tree, ignore_errors=True)
        raise
    if BROKEN == "static-check-baseline-on-live-root":
        shutil.rmtree(before, ignore_errors=True)
        return root, after
    return before, after


def static_check_report(root, merged, patch, static_check=None, shown=20):
    """--dry-run 的变异锚点静态核（文件头「变异锚点静态核」）：→ (拒绝说明的几行，提醒的几行)。拒绝说明为空才算过。
    新红的行最多列 shown 处、其余写一行「…还有 N 处」；shown 给 None 的（merge-and-verify.sh 的变异锚点核，红行全写进它的日志）一处不省。"""
    if BROKEN == "static-check-skipped":
        return [], []
    make_command = static_check or static_check_command
    howto = ("→ 怎么办：一个字都没改。补丁（含 mutations-replacements.tsv）弄断了变异表的锚点或带进了别的静态问题：照每处改——代码改写之后原文找不到的，"
             "在 mutations-replacements.tsv 里按新代码重取原文与替换文、意思照原行；改好再跑一次 --dry-run")
    before_tree = after_tree = None
    try:
        before_tree, after_tree = build_static_check_trees(root, merged, patch)
        # 两趟先后跑、不并行：每趟对整个工作区做可达性判约 16 GiB，并行时峰值 30.90 GiB，挡住两台排着的别的任务；两棵树同一刻拷出，跑着的几百秒里主工作区被改不影响判
        after_code, after_lines = run_static_check(make_command(after_tree))
        before_code, before_lines = run_static_check(make_command(before_tree))
    except (OSError, RuntimeError, ValueError, ImportError, subprocess.CalledProcessError) as error:   # ValueError：拷贝范围的闭包算不成（ClosureRefused）
        return [f"✗ 变异锚点静态核没做成：{error}", howto], []
    finally:
        for tree in (before_tree, after_tree):
            if tree is not None and os.path.realpath(tree) != os.path.realpath(root):
                shutil.rmtree(tree, ignore_errors=True)
    for which, code, lines in (("套上补丁的临时树", after_code, after_lines), ("没打补丁的树", before_code, before_lines)):
        if code not in (0, 1) or any(line.startswith("ERROR") for line in lines):
            return ([f"✗ 变异锚点静态核判不了：{which}上 crates-mutation-rows.py static-check 退 {code}"] + [f"  {line}" for line in lines[:6]]
                    + ["→ 怎么办：一个字都没改。照上面几行修好 static-check 读的登记表与变异表，或手跑 "
                       f"python3 {STATIC_CHECK_SCRIPT} static-check <树> 看原因，再跑一次"]), []
    checked = [line for line in after_lines if line.split("\t")[0] == "CHECKED"]
    if not checked:
        return [f"✗ 变异锚点静态核判不了：套上补丁的临时树上 static-check 没打 CHECKED 行（退 {after_code}）", howto], []
    after_red, before_red = static_check_red(after_lines), static_check_red(before_lines)
    inherited = collections.Counter(key for key, _ in before_red) if BROKEN != "static-check-baseline-ignored" else collections.Counter()
    fresh, still = [], 0
    for key, line in after_red:
        if inherited[key] > 0:
            inherited[key] -= 1
            still += 1
        else:
            fresh.append(line)
    if fresh:
        listed = fresh if shown is None else fresh[:shown]
        return ([f"✗ 补丁打上之后变异表有 {len(fresh)} 处静态核红（crates-mutation-rows.py static-check，补丁前的树上没有）："]
                + [f"  {line.replace(chr(9), ' ')}" for line in listed] + ([f"  …还有 {len(fresh) - len(listed)} 处"] if len(listed) < len(fresh) else []) + [howto]), []
    notes = [f"变异锚点静态核（static-check）：{checked[0].replace(chr(9), ' ')}"]
    if still:
        notes.append(f"! 补丁前就红的 {still} 处（不是这个补丁带来的，不拦；提交时门禁 code-source-discipline 的 mutation-tables 格照样判）")
    return [], notes


def unsafe_path_reason(path):
    parts = path.split(os.sep)
    if path in ("", ".") or os.path.isabs(path) or ".." in parts:
        return "路径出了仓根"
    if parts[0] == ".git":
        return "路径在 .git 里"
    return None
