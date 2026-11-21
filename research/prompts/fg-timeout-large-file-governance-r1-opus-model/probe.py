#!/usr/bin/env python3
"""攻方腿探针（fg-timeout-large-file-governance-r1，正文「要腿判的」第 5 到 11 问）。

用法：python3 probe.py --repo <仓根> --frozen <被判的那棵树> --work <新的空目录> [--model <模型目录>]
  --frozen 里的 .claude/hooks/bash-command-detector.sh 与 scripts/gate/large-file-check.py 是被判的两份；开头逐份对仓根
  research/prompts/fg-timeout-large-file-governance-r1-snapshot/judged-sha256.txt 核 sha256，对不上照跑、行里写 match=no。
  临时仓、裸远端、假判法树、检出记录都建在 --work 下；大文件是 SINGLE_FILE_SIZE_LIMIT_BYTES + 1 字节的稀疏文件（不可压缩那一格例外，写随机字节）。
  喂钩子与跑命令各起一个新会话（start_new_session），超时与收尾只对自己起的那一组发信号，跑完按组号在 /proc 里核留没留下进程。
  主仓只在「挂钟」那几格里当钩子的起点目录读（命令不执行，GIT_OPTIONAL_LOCKS=0 免得 git diff 顺手写回索引）。
输出：每格一行 `PROBE name=<格名> …`，末行 `PROBE name=done emitted=<行数>`（不含末行）。
退出码：0 跑完；2 用法错。
"""
import argparse
import hashlib
import json
import os
import re
import shlex
import shutil
import signal
import subprocess
import sys
import time

EMITTED = 0
SETTINGS = {}


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
    """/proc 里进程组号是 group 的进程号。"""
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
    def __init__(self, code, out, err, seconds, timed_out, leftover):
        self.code, self.out, self.err, self.seconds, self.timed_out, self.leftover = code, out, err, seconds, timed_out, leftover


def run_own_group(argv, cwd, env, stdin_text=None, limit_seconds=300):
    started = time.monotonic()
    child = subprocess.Popen(argv, cwd=cwd, env=env, encoding="utf-8", errors="replace", start_new_session=True,
                             stdin=subprocess.PIPE if stdin_text is not None else subprocess.DEVNULL,
                             stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    timed_out = False
    try:
        out, err = child.communicate(stdin_text, timeout=limit_seconds)
    except subprocess.TimeoutExpired:
        timed_out = True
        stop_own_group(child.pid)
        out, err = child.communicate()
    seconds = time.monotonic() - started
    leftover = group_members(child.pid)
    if leftover:
        stop_own_group(child.pid)
    return Outcome(child.returncode, out or "", err or "", seconds, timed_out, leftover)


def git_environment(extra=None, global_config=None):
    env = {name: value for name, value in os.environ.items()
           if not name.startswith(("GIT_", "BASH_COMMAND_DETECTOR_", "LARGE_FILE_CHECK_", "AGENT_HOOK_DETECTIONS"))}
    env.update(GIT_CONFIG_GLOBAL=global_config or SETTINGS["global_config"], GIT_CONFIG_NOSYSTEM="1",
               GIT_AUTHOR_NAME="probe", GIT_AUTHOR_EMAIL="probe@example.invalid",
               GIT_COMMITTER_NAME="probe", GIT_COMMITTER_EMAIL="probe@example.invalid",
               GIT_CEILING_DIRECTORIES=SETTINGS["work"], GIT_TERMINAL_PROMPT="0")
    env.update(extra or {})
    return env


def git(repository, *arguments, env_extra=None, check=True):
    completed = subprocess.run(["git", "-C", repository, *arguments], env=git_environment(env_extra),
                               capture_output=True, text=True)
    if check and completed.returncode != 0:
        raise RuntimeError(f"装置里的 git {' '.join(arguments)} 退 {completed.returncode}：{completed.stderr.strip()}")
    return completed.stdout


def write_small(repository, relative, text="small\n"):
    full = os.path.join(repository, relative)
    os.makedirs(os.path.dirname(full), exist_ok=True)
    with open(full, "w", encoding="utf-8") as handle:
        handle.write(text)


def make_big(repository, relative, size=None):
    """稀疏的超限文件：大小是上限加一字节（size 给了就照它）。"""
    full = os.path.join(repository, relative)
    os.makedirs(os.path.dirname(full), exist_ok=True)
    with open(full, "wb") as handle:
        handle.truncate(size if size is not None else SETTINGS["limit"] + 1)


def make_random_big(repository, relative):
    """不可压缩的超限文件：上限加一字节的随机字节。"""
    full = os.path.join(repository, relative)
    remaining = SETTINGS["limit"] + 1
    with open(full, "wb") as handle:
        while remaining:
            piece = min(remaining, 1 << 20)
            handle.write(os.urandom(piece))
            remaining -= piece


def new_repository(name, files=("base.txt",)):
    path = os.path.join(SETTINGS["repositories"], name)
    os.makedirs(path)
    git(path, "init", "-q")
    git(path, "symbolic-ref", "HEAD", "refs/heads/main")
    for relative in files:
        write_small(path, relative)
    git(path, "add", *files)
    git(path, "commit", "-q", "-m", "base")
    return path


def with_remote(repository, name):
    remote = os.path.join(SETTINGS["remotes"], name + ".git")
    subprocess.run(["git", "init", "-q", "--bare", remote], env=git_environment(), check=True, capture_output=True)
    git(repository, "remote", "add", "origin", remote)
    git(repository, "push", "-q", "-u", "origin", "main")
    return remote


def commits_of(repository):
    if not os.path.isdir(os.path.join(repository, ".git")):
        return set()
    listing = subprocess.run(["git", "-C", repository, "rev-list", "--branches", "--tags", "--remotes"],
                             env=git_environment(), capture_output=True, text=True)
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
    """这几笔提交相对各自父提交新增或改过的 blob 里，超过上限的个数。"""
    names = set()
    for commit in commits:
        output = subprocess.run(["git", "-C", repository, "diff-tree", "-r", "-m", "--root", "--no-commit-id", "--no-renames", commit],
                                env=git_environment(), capture_output=True, text=True).stdout
        for line in output.splitlines():
            if line.startswith(":"):
                names.add(line[1:].split()[3])
    return sum(1 for size in blob_sizes(repository, names).values() if size > SETTINGS["limit"])


def big_blobs_in_remote(remote):
    if not remote or not os.path.isdir(remote):
        return 0
    listing = subprocess.run(["git", "-C", remote, "rev-list", "--objects", "--all"], env=git_environment(),
                             capture_output=True, text=True).stdout
    names = {line.split()[0] for line in listing.splitlines() if line.strip()}
    return sum(1 for size in blob_sizes(remote, names).values() if size > SETTINGS["limit"])


def hook_environment(detections, extra=None, global_config=None):
    env = git_environment(global_config=global_config)
    env.update(CLAUDE_PROJECT_DIR=SETTINGS["frozen"], AGENT_HOOK_DETECTIONS=detections)
    env.update(extra or {})
    return env


def detections_text(path):
    try:
        with open(path, encoding="utf-8") as handle:
            return handle.read()
    except OSError:
        return ""


def feed_hook(hook, payload, env, limit_seconds=600):
    cwd = payload.get("cwd") if os.path.isdir(payload.get("cwd") or "") else SETTINGS["work"]
    return run_own_group(["bash", hook], cwd, env, stdin_text=json.dumps(payload), limit_seconds=limit_seconds)


def keep_stderr(name, text):
    with open(os.path.join(SETTINGS["stderr"], name + ".txt"), "w", encoding="utf-8") as handle:
        handle.write(text)


def payload_for(command, cwd, agent_type=None, **tool_extra):
    payload = {"tool_name": "Bash", "session_id": "probe", "cwd": cwd,
               "tool_input": {"command": command, "run_in_background": False, **tool_extra}}
    if agent_type:
        payload["agent_type"] = agent_type
    return payload


def large_file_case(name, question, cwd, command, repository, remote=None, hook=None, hook_extra=None,
                    run_extra=None, global_config=None, execute=True, payload=None):
    """喂钩子一次（不执行），再真执行这条命令一次，看超限的 blob 有没有进新提交或进远端；按两样合成结局。"""
    detections = os.path.join(SETTINGS["detections"], name + ".jsonl")
    commits_before = commits_of(repository)
    remote_before = big_blobs_in_remote(remote)
    hook_run = feed_hook(hook or SETTINGS["hook"], payload or payload_for(command, cwd),
                         hook_environment(detections, hook_extra, global_config))
    keep_stderr(name, hook_run.err)
    rejected = hook_run.code == 2 and "超过单文件上限" in hook_run.err
    recorded = "单文件上限" in detections_text(detections)
    command_code, landed = "not-run", "no"
    if execute:
        command_run = run_own_group(["bash", "-c", command], cwd, git_environment(run_extra, global_config))
        command_code = command_run.code
        if big_blobs_brought_by(repository, commits_of(repository) - commits_before):
            landed = "commit"
        if big_blobs_in_remote(remote) > remote_before:
            landed = "push" if landed == "no" else landed + "+push"
    if landed != "no":
        verdict = "rejected" if rejected else "passed-recorded" if recorded else "missed-silently"
    else:
        verdict = "rejected-nothing-lands" if rejected else "passed-recorded-nothing-lands" if recorded else "passed-nothing-lands"
    emit(name, question=question, hook_exit=hook_run.code, rejected=yes_no(rejected), recorded=yes_no(recorded),
         command_exit=command_code, landed=landed, verdict=verdict, hook_seconds=f"{hook_run.seconds:.2f}",
         hook_leftover=len(hook_run.leftover), command=command)
    return hook_run


def yes_no(flag):
    return "yes" if flag else "no"


def staged_big(name):
    repository = new_repository(name)
    make_big(repository, "big.bin")
    git(repository, "add", "big.bin")
    return repository


def other_branch_with_big(name):
    """main 是基准提交；分支 other 多一笔带 big.bin 的提交；停在 main 上。"""
    repository = new_repository(name)
    git(repository, "checkout", "-q", "-b", "other")
    make_big(repository, "big.bin")
    git(repository, "add", "big.bin")
    git(repository, "commit", "-q", "-m", "add big")
    git(repository, "checkout", "-q", "main")
    return repository


def commit_question_cases():
    """第 6 问：git commit 那一侧，超限的文件进了新提交而 ㉒ 放行又不记检出的写法。"""
    size = SETTINGS["limit"] + 1
    simple = [
        ("commit-control-staged", "git commit -qm x"),
        ("commit-amend-with-staged-big", "git commit -q --amend -m y"),
        ("commit-message-from-file", "printf 'x\\n' > ../message-for-commit.txt && git commit -q -F ../message-for-commit.txt"),
        ("commit-git-config-prefix", "git -c user.name=probe commit -qm x"),
        ("commit-command-builtin-prefix", "command git commit -qm x"),
        ("commit-command-line-alias", "git -c alias.ci=commit ci -qm x"),
        ("commit-command-line-shell-alias", "git -c 'alias.ci=!git commit' ci -qm x"),
        ("commit-through-xargs", "echo x | xargs git commit -qm"),
        ("commit-through-find-exec", "find . -maxdepth 0 -exec git commit -qm x ';'"),
        ("commit-through-eval", "eval 'git commit -qm x'"),
        ("commit-through-heredoc-to-bash", "bash <<'PROBE_EOS'\ngit commit -qm x\nPROBE_EOS"),
        ("commit-through-sh-c", "sh -c 'git commit -qm x'"),
        ("commit-abbreviated-message-option", "git commit -q --mes x"),
    ]
    for name, command in simple:
        repository = staged_big(name)
        large_file_case(name, 6, repository, command, repository)
    repository = staged_big("commit-repository-alias")
    git(repository, "config", "alias.ci", "commit")
    large_file_case("commit-repository-alias", 6, repository, "git ci -qm x", repository)
    repository = new_repository("commit-amend-keeps-head-big")
    make_big(repository, "big.bin")
    git(repository, "add", "big.bin")
    git(repository, "commit", "-q", "-m", "add big")
    large_file_case("commit-amend-keeps-head-big", 6, repository, "git commit -q --amend -m y", repository)
    for name, stash_command in (("commit-after-stash-pop", "git stash pop -q"), ("commit-after-stash-apply-index", "git stash apply -q --index")):
        repository = staged_big(name)
        git(repository, "stash", "-q")
        large_file_case(name, 6, repository, f"{stash_command} && git commit -qm x", repository)
    repository = new_repository("commit-add-update-chain")
    make_big(repository, "base.txt")
    large_file_case("commit-add-update-chain", 6, repository, "git add -u && git commit -qm x", repository)
    repository = staged_big("commit-abbreviated-include-option")
    write_small(repository, "base.txt", "changed\n")
    large_file_case("commit-abbreviated-include-option", 6, repository, "git commit -q -m x --inc base.txt", repository)
    repository = new_repository("commit-abbreviated-force-add", files=("base.txt", ".gitignore"))
    write_small(repository, ".gitignore", "ignored.bin\n")
    git(repository, "commit", "-q", "-am", "ignore")
    make_big(repository, "ignored.bin")
    large_file_case("commit-abbreviated-force-add", 6, repository, "git add --forc ignored.bin && git commit -qm x", repository)
    repository = new_repository("commit-file-created-in-same-command")
    large_file_case("commit-file-created-in-same-command", 6, repository,
                    f"truncate -s {size} gen.bin && git add gen.bin && git commit -qm x", repository)
    repository = new_repository("commit-all-file-grown-in-same-command")
    large_file_case("commit-all-file-grown-in-same-command", 6, repository, f"truncate -s {size} base.txt && git commit -qam x", repository)
    fresh = os.path.join(SETTINGS["repositories"], "commit-repository-made-in-same-command")
    large_file_case("commit-repository-made-in-same-command", 6, SETTINGS["repositories"],
                    f"mkdir {fresh} && cd {fresh} && git init -q && truncate -s {size} big.bin && git add big.bin && git commit -qm x", fresh)
    for name, command in (("commit-after-cherry-pick-no-commit", "git cherry-pick -n other && git commit -qm x"),
                          ("commit-after-checkout-path-from-branch", "git checkout other -- big.bin && git commit -qm x"),
                          ("commit-after-merge-squash", "git merge -q --squash other && git commit -qm x")):
        repository = other_branch_with_big(name)
        large_file_case(name, 6, repository, command, repository)
    repository = new_repository("commit-redo-after-reset-soft")
    make_big(repository, "big.bin")
    git(repository, "add", "big.bin")
    git(repository, "commit", "-q", "-m", "add big")
    large_file_case("commit-redo-after-reset-soft", 6, repository, "git reset -q --soft HEAD~1 && git commit -qm redo", repository)
    repository = new_repository("commit-exported-private-index")
    make_big(repository, "big.bin")
    index = os.path.join(SETTINGS["work"], "exported-private.index")
    git(repository, "read-tree", "HEAD", env_extra={"GIT_INDEX_FILE": index})
    git(repository, "add", "big.bin", env_extra={"GIT_INDEX_FILE": index})
    large_file_case("commit-exported-private-index", 6, repository, f"export GIT_INDEX_FILE={index} && git commit -qm x", repository)
    for name, index_home in (("commit-relative-private-index-kept-at-top", "."), ("commit-relative-private-index-kept-in-subdirectory", "sub")):
        repository = new_repository(name, files=("base.txt", "sub/keep.txt"))
        make_big(repository, "big.bin")
        index = os.path.join(repository, index_home, "probe.idx")
        git(repository, "read-tree", "HEAD", env_extra={"GIT_INDEX_FILE": index})
        git(repository, "add", "big.bin", env_extra={"GIT_INDEX_FILE": index})
        large_file_case(name, 6, os.path.join(repository, "sub"), "GIT_INDEX_FILE=probe.idx git commit -qm x", repository)
    repository = staged_big("commit-after-pushd")
    large_file_case("commit-after-pushd", 6, SETTINGS["repositories"], f"pushd {repository} >/dev/null && git commit -qm x", repository)
    relative_config = os.path.join(SETTINGS["work"], "gitconfig-diff-relative")
    shutil.copyfile(SETTINGS["global_config"], relative_config)
    with open(relative_config, "a", encoding="utf-8") as handle:
        handle.write("[diff]\n\trelative = true\n")
    for name, config in (("commit-path-from-subdirectory-control", None), ("commit-path-from-subdirectory-diff-relative-config", relative_config)):
        repository = new_repository(name, files=("base.txt", "sub/data.bin"))
        make_big(repository, "sub/data.bin")
        large_file_case(name, 6, os.path.join(repository, "sub"), "git commit -qm x data.bin", repository, global_config=config)


def push_ready(name):
    """main 跟着 origin/main；没推的两笔：一笔加了 big.bin，一笔又删了它（自证 large-push-bad 那一形，用真大小）。"""
    repository = new_repository(name)
    remote = with_remote(repository, name)
    make_big(repository, "big.bin")
    git(repository, "add", "big.bin")
    git(repository, "commit", "-q", "-m", "add big")
    git(repository, "rm", "-q", "big.bin")
    git(repository, "commit", "-q", "-m", "drop big")
    return repository, remote


def clean_main_with_dirty_feature(name, tag=False):
    """main 跟着上游、什么都没欠；分支 feature 上一笔带 big.bin 的提交没推；停在 main 上。"""
    repository = new_repository(name)
    remote = with_remote(repository, name)
    git(repository, "checkout", "-q", "-b", "feature")
    make_big(repository, "big.bin")
    git(repository, "add", "big.bin")
    git(repository, "commit", "-q", "-m", "add big")
    if tag:
        git(repository, "tag", "heavy")
    git(repository, "checkout", "-q", "main")
    return repository, remote


def push_question_cases():
    """第 6 问 push 那一侧与第 7 问 push 的误拒。"""
    for name, command in (("push-control", "git push -q"), ("push-head-to-other-ref", "git push -q origin HEAD:other")):
        repository, remote = push_ready(name)
        large_file_case(name, 6, repository, command, repository, remote=remote)
    repository = new_repository("push-new-branch-first-time")
    remote = with_remote(repository, "push-new-branch-first-time")
    git(repository, "checkout", "-q", "-b", "feature")
    make_big(repository, "big.bin")
    git(repository, "add", "big.bin")
    git(repository, "commit", "-q", "-m", "add big")
    large_file_case("push-new-branch-first-time", 6, repository, "git push -q -u origin feature", repository, remote=remote)
    for name, command, tag in (("push-named-other-branch", "git push -q origin feature", False),
                               ("push-all-branches", "git push -q --all origin", False),
                               ("push-mirror", "git push -q --mirror origin", False),
                               ("push-tags", "git push -q origin --tags", True)):
        repository, remote = clean_main_with_dirty_feature(name, tag=tag)
        large_file_case(name, 6, repository, command, repository, remote=remote)
    sub_remote = os.path.join(SETTINGS["remotes"], "push-recurse-submodules-sub.git")
    super_remote = os.path.join(SETTINGS["remotes"], "push-recurse-submodules-super.git")
    seed = new_repository("push-recurse-submodules-seed")
    for bare in (sub_remote, super_remote):
        subprocess.run(["git", "init", "-q", "--bare", bare], env=git_environment(), check=True, capture_output=True)
    git(seed, "remote", "add", "origin", sub_remote)
    git(seed, "push", "-q", "origin", "main")
    superproject = new_repository("push-recurse-submodules-super")
    git(superproject, "remote", "add", "origin", super_remote)
    git(superproject, "submodule", "--quiet", "add", sub_remote, "sub")
    git(superproject, "commit", "-q", "-m", "add submodule")
    git(superproject, "push", "-q", "-u", "origin", "main")
    submodule = os.path.join(superproject, "sub")
    git(submodule, "checkout", "-q", "main")
    make_big(submodule, "big.bin")
    git(submodule, "add", "big.bin")
    git(submodule, "commit", "-q", "-m", "add big in submodule")
    git(superproject, "add", "sub")
    git(superproject, "commit", "-q", "-m", "bump submodule")
    large_file_case("push-recurse-submodules", 6, superproject, "git push -q --recurse-submodules=on-demand", submodule, remote=sub_remote)
    repository, remote = push_ready("push-delete-other-ref-while-current-dirty")
    git(repository, "push", "-q", "origin", "main~2:refs/heads/stale")
    large_file_case("push-delete-other-ref-while-current-dirty", 7, repository, "git push -q origin :stale", repository, remote=remote)
    repository, remote = push_ready("push-clean-side-branch-while-current-dirty")
    git(repository, "branch", "side", "main~2")
    large_file_case("push-clean-side-branch-while-current-dirty", 7, repository, "git push -q origin side", repository, remote=remote)


def false_reject_cases():
    """第 7 问：没暂存的改大文件、被忽略的大文件、带空格与非 ASCII 的路径、软链接指到仓外大文件。"""
    repository = new_repository("false-unstaged-grown-plain-commit")
    make_big(repository, "base.txt")
    write_small(repository, "new.txt")
    git(repository, "add", "new.txt")
    large_file_case("false-unstaged-grown-plain-commit", 7, repository, "git commit -qm x", repository)
    repository = new_repository("false-ignored-big-add-all", files=("base.txt", ".gitignore"))
    write_small(repository, ".gitignore", "ignored.bin\n")
    git(repository, "commit", "-q", "-am", "ignore")
    make_big(repository, "ignored.bin")
    write_small(repository, "new.txt")
    large_file_case("false-ignored-big-add-all", 7, repository, "git add -A && git commit -qm x", repository)
    repository = new_repository("path-with-space-and-non-ascii")
    make_big(repository, "目录 名/大 文件.bin")
    git(repository, "add", "目录 名/大 文件.bin")
    large_file_case("path-with-space-and-non-ascii", 7, repository, "git commit -qm x", repository)
    outside = os.path.join(SETTINGS["work"], "outside")
    make_big(outside, "huge.bin")
    for name, command, stage in (("symlink-to-outside-big-staged", "git commit -qm x", True),
                                 ("symlink-to-outside-big-add-all", "git add -A && git commit -qm x", False)):
        repository = new_repository(name)
        os.symlink(os.path.join(outside, "huge.bin"), os.path.join(repository, "link.bin"))
        if stage:
            git(repository, "add", "link.bin")
        large_file_case(name, 7, repository, command, repository)


def bash_hooks():
    """被判那棵树 .claude/settings.json 里 PreToolUse 挂在 Bash 上的每一条钩子命令。"""
    with open(os.path.join(SETTINGS["frozen"], ".claude", "settings.json"), encoding="utf-8") as handle:
        settings = json.load(handle)
    found = []
    for matcher_entry in settings.get("hooks", {}).get("PreToolUse", []):
        if "Bash" in (matcher_entry.get("matcher") or "").split("|"):
            found += [hook["command"] for hook in matcher_entry.get("hooks", []) if hook.get("type") == "command"]
    return found


def every_bash_hook_on(name, command, cwd):
    """把一条出路命令喂给 PreToolUse[Bash] 上的每一条钩子（主 agent 的写法），交被拒的钩子名单。"""
    refused = []
    for index, hook_command in enumerate(bash_hooks()):
        detections = os.path.join(SETTINGS["detections"], f"{name}-hook{index}.jsonl")
        run = run_own_group(["bash", "-c", hook_command], cwd, hook_environment(detections),
                            stdin_text=json.dumps(payload_for(command, cwd)), limit_seconds=600)
        if run.code == 2:
            refused.append(hook_command.split("/")[-1])
        keep_stderr(f"{name}-hook{index}", run.err)
    return refused


def remedy_commands(stderr_text):
    """㉒ 拒绝说明「→ 怎么办」那一行里反引号括着的命令，照列出的次序。"""
    for line in stderr_text.splitlines():
        if line.strip().startswith("→ 怎么办：超限的文件逐个："):
            return re.findall(r"`([^`]+)`", line)
    return []


def follow_remedy(name, repository, random_bytes=False):
    """第 9 问：照拒绝说明给的出路一条一条照抄执行（每条先过 PreToolUse[Bash] 上的全部钩子），做完再 git commit。"""
    relative = "目录 名/大 文件.bin"
    os.makedirs(os.path.join(repository, "目录 名"), exist_ok=True)
    if random_bytes:
        make_random_big(repository, relative)
    else:
        make_big(repository, relative)
    git(repository, "add", relative)
    first = feed_hook(SETTINGS["hook"], payload_for("git commit -qm x", repository),
                      hook_environment(os.path.join(SETTINGS["detections"], name + "-first.jsonl")))
    keep_stderr(name + "-first", first.err)
    steps = remedy_commands(first.err)
    emit(name + "-refused-before-remedy", question=9, hook_exit=first.code, remedy_steps=len(steps), steps=" ;; ".join(steps))
    for position, step in enumerate(steps):
        refused_by = every_bash_hook_on(f"{name}-step{position}", step, repository)
        run = run_own_group(["bash", "-c", step], repository, git_environment())
        emit(f"{name}-step", question=9, step=step, refused_by=",".join(refused_by) or "none", step_exit=run.code,
             step_stderr=run.err.strip()[:160])
    gz = os.path.join(repository, relative + ".gz")
    emit(name + "-gzip-size", question=9, gz_bytes=os.path.getsize(gz) if os.path.exists(gz) else "missing", limit=SETTINGS["limit"])
    large_file_case(name + "-commit-after-remedy", 9, repository, "git commit -qm x", repository)


def remedy_question_cases():
    follow_remedy("remedy-followed-step-by-step", new_repository("remedy-followed-step-by-step"))
    follow_remedy("remedy-incompressible", new_repository("remedy-incompressible"), random_bytes=True)
    repository = staged_big("remedy-chained-in-one-command")
    large_file_case("remedy-chained-in-one-command", 9, repository,
                    "gzip -n -9 -k big.bin && git rm -q --cached big.bin && git add big.bin.gz && git commit -qm x", repository)
    repository, remote = push_ready("remedy-push-message")
    hook_run = feed_hook(SETTINGS["hook"], payload_for("git push -q", repository),
                         hook_environment(os.path.join(SETTINGS["detections"], "remedy-push-message.jsonl")))
    keep_stderr("remedy-push-message", hook_run.err)
    emit("remedy-push-message", question=9, hook_exit=hook_run.code,
         names_a_redo_command=yes_no(bool(re.search(r"`git (reset|rebase|commit-tree)", hook_run.err))),
         steps=" ;; ".join(re.findall(r"`([^`]+)`", hook_run.err)))


def timeout_question_cases():
    """第 5 问：㉑ 认 timeout 的各种写法；run_in_background 写成别的样子。命令是 true，不执行。"""
    huge = 10 ** 400   # 400 位的整数：hook 里 float() 转不了
    forms = [("integer-600000", 600000), ("float-240000-point-5", 240000.5), ("negative", -1), ("json-exponent-number", RAW_EXPONENT),
             ("string-exponent", "3" + "e" + "5"), ("string-with-unit", "300000ms"), ("string-with-spaces", " 300000 "),
             ("string-trailing-dot", "300000."), ("string-plus-sign", "+300000"), ("string-hexadecimal", "0x493e0"),
             ("integer-with-400-digits", huge), ("float-infinity", float("inf")), ("float-not-a-number", float("nan")),
             ("float-exactly-limit", 240000.0), ("boolean-true", True)]
    for label, value in forms:
        detections = os.path.join(SETTINGS["detections"], f"timeout-{label}.jsonl")
        payload = payload_for("true", SETTINGS["work"], timeout=value)
        text = json.dumps(payload).replace(json.dumps(RAW_EXPONENT), "3" + "e" + "5")
        run = run_own_group(["bash", SETTINGS["hook"]], SETTINGS["work"], hook_environment(detections), stdin_text=text)
        emit(f"timeout-{label}", question=5, hook_exit=run.code, refused_21=yes_no("前台命令的 timeout" in run.err),
             recorded=yes_no("timeout" in detections_text(detections)), shown=refused_value(run.err))
    for label, agent, background in (("background-string-true-main", None, "true"), ("background-string-true-subagent", "clerk", "true"),
                                     ("background-integer-one-main", None, 1), ("background-boolean-true-subagent", "clerk", True)):
        detections = os.path.join(SETTINGS["detections"], f"timeout-{label}.jsonl")
        payload = payload_for("true", SETTINGS["work"], agent_type=agent, timeout=600000)
        payload["tool_input"]["run_in_background"] = background
        run = feed_hook(SETTINGS["hook"], payload, hook_environment(detections))
        emit(f"timeout-{label}", question=5, hook_exit=run.code, refused_21=yes_no("前台命令的 timeout" in run.err),
             refused_10=yes_no("用 run_in_background 起命令" in run.err))


def refused_value(stderr_text):
    found = re.search(r"timeout 是 (\S+) 毫秒", stderr_text)
    return found.group(1)[:40] if found else "none"


RAW_EXPONENT = "probe-raw-exponent-sentinel"   # 换成 JSON 数的指数写法再喂（json.dumps 打不出指数写法的 300000）
LIMIT_VARIABLE = "BASH_COMMAND_DETECTOR_LARGE_FILE_LIMIT_BYTES"
MARK_VARIABLE = "BASH_COMMAND_DETECTOR_SELFTEST_MARK"
MARK_CONTENT = "bash-command-detector-selftest"


def source_of(relative):
    """被判那棵树里的一份；那棵树里没有（最小副本不带规范副本）就取仓根里的。"""
    frozen = os.path.join(SETTINGS["frozen"], relative)
    return frozen if os.path.exists(frozen) else os.path.join(SETTINGS["repo"], relative)


def tree_copy(destination, with_sop=True, judge_source=None):
    """钩子能起来的最小一棵树：hooks、.claude/scripts、规范副本的 scripts（with_sop）、判据库，与 scripts/gate/large-file-check.py。"""
    skip = shutil.ignore_patterns("__pycache__")
    parts = [".claude/hooks", ".claude/scripts"] + ([".claude/singlefs-ai-sop/scripts"] if with_sop else [])
    for relative in parts:
        shutil.copytree(source_of(relative), os.path.join(destination, relative), ignore=skip, symlinks=True)
    for relative in (".claude/gate.d/lib-forbidden-notations.py", ".claude/settings.json"):
        os.makedirs(os.path.dirname(os.path.join(destination, relative)), exist_ok=True)
        shutil.copyfile(source_of(relative), os.path.join(destination, relative))
    os.makedirs(os.path.join(destination, "scripts", "gate"), exist_ok=True)
    shutil.copyfile(judge_source or source_of("scripts/gate/large-file-check.py"), os.path.join(destination, "scripts", "gate", "large-file-check.py"))
    return os.path.join(destination, ".claude", "hooks", "bash-command-detector.sh")


def fake_judge_cases():
    """第 7 问的起几次、合计多久，第 8 问判法出错的几种：钩子起的判法换成假的（PROBE_FAKE_MODE 选行为），命令不执行。"""
    tree = os.path.join(SETTINGS["work"], "trees", "fake-judge")
    hook = tree_copy(tree, judge_source=os.path.join(SETTINGS["model"], "fake_large_file_check.py"))
    calls = os.path.join(SETTINGS["work"], "fake-calls.log")
    open(calls, "w").close()
    repositories = [new_repository(f"fake-judge-{label}", files=("a.txt", "b.txt")) for label in ("left", "middle", "right")]
    for repository in repositories:
        write_small(repository, "a.txt", "changed\n")
        write_small(repository, "b.txt", "changed\n")
    left = repositories[0]
    count_commands = [
        ("plain-commit", "git commit -qm x"), ("commit-all", "git commit -qam x"),
        ("two-adds-then-commit-all", "git add a.txt && git add b.txt && git commit -qam x"),
        ("commit-all-twice", "git commit -qam x && git commit -qam y"), ("push", "git push -q"),
        ("commit-all-in-three-repositories", " && ".join(f"git -C {repository} commit -qam x" for repository in repositories)),
        ("add-commit-push", "git add a.txt && git commit -qm x && git push -q")]
    for label, command in count_commands:
        before = sum(1 for _ in open(calls, encoding="utf-8"))
        run = feed_hook(hook, payload_for(command, left), hook_environment(os.path.join(SETTINGS["detections"], f"fake-count-{label}.jsonl"),
                                                                       {"PROBE_FAKE_MODE": "empty", "PROBE_FAKE_CALLS": calls}))
        after = sum(1 for _ in open(calls, encoding="utf-8"))
        emit(f"judge-runs-{label}", question=7, judge_runs=after - before, hook_exit=run.code, hook_seconds=f"{run.seconds:.2f}", command=command)
    for mode in ("raise", "json-then-noise", "noise-then-big", "exit-three", "keys-missing", "slow-with-child"):
        command = "git add a.txt && git add b.txt && git commit -qam x" if mode == "slow-with-child" else "git commit -qm x"
        detections = os.path.join(SETTINGS["detections"], f"fake-mode-{mode}.jsonl")
        run = feed_hook(hook, payload_for(command, left, timeout=600000),
                        hook_environment(detections, {"PROBE_FAKE_MODE": mode, "PROBE_FAKE_CALLS": calls}))
        keep_stderr(f"fake-mode-{mode}", run.err)
        recorded = detections_text(detections)
        emit(f"judge-fails-{mode}", question=8, hook_exit=run.code, refused_21=yes_no("前台命令的 timeout" in run.err),
             refused_22=yes_no("超过单文件上限" in run.err), recorded_22=yes_no("单文件上限" in recorded),
             recorded_hook_layer=yes_no("python 那一层" in run.err or "bash 那一层" in recorded),
             hook_seconds=f"{run.seconds:.2f}", leftover_after_hook=len(run.leftover), command=command)


def limit_variable_cases():
    """第 10 问：门限压小的环境变量平常跑认不认；自证标记是一份内容固定的文件，自己做一份也对得上。"""
    mark = os.path.join(SETTINGS["work"], "made-mark.txt")
    with open(mark, "w", encoding="utf-8") as handle:
        handle.write(MARK_CONTENT + "\n")
    for name, size, extra in (("limit-variable-without-mark", 2000, {LIMIT_VARIABLE: "1000"}),
                              ("limit-variable-with-made-mark", 2000, {LIMIT_VARIABLE: "1000", MARK_VARIABLE: mark}),
                              ("limit-raised-with-made-mark", None, {LIMIT_VARIABLE: str(10 ** 15), MARK_VARIABLE: mark})):
        repository = new_repository(name)
        make_big(repository, "big.bin", size=size)
        git(repository, "add", "big.bin")
        large_file_case(name, 10, repository, "git commit -qm x", repository, hook_extra=extra, execute=False)


def tree_question_cases():
    """第 11 问：钩子从哪棵树起，就起那棵树里的 large-file-check.py；git worktree 不带规范副本时判法起不来。第 8 问 git 不在 PATH。"""
    main_tree = os.path.join(SETTINGS["work"], "trees", "main-tree")
    main_hook = tree_copy(main_tree)
    with open(os.path.join(main_tree, ".gitignore"), "w", encoding="utf-8") as handle:
        handle.write("/.claude/singlefs-ai-sop/\n__pycache__/\n")
    git(main_tree, "init", "-q")
    git(main_tree, "add", "-A")
    git(main_tree, "commit", "-q", "-m", "tree")
    linked = os.path.join(SETTINGS["work"], "trees", "linked-worktree")
    git(main_tree, "worktree", "add", "-q", linked)
    linked_hook = os.path.join(linked, ".claude", "hooks", "bash-command-detector.sh")
    for name, hook, project in (("tree-hook-from-main-tree", main_hook, main_tree), ("tree-hook-from-linked-worktree", linked_hook, linked)):
        repository = staged_big(name)
        large_file_case(name, 11, repository, "git commit -qm x", repository, hook=hook, hook_extra={"CLAUDE_PROJECT_DIR": project}, execute=False)
    without_git = os.path.join(SETTINGS["work"], "bin-without-git")
    os.makedirs(without_git)
    for directory in ("/usr/local/bin", "/usr/bin", "/bin"):
        for entry in sorted(os.listdir(directory)) if os.path.isdir(directory) else []:
            target = os.path.join(without_git, entry)
            if not entry.startswith("git") and not os.path.lexists(target):
                os.symlink(os.path.join(directory, entry), target)
    repository = staged_big("judge-without-git-on-path")
    large_file_case("judge-without-git-on-path", 8, repository, "git commit -qm x", repository, hook_extra={"PATH": without_git}, execute=False)


def main_repository_timing_cases():
    """第 7 问：钩子在仓根上判 git commit / push 花多久（命令不执行，GIT_OPTIONAL_LOCKS=0；git 配置照这台主机的）。"""
    env = {name: value for name, value in os.environ.items() if not name.startswith(("BASH_COMMAND_DETECTOR_", "LARGE_FILE_CHECK_"))}
    env.update(GIT_OPTIONAL_LOCKS="0", CLAUDE_PROJECT_DIR=SETTINGS["frozen"],
               AGENT_HOOK_DETECTIONS=os.path.join(SETTINGS["detections"], "main-repository-timing.jsonl"))
    for label, command in (("no-git-word", "ls"), ("plain-commit", "git commit -m x"), ("commit-all", "git commit -a -m x"),
                           ("add-all-then-commit-all", "git add -A && git commit -a -m x"), ("push", "git push")):
        seconds, codes = [], set()
        for _repeat in range(3):
            run = feed_hook(SETTINGS["hook"], payload_for(command, SETTINGS["repo"]), env)
            seconds.append(run.seconds)
            codes.add(run.code)
        emit(f"main-repository-hook-seconds-{label}", question=7, fastest=f"{min(seconds):.2f}", slowest=f"{max(seconds):.2f}",
             hook_exits=",".join(str(code) for code in sorted(codes)), command=command)


def index_state(repository):
    path = os.path.join(repository, ".git", "index")
    with open(path, "rb") as handle:
        return os.stat(path).st_mtime_ns, hashlib.sha256(handle.read()).hexdigest()


def index_refresh_cases():
    """第 7 问：判 commit-worktree 时 git diff 会不会顺手写回索引（拿 index.lock）：只动了时间戳、内容没变的跟踪文件。命令不执行。"""
    for name, extra in (("index-written-back-by-judge", {}), ("index-written-back-by-judge-optional-locks-off", {"GIT_OPTIONAL_LOCKS": "0"})):
        repository = new_repository(name)
        moment = time.time() + 5
        os.utime(os.path.join(repository, "base.txt"), (moment, moment))
        before = index_state(repository)
        run = feed_hook(SETTINGS["hook"], payload_for("git commit -qam x", repository),
                        hook_environment(os.path.join(SETTINGS["detections"], name + ".jsonl"), extra))
        after = index_state(repository)
        emit(name, question=7, hook_exit=run.code, index_mtime_changed=yes_no(before[0] != after[0]),
             index_bytes_changed=yes_no(before[1] != after[1]))


def judged_hashes():
    listing = os.path.join(SETTINGS["repo"], "research", "prompts", "fg-timeout-large-file-governance-r1-snapshot", "judged-sha256.txt")
    with open(listing, encoding="utf-8") as handle:
        for line in handle:
            digest, _, relative = line.strip().partition("  ")
            path = os.path.join(SETTINGS["frozen"], relative)
            if not os.path.exists(path):
                emit("frozen-hash", file=relative, match="missing")
                continue
            with open(path, "rb") as source:
                actual = hashlib.sha256(source.read()).hexdigest()
            emit("frozen-hash", file=relative, match=yes_no(actual == digest), sha256=actual)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True)
    parser.add_argument("--frozen", required=True)
    parser.add_argument("--work", required=True)
    parser.add_argument("--model", default=os.path.dirname(os.path.realpath(__file__)))
    parser.add_argument("--only", default="", help="逗号隔开的组名，只跑这几组（给先跑一小段估时用）")
    arguments = parser.parse_args()
    work = os.path.realpath(arguments.work)
    if os.path.exists(work) and os.listdir(work):
        print(f"  ✗ --work {work} 已存在且非空", file=sys.stderr)
        return 2
    SETTINGS.update(repo=os.path.realpath(arguments.repo), frozen=os.path.realpath(arguments.frozen), work=work,
                    model=os.path.realpath(arguments.model),
                    hook=os.path.join(os.path.realpath(arguments.frozen), ".claude", "hooks", "bash-command-detector.sh"))
    with open(os.path.join(SETTINGS["frozen"], "scripts", "gate", "large-file-check.py"), encoding="utf-8") as handle:
        SETTINGS["limit"] = int(re.search(r"^SINGLE_FILE_SIZE_LIMIT_BYTES = (\d+)", handle.read(), re.M).group(1))
    for key in ("repositories", "remotes", "detections", "stderr", "trees"):
        SETTINGS[key] = os.path.join(work, key)
        os.makedirs(SETTINGS[key])
    SETTINGS["global_config"] = os.path.join(work, "gitconfig")
    with open(SETTINGS["global_config"], "w", encoding="utf-8") as handle:
        handle.write("[init]\n\tdefaultBranch = main\n[commit]\n\tgpgsign = false\n[protocol \"file\"]\n\tallow = always\n"
                     "[advice]\n\tdetachedHead = false\n[push]\n\tdefault = simple\n")
    emit("settings", limit=SETTINGS["limit"], frozen=SETTINGS["frozen"], work=work, git=subprocess.run(["git", "--version"], capture_output=True, text=True).stdout.strip())
    judged_hashes()
    groups = [("timeout", timeout_question_cases), ("commit", commit_question_cases), ("push", push_question_cases),
              ("false-reject", false_reject_cases), ("remedy", remedy_question_cases), ("fake-judge", fake_judge_cases),
              ("limit-variable", limit_variable_cases), ("tree", tree_question_cases), ("index-refresh", index_refresh_cases),
              ("main-repository-timing", main_repository_timing_cases)]
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
