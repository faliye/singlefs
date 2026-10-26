#!/usr/bin/env python3
"""z5_union.py <闭包目录> <冻结副本>：发布、回退两条闭包的并集，按 crate 前缀分；行数用单元起止行数、再用去掉空行与 // 注释行的行数。"""
import collections, sys
work, FZ = sys.argv[1], sys.argv[2]
def fns(n):
    rows = {}
    for l in open(f'{work}/closure-{n}.txt', encoding='utf-8'):
        k, full, place, fp = l.rstrip('\n').split('\t')
        if k == 'fn': rows[(full, place)] = fp
    return rows
a, b = fns('publish'), fns('roll_back_by_a_forward_publish'); u = dict(a); u.update(b)
print('name=z5_union publish_fn=%d rollback_fn=%d union_fn=%d shared=%d only_publish=%d only_rollback=%d' % (len(a), len(b), len(u), len(set(a) & set(b)), len(set(a) - set(b)), len(set(b) - set(a))))
crate = collections.Counter(f.split('::')[0] for f, p in u)
print('name=z5_union_by_crate ' + ' '.join(f'{k}={v}' for k, v in sorted(crate.items())))
def span(p):
    s, e = p.rsplit(':', 1)[1].split('-'); return int(e) - int(s) + 1
cache = {}; code = 0
for f, p in u:
    path, sp = p.rsplit(':', 1); s, e = map(int, sp.split('-'))
    if path not in cache: cache[path] = open(f'{FZ}/{path}', encoding='utf-8').read().split('\n')
    code += sum(1 for l in cache[path][s - 1:e] if l.strip() and not l.strip().startswith('//'))
print('name=z5_union_lines span_lines=%d code_lines=%d core_span_lines=%d' % (sum(span(p) for f, p in u), code, sum(span(p) for f, p in u if f.startswith('singlefs_core::'))))
for f, p in sorted(u):
    if not f.startswith('singlefs_core::'): print('name=z5_union_noncore fn=%s at=%s' % (f, p))
