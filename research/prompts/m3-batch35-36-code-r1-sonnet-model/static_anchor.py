#!/usr/bin/env python3
# 用法：python3 -I static_anchor.py <仓副本根> —— 变异表每一行原文在「文件」里命中次数不是 1 的，打「ANCHOR 行号 命中数 名字前 40 字」；另数这一轮新增行（名字前缀）与已删的两行还在不在。不编译、不跑 cargo。
import os, sys
repo = sys.argv[1]
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
for prefix in ('删除线第35批', '第36批 V', '第33批T'):
    print('PREFIX', prefix, sum(1 for _, r in rows if r[0].startswith(prefix)))
for gone in ('第33批T F2 乙两条都读不出就退', '第33批T F2 乙施加过记录、由记录重建的那一版走读失败也退'):
    print('DELETED-ROW-STILL-PRESENT', gone[:20], any(r[0].startswith(gone) for _, r in rows))
