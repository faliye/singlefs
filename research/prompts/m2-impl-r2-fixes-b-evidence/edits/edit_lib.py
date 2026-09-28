"""我的改动写成可重放的定点替换：别的会话在同一批文件上挪路径、改 use 行，最后在主工作区的新副本上重放一遍出补丁。"""
import os
import sys

ROOT = sys.argv[1] if len(sys.argv) > 1 else os.environ["EDIT_ROOT"]


def edit(relative_path, old, new, count=1):
    path = os.path.join(ROOT, relative_path)
    text = open(path, encoding="utf-8").read()
    found = text.count(old)
    if found != count:
        raise SystemExit(f"{relative_path}: 旧串命中 {found} 次，要 {count} 次：{old[:80]!r}")
    open(path, "w", encoding="utf-8").write(text.replace(old, new))


def replace_everywhere(relative_path, old, new):
    path = os.path.join(ROOT, relative_path)
    text = open(path, encoding="utf-8").read()
    if old not in text:
        raise SystemExit(f"{relative_path}: 没有 {old!r}")
    open(path, "w", encoding="utf-8").write(text.replace(old, new))
