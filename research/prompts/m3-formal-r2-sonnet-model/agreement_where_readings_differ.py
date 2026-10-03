#!/usr/bin/env python3
"""在 S 的读法两两不同的那些态上，实现等于哪一种读法。
用法：python3 agreement_where_readings_differ.py <n>
输出每一对读法 (x, y)：S_x != S_y 的态数，其中实现==S_x、实现==S_y、两者都不等的态数。
"""
import sys
from itertools import product
from multiprocessing import Pool
import reader_rule_model as m

PAIRS = [('whole', 'cont'), ('whole', 'stops'), ('cont', 'stops')]


def work(args):
    n, first = args
    out = {p: [0, 0, 0, 0] for p in PAIRS}   # 不同的态数、实现=x、实现=y、都不等
    for anchor in (None, m.ANCHOR_RECORD):
        for rest in product(m.SLOT_STATES, repeat=n - 1):
            table = m.build_table(anchor, (first,) + rest)
            r = m.impl(table)
            s = {k: m.spec(table, k) for k in ('whole', 'cont', 'stops')}
            for (x, y) in PAIRS:
                if s[x] != s[y]:
                    o = out[(x, y)]
                    o[0] += 1
                    if r == s[x]:
                        o[1] += 1
                    elif r == s[y]:
                        o[2] += 1
                    else:
                        o[3] += 1
    return out


if __name__ == '__main__':
    n = int(sys.argv[1])
    with Pool(4) as pool:
        parts = pool.map(work, [(n, f) for f in m.SLOT_STATES])
    for p in PAIRS:
        tot = [sum(part[p][i] for part in parts) for i in range(4)]
        print('n=%d readings=%s_vs_%s differ=%d impl_equals_%s=%d impl_equals_%s=%d impl_equals_neither=%d'
              % (n, p[0], p[1], tot[0], p[0], tot[1], p[1], tot[2], tot[3]))
