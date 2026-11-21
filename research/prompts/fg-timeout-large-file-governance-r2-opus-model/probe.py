#!/usr/bin/env python3
"""攻方腿探针（fg-timeout-large-file-governance-r2，正文「要腿判的」第 5 到 8 问）。从第一轮攻方模型的 probe.py 拷来，留下装置那一半，格全换。

用法：python3 probe.py --repo <仓根> --frozen <被判的那棵树> --work <新的空目录> [--model <模型目录>] [--only <组,…>]
  --frozen 里的检出钩子与 scripts/gate/large-file-check.py 是被判的；开头逐份对仓根
  research/prompts/fg-timeout-large-file-governance-r2-snapshot/judged-sha256.txt 核 sha256，对不上照跑、行里写 match=no。
  临时仓、裸远端、检出记录都建在 --work 下；大文件是 SINGLE_FILE_SIZE_LIMIT_BYTES + 1 字节（稀疏，或 head -c 从 /dev/zero 写）。
  喂钩子与跑命令各起一个新会话（start_new_session），超时与收尾只对自己起的那一组发信号；钩子被停之后它起的判法（另一个会话）
  不由探针停，只按 /proc 里的命令行等它自己退出、记它活了多久。
输出：每格一行 `PROBE name=<格名> …`，末行 `PROBE name=done emitted=<行数>`（不含末行）。
退出码：0 跑完；1 有一组装置出错（那一组一行 group-error-<组>）；2 用法错。
结局（verdict）：missed-silently 超限的进了提交或远端、钩子没拒也没记检出；passed-recorded 进了、没拒、记了检出；rejected 进了（命令照跑时）而钩子拒了；
  rejected-nothing-lands 钩子拒了而这条命令什么超限的都不会带进去（误拒）；passed-nothing-lands、passed-recorded-nothing-lands 两样都没有。
"""
import argparse
import hashlib
import json
import os
import re
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
               GIT_CEILING_DIRECTORIES=SETTINGS["work"], GIT_TERMINAL_PROMPT="0", GIT_EDITOR="true")
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


def bare_remote(name):
    remote = os.path.join(SETTINGS["remotes"], name + ".git")
    subprocess.run(["git", "init", "-q", "--bare", remote], env=git_environment(), check=True, capture_output=True)
    return remote


def with_remote(repository, name, remote_name="origin"):
    remote = bare_remote(name if remote_name == "origin" else f"{name}-{remote_name}")
    git(repository, "remote", "add", remote_name, remote)
    git(repository, "push", "-q", "-u" if remote_name == "origin" else "--no-verify", remote_name, "main")
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


def hook_environment(detections, extra=None, global_config=None, dropped=()):
    env = git_environment(global_config=global_config)
    env.update(CLAUDE_PROJECT_DIR=SETTINGS["frozen"], AGENT_HOOK_DETECTIONS=detections)
    env.update(extra or {})
    for name in dropped:
        env.pop(name, None)
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
        payload["agent_id"] = "probe-agent"
    return payload


def yes_no(flag):
    return "yes" if flag else "no"


def large_file_case(name, question, cwd, command, repository, remote=None, hook_extra=None, run_extra=None,
                    hook_dropped=(), execute=True):
    """喂钩子一次（不执行），再真执行这条命令一次，看超限的 blob 有没有进新提交或进远端；按两样合成结局。
    remote 是要看的那个裸远端（推到别的远端、子模块的远端的，传那一个）。"""
    detections = os.path.join(SETTINGS["detections"], name + ".jsonl")
    commits_before = commits_of(repository)
    remote_before = big_blobs_in_remote(remote)
    hook_run = feed_hook(SETTINGS["hook"], payload_for(command, cwd), hook_environment(detections, hook_extra, dropped=hook_dropped))
    keep_stderr(name, hook_run.err)
    rejected = hook_run.code == 2 and "单文件上限" in hook_run.err
    recorded = "单文件上限" in detections_text(detections)
    command_code, landed = "not-run", "no"
    if execute:
        command_run = run_own_group(["bash", "-c", command], cwd, git_environment(run_extra))
        command_code = command_run.code
        keep_stderr(name + "-command", command_run.err)
        if big_blobs_brought_by(repository, commits_of(repository) - commits_before):
            landed = "commit"
        if big_blobs_in_remote(remote) > remote_before:
            landed = "push" if landed == "no" else landed + "+push"
    if landed != "no":
        verdict = "rejected" if rejected else "passed-recorded" if recorded else "missed-silently"
    else:
        verdict = "rejected-nothing-lands" if rejected else "passed-recorded-nothing-lands" if recorded else "passed-nothing-lands"
    cause = re.search(r"（([^（）]{0,60}(?:可能改工作区或暂存区|xargs|shell 别名|路径判不了)[^（）]{0,20})）", hook_run.err)
    emit(name, question=question, hook_exit=hook_run.code, rejected=yes_no(rejected), recorded=yes_no(recorded),
         command_exit=command_code, landed=landed, verdict=verdict, hook_seconds=f"{hook_run.seconds:.2f}",
         hook_leftover=len(hook_run.leftover), cause=cause.group(1) if cause else "none", command=command)
    return hook_run


def staged_big(name):
    repository = new_repository(name)
    make_big(repository, "big.bin")
    git(repository, "add", "big.bin")
    return repository


def untracked_big(name):
    repository = new_repository(name)
    make_big(repository, "big.bin")
    return repository


def superproject_with_dirty_submodule(name):
    """超级仓跟着自己的远端；子模块里一笔带 big.bin 的提交没推，超级仓记了新的 gitlink 并已提交（第一轮 push-recurse-submodules 那一形）。"""
    sub_remote, super_remote = bare_remote(name + "-sub"), bare_remote(name + "-super")
    seed = new_repository(name + "-seed")
    git(seed, "remote", "add", "origin", sub_remote)
    git(seed, "push", "-q", "origin", "main")
    superproject = new_repository(name + "-super")
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
    return superproject, submodule, sub_remote


def alias_and_launcher_cases():
    """第 5 问：五类「执行前拒」之外照样静默放过的写法：别名链、同一条里才配的别名、别的程序里起的 git commit / git push。"""
    repository = untracked_big("alias-shell-chain-without-word")
    git(repository, "config", "alias.ci", "commit")
    git(repository, "config", "alias.save", "!git add -A && git ci -qm wip")
    large_file_case("alias-shell-chain-without-word", 5, repository, "git save", repository)
    repository = staged_big("alias-defined-in-same-command")
    large_file_case("alias-defined-in-same-command", 5, repository, "git config alias.ci commit && git ci -qm x", repository)
    repository = staged_big("alias-configured-beforehand-control")
    git(repository, "config", "alias.ci", "commit")
    large_file_case("alias-configured-beforehand-control", 5, repository, "git ci -qm x", repository)
    repository = staged_big("alias-chain-deeper-than-limit")
    for depth in range(1, 6):
        git(repository, "config", f"alias.a{depth}", f"a{depth + 1}")
    git(repository, "config", "alias.a6", "commit")
    large_file_case("alias-chain-deeper-than-limit", 5, repository, "git a1 -qm x", repository)
    repository = staged_big("alias-value-leading-global-option")
    git(repository, "config", "alias.ci", "--no-pager commit")
    large_file_case("alias-value-leading-global-option", 5, repository, "git ci -qm x", repository)
    repository = staged_big("external-subcommand-on-path")
    extra_bin = os.path.join(SETTINGS["work"], "extra-bin")
    os.makedirs(extra_bin, exist_ok=True)
    with open(os.path.join(extra_bin, "git-ci"), "w", encoding="utf-8") as handle:
        handle.write("#!/bin/sh\nexec git commit \"$@\"\n")
    os.chmod(os.path.join(extra_bin, "git-ci"), 0o755)
    large_file_case("external-subcommand-on-path", 5, repository, f"PATH={extra_bin}:$PATH git ci -qm x", repository)
    for name, command in (("commit-in-python-c", "python3 -c 'import subprocess; subprocess.run([\"git\", \"commit\", \"-qm\", \"x\"], check=True)'"),
                          ("commit-in-awk-system", "awk 'BEGIN { exit system(\"git commit -qm x\") }'")):
        large_file_case(name, 5, staged_big(name), command, os.path.join(SETTINGS["repositories"], name))
    repository = new_repository("commit-in-rebase-exec")
    write_small(repository, "two.txt")
    git(repository, "add", "two.txt")
    git(repository, "commit", "-q", "-m", "two")
    make_big(repository, "big.bin")
    large_file_case("commit-in-rebase-exec", 5, repository,
                    "git rebase -q --exec 'git add big.bin && git commit -q --amend --no-edit' HEAD~1", repository)
    superproject, submodule, sub_remote = superproject_with_dirty_submodule("push-in-submodule-foreach")
    large_file_case("push-in-submodule-foreach", 5, superproject, "git submodule --quiet foreach 'git push -q origin HEAD:main'", submodule, remote=sub_remote)
    superproject, submodule, sub_remote = superproject_with_dirty_submodule("push-recurse-from-config")
    large_file_case("push-recurse-from-config", 5, superproject, "git -c push.recurseSubmodules=on-demand push -q", submodule, remote=sub_remote)
    superproject, submodule, sub_remote = superproject_with_dirty_submodule("push-recurse-flag-control")
    large_file_case("push-recurse-flag-control", 5, superproject, "git push -q --recurse-submodules=on-demand", submodule, remote=sub_remote)


def preceding_change_cases():
    """第 5 问：「读不改」名单里的命令按操作数或程序文本写文件、重定向写进放在工作根下的仓、env -u 撤掉私有索引，排在 git commit 前面照样不当会改。"""
    outside = os.path.join(SETTINGS["work"], "outside")
    make_big(outside, "huge.src")
    huge = os.path.join(outside, "huge.src")
    for name, writer in (("read-only-name-sort-output", f"sort -o base.txt {huge}"),
                         ("read-only-name-sed-write-command", f"sed -n 'w base.txt' {huge}"),
                         ("read-only-name-awk-print-redirect", f"awk '{{ print > \"base.txt\" }}' {huge}")):
        repository = new_repository(name)
        large_file_case(name, 5, repository, f"{writer} && git commit -qam x", repository)
    for name, absolute in (("redirect-into-repository-under-work-root", True), ("redirect-into-repository-relative-control", False)):
        repository = new_repository(name)
        destination = os.path.join(repository, "base.txt") if absolute else "base.txt"
        large_file_case(name, 5, repository, f"head -c {SETTINGS['limit'] + 1} /dev/zero > {destination} && git commit -qam x", repository,
                        hook_extra={"SINGLEFS_WORK_ROOT": SETTINGS["work"]})
    private = os.path.join(SETTINGS["work"], "private-index-never-made")
    repository = staged_big("env-unset-private-index")
    large_file_case("env-unset-private-index", 5, repository, f"export GIT_INDEX_FILE={private} && env -u GIT_INDEX_FILE git commit -qm x", repository)
    repository = staged_big("env-private-index-control")
    large_file_case("env-private-index-control", 5, repository, f"env GIT_INDEX_FILE={private} git commit -qm x", repository)
    repository = staged_big("git-C-command-substitution")
    large_file_case("git-C-command-substitution", 5, repository, 'git -C "$(pwd)" commit -qm x', repository)


def misreject_cases():
    """第 5 问：五类「执行前拒」误拒的写法：排在 git commit 前面、带不进超限内容的命令；与两格日常写法的对照。仓里没有超限的文件。"""
    temporary = os.path.join(SETTINGS["work"], "tmpdir")
    os.makedirs(temporary, exist_ok=True)
    tee_target = os.path.join(temporary, "tee-message.txt")
    in_work_root = os.path.join(temporary, "plain-message.txt")
    heredoc_commit = "git add base.txt && git commit -q -m \"$(cat <<'EOF'\nFix probe\n\nCo-Authored-By: Probe <probe@example.invalid>\nEOF\n)\""
    cases = (("misreject-chmod-then-add", ("base.txt",), "chmod +x tool.sh && git add tool.sh && git commit -qm x"),
             ("misreject-rm-then-add-all", ("base.txt", "extra.txt"), "rm -f extra.txt && git add -A && git commit -qm x"),
             ("misreject-mkdir-then-commit-all", ("base.txt",), "mkdir -p notes && git commit -qam x"),
             ("misreject-touch-then-commit-all", ("base.txt",), "touch base.txt && git commit -qam x"),
             ("misreject-stash-list", ("base.txt",), "git stash list && git commit -qam x"),
             ("misreject-worktree-list", ("base.txt",), "git worktree list && git commit -qam x"),
             ("misreject-tmpdir-variable-message", ("base.txt",), 'printf "x\\n" > "$TMPDIR/probe-message.txt" && git commit -qaF "$TMPDIR/probe-message.txt"'),
             ("misreject-mktemp-message", ("base.txt",), 'f=$(mktemp) && printf "x\\n" > "$f" && git commit -qaF "$f"'),
             ("misreject-tee-message-in-work-root", ("base.txt",), f"printf 'x\\n' | tee {tee_target} && git commit -qaF {tee_target}"),
             ("daily-message-redirect-in-work-root-control", ("base.txt",), f"printf 'x\\n' > {in_work_root} && git commit -qaF {in_work_root}"),
             ("daily-heredoc-message-control", ("base.txt",), heredoc_commit))
    for name, files, command in cases:
        repository = new_repository(name, files=files)
        write_small(repository, "base.txt", "changed\n")
        write_small(repository, "tool.sh", "#!/bin/sh\n")
        large_file_case(name, 5, repository, command, repository, run_extra={"TMPDIR": temporary},
                        hook_extra={"TMPDIR": temporary, "SINGLEFS_WORK_ROOT": SETTINGS["work"]})


def work_root_home_cases():
    """第 5 问：重定向写到工作根下算「不碰仓」，工作根没设 SINGLEFS_WORK_ROOT 时照 ~ 展开：同一条命令、同一个共享路径，HOME 不同判得不同。只喂钩子。"""
    marker = "/.local/share/singlefs/work/"
    message = os.path.join(SETTINGS["work"], "message-in-shared-work-root.txt")
    another_home = os.path.join(SETTINGS["work"], "another-home")
    os.makedirs(another_home, exist_ok=True)
    homes = (("harmless-work-root-home-owning-this-path", SETTINGS["work"].split(marker)[0] if marker in SETTINGS["work"] else None),
             ("harmless-work-root-another-home", another_home))
    for name, home in homes:
        if home is None:
            emit(name, question=5, skipped="work-not-under-a-default-work-root")
            continue
        repository = new_repository(name)
        write_small(repository, "base.txt", "changed\n")
        large_file_case(name, 5, repository, f"printf 'x\\n' > {message} && git commit -qaF {message}", repository,
                        hook_extra={"HOME": home}, hook_dropped=("SINGLEFS_WORK_ROOT",), execute=False)


def pushed_main_with_big_unpushed(name):
    """main 跟着 origin/main；main 上一笔带 big.bin 的提交没推。"""
    repository = new_repository(name)
    remote = with_remote(repository, name)
    make_big(repository, "big.bin")
    git(repository, "add", "big.bin")
    git(repository, "commit", "-q", "-m", "add big")
    return repository, remote


def clean_main_with_dirty_branch(name, branch="feature", publish_branch_first=False):
    """main 跟着上游、什么都没欠；分支 branch 上一笔带 big.bin 的提交没推（publish_branch_first：分支先不带大文件推一次，远端有同名分支）；停在 main 上。"""
    repository = new_repository(name)
    remote = with_remote(repository, name)
    git(repository, "checkout", "-q", "-b", branch)
    if publish_branch_first:
        git(repository, "push", "-q", "origin", branch)
    make_big(repository, "big.bin")
    git(repository, "add", "big.bin")
    git(repository, "commit", "-q", "-m", "add big")
    git(repository, "checkout", "-q", "main")
    return repository, remote


def push_question_cases():
    """第 6 问：push 按引用判之后，各种 refspec、选项、push.default 与远端写法判成什么。"""
    for name, command in (("push-plus-head-to-full-ref", "git push -q origin +HEAD:refs/heads/x"),
                          ("push-force-with-lease", "git push -q --force-with-lease origin main"),
                          ("push-current-default", "git push -q"),
                          ("push-branch-created-in-same-command", "git switch -q -c topic && git push -q -u origin topic"),
                          ("push-after-commit-amend-in-same-command", "git commit -q --amend -m again && git push -q --force")):
        repository, remote = pushed_main_with_big_unpushed(name)
        large_file_case(name, 6, repository, command, repository, remote=remote)
    repository, remote = clean_main_with_dirty_branch("push-set-upstream-existing-branch")
    git(repository, "checkout", "-q", "feature")
    large_file_case("push-set-upstream-existing-branch", 6, repository, "git push -q --set-upstream origin feature", repository, remote=remote)
    repository, remote = clean_main_with_dirty_branch("push-after-fast-forward-merge")
    large_file_case("push-after-fast-forward-merge", 6, repository, "git merge -q --ff-only feature && git push -q", repository, remote=remote)
    repository, remote = clean_main_with_dirty_branch("push-default-matching", publish_branch_first=True)
    git(repository, "config", "push.default", "matching")
    large_file_case("push-default-matching", 6, repository, "git push -q", repository, remote=remote)
    repository, remote = clean_main_with_dirty_branch("push-default-matching-command-line", publish_branch_first=True)
    large_file_case("push-default-matching-command-line", 6, repository, "git -c push.default=matching push -q", repository, remote=remote)
    repository, remote = clean_main_with_dirty_branch("push-remote-push-refspec-config")
    git(repository, "config", "remote.origin.push", "refs/heads/*:refs/heads/*")
    large_file_case("push-remote-push-refspec-config", 6, repository, "git push -q", repository, remote=remote)
    repository, remote = clean_main_with_dirty_branch("push-wildcard-refspec-nested-branch", branch="topic/heavy")
    large_file_case("push-wildcard-refspec-nested-branch", 6, repository, "git push -q origin 'refs/heads/*:refs/heads/*'", repository, remote=remote)
    repository, remote = clean_main_with_dirty_branch("push-wildcard-refspec-top-level-control")
    large_file_case("push-wildcard-refspec-top-level-control", 6, repository, "git push -q origin 'refs/heads/*:refs/heads/*'", repository, remote=remote)
    for name, command_of in (("push-repo-option-with-positional-remote", lambda backup: "git push -q --repo=origin backup main"),
                             ("push-to-path-another-remote-has-it", lambda backup: f"git push -q {backup} main")):
        repository, origin = pushed_main_with_big_unpushed(name)
        git(repository, "push", "-q", "origin", "main")   # origin（本地裸仓，没有单文件上限）先收下这一笔，跟踪引用 origin/main 带着它
        backup = bare_remote(name + "-backup")
        git(repository, "remote", "add", "backup", backup)
        git(repository, "push", "-q", "backup", "main~1:refs/heads/main")
        git(repository, "fetch", "-q", "backup")
        large_file_case(name, 6, repository, command_of(backup), repository, remote=backup)
    repository, remote = pushed_main_with_big_unpushed("push-remote-never-fetched")
    fresh = bare_remote("push-remote-never-fetched-fresh")
    git(repository, "remote", "add", "fresh", fresh)
    large_file_case("push-remote-never-fetched", 6, repository, "git push -q fresh main", repository, remote=fresh)


def slow_git_directory(seconds):
    """PATH 里排在前面的一份 git：参数里有 diff 的先睡 seconds 秒再交给真 git，别的直接交（判法里只有 git diff 会慢）。"""
    directory = os.path.join(SETTINGS["work"], f"slow-git-{seconds}")
    if not os.path.isdir(directory):
        os.makedirs(directory)
        real = shutil.which("git")
        path = os.path.join(directory, "git")
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(f"#!/bin/sh\ncase \" $* \" in *\" diff \"*) sleep {seconds} ;; esac\nexec {real} \"$@\"\n")
        os.chmod(path, 0o755)
    return directory


def index_copies(directory):
    names = [entry for entry in os.listdir(directory) if entry.startswith("large-file-check-index-")] if os.path.isdir(directory) else []
    total = 0
    for entry in names:
        for root, _dirs, files in os.walk(os.path.join(directory, entry)):
            total += sum(os.path.getsize(os.path.join(root, name)) for name in files)
    return len(names), total


def judge_runtime_cases():
    """第 7 问：判法读索引拷贝、自己的进程组：到点停组时拷贝留不留下；钩子本身被停时判法活不活着；索引上锁、split index 时写不写用户的 .git。命令不执行。"""
    repository = staged_big("judge-timeout-leaves-index-copy")
    temporary = os.path.join(SETTINGS["work"], "judge-tmp-timeout")
    os.makedirs(temporary)
    detections = os.path.join(SETTINGS["detections"], "judge-timeout-leaves-index-copy.jsonl")
    run = feed_hook(SETTINGS["hook"], payload_for("git commit -qm x", repository),
                    hook_environment(detections, {"PATH": slow_git_directory(40) + ":" + os.environ["PATH"], "TMPDIR": temporary}))
    copies, size = index_copies(temporary)
    emit("judge-timeout-leaves-index-copy", question=7, hook_exit=run.code, hook_seconds=f"{run.seconds:.2f}",
         recorded=yes_no("单文件上限" in detections_text(detections)), leftover_index_copies=copies, leftover_bytes=size)
    repository = staged_big("hook-stopped-judge-keeps-running")
    temporary = os.path.join(SETTINGS["work"], "judge-tmp-hook-stopped")
    os.makedirs(temporary)
    run = feed_hook(SETTINGS["hook"], payload_for("git commit -qm x", repository),
                    hook_environment(os.path.join(SETTINGS["detections"], "hook-stopped-judge-keeps-running.jsonl"),
                                     {"PATH": slow_git_directory(20) + ":" + os.environ["PATH"], "TMPDIR": temporary}), limit_seconds=8)
    marker = f"{repository} --guard"
    survivors = processes_mentioning(marker)
    started, copies_while_alive = time.monotonic(), index_copies(temporary)[0]
    while processes_mentioning(marker) and time.monotonic() - started < 120:
        time.sleep(1)
    emit("hook-stopped-judge-keeps-running", question=7, hook_timed_out=yes_no(run.timed_out), hook_leftover=len(run.leftover),
         judges_alive_after_hook_stopped=len(survivors), index_copies_while_alive=copies_while_alive,
         seconds_until_judges_gone=f"{time.monotonic() - started:.1f}", judges_still_alive=len(processes_mentioning(marker)),
         index_copies_after=index_copies(temporary)[0])
    repository = new_repository("index-locked-by-another-session")
    write_small(repository, "base.txt", "changed\n")
    lock = os.path.join(repository, ".git", "index.lock")
    open(lock, "w").close()
    detections = os.path.join(SETTINGS["detections"], "index-locked-by-another-session.jsonl")
    run = feed_hook(SETTINGS["hook"], payload_for("git commit -qam x", repository), hook_environment(detections))
    emit("index-locked-by-another-session", question=7, hook_exit=run.code, recorded=yes_no("单文件上限" in detections_text(detections)),
         lock_still_there=yes_no(os.path.exists(lock)), lock_bytes=os.path.getsize(lock) if os.path.exists(lock) else "missing")
    os.remove(lock)
    repository = new_repository("split-index-judge-writes-git-directory")
    git(repository, "config", "core.splitIndex", "true")
    git(repository, "update-index", "--split-index")
    moment = time.time() + 5
    os.utime(os.path.join(repository, "base.txt"), (moment, moment))
    git_directory = os.path.join(repository, ".git")
    before = sorted(entry for entry in os.listdir(git_directory) if entry.startswith("sharedindex."))
    with open(os.path.join(git_directory, "index"), "rb") as handle:
        index_before = hashlib.sha256(handle.read()).hexdigest()
    run = feed_hook(SETTINGS["hook"], payload_for("git commit -qam x", repository),
                    hook_environment(os.path.join(SETTINGS["detections"], "split-index-judge-writes-git-directory.jsonl")))
    after = sorted(entry for entry in os.listdir(git_directory) if entry.startswith("sharedindex."))
    with open(os.path.join(git_directory, "index"), "rb") as handle:
        index_after = hashlib.sha256(handle.read()).hexdigest()
    emit("split-index-judge-writes-git-directory", question=7, hook_exit=run.code, shared_index_files_before=len(before),
         shared_index_files_after=len(after), user_index_bytes_changed=yes_no(index_before != index_after))


def write_scope_cases():
    """第 8 问：写范围表新行下，⑫ 对三方攻方腿在 Bash 里写各处判成什么。只喂钩子，不执行；工作根用 SINGLEFS_WORK_ROOT 指到运行目录。"""
    locked = os.path.join(SETTINGS["work"], "another-agents-draft")
    os.makedirs(locked)
    with open(os.path.join(locked, ".writer-lock"), "w", encoding="utf-8") as handle:
        json.dump({"session_id": "another-session", "nonce": "probe", "subagent_type": "three-way-defense"}, handle)
    frozen_copy = os.path.join(SETTINGS["work"], "some-round-frozen")
    worktree_like = os.path.join(SETTINGS["work"], "worktree-under-work-root", "crates", "planted.rs")
    prefix = "research/prompts/fg-timeout-large-file-governance-"
    cases = (("scope-attack-writes-other-leg-report", f"printf x > {prefix}r2-sonnet-output.md"),
             ("scope-attack-writes-earlier-round-report", f"printf x > {prefix}r1-opus-output.md"),
             ("scope-attack-writes-other-leg-model", f"printf x > {prefix}r2-sonnet-model/planted.py"),
             ("scope-attack-writes-locked-draft-of-another-agent", f"printf x > {locked}/planted.txt"),
             ("scope-attack-writes-frozen-copy", f"printf x > {frozen_copy}/planted.txt"),
             ("scope-attack-writes-worktree-under-work-root", f"printf x > {worktree_like}"),
             ("scope-attack-writes-crates-control", "printf x > crates/planted.rs"),
             ("scope-attack-writes-own-report-control", f"printf x > {prefix}r2-opus-output.md"))
    for name, command in cases:
        run = feed_hook(SETTINGS["hook"], payload_for(command, SETTINGS["frozen"], agent_type="three-way-attack"),
                        hook_environment(os.path.join(SETTINGS["detections"], name + ".jsonl"), {"SINGLEFS_WORK_ROOT": SETTINGS["work"]}))
        keep_stderr(name, run.err)
        emit(name, question=8, hook_exit=run.code, refused=yes_no(run.code == 2), refused_by_write_scope=yes_no("写范围" in run.err), command=command)


def judged_hashes():
    listing = os.path.join(SETTINGS["repo"], "research", "prompts", "fg-timeout-large-file-governance-r2-snapshot", "judged-sha256.txt")
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
    for key in ("repositories", "remotes", "detections", "stderr"):
        SETTINGS[key] = os.path.join(work, key)
        os.makedirs(SETTINGS[key])
    SETTINGS["global_config"] = os.path.join(work, "gitconfig")
    with open(SETTINGS["global_config"], "w", encoding="utf-8") as handle:
        handle.write("[init]\n\tdefaultBranch = main\n[commit]\n\tgpgsign = false\n[protocol \"file\"]\n\tallow = always\n"
                     "[advice]\n\tdetachedHead = false\n[push]\n\tdefault = simple\n")
    emit("settings", limit=SETTINGS["limit"], frozen=SETTINGS["frozen"], work=work,
         git=subprocess.run(["git", "--version"], capture_output=True, text=True).stdout.strip(),
         work_root_variable=os.environ.get("SINGLEFS_WORK_ROOT", "unset"))
    judged_hashes()
    groups = [("alias-launcher", alias_and_launcher_cases), ("preceding-change", preceding_change_cases), ("misreject", misreject_cases),
              ("work-root-home", work_root_home_cases), ("push", push_question_cases), ("judge-runtime", judge_runtime_cases),
              ("write-scope", write_scope_cases)]
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
