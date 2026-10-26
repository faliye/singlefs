#!/usr/bin/env python3
"""把一条变异套到 --tree 的 crates/singlefs-core/src/<文件>：旧串必须恰好命中 --occurrence 指定那一处（默认要求恰好一处）。
用法：apply_mutation.py --tree <树> --orig-dir <原件目录> --file transaction.rs --old-file <旧串文件> --new-file <新串文件> [--occurrence N]
每次都先从原件目录拷回原件再改，所以套的永远是干净原件上的一条变异。"""
import argparse, shutil, sys
p = argparse.ArgumentParser()
p.add_argument('--tree', required=True); p.add_argument('--orig-dir', required=True)
p.add_argument('--file', required=True); p.add_argument('--old-file', required=True)
p.add_argument('--new-file', required=True); p.add_argument('--occurrence', type=int, default=0)
a = p.parse_args()
target = f'{a.tree}/crates/singlefs-core/src/{a.file}'
shutil.copyfile(f'{a.orig_dir}/{a.file}.orig', target)
text = open(target, encoding='utf-8').read()
old = open(a.old_file, encoding='utf-8').read().rstrip('\n')
new = open(a.new_file, encoding='utf-8').read().rstrip('\n')
count = text.count(old)
if a.occurrence == 0:
    if count != 1:
        sys.exit(f'旧串命中 {count} 处，要求恰好 1 处')
    text = text.replace(old, new, 1)
else:
    if count < a.occurrence:
        sys.exit(f'旧串只命中 {count} 处，要第 {a.occurrence} 处')
    pos = -1
    for _ in range(a.occurrence):
        pos = text.index(old, pos + 1)
    text = text[:pos] + new + text[pos + len(old):]
open(target, 'w', encoding='utf-8').write(text)
print(f'applied {a.file} occurrence={a.occurrence} old_hits={count}')
