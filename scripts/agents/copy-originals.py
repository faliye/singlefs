#!/usr/bin/env python3
# admission: always 拷的是此刻仓里那几份文件、核的是此刻仓里的样子，上一次拷过或核过不替这一次作保
# run-condition: none 只读仓内文件、写仓外的草稿目录，不要别的工具、设备与权限
"""改仓内文件之前，把原文件拷进自己的草稿目录留底，之后核对哪几份改过。目标路径由脚本照仓内的相对路径算，不用在 Bash 里拼变量目标。

用法：
  copy-originals.py copy <草稿目录> <文件>… [--root <仓根>]   # 每份拷到 <草稿目录>/originals/<相对仓根的路径>，清单写 <草稿目录>/originals/manifest.tsv
  copy-originals.py check <草稿目录>                           # 照清单逐份比仓里此刻的 sha256，列出自拷下以来改过的与不在了的
  copy-originals.py --selftest                                 # COPY_ORIGINALS_BREAK=<项> 时必须判红
  copy-originals.py --help                                     # 打这一段用法（-h 同），退 0

<草稿目录> 只收绝对路径，要落在工作根（lib_work_root.py 的 scratch_roots()：$SINGLEFS_WORK_ROOT，默认 ~/.local/share/singlefs/work）
  或过渡期旧根 /tmp/claude-<uid>/ 下面（根本身不收），解开符号链接之后不许落在 <仓根> 或这份脚本所在的仓里。
<文件> 写相对仓根的路径或仓内的绝对路径，要是仓里已有的普通文件（目录、仓外的、不在的拒）。根下 manifest.tsv 路径留给备份清单，不收同路径的文件或目录子项。<仓根> 不给时是这份脚本所在的仓。
清单 manifest.tsv：开头一行「# root<TAB><仓根>」，之后一行一份「<相对仓根的路径><TAB><sha256><TAB><拷贝相对草稿目录的路径>」；同一个草稿目录多次 copy 往里接着记。
目标已在而内容相同的不再拷、照记；内容不同的整批拒（一份都不拷、清单不动），不静默覆盖；清单里这份记的 sha256 与此刻的不同也拒。
  要留第二版就换一个草稿目录（或子目录）再 copy。
check 用清单开头记的仓根；改过的逐行写「改过 <路径>（拷贝在 <拷贝的绝对路径>）」，不在了的写「不在了 <路径>」。

退出码：0 拷成了（check：清单里的都没改过）；1 check 列出了改过或不在了的；2 用法错（缺参数、不认的子命令或选项）；
  3 拒绝（草稿目录不是绝对路径、落在仓里或工作根外，文件不在仓里、不在、不是普通文件，已有目标内容不同，草稿目录里没有清单）。

弄坏开关（只给 --selftest 证红用，逐项写开关名、弄坏哪一处、打开后自证哪一格红；格名是失败行「✗ 自检：」之后、「应当是」之前那一段）：
  COPY_ORIGINALS_BREAK=relative-scratch-accepted  scratch_problem() 不拒相对路径的草稿目录
      「拒相对路径的草稿目录」
  COPY_ORIGINALS_BREAK=inside-repository-accepted  scratch_problem() 不拒落在仓里的草稿目录
      「拒落在仓里的草稿目录」「拒落在仓里的草稿目录时一个字节不写」
  COPY_ORIGINALS_BREAK=outside-scratch-roots-accepted  scratch_problem() 不拒工作根与旧根之外的草稿目录
      「拒工作根之外的草稿目录」
  COPY_ORIGINALS_BREAK=source-checks-skipped  source_problem() 不拒仓外的、不在的与不是普通文件的源
      「拒仓外的源文件」「拒不在的源文件」「拒目录当源」「拒绝时整批一份都不拷」
  COPY_ORIGINALS_BREAK=differing-target-overwritten  plan_copies() 不拒内容不同的已有目标，照拷覆盖
      「拒内容不同的已有目标」「拒内容不同的已有目标时拷贝原样不动」「拒内容不同的已有目标（没有清单、拷贝是别处放的）」
  COPY_ORIGINALS_BREAK=check-ignores-changes  check() 不比 sha256，只看文件在不在
      「核对出改动」「核对点名改过的那一份」
  COPY_ORIGINALS_BREAK=empty-file-list-accepted  parse_arguments() 不拒 copy 后面没给文件
      「用法错：copy 没给文件」
  COPY_ORIGINALS_BREAK=guard-off  refuse_break_outside_selftest() 不拒平常跑时设上的弄坏开关；它是关守卫项，只在它是唯一一项时认，与别的项同写照拒并点名它
      「平常跑设了弄坏开关退 2 并点名只在自证里认」
  COPY_ORIGINALS_BREAK=guard-off-with-others  refuse_break_outside_selftest() 关守卫项与别的项同写也照认（改前的样子）；这一项只在自证进程本身里认
      「关守卫项与别的项同写：进程里也拒并点名它」
这个变量只在自证（--selftest）里认：平常跑（命令行不是单独一个 --selftest）设了它，不管值是什么，参数解析之前就退 2、点名它只在自证里认。
关守卫项 guard-off 整值比：只在它是唯一一项时照认（平常跑也放行）；与别的项同写一律退 2，点名「关守卫项 guard-off 只在它是唯一一项时认」。
guard-off-with-others 只在自证进程本身调 refuse_break_outside_selftest(…, in_selftest=True) 时认；平常跑同写着它照拒，它自己不成借道。
不认识的值直接拒绝（退 2），免得拼错的开关被当成「没开」。
"""
import hashlib
import fcntl
import importlib.util
import os
import re
import shutil
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "scripts"))
from project_preflight import preflight  # noqa: E402

BREAK_VARIABLE = "COPY_ORIGINALS_BREAK"
KNOWN_BREAK_ITEMS = ("relative-scratch-accepted", "inside-repository-accepted", "outside-scratch-roots-accepted", "source-checks-skipped",
                     "differing-target-overwritten", "check-ignores-changes", "empty-file-list-accepted", "guard-off", "guard-off-with-others")
GUARD_OFF_ITEM = "guard-off"   # 关守卫项：只在它是唯一一项时认（文件头开关表下面那几行）
ORIGINALS_DIRECTORY_NAME = "originals"
MANIFEST_NAME = "manifest.tsv"
MANIFEST_ROOT_PREFIX = "# root\t"
USAGE_EXIT = 2
REFUSED_EXIT = 3
CHANGED_EXIT = 1


class UsageError(Exception):
    pass


class Refused(Exception):
    def __init__(self, reason, next_step):
        super().__init__(reason)
        self.reason, self.next_step = reason, next_step


def break_items():
    items = [item for item in os.environ.get(BREAK_VARIABLE, "").split(",") if item]
    unknown = [item for item in items if item not in KNOWN_BREAK_ITEMS]
    if unknown:
        raise UsageError(f"{BREAK_VARIABLE} 里的 {'、'.join(unknown)} 不认识，只认 {'、'.join(KNOWN_BREAK_ITEMS)}")
    return items


def break_is_set(item):
    return item in break_items()


def refuse_break_outside_selftest(arguments, value=None, in_selftest=False):
    """弄坏开关只在自证里认：命令行不是单独一个 --selftest 而变量非空，交回退出码 2（不管值是什么，参数解析之前判）；否则交 None。
    value 不给时读环境；in_selftest 只有自证进程本身在进程里调时给 True（弄坏开关 guard-off-with-others 只在那时认）。"""
    value = os.environ.get(BREAK_VARIABLE, "") if value is None else value
    if not value or arguments == ["--selftest"]:
        return None
    items = value.split(",")
    if items == [GUARD_OFF_ITEM]:   # 关守卫项单独一项：这一道不拒（给自证里「平常跑设了弄坏开关退 2」那一格证红）
        return None
    if GUARD_OFF_ITEM in items:
        if in_selftest and break_is_set("guard-off-with-others"):   # 弄坏开关 guard-off-with-others：同写也照认（改前的样子）
            return None
        print(f"  ✗ {BREAK_VARIABLE}={value} 是弄坏开关，只在自证（--selftest）里认；关守卫项 {GUARD_OFF_ITEM} 只在它是唯一一项时认，与别的项同写照拒，这一次什么都没做")
        print(f"     → 怎么办：unset {BREAK_VARIABLE} 再来；要关这一道就单独设 {BREAK_VARIABLE}={GUARD_OFF_ITEM}；要看开关红在哪一格，一次只设一项跑 --selftest")
        return USAGE_EXIT
    print(f"  ✗ {BREAK_VARIABLE}={value} 是弄坏开关，只在自证（--selftest）里认；平常跑设了就拒，这一次什么都没做")
    print(f"     → 怎么办：unset {BREAK_VARIABLE} 再来；要看开关红在哪一格，跑 {BREAK_VARIABLE}=<项> python3 scripts/agents/copy-originals.py --selftest")
    return USAGE_EXIT


def load_work_root_library():
    path = os.path.join(REPOSITORY_ROOT, ".claude", "hooks", "lib_work_root.py")
    specification = importlib.util.spec_from_file_location("lib_work_root", path)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def file_sha256(path):
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def is_within(path, directory):
    return path == directory or path.startswith(directory.rstrip("/") + "/")


def scratch_problem(scratch, repository_roots, scratch_roots):
    """草稿目录不能用时交回 (原因, 下一步)，能用交 None。"""
    if not os.path.isabs(scratch) and not break_is_set("relative-scratch-accepted"):
        return (f"草稿目录 {scratch} 不是绝对路径", "写派发提示给的草稿目录的绝对路径（~/.local/share/singlefs/work/<轮>/ 这类），不写相对路径")
    resolved = os.path.realpath(scratch)
    if not break_is_set("inside-repository-accepted"):
        for root in repository_roots:
            if is_within(resolved, os.path.realpath(root)):
                return (f"草稿目录 {scratch} 落在仓 {root} 里", "原文件拷到仓外的草稿目录（工作根 ~/.local/share/singlefs/work/ 下派发提示给的那一个），不拷进仓里")
    if not break_is_set("outside-scratch-roots-accepted"):
        roots = [os.path.realpath(root) for root in scratch_roots]
        if not any(resolved != root and is_within(resolved, root) for root in roots):
            return (f"草稿目录 {scratch} 不在工作根或旧根下面（认的根：{'、'.join(roots)}，根本身不收）",
                    "用派发提示给的草稿目录，它在工作根 ~/.local/share/singlefs/work/ 下")
    return None


def source_problem(word, root):
    """一份要拷的文件：交回 (相对仓根的路径, None) 或 (None, (原因, 下一步))。"""
    absolute = os.path.normpath(word if os.path.isabs(word) else os.path.join(root, word))
    relative = os.path.relpath(absolute, root)
    if relative == MANIFEST_NAME or relative.startswith(MANIFEST_NAME + "/"):
        return None, (f"{word} 的备份路径与清单 {MANIFEST_NAME} 冲突", "此路径请单独留底，不能写到 originals/manifest.tsv 上")
    if any(character in relative for character in "\t\r\n"):
        return None, (f"文件路径含清单不能表示的制表符或换行：{word!r}", "重命名后再留底，清单使用逐行 TSV 格式")
    if break_is_set("source-checks-skipped"):
        return relative, None
    if relative == ".." or relative.startswith("../") or relative == ".":
        return None, (f"{word} 不在仓 {root} 里", "只拷仓内文件；仓外的文件不用这个脚本留底")
    if not is_within(os.path.realpath(absolute), os.path.realpath(root)):
        return None, (f"{word} 的真实路径不在仓 {root} 里", "源文件的父目录不能通过符号链接指向仓外")
    if not os.path.lexists(absolute):
        return None, (f"{word} 不在", "先 ls 核对路径（写相对仓根的路径），新建的文件没有原文件可留")
    if os.path.islink(absolute) or not os.path.isfile(absolute):
        return None, (f"{word} 不是普通文件（目录或符号链接）", "逐个写出目录里要留底的文件，不写目录")
    return relative, None


def read_manifest(scratch):
    """交回 (仓根, {相对路径: (sha256, 拷贝相对草稿目录的路径)})；没有清单交 (None, {})。"""
    path = os.path.join(scratch, ORIGINALS_DIRECTORY_NAME, MANIFEST_NAME)
    if not os.path.isfile(path):
        return None, {}
    root, rows = None, {}
    with open(path, encoding="utf-8") as handle:
        for line in handle.read().splitlines():
            fields = line.split("\t")
            if len(fields) == 3 and re.fullmatch(r"[0-9a-f]{64}", fields[1]):
                relative, digest, copy = fields
                rows[relative] = (digest, copy)
            elif line.startswith(MANIFEST_ROOT_PREFIX):
                root = line[len(MANIFEST_ROOT_PREFIX):]
            elif line and (not line.startswith("#") or "\t" in line):
                raise Refused(f"清单 {path} 有损坏的记录：{line!r}", "从可信备份恢复清单后再核对，不忽略损坏的记录")
    if not root:
        raise Refused(f"清单 {path} 缺仓根", "恢复完整清单后再核对")
    return root, rows


def write_manifest(scratch, root, rows):
    directory = os.path.join(scratch, ORIGINALS_DIRECTORY_NAME)
    text = MANIFEST_ROOT_PREFIX + root + "\n" + "".join(f"{relative}\t{digest}\t{copy}\n" for relative, (digest, copy) in rows.items())
    handle, temporary = tempfile.mkstemp(dir=directory, prefix=".manifest-")
    with os.fdopen(handle, "w", encoding="utf-8") as stream:
        stream.write(text)
    os.replace(temporary, os.path.join(directory, MANIFEST_NAME))


def checked_backup_path(scratch, relative):
    """逐层拒绝备份路径里的符号链接，包含 originals、清单和已有目标。"""
    path = scratch
    for component in (ORIGINALS_DIRECTORY_NAME, *relative.split("/")):
        path = os.path.join(path, component)
        if os.path.islink(path):
            raise Refused(f"备份路径 {path} 是符号链接", "换一个没有符号链接的草稿目录留底，备份必须独立保存在该目录内")
    return path


def plan_copies(scratch, root, relatives, manifest_root, rows):
    """逐份判：交回 [(相对路径, sha256, 拷贝的绝对路径, 要不要拷)]；有一份不行就抛 Refused（一份都不拷）。"""
    if manifest_root is not None and os.path.realpath(manifest_root) != os.path.realpath(root):
        raise Refused(f"草稿目录 {scratch} 的清单记的仓根是 {manifest_root}，这一次是 {root}", "换一个草稿目录（或子目录）给这个仓留底")
    plan = []
    for relative in dict.fromkeys(relatives):
        digest = file_sha256(os.path.join(root, relative))
        copy = checked_backup_path(scratch, relative)
        overwrite = break_is_set("differing-target-overwritten")
        if relative in rows and rows[relative][0] != digest and not overwrite:
            raise Refused(f"{relative} 在清单里记的 sha256 是 {rows[relative][0]}，仓里此刻是 {digest}：自上次拷下以来改过",
                          f"拷下的那一份还在 {copy}，不覆盖；要留此刻这一版就换一个草稿目录（或子目录）再 copy，先用 check 看改过哪些")
        if os.path.lexists(copy):
            if not os.path.isfile(copy) or os.path.islink(copy):
                raise Refused(f"目标 {copy} 已在而不是普通文件", "换一个草稿目录（或子目录）再 copy")
            if os.path.samefile(copy, os.path.join(root, relative)):
                raise Refused(f"目标 {copy} 与源文件是同一文件", "换一个草稿目录留底，不能把源文件的硬链接当备份")
            if file_sha256(copy) != digest and not overwrite:
                raise Refused(f"目标 {copy} 已在，内容与仓里的 {relative} 不同", "不覆盖已有的拷贝；换一个草稿目录（或子目录）再 copy")
            plan.append((relative, digest, copy, overwrite and file_sha256(copy) != digest))
        else:
            plan.append((relative, digest, copy, True))
    return plan


def copy_originals(scratch, words, root):
    if any(character in root or character in scratch for character in "\t\r\n"):
        raise Refused("仓根或草稿目录含制表符或换行", "使用逐行 TSV 清单能够表示的目录路径")
    work_root = load_work_root_library()
    problem = scratch_problem(scratch, (root, REPOSITORY_ROOT), work_root.scratch_roots())
    if problem:
        raise Refused(*problem)
    scratch = os.path.realpath(scratch)
    relatives = []
    for word in words:
        relative, problem = source_problem(word, root)
        if problem:
            raise Refused(*problem)
        relatives.append(relative)
    checked_backup_path(scratch, MANIFEST_NAME)
    os.makedirs(scratch, exist_ok=True)
    descriptor = os.open(scratch, os.O_RDONLY | os.O_DIRECTORY)
    try:
        fcntl.flock(descriptor, fcntl.LOCK_EX)
        return copy_originals_locked(scratch, relatives, root)
    finally:
        os.close(descriptor)


def copy_originals_locked(scratch, relatives, root):
    """目录锁覆盖读清单、检查目标、复制和写清单，避免并发更新互相覆盖。"""
    checked_backup_path(scratch, MANIFEST_NAME)
    manifest_root, rows = read_manifest(scratch)
    plan = plan_copies(scratch, root, relatives, manifest_root, rows)
    os.makedirs(os.path.join(scratch, ORIGINALS_DIRECTORY_NAME), exist_ok=True)
    copied = 0
    for relative, digest, copy, needs_copy in plan:
        if needs_copy:
            os.makedirs(os.path.dirname(copy), exist_ok=True)
            handle, temporary = tempfile.mkstemp(dir=os.path.dirname(copy), prefix=".copy-")
            os.close(handle)
            shutil.copy2(os.path.join(root, relative), temporary)
            if file_sha256(temporary) != digest:
                os.unlink(temporary)
                raise Refused(f"{relative} 拷的时候变了（拷下的 sha256 与开拷前的不同）", "等别的写者写完再 copy")
            os.replace(temporary, copy)
            copied += 1
        rows[relative] = (digest, os.path.relpath(copy, scratch))
    write_manifest(scratch, root, rows)
    print(f"  ✓ 留了 {len(plan)} 份（新拷 {copied} 份、已在 {len(plan) - copied} 份），清单 {os.path.join(scratch, ORIGINALS_DIRECTORY_NAME, MANIFEST_NAME)}"
          f" 共记 {len(rows)} 份；核对：python3 scripts/agents/copy-originals.py check {scratch}")
    return 0


def check(scratch):
    if not os.path.isabs(scratch) and not break_is_set("relative-scratch-accepted"):
        raise Refused(f"草稿目录 {scratch} 不是绝对路径", "写 copy 时给的那个草稿目录的绝对路径")
    root, rows = read_manifest(scratch)
    if root is None:
        raise Refused(f"{scratch} 里没有清单 {ORIGINALS_DIRECTORY_NAME}/{MANIFEST_NAME}", "先 copy-originals.py copy <草稿目录> <文件>… 留底，再 check")
    changed, missing = [], []
    for relative, (digest, copy) in rows.items():
        path = os.path.join(root, relative)
        if not os.path.isfile(path):
            missing.append(relative)
        elif not break_is_set("check-ignores-changes") and file_sha256(path) != digest:
            changed.append((relative, os.path.join(scratch, copy)))
    for relative, copy in changed:
        print(f"  改过 {relative}（拷贝在 {copy}）")
    for relative in missing:
        print(f"  不在了 {relative}")
    unchanged = len(rows) - len(changed) - len(missing)
    print(f"  {'✓' if not changed and not missing else '·'} 清单 {len(rows)} 份（仓根 {root}）：改过 {len(changed)} 份、不在了 {len(missing)} 份、没改 {unchanged} 份")
    return CHANGED_EXIT if changed or missing else 0


def parse_arguments(arguments):
    """交回 (子命令, 草稿目录, 文件, 仓根)；不对抛 UsageError。"""
    if not arguments:
        raise UsageError("缺子命令")
    command, rest = arguments[0], arguments[1:]
    if command not in ("copy", "check"):
        raise UsageError(f"不认的子命令 {command}")
    root, words = None, []
    position = 0
    while position < len(rest):
        word = rest[position]
        if word == "--root" and command == "copy":
            if position + 1 >= len(rest):
                raise UsageError("--root 缺值")
            root = rest[position + 1]
            position += 2
            continue
        if word.startswith("-"):
            raise UsageError(f"不认的选项 {word}")
        words.append(word)
        position += 1
    if not words:
        raise UsageError(f"{command} 缺草稿目录")
    if command == "check" and len(words) != 1:
        raise UsageError("check 只收一个草稿目录")
    if command == "copy" and len(words) < 2 and not break_is_set("empty-file-list-accepted"):
        raise UsageError("copy 没给要留底的文件")
    return command, words[0], words[1:], os.path.realpath(root) if root else REPOSITORY_ROOT


def run(arguments):
    if arguments and arguments[0] in ("-h", "--help"):
        print(__doc__)
        return 0
    try:
        command, scratch, words, root = parse_arguments(arguments)
        if command == "copy":
            return copy_originals(scratch, words, root)
        return check(scratch)
    except UsageError as error:
        print(f"  ✗ 用法错：{error}")
        print("     → 怎么办：copy-originals.py copy <草稿目录的绝对路径> <仓内文件>… [--root <仓根>]，或 copy-originals.py check <草稿目录>；全文看 --help")
        return USAGE_EXIT
    except Refused as refusal:
        print(f"  ✗ 拒绝：{refusal.reason}")
        print(f"     → 怎么办：{refusal.next_step}")
        return REFUSED_EXIT


def selftest():
    import contextlib
    import io
    try:
        break_items()
    except UsageError as error:
        print(f"  ✗ {error}")
        print(f"     → 怎么办：{BREAK_VARIABLE} 只写文件头开关表里列的项")
        return USAGE_EXIT
    uid_root = f"/tmp/claude-{os.getuid()}"
    os.makedirs(uid_root, exist_ok=True)
    work = tempfile.mkdtemp(prefix="copy-originals-selftest-", dir=uid_root)
    outside_roots = tempfile.mkdtemp(prefix="copy-originals-selftest-outside-", dir="/tmp")   # 工作根与旧根之外
    results = []
    def call(*arguments):
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            try:
                code = run(list(arguments))
            except Exception as error:   # 弄坏开关下走到不该走的分支时抛的，记成这一格的结果而不是让自检中断
                code = f"抛了 {type(error).__name__}"
        return code, output.getvalue()
    def call_in(directory, *arguments):
        before = os.getcwd()
        os.chdir(directory)
        try:
            return call(*arguments)
        finally:
            os.chdir(before)
    def check_cell(label, want, got):
        results.append((label, want, got))
    try:
        repository = os.path.join(work, "repository")
        os.makedirs(os.path.join(repository, "scripts", "gate"))
        os.makedirs(os.path.join(repository, ".claude", "hooks"))
        with open(os.path.join(repository, "scripts", "gate", "merge.sh"), "w") as handle:
            handle.write("echo merge\n")
        with open(os.path.join(repository, ".claude", "hooks", "guard.sh"), "w") as handle:
            handle.write("echo guard\n")
        outside = os.path.join(work, "outside.txt")
        with open(outside, "w") as handle:
            handle.write("outside\n")
        scratch = os.path.join(work, "scratch")
        code, _ = call("copy", scratch, "scripts/gate/merge.sh", os.path.join(repository, ".claude/hooks/guard.sh"), "--root", repository)
        backup = os.path.join(scratch, "originals", "scripts", "gate", "merge.sh")
        manifest_root, rows = read_manifest(scratch)
        check_cell("拷成：退 0", 0, code)
        check_cell("拷成：拷贝照相对仓根的路径落在 originals/ 下、内容相同", True,
                   os.path.isfile(backup) and open(backup).read() == "echo merge\n"
                   and open(os.path.join(scratch, "originals", ".claude", "hooks", "guard.sh")).read() == "echo guard\n")
        check_cell("拷成：清单记仓根与两份的 sha256", (repository, {"scripts/gate/merge.sh": hashlib.sha256(b"echo merge\n").hexdigest(),
                                                                 ".claude/hooks/guard.sh": hashlib.sha256(b"echo guard\n").hexdigest()}),
                   (manifest_root, {relative: digest for relative, (digest, _) in rows.items()}))
        check_cell("拷成：同样再拷一次照退 0（已在、内容相同）", 0, call("copy", scratch, "scripts/gate/merge.sh", "--root", repository)[0])
        check_cell("核对：没改过退 0", 0, call("check", scratch)[0])
        with open(os.path.join(repository, "scripts", "gate", "merge.sh"), "a") as handle:
            handle.write("echo changed\n")
        code, output = call("check", scratch)
        check_cell("核对出改动", CHANGED_EXIT, code)
        check_cell("核对点名改过的那一份", (True, False), ("改过 scripts/gate/merge.sh" in output, "guard.sh" in output.split("清单")[0]))
        code, _ = call("copy", scratch, "scripts/gate/merge.sh", "--root", repository)
        check_cell("拒内容不同的已有目标", REFUSED_EXIT, code)
        check_cell("拒内容不同的已有目标时拷贝原样不动", "echo merge\n", open(backup).read())
        other = os.path.join(work, "other-scratch")
        os.makedirs(os.path.join(other, "originals", "scripts", "gate"))
        with open(os.path.join(other, "originals", "scripts", "gate", "merge.sh"), "w") as handle:
            handle.write("something else\n")
        check_cell("拒内容不同的已有目标（没有清单、拷贝是别处放的）", REFUSED_EXIT, call("copy", other, "scripts/gate/merge.sh", "--root", repository)[0])
        check_cell("拒相对路径的草稿目录", REFUSED_EXIT, call_in(work, "copy", "relative-scratch", "scripts/gate/merge.sh", "--root", repository)[0])
        inside = os.path.join(repository, "scratch-in-repository")
        check_cell("拒落在仓里的草稿目录", REFUSED_EXIT, call("copy", inside, "scripts/gate/merge.sh", "--root", repository)[0])
        check_cell("拒落在仓里的草稿目录时一个字节不写", False, os.path.exists(inside))
        check_cell("拒工作根之外的草稿目录", REFUSED_EXIT, call("copy", os.path.join(outside_roots, "scratch"), "scripts/gate/merge.sh", "--root", repository)[0])
        fresh = os.path.join(work, "fresh-scratch")
        check_cell("拒仓外的源文件", REFUSED_EXIT, call("copy", fresh, outside, "--root", repository)[0])
        check_cell("拒不在的源文件", REFUSED_EXIT, call("copy", fresh, "scripts/gate/absent.sh", "--root", repository)[0])
        check_cell("拒目录当源", REFUSED_EXIT, call("copy", fresh, "scripts/gate", "--root", repository)[0])
        check_cell("拒绝时整批一份都不拷", False, os.path.exists(fresh))
        check_cell("用法错：没给子命令", USAGE_EXIT, call()[0])
        check_cell("用法错：不认的子命令", USAGE_EXIT, call("restore", scratch)[0])
        check_cell("用法错：copy 没给文件", USAGE_EXIT, call("copy", fresh, "--root", repository)[0])
        check_cell("用法错：不认的选项", USAGE_EXIT, call("check", scratch, "--force-it")[0])
        check_cell("核对：没有清单的草稿目录拒", REFUSED_EXIT, call("check", os.path.join(work, "never-copied"))[0])
        # 平常跑设了弄坏开关：子进程里带上变量、不带 --selftest，要退 2 并点名变量只在自证里认（探针选项没有哪个子命令认得，没有这道拒的只会退用法错）
        # 关守卫项 guard-off 设着时探针只写它一项（同写会被拒，这一格就红不出来）
        probe_value = GUARD_OFF_ITEM if break_is_set(GUARD_OFF_ITEM) else "relative-scratch-accepted"
        probe_arguments = [sys.executable, "-B", os.path.abspath(__file__), "copy", scratch, "--force", "--break-guard-probe-unknown-option"]
        probe = subprocess.run(probe_arguments, env=dict(os.environ, **{BREAK_VARIABLE: probe_value}), stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=60)
        named = any(BREAK_VARIABLE in row and "自证" in row for row in (probe.stdout + probe.stderr).split("\n"))
        check_cell("平常跑设了弄坏开关退 2 并点名只在自证里认", (USAGE_EXIT, True), (probe.returncode, named))
        # 关守卫项与别的项同写：平常跑的子进程要退 2 并点名关守卫项只在它是唯一一项时认；自证进程里调同一个函数也要拒（guard-off-with-others 只在这里认）
        combined = f"{GUARD_OFF_ITEM},relative-scratch-accepted"
        guard_named = f"关守卫项 {GUARD_OFF_ITEM} 只在它是唯一一项时认"
        probe = subprocess.run(probe_arguments, env=dict(os.environ, **{BREAK_VARIABLE: combined}), stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=60)
        check_cell("关守卫项与别的项同写：平常跑的子进程退 2 并点名它", (USAGE_EXIT, True), (probe.returncode, guard_named in probe.stdout + probe.stderr))
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            code = refuse_break_outside_selftest(["copy", scratch, "--break-guard-probe-unknown-option"], value=combined, in_selftest=True)
        check_cell("关守卫项与别的项同写：进程里也拒并点名它", (USAGE_EXIT, True), (code, guard_named in output.getvalue()))
    finally:
        shutil.rmtree(work, ignore_errors=True)
        shutil.rmtree(outside_roots, ignore_errors=True)
    failures = [item for item in results if item[1] != item[2]]
    for label, want, got in failures:
        print(f"  ✗ 自检：{label} 应当是 {want!r}，实际 {got!r}")  # gate-lint:detail
    if failures:
        print("    → 看 scratch_problem() / source_problem() / plan_copies() / check() / parse_arguments()；"
              f"{BREAK_VARIABLE} 设着的话（项与红的格见文件头）这里本来就该红")
        return 1
    print(f"  ✓ 自检通过（查了 {len(results)} 格）：拷成并照相对仓根的路径落位、清单记 sha256、再拷同样的放行；核对出改动并点名那一份；"
          "拒相对路径、仓内、工作根外的草稿目录，拒仓外、不在的源与目录，拒内容不同的已有目标且原样不动；用法错退 2；平常跑设了弄坏开关退 2，关守卫项与别的项同写也退 2 并点名它")
    return 0


if __name__ == "__main__":
    refused_exit = refuse_break_outside_selftest(sys.argv[1:])
    if refused_exit is not None:
        sys.exit(refused_exit)
    preflight(__file__)
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    sys.exit(run(sys.argv[1:]))
