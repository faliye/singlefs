#!/usr/bin/env python3
"""攻方腿探针（fg-timeout-large-file-governance-r3，正文「要腿判的」第 5 到 10 问）。装置那一半（emit、进程组、git 环境、超限文件、喂钩子）照第二轮攻方探针改过的副本写，格全新。

用法：python3 probe.py --repo <仓根> --frozen <被判的那棵树> --work <新的空目录> [--only <组,…>]
  --frozen 里的 git 钩子、判法、检出钩子、项目代码闸与共用模块是被判的；开头逐份对仓根
  research/prompts/fg-timeout-large-file-governance-r3-snapshot/judged-sha256.txt 核 sha256，对不上照跑、行里写 match=no。
  临时仓、裸远端、假工作根、检出记录都建在 --work 下；超限文件是 SINGLE_FILE_SIZE_LIMIT_BYTES + 1 字节的稀疏文件。
  临时仓里进仓的钩子目录 scripts/gate/git-hooks/ 放三份转发桩（exec 被判那棵树里的同名钩子），core.hooksPath 写成相对路径，与本仓的装法相同。
  起命令一律放进新会话（start_new_session），超时与收尾只对自己起的那一组发信号；钩子起的判法在它自己的会话里，探针不停它，只按 /proc 等它退。
输出：每格一行 `PROBE name=<格名> …`，末行 `PROBE name=done emitted=<行数>`（不含末行）。
退出码：0 跑完；1 有一组装置出错（那一组一行 group-error-<组>）；2 用法错。
结局（verdict）：missed-silently 超限的进了提交或远端、检出钩子没拒、git 钩子没拒也没写 ⊘；passed-recorded 进了、git 钩子写了 ⊘；
  rejected 检出钩子拒了这条命令（命令照跑时会进）；commit-lands-push-refused 提交进了、推被 git 钩子拒了；refused-by-git-hook 什么都没进、git 钩子拒了；
  rejected-nothing-lands 检出钩子或 git 钩子拒了而什么超限的新内容都不会带上去（误拒）；passed-nothing-lands 两样都没有。
"""
import argparse
import datetime
import hashlib
import json
import os
import re
import shlex
import signal
import subprocess
import sys
import time

EMITTED = 0
SETTINGS = {}
HOOK_NAMES = ("pre-commit", "pre-push", "pre-merge-commit")


def emit(name, **fields):
    global EMITTED
    EMITTED += 1
    parts = [f"name={name}"]
    for key, value in fields.items():
        text = str(value)
        if not re.fullmatch(r"[\w.+\-/=,%]+", text):
            text = json.dumps(text, ensure_ascii=False)
        parts.append(f"{key}={text}")
    print("PROBE " + " ".join(parts), flush=True)


def group_members(group):
    found = []
    for entry in os.listdir("/proc"):
        if not entry.isdigit():
            continue
        try:
            with open(f"/proc/{entry}/stat", "rb") as handle:
                text = handle.read().decode(errors="replace")
        except OSError:
            continue
        fields = text.rsplit(")", 1)[-1].split()
        if len(fields) > 2 and fields[2] == str(group):
            found.append(int(entry))
    return found


def processes_mentioning(marker):
    """/proc 里命令行带 marker 的进程号（只读，不发信号）。"""
    found = []
    for entry in os.listdir("/proc"):
        if not entry.isdigit() or int(entry) == os.getpid():
            continue
        try:
            with open(f"/proc/{entry}/cmdline", "rb") as handle:
                text = handle.read().replace(b"\0", b" ").decode(errors="replace")
        except OSError:
            continue
        if marker in text:
            found.append(int(entry))
    return found


def stop_own_group(group):
    """只对自己用新会话起的那一个子进程的组发信号：先 TERM，过宽限再 KILL。"""
    for chosen in (signal.SIGTERM, signal.SIGKILL):
        if not group_members(group):
            return
        try:
            os.killpg(group, chosen)
        except ProcessLookupError:
            return
        time.sleep(2)


class Outcome:
    def __init__(self, code, out, err, seconds, leftover):
        self.code, self.out, self.err, self.seconds, self.leftover = code, out, err, seconds, leftover


def run_own_group(argv, cwd, env, stdin_text=None, limit_seconds=300):
    started = time.monotonic()
    child = subprocess.Popen(argv, cwd=cwd, env=env, encoding="utf-8", errors="replace", start_new_session=True,
                             stdin=subprocess.PIPE if stdin_text is not None else subprocess.DEVNULL,
                             stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    try:
        out, err = child.communicate(stdin_text, timeout=limit_seconds)
    except subprocess.TimeoutExpired:
        stop_own_group(child.pid)
        out, err = child.communicate()
        err = (err or "") + "\n[probe: timed out]"
    seconds = time.monotonic() - started
    leftover = group_members(child.pid)
    if leftover:
        stop_own_group(child.pid)
    return Outcome(child.returncode, out or "", err or "", seconds, leftover)


def wait_until_gone(marker, limit_seconds):
    """等命令行带 marker 的进程都退出（不发信号），交等了多久；到点还在交 None。"""
    started = time.monotonic()
    while processes_mentioning(marker):
        if time.monotonic() - started > limit_seconds:
            return None
        time.sleep(0.5)
    return time.monotonic() - started


def git_environment(extra=None):
    env = {name: value for name, value in os.environ.items()
           if not name.startswith(("GIT_", "BASH_COMMAND_DETECTOR_", "LARGE_FILE_", "LIB_", "PROJECT_CODE_GUARD_", "AGENT_HOOK_DETECTIONS"))}
    env.update(GIT_CONFIG_GLOBAL=SETTINGS["global_config"], GIT_CONFIG_NOSYSTEM="1",
               GIT_AUTHOR_NAME="probe", GIT_AUTHOR_EMAIL="probe@example.invalid",
               GIT_COMMITTER_NAME="probe", GIT_COMMITTER_EMAIL="probe@example.invalid",
               GIT_CEILING_DIRECTORIES=SETTINGS["work"], GIT_TERMINAL_PROMPT="0", GIT_EDITOR="true")
    env.update(extra or {})
    return env


def git(repository, *arguments, env_extra=None, check=True):
    """装置自己的 git：一律 -c core.hooksPath=/dev/null，钩子不起。"""
    completed = subprocess.run(["git", "-C", repository, "-c", "core.hooksPath=/dev/null", *arguments], env=git_environment(env_extra),
                               capture_output=True, text=True)
    if check and completed.returncode != 0:
        raise RuntimeError(f"装置里的 git {' '.join(arguments)} 退 {completed.returncode}：{completed.stderr.strip()[:300]}")
    return completed.stdout.strip()


def yes_no(flag):
    return "yes" if flag else "no"


def write_small(repository, relative, text="small\n"):
    full = os.path.join(repository, relative)
    os.makedirs(os.path.dirname(full), exist_ok=True)
    with open(full, "w", encoding="utf-8") as handle:
        handle.write(text)


def make_big(repository, relative):
    """稀疏的超限文件：上限加一字节。"""
    full = os.path.join(repository, relative)
    os.makedirs(os.path.dirname(full), exist_ok=True)
    with open(full, "wb") as handle:
        handle.truncate(SETTINGS["limit"] + 1)


def write_hook_stubs(repository):
    """进仓的钩子目录：三份转发桩，exec 被判那棵树里的同名钩子（标准输入与参数原样带过去）。"""
    directory = os.path.join(repository, "scripts", "gate", "git-hooks")
    os.makedirs(directory, exist_ok=True)
    for name in HOOK_NAMES:
        path = os.path.join(directory, name)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(f'#!/usr/bin/env bash\nprintf "%s %s\\n" {name} "$PWD" >> {shlex.quote(SETTINGS["hook_log"])}\nexec {shlex.quote(os.path.join(SETTINGS["frozen"], "scripts", "gate", "git-hooks", name))} "$@"\n')
        os.chmod(path, 0o755)


def hooked_repository(name, hooks_in_first_commit=True):
    """临时仓：分支 main，core.hooksPath 是相对路径 scripts/gate/git-hooks（与本仓的装法相同）。
    hooks_in_first_commit 假时第一笔提交只有 base.txt、第二笔才加进钩子目录（检出第一笔的树里没有钩子）。"""
    path = os.path.join(SETTINGS["repositories"], name)
    os.makedirs(path)
    git(path, "init", "-q")
    git(path, "symbolic-ref", "HEAD", "refs/heads/main")
    write_small(path, "base.txt")
    write_small(path, "notes/readme.txt")
    if hooks_in_first_commit:
        write_hook_stubs(path)
    git(path, "add", "-A")
    git(path, "commit", "-q", "-m", "base")
    if not hooks_in_first_commit:
        write_hook_stubs(path)
        git(path, "add", "-A")
        git(path, "commit", "-q", "-m", "hooks")
    git(path, "config", "core.hooksPath", "scripts/gate/git-hooks")
    return path


def bare_remote(name):
    remote = os.path.join(SETTINGS["remotes"], name + ".git")
    subprocess.run(["git", "init", "-q", "--bare", remote], env=git_environment(), check=True, capture_output=True)
    return remote


def with_origin(repository, name):
    remote = bare_remote(name)
    git(repository, "remote", "add", "origin", remote)
    git(repository, "push", "-q", "-u", "origin", "main")
    return remote


def commit_big_without_hooks(repository, relative="big.bin", message="big"):
    make_big(repository, relative)
    git(repository, "add", relative)
    git(repository, "commit", "-q", "-m", message)


def all_commits(repository):
    listing = subprocess.run(["git", "-C", repository, "rev-list", "--all"], env=git_environment(), capture_output=True, text=True)
    return set(listing.stdout.split())


def blob_sizes(repository, names):
    if not names:
        return {}
    completed = subprocess.run(["git", "-C", repository, "cat-file", "--batch-check=%(objectname) %(objecttype) %(objectsize)"],
                               input="\n".join(sorted(names)) + "\n", env=git_environment(), capture_output=True, text=True)
    sizes = {}
    for line in completed.stdout.splitlines():
        fields = line.split()
        if len(fields) == 3 and fields[1] == "blob":
            sizes[fields[0]] = int(fields[2])
    return sizes


def big_blobs_brought_by(repository, commits):
    names = set()
    for commit in commits:
        output = subprocess.run(["git", "-C", repository, "diff-tree", "-r", "-m", "--root", "--no-commit-id", "--no-renames", commit],
                                env=git_environment(), capture_output=True, text=True).stdout
        for line in output.splitlines():
            if line.startswith(":"):
                names.add(line[1:].split()[3])
    return sum(1 for size in blob_sizes(repository, names).values() if size > SETTINGS["limit"])


def big_blobs_in(repository):
    """仓里所有引用能到的对象（含指着 blob、tree 的标签）里超限的 blob 个数。"""
    if not repository or not os.path.isdir(repository):
        return 0
    listing = subprocess.run(["git", "-C", repository, "rev-list", "--objects", "--all"], env=git_environment(),
                             capture_output=True, text=True).stdout
    names = {line.split()[0] for line in listing.splitlines() if line.strip()}
    return sum(1 for size in blob_sizes(repository, names).values() if size > SETTINGS["limit"])


def hook_environment(detections, extra=None):
    env = git_environment()
    env.update(CLAUDE_PROJECT_DIR=SETTINGS["frozen"], AGENT_HOOK_DETECTIONS=detections)
    env.update(extra or {})
    return env


def feed(hook_name, payload, extra=None, label="feed"):
    detections = os.path.join(SETTINGS["detections"], label + ".jsonl")
    hook = os.path.join(SETTINGS["frozen"], ".claude", "hooks", hook_name)
    cwd = payload.get("cwd") if os.path.isdir(payload.get("cwd") or "") else SETTINGS["work"]
    run = run_own_group(["bash", hook], cwd, hook_environment(detections, extra), stdin_text=json.dumps(payload), limit_seconds=300)
    keep_stderr(label + "-" + hook_name, run.err)
    return run


def payload_for(command, cwd, agent_type=None, transcript=None):
    payload = {"tool_name": "Bash", "session_id": "probe-session", "cwd": cwd, "tool_input": {"command": command, "run_in_background": False}}
    if agent_type:
        payload.update(agent_type=agent_type, agent_id="probe-agent")
    if transcript:
        payload["transcript_path"] = transcript
    return payload


def keep_stderr(name, text):
    with open(os.path.join(SETTINGS["stderr"], name + ".txt"), "w", encoding="utf-8") as handle:
        handle.write(text)


def hook_calls():
    try:
        with open(SETTINGS["hook_log"], encoding="utf-8") as handle:
            return handle.read().splitlines()
    except OSError:
        return []


def detector_reason(run):
    if run.code != 2:
        return "none"
    if "绕开了 git 的钩子" in run.err:
        return "bypass"
    if "钩子没装上" in run.err:
        return "not-installed"
    first = next((line.strip() for line in run.err.splitlines() if "✗" in line), "other")
    return first[:80]


def large_file_case(name, question, cwd, command, repository, remote=None, run_extra=None, feed_detector=True):
    """喂检出钩子一次（不执行），再真执行这条命令一次（git 钩子照装好的样子起），看超限的 blob 有没有进新提交、进远端。"""
    commits_before = all_commits(repository)
    remote_before = big_blobs_in(remote)
    calls_before = len(hook_calls())
    detector = feed("bash-command-detector.sh", payload_for(command, cwd), label=name) if feed_detector else None
    reason = detector_reason(detector) if detector else "not-fed"
    run = run_own_group(["bash", "-c", command], cwd, git_environment(run_extra))
    keep_stderr(name + "-command", run.err)
    git_hook_refused = "单文件上限" in run.err
    recorded = "⊘" in run.err
    landed_commit = big_blobs_brought_by(repository, all_commits(repository) - commits_before) > 0
    landed_remote = big_blobs_in(remote) > remote_before
    calls = hook_calls()[calls_before:]
    if landed_commit or landed_remote:
        if reason not in ("none", "not-fed"):
            verdict = "rejected"
        elif landed_remote:
            verdict = "passed-recorded" if recorded else "missed-silently"
        else:
            verdict = "commit-lands-push-refused" if git_hook_refused else "passed-recorded" if recorded else "missed-silently"
    else:
        verdict = ("refused-by-git-hook" if git_hook_refused else "rejected-nothing-lands" if reason not in ("none", "not-fed")
                   else "passed-nothing-lands")
    emit(name, question=question, detector_exit=detector.code if detector else "not-fed", detector_reason=reason, command_exit=run.code,
         landed_commit=yes_no(landed_commit), landed_remote=yes_no(landed_remote), remote_had_big_before=yes_no(remote_before > 0), git_hook_refused=yes_no(git_hook_refused),
         git_hook_note=yes_no(recorded), hooks_invoked=",".join(line.split()[0] for line in calls) or "none", verdict=verdict,
         seconds=f"{run.seconds:.1f}", command=command)


def staged_big(name, with_remote=False):
    repository = hooked_repository(name)
    remote = with_origin(repository, name) if with_remote else None
    make_big(repository, "big.bin")
    git(repository, "add", "big.bin")
    return repository, remote


def unpushed_big(name):
    repository = hooked_repository(name)
    remote = with_origin(repository, name)
    commit_big_without_hooks(repository)
    return repository, remote


def diverged_with_big_feature(name):
    repository = hooked_repository(name)
    remote = with_origin(repository, name)
    git(repository, "checkout", "-q", "-b", "feature")
    commit_big_without_hooks(repository, message="feature big")
    git(repository, "checkout", "-q", "main")
    write_small(repository, "main-only.txt")
    git(repository, "add", "main-only.txt")
    git(repository, "commit", "-q", "-m", "main work")
    git(repository, "push", "-q", "origin", "main")
    return repository, remote


def bypass_cases():
    """第 5 问：git 钩子当主闸之后的绕法。"""
    repository, _ = staged_big("control-commit-staged-big")
    large_file_case("control-commit-staged-big", 5, repository, "git commit -qm x", repository)
    repository, remote = unpushed_big("control-push-unpushed-big")
    large_file_case("control-push-unpushed-big", 5, repository, "git push -q origin main", repository, remote)

    repository, remote = diverged_with_big_feature("merge-no-verify-then-push")
    large_file_case("merge-no-verify-then-push", 5, repository, "git merge --no-verify -q --no-edit feature && git push -q origin main", repository, remote)

    source = hooked_repository("am-patch-source")
    commit_big_without_hooks(source, message="patch with big")
    patch = os.path.join(SETTINGS["work"], "big.mbox")
    with open(patch, "w", encoding="utf-8") as handle:
        handle.write(git(source, "format-patch", "-1", "--binary", "--stdout") + "\n")
    repository = hooked_repository("am-then-push")
    remote = with_origin(repository, "am-then-push")
    large_file_case("am-then-push", 5, repository, f"git am -q {shlex.quote(patch)} && git push -q origin main", repository, remote)

    repository, remote = staged_big("commit-tree-update-ref-then-push", with_remote=True)
    large_file_case("commit-tree-update-ref-then-push", 5, repository,
                    'git update-ref refs/heads/main "$(git commit-tree "$(git write-tree)" -p HEAD -m x)" && git push -q origin main', repository, remote)

    repository, remote = staged_big("stash-then-push-stash", with_remote=True)
    large_file_case("stash-then-push-stash", 5, repository, "git stash push -q -- big.bin && git push -q origin refs/stash:refs/heads/stashed", repository, remote)

    empty = os.path.join(SETTINGS["work"], "empty-directory")
    os.makedirs(empty, exist_ok=True)
    repository, _ = staged_big("work-tree-option-commit")
    large_file_case("work-tree-option-commit", 5, repository, f"git --work-tree={shlex.quote(empty)} commit -qm x", repository)
    repository, remote = unpushed_big("git-dir-option-push-from-outside")
    large_file_case("git-dir-option-push-from-outside", 5, empty, f"git --git-dir={shlex.quote(os.path.join(repository, '.git'))} push -q origin main",
                    repository, remote)
    repository, remote = unpushed_big("git-dir-variable-push-from-subdirectory")
    large_file_case("git-dir-variable-push-from-subdirectory", 5, os.path.join(repository, "notes"),
                    f"GIT_DIR={shlex.quote(os.path.join(repository, '.git'))} git push -q origin main", repository, remote)

    repository = hooked_repository("old-commit-linked-worktree", hooks_in_first_commit=False)
    remote = with_origin(repository, "old-commit-linked-worktree")
    first = git(repository, "rev-list", "--max-parents=0", "HEAD")
    linked = os.path.join(SETTINGS["repositories"], "old-commit-linked-worktree-linked")
    git(repository, "worktree", "add", "-q", "-b", "old-line", linked, first)
    make_big(linked, "big.bin")
    large_file_case("old-commit-linked-worktree-commit-and-push", 5, linked,
                    "git add big.bin && git commit -qm x && git push -q origin old-line", repository, remote)

    repository, remote = hooked_repository("sparse-checkout-commit-and-push"), None
    remote = with_origin(repository, "sparse-checkout-commit-and-push")
    make_big(repository, "big.bin")
    large_file_case("sparse-checkout-commit-and-push", 5, repository,
                    "git sparse-checkout set --no-cone /base.txt /big.bin && git add big.bin && git commit -qm x && git push -q origin main", repository, remote)

    repository = hooked_repository("tag-to-blob-push")
    remote = with_origin(repository, "tag-to-blob-push")
    make_big(repository, "big.bin")
    large_file_case("tag-to-blob-push", 5, repository, 'git tag big-blob "$(git hash-object -w big.bin)" && git push -q origin refs/tags/big-blob',
                    repository, remote)
    repository = hooked_repository("tag-to-tree-push")
    remote = with_origin(repository, "tag-to-tree-push")
    make_big(repository, "big.bin")
    large_file_case("tag-to-tree-push", 5, repository,
                    "tree=$(printf '100644 blob %s\\tbig.bin\\n' \"$(git hash-object -w big.bin)\" | git mktree) && git tag -a big-tree -m t \"$tree\" "
                    "&& git push -q origin refs/tags/big-tree", repository, remote)

    repository, _ = staged_big("selftest-word-in-message-break-switch")
    large_file_case("selftest-word-in-message-break-switch", 5, repository,
                    "LARGE_FILE_GIT_HOOK_BREAK=pre-commit-judge-skipped git commit -qm --selftest", repository)
    repository, remote = unpushed_big("selftest-word-as-shell-name-limit-raised-push")
    large_file_case("selftest-word-as-shell-name-limit-raised-push", 5, repository,
                    "LARGE_FILE_GIT_HOOK_LIMIT_BYTES=999999999999 bash -c 'git push -q origin main' --selftest", repository, remote)

    stub = os.path.join(SETTINGS["work"], "python3-stub")
    os.makedirs(stub, exist_ok=True)
    with open(os.path.join(stub, "python3"), "w", encoding="utf-8") as handle:
        handle.write("#!/bin/sh\nexit 0\n")
    os.chmod(os.path.join(stub, "python3"), 0o755)
    repository, remote = staged_big("path-python3-stub-commit-and-push", with_remote=True)
    large_file_case("path-python3-stub-commit-and-push", 5, repository,
                    f'PATH={shlex.quote(stub)}:"$PATH" git commit -qm x && PATH={shlex.quote(stub)}:"$PATH" git push -q origin main', repository, remote)

    repository, _ = staged_big("config-remove-section-then-commit")
    large_file_case("config-remove-section-then-commit", 5, repository, "git config --remove-section core && git commit -qm x", repository)
    repository, _ = staged_big("config-rename-section-then-commit")
    large_file_case("config-rename-section-then-commit", 5, repository, "git config --rename-section core moved && git commit -qm x", repository)

    global_file = os.path.join(SETTINGS["work"], "global-with-hooks-path.gitconfig")
    with open(global_file, "w", encoding="utf-8") as handle:
        handle.write("[core]\n\thooksPath = /dev/null\n[user]\n\tname = probe\n\temail = probe@example.invalid\n")
    repository, _ = staged_big("global-config-hooks-path-control")
    large_file_case("global-config-hooks-path-control", 5, repository, f"GIT_CONFIG_GLOBAL={shlex.quote(global_file)} git commit -qm x", repository)


def push_misreject_cases():
    """第 6 问：pre-push 对删除引用、只推标签、远端已有的超限 blob 历史各判成什么；收窄后的 ㉒ 对日常写法。"""
    repository, remote = unpushed_big("push-delete-remote-branch-only")
    git(repository, "push", "-q", "origin", "HEAD~1:refs/heads/stale")
    large_file_case("push-delete-remote-branch-only", 6, repository, "git push -q origin :stale", repository, remote)
    large_file_case("push-tag-on-pushed-commit-only", 6, repository, "git tag -a v1 -m v origin/main && git push -q origin v1", repository, remote)

    # 远端（写成路径）已有带超限 blob 的提交，之后被另一份克隆推进了一笔：本地不认得行里的远端 sha，判全部历史
    repository = hooked_repository("push-path-remote-advanced-elsewhere")
    backup = bare_remote("push-path-remote-advanced-elsewhere-backup")
    commit_big_without_hooks(repository)
    git(repository, "push", "-q", backup, "main")
    other = os.path.join(SETTINGS["repositories"], "push-path-remote-advanced-elsewhere-other")
    subprocess.run(["git", "clone", "-q", backup, other], env=git_environment(), check=True, capture_output=True)
    write_small(other, "other.txt")
    git(other, "add", "other.txt")
    git(other, "commit", "-q", "-m", "other")
    git(other, "push", "-q", "origin", "main")
    write_small(repository, "mine.txt")
    git(repository, "add", "mine.txt")
    git(repository, "commit", "-q", "-m", "mine")
    large_file_case("push-path-remote-advanced-elsewhere-force", 6, repository, f"git push -q --force {shlex.quote(backup)} main", repository, backup)
    # 同一种历史，推向配置过的远端名、跟踪引用带着那笔提交（对照）
    repository, remote = unpushed_big("push-configured-remote-already-has-big-control")
    git(repository, "push", "-q", "origin", "main")
    write_small(repository, "mine.txt")
    git(repository, "add", "mine.txt")
    git(repository, "commit", "-q", "-m", "mine")
    large_file_case("push-configured-remote-already-has-big-control", 6, repository, "git push -q origin main", repository, remote)

    # 没取回过的远端判全部历史：在被判的仓本身上量判法要多久（只读，索引拷贝放进运行目录）
    copy_directory = os.path.join(SETTINGS["work"], "index-copy")
    os.makedirs(copy_directory, exist_ok=True)
    judge = run_own_group([sys.executable, "-B", os.path.join(SETTINGS["frozen"], "scripts", "gate", "large-file-check.py"), SETTINGS["repo"],
                           "--guard", "push", "--remote", os.path.join(SETTINGS["work"], "never-configured-remote.git"), "--ref", "HEAD",
                           "--judge-unknown", "--index-copy-directory", copy_directory], SETTINGS["repo"], dict(os.environ), limit_seconds=900)
    try:
        result = json.loads(judge.out.strip().splitlines()[-1])
    except (ValueError, IndexError):
        result = {}
    emit("judge-unknown-full-history-elapsed", question=6, exit=judge.code, seconds=f"{judge.seconds:.1f}", checked_commits=result.get("checked", "?"),
         too_big=len(result.get("too_big", [])), unreadable=len(result.get("unreadable", [])), not_run=result.get("not_run", "?") or "none",
         judge_seconds_limit=600)

    # 收窄后的 ㉒：日常与近日常的写法只喂检出钩子，不执行
    repository = hooked_repository("detector-daily-forms")
    forms = (("detector-set-hooks-path-to-the-right-value", "git config core.hooksPath scripts/gate/git-hooks"),
             ("detector-command-line-hooks-path-right-value", "git -c core.hooksPath=scripts/gate/git-hooks commit -qm x"),
             ("detector-message-mentions-short-n", "git commit -q -m 'drop the -n flag'"),
             ("detector-heredoc-message-mentions-no-verify", "git commit -q -F - <<'EOF'\nmention --no-verify in the message\nEOF"),
             ("detector-read-hooks-path", "git config --show-origin --get core.hooksPath"),
             ("detector-plain-commit-control", "git commit -qm x"))
    for name, command in forms:
        run = feed("bash-command-detector.sh", payload_for(command, repository), label=name)
        emit(name, question=6, detector_exit=run.code, detector_reason=detector_reason(run), command=command)


def slow_git_directory(name, seconds):
    """判法 PATH 前面的一份 git：起 diff 的那一次先睡 seconds 秒，再交给真 git（只在钩子认得自证的时候才放得进判法的 PATH）。"""
    directory = os.path.join(SETTINGS["work"], name)
    os.makedirs(directory, exist_ok=True)
    real = subprocess.run(["bash", "-c", "command -v git"], capture_output=True, text=True).stdout.strip()
    with open(os.path.join(directory, "git"), "w", encoding="utf-8") as handle:
        handle.write(f'#!/bin/sh\ncase " $* " in *" diff "*) sleep {seconds} ;; esac\nexec {real} "$@"\n')
    os.chmod(os.path.join(directory, "git"), 0o755)
    return directory


def wait_for_judge(marker, limit_seconds=30):
    started = time.monotonic()
    while not processes_mentioning(marker):
        if time.monotonic() - started > limit_seconds:
            return False
        time.sleep(0.2)
    return True


def push_guard_cases():
    """第 7 问：--guard push 只认行里远端 sha；本地没有的远端 sha、浅克隆、远端 sha 全 0。"""
    for name, by_path in (("force-push-remote-sha-unknown-locally-configured-remote", False), ("force-push-remote-sha-unknown-locally-path-remote", True)):
        repository, remote = unpushed_big(name)
        other = os.path.join(SETTINGS["repositories"], name + "-other")
        subprocess.run(["git", "clone", "-q", remote, other], env=git_environment(), check=True, capture_output=True)
        write_small(other, "other.txt")
        git(other, "add", "other.txt")
        git(other, "commit", "-q", "-m", "other")
        git(other, "push", "-q", "origin", "main")
        target = shlex.quote(remote) if by_path else "origin"
        large_file_case(name, 7, repository, f"git push -q --force {target} main", repository, remote)

    # 浅克隆：远端最新那笔提交带着超限 blob（远端已有），浅克隆加一笔小提交推新分支
    seed = hooked_repository("shallow-seed")
    remote = bare_remote("shallow-seed")
    commit_big_without_hooks(seed)
    git(seed, "push", "-q", remote, "main")
    shallow = os.path.join(SETTINGS["repositories"], "shallow-clone")
    subprocess.run(["git", "clone", "-q", "--depth", "1", "file://" + remote, shallow], env=git_environment(), check=True, capture_output=True)
    git(shallow, "config", "core.hooksPath", "scripts/gate/git-hooks")
    write_small(shallow, "mine.txt")
    git(shallow, "add", "mine.txt")
    git(shallow, "commit", "-q", "-m", "mine")
    large_file_case("shallow-clone-new-branch-by-path", 7, shallow, f"git push -q {shlex.quote(remote)} HEAD:refs/heads/topic-by-path", shallow, remote)
    large_file_case("shallow-clone-new-branch-by-configured-name-control", 7, shallow, "git push -q origin HEAD:refs/heads/topic-by-name", shallow, remote)
    repository, remote = unpushed_big("new-branch-remote-sha-all-zero-control")
    large_file_case("new-branch-remote-sha-all-zero-control", 7, repository, "git push -q origin main:refs/heads/fresh-name", repository, remote)


def race_case(name, prepare, command):
    """别的会话在 pre-commit 判法读完索引拷贝之后、git 重读索引之前 git add 超限文件。判法放慢靠自证用的 LARGE_FILE_GIT_HOOK_JUDGE_PATH
    （命令行里带 --selftest 一词的祖先进程让钩子认它，这一招本身是第 5 问的一格）。"""
    repository = hooked_repository(name)
    prepare(repository)
    make_big(repository, "big.bin")
    before = all_commits(repository)
    slow = slow_git_directory(name + "-slow-git", 6)
    env = git_environment({"LARGE_FILE_GIT_HOOK_JUDGE_PATH": slow})
    child = subprocess.Popen(["bash", "-c", command, "--selftest"], cwd=repository, env=env, start_new_session=True,
                             stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    seen = wait_for_judge(f"{os.path.basename(repository)} --guard commit-index")
    time.sleep(1.0)
    added = subprocess.run(["git", "-C", repository, "add", "big.bin"], env=git_environment(), capture_output=True, text=True)
    try:
        _out, err = child.communicate(timeout=120)
    except subprocess.TimeoutExpired:
        stop_own_group(child.pid)
        _out, err = child.communicate()
    keep_stderr(name + "-command", err)
    landed = big_blobs_brought_by(repository, all_commits(repository) - before) > 0
    emit(name, question=7, judge_seen=yes_no(seen), concurrent_add_exit=added.returncode, commit_exit=child.returncode, landed_commit=yes_no(landed),
         git_hook_refused=yes_no("单文件上限" in err), verdict="missed-silently" if landed and child.returncode == 0 else "nothing-lands",
         add_stderr=added.stderr.strip()[:120] or "none", command=command)


def stage_small(repository):
    write_small(repository, "small.txt")
    git(repository, "add", "small.txt")


def grow_tracked(repository):
    write_small(repository, "base.txt", "grown but small\n")


def signal_case(name, chosen_signal):
    """git commit 在 pre-commit 判法跑着的时候整组收到信号（Claude Code 的 Bash 到点、Ctrl-C、终端断开都是这一类）：
    钩子自己（与 git 同一组）收到，判法在它自己的会话里收不到。看索引拷贝的目录留没留、判法活多久。"""
    repository = hooked_repository(name)
    stage_small(repository)
    temporary = os.path.join(SETTINGS["work"], name + "-tmp")
    os.makedirs(temporary)
    slow = slow_git_directory(name + "-slow-git", 8)
    env = git_environment({"LARGE_FILE_GIT_HOOK_JUDGE_PATH": slow, "TMPDIR": temporary})
    child = subprocess.Popen(["bash", "-c", "git commit -qm x", "--selftest"], cwd=repository, env=env, start_new_session=True,
                             stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    marker = f"{os.path.basename(repository)} --guard commit-index"
    seen = wait_for_judge(marker)
    time.sleep(0.5)
    signalled = time.monotonic()
    try:
        os.killpg(child.pid, chosen_signal)
    except ProcessLookupError:
        pass
    try:
        child.communicate(timeout=60)
    except subprocess.TimeoutExpired:
        stop_own_group(child.pid)
        child.communicate()
    command_gone = time.monotonic() - signalled
    copies_right_after = sorted(entry for entry in os.listdir(temporary) if entry.startswith("large-file-git-hook-index-"))
    judge_alive_after_command = bool(processes_mentioning(marker))
    hook_alive_after_command = bool(processes_mentioning(f"large-file-git-hook.py pre-commit"))
    judge_waited = wait_until_gone(marker, 60)
    hook_waited = wait_until_gone("large-file-git-hook.py pre-commit", 60)
    copies_at_end = sorted(entry for entry in os.listdir(temporary) if entry.startswith("large-file-git-hook-index-"))
    files_at_end = sum(len(os.listdir(os.path.join(temporary, entry))) for entry in copies_at_end)
    emit(name, question=7, judge_seen=yes_no(seen), signal=chosen_signal.name, command_exit=child.returncode, command_gone_seconds=f"{command_gone:.1f}",
         judge_alive_after_command=yes_no(judge_alive_after_command), hook_alive_after_command=yes_no(hook_alive_after_command),
         judge_gone_after_seconds="timeout" if judge_waited is None else f"{judge_waited:.1f}",
         hook_gone_after_seconds="timeout" if hook_waited is None else f"{hook_waited:.1f}",
         index_copy_directories_right_after=len(copies_right_after), index_copy_directories_at_end=len(copies_at_end), files_in_them_at_end=files_at_end)


def judge_runtime_cases():
    race_case("race-add-during-as-is-commit", stage_small, "git commit -qm as-is")
    race_case("race-add-during-commit-all-control", grow_tracked, "git commit -qam all")
    signal_case("hook-sigterm-mid-judge", signal.SIGTERM)
    signal_case("hook-sigint-mid-judge", signal.SIGINT)
    signal_case("hook-sighup-mid-judge", signal.SIGHUP)


def write_lock(directory, prompt, claimed_by=None, dispatched_ago=600):
    os.makedirs(directory, exist_ok=True)
    moment = time.time() - dispatched_ago
    lock = {"session_id": "probe-session", "dispatched_at": moment, "dispatched_date": datetime.datetime.fromtimestamp(moment, datetime.timezone.utc).isoformat(),
            "prompt_sha256": hashlib.sha256(prompt.encode("utf-8")).hexdigest(), "subagent_type": "sweep", "transcript_path": "", "nonce": hashlib.sha256(directory.encode()).hexdigest()[:16]}
    with open(os.path.join(directory, ".writer-lock"), "w", encoding="utf-8") as handle:
        json.dump(lock, handle)
    if claimed_by:
        with open(os.path.join(directory, ".writer-claim-" + lock["nonce"]), "w", encoding="utf-8") as handle:
            json.dump({"agent_id": claimed_by, "claimed_at": moment + 1}, handle)


def write_transcript(path, receipts):
    moment = datetime.datetime.fromtimestamp(time.time() - 300, datetime.timezone.utc).isoformat().replace("+00:00", "Z")
    with open(path, "w", encoding="utf-8") as handle:
        for agent_id, prompt in receipts:
            handle.write(json.dumps({"type": "user", "timestamp": moment,
                                     "toolUseResult": {"status": "async_launched", "agentId": agent_id, "prompt": prompt}}) + "\n")


def scope_cases():
    """第 8 问：⑫ 写者锁接到只带 Read、Bash 的定义之后；三方腿收窄之后的模式。只喂检出钩子，不执行；工作根用 SINGLEFS_WORK_ROOT 指到运行目录里的假工作根。"""
    root = os.path.join(SETTINGS["work"], "fake-work-root")
    own = os.path.join(root, "own-draft")
    write_lock(own, "own prompt")
    write_lock(os.path.join(root, "other-claimed-draft"), "other prompt", claimed_by="other-agent")
    write_lock(os.path.join(root, "other-unclaimed-draft"), "other prompt")
    write_lock(os.path.join(own, "copied-previous-draft"), "earlier prompt", claimed_by="earlier-agent")
    os.makedirs(os.path.join(root, "owner-only-directory"))
    with open(os.path.join(root, "owner-only-directory", ".owner"), "w", encoding="utf-8") as handle:
        handle.write("session: another-session\npurpose: probe\n")
    os.makedirs(os.path.join(root, "worktree-like", "crates"))
    with_receipt = os.path.join(SETTINGS["work"], "transcript-with-receipt.jsonl")
    without_receipt = os.path.join(SETTINGS["work"], "transcript-without-receipt.jsonl")
    write_transcript(with_receipt, [("probe-agent", "own prompt"), ("other-agent", "other prompt")])
    write_transcript(without_receipt, [("other-agent", "other prompt")])
    prefix = "research/prompts/fg-timeout-large-file-governance-"
    cases = (
        ("lock-own-draft-holder-control", "sweep", with_receipt, f"printf x > {own}/f.txt"),
        ("lock-other-draft-claimed", "sweep", with_receipt, f"printf x > {root}/other-claimed-draft/f.txt"),
        ("lock-other-draft-unclaimed-receipt-mismatch", "sweep", with_receipt, f"printf x > {root}/other-unclaimed-draft/f.txt"),
        ("lock-other-draft-unclaimed-receipt-missing", "sweep", without_receipt, f"printf x > {root}/other-unclaimed-draft/f.txt"),
        ("lock-owner-only-directory-of-another-session", "sweep", with_receipt, f"printf x > {root}/owner-only-directory/f.txt"),
        ("lock-unowned-worktree-like-directory", "sweep", with_receipt, f"printf x > {root}/worktree-like/crates/planted.rs"),
        ("lock-copied-lock-inside-own-draft", "sweep", with_receipt, f"printf x > {own}/copied-previous-draft/f.txt"),
        ("lock-write-edit-definition-into-claimed-draft", "clerk", with_receipt, f"printf x > {root}/other-claimed-draft/g.txt"),
        ("scope-attack-own-report-control", "three-way-attack", with_receipt, f"printf x > {prefix}r3-opus-output.md"),
        ("scope-attack-earlier-round-report", "three-way-attack", with_receipt, f"printf x > {prefix}r1-opus-output.md"),
        ("scope-attack-earlier-round-model", "three-way-attack", with_receipt, f"printf x > {prefix}r2-opus-model/planted.py"),
        ("scope-attack-own-second-sample-report", "three-way-attack", with_receipt, f"printf x > {prefix}r3-opus-s2-output.md"),
        ("scope-attack-own-second-sample-model", "three-way-attack", with_receipt, f"printf x > {prefix}r3-opus-s2-model/probe.py"),
        ("scope-defense-writes-forward-report-of-another-round", "three-way-defense", with_receipt, f"printf x > {prefix}r1-sonnet-output.md"),
        ("scope-forward-writes-defense-report-of-another-round", "three-way-forward", with_receipt, f"printf x > {prefix}r2-sonnet-output.md"),
    )
    for name, agent_type, transcript, command in cases:
        claims_before = sorted(os.path.relpath(os.path.join(base, entry), root) for base, _dirs, files in os.walk(root) for entry in files
                               if entry.startswith(".writer-claim-"))
        run = feed("bash-command-detector.sh", payload_for(command, SETTINGS["frozen"], agent_type=agent_type, transcript=transcript),
                   extra={"SINGLEFS_WORK_ROOT": root}, label=name)
        claims_after = sorted(os.path.relpath(os.path.join(base, entry), root) for base, _dirs, files in os.walk(root) for entry in files
                              if entry.startswith(".writer-claim-"))
        emit(name, question=8, agent_type=agent_type, detector_exit=run.code, refused=yes_no(run.code == 2),
             by_writer_lock=yes_no("写者锁" in run.err), by_write_scope=yes_no("写范围" in run.err),
             claims_created=",".join(sorted(set(claims_after) - set(claims_before))) or "none", command=command)


def gate_lint_words():
    """整轮门禁真传给 clippy 的那张表：从被判那棵树的 check.sh 里取出 CODE_DISCIPLINE_LINTS=( … ) 那一段，交给 bash 自己展开。"""
    with open(os.path.join(SETTINGS["frozen"], ".claude", "singlefs-ai-sop", "scripts", "check.sh"), encoding="utf-8") as handle:
        text = handle.read()
    block = re.search(r"^CODE_DISCIPLINE_LINTS=\(.*?^\)", text, re.M | re.S).group(0)
    path = os.path.join(SETTINGS["work"], "gate-lint-table.sh")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(block + "\n")
    listed = subprocess.run(["bash", "-c", 'source "$1" && printf "%s\\0" "${CODE_DISCIPLINE_LINTS[@]}"', "table", path], capture_output=True, text=True, check=True)
    return [word for word in listed.stdout.split("\0") if word]


def lint_switch_cases():
    """第 9 问：LIB_CODE_DISCIPLINE_LINTS_BREAK 平常（不在自证里）设在钩子的环境里，两道钩子各判成什么。"""
    words = gate_lint_words()
    clippy = "task-server/task submit -- cargo clippy --all-targets -- " + shlex.join(["-D", "warnings", *words])
    build = "task-server/task submit -- cargo build --all-targets"
    for suffix, extra in (("unset-control", {}), ("set", {"LIB_CODE_DISCIPLINE_LINTS_BREAK": "first-word-per-line"})):
        guard = feed("project-code-guard.sh", payload_for(clippy, SETTINGS["frozen"]), extra=extra, label=f"lints-switch-{suffix}-guard")
        missing = re.search(r"`--` 之后缺 ([^（]*)（", guard.err)
        emit(f"lints-switch-{suffix}-project-code-guard-full-clippy", question=9, guard_exit=guard.code,
             refused_at_entry=yes_no("是弄坏开关" in guard.err), missing=missing.group(1) if missing else "none", command=clippy[:120])
        detector = feed("bash-command-detector.sh", payload_for(build, SETTINGS["frozen"]), extra=extra, label=f"lints-switch-{suffix}-detector")
        outlet = re.search(r"cargo clippy[^`]*?-- (-D warnings[^`]*)`", detector.err)
        emit(f"lints-switch-{suffix}-detector-outlet", question=9, detector_exit=detector.code, refused_at_entry=yes_no("是弄坏开关" in detector.err),
             outlet_equals_gate=yes_no(bool(outlet) and shlex.split(outlet.group(1)) == ["-D", "warnings", *words]),
             outlet=outlet.group(1)[:200] if outlet else "none")


def selftest_single_cell_cases():
    """第 10 问：两份新脚本能不能只跑一格（--selftest-inner 后面点一个格名）。"""
    for script, cell in (("large-file-git-hook.py", "commit-small-passes"), ("git-hooks-install.py", "not-installed-red")):
        run = run_own_group([sys.executable, "-B", os.path.join(SETTINGS["frozen"], "scripts", "gate", script), "--selftest-inner", cell],
                            SETTINGS["work"], git_environment(), limit_seconds=900)
        count = re.search(r"--selftest：(\d+) 格", run.out)
        emit(f"selftest-inner-with-cell-name-{script}", question=10, exit=run.code, cells_run=count.group(1) if count else "?",
             seconds=f"{run.seconds:.1f}", asked_for=cell)


def judged_hashes():
    listing = os.path.join(SETTINGS["repo"], "research", "prompts", "fg-timeout-large-file-governance-r3-snapshot", "judged-sha256.txt")
    with open(listing, encoding="utf-8") as handle:
        for line in handle:
            digest, _, relative = line.strip().partition("  ")
            path = os.path.join(SETTINGS["frozen"], relative)
            if not os.path.exists(path):
                emit("frozen-hash", file=relative, match="missing")
                continue
            with open(path, "rb") as source:
                actual = hashlib.sha256(source.read()).hexdigest()
            emit("frozen-hash", file=relative, match=yes_no(actual == digest))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True)
    parser.add_argument("--frozen", required=True)
    parser.add_argument("--work", required=True)
    parser.add_argument("--only", default="", help="逗号隔开的组名，只跑这几组（先跑一小段估时用）")
    arguments = parser.parse_args()
    work = os.path.realpath(arguments.work)
    if os.path.exists(work) and os.listdir(work):
        print(f"  ✗ --work {work} 已存在且非空", file=sys.stderr)
        return 2
    os.makedirs(work, exist_ok=True)
    SETTINGS.update(repo=os.path.realpath(arguments.repo), frozen=os.path.realpath(arguments.frozen), work=work, hook_log=os.path.join(work, "hook-calls.log"))
    with open(os.path.join(SETTINGS["frozen"], "scripts", "gate", "large-file-check.py"), encoding="utf-8") as handle:
        SETTINGS["limit"] = int(re.search(r"^SINGLE_FILE_SIZE_LIMIT_BYTES = (\d+)", handle.read(), re.M).group(1))
    for key in ("repositories", "remotes", "detections", "stderr"):
        SETTINGS[key] = os.path.join(work, key)
        os.makedirs(SETTINGS[key])
    SETTINGS["global_config"] = os.path.join(work, "gitconfig")
    with open(SETTINGS["global_config"], "w", encoding="utf-8") as handle:
        handle.write("[init]\n\tdefaultBranch = main\n[commit]\n\tgpgsign = false\n[tag]\n\tgpgsign = false\n[protocol \"file\"]\n\tallow = always\n"
                     "[advice]\n\tdetachedHead = false\n[push]\n\tdefault = simple\n")
    emit("settings", limit=SETTINGS["limit"], frozen=SETTINGS["frozen"], work=work,
         git=subprocess.run(["git", "--version"], capture_output=True, text=True).stdout.strip())
    judged_hashes()
    groups = [("bypass", bypass_cases), ("push-misreject", push_misreject_cases), ("push-guard", push_guard_cases), ("judge-runtime", judge_runtime_cases),
              ("scope", scope_cases), ("lint-switch", lint_switch_cases), ("selftest-single-cell", selftest_single_cell_cases)]
    chosen = {name for name in arguments.only.split(",") if name}
    for name, function in groups:
        if chosen and name not in chosen:
            continue
        started = time.monotonic()
        try:
            function()
        except Exception as error:   # 一组装置出错不吞：写一行、接着跑别的组，整轮退 1
            emit(f"group-error-{name}", error=repr(error)[:300])
            SETTINGS["failed"] = True
        emit(f"group-seconds-{name}", seconds=f"{time.monotonic() - started:.1f}")
    print(f"PROBE name=done emitted={EMITTED}", flush=True)
    return 1 if SETTINGS.get("failed") else 0


if __name__ == "__main__":
    sys.exit(main())
