#!/usr/bin/env python3
"""apply-mutation.py <副本根> <spec.tsv> <名字>：把 spec 里那一行套到副本上（旧串恰好命中一次），原文件存成 <文件>.m3r1-orig；
apply-mutation.py <副本根> --restore <相对路径>：换回原文件。"""
import sys, os
root = sys.argv[1]
if sys.argv[2] == '--restore':
    path = os.path.join(root, sys.argv[3])
    os.replace(path + '.m3r1-orig', path)
    os.utime(path, None)  # 换回的文件 mtime 取此刻，cargo 才会重编（第四批的教训：旧 mtime 让变异编出的产物留在编译目录里）
    sys.exit(0)
spec, name = sys.argv[2], sys.argv[3]
for line in open(spec, encoding='utf-8'):
    cols = line.rstrip('\n').split('\t')
    if cols[0] != name:
        continue
    path = os.path.join(root, cols[1])
    old, new = cols[2].replace('\\n', '\n'), cols[3].replace('\\n', '\n')
    text = open(path, encoding='utf-8').read()
    assert text.count(old) == 1, f'{name} 旧串命中 {text.count(old)} 次'
    open(path + '.m3r1-orig', 'w', encoding='utf-8').write(text)
    open(path, 'w', encoding='utf-8').write(text.replace(old, new, 1))
    print(cols[1])
    sys.exit(0)
sys.exit(f'没有 {name}')
