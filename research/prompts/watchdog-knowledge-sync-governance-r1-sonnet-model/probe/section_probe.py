#!/usr/bin/env python3
"""正推腿探针：在临时小仓里把「知识同步提醒」的几个边界各造一遍 git 操作，直接调 scripts/agents/agent-watch.py 里的函数，打出它们的原样结果。
用法：section_probe.py --repo <仓根> --work <运行目录>
  --repo  取 <仓根>/scripts/agents/agent-watch.py 当被测模块（只读它，不碰 <仓根> 的 git）
  --work  小仓都建在这个目录下（要不存在或是空目录）
每格打若干行 name=<格名> <键>=<值>：status（模块看到的 git status --short）、commits（新提交与它们动到的路径）、knowledge（点名的知识类文件与标签、份数、计数）、others（别的变了的路径）。
只打现象、不判对错；判读写在报告里。末行 name=probe_done cases=<格数>。
"""
import argparse
import importlib.util
import os
import subprocess
import sys

os.environ["GIT_CONFIG_GLOBAL"] = os.devnull       # 小仓里的 git（含被测模块起的）不读用户与系统的配置
os.environ["GIT_CONFIG_NOSYSTEM"] = "1"
for dropped in ("GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE"):
    os.environ.pop(dropped, None)


def load_watch(repo):
    spec = importlib.util.spec_from_file_location("agent_watch_under_probe", os.path.join(repo, "scripts", "agents", "agent-watch.py"))
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


def show(watch, name, directory, start):
    now = watch.repository_snapshot(directory)
    commit_rows, commit_paths = watch.new_commits_since(directory, start, now)
    knowledge = watch.knowledge_files_changed(start, now, commit_paths)
    others = watch.paths_changed_since(start.status_by_path, now.status_by_path)
    print(f"name={name} status={sorted(now.status_by_path.items())!r}")
    print(f"name={name} commits={commit_rows!r} commit_paths={commit_paths!r}")
    print(f"name={name} knowledge={knowledge!r} count={len(knowledge)} counts={watch.status_counts(now.status_by_path)!r}")
    print(f"name={name} others={[row for _, row in others]!r}")
    return now


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True)
    parser.add_argument("--work", required=True)
    arguments = parser.parse_args()
    if os.path.exists(arguments.work) and os.listdir(arguments.work):
        print(f"✗ --work {arguments.work} 已存在且非空", file=sys.stderr)
        return 2
    os.makedirs(arguments.work, exist_ok=True)
    work = os.path.abspath(arguments.work)
    watch = load_watch(arguments.repo)
    cases = 0

    # 一、起来之后新建了一个带三份文件的未跟踪目录（git status --short 写成一行）：点名几份、计数几个
    directory = new_repository(work, "untracked-directory", {"a.txt": "a\n", ".claude/kb/seed.md": "x\n"})
    start = watch.repository_snapshot(directory)
    for file_name in ("one.md", "two.md", "three.md"):
        write(directory, f".claude/kb/newdir/{file_name}", "新\n")
    show(watch, "untracked-directory", directory, start)
    cases += 1

    # 二、范围表里的 .claude/singlefs-ai-sop/rules/ 被仓的 .gitignore 忽略（本仓的 .gitignore 第 8 行是 /.claude/singlefs-ai-sop/）：起来之后改了它底下的文件
    directory = new_repository(work, "ignored-scope", {"a.txt": "a\n", ".gitignore": "/.claude/singlefs-ai-sop/\n"})
    write(directory, ".claude/singlefs-ai-sop/rules/sample.md", "起来时\n")
    start = watch.repository_snapshot(directory)
    write(directory, ".claude/singlefs-ai-sop/rules/sample.md", "之后改了\n")
    write(directory, ".claude/singlefs-ai-sop/rules/second.md", "之后新建\n")
    print(f"name=ignored-scope raw_status={watch.run_git(directory, ['status', '--short'])!r}")
    show(watch, "ignored-scope", directory, start)
    cases += 1

    # 三、路径带空格、带中文
    directory = new_repository(work, "space-and-cjk-path", {"a.txt": "a\n", ".claude/kb/seed.md": "x\n"})
    start = watch.repository_snapshot(directory)
    write(directory, ".claude/kb/two words.md", "空格\n")
    write(directory, ".claude/kb/中文名.md", "中文\n")
    print(f"name=space-and-cjk-path raw_status={watch.run_git(directory, ['status', '--short'])!r}")
    show(watch, "space-and-cjk-path", directory, start)
    cases += 1

    # 四、改名：一头在范围里、一头在范围外（两个方向各一条）
    directory = new_repository(work, "rename-across-scope", {"a.txt": "a\n", "docs/old.md": "x\n", ".claude/kb/old-kb.md": "y\n"})
    start = watch.repository_snapshot(directory)
    git(directory, "mv", "docs/old.md", ".claude/kb/moved-in.md")
    git(directory, "mv", ".claude/kb/old-kb.md", "docs/moved-out.md")
    show(watch, "rename-across-scope", directory, start)
    cases += 1

    # 五、起来之后 HEAD 退回去（git reset --hard HEAD~1）：没有新提交，两个 HEAD 之间不同的路径怎么标
    directory = new_repository(work, "head-moved-backward", {"a.txt": "a\n"})
    write(directory, ".claude/kb/second.md", "第二条提交带的\n")
    git(directory, "add", "-A")
    git(directory, "commit", "-q", "-m", "第二条")
    start = watch.repository_snapshot(directory)
    git(directory, "reset", "-q", "--hard", "HEAD~1")
    show(watch, "head-moved-backward", directory, start)
    cases += 1

    # 六、起来时未跟踪目录里有一个指向目录的符号链接，之后把链接改指别处：内容摘要变不变
    outside_first = os.path.join(work, "link-target-first")
    outside_second = os.path.join(work, "link-target-second")
    write(outside_first, "inside.md", "第一个目标\n")
    write(outside_second, "inside.md", "第二个目标，内容不同\n")
    directory = new_repository(work, "symlink-in-untracked-directory", {"a.txt": "a\n", ".claude/kb/seed.md": "x\n"})
    write(directory, ".claude/kb/packdir/real.md", "真文件\n")
    os.symlink(outside_first, os.path.join(directory, ".claude/kb/packdir/linked"))
    start = watch.repository_snapshot(directory)
    os.remove(os.path.join(directory, ".claude/kb/packdir/linked"))
    os.symlink(outside_second, os.path.join(directory, ".claude/kb/packdir/linked"))
    now = show(watch, "symlink-in-untracked-directory", directory, start)
    print(f"name=symlink-in-untracked-directory digest_equal={start.digest_by_path == now.digest_by_path}")
    cases += 1

    # 七、起来时 .claude/kb/seed.md 已改着（未暂存），之后只 git add（内容没动）
    directory = new_repository(work, "stage-only", {"a.txt": "a\n", ".claude/kb/seed.md": "原文\n"})
    write(directory, ".claude/kb/seed.md", "原文\n已改\n")
    start = watch.repository_snapshot(directory)
    git(directory, "add", ".claude/kb/seed.md")
    show(watch, "stage-only", directory, start)
    cases += 1

    # 八、起来时已改着，之后只碰 mtime（内容与状态码都没变）
    directory = new_repository(work, "mtime-only", {"a.txt": "a\n", ".claude/kb/seed.md": "原文\n"})
    write(directory, ".claude/kb/seed.md", "原文\n已改\n")
    start = watch.repository_snapshot(directory)
    os.utime(os.path.join(directory, ".claude/kb/seed.md"), (1_000_000_000, 1_000_000_000))
    show(watch, "mtime-only", directory, start)
    cases += 1

    # 九、新提交删掉一份知识类文件
    directory = new_repository(work, "deleted-by-commit", {"a.txt": "a\n", ".claude/kb/gone.md": "会被删\n"})
    start = watch.repository_snapshot(directory)
    git(directory, "rm", "-q", ".claude/kb/gone.md")
    git(directory, "commit", "-q", "-m", "删")
    show(watch, "deleted-by-commit", directory, start)
    cases += 1

    # 十、起来时已改着，之后提交了、又接着改
    directory = new_repository(work, "commit-then-edit-again", {"a.txt": "a\n", ".claude/kb/seed.md": "原文\n"})
    write(directory, ".claude/kb/seed.md", "原文\n起来时已改\n")
    start = watch.repository_snapshot(directory)
    git(directory, "commit", "-q", "-a", "-m", "提交起来时已改的")
    write(directory, ".claude/kb/seed.md", "原文\n起来时已改\n提交之后又改\n")
    show(watch, "commit-then-edit-again", directory, start)
    cases += 1

    print(f"name=probe_done cases={cases}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
