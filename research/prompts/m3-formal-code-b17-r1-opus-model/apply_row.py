#!/usr/bin/env python3
"""用法：apply_row.py <rows.tsv> <行号 1 起> <仓副本根> apply|revert <冻结树根>"""
import sys, shutil, os
rows, n, root, mode, frozen = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4], sys.argv[5]
cols = open(rows, encoding='utf-8').read().split('\n')[n - 1].split('\t')
path = os.path.join(root, cols[1])
if mode == 'revert':
    shutil.copyfile(os.path.join(frozen, cols[1]), path)
    print('reverted', cols[1])
    sys.exit(0)
src = open(path, encoding='utf-8').read()
old = cols[2].replace('\\n', '\n')
new = cols[3].replace('\\n', '\n')
count = src.count(old)
print('from_count', count)
if count != 1:
    sys.exit(3)
open(path, 'w', encoding='utf-8').write(src.replace(old, new))
print('applied', cols[1])
