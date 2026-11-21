#!/usr/bin/env python3
"""辩方腿探针：在临时小仓里复现第二轮辩方报告点名的几处现象，直接调 scripts/agents/agent-watch.py 里的函数，打出原样读数；只打现象、不判对错。
用法：defense_probe.py --repo <仓根> --work <运行目录，要不存在或是空目录>
  --repo  取 <仓根>/scripts/agents/agent-watch.py 当被测模块（只读它，不碰 <仓根> 的 git）
  --work  小仓与落盘文件都建在这个目录下
每格打一行 DEFENSE name=<格名> <键>=<值>；首行 name=watchdog 打被测模块的 sha256，末行 name=done emitted=<前面 DEFENSE 行数>。
没有长命令与常驻进程：只起 git（带 60 秒时限），命名管道只造、不打开。
"""
import argparse
import hashlib
import importlib.util
import json
import os
import subprocess
import sys
import time

os.environ["GIT_CONFIG_GLOBAL"] = os.devnull
os.environ["GIT_CONFIG_NOSYSTEM"] = "1"
for dropped in ("GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_COMMON_DIR"):
    os.environ.pop(dropped, None)

emitted_names = []


def emit(name, **fields):
    emitted_names.append(name)
    print("DEFENSE name=" + name + "".join(f" {key}={value!r}" for key, value in fields.items()), flush=True)


def load_watch(repo):
    path = os.path.join(repo, "scripts", "agents", "agent-watch.py")
    with open(path, "rb") as handle:
        emit("watchdog", path=path, sha256=hashlib.sha256(handle.read()).hexdigest())
    spec = importlib.util.spec_from_file_location("agent_watch_under_probe", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def git(directory, *arguments):
    return subprocess.run(["git", "-c", "user.name=probe", "-c", "user.email=probe@example.invalid", "-c", "commit.gpgsign=false",
                           "-c", f"core.hooksPath={os.devnull}", "-C", directory, *arguments],
                          check=True, capture_output=True, text=True, timeout=60).stdout


def write(directory, relative_path, text):
    path = os.path.join(directory, relative_path)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)


def new_repository(work, name, files):
    directory = os.path.join(work, name)
    os.makedirs(directory)
    git(directory, "init", "-q")
    git(directory, "symbolic-ref", "HEAD", "refs/heads/probe-branch")
    for relative_path, text in files.items():
        write(directory, relative_path, text)
    git(directory, "add", "-A")
    git(directory, "commit", "-q", "-m", "起点")
    return directory


def arguments_of(repository, limit=20):
    return argparse.Namespace(repository_report_lines=limit, repository_root=repository, max_minutes=60)


def knowledge_since(watch, repository, baseline):
    now = watch.repository_snapshot(repository)
    _, commit_paths = watch.new_commits_since(repository, baseline.snapshot, now)
    return watch.knowledge_files_changed(baseline.snapshot, now, commit_paths), now


def scenario_line_bound(watch, work):
    repository = new_repository(work, "line-bound", {".claude/main-agent.md": "原文\n", ".claude/kb/seed.md": "原文\n"})
    baseline = watch.take_repository_baseline(arguments_of(repository), time.time(), None)
    for index in range(30):
        write(repository, f".claude/kb/page-{index:02d}.md", "页\n")
        git(repository, "add", "--", f".claude/kb/page-{index:02d}.md")
        git(repository, "commit", "-q", "-m", f"新提交 {index:02d}")
        write(repository, f"outside-{index:02d}.txt", "范围外\n")
    lines = watch.repository_status_section(baseline, arguments_of(repository))[0]
    emit("line-bound", limit=20, lines=len(lines), bound_by_reading=3 * 20 + 15, widest=max(len(line) for line in lines))


def scenario_chmod(watch, work):
    clean = new_repository(work, "chmod-clean", {".claude/main-agent.md": "原文\n", ".claude/kb/k.md": "原文\n"})
    baseline = watch.take_repository_baseline(arguments_of(clean), time.time(), None)
    os.chmod(os.path.join(clean, ".claude/kb/k.md"), 0o755)
    knowledge, now = knowledge_since(watch, clean, baseline)
    emit("chmod-clean-at-baseline", baseline_status=sorted(baseline.snapshot.status_by_path.items()), status_now=sorted(now.status_by_path.items()), named=knowledge)
    dirty = new_repository(work, "chmod-dirty", {".claude/main-agent.md": "原文\n", ".claude/kb/k.md": "原文\n"})
    write(dirty, ".claude/kb/k.md", "基线时已改\n")
    git(dirty, "add", "--", ".claude/kb/k.md")
    baseline = watch.take_repository_baseline(arguments_of(dirty), time.time(), None)
    os.chmod(os.path.join(dirty, ".claude/kb/k.md"), 0o755)
    knowledge, now = knowledge_since(watch, dirty, baseline)
    emit("chmod-dirty-staged-at-baseline", baseline_status=sorted(baseline.snapshot.status_by_path.items()), status_now=sorted(now.status_by_path.items()), named=knowledge)


def scenario_rename_config(watch, work):
    config_path = os.path.join(work, "renames-off.gitconfig")
    with open(config_path, "w", encoding="utf-8") as handle:
        handle.write("[status]\n\trenames = false\n[diff]\n\trenames = false\n")
    for index, (label, config) in enumerate((("没有配置", os.devnull), ("status.renames=false", config_path))):
        os.environ["GIT_CONFIG_GLOBAL"] = config
        try:
            repository = new_repository(work, f"rename-{index}", {".claude/main-agent.md": "原文\n", ".claude/rules/old-name.md": "一条规则\n" * 20})
            baseline = watch.take_repository_baseline(arguments_of(repository), time.time(), None)
            git(repository, "mv", "--", ".claude/rules/old-name.md", ".claude/rules/new-name.md")
            knowledge, now = knowledge_since(watch, repository, baseline)
            emit("rename-config", label=label, status=sorted(now.status_by_path.items()), named=knowledge, counts=watch.status_counts(now.status_by_path))
        finally:
            os.environ["GIT_CONFIG_GLOBAL"] = os.devnull


def state_of(watch, work, name):
    directory = os.path.join(work, f"state-{name}")
    os.makedirs(directory)
    return directory, os.path.join(directory, watch.KNOWLEDGE_STATE_FILE_NAME)


def scenario_persistent_failure(watch, work):
    repository = new_repository(work, "persistent-failure", {".claude/main-agent.md": "原文\n", ".claude/kb/k.md": "原文\n"})
    _, state_path = state_of(watch, work, "persistent-failure")
    problem = watch.save_knowledge_state(state_path, repository, watch.repository_snapshot(repository), time.time() - 7200)
    baseline = watch.take_repository_baseline(arguments_of(repository), time.time(), state_path)
    write(repository, ".claude/kb/k.md", "基线之后改了\n")
    saved_timeout = os.environ.get(watch.DIGEST_TIMEOUT_VARIABLE)
    os.environ[watch.DIGEST_TIMEOUT_VARIABLE] = "0"
    try:
        printed = [watch.knowledge_sync_section(baseline, arguments_of(repository), on_alert=True) for _ in range(3)]
    finally:
        if saved_timeout is None:
            os.environ.pop(watch.DIGEST_TIMEOUT_VARIABLE, None)
        else:
            os.environ[watch.DIGEST_TIMEOUT_VARIABLE] = saved_timeout
    with open(state_path, encoding="utf-8") as handle:
        reminded_after = json.load(handle)["reminded_at"]
    emit("persistent-failure-repeats", save_problem=problem, printed_lines_per_alert_exit=[len(lines) for lines in printed],
         second_line_head=printed[0][1][:48] if len(printed[0]) > 1 else "", reminded_at_unchanged=reminded_after == baseline.reminded_at)


def scenario_duplicate_reminders(watch, work):
    repository = new_repository(work, "duplicates", {".claude/main-agent.md": "原文\n"})
    _, state_path = state_of(watch, work, "duplicates")
    watch.save_knowledge_state(state_path, repository, watch.repository_snapshot(repository), time.time() - 7200)
    reader = watch.take_repository_baseline(arguments_of(repository), time.time(), state_path)
    other_reader = watch.take_repository_baseline(arguments_of(repository), time.time(), state_path)
    first = watch.knowledge_sync_section(reader, arguments_of(repository), on_alert=True)
    second = watch.knowledge_sync_section(other_reader, arguments_of(repository), on_alert=True)
    emit("duplicate-reminders", both_loaded_same_state=reader.reminded_at == other_reader.reminded_at, first_printed=len(first), second_printed=len(second))


def scenario_non_utf8_symlink(watch, work):
    repository = new_repository(work, "non-utf8-link", {".claude/main-agent.md": "原文\n", ".claude/kb/seed.md": "原文\n"})
    directory, state_path = state_of(watch, work, "non-utf8-link")
    # 链接的目标不是合法 UTF-8：摘要取 os.readlink 的 str（带代理字符），落盘按 UTF-8 编码时才会出错
    os.makedirs(os.path.join(repository, ".claude/kb"), exist_ok=True)
    os.symlink(b"\xff\xfe-target", os.path.join(os.fsencode(repository), b".claude", b"kb", b"odd-link"))
    try:
        baseline = watch.take_repository_baseline(arguments_of(repository), time.time(), state_path)
        outcome = f"返回 snapshot_is_none={baseline.snapshot is None}"
    except Exception as error:
        outcome = f"抛出 {type(error).__name__}"
    emit("non-utf8-symlink-target", outcome=outcome, state_directory_entries=sorted(os.listdir(directory)))


def scenario_fifo_reach(watch, work):
    repository = new_repository(work, "fifo-reach", {".gitignore": "/.claude/singlefs-ai-sop/\n", ".claude/main-agent.md": "原文\n"})
    write(repository, ".claude/singlefs-ai-sop/rules/plain.md", "规则\n")
    os.mkfifo(os.path.join(repository, ".claude/singlefs-ai-sop/rules/channel"))
    os.makedirs(os.path.join(repository, ".claude/kb/only-pipe"))
    os.mkfifo(os.path.join(repository, ".claude/kb/only-pipe/channel"))
    emit("fifo-reach", ignored_listed=watch.ignored_knowledge_paths(repository), status_keys=sorted(watch.repository_snapshot(repository).status_by_path))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True)
    parser.add_argument("--work", required=True)
    arguments = parser.parse_args()
    if os.path.exists(arguments.work) and os.listdir(arguments.work):
        print("运行目录已存在且非空", file=sys.stderr)
        return 2
    os.makedirs(arguments.work, exist_ok=True)
    watch = load_watch(arguments.repo)
    for scenario in (scenario_line_bound, scenario_chmod, scenario_rename_config, scenario_persistent_failure, scenario_duplicate_reminders,
                     scenario_non_utf8_symlink, scenario_fifo_reach):
        try:
            scenario(watch, arguments.work)
        except Exception as error:
            emit(scenario.__name__.replace("scenario_", "").replace("_", "-") + "-exception", exception=f"{type(error).__name__}: {error}")
    print(f"DEFENSE name=done emitted={len(emitted_names)}", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
