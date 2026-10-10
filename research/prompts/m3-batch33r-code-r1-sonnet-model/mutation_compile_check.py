#!/usr/bin/env python3
"""R4 的编译核：对变异表第 16 行与第 3102–3109 行，逐行在「冻结副本的拷贝」里把原文换成替换文（原文必须恰好命中一次），
跑 `cargo check --offline -p singlefs-checker-tier --lib`（只编不跑），再把文件还原成冻结副本里的样子。
用法：SINGLEFS_FROZEN_TREE=<冻结副本> SINGLEFS_COPY=<拷贝> CARGO_TARGET_DIR=<编译目录> python3 mutation_compile_check.py <行号> [<行号> …]
输出行以 MUTCHECK 开头；cargo 经 capped.sh 与 nice（线程上限由 SINGLEFS_THREADS 给，默认 6）。"""
import os, shutil, subprocess, sys
frozen = os.environ["SINGLEFS_FROZEN_TREE"]; copy = os.environ["SINGLEFS_COPY"]
threads = os.environ.get("SINGLEFS_THREADS", "6")
repo = os.environ.get("SINGLEFS_REPO", os.path.expanduser("~/code/singlefs"))
rows = open(os.path.join(frozen, "crates/mutations.tsv"), encoding="utf-8").read().split("\n")
for number in map(int, sys.argv[1:]):
    name, path, original, replacement, _args, test = rows[number - 1].split("\t")
    original, replacement = original.replace("\\n", "\n"), replacement.replace("\\n", "\n")
    if os.environ.get("SINGLEFS_MUTCHECK_SELFTEST") == "1":
        replacement += " /*selftest*/ @@broken@@"      # 自证会红：故意写坏的替换文，cargo check 必须报错
    source = open(os.path.join(frozen, path), encoding="utf-8").read()
    hits = source.count(original)
    if hits != 1:
        print(f"MUTCHECK row={number} anchor_hits={hits} (not applied)"); continue
    target = os.path.join(copy, path)
    open(target + ".mut", "w", encoding="utf-8").write(source.replace(original, replacement))
    os.replace(target + ".mut", target)
    try:
        result = subprocess.run(
            ["nice", "-n", "19", "bash", os.path.join(repo, "research/scripts/capped.sh"), threads,
             "cargo", "check", "--offline", "-p", "singlefs-checker-tier", "--lib", "--message-format", "short"],
            cwd=copy, capture_output=True, text=True)
        errors = [line for line in result.stderr.splitlines() if ": error" in line or line.startswith("error")]
        compiled = "Checking singlefs-checker-tier" in result.stderr
        print(f"MUTCHECK row={number} anchor_hits=1 cargo_check_exit={result.returncode} error_lines={len(errors)} recompiled_checker_tier={compiled} named_test={test[:70]}")
        for line in errors[:3]:
            print("MUTCHECK   " + line[:200])
    finally:
        shutil.copyfile(os.path.join(frozen, path), target + ".restore")
        os.replace(target + ".restore", target)
