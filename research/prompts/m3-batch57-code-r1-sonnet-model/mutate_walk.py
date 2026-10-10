#!/usr/bin/env python3
# 用法：mutate_walk.py <原文件> <目标文件> <旧串> <新串>；旧串必须在原文件里恰好命中一次，替换后写到目标文件
import sys
src, dst, old, new = sys.argv[1:5]
old = old.replace('\\n', '\n'); new = new.replace('\\n', '\n')
text = open(src, encoding='utf-8').read()
n = text.count(old)
if n != 1:
    sys.exit(f'旧串命中 {n} 次，不是 1 次')
open(dst, 'w', encoding='utf-8').write(text.replace(old, new))
print('ok')
