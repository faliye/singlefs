#!/usr/bin/env python3
"""施加 / 撤回一行变异（只在草稿副本里用）。读法照 scripts/mutation/crates-mutation-rows.py：六段制表符分隔，只把 \\n 还原成换行。
  mut.py apply TREE (行号|变异名全名) [--table 表路径]   原文必须恰好命中一次，否则退 3；先把原文件备份到 TREE/.mutbak/
  mut.py apply-spec TREE 规格文件                          规格文件三行：文件相对路径 / 原文 / 替换文（\\n 还原成换行）
  mut.py restore TREE                                      把备份放回去并 touch（修改时间换新，cargo 才会重编），清掉备份
  mut.py show TREE 行号                                    打印这一行的六段
"""
import os, shutil, sys, time

def unescape(text):
    return text.replace("\\n", "\n")

def read_rows(path):
    with open(path, encoding="utf-8") as handle:
        return [line.rstrip("\n").split("\t") for line in handle]

def pick_row(tree, selector, table=None):
    rows = read_rows(table or os.path.join(tree, "crates", "mutations.tsv"))
    if selector.isdigit():
        return rows[int(selector) - 1], int(selector)
    hits = [(row, number) for number, row in enumerate(rows, 1) if row[0] == selector]
    if len(hits) != 1:
        sys.exit(f"名字命中 {len(hits)} 行：{selector}")
    return hits[0]

def backup_path(tree, relative):
    return os.path.join(tree, ".mutbak", relative.replace("/", "__"))

def apply(tree, relative, original, replacement):
    path = os.path.join(tree, relative)
    with open(path, encoding="utf-8") as handle:
        text = handle.read()
    hits = text.count(original)
    if hits != 1:
        print(f"ANCHOR_HITS {hits} {relative}")
        sys.exit(3)
    os.makedirs(os.path.join(tree, ".mutbak"), exist_ok=True)
    backup = backup_path(tree, relative)
    if os.path.exists(backup):
        sys.exit(f"已有备份没撤回：{backup}")
    shutil.copy2(path, backup)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text.replace(original, replacement, 1))
    print(f"APPLIED {relative}")

def restore(tree):
    directory = os.path.join(tree, ".mutbak")
    if not os.path.isdir(directory):
        print("RESTORED 0")
        return
    count = 0
    for name in sorted(os.listdir(directory)):
        relative = name.replace("__", "/")
        shutil.copyfile(os.path.join(directory, name), os.path.join(tree, relative))
        os.utime(os.path.join(tree, relative), None)
        os.remove(os.path.join(directory, name))
        count += 1
    os.rmdir(directory)
    print(f"RESTORED {count}")

def main(argv):
    command = argv[1]
    if command == "restore":
        restore(argv[2])
    elif command == "apply":
        table = argv[argv.index("--table") + 1] if "--table" in argv else None
        row, number = pick_row(argv[2], argv[3], table)
        assert len(row) == 6, f"第 {number} 行不是六段"
        print(f"ROW {number} {row[0]}")
        apply(argv[2], row[1], unescape(row[2]), unescape(row[3]))
    elif command == "apply-spec":
        with open(argv[3], encoding="utf-8") as handle:
            lines = handle.read().split("\n")
        apply(argv[2], lines[0], unescape(lines[1]), unescape(lines[2]))
    elif command == "show":
        row, number = pick_row(argv[2], argv[3])
        for index, field in enumerate(row):
            print(f"[{number}.{index}] {field}")
    else:
        sys.exit(__doc__)

main(sys.argv)
