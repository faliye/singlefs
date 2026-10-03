# 独立再算：只用定义，不读 Rust 装置。一盘几何（G1 u=2、G3 u=1），P1 / P2 / P3free / P3ord。
import itertools, sys
from collections import defaultdict

def chain(tag, units):
    ev = []
    for u in range(1, units + 1):
        ev.append(('w', f'{tag.lower()}{u}'))
    ev.append(('b',))
    ev.append(('w', f'r{tag}'))
    ev.append(('b',))
    ev.append(('f', f'R{tag}'))
    ev.append(('w', f'S{tag}'))   # in place
    ev.append(('b',))
    return ev

def names_of(ev):
    return [e[1] for e in ev if e[0] != 'b']

import os
THIRD = os.environ.get('THIRD', '1') == '1'
INPLACE = lambda n: THIRD and n.startswith('S')

def segments(stream):
    segs, cur, rel_pending = [], [], False
    # 一盘：任何屏障或 FUA 放行
    pending = False
    for e in stream:
        if e[0] == 'w':
            cur.append(e[1]); pending = True
        elif e[0] == 'f':
            cur.append(e[1]); pending = False
            if cur: segs.append(cur); cur = []
        else:
            pending = False
            if cur: segs.append(cur); cur = []
    if cur: segs.append(cur)
    return segs

def dom4(stream, allnames):
    segs = segments(stream)
    out = set()
    zero = tuple(0 for _ in allnames)
    out.add(zero)
    pos = {n: i for i, n in enumerate(allnames)}
    prefix = [0] * len(allnames)
    for seg in segs:
        choices = [(0, 1, 2) if INPLACE(n) else (0, 1) for n in seg]
        for combo in itertools.product(*choices):
            v = list(prefix)
            for n, c in zip(seg, combo):
                v[pos[n]] = c
            out.add(tuple(v))
        for n in seg:
            prefix[pos[n]] = 1
    return out

def interleavings(a, b, ordered):
    res = []
    def rec(i, j, cur):
        if i == len(a) and j == len(b):
            res.append(list(cur)); return
        if i < len(a):
            cur.append(a[i]); rec(i + 1, j, cur); cur.pop()
        if j < len(b):
            ev = b[j]
            ok = True
            if ordered and ev[0] != 'b' and ev[1] in ('rB', 'RB', 'SB'):
                need = ev[1].replace('B', 'A')
                ok = any(e[0] != 'b' and e[1] == need for e in a[:i])
            if ok:
                cur.append(ev); rec(i, j + 1, cur); cur.pop()
    rec(0, 0, [])
    return res

def run(units):
    A, B = chain('A', units), chain('B', units)
    allnames = names_of(A) + names_of(B)
    idx = {n: i for i, n in enumerate(allnames)}
    ANAMES, BNAMES = names_of(A), names_of(B)
    def cross(v):
        return any(v[idx[n]] for n in ANAMES) and any(v[idx[n]] for n in BNAMES)
    def irec(v):
        return v[idx['rB']] == 1 and v[idx['rA']] != 1
    def ifull(v):
        return all(v[idx[n]] == 1 for n in BNAMES) and any(v[idx[n]] != 0 for n in ANAMES) and any(v[idx[n]] != 1 for n in ANAMES)
    print(f'--- units={units}')
    d_a, d_b = dom4(A, allnames), dom4(B, allnames)
    # K2 积
    k2 = set()
    for sa in d_a:
        for sb in d_b:
            k2.add(tuple(x | y for x, y in zip(sa, sb)))
    ser = dom4(A + B, allnames)
    print('chain', len(dom4(A, allnames)), 'K2', len(k2), 'P1 T', len(ser))
    # P2
    # a-units, b-units, barrier, rA rB, barrier, R, S, barrier
    p2names = [f'a{u}' for u in range(1, units + 1)] + [f'b{u}' for u in range(1, units + 1)] + ['rA', 'rB', 'R', 'S']
    p2stream = [('w', n) for n in p2names[:2 * units]] + [('b',), ('w', 'rA'), ('w', 'rB'), ('b',), ('f', 'R'), ('w', 'S'), ('b',)]
    p2 = dom4(p2stream, p2names)
    print('P2 T', len(p2))
    for label, ordered in (('P3free', False), ('P3ord', True)):
        sigmas = interleavings(A, B, ordered)
        T = set()
        per = []
        for s in sigmas:
            d = dom4(s, allnames)
            per.append((s, d))
            T |= d
        print(label, 'sigma', len(sigmas), 'T', len(T))
        sizes = [len(d) for _, d in per]
        print('  K1 size min', min(sizes), 'max', max(sizes), 'serial', len(ser))
        cross_T = {t for t in T if cross(t)}
        miss = [len(cross_T - d) for _, d in per]
        nonc = [len((T - cross_T) - d) for _, d in per]
        print('  m1 min', min(miss), 'max', max(miss), 'positive', sum(1 for m in miss if m > 0), 'noncross max', max(nonc), 'noncross positive', sum(1 for m in nonc if m > 0))
        print('  |T∩C|', len(cross_T), 'K2 \\ T', len(k2 - T), 'T\\K2 cross', len(cross_T - k2), 'T\\K2', len(T - k2))
        for nm, pred in (('rec', irec), ('full', ifull)):
            It = {t for t in T if pred(t)}
            cov = [len({t for t in It if t in d}) for _, d in per]
            print('  I_' + nm, '|I∩T|', len(It), 'K1 cover min', min(cov), 'max', max(cov), 'K2 cover', len(It & k2), 'K2 extra', len({t for t in k2 if pred(t)} - T))
    # P1 的 I
    for nm, pred in (('rec', irec), ('full', ifull)):
        print('P1 I_' + nm, len({t for t in ser if pred(t)}))
    print('P1 K2 vs T: K2 \\ T', len(k2 - ser), 'cross T', len({t for t in ser if cross(t)}))

run(2)
run(1)
