#!/usr/bin/env python3
# 用法：python3 -I static_anchor.py <仓副本根> <rows.txt> —— 变异表每一行原文在「文件」里命中次数不是 1 的，打「ANCHOR 行号 命中数 名字前 40 字」；另数 rows.txt 各前缀的行数、
# 列这一轮被删的两行还在不在；不编译、不跑 cargo。
import os, sys
repo, rows_file = sys.argv[1], sys.argv[2]
rows = [(i + 1, l.rstrip('\n').split('\t')) for i, l in enumerate(open(os.path.join(repo, 'crates/mutations.tsv'), encoding='utf-8')) if l.count('\t') == 5]
cache = {}
bad = 0
for ln, (name, path, old, new, args, tests) in rows:
    text = cache.setdefault(path, open(os.path.join(repo, path), encoding='utf-8').read() if os.path.exists(os.path.join(repo, path)) else None)
    hits = -1 if text is None else text.count(old.replace('\\n', '\n'))
    if hits != 1:
        bad += 1
        print(f'ANCHOR {ln} hits={hits} {name[:40]}')
print('ROWS', len(rows), 'ANCHOR-BAD', bad)
print('NAMES-UNIQUE', len({r[0] for _, r in rows}) == len(rows))
for prefix in [l.rstrip('\n') for l in open(rows_file, encoding='utf-8') if l.strip()]:
    print('PREFIX', prefix[:40], sum(1 for _, r in rows if r[0].startswith(prefix)))
for gone in ('删除线第35批 F2_EXACT 退回 F2_EXACT 之前的上限取法', '第39批 Z GPU 恢复内核 F2 对 c_见证 = 0 另设守卫'):
    print('DELETED-ROW-STILL-PRESENT', gone[:24], any(r[0].startswith(gone) for _, r in rows))
