#!/usr/bin/env python3
# admission: always 每个实现员补丁交回都要打一次，判的是这一刻的补丁目录与主工作区
# run-condition: command git
"""把实现员交回的补丁目录打进主工作区：补丁、变异表按名字合并、跑 code-source-discipline 与轻型变异登记表的核法，记一笔「自上次崩溃验证全绿以来打了几个补丁」。

用法：
    apply-writer-patch.py <补丁目录> [--root 仓根] [--dry-run] [--snapshot-ruled '快照冲突已判：<轮名> <理由>']…
    apply-writer-patch.py --selftest          # APPLY_WRITER_PATCH_BREAK=<项> 时必须判红

补丁目录里的文件（格式由 .claude/agents/implementation-writer.md「产出」一节定）：
    crates.patch                 `git diff -- crates litmus ':!crates/mutations.tsv'` 的输出；没有代码改动可以没有
    mutations-append.tsv         要追加的变异行，每行六段（与 crates/mutations.tsv 同），名字在表里不许已有、在这个文件里也不许出现两次
    mutations-replacements.tsv   要整行换掉的变异行，每行六段，按第一段的名字找表里恰好一行换掉
    mutations-delete.txt         要删的变异名，一行一个，表里要恰好一行
    result.md                    实现员的报告；补丁动了 crates/singlefs-checker/src/ 的，报告里要有「受影响的层 0 流与崩溃枚举用例」一节
次序：先全部核一遍（补丁 git apply --check、名字在不在、行是不是六段、checker 那一节在不在），有一处不对就一个字不改、退 2；
都对了才 git apply、改名换上新的变异表，跑 .claude/gate.d/code-source-discipline.sh 的 mutation-tables 那一格（--check mutation-tables），
再跑轻型变异登记表的核法（这份脚本所在仓里的 scripts/mutation/crates-mutation-light-rows.py check <被打的仓根>），把这一次记进 <git common-dir>/singlefs-applied-writer-patches.tsv。
轻型登记（.claude/kb/milestone/03-third-txn.md 记的同一个漏法第二次出现：新测试目标进了核心变异行而轻型登记表没跟上）：只报不拦，
  补丁照样打上；核红时与 code-source-discipline 那一行同形报一行「crates-mutation-light-rows：退出码 N，<汇总>」，下面逐行抄它列的每一处
  （缺行的那一处带照挑法该补的原文「<变异名><制表符><测试目标>」），退出码记 1。
自上次崩溃验证全绿（门禁 checker-tier-crash-point-replay --full 判绿那一趟在它的日志目录里留的 all-green：git common-dir 里
singlefs-crash-point-replay-logs/*/all-green 最新的那一份；用例一级没有全绿标记了）以来打过的补丁到 CRASH_VERIFY_HINT_PATCHES 个（5，推的）就打一行提示：
只提示、不拦，跑不跑由用户定。
副本锁（主 agent 2026-10-10 的失误：scripts/mutation/prove-red.sh --copy 正在副本里就地施加变异时，往同一份副本打了补丁）：
  --root 指的不是这份脚本所在的主工作区时，开头在 <副本根>/.prove-red.lock 上试拿排他锁（flock，不等），拿到了打完再放、拿不到就拒绝、退 2、一个字没改，
  出路写「副本上有证红在跑，等它跑完或换一份副本」。prove-red.sh 跑的时候拿着同一把锁（它按路径载入这里的 hold_copy_lock / release_copy_lock，锁的写法只在这一处）。
  锁随持有的进程放：正常收尾时删掉锁文件；被杀时内核放掉锁、文件留着，下一个照样拿得到。
主工作区合入锁（.claude/kb/milestone/03-third-txn.md「待优化：流程、门禁与装置上踩过的坑」第 20 行）：
  --root 是这份脚本所在的主工作区（不给 --root 就是）时，核与打之前先拿 scripts/gate/lib_workspace_lock.py 的主工作区合入锁（持有者是这个进程），
  被别的合入占着就排队、每过一段时间报一行谁占着，持有者已死的判失效、报出来再接管；打完（成功、失败都）放掉。判法与锁文件在那份库的文件头，这里不另写。
  锁按工作树分开：这份脚本在 git worktree 加出来的树里时拿那棵树自己那一把，不排在主工作树的合入后面。
  merge-and-verify.sh 持着锁时起的这一份（SINGLEFS_WORKSPACE_LOCK_HOLDER 指着它的祖先）不另拿。拿不成（建不了锁文件、
  SINGLEFS_WORKSPACE_LOCK_GIVE_UP_SECONDS 设了而等到点）退 2、一个字没改。副本上不拿这一把（副本锁管副本）。
开工快照（主 agent 2026-10-15 的失误：一份改 checker 档 9 份文件的补丁打进主工作区，比别的会话开的代码轮 m3-b44-code-r1 取快照晚 3 分钟，改到了快照里的文件）：
  打进主工作区（与主工作区合入锁同一个判法：--root 是这份脚本所在的仓，或自证点名的临时仓）之前，补丁改到的文件——crates.patch 里 --- a/ 与 +++ b/ 两侧的路径，
  三份变异表文件有一份写了行就另算 crates/mutations.tsv——落在还没判完的三方轮的开工快照里就拒、退 2、一个字没改。哪几轮开着、快照清单怎么读、
  放行行认不认、拒绝说明怎么写只在 .claude/hooks/lib_snapshot_conflicts.py（派发闸 ⑥ 调同一份）；出路同派发闸 ⑥：等判决写完再打，
  或命令行 --snapshot-ruled '快照冲突已判：<轮名> <理由>'（可给几次，理由不许空）显式放过点名的那一轮，或用户已叫停的轮在 research/prompts/abandoned-rounds.tsv 登记一行收掉。
  副本上不查（副本不是那一轮取快照的工作区）。
  快照清单读不了（是目录、没有读权限、悬空的链接）的那一份不判，照「放行但必记检出」照打：输出里与别的提醒一样以 ! 起头，逐份点名那一轮与那份清单、写出路
  （lib_snapshot_conflicts.py 的 unreadable_lines），检出记录（AGENT_HOOK_DETECTIONS，没设取 .claude/hooks/lib_work_root.py 的 default_detections_path）
  追加一行、结局只记，会话号取 WORK_DIRECTORY_OWNER_SESSION、CLAUDE_CODE_SESSION_ID、CLAUDE_SESSION_ID 里先设了的那个，看门狗据此报给主 agent；退出码照打的结果，不因它变。
退出码：0 打上了（code-source-discipline 与轻型登记核法都绿）；1 打上了但 code-source-discipline 或轻型登记核法红；2 核的时候就不对、补丁改到开着的三方轮开工快照里的文件、副本上有证红在跑、
  或主工作区合入锁没拿成，一个字没改。

弄坏开关（只给 --selftest 证红用）。逐项写开关名、弄坏哪一处、打开后自证哪几项红（项名是 selftest() 里那一份补丁目录的名字，括号里是那条「✗ 自检：」的开头）：
  APPLY_WRITER_PATCH_BREAK=append-duplicates  plan() 不核追加的变异名在表里有没有
      duplicate（「✗ 自检：追加已有的名字应当拒绝」）
  APPLY_WRITER_PATCH_BREAK=append-internal-duplicates  plan() 不核追加表自己里有没有重名
      selfduplicate（「✗ 自检：追加表里两行同名」）
  APPLY_WRITER_PATCH_BREAK=apply-unchecked  plan() 里 git apply --check 不过也照样往下走
      stale（「✗ 自检：打不上的补丁应当在核的时候拒绝」：到真打时才失败，报的不是 apply --check）
  APPLY_WRITER_PATCH_BREAK=checker-section-ignored  plan() 不看动了 checker 的补丁报告里有没有「受影响的层 0 流与崩溃枚举用例」一节
      checker（「✗ 自检：动 checker 而报告没那一节应当拒绝」）
  APPLY_WRITER_PATCH_BREAK=no-hint  apply() 打过的补丁数到 CRASH_VERIFY_HINT_PATCHES 也不打提示
      onlytable（「✗ 自检：打过的补丁数到提示线应当打提示」）
  APPLY_WRITER_PATCH_BREAK=copy-lock-ignored  apply() 不试拿副本锁，副本上有证红在跑也照打
      locked（「✗ 自检：副本锁被占着时应当拒绝、退 2、一个字不改」）
  APPLY_WRITER_PATCH_BREAK=ignore-workspace-lock  apply() 打进主工作区时不认主工作区合入锁，别的合入拿着也照打
      queued（「✗ 自检：主工作区合入锁被别的合入占着时应当排队、报出持有者、对方放锁后才打」）、
      deadholder（「✗ 自检：主工作区合入锁的持有者已死时应当报出来、接管、照打」）
  scripts/gate/lib_workspace_lock.py 的 WORKSPACE_LOCK_BREAK=lock-in-common-dir  锁文件照改前放 git common-dir（所有工作树共用一把）
      pertree（「✗ 自检：主工作树的锁被占着时往 git worktree 加出来的树打补丁应当不排队、当场打上」）
  APPLY_WRITER_PATCH_BREAK=snapshot-ignored  apply() 打进主工作区之前不查开着的三方轮开工快照（改前的样子）
      snapshot（「✗ 自检：补丁改到开着的三方轮开工快照里的文件应当拒绝、退 2、一个字不改」）；snapshot-ruled 照绿（放行那一格分不出差别）
      连带红的格：「✗ 自检：快照清单读不了时应当照打、点名那一轮与那几份清单」
  APPLY_WRITER_PATCH_BREAK=snapshot-unreadable-unmarked  转给 lib_snapshot_conflicts.py 的同名开关：读不了的快照清单不分出来、照旧只记进提醒，不打放行但必记检出那几行、不记检出
      snapshot-unreadable（「✗ 自检：快照清单读不了时应当照打、点名那一轮与那几份清单、写明放行但必记检出、检出记录多一行」）
  APPLY_WRITER_PATCH_BREAK=light-rows-unchecked  apply() 打完不跑轻型变异登记表的核法（改前的样子）
      lightrows（「✗ 自检：打完之后轻型登记表缺核心测试目标的行应当报红、列出该补的原文、退 1、补丁照打」）；lightrows-green 照绿
      连带红的格：「✗ 自检：轻型登记表跟上了之后应当报绿、退 0」
"""
import fcntl
import glob
import hashlib
import importlib.util
import os
import subprocess
import sys
import tempfile
import time

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
from preflight import preflight  # noqa: E402
BROKEN = os.environ.get("APPLY_WRITER_PATCH_BREAK", "")
CRASH_VERIFY_HINT_PATCHES = int(os.environ.get("CRASH_VERIFY_HINT_PATCHES", "5"))   # 推的，没量过
CHECKER_SECTION = "受影响的层 0 流与崩溃枚举用例"
LOG_NAME = "singlefs-applied-writer-patches.tsv"
LIGHT_ROWS_CHECK = os.path.join(REPOSITORY_ROOT, "scripts", "mutation", "crates-mutation-light-rows.py")   # 轻型登记的核法（文件头「轻型登记」），取这份脚本所在仓里那一份
FULL_GREEN_RUN_GLOB = os.path.join("singlefs-crash-point-replay-logs", "*", "all-green")   # --full 判绿那一趟日志目录里留的那一份（不是复用的标记，只记这一趟绿过）
COPY_LOCK_NAME = ".prove-red.lock"   # 副本锁（文件头「副本锁」）：放在副本根


def load_workspace_lock():
    """主工作区合入锁（文件头「主工作区合入锁」）：按路径载入同目录的 lib_workspace_lock.py（prove-red.sh 按路径载入这一份时 sys.path 里不一定有本目录）。"""
    specification = importlib.util.spec_from_file_location("lib_workspace_lock_for_apply_writer_patch", os.path.join(SCRIPT_DIRECTORY, "lib_workspace_lock.py"))
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


WORKSPACE_LOCK = load_workspace_lock()


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


def touched_paths(patch_directory, include_mutation_files=True):
    """补丁改到的文件（仓根相对路径）：crates.patch 新旧两侧（含 Git 引号与重命名），三份变异表文件有一份写了行就另算 crates/mutations.tsv。"""
    touched = set()
    patch_path = os.path.join(patch_directory, "crates.patch")
    if os.path.isfile(patch_path):
        specification = importlib.util.spec_from_file_location(
            "rebuilt_packages_for_writer_paths", os.path.join(SCRIPT_DIRECTORY, "lib_rebuilt_packages.py"))
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        with open(patch_path, encoding="utf-8", errors="surrogateescape") as handle:
            touched.update(os.path.normpath(path) for path in module.patch_paths(handle.read()))
    for name in MUTATION_FILES if include_mutation_files else ():
        path = os.path.join(patch_directory, name)
        if os.path.isfile(path) and any(line.strip() and not line.startswith("#") for line in open(path, encoding="utf-8", errors="replace")):
            touched.add(MUTATION_TABLE)
    return touched


SESSION_VARIABLES = ("WORK_DIRECTORY_OWNER_SESSION", "CLAUDE_CODE_SESSION_ID", "CLAUDE_SESSION_ID")   # 检出记录的会话号依次取这几个（同 scripts/hosts/work-directory-ownership.sh）


def snapshot_library_break(name):
    """交给 lib_snapshot_conflicts.py 的弄坏开关：只转 APPLY_WRITER_PATCH_BREAK=snapshot-unreadable-unmarked 这一项（文件头「弄坏开关」）。"""
    return name == "snapshot-unreadable-unmarked" and BROKEN == "snapshot-unreadable-unmarked"


def snapshot_refusal(patch_directory, root, ruled_lines):
    """开工快照（文件头「开工快照」）：补丁改到的文件落在开着的三方轮快照里，交拒绝说明的几行；没撞交 []（提醒照样交回，前缀 !）。
    读不了的快照清单照「放行但必记检出」：提醒里逐份点名，没撞时往检出记录追加一行（文件头「开工快照」末段）。"""
    notes = []
    try:
        library = snapshot_conflicts_library()
    except (OSError, ImportError, SyntaxError) as error:
        return [f"✗ 载入不了开工快照的判法 {SNAPSHOT_CONFLICTS_LIBRARY}：{error}",
                "→ 怎么办：那份库随仓走（派发闸 ⑥ 调同一份），从 git 找回再打；查不了快照就不往主工作区打，一个字都没改"], []
    ruled = library.ruled_snapshot_conflicts("\n".join(ruled_lines))
    unreadable = []
    conflicts = library.conflicts_of_paths(root, touched_paths(patch_directory), ruled, notes, broken=snapshot_library_break, unreadable=unreadable)
    notes += library.unreadable_lines(root, unreadable, who="apply-writer-patch.py", this_time="打补丁")
    if not conflicts:
        session = next((os.environ[name] for name in SESSION_VARIABLES if os.environ.get(name)), None)
        recorded = library.record_unreadable(root, unreadable, f"apply-writer-patch.py {os.path.abspath(patch_directory)}", session_id=session,
                                             agent_type="主 agent（apply-writer-patch.py）")
        return [], notes + ([recorded] if recorded else [])
    text = library.refusal_text(root, "补丁要改的文件", conflicts, notes, "再打", "命令行加 --snapshot-ruled 写", this_time="打补丁")
    return text.splitlines() + ["→ 一个字都没改"], []


def print_now(line):
    print(line, flush=True)


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
            os.ftruncate(descriptor, 0)
            os.write(descriptor, f"{os.getpid()}\n".encode())
            return descriptor
        os.close(descriptor)
    return None


def release_copy_lock(root, descriptor):
    """放掉 hold_copy_lock 拿到的锁：锁文件还是自己拿着的那一份就先删掉，再关描述符（关了内核就放锁）。"""
    path = os.path.join(root, COPY_LOCK_NAME)
    try:
        if os.fstat(descriptor).st_ino == os.stat(path).st_ino:
            os.unlink(path)
    except FileNotFoundError:
        pass
    os.close(descriptor)


def copy_lock_holder(root):
    """锁文件里写的持有者进程号（读不到写「不明」），只给拒绝那一句用。"""
    try:
        return open(os.path.join(root, COPY_LOCK_NAME), encoding="utf-8").read().strip() or "不明"
    except OSError:
        return "不明"


def git(root, *arguments, check=False):
    return subprocess.run(["git", "-C", root, *arguments], capture_output=True, text=True, check=check)


def table_rows(path):
    return [line.rstrip("\n").split("\t") for line in open(path, encoding="utf-8") if line.strip() and not line.startswith("#")] if os.path.isfile(path) else []


def plan(patch_directory, root):
    """→ (问题清单, 新的变异表全文或 None, 补丁路径或 None)。"""
    problems = []
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
        if checked.returncode != 0 and BROKEN != "apply-unchecked":
            problems.append(f"git apply --check 不过：{checked.stderr.strip()[:300]}")
    table_path = os.path.join(root, "crates", "mutations.tsv")
    if not os.path.isfile(table_path):
        return problems + ["主工作区没有 crates/mutations.tsv"], None, patch
    with open(table_path, encoding="utf-8") as handle:
        lines = handle.read().splitlines(keepends=True)
    names = [line.split("\t", 1)[0] for line in lines if line.strip() and not line.startswith("#")]
    appended = table_rows(os.path.join(patch_directory, "mutations-append.tsv"))
    replaced = table_rows(os.path.join(patch_directory, "mutations-replacements.tsv"))
    deletion_path = os.path.join(patch_directory, "mutations-delete.txt")
    deleted = [line.strip() for line in open(deletion_path, encoding="utf-8")] if os.path.isfile(deletion_path) else []
    deleted = [name for name in deleted if name and not name.startswith("#")]
    for kind, rows in (("追加", appended), ("替换", replaced)):
        for row in rows:
            if len(row) != 6:
                problems.append(f"{kind}的「{row[0]}」不是六段（{len(row)} 段）")
    for row in appended:
        if names.count(row[0]) != 0 and BROKEN != "append-duplicates":
            problems.append(f"追加的「{row[0]}」表里已经有了")
    appended_names = [row[0] for row in appended]
    if BROKEN != "append-internal-duplicates":
        for name in dict.fromkeys(appended_names):
            if appended_names.count(name) > 1:
                problems.append(f"mutations-append.tsv 自己里「{name}」出现了 {appended_names.count(name)} 行：追加的变异名在追加表里也不许重名（改名，或合成一行）")
    for name in [row[0] for row in replaced] + deleted:
        if names.count(name) != 1:
            problems.append(f"「{name}」在表里有 {names.count(name)} 行，要恰好 1 行")
    if problems:
        return problems, None, patch
    replacement_by_name = {row[0]: "\t".join(row) + "\n" for row in replaced}
    merged = []
    for line in lines:
        name = line.split("\t", 1)[0] if line.strip() and not line.startswith("#") else None
        if name in deleted:
            continue
        merged.append(replacement_by_name.get(name, line))
    if merged and not merged[-1].endswith("\n"):
        merged[-1] += "\n"
    merged += ["\t".join(row) + "\n" for row in appended]
    return [], "".join(merged), patch


def patches_since_last_crash_verification(root, log_path):
    common = git(root, "rev-parse", "--git-common-dir").stdout.strip()
    common = common if os.path.isabs(common) else os.path.join(root, common)
    green_runs = glob.glob(os.path.join(common, FULL_GREEN_RUN_GLOB))
    since = max((os.path.getmtime(path) for path in green_runs), default=0.0)
    rows = [line.split("\t") for line in open(log_path, encoding="utf-8")] if os.path.isfile(log_path) else []
    return sum(1 for row in rows if row and float(row[0]) > since)


def light_rows_command(root):
    """轻型变异登记表的核法（文件头「轻型登记」）：这份脚本所在仓里那一份 crates-mutation-light-rows.py check <被打的仓根>。"""
    return [sys.executable or "python3", LIGHT_ROWS_CHECK, "check", root]


def apply(patch_directory, root, dry_run, gate33, main_workspace=None, say=print_now, snapshot_ruled=(), light_rows=None):
    """副本上先试拿副本锁（文件头「副本锁」），拿到了再核、再打，打完放锁；主工作区拿主工作区合入锁（文件头「主工作区合入锁」，排队等），
    核之前先查开着的三方轮开工快照（文件头「开工快照」；snapshot_ruled 是命令行 --snapshot-ruled 给的那几句）。
    main_workspace 不给就按 root 是不是这份脚本所在的仓判（自证给 True，把临时仓当主工作区）。
    light_rows 是打完之后跑的轻型登记核法命令（文件头「轻型登记」），不给就是 light_rows_command(root)。"""
    if light_rows is None:
        light_rows = light_rows_command(root)
    if main_workspace is None:
        main_workspace = os.path.realpath(root) == os.path.realpath(REPOSITORY_ROOT)
    if main_workspace:
        held = None
        if BROKEN != "ignore-workspace-lock":
            try:
                held = WORKSPACE_LOCK.acquire(root, os.getpid(), f"apply-writer-patch.py {os.path.abspath(patch_directory)}", say=say)
            except WORKSPACE_LOCK.LockError as error:
                return 2, [f"✗ 主工作区合入锁没拿成：{error.problem}", f"→ 怎么办：{error.howto}；一个字都没改"]
        try:
            refusal, notes = ([], []) if BROKEN == "snapshot-ignored" else snapshot_refusal(patch_directory, root, snapshot_ruled)
            if refusal:
                return 2, refusal
            code, lines = apply_holding_the_lock(patch_directory, root, dry_run, gate33, light_rows)
            return code, lines + notes
        finally:
            problem = WORKSPACE_LOCK.release(held)
            if problem:
                say(f"! 放主工作区合入锁：{problem}")
    lock = None
    if BROKEN != "copy-lock-ignored":
        try:
            lock = hold_copy_lock(root)
        except OSError as error:
            return 2, [f"✗ 在 {root} 上建不了副本锁 {COPY_LOCK_NAME}：{error}", "→ 怎么办：看 --root 指的目录在不在、可不可写；一个字都没改"]
        if lock is None:
            return 2, [f"✗ 副本 {root} 上有证红在跑（{COPY_LOCK_NAME} 被进程 {copy_lock_holder(root)} 占着），这一次不打",
                       "→ 怎么办：副本上有证红在跑，等它跑完或换一份副本；一个字都没改"]
    try:
        return apply_holding_the_lock(patch_directory, root, dry_run, gate33, light_rows)
    finally:
        if lock is not None:
            release_copy_lock(root, lock)


def apply_holding_the_lock(patch_directory, root, dry_run, gate33, light_rows):
    problems, merged, patch = plan(patch_directory, root)
    if problems:
        return 2, [f"✗ {problem}" for problem in problems] + ["→ 怎么办：一个字都没改。照每处改补丁目录（或退给实现员），再跑一次"]
    if dry_run:
        return 0, [f"✓ 核过了（--dry-run，没改）：补丁 {'有' if patch else '没有'}，变异表合并之后 {merged.count(chr(10))} 行"]
    if patch:
        applied = git(root, "apply", patch)
        if applied.returncode != 0:
            return 2, [f"✗ git apply 失败：{applied.stderr.strip()[:300]}", "→ 怎么办：工作区在核与打之间被改了；git status 看一眼再跑"]
    table_path = os.path.join(root, "crates", "mutations.tsv")
    temporary = f"{table_path}.{os.getpid()}.tmp"
    with open(temporary, "x", encoding="utf-8") as handle:
        handle.write(merged)
    os.replace(temporary, table_path)
    lines = [f"✓ 打上了：{os.path.basename(os.path.normpath(patch_directory))}（补丁 {'有' if patch else '没有'}）"]
    gate = subprocess.run(gate33, cwd=root, capture_output=True, text=True)
    gate_line = ([line for line in gate.stdout.splitlines() if line.strip()] or ["（没有输出）"])[-1]
    lines.append(f"code-source-discipline：退出码 {gate.returncode}，{gate_line.strip()}")
    light_rows_code = 0
    if BROKEN != "light-rows-unchecked":
        light_rows_code, light_rows_lines = light_rows_report(root, light_rows)
        lines += light_rows_lines
    common = git(root, "rev-parse", "--git-common-dir").stdout.strip()
    log_path = os.path.join(common if os.path.isabs(common) else os.path.join(root, common), LOG_NAME)
    digest = hashlib.sha256(open(patch, "rb").read()).hexdigest() if patch else "-"
    with open(log_path, "a", encoding="utf-8") as handle:
        handle.write(f"{time.time()}\t{os.path.abspath(patch_directory)}\t{digest}\n")
    count = patches_since_last_crash_verification(root, log_path)
    if count >= CRASH_VERIFY_HINT_PATCHES and BROKEN != "no-hint":
        lines.append(f"! 自上次崩溃验证全绿以来已打 {count} 个实现员补丁（提示线 {CRASH_VERIFY_HINT_PATCHES}，推的）：弹窗问用户要不要用 "
                     "SINGLEFS_HEAVY_TESTS=user-request 由主 agent 跑一次（checker-tier-crash-point-replay 按用例复用，只跑输入变了的崩溃枚举用例）；只提示，不拦")
    lines.append(f"调度表可贴的一行：| {os.path.basename(os.path.normpath(patch_directory))} | 已打（{time.strftime('%m-%d %H:%M', time.localtime())}） | "
                 f"code-source-discipline {gate.returncode}，轻型登记 {light_rows_code} |")
    return (0 if gate.returncode == 0 and light_rows_code == 0 else 1), lines


def light_rows_report(root, command):
    """打完之后跑轻型登记的核法（文件头「轻型登记」）：→ (退出码, 报的几行)。只报不拦：红时汇总那一行之外逐行抄它列的每一处（缺行的带该补的原文）。
    起不了的（脚本不在、解释器起不来）按红报，退出码记 1，出路写在那一行里。"""
    try:
        checked = subprocess.run(command, cwd=root, capture_output=True, text=True)
    except OSError as error:
        return 1, [f"crates-mutation-light-rows：起不了核法 {' '.join(command)}：{error}",
                   f"  → 怎么办：补丁已打上；轻型登记没核，看 {LIGHT_ROWS_CHECK} 在不在，手跑 python3 {LIGHT_ROWS_CHECK} check {root} 补核"]
    output = [line for line in checked.stdout.splitlines() if line.strip()]
    summary = (output or [(checked.stderr.strip().splitlines() or ["（没有输出）"])[-1]])[-1].strip()
    if checked.returncode == 0:
        return 0, [f"crates-mutation-light-rows：退出码 0，{summary}"]
    details = [line.strip() for line in output if line.strip().startswith("✗")]
    summary_line = next((line for line in reversed(details) if "处不对" in line), summary)
    howto = next((line.strip() for line in output if line.strip().startswith("→")), "")
    return 1, ([f"crates-mutation-light-rows：退出码 {checked.returncode}，{summary_line}（补丁已打上，只报不拦）"]
               + [f"  {line}" for line in details if line != summary_line]
               + ([f"  {howto}"] if howto else [f"  → 怎么办：照上面几处改 .claude/gate.d/crates-mutation-light-rows.tsv，再跑 python3 {LIGHT_ROWS_CHECK} check {root} 核"]))


def selftest():
    work = tempfile.mkdtemp(prefix="apply-writer-patch-selftest-")
    # 主工作区合入锁那四格：锁文件用临时仓自己的 git 目录，调用方设的锁位置、持有者变量与等待上限不带进来；排队时每 0.3 秒报一次、每 0.05 秒看一次
    for name in (WORKSPACE_LOCK.LOCK_FILE_VARIABLE, WORKSPACE_LOCK.HOLDER_VARIABLE, WORKSPACE_LOCK.GIVE_UP_SECONDS_VARIABLE):
        os.environ.pop(name, None)
    os.environ[WORKSPACE_LOCK.REPORT_SECONDS_VARIABLE] = "0.3"
    os.environ[WORKSPACE_LOCK.POLL_SECONDS_VARIABLE] = "0.05"
    # 排队最多等 10 秒（queued 那一格对方只拿 1.5 秒）：弄坏开关让锁永远拿不到时到点退 2、记红，不挂住
    os.environ[WORKSPACE_LOCK.GIVE_UP_SECONDS_VARIABLE] = "10"
    try:
        root = os.path.join(work, "repo")
        os.makedirs(os.path.join(root, "crates", "demo", "src"))
        os.makedirs(os.path.join(root, "crates", "singlefs-checker", "src"))
        open(os.path.join(root, "crates", "demo", "src", "lib.rs"), "w").write("fn a() -> u32 { 1 }\n")
        open(os.path.join(root, "crates", "singlefs-checker", "src", "lib.rs"), "w").write("fn check() {}\n")
        open(os.path.join(root, "crates", "mutations.tsv"), "w").write("# 表头\nkeep\tf\to\tr\t-p demo --lib\tt\nold\tf\to\tr\t-p demo --lib\tt\ngone\tf\to\tr\t-p demo --lib\tt\n")
        # 轻型变异登记表（文件头「轻型登记」）：变异表里没有改坏对象在核心代码下的行，只有表头的登记表核得过；lightrows 那两格再往里加核心行
        light_table = os.path.join(root, ".claude", "gate.d", "crates-mutation-light-rows.tsv")
        os.makedirs(os.path.dirname(light_table))
        open(light_table, "w").write("# 样本轻型登记表\n")
        for command in (["init", "-q"], ["add", "-A"], ["-c", "user.name=s", "-c", "user.email=s@s", "commit", "-qm", "base"]):
            git(root, *command, check=True)
        def patch_directory(name, source_change=None, append="", replace="", delete="", report=""):
            directory = os.path.join(work, name)
            os.makedirs(directory)
            if source_change:
                relative, new_text = source_change
                path = os.path.join(root, relative)
                original = open(path).read()
                open(path, "w").write(new_text)
                open(os.path.join(directory, "crates.patch"), "w").write(git(root, "diff", "--", relative).stdout)
                open(path, "w").write(original)
            for file_name, text in (("mutations-append.tsv", append), ("mutations-replacements.tsv", replace), ("mutations-delete.txt", delete), ("result.md", report)):
                if text:
                    open(os.path.join(directory, file_name), "w").write(text)
            return directory
        gate33 = ["true"]
        failures = []
        good = patch_directory("good", ("crates/demo/src/lib.rs", "fn a() -> u32 { 2 }\n"), append="new\tf\to\tr\t-p demo --lib\tt\n",
                               replace="old\tf\to2\tr2\t-p demo --lib\tt\n", delete="gone\n")
        code, lines = apply(good, root, False, gate33)
        table = open(os.path.join(root, "crates", "mutations.tsv")).read()
        if code != 0 or "fn a() -> u32 { 2 }" not in open(os.path.join(root, "crates", "demo", "src", "lib.rs")).read() \
                or table != "# 表头\nkeep\tf\to\tr\t-p demo --lib\tt\nold\tf\to2\tr2\t-p demo --lib\tt\nnew\tf\to\tr\t-p demo --lib\tt\n":
            failures.append(f"好的补丁应当打上、按名字合表，实际 {code} {lines} {table!r}")
        checker = patch_directory("checker", ("crates/singlefs-checker/src/lib.rs", "fn check() { assert!(true); }\n"), report="只改了判定\n")
        code, lines = apply(checker, root, False, gate33)
        if code != 2 or not any("受影响的层 0 流" in line for line in lines):
            failures.append(f"动 checker 而报告没那一节应当拒绝，实际 {code} {lines}")
        duplicate = patch_directory("duplicate", append="keep\tf\to\tr\t-p demo --lib\tt\n")
        code, lines = apply(duplicate, root, False, gate33)
        if code != 2:
            failures.append(f"追加已有的名字应当拒绝，实际 {code} {lines}")
        selfduplicate = patch_directory("selfduplicate", append="twin\tf\to\tr\t-p demo --lib\tt\ntwin\tf\to2\tr2\t-p demo --lib\tt\n")
        code, lines = apply(selfduplicate, root, False, gate33)
        if code != 2 or not any("twin" in line and "mutations-append.tsv" in line for line in lines):
            failures.append(f"追加表里两行同名（都不在表里）应当拒绝并点名重名的变异名，实际 {code} {lines}")
        stale = patch_directory("stale", ("crates/demo/src/lib.rs", "fn a() -> u32 { 9 }\n"))
        open(os.path.join(root, "crates", "demo", "src", "lib.rs"), "w").write("fn a() -> u32 { 3 }\n")
        code, lines = apply(stale, root, False, gate33)
        if code != 2 or not any("apply --check" in line for line in lines):
            failures.append(f"打不上的补丁应当在核的时候拒绝，实际 {code} {lines}")
        os.environ["CRASH_VERIFY_HINT_PATCHES"] = "1"
        global CRASH_VERIFY_HINT_PATCHES
        CRASH_VERIFY_HINT_PATCHES = 1
        only_table = patch_directory("onlytable", append="another\tf\to\tr\t-p demo --lib\tt\n")
        code, lines = apply(only_table, root, False, gate33)
        if code != 0 or not any("崩溃验证全绿以来已打" in line for line in lines):
            failures.append(f"打过的补丁数到提示线应当打提示，实际 {code} {lines}")
        # 副本锁（文件头「副本锁」）：这一份当成证红在跑、拿着锁，apply 要拒绝、一个字不改；放了锁再打就照打，打完锁文件不留
        locked = patch_directory("locked", append="locked-row\tf\to\tr\t-p demo --lib\tt\n")
        table_path = os.path.join(root, "crates", "mutations.tsv")
        before = open(table_path).read()
        held = hold_copy_lock(root)
        try:
            code, lines = apply(locked, root, False, gate33)
            after_refusal = open(table_path).read()
        finally:
            if held is not None:
                release_copy_lock(root, held)
        if held is None or code != 2 or not any("副本上有证红在跑" in line for line in lines) or after_refusal != before:
            failures.append(f"副本锁被占着时应当拒绝、退 2、一个字不改，实际 {code} {lines}（锁{'拿到了' if held is not None else '没拿到'}）")
        if after_refusal != before:
            # 上一格在弄坏开关（copy-lock-ignored）下照打了：表退回去，这一格只判自己的，不因为「locked-row 表里已经有了」连带红
            with open(table_path, "w") as handle:
                handle.write(before)
        code, lines = apply(locked, root, False, gate33)
        if code != 0 or "locked-row\t" not in open(table_path).read() or os.path.exists(os.path.join(root, COPY_LOCK_NAME)):
            failures.append(f"副本锁没被占着时应当照打、打完不留锁文件，实际 {code} {lines}")
        # 主工作区合入锁（文件头「主工作区合入锁」）：临时仓当主工作区（main_workspace=True），锁文件在它自己的 git 目录（.git）里
        lock_library = os.path.join(SCRIPT_DIRECTORY, "lib_workspace_lock.py")
        timeline = os.path.join(work, "workspace-lock-timeline.txt")
        # queued：另一个假的合入（bash 进程，持有者是它自己）先拿着锁 1.5 秒；这里排队、报出它、它放了才打
        queued = patch_directory("queued", append="queued-row\tf\to\tr\t-p demo --lib\tt\n")
        holder = subprocess.Popen(["bash", "-c", 'python3 "$0" acquire --root "$1" --holder-pid $$ --command "假合入 A" > /dev/null || exit 1; echo in >> "$2"; sleep 1.5; '
                                   'echo "out $(date +%s%N)" >> "$2"; python3 "$0" release --root "$1" --holder-pid $$ > /dev/null', lock_library, root, timeline])
        deadline = time.time() + 10
        while time.time() < deadline and not (os.path.exists(timeline) and "in" in open(timeline).read()):
            time.sleep(0.05)
        said = []
        code, lines = apply(queued, root, False, gate33, main_workspace=True, say=said.append)
        try:
            holder_code = holder.wait(timeout=15)   # 持锁的假合入自己拿锁最多等 10 秒（自证开头设的放弃时限），到点还没退就停掉它（只停自己起的这一个进程号）、判红
        except subprocess.TimeoutExpired:
            holder.kill()
            holder.wait()
            holder_code = "超时"
        applied_at = os.stat(table_path).st_mtime_ns
        released_at = [int(line.split()[1]) for line in open(timeline) if line.startswith("out ")] if os.path.exists(timeline) else []
        if code != 0 or holder_code != 0 or not released_at or applied_at <= released_at[0] or "queued-row\t" not in open(table_path).read() \
                or not any("在排队" in line and f"进程 {holder.pid}" in line for line in said):
            failures.append(f"主工作区合入锁被别的合入占着时应当排队、报出持有者、对方放锁后才打，实际退 {code}、持锁的假合入退 {holder_code}、"
                            f"表写于 {applied_at}、对方放锁于 {released_at}，报的 {said}")
        # deadholder：锁文件里记着一个已经退出的进程：报「已死」、接管、照打，打完记录清空
        deadholder = patch_directory("deadholder", append="dead-row\tf\to\tr\t-p demo --lib\tt\n")
        dead = subprocess.Popen(["true"])
        dead.wait()
        lock_file = WORKSPACE_LOCK.lock_path(root)
        with open(lock_file, "w") as handle:
            handle.write(f"pid={dead.pid}\nstart=1\nsession=死掉的会话\ncommand=假合入 dead\nsince={time.time() - 30}\n")
        said = []
        code, lines = apply(deadholder, root, False, gate33, main_workspace=True, say=said.append)
        if code != 0 or "dead-row\t" not in open(table_path).read() or not any("已死" in line and "接管" in line and "死掉的会话" in line for line in said) \
                or WORKSPACE_LOCK.parse_record(open(lock_file).read()) is not None:
            failures.append(f"主工作区合入锁的持有者已死时应当报出来、接管、照打、打完清空记录，实际退 {code}，报的 {said}，锁文件 {open(lock_file).read()!r}")
        # reentry：父进程（当成持锁的 merge-and-verify.sh）拿着锁、SINGLEFS_WORKSPACE_LOCK_HOLDER 指着它，这个进程是它底下起的 apply：不排队、照打、不动记录
        # 锁文件先清空：上一格在弄坏开关下留着的死记录不带进这一格（红只落在点名的那几格）；父进程要锁也有放弃时限（环境里的 10 秒），没拿成这一格判红、不挂住
        reentry = patch_directory("reentry", append="reentry-row\tf\to\tr\t-p demo --lib\tt\n")
        with open(lock_file, "w") as handle:
            handle.write("")
        said = []
        try:
            held = WORKSPACE_LOCK.acquire(root, os.getppid(), "假 merge-and-verify", say=lambda line: None)
        except WORKSPACE_LOCK.LockError as error:
            held, code, lines, still_held = None, 2, [f"父进程自己要锁就没拿成：{error.problem}"], None
        if held is not None:
            os.environ[WORKSPACE_LOCK.HOLDER_VARIABLE] = str(os.getppid())
            try:
                code, lines = apply(reentry, root, False, gate33, main_workspace=True, say=said.append)
                still_held = WORKSPACE_LOCK.parse_record(open(lock_file).read())
            finally:
                os.environ.pop(WORKSPACE_LOCK.HOLDER_VARIABLE, None)
                WORKSPACE_LOCK.release(held)
        if code != 0 or "reentry-row\t" not in open(table_path).read() or not still_held or still_held.get("pid") != str(os.getppid()):
            failures.append(f"merge-and-verify 持锁时起的 apply 应当不排队、照打、不动记录，实际退 {code} {lines}，报的 {said}，锁记录 {still_held}")
        # pertree：每棵工作树各一把（lib_workspace_lock.py 文件头「锁文件」）。临时仓（主工作树）的锁被一个活着的进程（自证起的 sleep）占着，
        # 往它 git worktree 加出来的树打补丁不排队、当场打上；放弃时限 2 秒（弄坏开关 WORKSPACE_LOCK_BREAK=lock-in-common-dir 下排到点退 2、记红，不挂住）
        linked_root = os.path.join(work, "linked")
        git(root, "worktree", "add", "-q", linked_root, check=True)
        per_tree = patch_directory("pertree", append="worktree-row\tf\to\tr\t-p demo --lib\tt\n")
        main_holder = subprocess.Popen(["sleep", "30"])
        try:
            with open(lock_file, "w") as handle:
                handle.write(f"pid={main_holder.pid}\nstart={WORKSPACE_LOCK.process_start(main_holder.pid)}\nsession=主工作树的合入\ncommand=假合入 main\nsince={time.time()}\n")
            os.environ[WORKSPACE_LOCK.GIVE_UP_SECONDS_VARIABLE] = "2"
            said = []
            code, lines = apply(per_tree, linked_root, False, gate33, main_workspace=True, say=said.append)
        finally:
            os.environ[WORKSPACE_LOCK.GIVE_UP_SECONDS_VARIABLE] = "10"
            main_holder.kill()
            main_holder.wait()
            with open(lock_file, "w") as handle:
                handle.write("")
        linked_table = os.path.join(linked_root, "crates", "mutations.tsv")
        if code != 0 or "worktree-row\t" not in open(linked_table).read() or any("在排队" in line for line in said):
            failures.append(f"主工作树的锁被占着时往 git worktree 加出来的树打补丁应当不排队、当场打上，实际退 {code} {lines}，报的 {said}")
        # 开工快照（文件头「开工快照」）：临时仓里开一轮 r-open（快照目录、正文在，判决不在），清单列着 crates/demo/src/lib.rs 与 crates/mutations.tsv；
        # 当主工作区打（main_workspace=True）。snapshot：改 lib.rs 的补丁与只追加变异行的补丁都拒、退 2、一个字不改；
        # snapshot-ruled：同一份补丁带 --snapshot-ruled「快照冲突已判：r-open <理由>」照打
        prompts = os.path.join(root, "research", "prompts")
        os.makedirs(os.path.join(prompts, "r-open-snapshot"))
        open(os.path.join(prompts, "_r-open-body.md"), "w").write("# 样本三方轮的正文\n")
        open(os.path.join(prompts, "r-open-snapshot", "crates-sha256.txt"), "w").write(
            f"{'a' * 64}  ./crates/demo/src/lib.rs\n{'b' * 64}  tree/crates/mutations.tsv\n")
        for command in (["add", "-A"], ["-c", "user.name=s", "-c", "user.email=s@s", "commit", "-qm", "上面几格打过的样子"]):
            git(root, *command, check=True)   # 补丁照此刻的工作区造（stale 那一格把工作区改得与索引不同）
        source_before, table_before = open(os.path.join(root, "crates", "demo", "src", "lib.rs")).read(), open(table_path).read()
        snapshot_code = patch_directory("snapshot", ("crates/demo/src/lib.rs", "fn a() -> u32 { 7 }\n"))
        snapshot_table = patch_directory("snapshot-table", append="snapshot-row\tf\to\tr\t-p demo --lib\tt\n")
        code, lines = apply(snapshot_code, root, False, gate33, main_workspace=True, say=lambda line: None)
        table_code, table_lines = apply(snapshot_table, root, False, gate33, main_workspace=True, say=lambda line: None)
        source_after, table_after = open(os.path.join(root, "crates", "demo", "src", "lib.rs")).read(), open(table_path).read()
        if code != 2 or table_code != 2 or source_after != source_before or table_after != table_before \
                or not any("r-open 的开工快照里" in line for line in lines) or not any("crates/demo/src/lib.rs（出现在补丁" in line for line in lines) \
                or not any("crates/mutations.tsv（出现在补丁" in line for line in table_lines) or not any("--snapshot-ruled" in line for line in lines):
            failures.append(f"补丁改到开着的三方轮开工快照里的文件应当拒绝、退 2、一个字不改，实际改代码那一份退 {code} {lines}，只追加变异行那一份退 {table_code} {table_lines}")
        if source_after != source_before or table_after != table_before:
            # 上一格在弄坏开关（snapshot-ignored）下照打了：两份退回去，下一格只判自己的
            open(os.path.join(root, "crates", "demo", "src", "lib.rs"), "w").write(source_before)
            open(table_path, "w").write(table_before)
        code, lines = apply(snapshot_code, root, False, gate33, main_workspace=True, say=lambda line: None,
                            snapshot_ruled=("快照冲突已判：r-open 只改样本函数的返回值，判过不碰那一轮的证据",))
        if code != 0 or "fn a() -> u32 { 7 }" not in open(os.path.join(root, "crates", "demo", "src", "lib.rs")).read():
            failures.append(f"补丁改到开着的三方轮快照里的文件、命令行给了「快照冲突已判：r-open <理由>」应当照打，实际退 {code} {lines}")
        # snapshot-unreadable：再开两轮，r-folder 的清单是目录、r-locked 的清单没有读权限（列的是补丁不碰的文件）；补丁只追加变异行（crates/mutations.tsv 在 r-open
        # 的清单里，命令行放过 r-open）。照打、退 0，输出逐份点名那一轮与那份清单、写明放行但必记检出，检出记录多一行（结局只记）
        import json
        for round_name in ("r-folder", "r-locked"):
            open(os.path.join(prompts, f"_{round_name}-body.md"), "w").write(f"# {round_name} 正文\n")
        os.makedirs(os.path.join(prompts, "r-folder-snapshot", "crates-sha256.txt"))
        os.makedirs(os.path.join(prompts, "r-locked-snapshot"))
        locked_list = os.path.join(prompts, "r-locked-snapshot", "crates-sha256.txt")
        open(locked_list, "w").write(f"{'c' * 64}  crates/demo/src/untouched.rs\n")
        os.chmod(locked_list, 0)
        unreadable_rounds = ("r-folder",) if os.access(locked_list, os.R_OK) else ("r-folder", "r-locked")   # 以 root 跑时去掉读权限也读得了
        detections = os.path.join(work, "detections.jsonl")
        saved_detections = os.environ.get("AGENT_HOOK_DETECTIONS")
        os.environ["AGENT_HOOK_DETECTIONS"] = detections
        try:
            unreadable_patch = patch_directory("snapshot-unreadable", append="unreadable-row\tf\to\tr\t-p demo --lib\tt\n")
            code, lines = apply(unreadable_patch, root, False, gate33, main_workspace=True, say=lambda line: None,
                                snapshot_ruled=("快照冲突已判：r-open 只追加样本变异行，判过不碰那一轮的证据",))
        finally:
            if saved_detections is None:
                os.environ.pop("AGENT_HOOK_DETECTIONS", None)
            else:
                os.environ["AGENT_HOOK_DETECTIONS"] = saved_detections
        recorded = []
        if os.path.exists(detections):
            for line in open(detections, encoding="utf-8"):
                try:
                    recorded.append(json.loads(line))
                except ValueError:
                    recorded.append({"坏行": line})
        named = all(any(round_name in line and "放行但必记检出" in line for line in lines) for round_name in unreadable_rounds)
        if code != 0 or "unreadable-row\t" not in open(table_path).read() or not named or len(recorded) != 1 or recorded[0].get("outcome") != "只记" \
                or len(recorded[0].get("findings") or []) != len(unreadable_rounds):
            failures.append(f"快照清单读不了时应当照打、点名那一轮与那几份清单、写明放行但必记检出、检出记录多一行，实际退 {code} {lines}，检出 {recorded}")
        # lightrows：追加一条改坏对象在核心代码下、测试目标新的变异行，轻型登记表没跟上：补丁照打、报红、列出该补的原文、退 1；
        # lightrows-green：登记表补上那一行之后再打一份，报绿、退 0。两格走副本那一支（main_workspace 不给，临时仓不是这份脚本所在的仓），不碰上面开着的那一轮快照
        core_row = "corenew\tcrates/singlefs-core/src/fresh.rs\to\tr\t-p singlefs-core --test fresh_target\tt\n"
        light_rows_red = patch_directory("lightrows", append=core_row)
        code, lines = apply(light_rows_red, root, False, gate33)
        wanted_row = "corenew\t-p singlefs-core --test fresh_target"
        if code != 1 or "corenew\t" not in open(table_path).read() or not any(line.startswith("crates-mutation-light-rows：退出码 1") for line in lines) \
                or not any(wanted_row in line for line in lines):
            failures.append(f"打完之后轻型登记表缺核心测试目标的行应当报红、列出该补的原文、退 1、补丁照打，实际退 {code} {lines}")
        with open(light_table, "a") as handle:
            handle.write(wanted_row + "\n")
        light_rows_green = patch_directory("lightrows-green", append="noncore\tf\to\tr\t-p demo --lib\tt\n")
        code, lines = apply(light_rows_green, root, False, gate33)
        if code != 0 or not any(line.startswith("crates-mutation-light-rows：退出码 0") for line in lines):
            failures.append(f"轻型登记表跟上了之后应当报绿、退 0，实际退 {code} {lines}")
        for failure in failures:
            print(f"  ✗ 自检：{failure}")  # gate-lint:detail
        if failures:
            print("  → 看 plan() / apply() 的判法；APPLY_WRITER_PATCH_BREAK 设着的话这里本来就该红")
            return 1
        print("  ✓ apply-writer-patch 自检通过：好补丁打上、变异表按名字追加替换删除，动 checker 没那一节、追加重名（表里已有的、追加表自己里两行同名的）、打不上的都在核的时候拒绝，到提示线打提示，副本锁被占着时拒绝、没占时照打，"
              "主工作区合入锁被别的合入占着时排队报出持有者、对方放了才打，持有者已死的报出来接管，merge-and-verify 持锁时起的不排队，"
              "主工作树的锁被占着时往 git worktree 加出来的树打补丁不排队，补丁改到开着的三方轮开工快照里的文件（代码与变异表）拒、命令行给了快照冲突已判的照打，"
              "快照清单读不了的照打、点名那一轮与那份清单并记一条检出，"
              "打完轻型登记表缺核心测试目标的行报红列出该补的原文、补上之后报绿（查了 16 种）")
        return 0
    finally:
        import shutil
        shutil.rmtree(work, ignore_errors=True)


if __name__ == "__main__":
    preflight(__file__)
    arguments = sys.argv[1:]
    if arguments == ["--selftest"]:
        sys.exit(selftest())
    root = REPOSITORY_ROOT
    if "--root" in arguments:
        position = arguments.index("--root")
        root = os.path.abspath(arguments[position + 1])
        arguments = arguments[:position] + arguments[position + 2:]
    dry_run = "--dry-run" in arguments
    arguments = [argument for argument in arguments if argument != "--dry-run"]
    snapshot_ruled = []
    while "--snapshot-ruled" in arguments:
        position = arguments.index("--snapshot-ruled")
        if position + 1 >= len(arguments):
            print("✗ --snapshot-ruled 后面缺那一句\n→ 怎么办：写成 --snapshot-ruled '快照冲突已判：<轮名> <理由>'（文件头「开工快照」）")
            sys.exit(2)
        snapshot_ruled.append(arguments[position + 1])
        arguments = arguments[:position] + arguments[position + 2:]
    if len(arguments) != 1 or not os.path.isdir(arguments[0]):
        print("✗ 要给一个补丁目录\n→ 怎么办：apply-writer-patch.py <补丁目录> [--root 仓根] [--dry-run] [--snapshot-ruled '快照冲突已判：<轮名> <理由>']")
        sys.exit(2)
    code, lines = apply(arguments[0], root, dry_run, ["bash", os.path.join(root, ".claude", "gate.d", "code-source-discipline.sh"), "--check", "mutation-tables"],
                        snapshot_ruled=snapshot_ruled)
    print("\n".join(lines))
    sys.exit(code)
