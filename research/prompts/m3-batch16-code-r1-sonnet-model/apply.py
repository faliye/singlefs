#!/usr/bin/env python3
"""在 WORKDIR/w（冻结树的拷贝）上施加一处变异。
用法：
  apply.py row <N>                     施加冻结树 crates/mutations.tsv 第 N 行（原文必须恰好命中一次）；输出「target=<cargo 目标> test=<必须红的测试名>」
  apply.py edit <文件> <原文> <替换文>   原文/替换文里的 \\n 是换行；原文必须恰好命中一次
  apply.py restore                     把变异动过的文件从冻结树拷回来
环境：FROZEN_TREE（默认 /tmp/claude-1000/m3-batch16-code-r1/tree）、WORKDIR（必填）。
"""
import os, shutil, sys

TREE = os.environ.get("FROZEN_TREE", "/tmp/claude-1000/m3-batch16-code-r1/tree").rstrip("/") + "/"
WORK = os.environ["WORKDIR"].rstrip("/") + "/w/"
TOUCHED = WORK + ".touched"


def remember(path):
    with open(TOUCHED, "a", encoding="utf-8") as handle:
        handle.write(path + "\n")


def mutate(path, original, replacement):
    shutil.copy(TREE + path, WORK + path)
    text = open(WORK + path, encoding="utf-8").read()
    old = original.replace("\\n", "\n")
    new = replacement.replace("\\n", "\n")
    count = text.count(old)
    assert count == 1, f"锚点命中 {count} 次：{path}"
    open(WORK + path, "w", encoding="utf-8").write(text.replace(old, new))
    remember(path)


mode = sys.argv[1]
if mode == "row":
    rows = open(TREE + "crates/mutations.tsv", encoding="utf-8").read().split("\n")
    name, path, original, replacement, arguments, test = rows[int(sys.argv[2]) - 1].split("\t")
    mutate(path, original, replacement)
    print("target=" + arguments.split("--test ")[1].strip() if "--test " in arguments else "target=--lib", "test=" + test)
elif mode == "edit":
    mutate(sys.argv[2], sys.argv[3], sys.argv[4])
    print("edited", sys.argv[2])
elif mode == "restore":
    if os.path.exists(TOUCHED):
        for path in sorted(set(open(TOUCHED, encoding="utf-8").read().split())):
            shutil.copy(TREE + path, WORK + path)
        os.remove(TOUCHED)
    print("restored")
