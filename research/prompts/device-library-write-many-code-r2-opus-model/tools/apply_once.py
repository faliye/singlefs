#!/usr/bin/env python3
"""攻方腿模型（device-library-write-many-code-r2）的改文件工具，只改 rerun.sh 在 --work 底下拷出来的树：
  apply_once.py replace <文件> <原文文件> <替换文文件>      原文在文件里恰好命中一次才换，否则退 1
  apply_once.py replace-row <文件> <变异表> <变异名>         照变异表（crates/mutations.tsv 的六段格式）里名字恰好相同的那一行换，
                                                            原文、替换文里的 \\n 还原成换行；那一行找不到、不止一行、原文不是恰好命中一次都退 1
  apply_once.py insert-tests <文件> <测试块文件>            把测试块插在文件最后一个右花括号（mod tests 收尾）之前，文件末尾不是右花括号退 1
每次成功打一行 apply_once：<子命令> <文件> 成。"""
import os
import sys


def read_text(path):
    with open(path, encoding="utf-8") as handle:
        return handle.read()


def replace_once(text, anchor, replacement, label):
    hits = text.count(anchor)
    if hits != 1:
        print(f"apply_once：{label} 的原文在文件里命中 {hits} 次，不是恰好一次", file=sys.stderr)
        return None
    return text.replace(anchor, replacement)


def row_of(table_path, name):
    rows = [line.rstrip("\n").split("\t") for line in read_text(table_path).split("\n")
            if line and not line.startswith("#")]
    matched = [fields for fields in rows if fields[0] == name]
    if len(matched) != 1 or len(matched[0]) != 6:
        print(f"apply_once：变异表 {table_path} 里名为「{name}」的成形行有 {len(matched)} 行，要恰好一行", file=sys.stderr)
        return None
    return matched[0][2].replace("\\n", "\n"), matched[0][3].replace("\\n", "\n")


def main(arguments):
    command, path = arguments[0], arguments[1]
    text = read_text(path)
    if command == "replace":
        text = replace_once(text, read_text(arguments[2]), read_text(arguments[3]), arguments[2])
    elif command == "replace-row":
        row = row_of(arguments[2], arguments[3])
        text = None if row is None else replace_once(text, row[0], row[1], arguments[3])
    elif command == "insert-tests":
        stripped = text.rstrip()
        if not stripped.endswith("}"):
            print(f"apply_once：{path} 末尾不是右花括号", file=sys.stderr)
            return 1
        text = stripped[:-1] + read_text(arguments[2]) + "}\n"
    else:
        print(f"apply_once：认不出的子命令 {command}", file=sys.stderr)
        return 2
    if text is None:
        return 1
    with open(path + ".next", "w", encoding="utf-8") as handle:
        handle.write(text)
    os.replace(path + ".next", path)
    print(f"apply_once：{command} {path} 成")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
