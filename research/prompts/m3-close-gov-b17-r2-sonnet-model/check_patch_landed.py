#!/usr/bin/env python3
"""H1：修补补丁里每一处新写进去的片段，现在还在不在主工作区那份文件里。
用法：python3 -I check_patch_landed.py <仓根> <补丁>
对每个文件的每个 hunk，把 - 行与 + 行各自接成一段，字符级比对，取 insert / replace 的新片段（>=6 字，多行的按行拆），
在现文里逐字找（str.__contains__），打 present / ABSENT。"""
import difflib, os, sys

repo, patch_path = os.path.abspath(sys.argv[1]), sys.argv[2]
lines = open(patch_path, encoding="utf-8").read().split("\n")
current_file, minus, plus, results = None, [], [], []


def flush():
    global minus, plus
    if current_file and (minus or plus):
        a, b = "\n".join(minus), "\n".join(plus)
        text = open(os.path.join(repo, current_file), encoding="utf-8").read()
        for tag, i1, i2, j1, j2 in difflib.SequenceMatcher(None, a, b, autojunk=False).get_opcodes():
            if tag not in ("insert", "replace"):
                continue
            for piece in b[j1:j2].split("\n"):
                if len(piece.strip()) >= 6:
                    results.append((current_file, "present" if piece in text else "ABSENT", piece))
    minus, plus = [], []


for line in lines:
    if line.startswith("+++ "):
        flush()
        current_file = line[6:] if line.startswith("+++ b/") else line[4:]
    elif line.startswith("--- ") or line.startswith("@@"):
        flush()
    elif line.startswith("-"):
        minus.append(line[1:])
    elif line.startswith("+"):
        plus.append(line[1:])
    else:
        flush()
flush()
by_file = {}
for path, state, piece in results:
    by_file.setdefault(path, []).append((state, piece))
for path, items in by_file.items():
    absent = [piece for state, piece in items if state == "ABSENT"]
    print(f"file={path} fragments={len(items)} present={len(items) - len(absent)} absent={len(absent)}")
    for piece in absent:
        print(f"  ABSENT: {piece[:110]}")
