#!/usr/bin/env python3
"""在仓副本里施加一条变异：原文在文件里必须恰好命中一次（\\n 表示换行）。用法：apply-mutation.py 副本根 文件 原文 替换文"""
import sys
root, relative, original, replacement = sys.argv[1:5]
original = original.replace("\\n", "\n")
replacement = replacement.replace("\\n", "\n")
path = f"{root}/{relative}"
text = open(path, encoding="utf-8").read()
hits = text.count(original)
if hits != 1:
    sys.exit(f"{relative}: 原文命中 {hits} 次，不是恰好一次")
open(path, "w", encoding="utf-8").write(text.replace(original, replacement))
print(f"施加：{relative}")
