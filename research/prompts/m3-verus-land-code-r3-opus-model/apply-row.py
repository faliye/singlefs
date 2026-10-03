#!/usr/bin/env python3
"""把变异表里的一行套到一棵树上：第 1 段名字，第 2 段从树根起的相对路径，第 3 段原文，第 4 段替换文（\\n 表示换行）。
原文在文件里必须恰好命中一次，否则退 3 并报命中次数。用法：apply-row.py <表> <行号（表里的物理行号）> <树根>"""
import sys
import pathlib

table_path, line_number_text, tree_root = sys.argv[1], sys.argv[2], sys.argv[3]
line_number = int(line_number_text)
lines = pathlib.Path(table_path).read_text(encoding="utf-8").split("\n")
columns = lines[line_number - 1].split("\t")
relative_path, original, replacement = columns[1], columns[2].replace("\\n", "\n"), columns[3].replace("\\n", "\n")
target = pathlib.Path(tree_root) / relative_path
text = target.read_text(encoding="utf-8")
hits = text.count(original)
if hits != 1:
    print(f"ANCHOR hits={hits} row={line_number} name={columns[0]}")
    sys.exit(3)
target.write_text(text.replace(original, replacement, 1), encoding="utf-8")
print(f"APPLIED row={line_number} name={columns[0]}")
