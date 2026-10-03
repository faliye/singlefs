#!/usr/bin/env python3
"""crates/mutations.tsv 第 448 行（把 recovery.rs:3115 那一判换成 `|| false`）在 A 域上与原实现逐态比。
译本 impl448 与 reader_rule_model.impl 只差一处：锚点读不出那一支不看序号。
用法：python3 mutant448_equivalence.py <n>
"""
import sys
from itertools import product
from multiprocessing import Pool
import reader_rule_model as m


def impl448(table, t=m.T):
    flagged = [r for r in table.values() if r[1] == t and r[3]]
    if len(flagged) > 1:
        return ('err',)
    anchor = flagged[0][0] if flagged else None
    above = sorted([r for r in table.values() if r[1] > t], key=lambda r: r[0])
    expected = anchor + 1 if anchor is not None else None
    rebuilt = m.CHOSEN
    open_pub = []
    for r in above[:m.LIMIT]:
        c, txg, od, last, commit, txn = r
        if expected is not None:
            if c != expected:
                break
        elif txg != t + 1:          # 只差这里：没有 `or od != 1`
            break
        if not open_pub and od != 1:
            break
        if open_pub:
            p = open_pub[-1]
            if txg != p[1]:
                break
            if od != p[2] + 1:
                break
            if not p[4] and txn != p[5]:
                break
        expected = c + 1
        if not commit and last:
            break
        if last and m.follows_impl(r, table):
            break
        open_pub.append(r)
        if not last:
            continue
        rebuilt = ('rec', c, txg)
        open_pub = []
    return ('ok', rebuilt)


def work(args):
    n, first = args
    diff = 0
    total = 0
    for anchor in (None, m.ANCHOR_RECORD):
        for rest in product(m.SLOT_STATES, repeat=n - 1):
            table = m.build_table(anchor, (first,) + rest)
            total += 1
            diff += m.impl(table) != impl448(table)
    return total, diff


if __name__ == '__main__':
    n = int(sys.argv[1])
    with Pool(4) as pool:
        parts = pool.map(work, [(n, f) for f in m.SLOT_STATES])
    print('n=%d states=%d differing_outcomes=%d' % (n, sum(p[0] for p in parts), sum(p[1] for p in parts)))
