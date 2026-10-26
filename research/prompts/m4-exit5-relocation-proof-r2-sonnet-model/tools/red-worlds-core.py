#!/usr/bin/env python3
"""core 里直验的原型（patches/ 里的 s1 与 m4）的必红世界：逐行改坏一处，`cargo verus focus -p singlefs-core` 必须报证明错。

用法：red-worlds-core.py <已打补丁的最小工作区> <行表 tsv> <工作目录> <cargo 命令前缀…>
行表四列（制表符分隔）：相对工作区根的文件、世界名、原文、替换文。原文必须在文件里恰好命中一次。
每个世界拷一份工作区到 <工作目录>/<序号>/，各自一个编译目录（cargo 的指纹按修改时间判，拷贝保留了旧修改时间，共用编译目录会拿旧结果）。
"""
import os
import re
import shutil
import subprocess
import sys

tree, table, work, *prefix = sys.argv[1:]
env = dict(os.environ)
env["PATH"] = os.path.expanduser("~/.local/share/singlefs/formal-tools/verus-0.2026.09.20/verus-x86-linux") + ":" + env["PATH"]
for number, row in enumerate(open(table, encoding="utf-8")):
    if not row.strip() or row.startswith("#"):
        continue
    relative, name, old, new = row.rstrip("\n").split("\t")
    old = old.replace("\\n", "\n")
    new = new.replace("\\n", "\n")
    copy = os.path.join(work, f"world{number}")
    shutil.copytree(tree, copy, ignore=shutil.ignore_patterns("target"))
    path = os.path.join(copy, relative)
    text = open(path, encoding="utf-8").read()
    if text.count(old) != 1:
        print(f"{name}\t原文命中{text.count(old)}次，跳过")
        continue
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text.replace(old, new))
    own = dict(env, CARGO_TARGET_DIR=os.path.join(work, f"target{number}"))
    ran = subprocess.run([*prefix, "cargo", "verus", "focus", "--offline", "-p", "singlefs-core", "--manifest-path", os.path.join(copy, "Cargo.toml")],
                         capture_output=True, text=True, env=own)
    text = ran.stdout + ran.stderr
    result = [line.strip() for line in text.splitlines() if line.startswith("verification results")]
    first = next((line for line in text.splitlines() if line.startswith("error") and "could not compile" not in line and "aborting" not in line), "")
    landing = ""
    lines = text.splitlines()
    for i, line in enumerate(lines):
        if line.startswith("error") and "could not compile" not in line:
            for follow in lines[i + 1:i + 4]:
                if "-->" in follow:
                    landing = re.sub(r"^.*(crates/singlefs-core/)", r"\1", follow.split("-->")[1].strip())
                    break
            break
    print(f"{name}\trc={ran.returncode}\t{result[-1] if result else '(无结果行)'}\t首个错：{first[:60]}\t落点={landing}")
    shutil.rmtree(copy, ignore_errors=True)
    shutil.rmtree(own["CARGO_TARGET_DIR"], ignore_errors=True)
