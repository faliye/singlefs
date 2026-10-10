#!/usr/bin/env python3
"""把冻结副本 crates/mutations.tsv 的第 N 行（1 起）施加到自己的拷贝上，或还原。
用法：mutate_row.py apply <行号> | restore <行号>
行号指向的行：标签 TAB 文件 TAB 原文 TAB 替换文 TAB 命令 TAB 用例名；原文/替换文里的 \\n 是换行。
原文必须在拷贝文件里恰好命中一次。"""
import sys, shutil, os
FROZEN = os.path.expanduser("~/.local/share/singlefs/work/m3-batch58-code-r1/repo")
MINE = os.path.expanduser("~/.local/share/singlefs/work/m3-batch58-code-r1-sonnet/repo")
mode, number = sys.argv[1], int(sys.argv[2])
with open(f"{FROZEN}/crates/mutations.tsv", encoding="utf-8") as f:
    row = f.read().split("\n")[number - 1].split("\t")
label, path, old, new = row[0], row[1], row[2].replace("\\n", "\n"), row[3].replace("\\n", "\n")
mine = f"{MINE}/{path}"
if mode == "restore":
    shutil.copyfile(f"{FROZEN}/{path}", mine)
    print("restored", path)
    sys.exit(0)
text = open(mine, encoding="utf-8").read()
hits = text.count(old)
if hits != 1:
    print(f"锚点命中 {hits} 次，不是 1：{label}")
    sys.exit(3)
open(mine, "w", encoding="utf-8").write(text.replace(old, new))
print("applied", label, "->", path)
print("COMMAND", row[4])
print("TESTNAME", row[5])
