#!/usr/bin/env python3
"""在草稿副本里改一处（恰好命中一次才改），跑完用 restore 还原成冻结副本里的原样。
用法：probe.py apply <草稿副本根> <相对文件> <原文文件> <替换文文件>
      probe.py restore <草稿副本根> <冻结副本根> <相对文件>
原文/替换文放在文件里（\\n 以真换行写），避免命令行转义。只改草稿副本，不碰冻结副本与仓。"""
import sys, shutil, os
mode = sys.argv[1]
if mode == 'apply':
    root, rel, orig_f, repl_f = sys.argv[2:6]
    p = os.path.join(root, rel)
    s = open(p, encoding='utf-8').read()
    o = open(orig_f, encoding='utf-8').read().rstrip('\n')
    r = open(repl_f, encoding='utf-8').read().rstrip('\n')
    n = s.count(o)
    if n != 1:
        sys.exit(f'原文命中 {n} 次，不改')
    open(p, 'w', encoding='utf-8').write(s.replace(o, r))
    print('已改', rel)
elif mode == 'restore':
    root, frozen, rel = sys.argv[2:5]
    shutil.copyfile(os.path.join(frozen, rel), os.path.join(root, rel))
    print('已还原', rel)
