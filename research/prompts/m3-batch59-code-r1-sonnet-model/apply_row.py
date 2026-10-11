#!/usr/bin/env python3
"""apply_row.py <repo> <mutations.tsv> <row> | apply_row.py --custom <repo> <relpath> <orig-file> <new-file> | restore
把变异表第 <row> 行的替换打到 <repo> 里（只在自己的拷贝上）；原文必须恰好命中一次。"""
import sys
if sys.argv[1] == '--custom':
    repo, rel, orig_f, new_f = sys.argv[2:6]
    orig = open(orig_f, encoding='utf-8').read(); new = open(new_f, encoding='utf-8').read()
    path = repo + '/' + rel
else:
    repo, tsv, row = sys.argv[1], sys.argv[2], int(sys.argv[3])
    cols = open(tsv, encoding='utf-8').read().split('\n')[row-1].split('\t')
    rel = cols[1]; orig = cols[2].replace('\\n', '\n'); new = cols[3].replace('\\n', '\n')
    path = repo + '/' + rel
txt = open(path, encoding='utf-8').read()
assert txt.count(orig) == 1, ('命中数', txt.count(orig))
open(path, 'w', encoding='utf-8').write(txt.replace(orig, new))
print('applied', path)
