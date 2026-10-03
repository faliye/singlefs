#!/usr/bin/env python3
"""apply-mutation.py <副本根> <表.tsv> <名字或 @行号>：把表里那一行（第 1 列名字、第 2 列文件、第 3 列原文、第 4 列替换文，\\n 表示换行）
套到副本上；原文在文件里必须恰好命中一次。原文件先存成 <文件>.opus-orig（已存在就不再存，几行叠着套时留的是最早那份）。
apply-mutation.py <副本根> --restore：把副本里所有 *.opus-orig 换回原名。只动副本，不碰仓。"""
import os
import sys

root = sys.argv[1]
if sys.argv[2] == '--restore':
    for directory, _, names in os.walk(os.path.join(root, 'crates')):
        for name in names:
            if name.endswith('.opus-orig'):
                path = os.path.join(directory, name)
                os.replace(path, path[: -len('.opus-orig')])
                # 换回的文件带着套变异那一刻的 mtime，比上一次编出来的产物旧：不 touch 的话 cargo 认它没变，下一次编译沿用变异过的那一份。
                os.utime(path[: -len('.opus-orig')])
                print('换回', os.path.relpath(path[: -len('.opus-orig')], root))
    sys.exit(0)
table, key = sys.argv[2], sys.argv[3]
for number, line in enumerate(open(table, encoding='utf-8'), start=1):
    columns = line.rstrip('\n').split('\t')
    if line.startswith('#') or len(columns) < 4:
        continue
    if key != columns[0] and key != f'@{number}':
        continue
    path = os.path.join(root, columns[1])
    old = columns[2].replace('\\n', '\n')
    new = columns[3].replace('\\n', '\n')
    text = open(path, encoding='utf-8').read()
    hits = text.count(old)
    if hits != 1:
        sys.exit(f'{key}：原文在 {columns[1]} 里命中 {hits} 次')
    if not os.path.exists(path + '.opus-orig'):
        open(path + '.opus-orig', 'w', encoding='utf-8').write(text)
    open(path, 'w', encoding='utf-8').write(text.replace(old, new, 1))
    print(f'套上 {key}（{columns[0]}）→ {columns[1]}')
    sys.exit(0)
sys.exit(f'表里没有 {key}')
