#!/usr/bin/env python3
# admission: always 每个实现员补丁交回都要打一次，判的是这一刻的补丁目录与主工作区
# run-condition: command git
"""把实现员交回的补丁目录打进主工作区：补丁、变异表按名字合并、跑 code-source-discipline，记一笔「自上次崩溃验证全绿以来打了几个补丁」。

用法：
    apply-writer-patch.py <补丁目录> [--root 仓根] [--dry-run]
    apply-writer-patch.py --selftest          # APPLY_WRITER_PATCH_BREAK=<项> 时必须判红

补丁目录里的文件（格式由 .claude/agents/implementation-writer.md「产出」一节定）：
    crates.patch                 `git diff -- crates litmus ':!crates/mutations.tsv'` 的输出；没有代码改动可以没有
    mutations-append.tsv         要追加的变异行，每行六段（与 crates/mutations.tsv 同），名字在表里不许已有
    mutations-replacements.tsv   要整行换掉的变异行，每行六段，按第一段的名字找表里恰好一行换掉
    mutations-delete.txt         要删的变异名，一行一个，表里要恰好一行
    report.md                    实现员的报告；补丁动了 crates/singlefs-checker/src/ 的，报告里要有「受影响的层 0 流与崩溃枚举用例」一节
次序：先全部核一遍（补丁 git apply --check、名字在不在、行是不是六段、checker 那一节在不在），有一处不对就一个字不改、退 2；
都对了才 git apply、改名换上新的变异表，跑 .claude/gate.d/code-source-discipline.sh 的 mutation-tables 那一格（--check mutation-tables），把这一次记进 <git common-dir>/singlefs-applied-writer-patches.tsv。
自上次崩溃验证全绿（git common-dir 里最新的 singlefs-crash-case-green.* 标记）以来打过的补丁到 CRASH_VERIFY_HINT_PATCHES 个（5，推的）就打一行提示：
只提示、不拦，跑不跑由用户定。
副本锁（主 agent 2026-10-10 的失误：scripts/mutation/prove-red.sh --copy 正在副本里就地施加变异时，往同一份副本打了补丁）：
  --root 指的不是这份脚本所在的主工作区时，开头在 <副本根>/.prove-red.lock 上试拿排他锁（flock，不等），拿到了打完再放、拿不到就拒绝、退 2、一个字没改，
  出路写「副本上有证红在跑，等它跑完或换一份副本」。prove-red.sh 跑的时候拿着同一把锁（它按路径载入这里的 hold_copy_lock / release_copy_lock，锁的写法只在这一处）。
  锁随持有的进程放：正常收尾时删掉锁文件；被杀时内核放掉锁、文件留着，下一个照样拿得到。
主工作区合入锁（.claude/kb/milestone/03-third-txn.md「待优化：流程、门禁与装置上踩过的坑」第 20 行）：
  --root 是这份脚本所在的主工作区（不给 --root 就是）时，核与打之前先拿 scripts/gate/lib_workspace_lock.py 的主工作区合入锁（持有者是这个进程），
  被别的合入占着就排队、每过一段时间报一行谁占着，持有者已死的判失效、报出来再接管；打完（成功、失败都）放掉。判法与锁文件在那份库的文件头，这里不另写。
  merge-and-verify.sh 持着锁时起的这一份（SINGLEFS_WORKSPACE_LOCK_HOLDER 指着它的祖先）不另拿。拿不成（建不了锁文件、
  SINGLEFS_WORKSPACE_LOCK_GIVE_UP_SECONDS 设了而等到点）退 2、一个字没改。副本上不拿这一把（副本锁管副本）。
退出码：0 打上了（code-source-discipline 绿）；1 打上了但 code-source-discipline 红；2 核的时候就不对、副本上有证红在跑、或主工作区合入锁没拿成，一个字没改。

弄坏开关（只给 --selftest 证红用）。逐项写开关名、弄坏哪一处、打开后自证哪几项红（项名是 selftest() 里那一份补丁目录的名字，括号里是那条「✗ 自检：」的开头）：
  APPLY_WRITER_PATCH_BREAK=append-duplicates  plan() 不核追加的变异名在表里有没有
      duplicate（「✗ 自检：追加已有的名字应当拒绝」）
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
MARKER_PREFIX = "singlefs-crash-case-green."
COPY_LOCK_NAME = ".prove-red.lock"   # 副本锁（文件头「副本锁」）：放在副本根


def load_workspace_lock():
    """主工作区合入锁（文件头「主工作区合入锁」）：按路径载入同目录的 lib_workspace_lock.py（prove-red.sh 按路径载入这一份时 sys.path 里不一定有本目录）。"""
    specification = importlib.util.spec_from_file_location("lib_workspace_lock_for_apply_writer_patch", os.path.join(SCRIPT_DIRECTORY, "lib_workspace_lock.py"))
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


WORKSPACE_LOCK = load_workspace_lock()


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
        touched = [line[len("+++ b/"):].strip() for line in open(patch, encoding="utf-8", errors="replace") if line.startswith("+++ b/")]
        if "crates/mutations.tsv" in touched:
            problems.append("crates.patch 里动了 crates/mutations.tsv：变异表的改动写进那三个文件，按名字合并")
        report_path = os.path.join(patch_directory, "report.md")
        report = open(report_path, encoding="utf-8", errors="replace").read() if os.path.isfile(report_path) else ""
        if any(path.startswith("crates/singlefs-checker/src/") for path in touched) and CHECKER_SECTION not in report and BROKEN != "checker-section-ignored":
            problems.append(f"补丁动了 crates/singlefs-checker/src/，report.md 里却没有「{CHECKER_SECTION}」一节")
        checked = git(root, "apply", "--check", patch)
        if checked.returncode != 0 and BROKEN != "apply-unchecked":
            problems.append(f"git apply --check 不过：{checked.stderr.strip()[:300]}")
    table_path = os.path.join(root, "crates", "mutations.tsv")
    if not os.path.isfile(table_path):
        return problems + ["主工作区没有 crates/mutations.tsv"], None, patch
    lines = open(table_path, encoding="utf-8").read().splitlines(keepends=True)
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
    markers = glob.glob(os.path.join(common, MARKER_PREFIX + "*"))
    since = max((os.path.getmtime(path) for path in markers), default=0.0)
    rows = [line.split("\t") for line in open(log_path, encoding="utf-8")] if os.path.isfile(log_path) else []
    return sum(1 for row in rows if row and float(row[0]) > since)


def apply(patch_directory, root, dry_run, gate33, main_workspace=None, say=print_now):
    """副本上先试拿副本锁（文件头「副本锁」），拿到了再核、再打，打完放锁；主工作区拿主工作区合入锁（文件头「主工作区合入锁」，排队等）。
    main_workspace 不给就按 root 是不是这份脚本所在的仓判（自证给 True，把临时仓当主工作区）。"""
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
            return apply_holding_the_lock(patch_directory, root, dry_run, gate33)
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
        return apply_holding_the_lock(patch_directory, root, dry_run, gate33)
    finally:
        if lock is not None:
            release_copy_lock(root, lock)


def apply_holding_the_lock(patch_directory, root, dry_run, gate33):
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
    common = git(root, "rev-parse", "--git-common-dir").stdout.strip()
    log_path = os.path.join(common if os.path.isabs(common) else os.path.join(root, common), LOG_NAME)
    digest = hashlib.sha256(open(patch, "rb").read()).hexdigest() if patch else "-"
    with open(log_path, "a", encoding="utf-8") as handle:
        handle.write(f"{time.time()}\t{os.path.abspath(patch_directory)}\t{digest}\n")
    count = patches_since_last_crash_verification(root, log_path)
    if count >= CRASH_VERIFY_HINT_PATCHES and BROKEN != "no-hint":
        lines.append(f"! 自上次崩溃验证全绿以来已打 {count} 个实现员补丁（提示线 {CRASH_VERIFY_HINT_PATCHES}，推的）：弹窗问用户要不要用 "
                     "SINGLEFS_HEAVY_TESTS=user-request 由主 agent 跑一次（checker-tier-crash-point-replay 按用例复用，只跑输入变了的崩溃枚举用例）；只提示，不拦")
    lines.append(f"调度表可贴的一行：| {os.path.basename(os.path.normpath(patch_directory))} | 已打（{time.strftime('%m-%d %H:%M', time.localtime())}） | code-source-discipline {gate.returncode} |")
    return (0 if gate.returncode == 0 else 1), lines


def selftest():
    work = tempfile.mkdtemp(prefix="apply-writer-patch-selftest-")
    # 主工作区合入锁那三格：锁文件用临时仓自己的 git common-dir，调用方设的锁位置、持有者变量与等待上限不带进来；排队时每 0.3 秒报一次、每 0.05 秒看一次
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
            for file_name, text in (("mutations-append.tsv", append), ("mutations-replacements.tsv", replace), ("mutations-delete.txt", delete), ("report.md", report)):
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
        # 主工作区合入锁（文件头「主工作区合入锁」）：临时仓当主工作区（main_workspace=True），锁文件在它的 git common-dir 里
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
        for failure in failures:
            print(f"  ✗ 自检：{failure}")  # gate-lint:detail
        if failures:
            print("  → 看 plan() / apply() 的判法；APPLY_WRITER_PATCH_BREAK 设着的话这里本来就该红")
            return 1
        print("  ✓ apply-writer-patch 自检通过：好补丁打上、变异表按名字追加替换删除，动 checker 没那一节、追加重名、打不上的都在核的时候拒绝，到提示线打提示，副本锁被占着时拒绝、没占时照打，"
              "主工作区合入锁被别的合入占着时排队报出持有者、对方放了才打，持有者已死的报出来接管，merge-and-verify 持锁时起的不排队（查了 9 种）")
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
    if len(arguments) != 1 or not os.path.isdir(arguments[0]):
        print("✗ 要给一个补丁目录\n→ 怎么办：apply-writer-patch.py <补丁目录> [--root 仓根] [--dry-run]")
        sys.exit(2)
    code, lines = apply(arguments[0], root, dry_run, ["bash", os.path.join(root, ".claude", "gate.d", "code-source-discipline.sh"), "--check", "mutation-tables"])
    print("\n".join(lines))
    sys.exit(code)
