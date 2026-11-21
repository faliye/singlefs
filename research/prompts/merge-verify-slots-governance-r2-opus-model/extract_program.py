#!/usr/bin/env python3
"""从冻结的 merge-and-verify.sh 里取一段单引号赋值的内嵌 python（<名>=' 起头的那一行到只有一个 ' 的那一行），原样打到标准输出。

用法：extract_program.py <merge-and-verify.sh> <变量名>
取不到（没有起头那一行、没有收尾那一行）退 1、标准错误写原因。只读，不改任何文件。
"""
import sys


def extract(path, name):
    lines = open(path, encoding="utf-8").read().split("\n")
    start = None
    for index, line in enumerate(lines):
        if line == f"{name}='":
            start = index + 1
            break
    if start is None:
        raise LookupError(f"{path} 里没有「{name}='」那一行")
    for index in range(start, len(lines)):
        if lines[index] == "'":
            return "\n".join(lines[start:index]) + "\n"
    raise LookupError(f"{path} 里 {name} 没有收尾的「'」那一行")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("用法：extract_program.py <merge-and-verify.sh> <变量名>", file=sys.stderr)
        sys.exit(2)
    try:
        sys.stdout.write(extract(sys.argv[1], sys.argv[2]))
    except (OSError, LookupError) as error:
        print(f"  ✗ {error}", file=sys.stderr)
        sys.exit(1)
