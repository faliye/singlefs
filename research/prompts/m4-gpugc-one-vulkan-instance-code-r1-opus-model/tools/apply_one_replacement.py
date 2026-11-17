#!/usr/bin/env python3
"""把副本里一份文件的一处原文换成替换文：原文要恰好命中一次。用法：apply_one_replacement.py <文件> <原文文件> <替换文文件>。"""
import pathlib
import sys

target, original_file, replacement_file = sys.argv[1:4]
path = pathlib.Path(target)
text = path.read_text(encoding="utf-8")
original = pathlib.Path(original_file).read_text(encoding="utf-8").rstrip("\n")
replacement = pathlib.Path(replacement_file).read_text(encoding="utf-8").rstrip("\n")
hits = text.count(original)
if hits != 1:
    sys.exit(f"原文命中 {hits} 次，要恰好 1 次：{original_file}")
path.write_text(text.replace(original, replacement), encoding="utf-8")
