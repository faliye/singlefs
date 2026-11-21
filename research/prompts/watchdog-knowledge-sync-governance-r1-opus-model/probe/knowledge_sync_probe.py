#!/usr/bin/env python3
"""攻方腿探针：看门狗定时回报里「知识同步提醒」一节在临时小仓上的行为。

用法：
  knowledge_sync_probe.py <agent-watch.py 的路径> <运行目录>
      在 <运行目录> 底下逐个场景建小仓，直接调 agent-watch.py 里的 take_repository_baseline 与 repository_status_section
      （与看门狗起来时、到点时调的是同一对函数），中间照场景做 git 操作；每个场景打一行 PROBE name=<场景> …，
      再把这一节的原样各行以「  | 」起头打出来；末行 PROBE name=done emitted=<前面 PROBE 行的条数>。
  knowledge_sync_probe.py --fifo-child <agent-watch.py 的路径> <小仓>
      只给本探针自己起：在小仓上记一次基线，记完打 fifo-child-finished（目录里有命名管道时预期卡住）。
只碰 <运行目录> 底下自己建的小仓：git 带 GIT_CEILING_DIRECTORIES=<运行目录>，不往上找到别的仓；
小仓里的 git 一律不读用户与系统配置（GIT_CONFIG_GLOBAL=/dev/null、GIT_CONFIG_NOSYSTEM=1），只有三个配置场景故意给一份自造的全局配置。
退出码：0 全部场景跑完（被判对象自己抛的异常算读数，写进那一行的 exception=）；1 探针自己搭场景失败；2 用法错。
"""
import argparse
import hashlib
import importlib.util
import json
import os
import re
import signal
import subprocess
import sys
import time

NEUTRAL_GIT_CONFIG = {"GIT_CONFIG_GLOBAL": os.devnull, "GIT_CONFIG_NOSYSTEM": "1"}
GIT_IDENTITY = ["-c", "user.name=probe", "-c", "user.email=probe@example.invalid", "-c", "commit.gpgsign=false", "-c", f"core.hooksPath={os.devnull}"]
FIFO_CHILD_SECONDS = 20          # 命名管道那一格等子进程记完基线的上限
SPARSE_IMAGE_BYTES = 2 << 30     # 稀疏镜像那一格的文件大小（2 GiB，不占盘）
emitted_lines = 0
setup_failures = []


def emit(name, **fields):
    global emitted_lines
    emitted_lines += 1
    text = " ".join(f"{key}={value}" for key, value in fields.items())
    print(f"PROBE name={name} {text}", flush=True)


def show(lines):
    for line in lines:
        print(f"  | {line}", flush=True)


def load_watchdog(path):
    spec = importlib.util.spec_from_file_location("agent_watch_under_probe", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def git(repository, *arguments, check=True):
    environment = dict(os.environ, **NEUTRAL_GIT_CONFIG)
    finished = subprocess.run(["git", *GIT_IDENTITY, "-C", repository, *arguments], capture_output=True, text=True,
                              errors="replace", env=environment, timeout=60)
    if check and finished.returncode != 0:
        raise RuntimeError(f"git {' '.join(arguments)} 退 {finished.returncode}：{finished.stderr.strip()}")
    return finished


def write_file(repository, relative_path, text):
    path = os.path.join(repository, relative_path)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)


def new_repository(run_directory, name, files):
    """建一个小仓：分支 main，files 一次提交成起点。"""
    repository = os.path.join(run_directory, name)
    os.makedirs(repository)
    git(repository, "init", "-q")
    git(repository, "symbolic-ref", "HEAD", "refs/heads/main")
    for relative_path, text in files.items():
        write_file(repository, relative_path, text)
    git(repository, "add", "-A")
    git(repository, "commit", "-q", "-m", "起点")
    return repository


def commit_paths(repository, title, files=None, removed=()):
    """只把点名的路径提交成一条提交。"""
    for relative_path, text in (files or {}).items():
        write_file(repository, relative_path, text)
    for relative_path in removed:
        git(repository, "rm", "-q", "--", relative_path)
    touched = [*(files or {}), *removed]
    if files:
        git(repository, "add", "--", *files)
    git(repository, "commit", "-q", "-m", title, "--", *touched)


def section_arguments(repository, limit=20):
    return argparse.Namespace(repository_report_lines=limit, repository_root=repository, max_minutes=60)


def baseline_of(watchdog, repository, limit=20):
    return watchdog.take_repository_baseline(section_arguments(repository, limit))


def section_of(watchdog, repository, baseline, limit=20):
    return watchdog.repository_status_section(baseline, section_arguments(repository, limit))


def named_files(watchdog, lines):
    """这一节逐份点名的知识类文件 → [(路径, 来源标签)]。"""
    tags = "|".join(re.escape(tag) for tag in (watchdog.COMMITTED_KNOWLEDGE_TAG, watchdog.UNCOMMITTED_KNOWLEDGE_TAG,
                                               watchdog.BOTH_KNOWLEDGE_TAG, watchdog.DROPPED_KNOWLEDGE_TAG))
    pattern = re.compile(rf"^    (\S.*)（({tags})）$")
    return [match.groups() for match in (pattern.match(line) for line in lines) if match]


def other_path_rows(watchdog, lines):
    kinds = (watchdog.APPEARED_PATH_KIND, watchdog.CHANGED_PATH_KIND, watchdog.VANISHED_PATH_KIND)
    return [line.strip() for line in lines if any(line.startswith(f"    {kind} [") for kind in kinds)]


def reminder_state(lines):
    first = lines[0] if lines else ""
    if "没查成" in first:
        return "unreadable"
    found = re.search(r"知识类文件变过 (\d+) 份", first)
    return f"changed-{found.group(1)}" if found else ("unchanged" if "知识类文件没变" in first else "absent")


def report(watchdog, name, lines, expectation, **extra):
    """一个场景的读数行：点名了哪些、别的路径列了哪些、提醒句怎么说，以及这一格预期的读法（expectation 只是写给读的人看的）。"""
    named = named_files(watchdog, lines)
    emit(name, reminder=reminder_state(lines), named_count=len(named), other_count=len(other_path_rows(watchdog, lines)),
         **extra, expectation=json.dumps(expectation, ensure_ascii=False),
         named=json.dumps(named, ensure_ascii=False), other=json.dumps(other_path_rows(watchdog, lines), ensure_ascii=False))
    show(lines)


# ── 第 5 问：漏点名 ──
def scenario_space_untracked_new(watchdog, run):
    repository = new_repository(run, "space-untracked-new", {"CLAUDE.md": "说明\n"})
    baseline = baseline_of(watchdog, repository)
    write_file(repository, ".claude/kb/a b.md", "新知识页\n")
    report(watchdog, "space-untracked-new", section_of(watchdog, repository, baseline),
           "看门狗起来之后新建一份路径带空格的知识类文件 .claude/kb/a b.md：git status --short 给这一行加引号，前缀对不上，点不点名")


def scenario_space_tracked_rewritten(watchdog, run):
    repository = new_repository(run, "space-tracked-rewritten", {".claude/kb/a b.md": "原文\n"})
    write_file(repository, ".claude/kb/a b.md", "起来时已改\n")
    baseline = baseline_of(watchdog, repository)
    write_file(repository, ".claude/kb/a b.md", "起来时已改\n之后又改\n")
    report(watchdog, "space-tracked-rewritten", section_of(watchdog, repository, baseline),
           "起来时已改着、之后又改了内容（状态码不变）的 .claude/kb/a b.md：范围外只比状态码，点不点名、别的路径里有没有")


def scenario_quote_committed(watchdog, run):
    repository = new_repository(run, "quote-committed", {"CLAUDE.md": "说明\n"})
    baseline = baseline_of(watchdog, repository)
    commit_paths(repository, "加一份文件名带双引号的规则", {'.claude/rules/say"hi".md': "规则\n"})
    report(watchdog, "quote-committed", section_of(watchdog, repository, baseline),
           "新提交加了一份文件名带双引号的规则：git diff --name-only 给它加引号，点不点名")


def scenario_space_committed(watchdog, run):
    repository = new_repository(run, "space-committed", {"CLAUDE.md": "说明\n"})
    baseline = baseline_of(watchdog, repository)
    commit_paths(repository, "加一份路径带空格的知识页", {".claude/kb/c d.md": "知识页\n"})
    report(watchdog, "space-committed", section_of(watchdog, repository, baseline),
           "新提交加了一份路径带空格的知识页：git diff --name-only 不给空格加引号的话照常点名")


def scenario_rename_in_to_out(watchdog, run):
    repository = new_repository(run, "rename-in-to-out", {".claude/rules/x.md": "规则\n" * 20})
    baseline = baseline_of(watchdog, repository)
    os.makedirs(os.path.join(repository, "docs"), exist_ok=True)
    git(repository, "mv", ".claude/rules/x.md", "docs/x.md")
    report(watchdog, "rename-in-to-out", section_of(watchdog, repository, baseline),
           "已暂存的改名，旧路径在范围里、新路径在范围外：点名旧路径")


def scenario_rename_out_to_in(watchdog, run):
    repository = new_repository(run, "rename-out-to-in", {"docs/y.md": "说明\n" * 20})
    baseline = baseline_of(watchdog, repository)
    os.makedirs(os.path.join(repository, ".claude/rules"), exist_ok=True)
    git(repository, "mv", "docs/y.md", ".claude/rules/y.md")
    report(watchdog, "rename-out-to-in", section_of(watchdog, repository, baseline),
           "已暂存的改名，旧路径在范围外、新路径在范围里：点名新路径")


def scenario_ignored_parent(watchdog, run):
    """照真仓 .gitignore 的「/.claude/singlefs-ai-sop/」一行：范围条目 .claude/singlefs-ai-sop/rules/ 落在被忽略的目录底下。"""
    repository = new_repository(run, "ignored-parent", {".gitignore": "/.claude/singlefs-ai-sop/\n", "CLAUDE.md": "说明\n"})
    write_file(repository, ".claude/singlefs-ai-sop/rules/sop-first.md", "上游规则原文\n")
    baseline = baseline_of(watchdog, repository)
    write_file(repository, ".claude/singlefs-ai-sop/rules/sop-first.md", "上游规则改了\n")
    write_file(repository, ".claude/singlefs-ai-sop/rules/new-rule.md", "上游新规则\n")
    report(watchdog, "ignored-parent", section_of(watchdog, repository, baseline),
           "被忽略的目录底下的上游规则改了一份、新加一份：git status 与 git diff 都看不见，点不点名、别的路径里有没有")


def scenario_modified_commit_modify(watchdog, run):
    repository = new_repository(run, "modified-commit-modify", {".claude/kb/k.md": "原文\n"})
    write_file(repository, ".claude/kb/k.md", "起来时已改\n")
    baseline = baseline_of(watchdog, repository)
    commit_paths(repository, "提交起来时的改动", {".claude/kb/k.md": "起来时已改\n"})
    write_file(repository, ".claude/kb/k.md", "起来时已改\n提交之后又改\n")
    report(watchdog, "modified-commit-modify", section_of(watchdog, repository, baseline), "起来时改着、之后提交了又接着改：两样都有")


def scenario_stash_then_pop(watchdog, run):
    repository = new_repository(run, "stash-then-pop", {".claude/kb/k.md": "原文\n"})
    write_file(repository, ".claude/kb/k.md", "起来时已改\n")
    baseline = baseline_of(watchdog, repository)
    git(repository, "stash", "-q")
    git(repository, "stash", "pop", "-q")
    report(watchdog, "stash-then-pop", section_of(watchdog, repository, baseline), "git stash 再 pop，工作区内容回到起来时：不点名")


def scenario_stash_only(watchdog, run):
    repository = new_repository(run, "stash-only", {".claude/kb/k.md": "原文\n"})
    write_file(repository, ".claude/kb/k.md", "起来时已改\n")
    baseline = baseline_of(watchdog, repository)
    git(repository, "stash", "-q")
    report(watchdog, "stash-only", section_of(watchdog, repository, baseline), "git stash 收走起来时的改动：标已不在")


def scenario_reset_soft(watchdog, run):
    repository = new_repository(run, "reset-soft", {"CLAUDE.md": "说明\n"})
    commit_paths(repository, "改一份知识页", {".claude/kb/k.md": "知识页\n"})
    baseline = baseline_of(watchdog, repository)
    git(repository, "reset", "-q", "--soft", "HEAD~1")
    report(watchdog, "reset-soft", section_of(watchdog, repository, baseline),
           "git reset --soft HEAD~1：工作区与索引里的内容一字没变，只是 HEAD 往回挪了一条，点不点名、标什么")


def scenario_branch_roundtrip(watchdog, run):
    repository = new_repository(run, "branch-roundtrip", {"CLAUDE.md": "说明\n"})
    git(repository, "checkout", "-q", "-b", "other")
    commit_paths(repository, "别的分支上改规则", {".claude/rules/z.md": "规则\n"})
    git(repository, "checkout", "-q", "main")
    baseline = baseline_of(watchdog, repository)
    git(repository, "checkout", "-q", "other")
    git(repository, "checkout", "-q", "main")
    report(watchdog, "branch-roundtrip", section_of(watchdog, repository, baseline), "换到别的分支再换回来：不点名")


def scenario_branch_switch_away(watchdog, run):
    repository = new_repository(run, "branch-switch-away", {"CLAUDE.md": "说明\n"})
    git(repository, "checkout", "-q", "-b", "other")
    commit_paths(repository, "别的分支上的旧提交", {".claude/rules/z.md": "规则\n"})
    git(repository, "checkout", "-q", "main")
    commit_paths(repository, "main 上的提交", {"a.txt": "a\n"})
    baseline = baseline_of(watchdog, repository)
    git(repository, "checkout", "-q", "other")
    report(watchdog, "branch-switch-away", section_of(watchdog, repository, baseline),
           "起来之后换到一个分叉的分支：那条分支上起来之前就有的提交算不算「看门狗起来之后的新提交」")


def scenario_commit_deletes(watchdog, run):
    repository = new_repository(run, "commit-deletes", {".claude/rules/old.md": "旧规则\n"})
    baseline = baseline_of(watchdog, repository)
    commit_paths(repository, "删掉一份规则", removed=(".claude/rules/old.md",))
    report(watchdog, "commit-deletes", section_of(watchdog, repository, baseline), "新提交删掉一份规则：点名，标签与出路有没有说它已删")


def scenario_untracked_new_directory(watchdog, run):
    repository = new_repository(run, "untracked-new-directory", {".claude/skills/old/SKILL.md": "旧技能\n"})
    baseline = baseline_of(watchdog, repository)
    write_file(repository, ".claude/skills/new/SKILL.md", "新技能\n")
    report(watchdog, "untracked-new-directory", section_of(watchdog, repository, baseline),
           "起来之后新建一整个未跟踪的技能目录：git status 写成一行「?? .claude/skills/new/」，点名的是目录还是底下的文件")


def scenario_untracked_directory_grows(watchdog, run):
    repository = new_repository(run, "untracked-directory-grows", {"CLAUDE.md": "说明\n"})
    write_file(repository, ".claude/kb/drafts/notes.md", "草稿\n")
    baseline = baseline_of(watchdog, repository)
    write_file(repository, ".claude/kb/drafts/second-note.md", "又一份草稿\n")
    report(watchdog, "untracked-directory-grows", section_of(watchdog, repository, baseline),
           "起来时就未跟踪的目录底下又多一份：状态码不变、目录摘要变了，点名目录")


# ── 第 6 问：误点名与噪声 ──
def scenario_stage_only(watchdog, run):
    repository = new_repository(run, "stage-only", {".claude/kb/k.md": "原文\n"})
    write_file(repository, ".claude/kb/k.md", "起来时已改\n")
    baseline = baseline_of(watchdog, repository)
    git(repository, "add", "--", ".claude/kb/k.md")
    report(watchdog, "stage-only", section_of(watchdog, repository, baseline),
           "别的会话只是把起来时的改动暂存了（内容一字没变）：点不点名")


def scenario_chmod_only(watchdog, run):
    repository = new_repository(run, "chmod-only", {".claude/kb/k.md": "原文\n"})
    baseline = baseline_of(watchdog, repository)
    os.chmod(os.path.join(repository, ".claude/kb/k.md"), 0o755)
    report(watchdog, "chmod-only", section_of(watchdog, repository, baseline), "只改了执行位（内容一字没变）：点不点名")


def scenario_ignored_inside_untracked(watchdog, run):
    repository = new_repository(run, "ignored-inside-untracked", {".gitignore": "*.img\n__pycache__/\n", "CLAUDE.md": "说明\n"})
    write_file(repository, ".claude/kb/drafts/notes.md", "草稿\n")
    write_file(repository, ".claude/kb/drafts/cache.img", "被忽略的镜像，起来时\n")
    baseline = baseline_of(watchdog, repository)
    write_file(repository, ".claude/kb/drafts/cache.img", "被忽略的镜像，之后变了\n")
    report(watchdog, "ignored-inside-untracked", section_of(watchdog, repository, baseline),
           "未跟踪的知识类目录里只有一份被 .gitignore 忽略的文件变了（git 看不见它）：点不点名")


def scenario_line_bound(watchdog, run):
    limit = 20
    repository = new_repository(run, "line-bound", {"CLAUDE.md": "说明\n"})
    baseline = baseline_of(watchdog, repository, limit)
    long_title = "很长的提交标题" * 30
    for index in range(30):
        commit_paths(repository, f"{long_title} {index}", {f".claude/kb/page-{index:02d}.md": "页\n"})
    for index in range(30):
        write_file(repository, f"other-file-{index:02d}.txt", "别的\n")   # 放仓根：放进一个新目录会被 git status 并成一行
    lines = section_of(watchdog, repository, baseline, limit)
    report(watchdog, "line-bound", lines, "三张清单各 30 条、上限 20：这一节一共几行，最长一行几个字",
           lines=len(lines), bound=3 * limit + 10, widest=max(len(line) for line in lines))


# ── 第 7 问：拖慢与弄挂 ──
def scenario_git_count(watchdog, run):
    repository = new_repository(run, "git-count", {"CLAUDE.md": "说明\n"})
    original = watchdog.run_git
    calls = []

    def counting(root, git_arguments):
        calls.append(git_arguments[0])
        return original(root, git_arguments)
    watchdog.run_git = counting
    try:
        baseline = baseline_of(watchdog, repository)
        at_baseline = list(calls)
        calls.clear()
        section_of(watchdog, repository, baseline)
        at_tick_still = list(calls)
        commit_paths(repository, "往前走一条", {"a.txt": "a\n"})
        calls.clear()
        lines = section_of(watchdog, repository, baseline)
        at_tick_moved = list(calls)
    finally:
        watchdog.run_git = original
    report(watchdog, "git-count", lines, "基线与到点各起几条 git", baseline_git=len(at_baseline), tick_git_head_still=len(at_tick_still),
           tick_git_head_moved=len(at_tick_moved), commands=json.dumps([at_baseline, at_tick_still, at_tick_moved]))


def scenario_index_lock(watchdog, run):
    repository = new_repository(run, "index-lock", {".claude/kb/k.md": "原文\n"})
    baseline = baseline_of(watchdog, repository)
    write_file(repository, ".claude/kb/k.md", "改了\n")
    with open(os.path.join(repository, ".git", "index.lock"), "w", encoding="utf-8") as handle:
        handle.write("")   # 别的会话正在提交时占着的那把锁
    lines = section_of(watchdog, repository, baseline)
    os.remove(os.path.join(repository, ".git", "index.lock"))
    report(watchdog, "index-lock", lines, "索引锁被别的会话占着时到点：照常读出还是没查成")


def scenario_sparse_ignored_image(watchdog, run):
    repository = new_repository(run, "sparse-ignored-image", {".gitignore": "*.img\n", "CLAUDE.md": "说明\n"})
    write_file(repository, ".claude/kb/drafts/notes.md", "草稿\n")
    image = os.path.join(repository, ".claude/kb/drafts/disk.img")
    with open(image, "wb") as handle:
        handle.truncate(SPARSE_IMAGE_BYTES)   # 稀疏：不占盘，读出来是全零
    started = time.monotonic()
    baseline = baseline_of(watchdog, repository)
    baseline_seconds = time.monotonic() - started
    started = time.monotonic()
    lines = section_of(watchdog, repository, baseline)
    tick_seconds = time.monotonic() - started
    os.remove(image)
    report(watchdog, "sparse-ignored-image", lines, "未跟踪的知识类目录里有一份被忽略的 2 GiB 稀疏镜像：基线与到点各花几秒（git 的 30 秒时限管不到算摘要）",
           image_gibibytes=SPARSE_IMAGE_BYTES / (1 << 30), baseline_seconds=f"{baseline_seconds:.2f}", tick_seconds=f"{tick_seconds:.2f}")


def scenario_fifo_in_untracked_directory(watchdog_path, watchdog, run):
    repository = new_repository(run, "fifo-in-untracked-directory", {"CLAUDE.md": "说明\n"})
    write_file(repository, ".claude/kb/pipes/notes.md", "草稿\n")
    os.mkfifo(os.path.join(repository, ".claude/kb/pipes/channel"))
    child = subprocess.Popen([sys.executable, os.path.abspath(__file__), "--fifo-child", watchdog_path, repository],
                             stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, errors="replace", start_new_session=True)
    try:
        output, error_output = child.communicate(timeout=FIFO_CHILD_SECONDS)
        hung = "no"
    except subprocess.TimeoutExpired:
        hung = "yes"
        os.killpg(child.pid, signal.SIGTERM)   # 只停自己用新会话起的那一组
        try:
            output, error_output = child.communicate(timeout=5)
        except subprocess.TimeoutExpired:
            os.killpg(child.pid, signal.SIGKILL)
            output, error_output = child.communicate()
    try:
        os.killpg(child.pid, 0)
        left = "yes"
    except ProcessLookupError:
        left = "no"
    emit("fifo-in-untracked-directory", hung=hung, waited_seconds=FIFO_CHILD_SECONDS, child_exit=child.returncode, group_left=left,
         expectation=json.dumps("未跟踪的知识类目录里有一个命名管道（外加一份普通文件，git 才把这个目录列出来）：记基线时卡不卡住", ensure_ascii=False),
         child_output=json.dumps((output or "")[-200:] + (error_output or "")[-300:], ensure_ascii=False))


def fifo_child(watchdog_path, repository):
    os.environ.update(NEUTRAL_GIT_CONFIG)
    watchdog = load_watchdog(watchdog_path)
    baseline_of(watchdog, repository)
    print("fifo-child-finished", flush=True)
    return 0


def scenario_symlink_outside(watchdog, run):
    outside = os.path.join(run, "outside-target")
    os.makedirs(outside)
    write_file(outside, "linked.md", "仓外原文\n")
    repository = new_repository(run, "symlink-outside", {"CLAUDE.md": "说明\n"})
    write_file(repository, ".claude/kb/links/notes.md", "草稿\n")
    os.symlink(outside, os.path.join(repository, ".claude/kb/links/outside-directory"))
    os.symlink(os.path.join(outside, "linked.md"), os.path.join(repository, ".claude/kb/links/outside-file.md"))
    baseline = baseline_of(watchdog, repository)
    write_file(outside, "linked.md", "仓外改了\n")
    write_file(outside, "added.md", "仓外新加\n")
    report(watchdog, "symlink-outside", section_of(watchdog, repository, baseline),
           "未跟踪的知识类目录里有指到仓外的目录链接与文件链接，仓外那一头改了：点不点名（不跟链接，读不到）")


# ── 第 9 问：读哪个仓、继承的环境与配置 ──
def scenario_root_precedence(watchdog, run):
    """只调 repository_root_of 看它挑哪个仓，不在挑出来的仓上起 git（缺省那一档挑的是本脚本所在的真仓）。"""
    repository = new_repository(run, "root-precedence", {"CLAUDE.md": "说明\n"})
    saved = os.environ.get("TASK_SERVER_SOURCE_REPOSITORY")
    picks = {}
    try:
        os.environ["TASK_SERVER_SOURCE_REPOSITORY"] = repository
        picks["source-set"] = watchdog.repository_root_of(argparse.Namespace(repository_root=None))
        picks["explicit-wins"] = watchdog.repository_root_of(argparse.Namespace(repository_root=run))
        os.environ.pop("TASK_SERVER_SOURCE_REPOSITORY")
        picks["source-unset"] = watchdog.repository_root_of(argparse.Namespace(repository_root=None))
        os.environ["TASK_SERVER_SOURCE_REPOSITORY"] = "relative/repository"
        relative = watchdog.take_repository_baseline(argparse.Namespace(repository_report_lines=20, repository_root=None, max_minutes=60))
        picks["source-relative"] = f"root={relative.root} snapshot={'None' if relative.snapshot is None else 'taken'} reason={relative.unreadable_reason}"
    finally:
        if saved is None:
            os.environ.pop("TASK_SERVER_SOURCE_REPOSITORY", None)
        else:
            os.environ["TASK_SERVER_SOURCE_REPOSITORY"] = saved
    emit("root-precedence", script_repository=watchdog.REPOSITORY_ROOT, small_repository=repository,
         expectation=json.dumps("--repository-root 先于 TASK_SERVER_SOURCE_REPOSITORY 先于本脚本所在的仓；来源仓根是相对路径时怎么记", ensure_ascii=False),
         picks=json.dumps(picks, ensure_ascii=False))


def scenario_worktree_inherited_variables(watchdog, run):
    repository = new_repository(run, "worktree-main", {".claude/kb/k.md": "原文\n"})
    other = new_repository(run, "worktree-other-repository", {"x.txt": "x\n"})
    worktree = os.path.join(run, "worktree-linked")
    git(repository, "worktree", "add", "-q", "-b", "linked", worktree)
    readings = {}
    saved = {name: os.environ.get(name) for name in ("GIT_DIR", "GIT_COMMON_DIR")}
    try:
        os.environ["GIT_DIR"] = os.path.join(other, ".git")   # 从别的 git 命令里继承来的、指到别的仓的变量
        baseline = baseline_of(watchdog, worktree)
        write_file(worktree, ".claude/kb/k.md", "工作树里改了\n")
        lines = section_of(watchdog, worktree, baseline)
        readings["git-dir-inherited"] = {"branch": None if baseline.snapshot is None else baseline.snapshot.branch, "reason": baseline.unreadable_reason,
                                         "named": named_files(watchdog, lines)}
        os.environ.pop("GIT_DIR")
        os.environ["GIT_COMMON_DIR"] = os.path.join(other, ".git")
        common = baseline_of(watchdog, worktree)
        readings["git-common-dir-inherited"] = {"branch": None if common.snapshot is None else common.snapshot.branch, "reason": common.unreadable_reason}
    finally:
        for name, value in saved.items():
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value
    emit("worktree-inherited-variables", expectation=json.dumps("git worktree 里起，环境里继承了指到别的仓的 GIT_DIR（去掉了）与 GIT_COMMON_DIR（没去掉）", ensure_ascii=False),
         readings=json.dumps(readings, ensure_ascii=False))


def scenario_global_config(watchdog, run, label, config_text, action, expectation):
    """run_git 不关用户的全局配置（自证里关了）：给一份自造的全局配置，看这一节读成什么。"""
    repository = new_repository(run, f"global-config-{label}", {".claude/kb/k.md": "原文\n"})
    config = os.path.join(run, f"global-config-{label}.gitconfig")
    with open(config, "w", encoding="utf-8") as handle:
        handle.write(config_text)
    saved = os.environ.get("GIT_CONFIG_GLOBAL")
    os.environ["GIT_CONFIG_GLOBAL"] = config
    try:
        baseline = baseline_of(watchdog, repository)
        action(repository)
        lines = section_of(watchdog, repository, baseline)
        raw = "" if baseline.snapshot is None else repr(sorted(baseline.snapshot.status_by_path.items())[:3])
    finally:
        os.environ["GIT_CONFIG_GLOBAL"] = saved if saved is not None else os.devnull
    report(watchdog, f"global-config-{label}", lines, expectation, baseline_status=json.dumps(raw, ensure_ascii=False))


# ── 第 11 问：「没查成」与「余下没列」给的命令照原样能不能跑 ──
def scenario_manual_commands_verbatim(watchdog, run):
    repository = new_repository(run, "manual-commands", {"CLAUDE.md": "说明\n"})
    unread = watchdog.RepositoryBaseline(repository, None, "探针造的：起来时没读到")
    lines = section_of(watchdog, repository, unread)
    command = next((line.split("自己看：", 1)[1] for line in lines if "自己看：" in line), "")
    show(lines)
    child = subprocess.Popen(["bash", "-c", command], cwd=run, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, errors="replace",
                             env=dict(os.environ, **NEUTRAL_GIT_CONFIG), start_new_session=True)
    try:
        output, error_output = child.communicate(timeout=30)
    except subprocess.TimeoutExpired:
        os.killpg(child.pid, signal.SIGKILL)
        output, error_output = child.communicate()
    emit("manual-commands-verbatim", command=json.dumps(command, ensure_ascii=False), exit=child.returncode,
         expectation=json.dumps("「没查成」那一行给的手动命令原样交给 bash：两条之间是全角分号", ensure_ascii=False),
         stderr_first=json.dumps((error_output.strip().splitlines() or [""])[0], ensure_ascii=False), stdout_lines=len(output.splitlines()))


def scenario_remainder_command_verbatim(watchdog, run):
    repository = new_repository(run, "remainder-command", {"CLAUDE.md": "说明\n"})
    baseline = baseline_of(watchdog, repository, 1)
    commit_paths(repository, "动两份知识页", {".claude/kb/one.md": "页\n", ".claude/kb/two.md": "页\n"})
    lines = section_of(watchdog, repository, baseline, 1)
    show(lines)
    command = next((line.split("没列：", 1)[1] for line in lines if "份没列：" in line), "")
    child = subprocess.Popen(["bash", "-c", command], cwd=run, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, errors="replace",
                             env=dict(os.environ, **NEUTRAL_GIT_CONFIG), start_new_session=True)
    try:
        output, error_output = child.communicate(timeout=30)
    except subprocess.TimeoutExpired:
        os.killpg(child.pid, signal.SIGKILL)
        output, error_output = child.communicate()
    emit("remainder-command-verbatim", command=json.dumps(command, ensure_ascii=False), exit=child.returncode,
         expectation=json.dumps("「余 N 份没列」那一行给的看全部的命令原样交给 bash", ensure_ascii=False),
         stderr_first=json.dumps((error_output.strip().splitlines() or [""])[0], ensure_ascii=False), stdout_lines=len(output.splitlines()))


def add_untracked_knowledge(repository):
    write_file(repository, ".claude/rules/new-rule.md", "新规则\n")


def modify_knowledge(repository):
    write_file(repository, ".claude/kb/k.md", "改了\n")


SCENARIOS = (scenario_space_untracked_new, scenario_space_tracked_rewritten, scenario_quote_committed, scenario_space_committed,
             scenario_rename_in_to_out, scenario_rename_out_to_in, scenario_ignored_parent, scenario_modified_commit_modify,
             scenario_stash_then_pop, scenario_stash_only, scenario_reset_soft, scenario_branch_roundtrip, scenario_branch_switch_away,
             scenario_commit_deletes, scenario_untracked_new_directory, scenario_untracked_directory_grows, scenario_stage_only,
             scenario_chmod_only, scenario_ignored_inside_untracked, scenario_line_bound, scenario_git_count, scenario_index_lock,
             scenario_sparse_ignored_image, scenario_symlink_outside, scenario_root_precedence, scenario_worktree_inherited_variables,
             scenario_manual_commands_verbatim, scenario_remainder_command_verbatim)
CONFIG_SCENARIOS = (
    ("status-branch", "[status]\n\tbranch = true\n", modify_knowledge, "全局配置 status.branch=true：--short 多出「## 分支」一行，计数与路径表读成什么"),
    ("untracked-hidden", "[status]\n\tshowUntrackedFiles = no\n", add_untracked_knowledge, "全局配置 status.showUntrackedFiles=no：起来之后新建的未跟踪规则点不点名"),
    ("color-always", "[color]\n\tstatus = always\n", modify_knowledge, "全局配置 color.status=always：状态行带颜色转义，改了的知识页点不点名"),
)


def main():
    if len(sys.argv) == 4 and sys.argv[1] == "--fifo-child":
        return fifo_child(sys.argv[2], sys.argv[3])
    if len(sys.argv) != 3:
        print("用法：knowledge_sync_probe.py <agent-watch.py 的路径> <运行目录>", file=sys.stderr)
        return 2
    watchdog_path, run = os.path.abspath(sys.argv[1]), os.path.abspath(sys.argv[2])
    if os.environ.get("AGENT_WATCH_BREAK"):
        print("✗ 环境里设着 AGENT_WATCH_BREAK：探针要读的是没弄坏的那一份，去掉再跑", file=sys.stderr)
        return 2
    os.makedirs(run)
    os.environ.update(NEUTRAL_GIT_CONFIG)
    os.environ["GIT_CEILING_DIRECTORIES"] = run
    with open(watchdog_path, "rb") as handle:
        emit("watchdog", path=watchdog_path, sha256=hashlib.sha256(handle.read()).hexdigest(), git=json.dumps(subprocess.run(
            ["git", "--version"], capture_output=True, text=True).stdout.strip()))
    watchdog = load_watchdog(watchdog_path)
    for scenario in SCENARIOS:
        try:
            scenario(watchdog, run)
        except (RuntimeError, OSError, subprocess.SubprocessError) as error:
            setup_failures.append(scenario.__name__)
            emit(scenario.__name__, setup_failed=json.dumps(repr(error), ensure_ascii=False))
        except Exception as error:   # 被判对象自己抛出 RepositoryUnreadable 之外的异常：算读数
            emit(scenario.__name__, exception=type(error).__name__, detail=json.dumps(repr(error)[:300], ensure_ascii=False))
    for label, config_text, action, expectation in CONFIG_SCENARIOS:
        try:
            scenario_global_config(watchdog, run, label, config_text, action, expectation)
        except Exception as error:
            emit(f"global-config-{label}", exception=type(error).__name__, detail=json.dumps(repr(error)[:300], ensure_ascii=False))
    scenario_fifo_in_untracked_directory(watchdog_path, watchdog, run)
    print(f"PROBE name=done emitted={emitted_lines} setup_failures={len(setup_failures)}", flush=True)
    return 1 if setup_failures else 0


if __name__ == "__main__":
    sys.exit(main())
