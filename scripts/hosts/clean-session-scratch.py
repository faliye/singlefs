#!/usr/bin/env python3
# admission: always 判的是此刻草稿根下的目录、归属、git 工作树与在跑的进程，上一次列出来的不替这一次作保
# run-condition: command git du
"""清盘：列（默认）或删（--delete）本会话在工作根与过渡期旧根下不再用的仓副本与编译目录。

为什么：.claude/kb/milestone/03-third-txn.md「待优化」第 55 行——2026-10-18 清盘删 git 工作树前只 git stash push -u，被忽略的文件（规范副本、
multi-host.env、qemu.env）跟着树丢了；删的范围按目录列，没先对 git worktree list 与用户给的排除名单逐个核，用户说不要动的那一个已被删。

用法：
    clean-session-scratch.py [--session <会话号>] [--keep <路径或名字>]… [--delete] [--work-root <目录>] [--tmp-root <目录>] [--repository <仓根>]
    clean-session-scratch.py --selftest      # CLEAN_SESSION_SCRATCH_BREAK=<项> 时点名的格必须红
  会话号依次取 --session、WORK_DIRECTORY_OWNER_SESSION、CLAUDE_CODE_SESSION_ID、CLAUDE_SESSION_ID，都没有就拒绝（退 2）。
  工作根不给取 .claude/hooks/lib_work_root.py 的 work_root()，旧根不给取它的 legacy_root()（/tmp/claude-<uid>），仓根不给取这份脚本所在的仓。
  --keep 可给几次：写绝对路径的，那个路径与它底下的都不碰；写名字的，路径里有一段恰好是这个名字的都不碰；包含保留项的候选父目录也不删。
  不带 --delete 只列、一个字不删。

找什么（只找这两种，草稿目录里的报告、补丁、日志不碰）：仓副本（有 .git，或同时有 Cargo.toml 与 crates/）与编译目录（带签名对得上的 CACHEDIR.TAG）；
  认法与往下找的深度是 scripts/agents/agent-watch.py 的 scratch_hog_kind / find_scratch_hogs（看门狗报「草稿根里超阈值的编译目录与仓副本」用同一份），
  这里按文件路径导入，不另写一份。
  ① 工作根下一层的每个目录：归属是本会话的才往里找——<目录>/.owner（scripts/hosts/work-directory-ownership.sh 写的三行，session 那一行）
     或 <目录>/.writer-lock（派发闸建的写者锁，session_id）是本会话；那个目录自己就是仓副本的，它自己就是一处。
     归属是别的会话、或两样都没有（认不出）的，不往里找，只在末尾数一句各几个。
  ② 旧根下 <旧根>/*/<会话号>/ 那一层（Claude Code 给每个会话的临时目录）：路径里的会话号就是归属，往里找。
每一处的判定（不删的写明为什么）：
  - 先 du -sh 量大小；
  - 有进程在用：/proc/<pid>/fd、cwd、exe 的链接目标落在它底下（只看得到本用户的进程，不按名字匹配），列出进程号，不删；
  - 命中 --keep，不删；
  - git 树（它自己的 .git，或这个仓 git worktree list 里的一棵）：git status --porcelain 有没提交的改动（改了的、没跟踪也没忽略的）不删；
    .git 是文件、又不在这个仓 worktree list 里的（别的仓的工作树）不删；
  - 其余会删。
删（--delete）：逐处删之前对那一处重新判进程占用、排除名单与 Git 状态；先往 <工作根>/sweep-clean-session-scratch-<日期>.log 追加「日期 要删 种类 路径 大小」，写不进就不删；
  git 树先把被忽略的文件（git ls-files -o -i --exclude-standard --directory，去掉编译目录与 target、target-* 两类）拷到
  <工作根>/scratch-kept-<会话号前 8 位>/<日期>-<树名>-<唯一后缀>/ 底下、照原来的相对路径放（那个目录经 work-directory-ownership.sh claim 认领），拷不成就不删；
  这个仓的工作树用 git worktree remove 删（不加 --force），别的用 shutil.rmtree；删完回读：路径还在、或工作树还在 git worktree list 里，记「没删干净」。
退出码：0 列完或删完（没有一处删失败）；1 有删失败的；2 参数不对、取不到会话号或工作根。

弄坏开关（只给 --selftest 证红用；一次设一个）：CLEAN_SESSION_SCRATCH_BREAK=
  ignored-dropped          git 树删之前不拷被忽略的文件            自证「被忽略的文件拷出来了」红
  worktree-rmtree          这个仓的工作树也用 rmtree 删            自证「工作树经 git worktree remove 删、不在 worktree list 里」红
  in-use-ignored           不看 /proc，有进程在用的也删            自证「有进程在用的不删、列出进程号」红
  other-sessions-included  不看归属，工作根下每个目录都往里找       自证「别的会话与认不出归属的不碰」红
  list-deletes             不带 --delete 也照删                    自证「只列时一个字不删」红
  keep-ignored             不认 --keep                             自证「排除名单里的不碰」红
  dirty-deleted            有没提交改动的 git 树也算会删            自证「会删的都删了」红（git worktree remove 不加 --force 拒删有改动的树，这一处记没删成、退 1）
"""
import argparse
import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from datetime import datetime
from zoneinfo import ZoneInfo

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
from preflight import preflight  # noqa: E402

BROKEN = os.environ.get("CLEAN_SESSION_SCRATCH_BREAK", "")
KNOWN_BREAKS = ("ignored-dropped", "worktree-rmtree", "in-use-ignored", "other-sessions-included", "list-deletes", "keep-ignored", "dirty-deleted")
OWNER_FILE = ".owner"
WRITER_LOCK_FILE = ".writer-lock"
OWNERSHIP_SCRIPT = os.path.join(SCRIPT_DIRECTORY, "work-directory-ownership.sh")
SESSION_VARIABLES = ("WORK_DIRECTORY_OWNER_SESSION", "CLAUDE_CODE_SESSION_ID", "CLAUDE_SESSION_ID")
COMPILE_DIRECTORY_KIND = "编译目录"


def load_by_path(name, path):
    specification = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


WORK_ROOTS = load_by_path("lib_work_root_for_clean_session_scratch", os.path.join(REPOSITORY_ROOT, ".claude", "hooks", "lib_work_root.py"))
# 仓副本与编译目录的认法：看门狗那一份（文件头「找什么」），不另写
SCRATCH_FINDER = load_by_path("agent_watch_for_clean_session_scratch", os.path.join(REPOSITORY_ROOT, "scripts", "agents", "agent-watch.py"))


def today():
    return datetime.now(ZoneInfo("Asia/Tokyo")).strftime("%Y-%m-%d")


def owner_of(directory):
    """工作根下一层目录的归属会话号：.owner 的 session 行（三行齐才认），没有再看 .writer-lock 的 session_id；都认不出交 None。"""
    try:
        with open(os.path.join(directory, OWNER_FILE), encoding="utf-8") as handle:
            lines = handle.read().splitlines()
        fields = dict(line.split(": ", 1) for line in lines if ": " in line)
        if len(lines) == 3 and all(fields.get(key) for key in ("session", "created", "purpose")):
            return fields["session"]
    except OSError:
        pass
    try:
        with open(os.path.join(directory, WRITER_LOCK_FILE), encoding="utf-8") as handle:
            return str(json.load(handle).get("session_id") or "") or None
    except (OSError, ValueError, AttributeError):
        return None


def in_use_by(path):
    """开着 path 或它底下文件的进程号（/proc/<pid>/fd、cwd、exe 的链接目标），升序；弄坏开关 in-use-ignored 下一律交空。"""
    if BROKEN == "in-use-ignored":
        return []
    target = os.path.realpath(path)
    users = set()
    for name in os.listdir("/proc"):
        if not name.isdigit() or int(name) == os.getpid():
            continue
        links = [f"/proc/{name}/cwd", f"/proc/{name}/exe"]
        try:
            links += [f"/proc/{name}/fd/{descriptor}" for descriptor in os.listdir(f"/proc/{name}/fd")]
        except OSError:
            pass
        for link in links:
            try:
                pointed = os.readlink(link)
            except OSError:
                continue
            if pointed == target or pointed.startswith(target + "/"):
                users.add(int(name))
                break
    return sorted(users)


def size_of(path):
    measured = subprocess.run(["du", "-sh", "--", path], capture_output=True, text=True)
    return (measured.stdout.split("\t", 1)[0].strip() or "?") if measured.returncode == 0 else "?"


def git(directory, *arguments):
    return subprocess.run(["git", "-C", directory, *arguments], capture_output=True, text=True)


def worktrees_of(repository):
    """这个仓 git worktree list 里除了主工作树之外的每一棵（realpath）。"""
    listed = git(repository, "worktree", "list", "--porcelain")
    paths = [line[len("worktree "):] for line in listed.stdout.splitlines() if line.startswith("worktree ")]
    return {os.path.realpath(path) for path in paths[1:]}


def kept_by_rule(path, keeps):
    if BROKEN == "keep-ignored":
        return None
    real = os.path.realpath(path)
    for keep in keeps:
        if os.path.isabs(keep) or keep.startswith("~"):
            anchor = os.path.realpath(os.path.expanduser(keep))
            if os.path.commonpath((real, anchor)) in (real, anchor):
                return keep
        elif keep in real.split("/"):
            return keep
    names = {keep for keep in keeps if not os.path.isabs(keep) and not keep.startswith("~")}
    if names:
        def unreadable(error):
            raise error
        try:
            for directory, children, files in os.walk(path, onerror=unreadable):
                matched = names.intersection(children) | names.intersection(files)
                if matched:
                    return sorted(matched)[0]
        except OSError:
            # 不能确认子树里没有保留项时，不删整个父目录。
            return "、".join(sorted(names))
    return None


def candidates(session, work_root, tmp_root):
    """(要判的每一处 [(路径, 种类, 归属说明)], 工作根下没往里找的 {说明: 个数})。"""
    found, skipped = [], {}
    roots = []
    for entry in sorted(os.scandir(work_root), key=lambda item: item.name) if os.path.isdir(work_root) else []:
        if not entry.is_dir(follow_symlinks=False):
            continue
        owner = owner_of(entry.path)
        if owner != session and BROKEN != "other-sessions-included":
            label = "认不出归属" if owner is None else "归属别的会话"
            skipped[label] = skipped.get(label, 0) + 1
            continue
        names = {child.name for child in os.scandir(entry.path)}
        kind = SCRATCH_FINDER.scratch_hog_kind(entry.path, names)
        if kind:
            found.append((entry.path, kind, f"工作根下一层，归属 {owner}"))
        else:
            roots.append((entry.path, f"在工作根 {entry.name}/ 里，归属 {owner}"))
    for project in sorted(os.scandir(tmp_root), key=lambda item: item.name) if os.path.isdir(tmp_root) else []:
        session_directory = os.path.join(project.path, session)
        if project.is_dir(follow_symlinks=False) and not os.path.islink(session_directory) and os.path.isdir(session_directory):
            roots.append((session_directory, f"在旧根本会话目录 {project.name}/{session}/ 里"))
    for root, label in roots:
        hogs, _visited, truncated = SCRATCH_FINDER.find_scratch_hogs([root])
        found += [(path, kind, label) for path, kind in hogs]
        if truncated:
            skipped[f"{root} 里目录太多没找完"] = 1
    return found, skipped


def judge(path, kind, keeps, repository_worktrees):
    """→ (会删吗, 理由, git 树的种类：worktree / clone / None)。"""
    users = in_use_by(path)
    if users:
        return False, f"有进程在用（{', '.join(map(str, users))}）", None
    rule = kept_by_rule(path, keeps)
    if rule:
        return False, f"排除名单里有「{rule}」", None
    git_marker = os.path.join(path, ".git")
    tree = None
    if os.path.realpath(path) in repository_worktrees:
        tree = "worktree"
    elif os.path.isdir(git_marker):
        tree = "clone"
    elif os.path.isfile(git_marker):
        return False, "别的仓的 git 工作树（不在这个仓的 git worktree list 里），不碰", None
    if tree and BROKEN != "dirty-deleted":
        status = git(path, "status", "--porcelain")
        if status.returncode != 0:
            return False, f"git status 读不出（{status.stderr.strip()[:120]}）", tree
        if status.stdout.strip():
            return False, f"有没提交的改动 {len(status.stdout.splitlines())} 处（git status --porcelain）", tree
    return True, "没有进程在用、不在排除名单里", tree


def save_ignored(path, kept_root):
    """git 树里被忽略的文件（去掉编译目录与 target、target-*）拷到 kept_root 底下、照相对路径放：→ (拷了的相对路径, 问题或 None)。"""
    listed = git(path, "ls-files", "-z", "-o", "-i", "--exclude-standard", "--directory")
    if listed.returncode != 0:
        return [], f"列不出被忽略的文件：{listed.stderr.strip()[:160]}"
    copied = []
    for relative in [item for item in listed.stdout.split("\0") if item]:
        relative = relative.rstrip("/")
        top = relative.split("/", 1)[0]
        source = os.path.join(path, relative)
        cache = SCRATCH_FINDER.scratch_hog_kind(source, {"CACHEDIR.TAG"}) == COMPILE_DIRECTORY_KIND
        if top == "target" or top.startswith("target-") or cache or "__pycache__" in relative.split("/"):
            continue
        destination = os.path.join(kept_root, relative)
        try:
            os.makedirs(os.path.dirname(destination), exist_ok=True)
            if os.path.isdir(source) and not os.path.islink(source):
                shutil.copytree(source, destination, symlinks=True, dirs_exist_ok=True)
            else:
                shutil.copy2(source, destination, follow_symlinks=False)
        except OSError as error:
            return copied, f"拷不了 {relative}：{error}"
        copied.append(relative)
    return copied, None


def claim_kept_directory(work_root, session):
    """被忽略的文件放哪：<工作根>/scratch-kept-<会话号前 8 位>，经 work-directory-ownership.sh claim 认领（归属文件只在那一份里写）。→ (目录, 问题或 None)"""
    name = f"scratch-kept-{session[:8]}"
    environment = dict(os.environ, WORK_DIRECTORY_OWNER_SESSION=session, SINGLEFS_WORK_ROOT=work_root)
    claimed = subprocess.run(["bash", OWNERSHIP_SCRIPT, "claim", name, "--purpose", "清盘时拷出来的被忽略的文件（scripts/hosts/clean-session-scratch.py）"],
                             capture_output=True, text=True, env=environment)
    if claimed.returncode != 0:
        return None, f"认领不了 {name}：{(claimed.stderr or claimed.stdout).strip()[-200:]}"
    return os.path.join(work_root, name), None


def append_log(log_path, line):
    try:
        with open(log_path, "a", encoding="utf-8") as handle:
            handle.write(line + "\n")
        return True
    except OSError:
        return False


def remove(path, tree, repository):
    """删一处：这个仓的工作树经 git worktree remove，别的 rmtree。→ 问题或 None（删完回读）。"""
    if tree == "worktree" and BROKEN != "worktree-rmtree":
        removed = git(repository, "worktree", "remove", path)
        if removed.returncode != 0:
            return f"git worktree remove 退 {removed.returncode}：{removed.stderr.strip()[:200]}"
    else:
        shutil.rmtree(path, ignore_errors=True)
    if os.path.lexists(path):
        return "删完路径还在"
    if os.path.realpath(path) in worktrees_of(repository):
        return "路径没了，但它还挂在 git worktree list 里（git worktree prune 能清，先看为什么没经 git worktree remove 删）"
    return None


def run(arguments, say=print, *, selftest=False):
    if BROKEN and not selftest:
        say("  ✗ CLEAN_SESSION_SCRATCH_BREAK 只给 --selftest 用，这一次什么都没做")
        say("  → 怎么办：unset CLEAN_SESSION_SCRATCH_BREAK 后再跑")
        return 2
    if BROKEN and BROKEN not in KNOWN_BREAKS:
        say(f"  ✗ CLEAN_SESSION_SCRATCH_BREAK 的「{BROKEN}」不认识（只认 {'、'.join(KNOWN_BREAKS)}）")
        say(f"  → 怎么办：弄坏开关只给自证用，平时不设")
        return 2
    session = arguments.session or next((os.environ[name] for name in SESSION_VARIABLES if os.environ.get(name)), "")
    if not session or session in (".", "..") or "/" in session or "\0" in session or any(character.isspace() for character in session):
        say("  ✗ 取不到会话号：归属认不出是谁的")
        say("  → 怎么办：在 Claude Code 会话的 Bash 里跑（它设 CLAUDE_CODE_SESSION_ID），或加 --session <会话号>")
        return 2
    work_root = os.path.realpath(arguments.work_root or WORK_ROOTS.work_root())
    tmp_root = os.path.realpath(arguments.tmp_root or WORK_ROOTS.legacy_root())
    repository = os.path.realpath(arguments.repository or REPOSITORY_ROOT)
    deleting = arguments.delete or BROKEN == "list-deletes"
    found, skipped = candidates(session, work_root, tmp_root)
    repository_worktrees = worktrees_of(repository)
    judged = []
    for path, kind, label in found:
        will, why, tree = judge(path, kind, arguments.keep, repository_worktrees)
        judged.append((path, kind, label, will, why, tree, size_of(path)))
    say(f"本会话 {session} 的仓副本与编译目录（工作根 {work_root}、旧根 {tmp_root}）：")
    for path, kind, label, will, why, tree, size in judged:
        tree_text = {"worktree": "，这个仓的 git 工作树", "clone": "，带 .git 的克隆"}.get(tree, "")
        say(f"  {'会删' if will else '不删'} {kind} {path}：{size}{tree_text}，{label}；{why}")
    for label, count in sorted(skipped.items()):
        say(f"  ! 工作根下{label}的目录 {count} 个，没往里找（别的会话的由它自己清，认不出归属的交用户看）")
    planned = [item for item in judged if item[3]]
    if not deleting:
        say(f"  ✓ 列了 {len(judged)} 处，会删 {len(planned)} 处、不删 {len(judged) - len(planned)} 处；只列没删（加 --delete 才删）")
        return 0
    log_path = os.path.join(work_root, f"sweep-clean-session-scratch-{today()}.log")
    failures, removed, kept_root = [], [], None
    for path, kind, label, _will, _why, tree, size in planned:
        will, why, tree = judge(path, kind, arguments.keep, worktrees_of(repository))
        if not will:
            say(f"  ! {path} 删之前现查：{why}，这一次不删")
            continue
        if not append_log(log_path, f"{today()}\t要删\t{kind}\t{path}\t{size}"):
            failures.append(f"{path}：写不进清盘日志 {log_path}，没删")
            continue
        if tree and BROKEN != "ignored-dropped":
            if kept_root is None:
                kept_root, problem = claim_kept_directory(work_root, session)
                if problem:
                    failures.append(f"{path}：{problem}，没删")
                    kept_root = None
                    continue
            try:
                destination = tempfile.mkdtemp(prefix=f"{today()}-{os.path.basename(path)}-", dir=kept_root)
            except OSError as error:
                failures.append(f"{path}：建不了独立备份目录：{error}，没删")
                continue
            copied, problem = save_ignored(path, destination)
            if problem:
                failures.append(f"{path}：被忽略的文件{problem}，没删")
                continue
            if copied:
                say(f"  被忽略的文件 {len(copied)} 项拷到 {destination}：{'、'.join(copied[:12])}{' …' if len(copied) > 12 else ''}")
        problem = remove(path, tree, repository)
        append_log(log_path, f"{today()}\t{'删了' if problem is None else '没删干净'}\t{kind}\t{path}")
        if problem:
            failures.append(f"{path}：{problem}")
        else:
            removed.append(path)
            say(f"  删了 {kind} {path}（{size}）")
    for failure in failures:
        say(f"  ✗ {failure}")  # gate-lint:detail
    if failures:
        say(f"  ✗ 会删的 {len(planned)} 处里 {len(failures)} 处没删成（逐处列在上面）")  # gate-lint:summary
        say(f"  → 怎么办：照每处的原因处理（进程在用的等它退、拷不出的看权限与空间），再跑一次；清盘日志 {log_path}")
        return 1
    say(f"  ✓ 删了 {len(removed)} 处（会删的 {len(planned)} 处），清盘日志 {log_path}")
    return 0


def parse(argv):
    parser = argparse.ArgumentParser(description="列或删本会话不再用的仓副本与编译目录")
    parser.add_argument("--session")
    parser.add_argument("--keep", action="append", default=[])
    parser.add_argument("--delete", action="store_true")
    parser.add_argument("--work-root")
    parser.add_argument("--tmp-root")
    parser.add_argument("--repository")
    return parser.parse_args(argv)


def selftest():
    """临时目录当工作根、旧根与主仓，造本会话的工作树、克隆、副本、编译目录与几处不该碰的，先只列、再删，逐格判。"""
    session, other = "selftest-session-0001", "selftest-session-0002"
    work = tempfile.mkdtemp(prefix="clean-session-scratch-selftest-")
    failures = []
    holder = None
    try:
        work_root, tmp_root, repository = (os.path.join(work, name) for name in ("work", "tmp", "repository"))
        for directory in (work_root, tmp_root, repository):
            os.makedirs(directory)

        def write(path, text):
            os.makedirs(os.path.dirname(path), exist_ok=True)
            with open(path, "w", encoding="utf-8") as handle:
                handle.write(text)

        def own(directory, owner):
            write(os.path.join(directory, OWNER_FILE), f"session: {owner}\ncreated: 2026-01-01\npurpose: 自证\n")

        def copy_at(path):
            write(os.path.join(path, "Cargo.toml"), "[workspace]\n")
            os.makedirs(os.path.join(path, "crates"), exist_ok=True)

        write(os.path.join(repository, ".gitignore"), "multi-host.env\ntarget/\n")
        write(os.path.join(repository, "README.md"), "样本仓\n")
        for command in (["init", "-q"], ["add", "-A"], ["-c", "user.name=s", "-c", "user.email=s@s", "commit", "-qm", "base"]):
            if git(repository, *command).returncode != 0:
                raise RuntimeError(f"样本仓 git {command} 没成")
        mine, theirs, keepme, unowned = (os.path.join(work_root, name) for name in ("mine", "theirs", "keepme", "unowned"))
        for directory, owner in ((mine, session), (theirs, other), (keepme, session)):
            os.makedirs(directory)
            own(directory, owner)
        os.makedirs(unowned)
        tree, dirty = os.path.join(mine, "tree"), os.path.join(mine, "dirty")
        for path in (tree, dirty):
            if git(repository, "worktree", "add", "-q", "--detach", path).returncode != 0:
                raise RuntimeError(f"样本工作树 {path} 没建成")
        write(os.path.join(tree, "multi-host.env"), "PEER=样本\n")
        write(os.path.join(tree, "target", "CACHEDIR.TAG"), "Signature: 8a477f597d28d172789f06886806bc55\n")
        write(os.path.join(dirty, "untracked.txt"), "没提交\n")
        copy_at(os.path.join(mine, "copy"))
        write(os.path.join(mine, "target-x", "CACHEDIR.TAG"), "Signature: 8a477f597d28d172789f06886806bc55\n")
        write(os.path.join(mine, "report.md"), "报告不碰\n")
        busy = os.path.join(mine, "busy")
        copy_at(busy)
        copy_at(os.path.join(theirs, "copy"))
        copy_at(os.path.join(keepme, "copy"))
        copy_at(os.path.join(unowned, "copy"))
        copy_at(os.path.join(tmp_root, "project", session, "copy"))
        copy_at(os.path.join(tmp_root, "project", other, "copy"))
        holder = subprocess.Popen(["sleep", "120"], cwd=busy)
        time.sleep(0.2)
        environment_backup = os.environ.get("SINGLEFS_WORK_ROOT")
        os.environ["SINGLEFS_WORK_ROOT"] = work_root
        common = ["--session", session, "--work-root", work_root, "--tmp-root", tmp_root, "--repository", repository, "--keep", "keepme"]

        def invoke(extra):
            said = []
            code = run(parse(common + extra), say=said.append, selftest=True)
            return code, "\n".join(said)

        expected_deleted = [tree, os.path.join(mine, "copy"), os.path.join(mine, "target-x"), os.path.join(tmp_root, "project", session, "copy")]
        untouched = [dirty, busy, os.path.join(mine, "report.md"), os.path.join(theirs, "copy"), os.path.join(keepme, "copy"),
                     os.path.join(unowned, "copy"), os.path.join(tmp_root, "project", other, "copy")]
        code, listed = invoke([])
        if code != 0 or not all(os.path.lexists(path) for path in expected_deleted + untouched) or listed.count("  会删 ") != 4:
            failures.append(("只列时一个字不删", f"退 {code}，会删 {listed.count('  会删 ')} 处，输出：{listed}"))
        code, deleted = invoke(["--delete"])
        gone = [path for path in expected_deleted if os.path.lexists(path)]
        if code != 0 or gone:
            failures.append(("会删的都删了", f"退 {code}，还在 {gone}，输出：{deleted}"))
        if os.path.realpath(tree) in worktrees_of(repository):
            failures.append(("工作树经 git worktree remove 删、不在 worktree list 里", git(repository, "worktree", "list").stdout))
        import glob
        backups = glob.glob(os.path.join(work_root, f"scratch-kept-{session[:8]}", f"{today()}-tree-*", "multi-host.env"))
        kept = backups[0] if len(backups) == 1 else ""
        if not os.path.isfile(kept) or os.path.lexists(os.path.join(os.path.dirname(kept), "target")):
            failures.append(("被忽略的文件拷出来了", f"{kept} 应当在、target/ 不拷，输出：{deleted}"))
        if not os.path.lexists(busy) or f"有进程在用（{holder.pid}）" not in deleted:
            failures.append(("有进程在用的不删、列出进程号", deleted))
        for path, cell in ((os.path.join(theirs, "copy"), "别的会话与认不出归属的不碰"), (os.path.join(unowned, "copy"), "别的会话与认不出归属的不碰"),
                           (os.path.join(tmp_root, "project", other, "copy"), "别的会话与认不出归属的不碰"),
                           (os.path.join(keepme, "copy"), "排除名单里的不碰"), (dirty, "有没提交改动的工作树不删"),
                           (os.path.join(mine, "report.md"), "草稿目录里的报告不碰")):
            if not os.path.lexists(path):
                failures.append((cell, f"{path} 被删了，输出：{deleted}"))
        if environment_backup is None:
            os.environ.pop("SINGLEFS_WORK_ROOT", None)
        else:
            os.environ["SINGLEFS_WORK_ROOT"] = environment_backup
    finally:
        if holder is not None:
            holder.kill()
            holder.wait()
        shutil.rmtree(work, ignore_errors=True)
    cells = ("只列时一个字不删", "会删的都删了", "工作树经 git worktree remove 删、不在 worktree list 里", "被忽略的文件拷出来了",
             "有进程在用的不删、列出进程号", "别的会话与认不出归属的不碰", "排除名单里的不碰", "有没提交改动的工作树不删", "草稿目录里的报告不碰")
    for cell, failure in failures:
        print(f"  ✗ 自证 {cell}：{failure}")  # gate-lint:detail
    if failures:
        print(f"  ✗ 自证 {len(cells)} 格里 {len({cell for cell, _ in failures})} 格判错")  # gate-lint:summary
        print("  → 怎么办：看 candidates / judge / save_ignored / remove 的判法；CLEAN_SESSION_SCRATCH_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ clean-session-scratch 自证通过（{len(cells)} 格）：只列不删；本会话的工作树经 git worktree remove 删、被忽略的文件拷出来（target 不拷），"
          "克隆外的副本、编译目录与旧根本会话目录里的副本删了；有进程在用的、排除名单里的、有没提交改动的、别的会话与认不出归属的、草稿里的报告都不碰")
    return 0


if __name__ == "__main__":
    preflight(__file__)
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    sys.exit(run(parse(sys.argv[1:])))
