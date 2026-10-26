#!/usr/bin/env python3
"""z3_writers.py <闭包目录>：core 里直接调 BlockDevice 写类原语（write_at / write_zeroes_at / discard_at）的非测试 fn，在不在发布 ∪ 回退（U2）、再加恢复（U3）里。
这是「会改盘面字节的 fn」的机械清单；不在 U3 里的就是三个闭包没罩到的写点。冻结副本的源码路径从环境变量 FROZEN 取。"""
import collections, os, re, sys
work = sys.argv[1]; FZ = os.environ['FROZEN']
def fns(n):
    rows = {}
    for l in open(f'{work}/closure-{n}.txt', encoding='utf-8'):
        k, full, place, fp = l.rstrip('\n').split('\t')
        if k == 'fn': rows[full] = place
    return rows
U2 = set(fns('publish')) | set(fns('roll_back_by_a_forward_publish')); U3 = U2 | set(fns('recover'))
pat = re.compile(r'\.(write_at|write_zeroes_at|discard_at)\s*\(|\b(write_at|write_zeroes_at|discard_at)\s*\(')
cache = {}; rows = []; test_from = {}
for l in open(f'{work}/dump-frozen.txt', encoding='utf-8'):
    p = l.rstrip('\n').split('\t')
    if p[0] != 'fn' or not p[1].startswith('singlefs_core::'): continue
    f, sp = p[2].rsplit(':', 1); a, b = map(int, sp.split('-'))
    if '/tests/' in f or '::tests::' in p[1]: continue
    if f not in cache:
        cache[f] = open(f'{FZ}/{f}', encoding='utf-8').read().split('\n')
        test_from[f] = next((i + 1 for i, x in enumerate(cache[f]) if x.startswith('#[cfg(test)]')), 10**9)
    if a >= test_from[f]: continue   # 文件里第一个顶层 #[cfg(test)] 之后的都是测试模块
    code = [x for x in cache[f][a - 1:b] if not x.strip().startswith('//')]
    prims = sorted({m.group(1) or m.group(2) for x in code for m in pat.finditer(x)})
    if prims:
        rows.append((p[1].replace('singlefs_core::', ''), f.split('/')[-1], a, b, prims))
c = collections.Counter()
for full, f, a, b, prims in rows:
    tag = 'U2' if 'singlefs_core::' + full in U2 else ('U3_only' if 'singlefs_core::' + full in U3 else 'outside')
    c[tag] += 1
    print('name=z3_writer where=%s fn=%s file=%s:%d-%d prims=%s' % (tag, full, f, a, b, '+'.join(prims)))
print('name=z3_writer_total fn=%d U2=%d U3_only=%d outside=%d' % (len(rows), c['U2'], c['U3_only'], c['outside']))
