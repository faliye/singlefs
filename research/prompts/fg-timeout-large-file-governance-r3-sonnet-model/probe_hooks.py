#!/usr/bin/env python3
"""正推腿第三轮探针：git 钩子判的是不是「那一刻真要进提交、真要推出去的内容」，装与核对，日常提交写法。

用法：python3 probe_hooks.py --repo <仓根> --work <新的空目录> [--group commit,push,install,detector,window]
  --repo  被判的树：读它的 scripts/gate/ 与 .claude/hooks/；开头先拿 research/prompts/fg-timeout-large-file-governance-r3-snapshot/judged-sha256.txt 逐份核。
  每一格在 --work 下自己建的沙盒仓里跑真 git；沙盒仓的 core.hooksPath 指到 --repo 里进仓的钩子目录（绝对路径），或指到沙盒里提交进去的钩子目录
  （相对路径，里面的三份只 exec --repo 里的真钩子：用来量「这一棵树里有没有钩子目录」）。
  超限的文件是真的 104857601 字节（稀疏文件，进 git 之后压得很小）：钩子的限额只在自证里认，这里不造假。
  探针自己的命令行里没有 --selftest，所以钩子里的 inside_selftest() 是假，弄坏开关与假限额都不生效。
输出：每格一行 `PROBE name=<格名> key=value …`，末行 `PROBE name=done emitted=<行数>`。字段：
  rc 命令退出码；refused 钩子的拒绝说明（「单文件上限」）出现在标准错误；landed_commit 超限的 blob 进了 HEAD 可达的历史；landed_remote 超限的 blob 进了远端；
  expect 文件头、拒绝说明与 agent-common 第 102 行承诺的结局（refused 钩子拦、lands 超限的 blob 进来了钩子没拦、passes 没有超限的东西）；verdict 与 expect 对上写 as-expected，对不上写 differs-from-expected。
  window 一组与 check-lists-old-worktree 的 expect 是「窗口闸只管 git commit 与合并提交」「check 退 0 并列出」这类对设计的读法，不是承诺。
退出码：0 跑完；1 有一格装置出错；2 用法错。
"""
import argparse
import json
import os
import re
import signal
import subprocess
import sys
import time

sys.dont_write_bytecode = True
LIMIT = 104857600
BIG = LIMIT + 1
SETTINGS = {}
EMITTED = [0]
SNAPSHOT_LIST = "research/prompts/fg-timeout-large-file-governance-r3-snapshot/judged-sha256.txt"
HOOK_NAMES = ("pre-commit", "pre-push", "pre-merge-commit")


class Done:
    def __init__(self, code, out, err, seconds):
        self.code, self.out, self.err, self.seconds = code, out, err, seconds


def emit(name, **fields):
    EMITTED[0] += 1
    print("PROBE " + " ".join([f"name={name}"] + [f"{key}={json.dumps(value, ensure_ascii=False)}" for key, value in fields.items()]), flush=True)


def git_env(extra=None):
    """git 读不到用户与系统的配置；TMPDIR 指到 --work 下，钩子留下的索引拷贝在那里数得到。"""
    env = {name: value for name, value in os.environ.items() if not name.startswith("GIT_") and name != "SINGLEFS_COMMIT_OUTSIDE_WINDOW"}
    env.update(GIT_CONFIG_GLOBAL="/dev/null", GIT_CONFIG_NOSYSTEM="1", GIT_AUTHOR_NAME="p", GIT_AUTHOR_EMAIL="p@p.invalid", GIT_COMMITTER_NAME="p",
               GIT_COMMITTER_EMAIL="p@p.invalid", GIT_TERMINAL_PROMPT="0", GIT_EDITOR="true", GIT_MERGE_AUTOEDIT="no",
               GIT_CEILING_DIRECTORIES=SETTINGS["work"], TMPDIR=SETTINGS["tmp"])
    env.update(extra or {})
    return env


def run(argv, cwd, env=None, stdin_text=None, seconds=300):
    """放进自己的会话起，到点只对这一个子进程的组发信号。"""
    started = time.monotonic()
    process = subprocess.Popen(argv, cwd=cwd, env=env if env is not None else git_env(), stdin=subprocess.PIPE if stdin_text is not None else subprocess.DEVNULL,
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, start_new_session=True)
    try:
        out, err = process.communicate(stdin_text, timeout=seconds)
        code = process.returncode
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGTERM)
        try:
            out, err = process.communicate(timeout=5)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            out, err = process.communicate()
        code = 124
    return Done(code, out, err, time.monotonic() - started)


def git(repository, *arguments, check=True, env=None, seconds=300):
    done = run(["git", "-C", repository, *arguments], repository, env=git_env(env), seconds=seconds)
    if check and done.code != 0:
        raise RuntimeError(f"装置里的 git {' '.join(arguments)} 退 {done.code}：{done.err.strip()[:300]}")
    return done


def verify_snapshot(repo):
    """被判的文件逐份对开工快照的 sha256。"""
    path = os.path.join(repo, SNAPSHOT_LIST)
    try:
        lines = open(path, encoding="utf-8").read().splitlines()
    except OSError:
        emit("frozen-hash", path=SNAPSHOT_LIST, match="no", reason="清单读不到")
        return
    for line in lines:
        digest, _, relative = line.partition("  ")
        if not relative or relative.startswith("research/prompts/_"):
            continue
        done = run(["sha256sum", os.path.join(repo, relative)], repo)
        emit("frozen-hash", path=relative, match="yes" if done.out.split()[:1] == [digest] else "no")


# ── 沙盒 ──
COUNTER = [0]


def make_big(repository, relative):
    path = os.path.join(repository, relative)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "wb") as handle:
        handle.truncate(BIG)


def write_small(repository, relative, text):
    path = os.path.join(repository, relative)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)


def real_hooks_directory():
    return os.path.join(SETTINGS["repo"], "scripts", "gate", "git-hooks")


def install_wrappers(repository):
    """沙盒里提交进去的钩子目录：三份只 exec --repo 里的真钩子（有没有这个目录随提交走，是这一格要量的）。"""
    directory = os.path.join(repository, "scripts", "gate", "git-hooks")
    os.makedirs(directory, exist_ok=True)
    for name in HOOK_NAMES:
        path = os.path.join(directory, name)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(f'#!/usr/bin/env bash\nexec "{os.path.join(real_hooks_directory(), name)}" "$@"\n')
        os.chmod(path, 0o755)


def new_repository(name, hooks="absolute", with_hooks_commit=True):
    """分支 main、一个小基准提交（钩子关着提交）。hooks：absolute 是 core.hooksPath 指到 --repo 的钩子目录；relative 是沙盒里提交进去的钩子目录加相对的
    core.hooksPath（with_hooks_commit 假时钩子目录不提交，只配 core.hooksPath）；none 不配。"""
    COUNTER[0] += 1
    path = os.path.join(SETTINGS["work"], f"{COUNTER[0]:02d}-{name}")
    done = run(["git", "init", "-q", "-b", "main", path], SETTINGS["work"])
    if done.code != 0:
        raise RuntimeError(f"git init 退 {done.code}：{done.err.strip()[:200]}")
    write_small(path, "base.txt", "base\n")
    write_small(path, "f.txt", "a\n")
    git(path, "add", "base.txt", "f.txt")
    git(path, "-c", "core.hooksPath=/dev/null", "commit", "-q", "-m", "base")
    if hooks == "relative":
        if with_hooks_commit:
            install_wrappers(path)
            git(path, "add", "scripts/gate/git-hooks")
            git(path, "-c", "core.hooksPath=/dev/null", "commit", "-q", "-m", "hooks")
        git(path, "config", "core.hooksPath", "scripts/gate/git-hooks")
    elif hooks == "absolute":
        git(path, "config", "core.hooksPath", real_hooks_directory())
    return path


def with_remote(repository):
    remote = repository + ".remote.git"
    done = run(["git", "init", "-q", "--bare", "-b", "main", remote], SETTINGS["work"])
    if done.code != 0:
        raise RuntimeError(f"git init --bare 退 {done.code}")
    git(repository, "remote", "add", "origin", remote)
    git(repository, "-c", "core.hooksPath=/dev/null", "push", "-q", "-u", "origin", "main")
    return remote


def commit_without_hooks(repository, message, *extra):
    git(repository, "-c", "core.hooksPath=/dev/null", "commit", "-q", "-m", message, *extra)


def blob_sizes(repository, spec):
    """spec 是 rev-list 的参数（--all、HEAD…）；交 {对象名: 大小}（只含 blob）。"""
    listing = git(repository, "rev-list", "--objects", *spec, check=False).out.split("\n")
    names = [line.split()[0] for line in listing if line.strip()]
    if not names:
        return {}
    done = run(["git", "-C", repository, "cat-file", "--batch-check=%(objectname) %(objecttype) %(objectsize)"], repository, stdin_text="\n".join(names) + "\n")
    sizes = {}
    for line in done.out.splitlines():
        fields = line.split()
        if len(fields) == 3 and fields[1] == "blob":
            sizes[fields[0]] = int(fields[2])
    return sizes


def big_in_history(repository, *spec):
    return any(size > LIMIT for size in blob_sizes(repository, spec or ("HEAD",)).values())


def big_in_remote(remote):
    if not remote:
        return False
    done = run(["git", "-C", remote, "cat-file", "--batch-all-objects", "--batch-check=%(objecttype) %(objectsize)"], remote)
    return any(line.split()[0] == "blob" and int(line.split()[1]) > LIMIT for line in done.out.splitlines() if len(line.split()) == 2)


def leftover_index_copies():
    return sorted(entry for entry in os.listdir(SETTINGS["tmp"]) if entry.startswith("large-file-git-hook-index-"))


def observe(name, repository, done, expect, remote=None, push_only=False, **more):
    """expect：refused（钩子拒、超限的 blob 不进来）、lands（超限的 blob 进来，钩子没拦）、passes（没有超限的东西，通过）。"""
    refused = "单文件上限" in done.err
    landed_commit = big_in_history(repository, "HEAD")
    landed_remote = big_in_remote(remote)
    landed = landed_remote if push_only else (landed_commit or landed_remote)
    actual = "refused" if refused and not landed else "lands" if landed else "passes"
    emit(name, rc=done.code, refused=refused, landed_commit=landed_commit, landed_remote=landed_remote, expect=expect,
         verdict="as-expected" if actual == expect else "differs-from-expected", stderr_head=done.err.strip()[:160], **more)


# ── 第 2 问：提交那一侧，钩子读的是哪一份内容 ──
def diverged(name, hooks="absolute"):
    """base → main 与 feature 各改一次 f.txt，停在 main 上；合并、cherry-pick、rebase 都会在 f.txt 上冲突。"""
    repository = new_repository(name, hooks)
    git(repository, "checkout", "-q", "-b", "feature")
    write_small(repository, "f.txt", "feature\n")
    git(repository, "add", "f.txt")
    commit_without_hooks(repository, "feature edit")
    git(repository, "checkout", "-q", "main")
    write_small(repository, "f.txt", "main\n")
    git(repository, "add", "f.txt")
    commit_without_hooks(repository, "main edit")
    return repository


def resolve_with_big(repository):
    make_big(repository, "f.txt")
    git(repository, "add", "f.txt")


def commit_cases():
    # --amend：钩子看到的索引就是改写之后的那一笔；超限的 blob 是这一次新加的
    repository = new_repository("amend-adds-big")
    make_big(repository, "big.bin")
    git(repository, "add", "big.bin")
    observe("commit-amend-adds-big", repository, git(repository, "commit", "-q", "--amend", "--no-edit", check=False), "refused")
    # 从子目录起的提交：钩子的当前目录是工作树顶层，GIT_INDEX_FILE 的相对路径从那里起算
    repository = new_repository("commit-from-subdirectory", "relative")
    write_small(repository, "sub/x.txt", "x\n")
    git(repository, "add", "sub/x.txt")
    commit_without_hooks(repository, "sub")
    make_big(repository, "sub/big.bin")
    git(repository, "add", "sub/big.bin")
    done = run(["git", "commit", "-q", "-m", "x"], os.path.join(repository, "sub"))
    observe("commit-from-subdirectory-relative-hooks-path", repository, done, "refused")
    # 私有索引（相对路径）提交：git 与钩子都从顶层起算
    repository = new_repository("private-relative-index")
    make_big(repository, "big.bin")
    private = {"GIT_INDEX_FILE": ".git/private-index"}
    git(repository, "read-tree", "HEAD", env=private)
    git(repository, "add", "big.bin", env=private)
    observe("commit-private-relative-index", repository, git(repository, "commit", "-q", "-m", "x", check=False, env=private), "refused")
    # --squash 之后的 git commit 走 pre-commit
    repository = new_repository("squash-then-commit")
    git(repository, "checkout", "-q", "-b", "side")
    make_big(repository, "big.bin")
    git(repository, "add", "big.bin")
    commit_without_hooks(repository, "side big")
    git(repository, "checkout", "-q", "main")
    git(repository, "merge", "-q", "--squash", "side")
    observe("commit-after-merge-squash", repository, git(repository, "commit", "-q", "-m", "squashed", check=False), "refused")
    # 冲突之后收尾：解冲突时自己把文件写成超限
    repository = diverged("merge-conflict-commit")
    git(repository, "merge", "-q", "--no-edit", "feature", check=False)
    resolve_with_big(repository)
    observe("merge-conflict-then-git-commit", repository, git(repository, "commit", "-q", "-m", "resolved", check=False), "refused")
    repository = diverged("merge-conflict-continue")
    git(repository, "merge", "-q", "--no-edit", "feature", check=False)
    resolve_with_big(repository)
    observe("merge-conflict-then-merge-continue", repository, git(repository, "merge", "--continue", check=False), "refused")
    repository = diverged("cherry-pick-conflict-continue")
    git(repository, "cherry-pick", "feature", check=False)
    resolve_with_big(repository)
    observe("cherry-pick-conflict-then-continue", repository, git(repository, "cherry-pick", "--continue", check=False), "refused")
    repository = diverged("revert-conflict-continue")
    write_small(repository, "f.txt", "later\n")
    git(repository, "add", "f.txt")
    commit_without_hooks(repository, "later edit")
    git(repository, "revert", "--no-edit", "HEAD~1", check=False)
    resolve_with_big(repository)
    observe("revert-conflict-then-continue", repository, git(repository, "revert", "--continue", check=False), "refused")
    repository = diverged("rebase-conflict-continue")
    git(repository, "checkout", "-q", "feature")
    git(repository, "rebase", "main", check=False)
    resolve_with_big(repository)
    observe("rebase-conflict-then-continue", repository, git(repository, "rebase", "--continue", check=False), "refused")
    # rebase -i 停在 edit：用户自己 git commit --amend（走 pre-commit），或直接 --continue（sequencer 自己提交）
    for variant, finish in (("amend", ("commit", "-q", "--amend", "--no-edit")), ("continue-directly", ("rebase", "--continue"))):
        repository = new_repository(f"rebase-i-edit-{variant}")
        write_small(repository, "two.txt", "two\n")
        git(repository, "add", "two.txt")
        commit_without_hooks(repository, "two")
        git(repository, "rebase", "-i", "HEAD~1", env={"GIT_SEQUENCE_EDITOR": "sed -i -e 's/^pick/edit/'"}, check=False)
        make_big(repository, "big.bin")
        git(repository, "add", "big.bin")
        observe(f"rebase-i-edit-then-{variant}", repository, git(repository, *finish, check=False), "refused")
    # 不起 pre-commit 的重放：无冲突的 cherry-pick 与 git am，提交落地；推送那一刻由 pre-push 拦（对照）
    repository = new_repository("cherry-pick-clean")
    remote = with_remote(repository)
    git(repository, "checkout", "-q", "-b", "side")
    make_big(repository, "big.bin")
    git(repository, "add", "big.bin")
    commit_without_hooks(repository, "side big")
    git(repository, "checkout", "-q", "main")
    observe("cherry-pick-clean-lands", repository, git(repository, "cherry-pick", "side", check=False), "lands", remote=remote)
    observe("cherry-pick-clean-then-push-refused", repository, git(repository, "push", "-q", "origin", "main", check=False), "refused", remote=remote, push_only=True)
    repository = new_repository("am-clean")
    remote = with_remote(repository)
    git(repository, "checkout", "-q", "-b", "side")
    make_big(repository, "big.bin")
    git(repository, "add", "big.bin")
    commit_without_hooks(repository, "side big")
    patch = os.path.join(SETTINGS["work"], "side.patch")
    with open(patch, "w", encoding="utf-8") as handle:
        handle.write(git(repository, "format-patch", "-1", "--binary", "--stdout").out)
    git(repository, "checkout", "-q", "main")
    observe("am-clean-lands", repository, git(repository, "am", "--binary", patch, check=False), "lands", remote=remote)
    observe("am-clean-then-push-refused", repository, git(repository, "push", "-q", "origin", "main", check=False), "refused", remote=remote, push_only=True)


# ── 检出钩子 ㉒ 喂一条命令（不执行）──
def detector(command, cwd, name, agent_type=None):
    hook = os.path.join(SETTINGS["repo"], ".claude", "hooks", "bash-command-detector.sh")
    payload = {"tool_name": "Bash", "session_id": "probe", "cwd": cwd, "tool_input": {"command": command, "run_in_background": False}}
    if agent_type:
        payload["agent_type"] = agent_type
    environment = git_env({"CLAUDE_PROJECT_DIR": SETTINGS["repo"], "AGENT_HOOK_DETECTIONS": os.path.join(SETTINGS["detections"], name + ".jsonl")})
    return run(["bash", hook], cwd, env=environment, stdin_text=json.dumps(payload), seconds=120)


def installer(repository):
    return run([sys.executable, "-B", os.path.join(SETTINGS["repo"], "scripts", "gate", "git-hooks-install.py"), "check", repository], SETTINGS["work"], seconds=120)


# ── 第 2 问：推送那一侧，每一行引用 ──
def push_cases():
    repository = new_repository("push-tag-to-blob")
    remote = with_remote(repository)
    make_big(repository, "big.bin")
    blob = git(repository, "hash-object", "-w", "big.bin").out.strip()
    git(repository, "tag", "bigblob", blob)
    observe("push-tag-pointing-at-big-blob", repository, git(repository, "push", "-q", "origin", "refs/tags/bigblob", check=False), "refused", remote=remote, push_only=True)
    tree = run(["git", "-C", repository, "mktree"], repository, stdin_text=f"100644 blob {blob}\tbig.bin\n").out.strip()
    git(repository, "tag", "bigtree", tree)
    observe("push-tag-pointing-at-tree-with-big-blob", repository, git(repository, "push", "-q", "origin", "refs/tags/bigtree", check=False), "refused", remote=remote, push_only=True)
    repository = new_repository("push-from-subdirectory", "relative")
    remote = with_remote(repository)
    write_small(repository, "sub/x.txt", "x\n")
    git(repository, "add", "sub/x.txt")
    commit_without_hooks(repository, "sub")
    make_big(repository, "big.bin")
    git(repository, "add", "big.bin")
    commit_without_hooks(repository, "big")
    done = run(["git", "push", "-q", "origin", "main"], os.path.join(repository, "sub"))
    observe("push-from-subdirectory-relative-hooks-path", repository, done, "refused", remote=remote, push_only=True)


# ── 第 3 问：安装与核对，检出老提交的工作树 ──
def install_cases():
    repository = new_repository("layered", "relative")
    remote = with_remote(repository)
    old = git(repository, "rev-parse", "HEAD~1").out.strip()
    old_tree = os.path.join(SETTINGS["work"], "worktree-old")
    git(repository, "worktree", "add", "-q", "--detach", old_tree, old)
    checked = installer(repository)
    emit("check-lists-old-worktree", rc=checked.code, lists_old_worktree="没人判" in checked.out and old_tree in checked.out, expect="rc=0 且列出",
         stdout_head=checked.out.strip()[-200:])
    make_big(old_tree, "big.bin")
    git(old_tree, "add", "big.bin")
    observe("old-commit-worktree-big-commit", old_tree, git(old_tree, "commit", "-q", "-m", "big", check=False), "refused")
    observe("old-commit-worktree-push", old_tree, git(old_tree, "push", "-q", "origin", "HEAD:refs/heads/oldbranch", check=False), "refused", remote=remote, push_only=True)
    for label, command in (("commit", "git commit -qm x"), ("push", "git push origin HEAD:refs/heads/oldbranch")):
        done = detector(command, old_tree, f"old-worktree-{label}")
        emit(f"detector-in-old-commit-worktree-{label}", exit=done.code, refused="钩子" in done.err, expect="refused", stderr_head=done.err.strip()[:160])
    new_tree = os.path.join(SETTINGS["work"], "worktree-new")
    git(repository, "worktree", "add", "-q", "--detach", new_tree, "HEAD")
    make_big(new_tree, "big.bin")
    git(new_tree, "add", "big.bin")
    observe("current-commit-worktree-big-commit-control", new_tree, git(new_tree, "commit", "-q", "-m", "big", check=False), "refused")
    # 主工作树自己检出老提交
    repository = new_repository("main-at-old-commit", "relative")
    git(repository, "checkout", "-q", "HEAD~1")
    checked = installer(repository)
    make_big(repository, "big.bin")
    git(repository, "add", "big.bin")
    observe("main-tree-at-old-commit-big-commit", repository, git(repository, "commit", "-q", "-m", "big", check=False), "refused", check_rc=checked.code,
            check_stdout_head=checked.out.strip()[:160])
    done = detector("git commit -qm x", repository, "main-at-old-commit")
    emit("detector-in-main-tree-at-old-commit", exit=done.code, refused="钩子" in done.err, expect="refused", stderr_head=done.err.strip()[:160])
    # 钩子目录只在暂存区、没提交：reset --hard 把它删掉
    repository = new_repository("hooks-staged-only", "relative", with_hooks_commit=False)
    install_wrappers(repository)
    git(repository, "add", "scripts/gate/git-hooks")
    git(repository, "reset", "-q", "--hard")
    checked = installer(repository)
    make_big(repository, "big.bin")
    git(repository, "add", "big.bin")
    observe("hooks-staged-then-reset-hard-big-commit", repository, git(repository, "commit", "-q", "-m", "big", check=False), "refused", check_rc=checked.code)
    # 目录在、pre-commit 一份不在：检出钩子靠 pre-commit 在不在认「这是本仓」，核对脚本照 HOOK_NAMES 逐份核
    repository = new_repository("pre-commit-file-missing", "relative")
    os.remove(os.path.join(repository, "scripts", "gate", "git-hooks", "pre-commit"))
    checked = installer(repository)
    done = detector("git commit -qm x", repository, "pre-commit-missing")
    emit("pre-commit-file-missing-check-vs-detector", check_rc=checked.code, check_names_pre_commit="pre-commit" in checked.out and "不在" in checked.out,
         detector_exit=done.code, expect="check_rc=1 且 detector_exit=2", stdout_head=checked.out.strip()[:200], stderr_head=done.err.strip()[:160])


# ── 第 4 问：日常提交写法过不过 ㉒，窗口闸与大文件判法放在一起 ──
def detector_cases():
    repository = new_repository("detector-hooked", "relative")
    message = os.path.join(SETTINGS["work"], "message.txt")
    with open(message, "w", encoding="utf-8") as handle:
        handle.write("subject\n\nbody\n")
    heredoc = "git commit -q -m \"$(cat <<'MESSAGE_END'\nsubject\n\nbody mentions xargs git commit -n and git push --no-verify\nMESSAGE_END\n)\""
    allowed = (
        ("daily-add-then-commit", "git add base.txt && git commit -qm x"),
        ("daily-commit-heredoc-message", heredoc),
        ("daily-commit-message-file", f"git commit -qF {message}"),
        ("daily-commit-all", "git commit -a -m x"),
        ("daily-commit-amend-no-edit", "git commit --amend --no-edit"),
        ("daily-commit-message-mentions-flags", "git commit -m 'docs: explain the -n flag and --no-verify'"),
        ("daily-commit-path-after-double-dash", "git commit -m x -- base.txt"),
        ("daily-commit-signoff-allow-empty", "git commit -s --allow-empty -m x"),
        ("daily-commit-outside-window-prefix", "SINGLEFS_COMMIT_OUTSIDE_WINDOW=1 git commit -qm x"),
        ("daily-commit-outside-window-export", "export SINGLEFS_COMMIT_OUTSIDE_WINDOW=1; git commit -qm x"),
        ("daily-cd-then-commit", f"cd {repository} && git commit -qm x"),
        ("daily-git-C-commit", f"git -C {repository} commit -qm x"),
        ("daily-push", "git push origin main"),
        ("daily-push-set-upstream", "git push -u origin HEAD"),
        ("daily-push-dry-run-short", "git push -n origin main"),
        ("daily-push-follow-tags", "git push --follow-tags origin main"),
        ("daily-push-force-with-lease", "git push --force-with-lease origin main"),
        ("daily-log-with-n", "git log -n 5"),
        ("daily-cherry-pick-no-commit-short", "git cherry-pick -n HEAD"),
        ("daily-merge-no-stat-short", "git merge -n feature"),
        ("daily-worktree-flow-update-ref-read-tree", "git update-ref refs/heads/main HEAD && git read-tree HEAD"),
        ("known-merge-no-verify-not-guarded", "git merge --no-verify feature"),
    )
    refused = (
        ("bypass-commit-no-verify", "git commit --no-verify -m x"),
        ("bypass-commit-n", "git commit -n -m x"),
        ("bypass-commit-nm", "git commit -nm x"),
        ("bypass-commit-abbreviation", "git commit --no-veri -m x"),
        ("bypass-push-no-verify", "git push --no-verify origin main"),
        ("bypass-hooks-path-on-command-line", "git -c core.hooksPath=/dev/null commit -m x"),
        ("bypass-config-env", "git --config-env=core.hooksPath=HOOKS commit -m x"),
        ("bypass-config-count-environment", "GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.hooksPath GIT_CONFIG_VALUE_0=/dev/null git commit -m x"),
        ("bypass-config-write", "git config core.hooksPath /dev/null"),
        ("bypass-outside-window-prefix-with-no-verify", "SINGLEFS_COMMIT_OUTSIDE_WINDOW=1 git commit --no-verify -m x"),
    )
    for expectation, cases in (("allowed", allowed), ("refused", refused)):
        for name, command in cases:
            done = detector(command, repository, name)
            actual = "refused" if done.code == 2 else "allowed" if done.code == 0 else f"exit-{done.code}"
            expected = "allowed" if name.startswith("known-") else expectation
            emit(name, exit=done.code, expect=expected, verdict="as-expected" if actual == expected else "differs-from-expected",
                 names_hook_bypass="绕开了 git 的钩子" in done.err, stderr_head=done.err.strip()[:200], command=command)


PRIVATE_WINDOW_GATE = '#!/usr/bin/env bash\necho "PRIVATE-WINDOW-GATE" >&2\n[[ "${SINGLEFS_COMMIT_OUTSIDE_WINDOW:-}" == 1 ]] && exit 0\nexit 1\n'


def window_repository(name):
    repository = new_repository(name)
    path = os.path.join(repository, ".git", "hooks", "pre-commit")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(PRIVATE_WINDOW_GATE)
    os.chmod(path, 0o755)
    return repository


def window_cases():
    def report(name, repository, done, head_before, expect):
        moved = git(repository, "rev-parse", "HEAD").out.strip() != head_before
        window_refused = "PRIVATE-WINDOW-GATE" in done.err and done.code != 0 and not moved
        actual = "window-refused" if window_refused else "commit-made" if moved else "other"
        emit(name, rc=done.code, head_moved=moved, expect=expect, verdict="as-expected" if actual == expect else "differs-from-expected", stderr_head=done.err.strip()[:120])

    for label, extra, expect in (("window-closed-small-commit", None, "window-refused"), ("window-open-by-variable-small-commit", {"SINGLEFS_COMMIT_OUTSIDE_WINDOW": "1"}, "commit-made")):
        repository = window_repository(label)
        write_small(repository, "s.txt", "s\n")
        git(repository, "add", "s.txt")
        before = git(repository, "rev-parse", "HEAD").out.strip()
        report(label, repository, git(repository, "commit", "-q", "-m", "x", check=False, env=extra), before, expect)
    repository = window_repository("window-open-by-variable-big-commit")
    make_big(repository, "big.bin")
    git(repository, "add", "big.bin")
    observe("window-open-by-variable-big-commit", repository, git(repository, "commit", "-q", "-m", "x", check=False, env={"SINGLEFS_COMMIT_OUTSIDE_WINDOW": "1"}), "refused")
    repository = window_repository("window-closed-amend")
    before = git(repository, "rev-parse", "HEAD").out.strip()
    report("window-closed-amend", repository, git(repository, "commit", "-q", "--amend", "-m", "again", check=False), before, "window-refused")
    # 窗口闸只管 git commit 与 git merge 生成的合并提交：cherry-pick、revert、rebase、am 起不了 pre-commit，窗口外照样生成提交
    repository = window_repository("window-closed-cherry-pick")
    git(repository, "checkout", "-q", "-b", "side")
    write_small(repository, "s.txt", "s\n")
    git(repository, "add", "s.txt")
    commit_without_hooks(repository, "side")
    git(repository, "checkout", "-q", "main")
    before = git(repository, "rev-parse", "HEAD").out.strip()
    report("window-closed-cherry-pick-not-covered", repository, git(repository, "cherry-pick", "side", check=False), before, "commit-made")
    repository = window_repository("window-closed-merge")
    git(repository, "checkout", "-q", "-b", "side")
    write_small(repository, "s.txt", "s\n")
    git(repository, "add", "s.txt")
    commit_without_hooks(repository, "side")
    git(repository, "checkout", "-q", "main")
    write_small(repository, "m.txt", "m\n")
    git(repository, "add", "m.txt")
    commit_without_hooks(repository, "main")
    before = git(repository, "rev-parse", "HEAD").out.strip()
    report("window-closed-merge-commit", repository, git(repository, "merge", "-q", "--no-edit", "side", check=False), before, "window-refused")


GROUPS = {"commit": (commit_cases,), "push": (push_cases,), "install": (install_cases,), "detector": (detector_cases,), "window": (window_cases,)}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True)
    parser.add_argument("--work", required=True)
    parser.add_argument("--group", default=",".join(GROUPS))
    arguments = parser.parse_args()
    unknown = [name for name in arguments.group.split(",") if name not in GROUPS]
    repo, work = os.path.realpath(arguments.repo), os.path.realpath(arguments.work)
    if unknown or not os.path.isdir(os.path.join(repo, "scripts", "gate", "git-hooks")) or (os.path.exists(work) and os.listdir(work)):
        print("用法：probe_hooks.py --repo <仓根> --work <新的空目录> [--group commit,push,install,detector,window]", file=sys.stderr)
        return 2
    SETTINGS.update(repo=repo, work=work, tmp=os.path.join(work, "tmp"), detections=os.path.join(work, "detections"))
    for key in ("tmp", "detections"):
        os.makedirs(SETTINGS[key], exist_ok=True)
    verify_snapshot(repo)
    failed = 0
    for group in arguments.group.split(","):
        for function in GROUPS[group]:
            try:
                function()
            except (RuntimeError, OSError) as error:
                failed += 1
                emit(f"setup-error-{group}", error=str(error)[:300])
    emit("leftover-index-copies", count=len(leftover_index_copies()), names=leftover_index_copies()[:5])
    print(f"PROBE name=done emitted={EMITTED[0]}", flush=True)
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
