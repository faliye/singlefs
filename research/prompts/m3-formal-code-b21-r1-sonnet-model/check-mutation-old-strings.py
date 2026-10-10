#!/usr/bin/env python3
"""第 21 批 K1：冻结树 crates/mutations.tsv 里「第21批」七行，旧串（\\n 还原成换行）在目标文件里各命中几次。用法：python3 check-mutation-old-strings.py <冻结树根>"""
import sys
root = sys.argv[1]
rows = [l.rstrip('\n').split('\t') for l in open(root + '/crates/mutations.tsv') if l.startswith('第21批')]
print(len(rows))
for r in rows:
    name, file, old, new = r[0], r[1], r[2].replace('\\n', '\n'), r[3].replace('\\n', '\n')
    src = open(root + '/' + file).read()
    print(name[:60], 'old count', src.count(old), 'new!=old', new != old, 'cols', len(r))
