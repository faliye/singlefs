#!/usr/bin/env python3
"""H2：真登记那一行与「改动范围剔掉 726 份」的对照，用不经库里换算的独立写法再量一次。
独立：只用 git 的 ls-tree / cat-file / diff 与 python 的 bytes.replace，不 import 库里的 rewrite_moved_*、boundary、size_may_match。
用法：probe_real.py --frozen <冻结副本> --repo <主工作区> --base <gate 基准>   （--repo 只读，不写）"""
import argparse
import os
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import load_library  # noqa: E402

OLD, NEW = "scripts/experiments", "scripts/experiment-apparatus"


def git(repo, *arguments, binary=False):
    done = subprocess.run(["git", "-C", repo, "-c", "core.quotepath=false", *arguments], capture_output=True)
    if done.returncode != 0:
        raise SystemExit(f"git {arguments} 退 {done.returncode}: {done.stderr.decode()[:200]}")
    return done.stdout if binary else done.stdout.decode("utf-8", "surrogateescape")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--frozen", required=True)
    parser.add_argument("--repo", required=True)
    parser.add_argument("--base", required=True)
    arguments = parser.parse_args()
    repo, base = arguments.repo, arguments.base
    # 一、真登记那一行：基准里 OLD 下的文件
    listing = git(repo, "ls-tree", "-r", "--full-tree", base, "--", OLD).split("\n")
    files = []
    for record in listing:
        if record:
            meta, _tab, name = record.partition("\t")
            files.append((name, meta.split(" ")[2]))
    present = raw_same = naive_same = 0
    different = []
    for name, oid in files:
        target = NEW + name[len(OLD):]
        path = os.path.join(repo, target)
        if not os.path.isfile(path):
            continue
        present += 1
        blob = subprocess.run(["git", "-C", repo, "cat-file", "blob", oid], capture_output=True).stdout
        with open(path, "rb") as handle:
            current = handle.read()
        raw_same += current == blob
        if current == blob.replace(OLD.encode(), NEW.encode()):
            naive_same += 1
        else:
            different.append(name)
    print(f"name=real_row base_files={len(files)} 落点在={present} 字节原样相同={raw_same} 整串替换之后相同={naive_same}")
    # 二、改动范围里剔掉的份数：库的 moved_only_paths 与独立写法比
    lag = load_library(arguments.frozen)
    tracked = git(repo, "diff", "--name-only", "--no-renames", base).split("\n")
    untracked = git(repo, "ls-files", "--others", "--exclude-standard").split("\n")
    paths = sorted({path for path in tracked + untracked if path})
    stripped, problem = lag.moved_only_paths(base, paths, cwd=repo)
    tree = {}
    for record in git(repo, "ls-tree", "-r", "--full-tree", base).split("\n"):
        if record:
            meta, _tab, name = record.partition("\t")
            mode, _kind, oid = meta.split(" ")
            tree[name] = (mode, oid)
    old_of = lambda p: OLD + p[len(NEW):] if p == NEW or p.startswith(NEW + "/") else p  # noqa: E731
    independent = set()
    cache = {}

    def blob(oid):
        if oid not in cache:
            cache[oid] = subprocess.run(["git", "-C", repo, "cat-file", "blob", oid], capture_output=True).stdout
        return cache[oid]

    for path in paths:
        source = old_of(path)
        if source in tree and tree[source][0] in ("100644", "100755"):
            full = os.path.join(repo, path)
            if os.path.isfile(full) and not os.path.islink(full):
                mode = "100755" if os.stat(full).st_mode & 0o100 else "100644"
                with open(full, "rb") as handle:
                    current = handle.read()
                before = blob(tree[source][1])
                if mode == tree[source][0] and (current == before or current == before.replace(OLD.encode(), NEW.encode())):
                    independent.add(path)
                    continue
        # 工作区里不在的旧路径：落点那一份与基准里这一份比
        if path.startswith(OLD + "/") and path in tree:
            target = NEW + path[len(OLD):]
            full = os.path.join(repo, target)
            if not os.path.exists(os.path.join(repo, path)) and os.path.isfile(full):
                with open(full, "rb") as handle:
                    current = handle.read()
                if current == blob(tree[path][1]).replace(OLD.encode(), NEW.encode()):
                    independent.add(path)
    print(f"name=moved_only 改动范围={len(paths)} 库剔掉={len(stripped)} 独立写法剔掉={len(independent)} 库有独立没有={len(stripped - independent)} 独立有库没有={len(independent - stripped)} 问题={problem}")
    for label, delta in (("库有独立没有", sorted(stripped - independent)[:5]), ("独立有库没有", sorted(independent - stripped)[:5])):
        print(f"  {label}前五条={delta}")


main()
