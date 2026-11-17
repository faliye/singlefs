#!/usr/bin/env python3
"""攻方腿模型的改文件工具（只改 --work 底下拷出来的树）：
  apply_once.py replace <文件> <原文文件> <替换文文件>   原文在文件里恰好命中一次才换，否则退 1
  apply_once.py insert-tests <文件> <测试块文件>         把测试块插在文件最后一个右花括号（mod tests 收尾）之前，文件末尾不是右花括号退 1"""
import sys


def main(arguments):
    command, path = arguments[0], arguments[1]
    with open(path, encoding="utf-8") as handle:
        text = handle.read()
    if command == "replace":
        with open(arguments[2], encoding="utf-8") as handle:
            anchor = handle.read()
        with open(arguments[3], encoding="utf-8") as handle:
            replacement = handle.read()
        hits = text.count(anchor)
        if hits != 1:
            print(f"apply_once：{arguments[2]} 在 {path} 里命中 {hits} 次，不是恰好一次", file=sys.stderr)
            return 1
        text = text.replace(anchor, replacement)
    elif command == "insert-tests":
        with open(arguments[2], encoding="utf-8") as handle:
            block = handle.read()
        stripped = text.rstrip()
        if not stripped.endswith("}"):
            print(f"apply_once：{path} 末尾不是右花括号", file=sys.stderr)
            return 1
        text = stripped[:-1] + block + "}\n"
    else:
        print(f"apply_once：认不出的子命令 {command}", file=sys.stderr)
        return 2
    with open(path + ".next", "w", encoding="utf-8") as handle:
        handle.write(text)
    import os
    os.replace(path + ".next", path)
    print(f"apply_once：{command} {path} 成")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
