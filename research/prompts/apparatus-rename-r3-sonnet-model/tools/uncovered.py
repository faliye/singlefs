#!/usr/bin/env python3
"""J3：把 python3 -m trace --count --missing 给库自证出的 .cover 文件里没跑到的行（>>>>>> 开头）列出来，只列库本体（函数定义之后）的行号与原文。
用法：uncovered.py <lib-lag-baseline.cover>"""
import sys

lines = open(sys.argv[1], encoding="utf-8").read().split("\n")
first_code = next(i for i, text in enumerate(lines) if "import collections" in text[:30]) + 1
missed = [(i + 1, text[7:].rstrip()) for i, text in enumerate(lines) if text.startswith(">>>>>>") and i + 1 > first_code]
print(f"name=uncovered_lines count={len(missed)} first_code_line={first_code}")
for number, text in missed:
    print(f"{number}: {text.strip()[:110]}")
